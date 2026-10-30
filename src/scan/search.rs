use crate::item::SearchItem;
use crate::location::SearchLocation;
use crate::scan::uri_to_path;
use crate::trash::{Trash, TrashExt};

pub fn scan_search<F: Fn(SearchItem) -> bool + Sync>(
    search_location: &SearchLocation,
    term: &str,
    show_hidden: bool,
    callback: F,
) {
    if term.is_empty() {
        return;
    }

    let pattern = regex::escape(term);
    let regex = match regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
    {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse regex {pattern:?}: {err}");
            return;
        }
    };

    match search_location {
        SearchLocation::Path(tab_path) => {
            ignore::WalkBuilder::new(tab_path)
                .standard_filters(false)
                .hidden(!show_hidden)
                //TODO: only use this on supported targets
                .same_file_system(true)
                .build_parallel()
                .run(|| {
                    Box::new(|entry_res| {
                        let Ok(entry) = entry_res else {
                            // Skip invalid entries
                            return ignore::WalkState::Skip;
                        };

                        let Some(file_name) = entry.file_name().to_str() else {
                            // Skip anything with an invalid name
                            return ignore::WalkState::Skip;
                        };

                        if regex.is_match(file_name) {
                            let path = entry.path();

                            let metadata = match entry.metadata() {
                                Ok(ok) => ok,
                                Err(err) => {
                                    log::warn!(
                                        "failed to read metadata for entry at {}: {}",
                                        path.display(),
                                        err
                                    );
                                    return ignore::WalkState::Continue;
                                }
                            };

                            if !callback(SearchItem::Path(
                                path.to_path_buf(),
                                file_name.to_string(),
                                metadata,
                            )) {
                                return ignore::WalkState::Quit;
                            }
                        }

                        ignore::WalkState::Continue
                    })
                });
        }
        SearchLocation::Recents => {
            let recent_files = match recently_used_xbel::parse_file() {
                Ok(recent_files) => recent_files,
                Err(err) => {
                    log::warn!("Error reading recent files: {err:?}");
                    return;
                }
            };

            for bookmark in recent_files.bookmarks {
                let path = uri_to_path(bookmark.href);
                if let Some(path) = path
                    && path.exists()
                {
                    let file_name = path.file_name();
                    if let Some(file_name) = file_name {
                        let file_name = file_name.to_string_lossy();
                        if regex.is_match(&file_name) {
                            match path.metadata() {
                                Ok(metadata) => {
                                    if !callback(SearchItem::Path(
                                        path.to_path_buf(),
                                        file_name.to_string(),
                                        metadata,
                                    )) {
                                        break;
                                    }
                                }
                                Err(err) => {
                                    log::warn!(
                                        "failed to read metadata for entry at {}: {}",
                                        path.display(),
                                        err
                                    );
                                }
                            };
                        }
                    }
                }
            }
        }
        SearchLocation::Trash => {
            Trash::scan_search(callback, &regex);
        }
    }
}
