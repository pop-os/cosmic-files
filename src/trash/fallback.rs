use mime_guess::mime;
use regex::Regex;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::config::IconSizes;
use crate::item::{Item, SearchItem};
use crate::trash::{Trash, TrashExt, TrashItemSize, trash_files_dir};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TrashItem {
    pub name: OsString,
    pub original_path: PathBuf,
    pub time_deleted: Option<SystemTime>,
}

pub fn item_from_trash_entry(_entry: TrashItem, _size: TrashItemSize, _sizes: IconSizes) -> Item {
    panic!("TrashItems should not exist on this platform")
}

pub fn item_from_trash_child(
    path: PathBuf,
    name: String,
    metadata: fs::Metadata,
    sizes: IconSizes,
) -> Option<Item> {
    None
}

impl TrashExt for Trash {
    fn entries() -> usize {
        0
    }

    fn list() -> Result<Vec<TrashItem>, String> {
        Err("trash listing not supported on this platform".into())
    }

    fn purge(_item: TrashItem) -> Result<(), String> {
        Err("trash purge not supported on this platform".into())
    }

    fn restore(_item: TrashItem) -> Result<(), String> {
        Err("trash restore not supported on this platform".into())
    }

    fn trash<P: AsRef<Path>>(_path: P) -> Result<(), String> {
        Err("trashing files not supported on this platform".into())
    }

    fn scan(_sizes: IconSizes) -> Vec<Item> {
        Vec::new()
    }

    fn scan_search<F: Fn(SearchItem) -> bool + Sync>(_callback: F, _regex: &Regex) {}
}
