// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "desktop")]
use cosmic::desktop;
use cosmic::widget;
pub use mime_guess::Mime;
use once_cell::sync::Lazy;
use std::{
    cmp::Ordering, collections::HashMap, env, ffi::OsString, path::PathBuf, process, sync::Mutex,
    time::Instant,
};

pub fn exec_to_command(exec: &str, path_opt: Option<OsString>) -> Option<process::Command> {
    let args_vec: Vec<String> = shlex::split(exec)?;
    let mut args = args_vec.iter();
    let mut command = process::Command::new(args.next()?);
    for arg in args {
        if arg.starts_with('%') {
            match arg.as_str() {
                "%f" | "%F" | "%u" | "%U" => {
                    if let Some(path) = &path_opt {
                        command.arg(path);
                    }
                }
                _ => {
                    log::warn!("unsupported Exec code {:?} in {:?}", arg, exec);
                    return None;
                }
            }
        } else {
            command.arg(arg);
        }
    }
    Some(command)
}

#[derive(Clone, Debug)]
pub struct MimeApp {
    pub id: String,
    pub path: Option<PathBuf>,
    pub name: String,
    pub exec: Option<String>,
    pub icon: widget::icon::Handle,
    pub is_default: bool,
}

impl MimeApp {
    //TODO: move to libcosmic, support multiple files
    pub fn command(&self, path_opt: Option<OsString>) -> Option<process::Command> {
        exec_to_command(self.exec.as_deref()?, path_opt)
    }
}

#[cfg(feature = "desktop")]
impl From<&desktop::DesktopEntryData> for MimeApp {
    fn from(app: &desktop::DesktopEntryData) -> Self {
        Self {
            id: app.id.clone(),
            path: app.path.clone(),
            name: app.name.clone(),
            exec: app.exec.clone(),
            icon: match &app.icon {
                desktop::IconSource::Name(name) => {
                    widget::icon::from_name(name.as_str()).size(32).handle()
                }
                desktop::IconSource::Path(path) => widget::icon::from_path(path.clone()),
            },
            is_default: false,
        }
    }
}

#[cfg(feature = "desktop")]
fn filename_eq(path_opt: &Option<PathBuf>, filename: &str) -> bool {
    path_opt
        .as_ref()
        .and_then(|path| path.file_name())
        .map(|x| x == filename)
        .unwrap_or(false)
}

pub struct MimeAppCache {
    cache: HashMap<Mime, Vec<MimeApp>>,
    terminals: Vec<MimeApp>,
}

impl MimeAppCache {
    pub fn new() -> Self {
        let mut mime_app_cache = Self {
            cache: HashMap::new(),
            terminals: Vec::new(),
        };
        mime_app_cache.reload();
        mime_app_cache
    }

    #[cfg(not(feature = "desktop"))]
    pub fn reload(&mut self) {}

    // Only available when using desktop feature of libcosmic, which only works on Unix-likes
    #[cfg(feature = "desktop")]
    pub fn reload(&mut self) {
        use crate::localize::LANGUAGE_SORTER;

        let start = Instant::now();

        self.cache.clear();
        self.terminals.clear();

        //TODO: get proper locale?
        let locale = None;

        // Load desktop applications by supported mime types
        //TODO: hashmap for all apps by id?
        let all_apps = desktop::load_applications(locale, false);
        for app in all_apps.iter() {
            for mime in app.mime_types.iter() {
                let apps = self
                    .cache
                    .entry(mime.clone())
                    .or_insert_with(|| Vec::with_capacity(1));
                if apps.iter().find(|x| x.id == app.id).is_none() {
                    apps.push(MimeApp::from(app));
                }
            }
            for category in app.categories.iter() {
                if category == "TerminalEmulator" {
                    self.terminals.push(MimeApp::from(app));
                    break;
                }
            }
        }

        let desktops: Vec<String> = env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .split(':')
            .map(|x| x.to_ascii_lowercase())
            .collect();

        // Load mimeapps.list files
        // https://specifications.freedesktop.org/mime-apps-spec/mime-apps-spec-latest.html
        //TODO: ensure correct lookup order
        let mut mimeapps_paths = Vec::new();
        match xdg::BaseDirectories::new() {
            Ok(xdg_dirs) => {
                for path in xdg_dirs.find_data_files("applications/mimeapps.list") {
                    mimeapps_paths.push(path);
                }
                for desktop in desktops.iter().rev() {
                    for path in
                        xdg_dirs.find_data_files(format!("applications/{desktop}-mimeapps.list"))
                    {
                        mimeapps_paths.push(path);
                    }
                }
                for path in xdg_dirs.find_config_files("mimeapps.list") {
                    mimeapps_paths.push(path);
                }
                for desktop in desktops.iter().rev() {
                    for path in xdg_dirs.find_config_files(format!("{desktop}-mimeapps.list")) {
                        mimeapps_paths.push(path);
                    }
                }
            }
            Err(err) => {
                log::warn!("failed to get xdg base directories: {}", err);
            }
        }

        //TODO: handle directory specific behavior
        for path in mimeapps_paths {
            let entry = match freedesktop_entry_parser::parse_entry(&path) {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!("failed to parse {:?}: {}", path, err);
                    continue;
                }
            };

            for attr in entry
                .section("Added Associations")
                .attrs()
                .chain(entry.section("Default Applications").attrs())
            {
                if let Ok(mime) = attr.name.parse::<Mime>() {
                    if let Some(filenames) = attr.value {
                        for filename in filenames.split_terminator(';') {
                            log::trace!("add {}={}", mime, filename);
                            let apps = self
                                .cache
                                .entry(mime.clone())
                                .or_insert_with(|| Vec::with_capacity(1));
                            if apps
                                .iter()
                                .find(|x| filename_eq(&x.path, filename))
                                .is_none()
                            {
                                if let Some(app) =
                                    all_apps.iter().find(|x| filename_eq(&x.path, filename))
                                {
                                    apps.push(MimeApp::from(app));
                                } else {
                                    log::debug!("failed to add association for {:?}: application {:?} not found", mime, filename);
                                }
                            }
                        }
                    }
                }
            }

            for attr in entry.section("Removed Associations").attrs() {
                if let Ok(mime) = attr.name.parse::<Mime>() {
                    if let Some(filenames) = attr.value {
                        for filename in filenames.split_terminator(';') {
                            log::trace!("remove {}={}", mime, filename);
                            if let Some(apps) = self.cache.get_mut(&mime) {
                                apps.retain(|x| !filename_eq(&x.path, filename));
                            }
                        }
                    }
                }
            }

            for attr in entry.section("Default Applications").attrs() {
                if let Ok(mime) = attr.name.parse::<Mime>() {
                    if let Some(filenames) = attr.value {
                        for filename in filenames.split_terminator(';') {
                            log::trace!("default {}={}", mime, filename);
                            if let Some(apps) = self.cache.get_mut(&mime) {
                                let mut found = false;
                                for app in apps.iter_mut() {
                                    if filename_eq(&app.path, filename) {
                                        app.is_default = true;
                                        found = true;
                                    } else {
                                        app.is_default = false;
                                    }
                                }
                                if found {
                                    break;
                                } else {
                                    log::debug!("failed to set default for {:?}: application {:?} not found", mime, filename);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort apps by name
        for apps in self.cache.values_mut() {
            apps.sort_by(|a, b| match (a.is_default, b.is_default) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => LANGUAGE_SORTER.compare(&a.name, &b.name),
            });
        }

        let elapsed = start.elapsed();
        log::info!("loaded mime app cache in {:?}", elapsed);
    }

    pub fn get(&self, key: &Mime) -> Vec<MimeApp> {
        self.cache
            .get(&key)
            .map_or_else(|| Vec::new(), |x| x.clone())
    }
}

static MIME_APP_CACHE: Lazy<Mutex<MimeAppCache>> = Lazy::new(|| Mutex::new(MimeAppCache::new()));

pub fn mime_apps(mime: &Mime) -> Vec<MimeApp> {
    let mime_app_cache = MIME_APP_CACHE.lock().unwrap();
    mime_app_cache.get(mime)
}

pub fn terminal() -> Option<MimeApp> {
    let mime_app_cache = MIME_APP_CACHE.lock().unwrap();

    //TODO: consider rules in https://github.com/Vladimir-csp/xdg-terminal-exec

    // Look for and return preferred terminals
    //TODO: fallback order beyond cosmic-term?
    for id in &["com.system76.CosmicTerm"] {
        for terminal in mime_app_cache.terminals.iter() {
            if &terminal.id == id {
                return Some(terminal.clone());
            }
        }
    }

    // Return whatever was the first terminal found
    mime_app_cache.terminals.first().map(|x| x.clone())
}
