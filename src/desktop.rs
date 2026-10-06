use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config::DesktopConfig;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DesktopChange {
    Primary(String),
    Position(PathBuf, DesktopPos),
}

impl DesktopChange {
    pub fn retain_before(&self, newer: &Self) -> bool {
        match (newer, self) {
            (Self::Primary(new_display), Self::Primary(display)) => {
                // Drop previous primary display changes with same display name
                if display == new_display {
                    return false;
                }
            }
            (Self::Position(new_path, new_pos), Self::Position(path, pos)) => {
                // Drop previous position changes that have the same display and either the same path or position
                if pos.display == new_pos.display && (path == new_path || pos == new_pos) {
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
    pub page: usize,
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DesktopPaste {
    pub pos: DesktopPos,
    pub rows: usize,
    pub cols: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesktopLayout {
    pub config: DesktopConfig,
    pub display_names: Vec<String>,
    // Must be BTreeMap to implement Hash for usage inside Location. In the future, remove from Location
    pub positions: BTreeMap<PathBuf, DesktopPos>,
    pub primary_display: Option<String>,
}

impl DesktopLayout {
    pub fn new(config: DesktopConfig) -> Self {
        DesktopLayout {
            config,
            display_names: Vec::new(),
            positions: BTreeMap::new(),
            primary_display: None,
        }
    }

    pub fn update(&mut self, changes: &[DesktopChange]) {
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
            }
        }
    }
}
