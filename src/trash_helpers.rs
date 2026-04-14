// SPDX-License-Identifier: GPL-3.0-only
use cosmic::widget;
use regex::Regex;
use std::cmp::Ordering as CmpOrdering;
use std::path::PathBuf;
use crate::config::IconSizes;
use crate::localize::LANGUAGE_SORTER;
use crate::tab::{Item, SearchItem, item_from_trash_entry};

#[cfg(not(any(
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
)))]
mod os_limited {
    use super::*;

    pub fn is_empty_blocking() -> bool {
        true
    }

    pub fn trash_folders_blocking() -> Vec<PathBuf> {
        Vec::new()
    }

    pub fn trash_entries() -> usize {
        0
    }

    pub fn trash_icon(icon_size: u16, _is_empty: bool) -> widget::icon::Handle {
        widget::icon::from_name("user-trash")
            .size(icon_size)
            .handle()
    }

    pub fn trash_icon_symbolic(icon_size: u16, _is_empty: bool) -> widget::icon::Handle {
        widget::icon::from_name("user-trash-symbolic")
            .size(icon_size)
            .handle()
    }

    pub fn scan_trash(_sizes: IconSizes) -> Vec<Item> {
        log::warn!("viewing trash not supported on this platform");
        Vec::new()
    }

    pub fn scan_search_trash<F: Fn(SearchItem) -> bool + Sync>(_callback: F, _regex: &Regex) {}
}

#[cfg(any(
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
))]
mod os_limited {
    use super::*;

    pub fn is_empty_blocking() -> bool {
        trash::os_limited::is_empty().unwrap_or(true)
    }

    pub fn trash_folders_blocking() -> Vec<PathBuf> {
        trash::os_limited::trash_folders().unwrap_or_default().into_iter().collect()
    }

    pub fn trash_entries() -> usize {
        match trash::os_limited::list() {
            Ok(entries) => entries.len(),
            Err(_err) => 0,
        }
    }

    pub fn trash_icon(icon_size: u16, is_empty: bool) -> widget::icon::Handle {
        widget::icon::from_name(if is_empty {
            "user-trash"
        } else {
            "user-trash-full"
        })
        .size(icon_size)
        .handle()
    }

    pub fn trash_icon_symbolic(icon_size: u16, is_empty: bool) -> widget::icon::Handle {
        widget::icon::from_name(if is_empty {
            "user-trash-symbolic"
        } else {
            "user-trash-full-symbolic"
        })
        .size(icon_size)
        .handle()
    }

    pub fn scan_trash(sizes: IconSizes) -> Vec<Item> {
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
            (true, false) => CmpOrdering::Less,
            (false, true) => CmpOrdering::Greater,
            _ => LANGUAGE_SORTER.compare(&a.display_name, &b.display_name),
        });
        items
    }

    pub fn scan_search_trash<F: Fn(SearchItem) -> bool + Sync>(callback: F, regex: &Regex) {
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

pub use os_limited::*;
