// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{app::Settings, iced::Limits};
use std::{env, fs, path::PathBuf, process};

use app::{App, Flags};
pub mod app;
pub mod clipboard;
use config::Config;
pub mod config;
pub mod dialog;
mod key_bind;
mod localize;
mod menu;
mod mime_app;
pub mod mime_icon;
mod mounter;
mod mouse_area;
mod operation;
mod spawn_detached;
use tab::Location;
pub mod tab;
mod thumbnailer;

pub(crate) fn err_str<T: ToString>(err: T) -> String {
    err.to_string()
}

pub fn desktop_dir() -> PathBuf {
    match dirs::desktop_dir() {
        Some(path) => path,
        None => {
            let path = home_dir().join("Desktop");
            log::warn!("failed to locate desktop directory, falling back to {path:?}");
            path
        }
    }
}

pub fn home_dir() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => home,
        None => {
            let path = PathBuf::from("/");
            log::warn!("failed to locate home directory, falling back to {path:?}");
            path
        }
    }
}

/// Runs application in desktop mode
#[rustfmt::skip]
pub fn desktop() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();

    let (config_handler, config) = Config::load();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);
    settings = settings.transparent(true);
    #[cfg(feature = "wayland")]
    {
        settings = settings.no_main_window(true);
    }

    let flags = Flags {
        config_handler,
        config,
        mode: app::Mode::Desktop,
        locations: vec![tab::Location::Path(desktop_dir())],
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}

/// Runs application with these settings
#[rustfmt::skip]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(unix, not(target_os = "redox")))]
    match fork::daemon(true, true) {
        Ok(fork::Fork::Child) => (),
        Ok(fork::Fork::Parent(_child_pid)) => process::exit(0),
        Err(err) => {
            eprintln!("failed to daemonize: {:?}", err);
            process::exit(1);
        }
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();

    let (config_handler, config) = Config::load();

    let mut locations = Vec::new();
    for arg in env::args().skip(1) {
        let location = if &arg == "--trash" {
            Location::Trash
        } else {
            match fs::canonicalize(&arg) {
                Ok(absolute) => Location::Path(absolute),
                Err(err) => {
                    log::warn!("failed to canonicalize {:?}: {}", arg, err);
                    continue;
                }
            }
        };
        locations.push(location);
    }

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);

    let flags = Flags {
        config_handler,
        config,
        mode: app::Mode::App,
        locations,
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}
