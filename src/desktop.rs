use libc::BTRFS_SUPER_MAGIC;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

use crate::{config::DesktopConfig, tab::HeadingOptions};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DesktopChange {
    Primary(String),
    Position(PathBuf, DesktopPos),
    Sort(String, HeadingOptions, bool),
}

impl DesktopChange {
    pub fn retain_before(&self, newer: &Self) -> bool {
        //TODO: clean out more prior items?

        match (newer, self) {
            (Self::Primary(new_display), Self::Primary(display)) => {
                if display == new_display {
                    // Drop previous primary display changes with same display name
                    return false;
                }
            }
            (Self::Position(new_path, new_pos), Self::Position(path, pos)) => {
                if path == new_path && pos.display == new_pos.display {
                    // Drop previous position changes that have the same path and display name
                    return false;
                }
            }
            (Self::Sort(new_display, ..), Self::Sort(display, ..)) => {
                if display == new_display {
                    // Drop previous sort changes with same display name
                    return false;
                }
            }
            _ => {}
        }

        true
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DesktopPos {
    pub display: String,
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DesktopSize {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesktopLayout {
    pub config: DesktopConfig,
    pub display_names: Vec<String>,
    // Must be BTreeMap to implement Hash for usage inside Location. In the future, remove from Location
    pub display_sorts: BTreeMap<String, (HeadingOptions, bool)>,
    pub positions: BTreeMap<PathBuf, DesktopPos>,
    pub primary_display: Option<String>,
}

impl DesktopLayout {
    pub fn new(config: DesktopConfig) -> Self {
        DesktopLayout {
            config,
            display_names: Vec::new(),
            display_sorts: BTreeMap::new(),
            positions: BTreeMap::new(),
            primary_display: None,
        }
    }

    pub fn resize(&self, display: &str, size: DesktopSize) {
        eprintln!("{}: {:?}", display, size);
    }

    pub fn update(&mut self, changes: &[DesktopChange]) {
        self.display_sorts.clear();
        self.positions.clear();
        for change in changes {
            match change {
                DesktopChange::Position(path, pos) => {
                    //TODO: resize grid if rows or columns do not match
                    if self.display_names.contains(&pos.display) {
                        self.positions.insert(path.clone(), pos.clone());
                    }
                }
                DesktopChange::Primary(display) => {
                    if self.display_names.contains(&display) {
                        self.primary_display = Some(display.clone());
                    }
                }
                DesktopChange::Sort(display, heading, dir) => {
                    self.display_sorts.insert(display.clone(), (*heading, *dir));
                }
            }
        }
    }
}
