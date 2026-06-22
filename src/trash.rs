use cosmic::widget;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::config::IconSizes;
use crate::tab::{Item, SearchItem};

fn percent_decode(s: &str) -> Option<String> {
    let mut r = String::with_capacity(s.len());
    let mut b = s.bytes();
    while let Some(c) = b.next() {
        if c == b'%' {
            let hi = b.next().and_then(hex)?;
            let lo = b.next().and_then(hex)?;
            r.push((hi << 4 | lo) as char);
        } else {
            r.push(c as char);
        }
    }
    Some(r)
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

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

    fn scan_search<F: Fn(SearchItem) -> bool + Sync>(_callback: F, _regex: &Regex) {}

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

/// Derive the actual filesystem path of a trashed item from its .trashinfo path
/// (or return the id directly if it's already a filesystem path).
pub fn trash_item_path(item: &trash::TrashItem) -> Option<PathBuf> {
    let id_path = Path::new(&item.id);
    if id_path.extension().map_or(false, |e| e == "trashinfo") {
        let trash_root = id_path.parent()?.parent()?;
        let file_name = id_path.file_stem()?;
        Some(trash_root.join("files").join(file_name))
    } else {
        Some(PathBuf::from(&item.id))
    }
}

/// For a path inside a trash `files/` directory, reconstruct the original path
/// by reading the parent `.trashinfo` file.
///
/// Given `~/.local/share/Trash/files/folder/sub/file.txt`:
/// - The top-level trashed item is `folder`
/// - Read `~/.local/share/Trash/info/folder.trashinfo` to get the original path
/// - Compute: `<original_path>/sub/file.txt`
pub fn original_path_for_trash_child(trash_path: &Path) -> Option<PathBuf> {
    let trash_files = trash_path.ancestors().find(|a| a.ends_with("files"))?;
    let trash_root = trash_files.parent()?;
    let top_name = trash_path
        .strip_prefix(trash_files)
        .ok()?
        .components()
        .next()?;

    let info_path = trash_root
        .join("info")
        .join(top_name)
        .with_extension("trashinfo");
    let info = std::fs::read_to_string(&info_path).ok()?;
    let original = info
        .lines()
        .find_map(|line| line.strip_prefix("Path="))?
        .trim();
    let original = percent_decode(original)?;

    let relative = trash_path.strip_prefix(trash_files.join(top_name)).ok()?;
    let mut result = PathBuf::from(&original);
    if !relative.as_os_str().is_empty() {
        result.push(relative);
    }
    Some(result)
}

/// Check whether a path is inside any trash `files/` directory.
pub fn is_trash_path(path: &Path) -> bool {
    path.ancestors().any(|a| a.ends_with("files"))
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
