use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::config::IconSizes;
use crate::item::{self, FsKind, Item};
use crate::localize::LANGUAGE_SORTER;
use crate::trash::item_from_trash_child;

// parse .hidden file and return files path
pub fn parse_hidden_file(path: &PathBuf) -> Box<[String]> {
    let Ok(file) = File::open(path) else {
        return Default::default();
    };

    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| {
            let line = line.trim();
            (!line.is_empty()).then_some(line.to_owned())
        })
        .collect()
}

pub fn scan_path(tab_path: &PathBuf, sizes: IconSizes) -> Vec<Item> {
    let mut items = Vec::new();
    let mut hidden_files = Box::from([]);
    let mut remote_scannable = false;

    #[cfg(feature = "gvfs")]
    {
        if let Ok(path_meta) = fs::metadata(tab_path)
            && FsKind::new(&path_meta) == FsKind::Gvfs
        {
            let file = gio::File::for_path(tab_path);

            // gio crate expects a comma delimited string
            let attr_string = [
                gio::FILE_ATTRIBUTE_STANDARD_DISPLAY_NAME.as_str(),
                gio::FILE_ATTRIBUTE_TIME_MODIFIED.as_str(),
                gio::FILE_ATTRIBUTE_STANDARD_SIZE.as_str(),
                gio::FILE_ATTRIBUTE_STANDARD_TYPE.as_str(),
                gio::FILE_ATTRIBUTE_STANDARD_NAME.as_str(),
            ]
            .join(",");

            match gio::prelude::FileExt::enumerate_children(
                &file,
                attr_string.as_str(),
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            ) {
                Ok(res) => {
                    remote_scannable = true;
                    items = res
                        .filter_map(|file| {
                            let file = file.ok()?;
                            Some(item::item_from_gvfs_info(
                                tab_path.join(file.name()),
                                file,
                                sizes,
                            ))
                        })
                        .collect();
                }
                Err(err) => {
                    log::warn!(
                        "could not enumerate {} via gio: {}",
                        tab_path.display(),
                        err
                    );
                }
            }
        }
    }

    if !remote_scannable {
        match fs::read_dir(tab_path) {
            Ok(entries) => {
                let trash = crate::trash::is_trash_path(tab_path);
                items = entries
                    .filter_map(|entry_res| {
                        let entry = entry_res
                            .inspect_err(|err| {
                                log::warn!(
                                    "failed to read entry in {}: {}",
                                    tab_path.display(),
                                    err
                                )
                            })
                            .ok()?;

                        let path = entry.path();

                        let name = entry
                            .file_name()
                            .into_string()
                            .inspect_err(|name_os| {
                                log::warn!(
                                    "failed to parse entry at {}: {:?} is not valid UTF-8",
                                    path.display(),
                                    name_os
                                )
                            })
                            .ok()?;

                        if name == ".hidden" && path.is_file() {
                            hidden_files = parse_hidden_file(&path);
                        }

                        let metadata = fs::metadata(&path)
                            .inspect_err(|err| {
                                log::warn!(
                                    "failed to read metadata for entry at {}: {}",
                                    path.display(),
                                    err
                                )
                            })
                            .ok()?;

                        if trash {
                            item_from_trash_child(path, name, metadata, sizes)
                        } else {
                            Some(item::item_from_entry(path, name, metadata, sizes))
                        }
                    })
                    .collect();
            }
            Err(err) => {
                log::warn!("failed to read directory {}: {}", tab_path.display(), err);
            }
        }
    }
    items.sort_unstable_by(|a, b| match (a.metadata.is_dir(), b.metadata.is_dir()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => LANGUAGE_SORTER.compare(&a.display_name, &b.display_name),
    });
    for item in &mut items {
        if hidden_files.contains(&item.name) {
            item.hidden = true;
        }
    }
    items
}
