// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use regex::Regex;
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
    pub extensions: Option<Vec<String>>,
    pub steps: Vec<String>,
}

impl ContextActionPreset {
    pub fn matches_selection(&self, selected: usize, selected_dir: usize, selected_paths: &[String]) -> bool {
        if selected == 0 {
            return false;
        }

        let selection_match = match self.selection {
            ContextActionSelection::Any => true,
            ContextActionSelection::Files => selected_dir == 0,
            ContextActionSelection::Folders => selected_dir == selected,
        };

        if !selection_match {
            return false;
        }

        if self.selection == ContextActionSelection::Folders {
            return true;
        }

        if let Some(allowed_exts) = &self.extensions {
            if selected_paths.is_empty() {
                return false;
            }

            let prepared_patterns: Vec<(String, Option<Regex>)> = allowed_exts
                .iter()
                .map(|ext| {
                    let ext_lower = ext.to_lowercase();
                    let regex = if ext_lower.chars().any(|c| "^$*+?()[]{}|".contains(c)) {
                        Regex::new(&ext_lower).ok()
                    } else {
                        None
                    };
                    (ext_lower, regex)
                })
                .collect();

            return selected_paths.iter().all(|path_str| {
                let path = std::path::Path::new(path_str);
                let filename_lower = path.file_name()
                    .and_then(|os_str| os_str.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                let ext_actual = path.extension()
                    .and_then(|os_str| os_str.to_str())
                    .map(|s| s.to_lowercase());

                prepared_patterns.iter().any(|(ext_lower, regex)| {
                    if let Some(re) = regex {
                        re.is_match(&filename_lower)
                    } else if ext_lower.is_empty() {
                        ext_actual.is_none()
                    } else {
                        ext_actual.as_ref() == Some(ext_lower)
                    }
                })
            });
        }
        true
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
