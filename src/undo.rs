// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::operation::Operation;
use cosmic::widget::toaster::ToastId;
use std::collections::VecDeque;

/// Maximum capacity of the undo/redo stacks (ring buffer).
const UNDO_STACK_CAPACITY: usize = 20;

/// An entry in the undo/redo stack.
///
/// The `entry_id` is a unique, monotonic identifier used by `Message::AssignToastId`
/// to locate the entry (NOT a stack index, which shifts on eviction).
#[derive(Clone, Debug)]
pub struct UndoEntry {
    #[allow(dead_code)] // Todo 3 (Message::AssignToastId) locates the entry by this id
    pub entry_id: u64,
    pub forward: Operation,
    pub inverse: Operation,
    #[allow(dead_code)] // Todo 5/6 (Edit menu + context menu) render this as the dynamic label
    pub description: String,
    pub destructive: bool, // true when inverse moves files to Trash (undo-copy / undo-create)
    #[allow(dead_code)]
    // Todo 3/8 (evict/clear) dismiss this toast via self.toasts.remove(toast_id)
    pub toast_id: Option<ToastId>, // set via Message::AssignToastId (Todo 3); dismisses stale toasts on evict/clear
}

/// A bounded ring-buffer stack for undo/redo entries.
///
/// This struct encapsulates the stack logic so it can be tested independently
/// of the full `App` state machine.
#[derive(Clone, Debug, Default)]
pub struct UndoStack {
    entries: VecDeque<UndoEntry>,
}

#[allow(dead_code)] // push (Todo 3 record_undo), peek/is_empty/len (Todo 5 menu), clear (Todo 4), iter (tests)
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
    /// dismiss that toast via `self.toasts.remove(toast_id)` (wired in Todo 3
    /// when `record_undo` assigns toast ids asynchronously), so a stale toast's
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;
    use std::path::PathBuf;

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
}
