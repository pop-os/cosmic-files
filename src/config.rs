// SPDX-License-Identifier: GPL-3.0-only

use std::{any::TypeId, num::NonZeroU16, path::PathBuf};

use cosmic::{
    Application,
    cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    iced::Subscription,
    theme,
};
use serde::{Deserialize, Serialize};

use crate::{
    FxOrderMap,
    app::App,
    tab::{HeadingOptions, Location, View},
};

pub const CONFIG_VERSION: u64 = 1;

// Default icon sizes
pub const ICON_SIZE_LIST: u16 = 32;
pub const ICON_SIZE_LIST_CONDENSED: u16 = 48;
pub const ICON_SIZE_GRID: u16 = 64;
// TODO: 5 is an arbitrary number. Maybe there's a better icon size max
pub const ICON_SCALE_MAX: u16 = 5;

macro_rules! percent {
    ($perc:expr, $pixel:ident) => {
        (($perc.get() as f32 * $pixel as f32) / 100.).clamp(1., ($pixel * ICON_SCALE_MAX) as _)
    };
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AppTheme {
    Dark,
    Light,
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => {
                let mut t = theme::system_dark();
                t.theme_type.prefer_dark(Some(true));
                t
            }
            Self::Light => {
                let mut t = theme::system_light();
                t.theme_type.prefer_dark(Some(false));
                t
            }
            Self::System => theme::system_preference(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Favorite {
    Home,
    Documents,
    Downloads,
    Music,
    Pictures,
    Videos,
    Path(PathBuf),
    Network {
        uri: String,
        name: String,
        path: PathBuf,
    },
}

impl Favorite {
    pub fn from_path(path: PathBuf) -> Self {
        // Ensure that special folders are handled properly
        [
            Self::Home,
            Self::Documents,
            Self::Downloads,
            Self::Music,
            Self::Pictures,
            Self::Videos,
        ]
        .into_iter()
        .find(|fav| fav.path_opt().as_ref() == Some(&path))
        .unwrap_or(Self::Path(path))
    }

    pub fn path_opt(&self) -> Option<PathBuf> {
        match self {
            Self::Home => dirs::home_dir(),
            Self::Documents => dirs::document_dir(),
            Self::Downloads => dirs::download_dir(),
            Self::Music => dirs::audio_dir(),
            Self::Pictures => dirs::picture_dir(),
            Self::Videos => dirs::video_dir(),
            Self::Path(path) => Some(path.clone()),
            Self::Network { path, .. } => Some(path.clone()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum TypeToSearch {
    Recursive,
    EnterPath,
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct State {
    pub sort_names: FxOrderMap<String, (HeadingOptions, bool)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sort_names: FxOrderMap::from_iter(dirs::download_dir().into_iter().map(|dir| {
                (
                    Location::Path(dir).normalize().to_string(),
                    (HeadingOptions::Modified, false),
                )
            })),
        }
    }
}

impl State {
    pub fn load() -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new_state(App::APP_ID, CONFIG_VERSION) {
            Ok(config_handler) => {
                let config = match Self::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        log::info!("errors loading config: {errs:?}");
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                log::error!("failed to create config handler: {err}");
                (None, Self::default())
            }
        }
    }

    pub fn subscription() -> Subscription<cosmic_config::Update<Self>> {
        struct ConfigSubscription;
        cosmic_config::config_state_subscription(
            TypeId::of::<ConfigSubscription>(),
            App::APP_ID.into(),
            CONFIG_VERSION,
        )
    }
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
    pub app_theme: AppTheme,
    pub dialog: DialogConfig,
    pub desktop: DesktopConfig,
    pub thumb_cfg: ThumbCfg,
    pub favorites: Vec<Favorite>,
    pub show_details: bool,
    pub tab: TabConfig,
    pub type_to_search: TypeToSearch,
}

impl Config {
    pub fn load() -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
            Ok(config_handler) => {
                let config = match Self::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        log::info!("errors loading config: {errs:?}");
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                log::error!("failed to create config handler: {err}");
                (None, Self::default())
            }
        }
    }

    pub fn subscription() -> Subscription<cosmic_config::Update<Self>> {
        struct ConfigSubscription;
        cosmic_config::config_subscription(
            TypeId::of::<ConfigSubscription>(),
            App::APP_ID.into(),
            CONFIG_VERSION,
        )
    }

    /// Construct tab config for dialog
    pub const fn dialog_tab(&self) -> TabConfig {
        TabConfig {
            folders_first: self.dialog.folders_first,
            icon_sizes: self.dialog.icon_sizes,
            military_time: self.tab.military_time,
            show_hidden: self.dialog.show_hidden,
            single_click: false,
            view: self.dialog.view,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            desktop: DesktopConfig::default(),
            dialog: DialogConfig::default(),
            thumb_cfg: ThumbCfg::default(),
            favorites: vec![
                Favorite::Home,
                Favorite::Documents,
                Favorite::Downloads,
                Favorite::Music,
                Favorite::Pictures,
                Favorite::Videos,
            ],
            show_details: false,
            tab: TabConfig::default(),
            type_to_search: TypeToSearch::Recursive,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct DesktopConfig {
    pub grid_spacing: NonZeroU16,
    pub icon_size: NonZeroU16,
    pub show_content: bool,
    pub show_mounted_drives: bool,
    pub show_trash: bool,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            grid_spacing: 100.try_into().unwrap(),
            icon_size: 100.try_into().unwrap(),
            show_content: true,
            show_mounted_drives: false,
            show_trash: false,
        }
    }
}

impl DesktopConfig {
    pub fn grid_spacing_for(&self, space: u16) -> u16 {
        percent!(self.grid_spacing, space) as _
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct DialogConfig {
    /// Show folders before files
    pub folders_first: bool,
    /// Icon zoom
    pub icon_sizes: IconSizes,
    /// Show details sidebar
    pub show_details: bool,
    /// Show hidden files and folders
    pub show_hidden: bool,
    /// Selected view, grid or list
    pub view: View,
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            folders_first: false,
            icon_sizes: IconSizes::default(),
            show_details: true,
            show_hidden: false,
            view: View::List,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct ThumbCfg {
    pub jobs: NonZeroU16,
    pub max_mem_mb: NonZeroU16,
    pub max_size_mb: NonZeroU16,
}

impl Default for ThumbCfg {
    fn default() -> Self {
        Self {
            jobs: 4.try_into().unwrap(),
            max_mem_mb: 2000.try_into().unwrap(),
            max_size_mb: 64.try_into().unwrap(),
        }
    }
}

/// Global and local [`crate::tab::Tab`] config.
///
/// [`TabConfig`] contains options that are passed to each instance of [`crate::tab::Tab`].
/// These options are set globally through the main config, but each tab may change options
/// locally. Local changes aren't saved to the main config.
#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct TabConfig {
    /// Show folders before files
    pub folders_first: bool,
    /// Icon zoom
    pub icon_sizes: IconSizes,
    #[serde(skip)]
    /// 24 hour clock; this is neither serialized nor deserialized because we use the user's global
    /// preference rather than save it
    pub military_time: bool,
    /// Show hidden files and folders
    pub show_hidden: bool,
    /// Single click to open
    pub single_click: bool,
    /// Selected view, grid or list
    pub view: View,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            folders_first: true,
            icon_sizes: IconSizes::default(),
            military_time: false,
            show_hidden: false,
            single_click: false,
            view: View::List,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
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

pub const TIME_CONFIG_ID: &str = "com.system76.CosmicAppletTime";

#[derive(Debug, Default, Clone, CosmicConfigEntry, PartialEq, Eq)]
#[version = 1]
pub struct TimeConfig {
    pub military_time: bool,
}
