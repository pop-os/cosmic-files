use gio::prelude::*;
use mime_guess::mime;
use regex::Regex;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::config::IconSizes;
use crate::err_str;
use crate::item::{DirSize, Item, ItemMetadata, ItemState, ItemThumbnail, SearchItem, folder_icon};
use crate::location::Location;
use crate::mime_icon::{mime_for_path, mime_icon};
use crate::trash::{Trash, TrashExt, TrashItemSize, trash_files_dir};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TrashItem {
    file: gio::File,
    pub name: OsString,
    pub original_path: PathBuf,
    size: TrashItemSize,
    pub time_deleted: Option<SystemTime>,
}

const SCAN_ATTRIBUTES: &str = "standard::name,\
standard::display-name,\
standard::type,\
standard::size,\
standard::icon,\
standard::is-hidden,\
time::modified,\
trash::orig-path,\
trash::deletion-date";

fn percent_decode(s: &str) -> Option<String> {
    let (mut r, mut b) = (String::with_capacity(s.len()), s.bytes());
    while let Some(c) = b.next() {
        if c == b'%' {
            let (hi, lo) = (b.next()?, b.next()?);
            let h = |x: u8| match x {
                b'0'..=b'9' => Some(x - b'0'),
                b'a'..=b'f' => Some(x - b'a' + 10),
                b'A'..=b'F' => Some(x - b'A' + 10),
                _ => None,
            };
            r.push((h(hi)? << 4 | h(lo)?) as char);
        } else {
            r.push(c as char);
        }
    }
    Some(r)
}

/// For a path inside a trash `files/` directory, reconstruct the original path
/// by reading the parent `.trashinfo` file.
///
/// Given `~/.local/share/Trash/files/folder/sub/file.txt`:
/// - The top-level trashed item is `folder`
/// - Read `~/.local/share/Trash/info/folder.trashinfo` to get the original path
/// - Compute: `<original_path>/sub/file.txt`
pub fn original_path_for_trash_child(p: &Path) -> Option<PathBuf> {
    let files = trash_files_dir(p)?;
    let root = files.parent()?;
    let top = p.strip_prefix(files).ok()?.components().next()?;
    let mut trashinfo_name = top.as_os_str().to_os_string();
    trashinfo_name.push(".trashinfo");
    let info = std::fs::read_to_string(root.join("info").join(trashinfo_name)).ok()?;
    let orig = percent_decode(info.lines().find_map(|l| l.strip_prefix("Path="))?.trim())?;
    let rel = p.strip_prefix(files.join(top)).ok()?;
    let mut result = PathBuf::from(&orig);
    if !rel.as_os_str().is_empty() {
        result.push(rel);
    }
    Some(result)
}

pub fn trash_file() -> gio::File {
    gio::File::for_uri("trash:///")
}

pub fn item_from_trash_entry(entry: TrashItem, size: TrashItemSize, sizes: IconSizes) -> Item {
    let name = entry.name.to_string_lossy().to_string();
    let display_name = Item::display_name(&name);

    let mut location_opt = entry.file.path().map(Location::Path);
    if location_opt.is_none() {
        match entry.file.query_info(
            gio::FILE_ATTRIBUTE_STANDARD_TARGET_URI,
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        ) {
            Ok(info) => {
                if let Some(target_uri) =
                    info.attribute_as_string(gio::FILE_ATTRIBUTE_STANDARD_TARGET_URI)
                {
                    location_opt = gio::File::for_uri(&target_uri).path().map(Location::Path);
                }
            }
            Err(err) => {
                log::error!("failed to query info for trash item {}: {}", name, err);
            }
        };
    }

    let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = match size {
        TrashItemSize::Entries(_) => (
            //TODO: make this a static
            "inode/directory".parse().unwrap(),
            folder_icon(&entry.original_path, sizes.grid()),
            folder_icon(&entry.original_path, sizes.list()),
            folder_icon(&entry.original_path, sizes.list_condensed()),
        ),
        TrashItemSize::Bytes(_) => {
            // This passes remote = true so it does not read from the original path
            let mime = mime_for_path(&entry.original_path, None, true);
            (
                mime.clone(),
                mime_icon(mime.clone(), sizes.grid()),
                mime_icon(mime.clone(), sizes.list()),
                mime_icon(mime, sizes.list_condensed()),
            )
        }
    };

    let image_dimensions = (mime.type_() == mime::IMAGE)
        .then(|| image::image_dimensions(&entry.original_path).ok())
        .flatten();

    Item {
        name,
        display_name,
        is_mount_point: false,
        metadata: ItemMetadata::Trash { size, entry },
        hidden: false,
        location_opt,
        image_dimensions,
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        thumbnail_opt: Some(ItemThumbnail::NotImage),
        dir_size: DirSize::NotDirectory,
        state: ItemState::default(),
    }
}

pub fn item_from_trash_child(
    path: PathBuf,
    name: String,
    metadata: fs::Metadata,
    sizes: IconSizes,
) -> Option<Item> {
    let Some(original_path) = original_path_for_trash_child(&path) else {
        log::warn!(
            "failed to resolve original path for trash item {}, skipping entry",
            path.display()
        );
        return None;
    };
    let size = if metadata.is_dir() {
        TrashItemSize::Entries(match fs::read_dir(&path) {
            Ok(read_dir) => read_dir.count(),
            Err(err) => {
                log::warn!("failed to read dir {}: {}", path.display(), err);
                0
            }
        })
    } else {
        TrashItemSize::Bytes(metadata.len())
    };
    let entry = TrashItem {
        file: gio::File::for_path(path),
        name: std::ffi::OsString::from(&name),
        original_path,
        size,
        time_deleted: None,
    };
    Some(item_from_trash_entry(entry, size, sizes))
}

impl TrashExt for Trash {
    fn entries() -> usize {
        let file = trash_file();
        match file.query_info(
            gio::FILE_ATTRIBUTE_TRASH_ITEM_COUNT,
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        ) {
            Ok(info) => info
                .attribute_uint32(gio::FILE_ATTRIBUTE_TRASH_ITEM_COUNT)
                .try_into()
                .unwrap(),
            Err(err) => {
                log::error!("failed to get trash item count: {}", err);
                0
            }
        }
    }

    fn list() -> Result<Vec<TrashItem>, String> {
        let file = trash_file();
        let mut infos = file
            .enumerate_children(
                SCAN_ATTRIBUTES,
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            )
            .map_err(err_str)?;

        let mut items = Vec::new();
        while let Some(info_res) = infos.next() {
            let info = match info_res {
                Ok(ok) => ok,
                Err(err) => {
                    log::error!("failed to enumerate trash item: {}", err);
                    continue;
                }
            };

            let Some(original_path) = info
                .attribute_as_string(gio::FILE_ATTRIBUTE_TRASH_ORIG_PATH)
                .map(PathBuf::from)
            else {
                log::error!(
                    "trash item {} does not have {} attribute",
                    info.name().display(),
                    gio::FILE_ATTRIBUTE_TRASH_ORIG_PATH
                );
                continue;
            };
            let Some(name) = original_path.file_name().map(|x| x.to_os_string()) else {
                log::error!("original path {} has no filename", original_path.display());
                continue;
            };

            let mut time_deleted = None;
            match info.attribute_as_string(gio::FILE_ATTRIBUTE_TRASH_DELETION_DATE) {
                Some(deletion_date) => {
                    match jiff::civil::DateTime::strptime("%Y-%m-%dT%H:%M:%S", &deletion_date) {
                        Ok(dt) => {
                            match dt.to_zoned(jiff::tz::TimeZone::system()).and_then(|zoned| {
                                (zoned - jiff::Zoned::UNIX_EPOCH).total(jiff::Unit::Second)
                            }) {
                                Ok(ok) => {
                                    time_deleted =
                                        Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(ok));
                                }
                                Err(err) => {
                                    log::error!(
                                        "failed to convert deletion date {}: {}",
                                        deletion_date,
                                        err
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("failed to parse deletion date {}: {}", deletion_date, err);
                        }
                    }
                }
                None => {
                    log::error!(
                        "trash item {} does not have {} attribute",
                        info.name().display(),
                        gio::FILE_ATTRIBUTE_TRASH_DELETION_DATE
                    );
                }
            }

            let size = if matches!(info.file_type(), gio::FileType::Directory) {
                let entries = match file.enumerate_children(
                    "standard::name",
                    gio::FileQueryInfoFlags::NONE,
                    gio::Cancellable::NONE,
                ) {
                    Ok(child_infos) => child_infos.count(),
                    Err(err) => {
                        log::error!(
                            "failed to enumerate trash directory {}: {}",
                            name.display(),
                            err
                        );
                        0
                    }
                };
                TrashItemSize::Entries(entries)
            } else {
                TrashItemSize::Bytes(info.size() as u64)
            };

            items.push(TrashItem {
                file: infos.child(&info),
                name,
                original_path,
                size,
                time_deleted,
            });
        }

        Ok(items)
    }

    fn purge(item: TrashItem) -> Result<(), String> {
        item.file.delete(gio::Cancellable::NONE).map_err(err_str)
    }

    fn restore(item: TrashItem) -> Result<(), String> {
        let dest = gio::File::for_path(&item.original_path);
        item.file
            .move_(
                &dest,
                gio::FileCopyFlags::NONE,
                gio::Cancellable::NONE,
                None,
            )
            .map_err(err_str)
    }

    fn trash<P: AsRef<Path>>(path: P) -> Result<(), String> {
        gio::File::for_path(path)
            .trash(gio::Cancellable::NONE)
            .map_err(err_str)
    }

    fn scan(sizes: IconSizes) -> Vec<Item> {
        let entries = match Self::list() {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("failed to enumerate trash: {}", err);
                return Vec::new();
            }
        };

        let mut items = Vec::with_capacity(entries.len());
        for entry in entries {
            let size = entry.size;
            items.push(item_from_trash_entry(entry, size, sizes))
        }

        items
    }

    fn scan_search<F: Fn(SearchItem) -> bool + Sync>(callback: F, regex: &Regex) {
        let entries = match Self::list() {
            Ok(entries) => entries,
            Err(err) => {
                log::warn!("failed to read trash items: {err}");
                return;
            }
        };

        for entry in entries {
            let name = entry.name.to_string_lossy();
            let size = entry.size;
            if regex.is_match(&name) && !callback(SearchItem::Trash(entry, size)) {
                break;
            }
        }
    }
}
