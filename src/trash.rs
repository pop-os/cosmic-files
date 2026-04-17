use cosmic::widget;
use regex::Regex;
use std::{collections::HashSet, path::PathBuf};

use crate::{
    config::IconSizes,
    tab::{Item, SearchItem},
};

pub trait TrashExt {
    fn is_empty() -> bool {
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

    fn scan_search<F: Fn(SearchItem) -> bool + Sync>(callback: F, regex: &Regex) {}

    fn icon(icon_size: u16) -> widget::icon::Handle {
        widget::icon::from_name(if Self::is_empty() {
            "user-trash"
        } else {
            "user-trash-full"
        })
        .size(icon_size)
        .handle()
    }

    fn icon_symbolic(icon_size: u16) -> widget::icon::Handle {
        widget::icon::from_name(if Self::is_empty() {
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
    fn is_empty() -> bool {
        trash::os_limited::is_empty().unwrap_or(true)
    }

    fn entries() -> usize {
        match trash::os_limited::list() {
            Ok(entries) => entries.len(),
            Err(_err) => 0,
        }
    }

    fn folders() -> Result<HashSet<PathBuf>, trash::Error> {
        trash::os_limited::trash_folders()
    }

    fn scan(sizes: IconSizes) -> Vec<Item> {
        use crate::{localize::LANGUAGE_SORTER, tab::item_from_trash_entry};
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
