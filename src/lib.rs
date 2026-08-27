// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::Settings;
use cosmic::iced::Limits;
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::{Path, PathBuf};
use std::{env, fs, process};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::{App, Flags};
use crate::config::{Config, State};
use crate::tab::Location;

pub mod app;
mod archive;
pub mod channel;
pub mod clipboard;
pub mod config;
mod context_action;
pub mod dialog;
mod exe_thumbnailer;
mod key_bind;
pub(crate) mod large_image;
pub(crate) mod load_image;
mod localize;
mod menu;
mod mime_app;
pub mod mime_icon;
mod mounter;
mod mouse_area;
pub mod operation;
mod spawn_detached;
pub mod tab;
mod thumbnail_cacher;
mod thumbnailer;
pub(crate) mod trash;
mod zoom;

pub(crate) type FxOrderMap<K, V> = ordermap::OrderMap<K, V, rustc_hash::FxBuildHasher>;

pub(crate) fn err_str<T: ToString>(err: T) -> String {
    err.to_string()
}

fn thumbnail_request(
    mut args: impl Iterator<Item = OsString>,
) -> Option<Result<(), Box<dyn std::error::Error>>> {
    let mode = args.next()?;
    if mode != OsStr::new("--thumbnail-exe") {
        return None;
    }

    Some((|| {
        let usage = || {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: cosmic-files --thumbnail-exe OUTPUT --size SIZE INPUT",
            )
        };
        let output = args.next().ok_or_else(usage)?;
        if args.next().as_deref() != Some(OsStr::new("--size")) {
            return Err(usage().into());
        }
        let size = args
            .next()
            .and_then(|arg| arg.into_string().ok())
            .and_then(|arg| arg.parse::<u32>().ok())
            .filter(|size| *size > 0)
            .ok_or_else(usage)?;
        let input = args.next().ok_or_else(usage)?;
        if args.next().is_some() {
            return Err(usage().into());
        }

        exe_thumbnailer::thumbnail(Path::new(&input), Path::new(&output), size)
    })())
}

#[cfg(test)]
mod tests {
    use super::thumbnail_request;
    use std::ffi::OsString;

    #[test]
    fn normal_arguments_are_not_thumbnail_requests() {
        let args = [OsString::from("/tmp/file.exe")];
        assert!(thumbnail_request(args.into_iter()).is_none());
    }

    #[test]
    fn malformed_thumbnail_request_returns_an_error() {
        let args = [OsString::from("--thumbnail-exe")];
        assert!(thumbnail_request(args.into_iter()).unwrap().is_err());
    }
}

pub fn desktop_dir() -> PathBuf {
    if let Some(path) = dirs::desktop_dir() {
        path
    } else {
        let path = home_dir().join("Desktop");
        log::warn!(
            "failed to locate desktop directory, falling back to {}",
            path.display()
        );
        path
    }
}

pub fn home_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home
    } else {
        let path = PathBuf::from("/");
        log::warn!(
            "failed to locate home directory, falling back to {}",
            path.display()
        );
        path
    }
}

pub fn is_wayland() -> bool {
    matches!(
        cosmic::app::cosmic::windowing_system(),
        Some(cosmic::app::cosmic::WindowingSystem::Wayland)
    )
}

/// Runs application in desktop mode
#[rustfmt::skip]
pub fn desktop() -> Result<(), Box<dyn std::error::Error>> {
    let log_format = tracing_subscriber::fmt::format()
        .pretty()
        .without_time()
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .with_thread_names(true);

    let log_layer = tracing_subscriber::fmt::Layer::default()
        .with_writer(std::io::stderr)
        .event_format(log_format);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .with(log_layer)
        .init();

    localize::localize();

    let (config_handler, config) = Config::load();
    let (state_handler, state) = State::load();

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);
    settings = settings.transparent(true);
    #[cfg(all(feature = "wayland", feature = "desktop-applet"))]
    {
        settings = settings.no_main_window(true);
    }

    let locations = vec![tab::Location::Desktop(desktop_dir(), String::new(), config.desktop)];
    let flags = Flags {
        config_handler,
        config,
        state_handler,
        state,
        mode: app::Mode::Desktop,
        locations,
        uris: Vec::new()
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}

/// Runs application with these settings
#[rustfmt::skip]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(result) = thumbnail_request(env::args_os().skip(1)) {
        return result;
    }
    if let Ok(executable) = env::current_exe() {
        thumbnailer::register_bundled_exe_thumbnailer(executable);
    }

    let log_format = tracing_subscriber::fmt::format()
        .pretty()
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .with_thread_names(true);

    let log_layer = tracing_subscriber::fmt::Layer::default()
        .with_writer(std::io::stderr)
        .event_format(log_format);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(log_layer)
        .init();

    localize::localize();

    let (config_handler, config) = Config::load();
    let (state_handler, state) = State::load();

    let mut daemonize = true;
    let mut locations = Vec::new();
    let mut uris = Vec::new();
    for arg in env::args().skip(1) {
        let location = if &arg == "--no-daemon" {
            daemonize = false;
            continue;
        } else if &arg == "--trash" {
            Location::Trash
        } else if &arg == "--recents" {
            if config.show_recents {
                Location::Recents
            } else {
                log::warn!("recents feature is disabled in config");
                continue;
            }
        } else if &arg == "--network" {
            Location::Network("network:///".to_string(), fl!("networks"), None)
        } else {
            //TODO: support more URLs
            let path = match url::Url::parse(&arg) {
                Ok(url) if url.scheme() == "file" => if let Ok(path) = url.to_file_path() { path } else {
                    log::warn!("invalid argument {arg:?}");
                    continue;
                },
                Ok(url) => {
                    uris.push(url);
                    continue;
                }
                _ => PathBuf::from(arg),
            };
            match fs::canonicalize(&path) {
                Ok(absolute) => Location::Path(absolute),
                Err(err) => {
                    log::warn!("failed to canonicalize {}: {}", path.display(), err);
                    continue;
                }
            }
        };
        locations.push(location);
    }

    if daemonize {
        #[cfg(all(unix, not(any(target_os = "macos", target_os = "redox"))))]
        match fork::daemon(true, true) {
            Ok(fork::Fork::Child) => (),
            Ok(fork::Fork::Parent(_child_pid)) => process::exit(0),
            Err(err) => {
                eprintln!("failed to daemonize: {err:?}");
                process::exit(1);
            }
        }
    }

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());
    settings = settings.size_limits(Limits::NONE.min_width(360.0).min_height(180.0));
    settings = settings.exit_on_close(false);

    #[cfg(feature = "jemalloc")]
    {
        settings = settings.default_mmap_threshold(None);
    }

    let flags = Flags {
        config_handler,
        config,
        state_handler,
        state,
        mode: app::Mode::App,
        locations,
        uris
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}
