// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Config {
    pub app_theme: AppTheme,
    pub tab: Tab,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            tab: Tab::default(),
        }
    }
}

/// Per tab config
#[derive(Clone, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
pub struct Tab {
    /// Show hidden files
    pub show_hidden: bool,
    // TODO: Other possible options
    // pub sort_by: fn(&PathBuf, &PathBuf) -> Ordering,
    // Icon handle sizes
    // icon_size_dialog: u16,
    // icon_size_list: u16,
    // icon_size_grid: u16,
}

impl Default for Tab {
    fn default() -> Self {
        Self { show_hidden: false }
    }
}
