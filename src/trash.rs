use cosmic::widget;
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::IconSizes;
use crate::tab::{Item, SearchItem};

/// Cached trash empty/full state. The real check walks every mount and blocks on slow ones,
/// so it runs off-thread via [`TrashExt::refresh_is_empty`]; GUI code reads the cache via
/// [`TrashExt::is_empty_cached`].
static TRASH_IS_EMPTY: AtomicBool = AtomicBool::new(true);

pub trait TrashExt {
    fn is_empty() -> bool {
        true
    }

    /// Last known empty state, read without touching the filesystem. Safe on the GUI thread.
    fn is_empty_cached() -> bool {
        TRASH_IS_EMPTY.load(Ordering::Relaxed)
    }

    /// Recompute and cache the state, returning the fresh value. Blocks on I/O across every
    /// mount — call only from a background thread.
    fn refresh_is_empty() -> bool {
        TRASH_IS_EMPTY.store(true, Ordering::Relaxed);
        true
    }

    fn entries() -> usize {
        0
    }

    fn folders() -> Result<HashSet<PathBuf>, trash::Error> {
        Err(trash::Error::Unknown {
            description: "reading trash folders not supported on this platform".into(),
        })
    }

    fn scan(_sizes: IconSizes) -> Vec<Item> {
        log::warn!("viewing trash not supported on this platform");
        Vec::new()
    }

    fn scan_search<F: Fn(SearchItem) -> bool + Sync>(_callback: F, _regex: &Regex) {}

    // Icons read the cached empty state so they are safe to build on the GUI thread.
    fn icon(icon_size: u16) -> widget::icon::Handle {
        widget::icon::from_name(if Self::is_empty_cached() {
            "user-trash"
        } else {
            "user-trash-full"
        })
        .size(icon_size)
        .handle()
    }

    fn icon_symbolic(icon_size: u16) -> widget::icon::Handle {
        widget::icon::from_name(if Self::is_empty_cached() {
            "user-trash-symbolic"
        } else {
            "user-trash-full-symbolic"
        })
        .size(icon_size)
        .handle()
    }
}

pub struct Trash;

// This config statement is from trash::os_limited
#[cfg(any(
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
))]
impl TrashExt for Trash {
    // Walks the trash dir on every mount and blocks on slow ones; never call on the GUI thread.
    fn is_empty() -> bool {
        trash::os_limited::is_empty().unwrap_or(true)
    }

    fn refresh_is_empty() -> bool {
        let is_empty = Self::is_empty();
        TRASH_IS_EMPTY.store(is_empty, Ordering::Relaxed);
        is_empty
    }

    fn entries() -> usize {
        match trash::os_limited::list() {
            Ok(entries) => entries.len(),
            Err(_err) => 0,
        }
    }

    // Not available on Windows only
    #[cfg(not(target_os = "windows"))]
    fn folders() -> Result<HashSet<PathBuf>, trash::Error> {
        trash::os_limited::trash_folders()
    }

    fn scan(sizes: IconSizes) -> Vec<Item> {
        use crate::localize::LANGUAGE_SORTER;
        use crate::tab::item_from_trash_entry;
        use std::cmp::Ordering;

        let entries = match trash::os_limited::list() {
            Ok(entry) => entry,
            Err(err) => {
                log::warn!("failed to read trash items: {err}");
                return Vec::new();
            }
        };
        let mut items: Vec<_> = entries
            .into_iter()
            .filter_map(|entry| {
                let metadata = trash::os_limited::metadata(&entry)
                    .inspect_err(|err| {
                        log::warn!("failed to get metadata for trash item {entry:?}: {err}")
                    })
                    .ok()?;
                Some(item_from_trash_entry(entry, metadata, sizes))
            })
            .collect();
        items.sort_by(|a, b| match (a.metadata.is_dir(), b.metadata.is_dir()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => LANGUAGE_SORTER.compare(&a.display_name, &b.display_name),
        });
        items
    }

    fn scan_search<F: Fn(SearchItem) -> bool + Sync>(callback: F, regex: &Regex) {
        let entries = match trash::os_limited::list() {
            Ok(entries) => entries,
            Err(err) => {
                log::warn!("failed to read trash items: {err}");
                return;
            }
        };

        for entry in entries {
            if let Ok(metadata) = trash::os_limited::metadata(&entry).inspect_err(|err| {
                log::warn!("failed to get metadata for trash item {entry:?}: {err}")
            }) {
                let name = entry.name.to_string_lossy();
                if regex.is_match(&name) && !callback(SearchItem::Trash(entry, metadata)) {
                    break;
                }
            }
        }
    }
}

// This config statement is from trash::os_limited, inverted
#[cfg(not(any(
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
)))]
impl TrashExt for Trash {}
