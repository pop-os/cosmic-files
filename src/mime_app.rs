// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "desktop")]
use cosmic::desktop;
use cosmic::widget;
pub use mime_guess::Mime;
use std::{
    cmp::Ordering,
    collections::HashMap,
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process,
    time::Instant,
};

// Supported exec key field codes
const EXEC_HANDLERS: [&str; 4] = ["%f", "%F", "%u", "%U"];
// Deprecated field codes. The spec advises to ignore these handlers.
const DEPRECATED_HANDLERS: [&str; 6] = ["%d", "%D", "%n", "%N", "%v", "%m"];

pub fn exec_to_command(
    exec: &str,
    path_opt: &[impl AsRef<OsStr>],
) -> Option<Vec<process::Command>> {
    let args_vec = shlex::split(exec)?;
    let program = args_vec.first()?;
    // Skip program to make indexing easier
    let args_vec = &args_vec[1..];

    // Base Command instance(s)
    // 1. We may need to launch multiple of the same process.
    // 2. Each of those processes will need to be passed args from exec.
    // 3. Each of those args may appear in any order.
    // 4. Arg order should be preserved.
    //
    // So, we'll go through exec in two passes. The first pass handles paths (%f etc) and args up
    // to the field code followed by the second which passes extra, non-% args to each processes.
    //
    // While it'd be marginally faster to process everything in one pass, that's problematic:
    // 1. path_opt may need to be cloned because it may be moved on each iteration (borrowck
    //    doesn't know we'll only use it once)
    // 2. We have to keep track of which modifier (%f etc) we've used/seen already
    // 3. We have to keep track of which processes received non-modifier args which gets messy fast
    // 4. `exec` is likely small so looping over it twice is not a big deal
    let field_code_pos = args_vec
        .iter()
        .position(|arg| EXEC_HANDLERS.contains(&arg.as_str()));
    let args_handler = field_code_pos.and_then(|i| args_vec.get(i));
    // msrv
    // .inspect(|handler| log::trace!("Found paths handler: {handler} for exec: {exec}"));
    // Number of args before the field code.
    // This won't be an off by one err below because take is not zero indexed.
    let field_code_pos = field_code_pos.unwrap_or_default();
    let mut processes = match args_handler.map(|s| s.as_str()) {
        Some("%f") => {
            let mut processes = Vec::with_capacity(path_opt.len());

            for path in path_opt.iter().map(AsRef::as_ref) {
                // TODO: %f and %F need to handle non-file URLs (see spec)
                if from_file_or_dir(path).is_none() {
                    log::warn!("Desktop file expects a file path instead of a URL: {path:?}");
                }

                // Passing multiple paths to %f should open an instance per path
                let mut process = process::Command::new(program);
                process.args(
                    args_vec
                        .iter()
                        .map(AsRef::as_ref)
                        .take(field_code_pos)
                        .chain(std::iter::once(path)),
                );
                processes.push(process);
            }

            processes
        }
        Some("%F") => {
            // TODO: %f and %F need to handle non-file URLs (see spec)
            for invalid in path_opt
                .iter()
                .map(AsRef::as_ref)
                .filter(|path| from_file_or_dir(path).is_none())
            {
                log::warn!("Desktop file expects a file path instead of a URL: {invalid:?}");
            }

            // Launch one instance with all args
            let mut process = process::Command::new(program);
            process.args(
                args_vec
                    .iter()
                    .map(OsStr::new)
                    .take(field_code_pos)
                    .chain(path_opt.iter().map(AsRef::as_ref)),
            );

            vec![process]
        }
        Some("%u") => path_opt
            .iter()
            .map(|path| {
                let mut process = process::Command::new(program);
                process.args(
                    args_vec
                        .iter()
                        .map(OsStr::new)
                        .take(field_code_pos)
                        .chain(std::iter::once(path.as_ref())),
                );
                process
            })
            .collect(),
        Some("%U") => {
            let mut process = process::Command::new(program);
            process.args(
                args_vec
                    .iter()
                    .map(OsStr::new)
                    .take(field_code_pos)
                    .chain(path_opt.iter().map(AsRef::as_ref)),
            );
            vec![process]
        }
        Some(invalid) => unreachable!("All valid variants were checked; got: {invalid}"),
        None => vec![process::Command::new(program)],
    };

    // Pass 2: Add remaining arguments that are not % to each process
    for arg in args_vec.iter().skip(field_code_pos) {
        match arg.as_str() {
            // Consume path field codes or fail on codes we don't handle yet
            field_code if arg.starts_with('%') => {
                if !EXEC_HANDLERS.contains(&field_code)
                    && !DEPRECATED_HANDLERS.contains(&field_code)
                {
                    log::warn!("unsupported Exec code {:?} in {:?}", field_code, exec);
                    return None;
                }
            }
            arg => {
                for process in &mut processes {
                    process.arg(arg);
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    for command in &processes {
        log::debug!(
            "Parsed program {} with args: {:?}",
            command.get_program().to_string_lossy(),
            command.get_args()
        );
    }

    Some(processes)
}

fn from_file_or_dir(path: impl AsRef<Path>) -> Option<url::Url> {
    url::Url::from_file_path(&path)
        .ok()
        .or_else(|| url::Url::from_directory_path(&path).ok())
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
    pub fn command<O: AsRef<OsStr>>(&self, path_opt: &[O]) -> Option<Vec<process::Command>> {
        exec_to_command(self.exec.as_deref()?, path_opt)
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
        let mut mime_app_cache = Self {
            cache: HashMap::new(),
            icons: HashMap::new(),
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
        //TODO: consider rules in https://github.com/Vladimir-csp/xdg-terminal-exec

        // Look for and return preferred terminals
        //TODO: fallback order beyond cosmic-term?
        for id in &["com.system76.CosmicTerm"] {
            for terminal in self.terminals.iter() {
                if &terminal.id == id {
                    return Some(terminal);
                }
            }
        }

        // Return whatever was the first terminal found
        self.terminals.first()
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
