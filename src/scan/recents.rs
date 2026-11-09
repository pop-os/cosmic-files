use std::cmp::Reverse;
use std::path::PathBuf;

use crate::config::IconSizes;
use crate::item::{Item, item_from_entry};

pub fn has_recents() -> bool {
    match recently_used_xbel::parse_file() {
        Ok(recent_files) => !recent_files.bookmarks.is_empty(),
        Err(_) => false,
    }
}

pub fn scan_recents(sizes: IconSizes) -> Vec<Item> {
    let recent_files = match recently_used_xbel::parse_file() {
        Ok(recent_files) => recent_files,
        Err(err) => {
            log::warn!("Error reading recent files: {err:?}");
            return Vec::new();
        }
    };
    let mut recents: Vec<_> = recent_files
        .bookmarks
        .into_iter()
        .filter_map(|bookmark| {
            let path = uri_to_path(bookmark.href)?;
            let last_edit = bookmark.modified.parse::<jiff::Timestamp>().ok()?;
            let last_visit = bookmark.visited.parse::<jiff::Timestamp>().ok()?;
            Some((path, last_edit.min(last_visit)))
        })
        .collect();

    recents.sort_by_key(|recent| Reverse(recent.1));

    recents
        .into_iter()
        .filter_map(|(path, _)| {
            if path.exists() {
                let file_name = path.file_name()?;
                let name = file_name.to_string_lossy().to_string();

                let metadata = match path.metadata() {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!(
                            "failed to read metadata for entry at {}: {}",
                            path.display(),
                            err
                        );
                        return None;
                    }
                };

                Some(item_from_entry(path, name, metadata, sizes))
            } else {
                log::warn!("recent file path does not exist: {}", path.display());
                None
            }
        })
        .take(50)
        .collect()
}

pub fn uri_to_path(uri: String) -> Option<PathBuf> {
    uri.parse::<url::Url>().ok().and_then(|url| {
        //TODO support for external drive or cloud?
        if url.scheme() == "file" {
            url.to_file_path().ok()
        } else {
            None
        }
    })
}
