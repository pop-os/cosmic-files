// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::operation::{Operation, OperationSelection};
use cosmic::widget::toaster::ToastId;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

/// Maximum capacity of the undo/redo stacks (ring buffer).
const UNDO_STACK_CAPACITY: usize = 20;

/// An entry in the undo/redo stack.
///
/// The `entry_id` is a unique, monotonic identifier used by `Message::AssignToastId`
/// to locate the entry (NOT a stack index, which shifts on eviction).
#[derive(Clone, Debug)]
pub struct UndoEntry {
    /// Unique, monotonic id assigned by `UndoHistory::record_undo`; used by
    /// `Message::AssignToastId` to locate the entry and by the evict/clear
    /// paths to dismiss its live toast.
    pub entry_id: u64,
    pub forward: Operation,
    pub inverse: Operation,
    #[allow(dead_code)]
    // Todo 5/6 (Edit menu + context menu) render this as the dynamic label;
    // Todo 8 replaces the literal strings with i18n keys.
    pub description: String,
    pub destructive: bool, // true when inverse moves files to Trash (undo-copy / undo-create)
    /// Set asynchronously via `Message::AssignToastId`; dismisses stale toasts
    /// on evict/clear.
    pub toast_id: Option<ToastId>,
}

/// A bounded ring-buffer stack for undo/redo entries.
///
/// This struct encapsulates the stack logic so it can be tested independently
/// of the full `App` state machine.
#[derive(Clone, Debug, Default)]
pub struct UndoStack {
    entries: VecDeque<UndoEntry>,
}

#[allow(dead_code)] // peek/is_empty/len (Todo 5 menu), iter (tests)
impl UndoStack {
    /// Creates a new empty undo stack.
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(UNDO_STACK_CAPACITY),
        }
    }

    /// Pushes an entry onto the stack, evicting the oldest if at capacity.
    ///
    /// Returns the `ToastId` of the evicted entry (if any); the caller must
    /// dismiss that toast via `self.toasts.remove(toast_id)` so a stale toast's
    /// Undo button can never reference an evicted entry.
    pub fn push(&mut self, entry: UndoEntry) -> Option<ToastId> {
        let evicted_toast = if self.entries.len() == UNDO_STACK_CAPACITY {
            self.entries.pop_front().and_then(|e| e.toast_id)
        } else {
            None
        };
        self.entries.push_back(entry);
        evicted_toast
    }

    /// Pops the most recent entry from the stack.
    pub fn pop(&mut self) -> Option<UndoEntry> {
        self.entries.pop_back()
    }

    /// Returns the most recent entry without removing it.
    pub fn peek(&self) -> Option<&UndoEntry> {
        self.entries.back()
    }

    /// Returns true if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of entries in the stack.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Clears all entries, returning their toast IDs for dismissal.
    pub fn clear(&mut self) -> Vec<ToastId> {
        self.entries.drain(..).filter_map(|e| e.toast_id).collect()
    }

    /// Returns an iterator over the entries (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &UndoEntry> {
        self.entries.iter()
    }

    /// Returns a mutable iterator over the entries (used by
    /// `Message::AssignToastId` to write the async toast id back).
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut UndoEntry> {
        self.entries.iter_mut()
    }
}

/// Which stack an entry should be pushed back onto.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StackTarget {
    Undo,
    Redo,
}

/// Toast ids the caller (`App`) must dismiss for entries evicted/cleared by an
/// operation of the undo history. Kept as plain data so the history stays pure
/// and unit-testable.
pub type Dismiss = Vec<ToastId>;

/// Builds the undo entry that records a completed user operation.
///
/// Returns `None` for operations that are not undoable (permanent delete,
/// empty-trash, compress, extract, ...). The `forward` operation is the
/// operation that was just dispatched (its `paths`/`from`/`to` are the paths
/// captured at the mutation call site before the operation ran); the `inverse`
/// reverses it, using `op_sel.selected` (the ACTUAL destination paths) and
/// `op_sel.undo_items` (the captured trash items) where applicable.
///
/// Description strings are placeholders; Todo 8 replaces them with i18n keys.
pub fn build_undo_entry(op: &Operation, op_sel: &OperationSelection) -> Option<UndoEntry> {
    let (forward, inverse, description, destructive) = match op {
        Operation::Delete { paths } => {
            let items = op_sel
                .undo_items
                .clone()
                .filter(|items| !items.is_empty())?;
            (
                Operation::Delete {
                    paths: paths.clone(),
                },
                Operation::Restore { items },
                "Undo Delete".to_string(),
                false,
            )
        }
        Operation::Rename { from, to } => (
            Operation::Rename {
                from: from.clone(),
                to: to.clone(),
            },
            Operation::Rename {
                from: to.clone(),
                to: from.clone(),
            },
            "Undo Rename".to_string(),
            false,
        ),
        Operation::Move {
            paths,
            to,
            cross_device_copy,
        } => {
            // The source directory is the parent the moved items originally came
            // from (in practice all items of one move share a parent — cut+paste
            // and DnD both originate from a single folder).
            let source_dir = paths
                .first()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| to.clone());
            (
                Operation::Move {
                    paths: paths.clone(),
                    to: to.clone(),
                    cross_device_copy: *cross_device_copy,
                },
                Operation::Move {
                    paths: op_sel.selected.clone(),
                    to: source_dir,
                    cross_device_copy: *cross_device_copy,
                },
                "Undo Move".to_string(),
                false,
            )
        }
        Operation::Copy { paths, to } => (
            Operation::Copy {
                paths: paths.clone(),
                to: to.clone(),
            },
            // Undo-copy moves the copies to the Trash (data-preserving), never
            // a permanent delete.
            Operation::Delete {
                paths: op_sel.selected.clone(),
            },
            "Undo Copy".to_string(),
            true,
        ),
        Operation::NewFile { path } => (
            Operation::NewFile { path: path.clone() },
            Operation::Delete {
                paths: vec![path.clone()],
            },
            "Undo Create".to_string(),
            true,
        ),
        Operation::NewFolder { path } => (
            Operation::NewFolder { path: path.clone() },
            Operation::Delete {
                paths: vec![path.clone()],
            },
            "Undo Create".to_string(),
            true,
        ),
        _ => return None,
    };

    Some(UndoEntry {
        entry_id: 0, // assigned by UndoHistory::record_undo
        forward,
        inverse,
        description,
        destructive,
        toast_id: None,
    })
}

/// The routing state machine for the undo/redo system.
///
/// Owns the two bounded stacks, the held in-flight entry, the per-operation
/// suppression map, the deferred-invalidation flag, and the monotonic entry-id
/// counter. All logic here is pure (methods return [`Dismiss`] vectors of toast
/// ids for the caller to apply to `self.toasts`), so it can be tested
/// independently of the full `App`.
///
/// Review-mandated invariants enforced here:
/// - suppression is per-operation (`suppressed_ops` keyed by pending-op id),
///   never a global boolean;
/// - the in-flight entry is held in `pending_undo`, OUTSIDE both stacks, so it
///   cannot be evicted by the 20-cap ring buffer;
/// - success routing pushes the entry RETURNED by `suppressed_ops.remove(&id)`
///   (never `pending_undo.take()`, which is None for destructive undo);
/// - `record_undo_for_retry` pushes back with cap-20 eviction but does NOT
///   clear the other stack;
/// - `pending_invalidation` is keyed to the in-flight inverse's specific id so
///   a concurrent normal user operation is never swallowed and never triggers
///   the clear.
#[derive(Clone, Debug, Default)]
pub struct UndoHistory {
    pub undo_stack: UndoStack,
    pub redo_stack: UndoStack,
    pub pending_undo: Option<UndoEntry>,
    /// pending-operation id -> the entry whose execution is suppressed (the
    /// inverse for an undo, the forward for a redo).
    pub suppressed_ops: FxHashMap<u64, UndoEntry>,
    /// The pending-operation id of the in-flight inverse that a permanent
    /// delete / empty-trash deferred (Todo 4 sets this); its completion
    /// performs the actual stack clear. NOT a plain bool, so a normal user op
    /// completing first is never misrouted.
    pub pending_invalidation: Option<u64>,
    /// Monotonic counter assigned to `UndoEntry::entry_id` by `record_undo`.
    pub next_entry_id: u64,
    /// Direction of the single in-flight suppressed op: `Some(true)` = an
    /// undo's inverse was dispatched, `Some(false)` = a redo's forward was
    /// dispatched. At most one suppressed op can be in flight (single-in-flight
    /// invariant), so a single slot is safe.
    pub pending_undo_direction: Option<bool>,
}

impl UndoHistory {
    /// Creates a new empty undo history.
    pub fn new() -> Self {
        Self {
            undo_stack: UndoStack::new(),
            redo_stack: UndoStack::new(),
            pending_undo: None,
            suppressed_ops: FxHashMap::default(),
            pending_invalidation: None,
            next_entry_id: 0,
            pending_undo_direction: None,
        }
    }

    /// Records a completed user operation as a new undo entry.
    ///
    /// Assigns a monotonic `entry_id`, pushes onto `undo_stack` (evicting the
    /// oldest at capacity 20), and clears `redo_stack` (a new operation makes
    /// redo unavailable, matching Windows). Returns the assigned `entry_id`
    /// (used by `Message::AssignToastId` to write the async toast id back) and
    /// the toast ids to dismiss for evicted/cleared entries.
    pub fn record_undo(&mut self, entry: UndoEntry) -> (u64, Dismiss) {
        let entry_id = self.next_entry_id;
        self.next_entry_id += 1;
        let mut entry = entry;
        entry.entry_id = entry_id;
        let mut dismiss = self.redo_stack.clear();
        if let Some(toast_id) = self.undo_stack.push(entry) {
            dismiss.push(toast_id);
        }
        (entry_id, dismiss)
    }

    /// Pushes an entry onto `target` with cap-20 eviction.
    fn push_to(&mut self, target: StackTarget, entry: UndoEntry) -> Dismiss {
        match target {
            StackTarget::Undo => self.undo_stack.push(entry).into_iter().collect(),
            StackTarget::Redo => self.redo_stack.push(entry).into_iter().collect(),
        }
    }

    /// Pushes a held entry back onto the target stack with cap-20 eviction,
    /// preserving the other stack untouched (a failed undo keeps the redo
    /// entries; a failed redo keeps the undo entries).
    pub fn record_undo_for_retry(&mut self, entry: UndoEntry, target: StackTarget) -> Dismiss {
        self.push_to(target, entry)
    }

    /// Routes a completed suppressed (inverse/redo) operation's SUCCESS.
    ///
    /// Removes the held entry from `suppressed_ops` and pushes it onto the
    /// OPPOSITE stack: an undo's inverse succeeded -> the entry becomes
    /// redoable (`redo_stack`); a redo's forward succeeded -> the entry becomes
    /// undoable (`undo_stack`). Uses the entry RETURNED by the map (never
    /// `pending_undo`, which is None for destructive undo once Todo 7 takes it
    /// at dialog-confirm time). Returns the toast ids to dismiss (evicted
    /// entries).
    pub fn route_suppressed_success(&mut self, id: u64) -> Option<Dismiss> {
        let entry = self.suppressed_ops.remove(&id)?;
        self.pending_undo = None;
        let target = match self.pending_undo_direction.take() {
            // An undo succeeded -> the entry is now redoable.
            Some(true) => StackTarget::Redo,
            // A redo succeeded -> the entry is now undoable.
            Some(false) => StackTarget::Undo,
            // Unreachable (direction is always set when a suppressed op is
            // dispatched); default to the common undo-success case.
            None => StackTarget::Redo,
        };
        Some(self.push_to(target, entry))
    }

    /// Routes a completed suppressed (inverse/redo) operation's FAILURE.
    ///
    /// Returns the held entry to its ORIGINAL stack so the user can retry:
    /// a failed undo's inverse -> entry back to `undo_stack`
    /// (`record_undo_for_retry(entry, StackTarget::Undo)`); a failed redo's
    /// forward -> entry back to `redo_stack` (`StackTarget::Redo`). Does NOT
    /// clear the other stack. Returns the toast ids to dismiss (evicted).
    pub fn route_suppressed_failure(&mut self, id: u64) -> Option<Dismiss> {
        let entry = self.suppressed_ops.remove(&id)?;
        self.pending_undo = None;
        let target = match self.pending_undo_direction.take() {
            // Failed undo -> entry returns to the undo stack.
            Some(true) => StackTarget::Undo,
            // Failed redo -> entry returns to the redo stack.
            Some(false) => StackTarget::Redo,
            // Unreachable; default to the common failed-undo case.
            None => StackTarget::Undo,
        };
        Some(self.record_undo_for_retry(entry, target))
    }

    /// Returns the id of the single in-flight inverse/redo (the sole key of
    /// `suppressed_ops`) when one is actually dispatched, `None` otherwise.
    ///
    /// The single-in-flight invariant guarantees `suppressed_ops` has at most
    /// one entry, so "the sole key" is unambiguous. Used by
    /// `App::invalidate_undo_history` (Todo 4) to decide whether a permanent
    /// delete / empty-trash clear must be DEFERRED until that inverse completes.
    pub fn inverse_in_flight(&self) -> Option<u64> {
        self.suppressed_ops.keys().next().copied()
    }

    /// Clears EVERYTHING: both stacks, the held entry, the suppression map, the
    /// deferred-invalidation flag, and the direction slot. Returns the combined
    /// toast ids to dismiss.
    ///
    /// This is the immediate-clear path of a permanent delete / empty-trash
    /// invalidation (Todo 4) when NO inverse is in flight (idle, or a Todo-7
    /// `DialogPage::ConfirmUndo` holds the entry in `pending_undo`). It is also
    /// what the deferred hook performs once the in-flight inverse completes.
    pub fn clear_all(&mut self) -> Dismiss {
        let mut dismiss = self.undo_stack.clear();
        dismiss.extend(self.redo_stack.clear());
        self.pending_undo = None;
        self.suppressed_ops.clear();
        self.pending_invalidation = None;
        self.pending_undo_direction = None;
        dismiss
    }

    /// The deferred-invalidation hook, called with the id of a completing
    /// operation. When it equals the id of the in-flight inverse that a
    /// permanent delete / empty-trash deferred (Todo 4), clears BOTH stacks,
    /// the held entry, and the suppression map, sets the flag back to `None`,
    /// and returns the toast ids to dismiss. A NORMAL user operation completing
    /// while `pending_invalidation` is set does NOT match and is routed (and
    /// recorded) normally — this is the fix for the race a plain bool would
    /// introduce.
    pub fn deferred_invalidation(&mut self, id: u64) -> Option<Dismiss> {
        if self.pending_invalidation == Some(id) {
            Some(self.clear_all())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fl;
    use crate::operation::Operation;
    use std::path::PathBuf;
    use trash::TrashItem;

    fn make_entry(id: u64, desc: &str) -> UndoEntry {
        UndoEntry {
            entry_id: id,
            forward: Operation::NewFile {
                path: PathBuf::from("/tmp/test"),
            },
            inverse: Operation::Delete {
                paths: vec![PathBuf::from("/tmp/test")],
            },
            description: desc.to_string(),
            destructive: false,
            toast_id: None,
        }
    }

    fn trash_item(original: &str, id: &str) -> TrashItem {
        let original = PathBuf::from(original);
        TrashItem {
            id: id.into(),
            name: original.file_name().unwrap().to_os_string(),
            original_parent: original.parent().unwrap().to_path_buf(),
            time_deleted: 0,
        }
    }

    #[test]
    fn undo_entries_stack() {
        let mut stack = UndoStack::new();

        // Push up to 20 entries
        for i in 0..20 {
            stack.push(make_entry(i, &format!("Entry {}", i)));
        }
        assert_eq!(stack.len(), 20);

        // The 21st evicts the oldest (front) and keeps the new entry
        let evicted_toast = stack.push(make_entry(20, "Entry 20"));
        assert_eq!(stack.len(), 20);
        assert!(evicted_toast.is_none()); // no toast_id set
        assert_eq!(stack.peek().unwrap().entry_id, 20);
        // The oldest (entry_id 0) should be gone
        assert!(!stack.iter().any(|e| e.entry_id == 0));
        assert!(stack.iter().any(|e| e.entry_id == 20));

        // Empty stack pop is None
        let mut empty_stack = UndoStack::new();
        assert!(empty_stack.pop().is_none());
    }

    #[test]
    fn undo_stack_clear_returns_toast_ids() {
        let mut stack = UndoStack::new();
        let mut entry = make_entry(1, "Test");
        entry.toast_id = Some(ToastId::from(slotmap::KeyData::from_ffi(1)));
        let toast_id = entry.toast_id.unwrap();
        stack.push(entry);

        let toast_ids = stack.clear();
        assert_eq!(toast_ids.len(), 1);
        assert_eq!(toast_ids[0], toast_id);
        assert!(stack.is_empty());
    }

    #[test]
    fn undo_record_delete() {
        let mut history = UndoHistory::new();
        let path = PathBuf::from("/home/user/docs/foo.txt");
        let item = trash_item(
            "/home/user/docs/foo.txt",
            "/home/user/.local/share/Trash/files/foo.txt.trashinfo",
        );

        let op = Operation::Delete {
            paths: vec![path.clone()],
        };
        let op_sel = OperationSelection {
            undo_items: Some(vec![item.clone()]),
            ..Default::default()
        };
        let entry = build_undo_entry(&op, &op_sel).expect("Delete should build an entry");
        let (entry_id, _) = history.record_undo(entry);

        // Stack top is UndoEntry{forward: Delete, inverse: Restore} with the
        // captured TrashItems; redo is empty.
        assert_eq!(history.undo_stack.len(), 1);
        assert!(history.redo_stack.is_empty());
        let top = history.undo_stack.peek().unwrap();
        assert_eq!(top.entry_id, entry_id);
        assert!(matches!(top.forward, Operation::Delete { .. }));
        assert!(matches!(&top.inverse, Operation::Restore { items } if items == &vec![item]));
        assert!(!top.destructive);

        // A Delete without captured items is not recorded.
        let op = Operation::Delete {
            paths: vec![PathBuf::from("/home/user/docs/bar.txt")],
        };
        assert!(build_undo_entry(&op, &OperationSelection::default()).is_none());
    }

    #[test]
    fn undo_record_rename_move_copy_create() {
        let mut history = UndoHistory::new();

        // Rename: inverse swaps from/to; not destructive.
        let op = Operation::Rename {
            from: PathBuf::from("/a/x.txt"),
            to: PathBuf::from("/a/y.txt"),
        };
        let entry = build_undo_entry(&op, &OperationSelection::default()).unwrap();
        assert!(matches!(
            &entry.inverse,
            Operation::Rename { from, to }
                if from == &PathBuf::from("/a/y.txt") && to == &PathBuf::from("/a/x.txt")
        ));
        assert!(!entry.destructive);

        // A new operation clears the redo stack.
        history.redo_stack.push(make_entry(999, "stale redo"));
        let (_, _) = history.record_undo(entry);
        assert!(history.redo_stack.is_empty());

        // Move: inverse moves the ACTUAL destination paths back to the source dir.
        let op = Operation::Move {
            paths: vec![PathBuf::from("/a/x.txt")],
            to: PathBuf::from("/b"),
            cross_device_copy: false,
        };
        let op_sel = OperationSelection {
            selected: vec![PathBuf::from("/b/x.txt")],
            ..Default::default()
        };
        let entry = build_undo_entry(&op, &op_sel).unwrap();
        assert!(matches!(
            &entry.inverse,
            Operation::Move { paths, to, cross_device_copy: false }
                if paths == &op_sel.selected && to == &PathBuf::from("/a")
        ));
        assert!(!entry.destructive);
        let (_, _) = history.record_undo(entry);

        // Copy: destructive, inverse is Delete of the actual copies (NOT PermanentlyDelete).
        let op = Operation::Copy {
            paths: vec![PathBuf::from("/a/x.txt")],
            to: PathBuf::from("/b"),
        };
        let op_sel = OperationSelection {
            selected: vec![PathBuf::from("/b/x.txt")],
            ..Default::default()
        };
        let entry = build_undo_entry(&op, &op_sel).unwrap();
        assert!(entry.destructive);
        assert!(matches!(&entry.inverse, Operation::Delete { paths } if paths == &op_sel.selected));
        let (_, _) = history.record_undo(entry);

        // NewFile / NewFolder: destructive, inverse is Delete of the created path.
        let op = Operation::NewFile {
            path: PathBuf::from("/a/new.txt"),
        };
        let entry = build_undo_entry(&op, &OperationSelection::default()).unwrap();
        assert!(entry.destructive);
        assert!(matches!(
            &entry.inverse,
            Operation::Delete { paths } if paths == &vec![PathBuf::from("/a/new.txt")]
        ));
        let (_, _) = history.record_undo(entry);

        let op = Operation::NewFolder {
            path: PathBuf::from("/a/newdir"),
        };
        let entry = build_undo_entry(&op, &OperationSelection::default()).unwrap();
        assert!(entry.destructive);
        assert!(matches!(
            &entry.inverse,
            Operation::Delete { paths } if paths == &vec![PathBuf::from("/a/newdir")]
        ));
        let (_, _) = history.record_undo(entry);

        // Non-undoable operations are not recorded.
        assert!(
            build_undo_entry(
                &Operation::PermanentlyDelete {
                    paths: vec![PathBuf::from("/a/x.txt")].into_boxed_slice(),
                },
                &OperationSelection::default()
            )
            .is_none()
        );
        assert!(build_undo_entry(&Operation::EmptyTrash, &OperationSelection::default()).is_none());
    }

    #[test]
    fn undo_inverse_move_uses_actual_paths() {
        // Move inverse paths equal op_sel.selected (the actual destination paths).
        let op = Operation::Move {
            paths: vec![PathBuf::from("/src/file.txt")],
            to: PathBuf::from("/dst"),
            cross_device_copy: true,
        };
        let op_sel = OperationSelection {
            selected: vec![PathBuf::from("/dst/file (copy).txt")],
            ..Default::default()
        };
        let entry = build_undo_entry(&op, &op_sel).unwrap();
        match &entry.inverse {
            Operation::Move {
                paths,
                to,
                cross_device_copy,
            } => {
                assert_eq!(paths, &op_sel.selected);
                assert_eq!(to, &PathBuf::from("/src"));
                assert!(cross_device_copy);
            }
            other => panic!("expected Move inverse, got {other:?}"),
        }

        // Copy inverse Delete paths equal op_sel.selected.
        let op = Operation::Copy {
            paths: vec![PathBuf::from("/src/file.txt")],
            to: PathBuf::from("/dst"),
        };
        let op_sel = OperationSelection {
            selected: vec![PathBuf::from("/dst/file (copy).txt")],
            ..Default::default()
        };
        let entry = build_undo_entry(&op, &op_sel).unwrap();
        match &entry.inverse {
            Operation::Delete { paths } => assert_eq!(paths, &op_sel.selected),
            other => panic!("expected Delete inverse, got {other:?}"),
        }
    }

    // CRITICAL race test: a user operation completing while an inverse is
    // in-flight IS recorded (not swallowed by the suppression), the inverse is
    // NOT recorded, and on inverse success the entry returned by
    // `suppressed_ops.remove(&id)` is pushed to `redo_stack`.
    #[test]
    fn undo_suppression_routes() {
        let mut history = UndoHistory::new();

        // Simulate an undo dispatched for the top undo entry: it is popped into
        // `pending_undo`, inserted into `suppressed_ops` keyed by the inverse's
        // pending-operation id, and the direction is recorded as an undo.
        let held_entry = make_entry(0, "original delete");
        history.undo_stack.push(held_entry.clone());
        let entry = history.undo_stack.pop().unwrap();
        history.pending_undo = Some(entry.clone());
        let inverse_id = 100;
        history.suppressed_ops.insert(inverse_id, entry);
        history.pending_undo_direction = Some(true);

        // A user operation completes while the inverse is in-flight: it is NOT
        // in `suppressed_ops`, so it records normally.
        let user_op = Operation::Rename {
            from: PathBuf::from("/a/x.txt"),
            to: PathBuf::from("/a/y.txt"),
        };
        let user_sel = OperationSelection {
            selected: vec![PathBuf::from("/a/y.txt")],
            ..Default::default()
        };
        let user_entry = build_undo_entry(&user_op, &user_sel).unwrap();
        let _ = history.record_undo(user_entry);
        assert_eq!(history.undo_stack.len(), 1, "user op must be recorded");

        // The inverse completes successfully: its entry is routed to the redo
        // stack (NOT recorded on the undo stack).
        let dismiss = history.route_suppressed_success(inverse_id);
        assert!(dismiss.is_some());
        assert_eq!(
            history.undo_stack.len(),
            1,
            "inverse must NOT be recorded on the undo stack"
        );
        assert_eq!(history.redo_stack.len(), 1);
        assert_eq!(
            history.redo_stack.peek().unwrap().entry_id,
            held_entry.entry_id
        );
        assert!(history.pending_undo.is_none());
        assert!(history.suppressed_ops.is_empty());
    }

    // A failed UNDO inverse returns its entry to `undo_stack` via
    // `record_undo_for_retry(entry, StackTarget::Undo)`; redo untouched.
    #[test]
    fn undo_suppression_failure_routes() {
        let mut history = UndoHistory::new();
        let entry = make_entry(0, "undo me");
        history.undo_stack.push(entry.clone());
        let entry = history.undo_stack.pop().unwrap();
        history.pending_undo = Some(entry.clone());
        let inverse_id = 100;
        history.suppressed_ops.insert(inverse_id, entry);
        history.pending_undo_direction = Some(true); // an undo is in flight

        let dismiss = history.route_suppressed_failure(inverse_id);
        assert!(dismiss.is_some());
        assert_eq!(
            history.undo_stack.len(),
            1,
            "failed undo entry returns to undo_stack"
        );
        assert_eq!(history.undo_stack.peek().unwrap().entry_id, 0);
        assert!(history.redo_stack.is_empty(), "redo_stack untouched");
        assert!(history.pending_undo.is_none());
        assert!(history.suppressed_ops.is_empty());
    }

    // A failed REDO inverse returns its entry to `redo_stack` via
    // `record_undo_for_retry(entry, StackTarget::Redo)`; undo untouched.
    #[test]
    fn undo_redo_failure_routes() {
        let mut history = UndoHistory::new();
        let entry = make_entry(1, "redo me");
        history.redo_stack.push(entry.clone());
        let entry = history.redo_stack.pop().unwrap();
        history.pending_undo = Some(entry.clone());
        let forward_id = 200;
        history.suppressed_ops.insert(forward_id, entry);
        history.pending_undo_direction = Some(false); // a redo is in flight

        let dismiss = history.route_suppressed_failure(forward_id);
        assert!(dismiss.is_some());
        assert!(history.undo_stack.is_empty(), "undo_stack untouched");
        assert_eq!(
            history.redo_stack.len(),
            1,
            "failed redo entry returns to redo_stack"
        );
        assert_eq!(history.redo_stack.peek().unwrap().entry_id, 1);
        assert!(history.pending_undo.is_none());
        assert!(history.suppressed_ops.is_empty());
    }

    // Deferred invalidation: `pending_invalidation = Some(inverse_id)` set while
    // the inverse is in-flight -> the inverse's completion (success) clears all
    // stacks and records nothing; a NORMAL user op completing while it is set is
    // NOT swallowed and does NOT trigger the clear.
    #[test]
    fn undo_deferred_invalidation() {
        let mut history = UndoHistory::new();
        history.undo_stack.push(make_entry(0, "a"));
        history.redo_stack.push(make_entry(1, "b"));
        let held = make_entry(2, "held");
        history.pending_undo = Some(held.clone());
        let inverse_id = 300;
        history.suppressed_ops.insert(inverse_id, held);
        history.pending_invalidation = Some(inverse_id);

        // The inverse completes: everything is cleared, nothing recorded.
        let dismiss = history.deferred_invalidation(inverse_id);
        assert!(dismiss.is_some());
        assert!(history.undo_stack.is_empty());
        assert!(history.redo_stack.is_empty());
        assert!(history.pending_undo.is_none());
        assert!(history.suppressed_ops.is_empty());
        assert!(history.pending_invalidation.is_none());

        // A normal user op completing while `pending_invalidation` is set for a
        // DIFFERENT id is NOT swallowed (it records) and does NOT trigger the clear.
        let mut history = UndoHistory::new();
        history.pending_invalidation = Some(inverse_id);
        assert!(history.deferred_invalidation(inverse_id + 1).is_none());
        let user_op = Operation::NewFile {
            path: PathBuf::from("/tmp/new"),
        };
        let user_entry = build_undo_entry(&user_op, &OperationSelection::default()).unwrap();
        let _ = history.record_undo(user_entry);
        assert_eq!(history.undo_stack.len(), 1, "normal user op still records");
        assert_eq!(
            history.pending_invalidation,
            Some(inverse_id),
            "flag not cleared"
        );
    }

    // `Message::AssignToastId`-style lookup: finds the entry by `entry_id` and
    // writes the id; a stale id is ignored.
    #[test]
    fn undo_toast_id_assignment() {
        let mut history = UndoHistory::new();
        let (entry_id, _) = history.record_undo(make_entry(99, "t"));

        let toast_id = ToastId::from(slotmap::KeyData::from_ffi(42));
        if let Some(entry) = history
            .undo_stack
            .iter_mut()
            .find(|entry| entry.entry_id == entry_id)
        {
            entry.toast_id = Some(toast_id);
        }
        assert_eq!(history.undo_stack.peek().unwrap().toast_id, Some(toast_id));

        // A stale entry_id is ignored (no panic, no match).
        let stale = history
            .undo_stack
            .iter_mut()
            .find(|entry| entry.entry_id == 999);
        assert!(stale.is_none());

        // Eviction: pushing 20 more entries evicts the oldest, whose id is then stale.
        for i in 0..20 {
            let _ = history.record_undo(make_entry(i, "more"));
        }
        assert_eq!(history.undo_stack.len(), 20);
        let gone = history
            .undo_stack
            .iter_mut()
            .find(|entry| entry.entry_id == entry_id);
        assert!(gone.is_none(), "evicted entry id must no longer be found");
    }

    // Todo 4 invalidation semantics.
    // Todo 4 invalidation semantics.
    //
    // (a) After an undo succeeds (entry routed to redo), a new Create clears redo.
    #[test]
    fn undo_invalidation_new_op_clears_redo() {
        let mut history = UndoHistory::new();
        let entry = make_entry(0, "original delete");
        history.undo_stack.push(entry.clone());
        let entry = history.undo_stack.pop().unwrap();
        let inverse_id = 100;
        history.suppressed_ops.insert(inverse_id, entry);
        history.pending_undo_direction = Some(true);
        // Undo succeeds: the entry lands on the redo stack.
        let _ = history.route_suppressed_success(inverse_id);
        assert_eq!(history.redo_stack.len(), 1);

        // A new Create records and clears redo.
        let create = build_undo_entry(
            &Operation::NewFile {
                path: PathBuf::from("/tmp/new"),
            },
            &OperationSelection::default(),
        )
        .unwrap();
        let _ = history.record_undo(create);
        assert!(history.redo_stack.is_empty(), "new op clears redo");
    }

    // (b) Permanent delete with NO undo activity: both stacks empty, nothing held,
    // (b) Permanent delete with NO undo activity: both stacks empty, nothing held,
    // nothing suppressed, nothing deferred.
    #[test]
    fn undo_invalidation_permanent_delete_idle_clears_all() {
        let mut history = UndoHistory::new();
        history.undo_stack.push(make_entry(0, "a"));
        history.redo_stack.push(make_entry(1, "b"));
        history.pending_undo = Some(make_entry(2, "held"));

        let dismiss = history.clear_all();
        assert!(dismiss.is_empty());
        assert!(history.undo_stack.is_empty());
        assert!(history.redo_stack.is_empty());
        assert!(history.pending_undo.is_none());
        assert!(history.suppressed_ops.is_empty());
        assert!(history.pending_invalidation.is_none());
        assert!(history.pending_undo_direction.is_none());
    }

    // (c) Permanent delete while an inverse is in flight: pending_invalidation is
    // (c) Permanent delete while an inverse is in flight: pending_invalidation is
    // Some(inverse_id) until that inverse completes, then all stacks empty and
    // nothing recorded. The permanent-delete op itself (its OWN id) does NOT
    // clear by itself.
    #[test]
    fn undo_invalidation_while_inverse_in_flight() {
        let mut history = UndoHistory::new();
        history.undo_stack.push(make_entry(0, "a"));
        history.redo_stack.push(make_entry(1, "b"));
        let held = make_entry(2, "held");
        history.pending_undo = Some(held.clone());
        let inverse_id = 300;
        history.suppressed_ops.insert(inverse_id, held);
        history.pending_undo_direction = Some(true);

        // App::invalidate_undo_history state (1): an inverse IS in flight.
        assert_eq!(history.inverse_in_flight(), Some(inverse_id));
        history.pending_invalidation = Some(inverse_id);

        // NOT cleared yet (deferred): stacks still hold their entries.
        assert_eq!(history.undo_stack.len(), 1);
        assert_eq!(history.redo_stack.len(), 1);
        assert_eq!(history.pending_invalidation, Some(inverse_id));

        // A different operation (e.g. the permanent delete itself) completing
        // does NOT trigger the clear and is not swallowed.
        assert!(history.deferred_invalidation(inverse_id + 1).is_none());
        assert_eq!(history.pending_invalidation, Some(inverse_id));

        // The in-flight inverse completes: everything cleared, nothing recorded.
        let dismiss = history.deferred_invalidation(inverse_id);
        assert!(dismiss.is_some());
        assert!(history.undo_stack.is_empty());
        assert!(history.redo_stack.is_empty());
        assert!(history.pending_undo.is_none());
        assert!(history.suppressed_ops.is_empty());
        assert!(history.pending_invalidation.is_none());
        assert!(history.pending_undo_direction.is_none());
    }

    // (d) Permanent delete while a destructive entry is HELD in pending_undo
    // (empty suppressed_ops - the Todo-7 ConfirmUndo dialog-open case): the clear
    // is immediate; the held entry is dropped.
    #[test]
    fn undo_invalidation_dialog_held_clears_immediately() {
        let mut history = UndoHistory::new();
        history.undo_stack.push(make_entry(0, "a"));
        history.redo_stack.push(make_entry(1, "b"));
        history.pending_undo = Some(make_entry(2, "held destructive"));

        // No inverse in flight: immediate clear.
        assert!(history.inverse_in_flight().is_none());
        let dismiss = history.clear_all();
        assert!(history.undo_stack.is_empty());
        assert!(history.redo_stack.is_empty());
        assert!(history.pending_undo.is_none(), "held entry dropped");
        assert!(history.suppressed_ops.is_empty());
        assert!(history.pending_invalidation.is_none());
        assert!(dismiss.is_empty());
    }

    // (e) After EmptyTrash completes, both stacks are empty.
    #[test]
    fn undo_invalidation_empty_trash_completes_clears_all() {
        let mut history = UndoHistory::new();
        history.undo_stack.push(make_entry(0, "delete"));
        history.redo_stack.push(make_entry(1, "redo"));

        // EmptyTrash is not itself undoable.
        assert!(build_undo_entry(&Operation::EmptyTrash, &OperationSelection::default()).is_none());
        // Completion: immediate clear (no inverse in flight).
        let dismiss = history.clear_all();
        assert!(dismiss.is_empty());
        assert!(history.undo_stack.is_empty());
        assert!(history.redo_stack.is_empty());
        assert!(history.pending_undo.is_none());
    }

    // Todo 8 acceptance: every new undo/redo i18n key resolves to its expected
    // English string in the compiled fluent bundle. A missing key falls back to
    // the key name itself (non-empty), so exact-text equality is the proof.
    #[test]
    fn undo_i18n_keys() {
        assert_eq!(fl!("redo"), "Redo");
        assert_eq!(fl!("undo-delete"), "Undo Delete");
        assert_eq!(fl!("undo-rename"), "Undo Rename");
        assert_eq!(fl!("undo-move"), "Undo Move");
        assert_eq!(fl!("undo-copy"), "Undo Copy");
        assert_eq!(fl!("undo-create"), "Undo Create");
        assert_eq!(fl!("redo-delete"), "Redo Delete");
        assert_eq!(fl!("redo-rename"), "Redo Rename");
        assert_eq!(fl!("redo-move"), "Redo Move");
        assert_eq!(fl!("redo-copy"), "Redo Copy");
        assert_eq!(fl!("redo-create"), "Redo Create");
        assert_eq!(fl!("confirm-undo-trash"), "Move to Trash");
        assert_eq!(fl!("confirm-undo-cancel"), "Cancel");
        // The count-based confirmation strings resolve with a plural placeholder.
        // Fluent wraps interpolated values in bidi isolation markers
        // (U+2068 / U+2069), so strip them before comparing exact text.
        let stripped = |s: String| s.replace(['\u{2068}', '\u{2069}'], "");
        assert_eq!(
            stripped(fl!("confirm-undo-copy", count = 1)),
            "Move 1 item to the Trash?"
        );
        assert_eq!(
            stripped(fl!("confirm-undo-copy", count = 3)),
            "Move 3 items to the Trash?"
        );
        assert_eq!(
            stripped(fl!("confirm-undo-create", count = 1)),
            // This key (per its fluent definition) shows the plural form of
            // "item" without echoing the count itself.
            "Delete the created item?"
        );
        assert_eq!(
            stripped(fl!("confirm-undo-create", count = 3)),
            "Delete the created items?"
        );
    }
}
