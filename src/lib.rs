// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{Application, Settings},
    cosmic_config::{self, CosmicConfigEntry},
};
use std::{
    path::PathBuf,
    process,
    sync::{Arc, Mutex},
};

use app::{App, Flags};
mod app;
use config::{Config, CONFIG_VERSION};
mod config;
pub mod dialog;
mod key_bind;
mod localize;
mod menu;
mod mime_icon;
mod mouse_area;
mod operation;
mod tab;

pub fn home_dir() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => home,
        None => {
            log::warn!("failed to locate home directory");
            PathBuf::from("/")
        }
    }
}

/// Runs application with these settings
pub fn dialog() -> Result<Option<Vec<PathBuf>>, Box<dyn std::error::Error>> {
    localize::localize();

    let mut settings = Settings::default();

    #[cfg(target_os = "redox")]
    {
        // Redox does not support resize if doing CSDs
        settings = settings.client_decorations(false);
    }

    let mut result_lock = Arc::new(Mutex::new(None));

    let flags = dialog::Flags {
        result_lock: result_lock.clone(),
    };
    cosmic::app::run::<dialog::App>(settings, flags)?;

    let mut result_guard = result_lock.lock().unwrap();
    let result = result_guard.take();
    Ok(result)
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

    let (config_handler, config) = match cosmic_config::Config::new(App::APP_ID, CONFIG_VERSION) {
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
    };

    let mut settings = Settings::default();
    settings = settings.theme(config.app_theme.theme());

    #[cfg(target_os = "redox")]
    {
        // Redox does not support resize if doing CSDs
        settings = settings.client_decorations(false);
    }

    //TODO: allow size limits on iced_winit
    //settings = settings.size_limits(Limits::NONE.min_width(400.0).min_height(200.0));

    let flags = Flags {
        config_handler,
        config,
    };
    cosmic::app::run::<App>(settings, flags)?;

    Ok(())
}
