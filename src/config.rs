// SPDX-License-Identifier: GPL-3.0-only

use std::num::NonZeroU16;

use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

use super::tab::HeadingOptions;

pub const CONFIG_VERSION: u64 = 1;

// Default icon sizes
pub const ICON_SIZE_LIST: u16 = 24;
pub const ICON_SIZE_LIST_CONDENSED: u16 = 48;
pub const ICON_SIZE_GRID: u16 = 64;
// TODO: 5 is an arbitrary number. Maybe there's a better icon size max
pub const ICON_SCALE_MAX: u16 = 5;

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
    pub tab: TabConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            tab: TabConfig::default(),
        }
    }
}

/// Global and local [`crate::tab::Tab`] config.
///
/// [`TabConfig`] contains options that are passed to each instance of [`crate::tab::Tab`].
/// These options are set globally through the main config, but each tab may change options
/// locally. Local changes aren't saved to the main config.
#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
pub struct TabConfig {
    /// Show hidden files and folders
    pub show_hidden: bool,
    /// Sorter
    pub sort_name: HeadingOptions,
    pub sort_direction: bool,
    /// Icon zoom
    pub icon_sizes: IconSizes,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_name: HeadingOptions::Name,
            sort_direction: true,
            icon_sizes: IconSizes::default(),
        }
    }
}

macro_rules! percent {
    ($perc:expr, $pixel:ident) => {
        (($perc.get() as f32 * $pixel as f32) / 100.).clamp(1., ($pixel * ICON_SCALE_MAX) as _)
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
pub struct IconSizes {
    pub list: NonZeroU16,
    pub grid: NonZeroU16,
}

impl Default for IconSizes {
    fn default() -> Self {
        Self {
            list: 100.try_into().unwrap(),
            grid: 100.try_into().unwrap(),
        }
    }
}

impl IconSizes {
    pub fn list(&self) -> u16 {
        percent!(self.list, ICON_SIZE_LIST) as _
    }

    pub fn list_condensed(&self) -> u16 {
        percent!(self.list, ICON_SIZE_LIST_CONDENSED) as _
    }

    pub fn grid(&self) -> u16 {
        percent!(self.grid, ICON_SIZE_GRID) as _
    }
}
