// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "desktop")]
use cosmic::desktop;
use cosmic::widget;
pub use mime_guess::Mime;
use std::{
    cmp::Ordering, collections::HashMap, env, ffi::OsString, fs, io, path::PathBuf, process,
    time::Instant,
};
#[cfg(feature = "desktop")]
use cosmic::desktop::DesktopEntry;

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
        let exec = self.exec.as_deref()?;
        
        // Special handling for terminals when we have a path
        if self.is_terminal() && path_opt.is_some() {
            // Check if the terminal has a special working directory argument
            let working_dir_arg = if exec.contains("warp") {
                "--working-directory"
            } else if exec.contains("gnome-terminal") {
                "--working-directory"
            } else if exec.contains("xfce4-terminal") {
                "--working-directory"
            } else if exec.contains("konsole") {
                "--workdir"
            } else {
                // Default to no special argument, will use current_dir instead
                ""
            };

            // If we have a special working directory argument, use it
            if !working_dir_arg.is_empty() {
                let mut args_vec = shlex::split(exec)?;
                if let Some(path) = &path_opt {
                    args_vec.push(working_dir_arg.to_string());
                    args_vec.push(path.to_string_lossy().into_owned());
                }
                let mut args = args_vec.iter();
                let mut command = process::Command::new(args.next()?);
                for arg in args {
                    command.arg(arg);
                }
                return Some(command);
            }
        }
        
        // Default handling for non-terminals or terminals without path
        exec_to_command(exec, path_opt)
    }

    #[cfg(feature = "desktop")]
    pub fn is_terminal(&self) -> bool {
        // Check if this app is a terminal emulator
        if let Some(path) = &self.path {
            if let Ok(bytes) = std::fs::read_to_string(path) {
                if let Ok(entry) = DesktopEntry::decode(path, &bytes) {
                    return entry.categories()
                        .map(|cats| cats.split(';').any(|c| c == "TerminalEmulator"))
                        .unwrap_or(false);
                }
            }
        }
        false
    }

    #[cfg(not(feature = "desktop"))]
    pub fn is_terminal(&self) -> bool {
        false
    }
}

// This allows usage of MimeApp in a dropdown
impl AsRef<str> for MimeApp {
    fn as_ref(&self) -> &str {
        &self.name
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
    icons: HashMap<Mime, Vec<widget::icon::Handle>>,
    terminals: Vec<MimeApp>,
}

impl MimeAppCache {
    pub fn new() -> Self {
        let mut cache = Self {
            cache: HashMap::new(),
            icons: HashMap::new(),
            terminals: Vec::new(),
        };
        cache.reload();
        #[cfg(feature = "desktop")]
        cache.start_watcher();
        cache
    }

    #[cfg(not(feature = "desktop"))]
    pub fn reload(&mut self) {}

    // Only available when using desktop feature of libcosmic, which only works on Unix-likes
    #[cfg(feature = "desktop")]
    pub fn reload(&mut self) {
        use crate::localize::LANGUAGE_SORTER;

        let start = Instant::now();

        self.cache.clear();
        self.icons.clear();
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
                if !apps.iter().any(|x| x.id == app.id) {
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
                            if !apps.iter().any(|x| filename_eq(&x.path, filename)) {
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

        // Copy icons to special cache
        //TODO: adjust dropdown API so this is no longer needed
        for (mime, apps) in self.cache.iter() {
            self.icons.insert(
                mime.clone(),
                apps.iter().map(|app| app.icon.clone()).collect(),
            );
        }

        let elapsed = start.elapsed();
        log::info!("loaded mime app cache in {:?}", elapsed);
    }

    pub fn get(&self, key: &Mime) -> &[MimeApp] {
        static EMPTY: Vec<MimeApp> = Vec::new();
        self.cache.get(key).unwrap_or(&EMPTY)
    }

    pub fn icons(&self, key: &Mime) -> &[widget::icon::Handle] {
        static EMPTY: Vec<widget::icon::Handle> = Vec::new();
        self.icons.get(key).unwrap_or(&EMPTY)
    }

    pub fn terminal(&self) -> Option<&MimeApp> {
        // First check MIME-based default
        let mime_default = self.get_default_terminal_from_mime();
        if let Some(term) = mime_default {
            return Some(term);
        }

        // Fallback to update-alternatives system
        self.get_system_default_terminal()
    }

    fn get_default_terminal_from_mime(&self) -> Option<&MimeApp> {
        // Check both common terminal MIME types
        let mime_types = [
            "x-scheme-handler/terminal",
            "application/x-terminal-emulator"
        ];

        for mime_str in mime_types {
            if let Ok(mime) = mime_str.parse::<Mime>() {
                if let Some(apps) = self.cache.get(&mime) {
                    if let Some(default) = apps.iter().find(|app| app.is_default) {
                        return Some(default);
                    }
                }
            }
        }
        None
    }

    fn get_system_default_terminal(&self) -> Option<&MimeApp> {
        if let Ok(output) = std::process::Command::new("update-alternatives")
            .arg("--query")
            .arg("x-terminal-emulator")
            .output() 
        {
            if let Ok(query_output) = String::from_utf8(output.stdout) {
                // First find the current value (executable path)
                let mut current_exec = None;
                for line in query_output.lines() {
                    if line.starts_with("Value: ") {
                        current_exec = Some(line.trim_start_matches("Value: ").to_string());
                        break;
                    }
                }

                // Try to find a terminal that matches this executable
                if let Some(exec_path) = current_exec {
                    for terminal in &self.terminals {
                        if let Some(exec) = &terminal.exec {
                            // Extract the executable from the Exec= line
                            if let Some(cmd) = exec.split_whitespace().next() {
                                // Compare just the binary name as a fallback
                                if exec_path.ends_with(cmd) {
                                    return Some(terminal);
                                }
                                // Also try comparing the full path
                                if exec_path == cmd {
                                    return Some(terminal);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to first available terminal if no system default is set
        self.terminals.first()
    }

    #[cfg(feature = "desktop")]
    fn start_watcher(&mut self) {
        use notify::{RecommendedWatcher, Watcher, RecursiveMode};
        use std::time::Duration;

        let paths = vec![
            dirs::config_dir().unwrap().join("mimeapps.list"),
            dirs::config_local_dir().unwrap().join("applications/mimeapps.list")
        ];

        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, _>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() {
                        // Note: Since we can't directly access self here,
                        // we'll need to use a channel or other mechanism to trigger reload
                        log::info!("MIME associations changed, reload needed");
                    }
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_secs(2))
        );

        if let Ok(mut watcher) = watcher {
            for path in paths {
                let _ = watcher.watch(&path, RecursiveMode::NonRecursive);
            }
        }
    }

    #[cfg(not(feature = "desktop"))]
    pub fn set_default(&mut self, mime: Mime, id: String) {
        log::warn!(
            "failed to set default handler for {mime:?} to {id:?}: desktop feature not enabled"
        );
    }

    #[cfg(feature = "desktop")]
    pub fn set_default(&mut self, mime: Mime, mut id: String) {
        let Some(path) = cosmic_mime_apps::local_list_path() else {
            log::warn!("failed to find mimeapps.list path");
            return;
        };

        let mut list = cosmic_mime_apps::List::default();
        match fs::read_to_string(&path) {
            Ok(string) => {
                list.load_from(&string);
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::NotFound {
                    log::warn!("failed to read {path:?}: {err}");
                    return;
                }
            }
        }

        let suffix = ".desktop";
        if !id.ends_with(suffix) {
            id.push_str(suffix);
        }
        list.set_default_app(mime, id);

        let mut string = list.to_string();
        string.push('\n');
        match fs::write(&path, string) {
            Ok(()) => {
                self.reload();
            }
            Err(err) => {
                log::warn!("failed to write {path:?}: {err}");
            }
        }
    }
}

impl Default for MimeAppCache {
    fn default() -> Self {
        Self::new()
    }
}
