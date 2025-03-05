// SPDX-License-Identifier: GPL-3.0-only

use std::{any::TypeId, num::NonZeroU16, path::PathBuf};

use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, ConfigGet, CosmicConfigEntry},
    iced::Subscription,
    theme, Application,
};
use serde::{Deserialize, Serialize};

use crate::{app::App, tab::View};

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NavBarSize {
    Small,
    Medium,
    Large,
}

impl NavBarSize {
    pub fn size(&self) -> f32 {
        match self {
            NavBarSize::Small => 64.0,
            NavBarSize::Medium => 160.0,
            NavBarSize::Large => 280.0,
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
}

impl Favorite {
    pub fn from_path(path: PathBuf) -> Self {
        // Ensure that special folders are handled properly
        for favorite in &[
            Self::Home,
            Self::Documents,
            Self::Downloads,
            Self::Music,
            Self::Pictures,
            Self::Videos,
        ] {
            if let Some(favorite_path) = favorite.path_opt() {
                if favorite_path == path {
                    return favorite.clone();
                }
            }
        }
        Self::Path(path)
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
        }
    }
}

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub struct Config {
    pub app_theme: AppTheme,
    pub nav_bar_size: NavBarSize,
    pub desktop: DesktopConfig,
    pub favorites: Vec<Favorite>,
    pub show_details: bool,
    pub tab: TabConfig,
}

impl Config {
    pub fn load() -> (Option<cosmic_config::Config>, Self) {
        match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
            Ok(config_handler) => {
                let config = match Config::get_entry(&config_handler) {
                    Ok(ok) => ok,
                    Err((errs, config)) => {
                        log::info!("errors loading config: {:?}", errs);
                        config
                    }
                };
                (Some(config_handler), config)
            }
            Err(err) => {
                log::error!("failed to create config handler: {}", err);
                (None, Config::default())
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: AppTheme::System,
            nav_bar_size: NavBarSize::Large,
            desktop: DesktopConfig::default(),
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

/// Global and local [`crate::tab::Tab`] config.
///
/// [`TabConfig`] contains options that are passed to each instance of [`crate::tab::Tab`].
/// These options are set globally through the main config, but each tab may change options
/// locally. Local changes aren't saved to the main config.
#[derive(Clone, Copy, Debug, Eq, PartialEq, CosmicConfigEntry, Deserialize, Serialize)]
#[serde(default)]
pub struct TabConfig {
    pub view: View,
    /// Show folders before files
    pub folders_first: bool,
    /// Show hidden files and folders
    pub show_hidden: bool,
    /// Icon zoom
    pub icon_sizes: IconSizes,
    #[serde(skip, default = "military_time_enabled")]
    /// 24 hour clock; this is neither serialized nor deserialized because we use the user's global
    /// preference rather than save it
    pub military_time: bool,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            view: View::List,
            folders_first: true,
            show_hidden: false,
            icon_sizes: IconSizes::default(),
            military_time: military_time_enabled(),
        }
    }
}

/// Return whether the user enabled military time via the Time applet.
fn military_time_enabled() -> bool {
    // Borrowed from COSMIC Greeter
    match cosmic_config::Config::new("com.system76.CosmicAppletTime", 1) {
        Ok(config_handler) => config_handler.get("military_time").unwrap_or_default(),
        Err(err) => {
            log::error!(
                "failed to create CosmicAppletTime config handler: {:?}",
                err
            );
            false
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
