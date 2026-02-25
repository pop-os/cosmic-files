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

        match (self, newer) {
            (Self::Primary(output_name), Self::Primary(new_output_name)) => {
                if output_name == new_output_name {
                    // Drop previous primary output changes with same output name
                    return false;
                }
            }
            (Self::Position(path, pos), Self::Position(new_path, new_pos)) => {
                if path == new_path && pos.display == new_pos.display {
                    // Drop previous position changes that have the same path and output
                    return false;
                }
            }
            (
                Self::Sort(output_name, ..),
                Self::Position(
                    _,
                    DesktopPos {
                        display: new_output_name,
                        ..
                    },
                )
                | Self::Sort(new_output_name, ..),
            ) => {
                if output_name == new_output_name {
                    // Drop previous position and sort changes with same output name
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
    pub rows: usize,
    pub cols: usize,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DesktopLayout {
    pub config: DesktopConfig,
    pub output_names: Vec<String>,
    // Must be BTreeMap to implement Hash for usage inside Location. In the future, remove from Location
    pub positions: BTreeMap<PathBuf, DesktopPos>,
    pub primary_output_name: Option<String>,
}

impl DesktopLayout {
    pub fn new(config: DesktopConfig) -> Self {
        DesktopLayout {
            config,
            output_names: Vec::new(),
            positions: BTreeMap::new(),
            primary_output_name: None,
        }
    }

    pub fn update_positions(&mut self, changes: &[DesktopChange]) {
        self.positions.clear();
        for change in changes {
            match change {
                DesktopChange::Position(path, pos) => {
                    //TODO: resize grid if rows or columns do not match
                    if self.output_names.contains(&pos.display) {
                        self.positions.insert(path.clone(), pos.clone());
                    }
                }
                DesktopChange::Primary(output_name) => {
                    if self.output_names.contains(&output_name) {
                        self.primary_output_name = Some(output_name.clone());
                    }
                }
                DesktopChange::Sort(..) => {
                    //TODO: what should sort do?
                }
            }
        }
    }
}
