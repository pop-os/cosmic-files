// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mime_app;
use crate::spawn_detached::spawn_detached;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ContextActionSelection {
    #[default]
    #[serde(alias = "any")]
    Any,
    #[serde(alias = "files")]
    Files,
    #[serde(alias = "folders")]
    Folders,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct ContextActionPreset {
    pub name: String,
    pub confirm: bool,
    pub selection: ContextActionSelection,
    /// File name extensions (without the leading dot, case-insensitive) this
    /// action applies to. An empty list matches every file.
    pub extensions: Vec<String>,
    pub steps: Vec<String>,
}

impl ContextActionPreset {
    pub fn matches_selection(&self, selected: usize, selected_dir: usize, names: &[&str]) -> bool {
        if selected == 0 {
            return false;
        }

        match self.selection {
            ContextActionSelection::Any => (),
            ContextActionSelection::Files => {
                if selected_dir != 0 {
                    return false;
                }
            }
            ContextActionSelection::Folders => {
                if selected_dir != selected {
                    return false;
                }
            }
        }

        self.matches_extensions(names)
    }

    /// The action is only offered when every selected file has one of the
    /// configured extensions. An empty extension list matches any selection.
    fn matches_extensions(&self, names: &[&str]) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        names.iter().all(|name| {
            Path::new(name)
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|ext| {
                    self.extensions
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(ext))
                })
        })
    }

    pub fn run(&self, paths: &[PathBuf]) {
        if self.steps.is_empty() {
            log::warn!("context action {:?} has no steps", self.name);
            return;
        }

        for step in &self.steps {
            let Some(commands) = mime_app::exec_to_command(step, &self.name, None, paths) else {
                log::warn!(
                    "failed to parse context action {:?}: invalid Exec {:?}",
                    self.name,
                    step
                );
                return;
            };

            for mut command in commands {
                if let Err(err) = spawn_detached(&mut command) {
                    log::warn!(
                        "failed to run context action {:?} step {:?}: {}",
                        self.name,
                        step,
                        err
                    );
                    return;
                }
            }
        }
    }
}

pub fn run(actions: &[ContextActionPreset], action: usize, paths: &[PathBuf]) {
    if let Some(preset) = actions.get(action) {
        preset.run(paths);
    } else {
        log::warn!("invalid context action index `{action}`");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset(selection: ContextActionSelection, extensions: &[&str]) -> ContextActionPreset {
        ContextActionPreset {
            name: "test".to_string(),
            confirm: false,
            selection,
            extensions: extensions.iter().map(|s| (*s).to_string()).collect(),
            steps: vec![],
        }
    }

    #[test]
    fn empty_extensions_match_any_file() {
        let preset = preset(ContextActionSelection::Files, &[]);
        assert!(preset.matches_selection(1, 0, &["archive.zip"]));
        assert!(preset.matches_selection(1, 0, &["no-extension"]));
    }

    #[test]
    fn extensions_match_case_insensitively() {
        let preset = preset(ContextActionSelection::Files, &["avif"]);
        assert!(preset.matches_selection(1, 0, &["photo.avif"]));
        assert!(preset.matches_selection(1, 0, &["photo.AVIF"]));
        assert!(preset.matches_selection(1, 0, &["photo.AvIf"]));
        assert!(!preset.matches_selection(1, 0, &["photo.png"]));
        assert!(!preset.matches_selection(1, 0, &["no-extension"]));
    }

    #[test]
    fn extensions_can_list_multiple_values() {
        let preset = preset(ContextActionSelection::Files, &["zip", "7z"]);
        assert!(preset.matches_selection(1, 0, &["archive.zip"]));
        assert!(preset.matches_selection(1, 0, &["archive.7z"]));
        assert!(!preset.matches_selection(1, 0, &["archive.rar"]));
    }

    #[test]
    fn every_selected_file_must_match() {
        let preset = preset(ContextActionSelection::Any, &["avif"]);
        assert!(preset.matches_selection(2, 0, &["a.avif", "b.avif"]));
        assert!(!preset.matches_selection(2, 0, &["a.avif", "b.png"]));
    }

    #[test]
    fn files_selection_rejects_folders() {
        let preset = preset(ContextActionSelection::Files, &["zip"]);
        assert!(!preset.matches_selection(1, 1, &["folder.zip"]));
        assert!(!preset.matches_selection(2, 1, &["a.zip", "b"]));
    }

    #[test]
    fn folders_selection_accepts_only_folders() {
        let preset = preset(ContextActionSelection::Folders, &[]);
        assert!(preset.matches_selection(2, 2, &["a", "b"]));
        assert!(!preset.matches_selection(2, 1, &["a", "b"]));
    }

    #[test]
    fn no_selection_matches_nothing() {
        let preset = preset(ContextActionSelection::Any, &[]);
        assert!(!preset.matches_selection(0, 0, &[]));
    }
}
