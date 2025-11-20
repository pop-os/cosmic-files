// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::{mime_app, spawn_detached::spawn_detached};

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
    pub steps: Vec<String>,
}

impl ContextActionPreset {
    pub fn matches_selection(&self, selected: usize, selected_dir: usize) -> bool {
        if selected == 0 {
            return false;
        }

        match self.selection {
            ContextActionSelection::Any => true,
            ContextActionSelection::Files => selected_dir == 0,
            ContextActionSelection::Folders => selected_dir == selected,
        }
    }

    pub fn run<O: AsRef<std::ffi::OsStr>>(&self, paths: &[O]) {
        if self.steps.is_empty() {
            log::warn!("context action {:?} has no steps", self.name);
            return;
        }

        for step in &self.steps {
            let Some(commands) = mime_app::exec_to_command(step, paths) else {
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

pub fn run<O: AsRef<std::ffi::OsStr>>(actions: &[ContextActionPreset], action: usize, paths: &[O]) {
    if let Some(preset) = actions.get(action) {
        preset.run(paths);
    } else {
        log::warn!("invalid context action index `{action}`");
    }
}
