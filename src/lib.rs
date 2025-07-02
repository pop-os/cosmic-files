// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{app::Settings, iced::Limits};
use std::{env, fs, path::PathBuf, process, error::Error};

use clap_lex::RawArgs;


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
pub mod operation;
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

    let (config_handler, config_data) = Config::load();

    let mut settings = Settings::default();
    settings = settings.theme(config_data.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);
    settings = settings.transparent(true);
    #[cfg(all(feature = "wayland", feature = "desktop-applet"))]
    {
        settings = settings.no_main_window(true);
    }

    let locations = vec![tab::Location::Desktop(desktop_dir(), String::new(), config_data.desktop)];
    let flags = Flags {
        config_handler,
        config: config_data,
        mode: app::Mode::Desktop,
        locations,
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}

/// Runs application with these settings
#[rustfmt::skip]
pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    localize::localize();

    // Parse the arguments
    let raw_args = RawArgs::from_args();
    let mut cursor = raw_args.cursor();

    let mut daemonize = true;
    let mut locations = Vec::new();

    while let Some(arg) = raw_args.next_os(&mut cursor) {
        match arg.to_str() {
            Some("--help") | Some("-h") => {
                print_help();
                process::exit(0);
            }
            Some("--version") | Some("-V") => {
                println!(
                    "cosmic-files {} (git commit {})",
                    env!("CARGO_PKG_VERSION"),
                    env!("VERGEN_GIT_SHA")
                );
                process::exit(0);
            }
            Some("--no-daemon") => {
                daemonize = false;
            }
            Some("--trash") => locations.push(tab::Location::Trash),
            Some("--recents") => locations.push(tab::Location::Recents),
            Some("--network") => locations.push(tab::Location::Network("network:///".to_string(), fl!("networks"))),
            Some(other) => {
                // Support URLs and +
                let path = match url::Url::parse(other) {
                    Ok(url) => match url.to_file_path() {
                        Ok(path) => path,
                        Err(()) => {
                            log::warn!("invalid argument {:?}", other);
                            continue;
                        }
                    },
                    Err(_) => PathBuf::from(other),
                };
                match std::fs::canonicalize(&path) {
                    Ok(absolute) => locations.push(tab::Location::Path(absolute)),
                    Err(err) => {
                        log::warn!("failed to canonicalize {:?}: {}", path, err);
                    }
                }
            }
            None => {
                log::warn!("Invalid argument encountered");
            }
        }
    }

    if daemonize {
        #[cfg(all(unix, not(target_os = "redox")))]
        match fork::daemon(true, true) {
            Ok(fork::Fork::Child) => (),
            Ok(fork::Fork::Parent(_child_pid)) => process::exit(0),
            Err(err) => {
                eprintln!("failed to daemonize: {:?}", err);
                process::exit(1);
            }
        }
    }
    
    let (config_handler, config_data) = Config::load();

    let mut settings = Settings::default();
    settings = settings.theme(config_data.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);

    #[cfg(feature = "jemalloc")]
    {
        settings = settings.default_mmap_threshold(None);
    }

    let flags = Flags {
        config_handler,
        config: config_data,
        mode: app::Mode::App,
        locations,
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}

fn print_help() {
    println!(
        r#"COSMIC Files
Designed for the COSMIC™ desktop environment, cosmic-files is a libcosmic-based file manager.
	    
Project home page: https://github.com/pop-os/cosmic-files
	    
Options:
  -h, --help       Show this message
  -v, --version    Show the version of cosmic-files"#
    );
}
