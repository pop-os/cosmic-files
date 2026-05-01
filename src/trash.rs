use cosmic::widget;
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
use std::path::Path;

use crate::config::IconSizes;
use crate::tab::{Item, SearchItem};

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
const MISSING_TRASH_FILE: &str = "trash item referenced by .trashinfo is missing from trash/files";

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

pub struct Trash;

#[cfg(target_os = "windows")]
fn metadata(item: &trash::TrashItem) -> Result<trash::TrashItemMetadata, trash::Error> {
    trash::os_limited::metadata(item)
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
fn metadata(item: &trash::TrashItem) -> Result<trash::TrashItemMetadata, trash::Error> {
    let file = restorable_file_in_trash_from_info_file(&item.id)?;
    ensure_virtually_exists(&file)?;

    let metadata = std::fs::symlink_metadata(&file).map_err(|e| fs_error(&file, e))?;
    let size = if metadata.is_dir() {
        trash::TrashItemSize::Entries(
            std::fs::read_dir(&file)
                .map_err(|e| fs_error(&file, e))?
                .count(),
        )
    } else {
        trash::TrashItemSize::Bytes(metadata.len())
    };

    Ok(trash::TrashItemMetadata { size })
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
fn restorable_file_in_trash_from_info_file(
    info_file: impl AsRef<std::ffi::OsStr>,
) -> Result<PathBuf, trash::Error> {
    let info_path = Path::new(info_file.as_ref());
    let trash_folder =
        info_path
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| trash::Error::Unknown {
                description: format!("invalid trash info path: {info_path:?}"),
            })?;
    let name_in_trash = info_path.file_stem().ok_or_else(|| trash::Error::Unknown {
        description: format!("invalid trash info path: {info_path:?}"),
    })?;

    Ok(trash_folder.join("files").join(name_in_trash))
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
fn ensure_virtually_exists(path: &Path) -> Result<(), trash::Error> {
    if path.try_exists().map_err(|e| fs_error(path, e))? || path.is_symlink() {
        Ok(())
    } else {
        Err(fs_error(
            path,
            std::io::Error::new(std::io::ErrorKind::NotFound, MISSING_TRASH_FILE),
        ))
    }
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
fn fs_error(path: impl Into<PathBuf>, source: std::io::Error) -> trash::Error {
    trash::Error::FileSystem {
        path: path.into(),
        source,
    }
}

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
                let metadata = metadata(&entry)
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
            if let Ok(metadata) = metadata(&entry).inspect_err(|err| {
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

#[cfg(test)]
#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "ios"),
    not(target_os = "android")
))]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};
    use std::{fs, io};

    use tempfile::tempdir;

    use super::metadata;

    fn trash_item(info_file: &Path, original_parent: &Path, name: &str) -> trash::TrashItem {
        trash::TrashItem {
            id: info_file.as_os_str().to_os_string(),
            name: OsString::from(name),
            original_parent: original_parent.to_path_buf(),
            time_deleted: 0,
        }
    }

    fn assert_missing_trash_file(err: trash::Error, expected_path: PathBuf) {
        match err {
            trash::Error::FileSystem { path, source } => {
                assert_eq!(expected_path, path);
                assert_eq!(io::ErrorKind::NotFound, source.kind());
            }
            other => panic!("expected missing trash file error, got {other:?}"),
        }
    }

    #[test]
    fn metadata_returns_error_when_trash_file_is_missing() -> io::Result<()> {
        let temp = tempdir()?;
        let trash = temp.path().join("Trash");
        let info_dir = trash.join("info");
        fs::create_dir_all(&info_dir)?;

        let info_file = info_dir.join("missing.trashinfo");
        fs::write(&info_file, "")?;

        let item = trash_item(&info_file, temp.path(), "missing");
        let err = metadata(&item).expect_err("missing trash file should be an error");

        assert_missing_trash_file(err, trash.join("files").join("missing"));
        Ok(())
    }

    #[test]
    fn metadata_reads_existing_trash_file_size() -> io::Result<()> {
        let temp = tempdir()?;
        let trash = temp.path().join("Trash");
        let info_dir = trash.join("info");
        let files_dir = trash.join("files");
        fs::create_dir_all(&info_dir)?;
        fs::create_dir_all(&files_dir)?;

        let info_file = info_dir.join("example.trashinfo");
        let trashed_file = files_dir.join("example");
        fs::write(&info_file, "")?;
        fs::write(&trashed_file, b"data")?;

        let item = trash_item(&info_file, temp.path(), "example");
        let metadata = metadata(&item).expect("existing trash file should have metadata");

        assert_eq!(trash::TrashItemSize::Bytes(4), metadata.size);
        Ok(())
    }
}
