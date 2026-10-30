use cosmic::widget;
use regex::Regex;
use std::path::Path;

use crate::config::IconSizes;
use crate::item::{Item, SearchItem};

#[cfg(feature = "gvfs")]
#[path = "gvfs.rs"]
mod imp;

#[cfg(not(feature = "gvfs"))]
#[path = "fallback.rs"]
mod imp;

pub use imp::*;

// from trash-rs, license MIT
/// Size of a [`TrashItem`] in bytes or entries
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum TrashItemSize {
    /// Number of bytes in a file
    Bytes(u64),
    /// Number of entries in a directory, non-recursive
    Entries(usize),
}

pub trait TrashExt {
    fn is_empty() -> bool {
        Self::entries() == 0
    }

    fn entries() -> usize;

    fn list() -> Result<Vec<TrashItem>, String>;

    fn purge(_item: TrashItem) -> Result<(), String>;

    fn restore(_item: TrashItem) -> Result<(), String>;

    fn trash<P: AsRef<Path>>(_path: P) -> Result<(), String>;

    fn scan(_sizes: IconSizes) -> Vec<Item>;

    fn scan_search<F: Fn(SearchItem) -> bool + Sync>(_callback: F, _regex: &Regex);

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

fn is_trash_root(root: &Path) -> bool {
    let Some(name) = root.file_name().and_then(|x| x.to_str()) else {
        return false;
    };
    (name == "Trash" || name.starts_with(".Trash")) && root.join("info").is_dir()
}

fn trash_files_dir(path: &Path) -> Option<&Path> {
    path.ancestors()
        .find(|a| a.ends_with("files") && a.parent().is_some_and(is_trash_root))
}

/// Check whether a path is inside any trash `files/` directory.
pub fn is_trash_path(path: &Path) -> bool {
    trash_files_dir(path).is_some()
}

pub struct Trash;
