// SPDX-License-Identifier: GPL-3.0-only

use std::num::NonZeroU16;

use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

use crate::tab::HeadingOptions;

pub const CONFIG_VERSION: u64 = 1;

// Default icon sizes
const ICON_SIZE_LIST: u16 = 24;
const ICON_SIZE_LIST_CONDENSED: u16 = 48;
const ICON_SIZE_GRID: u16 = 64;
// TODO: 5 is an arbitrary number. Maybe there's a better icon size max
const ICON_SCALE_MAX: u16 = 5;

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
#[derive(Clone, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
pub struct TabConfig {
    /// Show hidden files and folders
    pub show_hidden: bool,
    // TODO: Other possible options
    // pub sort_by: fn(&PathBuf, &PathBuf) -> Ordering,
    // Icon zoom
    pub icon_sizes: IconSizes,
    pub visible_columns: Vec<VisibleColumns>,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            show_hidden: false,
            icon_sizes: IconSizes::default(),
            visible_columns: VisibleColumns::default(),
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct VisibleColumns {
    pub active: bool,
    pub heading: HeadingOptions,
}
impl VisibleColumns {
    pub fn new(active: bool, heading: HeadingOptions) -> Self {
        Self { active, heading }
    }
    fn default() -> Vec<Self> {
        vec![
            VisibleColumns::new(true, HeadingOptions::Name),
            VisibleColumns::new(true, HeadingOptions::Modified),
            VisibleColumns::new(false, HeadingOptions::Accessed),
            VisibleColumns::new(false, HeadingOptions::Created),
            VisibleColumns::new(true, HeadingOptions::Size),
        ]
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
