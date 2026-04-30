// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use bstr::{BString, ByteSlice, ByteVec};
#[cfg(feature = "desktop")]
use cosmic::desktop;
use cosmic::widget;
pub use mime_guess::Mime;
use rustc_hash::FxHashMap;
use std::cmp::Ordering;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{fs, io, process};

pub fn exec_to_command(
    exec: &str,
    entry_name: &str,
    entry_path: Option<&Path>,
    path_opt: &[impl AsRef<OsStr>],
) -> Option<Vec<process::Command>> {
    let arguments = shlex::split(exec)?;

    if arguments.is_empty() {
        tracing::error!("command does not contain any arguments");
        return None;
    }

    let mut commands = Vec::new();

    let paths = path_opt
        .iter()
        .map(AsRef::as_ref)
        .map(Some)
        // Add a single `None` if no path was given.
        .chain(std::iter::repeat_n(
            None,
            if path_opt.is_empty() { 1 } else { 0 },
        ));

    for path in paths {
        let mut batch_process = false;
        let mut args = Vec::with_capacity(arguments.len());
        let mut field_code_used = false;

        for argument in arguments.iter().skip(1) {
            let mut new_argument = BString::new(Vec::with_capacity(argument.capacity()));
            let mut chars = argument.chars();
            while let Some(char) = chars.next() {
                // https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html
                if char == '%' {
                    match chars.next() {
                        Some('%') => new_argument.push_char(char),
                        Some('c') => new_argument.push_str(entry_name),
                        Some('k') => {
                            if let Some(path) = entry_path {
                                new_argument.push_str(path.as_os_str().as_bytes());
                            }
                        }

                        // %f and %u behave the same in a file manager.
                        Some('f' | 'u')
                            if let Some(path) = path
                                && !field_code_used =>
                        {
                            // TODO: files on remote file systems should be copied to a temporary local file.
                            batch_process = true;
                            field_code_used = true;
                            new_argument.push_str(path.as_bytes());
                        }

                        // %F and %U behave the same in a file manager.
                        Some('F') | Some('U') if !field_code_used && new_argument.is_empty() => {
                            field_code_used = true;
                            for path in path_opt.iter().map(AsRef::as_ref) {
                                args.push(BString::new(path.as_bytes().to_owned()));
                            }
                        }

                        _ => (),
                    }
                } else {
                    new_argument.push_char(char);
                }
            }

            if !new_argument.is_empty() {
                args.push(new_argument);
            }
        }

        let mut command = process::Command::new(&arguments[0]);

        for arg in args {
            match arg.to_os_str() {
                Ok(arg) => {
                    command.arg(arg);
                }
                Err(_) => {
                    tracing::error!("invalid string encoding in command");
                    return None;
                }
            }
        }

        commands.push(command);

        if !batch_process {
            break;
        }
    }

    #[cfg(debug_assertions)]
    for command in &commands {
        log::debug!(
            "Parsed program {} with args: {:?}",
            command.get_program().to_string_lossy(),
            command.get_args()
        );
    }

    Some(commands)
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
        exec_to_command(
            self.exec.as_deref()?,
            &self.name,
            self.path.as_deref(),
            path_opt,
        )
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
                desktop::fde::IconSource::Name(name) => {
                    widget::icon::from_name(name.as_str()).size(32).handle()
                }
                desktop::fde::IconSource::Path(path) => widget::icon::from_path(path.clone()),
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
        .is_some_and(|x| x == filename)
}

pub struct MimeAppCache {
    apps: Vec<MimeApp>,
    cache: FxHashMap<Mime, Vec<MimeApp>>,
    icons: FxHashMap<Mime, Box<[widget::icon::Handle]>>,
    terminals: Vec<MimeApp>,
}

impl MimeAppCache {
    pub fn new() -> Self {
        let mut mime_app_cache = Self {
            apps: Vec::new(),
            cache: FxHashMap::default(),
            icons: FxHashMap::default(),
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

        self.apps.clear();
        self.cache.clear();
        self.icons.clear();
        self.terminals.clear();

        //TODO: get proper locale?
        let locale = &[];

        // Load desktop applications by supported mime types
        //TODO: hashmap for all apps by id?
        let all_apps: Box<[_]> = desktop::load_applications(locale, false, None).collect();
        for app in &all_apps {
            //TODO: just collect apps that can be executed with a file argument?
            if !app.mime_types.is_empty() {
                self.apps.push(MimeApp::from(app));
            }
            for mime in &app.mime_types {
                let apps = self
                    .cache
                    .entry(mime.clone())
                    .or_insert_with(|| Vec::with_capacity(1));
                if !apps.iter().any(|x| x.id == app.id) {
                    apps.push(MimeApp::from(app));
                }
            }
            for category in &app.categories {
                if category == "TerminalEmulator" {
                    self.terminals.push(MimeApp::from(app));
                    break;
                }
            }
        }

        let mut list = cosmic_mime_apps::List::default();

        let paths = cosmic_mime_apps::list_paths();
        list.load_from_paths(&paths);

        for (mime, filenames) in list
            .added_associations
            .iter()
            .chain(list.default_apps.iter())
        {
            for filename in filenames {
                log::trace!("add {mime}={filename}");
                let apps = self
                    .cache
                    .entry(mime.clone())
                    .or_insert_with(|| Vec::with_capacity(1));
                if !apps.iter().any(|x| filename_eq(&x.path, filename)) {
                    if let Some(app) = all_apps.iter().find(|&x| filename_eq(&x.path, filename)) {
                        apps.push(MimeApp::from(app));
                    } else {
                        log::info!(
                            "failed to add association for {mime:?}: application {filename:?} not found"
                        );
                    }
                }
            }
        }

        for (mime, filenames) in list.removed_associations.iter() {
            for filename in filenames {
                log::trace!("remove {mime}={filename}");
                if let Some(apps) = self.cache.get_mut(mime) {
                    apps.retain(|x| !filename_eq(&x.path, filename));
                }
            }
        }

        for (mime, filenames) in list.default_apps.iter() {
            for filename in filenames {
                log::trace!("default {mime}={filename}");
                if let Some(apps) = self.cache.get_mut(mime) {
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
                    }
                    log::debug!(
                        "failed to set default for {mime:?}: application {filename:?} not found"
                    );
                }
            }
        }

        // Sort apps by name
        self.apps
            .sort_by(|a, b| match (a.is_default, b.is_default) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => LANGUAGE_SORTER.compare(&a.name, &b.name),
            });
        for apps in self.cache.values_mut() {
            apps.sort_by(|a, b| match (a.is_default, b.is_default) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => LANGUAGE_SORTER.compare(&a.name, &b.name),
            });
        }

        // Copy icons to special cache
        //TODO: adjust dropdown API so this is no longer needed
        self.icons.extend(self.cache.iter().map(|(mime, apps)| {
            (
                mime.clone(),
                apps.iter().map(|app| app.icon.clone()).collect(),
            )
        }));

        let elapsed = start.elapsed();
        log::info!("loaded mime app cache in {elapsed:?}");
    }

    pub fn apps(&self) -> &[MimeApp] {
        &self.apps
    }

    pub fn get(&self, key: &Mime) -> &[MimeApp] {
        self.cache.get(key).map_or(&[], Vec::as_slice)
    }

    pub fn icons(&self, key: &Mime) -> &[widget::icon::Handle] {
        self.icons.get(key).map_or(&[], Box::as_ref)
    }

    fn get_default_terminal(&self) -> Option<String> {
        let output = process::Command::new("xdg-mime")
            .args(["query", "default", "x-scheme-handler/terminal"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        String::from_utf8(output.stdout)
            .ok()
            .map(|string| string.trim().replace(".desktop", ""))
    }

    pub fn terminal(&self) -> Option<&MimeApp> {
        //TODO: consider rules in https://github.com/Vladimir-csp/xdg-terminal-exec
        // The current approach works but might not adhere to the spec (yet)

        // Look for and return preferred terminals
        //TODO: fallback order beyond cosmic-term?

        let mut preference_order = vec!["com.system76.CosmicTerm".to_string()];

        if let Some(id) = self.get_default_terminal() {
            preference_order.insert(0, id);
        }

        for id in &preference_order {
            for terminal in &self.terminals {
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
                    log::warn!("failed to read {}: {}", path.display(), err);
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
                log::warn!("failed to write {}: {}", path.display(), err);
            }
        }
    }
}

impl Default for MimeAppCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::exec_to_command;

    #[test]
    fn keys_within_words() {
        let exec = "/usr/bin/foo --option=%f";
        let paths = ["file1"];
        let commands = exec_to_command(exec, "keys_within_words", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();

        assert_eq!("/usr/bin/foo", command.get_program().to_str().unwrap());
        assert_eq!(
            "--option=file1",
            command.get_args().next().unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn no_path_f_field_code() {
        let exec = "/usr/bin/foo %f";
        let paths: [&str; 0] = [];
        let commands = exec_to_command(exec, "no_path_f_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();

        assert_eq!("/usr/bin/foo", command.get_program().to_str().unwrap());
        assert_eq!(0, command.get_args().len());
    }

    #[test]
    fn one_path_f_field_code() {
        let exec = "/usr/bin/foo %f";
        let paths = ["file1"];
        let commands = exec_to_command(exec, "one_path_f_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();

        assert_eq!("/usr/bin/foo", command.get_program().to_str().unwrap());
        assert_eq!(
            "file1",
            command.get_args().next().unwrap().to_str().unwrap()
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn one_path_F_field_code() {
        let exec = "/usr/bin/cosmic-term -w %F";
        let paths = ["/home/user"];
        let commands = exec_to_command(exec, "one_path_F_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();
        let mut args = command.get_args();

        assert_eq!(
            "/usr/bin/cosmic-term",
            command.get_program().to_str().unwrap()
        );
        assert_eq!("-w", args.next().unwrap().to_str().unwrap());
        assert_eq!(paths[0], args.next().unwrap().to_str().unwrap());
    }

    #[test]
    fn one_path_u_field_code() {
        let exec = "/usr/bin/cosmic-term -w %u";
        let paths = ["/home/user"];
        let commands = exec_to_command(exec, "one_path_u_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();
        let mut args = command.get_args();

        assert_eq!(
            "/usr/bin/cosmic-term",
            command.get_program().to_str().unwrap()
        );
        assert_eq!("-w", args.next().unwrap().to_str().unwrap());
        assert_eq!(paths[0], args.next().unwrap().to_str().unwrap());
    }

    #[test]
    #[allow(non_snake_case)]
    fn one_path_U_field_code() {
        let exec = "/usr/bin/rmrfbye %U";
        let paths = ["/"];
        let commands = exec_to_command(exec, "one_path_U_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();

        assert_eq!("/usr/bin/rmrfbye", command.get_program().to_str().unwrap());
        assert_eq!("/", command.get_args().next().unwrap().to_str().unwrap());
    }

    #[test]
    fn mult_path_f_field_code() {
        let exec = "/usr/games/ppsspp %f";
        let paths = [
            "/usr/share/games/psp/miku.iso",
            "/usr/share/games/psp/eternia.iso",
        ];
        let commands = exec_to_command(exec, "mult_path_f_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(paths.len(), commands.len());
        for (command, path) in commands.into_iter().zip(paths.iter()) {
            assert_eq!("/usr/games/ppsspp", command.get_program().to_str().unwrap());

            assert_eq!(1, command.get_args().len());
            let command_path = command.get_args().next().unwrap();
            assert_eq!(*path, command_path.to_str().unwrap());
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn mult_path_F_field_code() {
        let exec = "/usr/games/gzdoom %F";
        let paths = [
            "/usr/share/games/doom2/hr.wad",
            "/usr/share/games/doom2/hrmus.wad",
        ];
        let commands = exec_to_command(exec, "mult_path_F_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();

        assert_eq!("/usr/games/gzdoom", command.get_program().to_str().unwrap());
        assert!(
            paths
                .iter()
                .zip(command.get_args())
                .all(|(&expected, actual)| expected == actual.to_string_lossy())
        );
    }

    #[test]
    fn mult_path_u_field_code() {
        let exec = "/usr/bin/cosmic_browser %u";
        let paths = [
            "file:///home/josh/Books/osstep.pdf",
            "https://redox-os.org/",
            "https://system76.com/",
        ];
        let commands = exec_to_command(exec, "mult_path_u_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(paths.len(), commands.len());
        for (command, path) in commands.into_iter().zip(paths.iter()) {
            assert_eq!(
                "/usr/bin/cosmic_browser",
                command.get_program().to_str().unwrap()
            );

            assert_eq!(1, command.get_args().len());
            let command_path = command.get_args().next().unwrap();
            assert_eq!(*path, command_path.to_str().unwrap());
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn mult_path_U_field_code() {
        let exec = "/usr/bin/mpv %U";
        let paths = [
            "frieren01.mkv",
            "rtmp://example.org/this/video/doesnt/exist.avi",
        ];
        let commands = exec_to_command(exec, "mult_path_U_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();
        assert_eq!(paths.len(), command.get_args().count());

        assert_eq!("/usr/bin/mpv", command.get_program().to_str().unwrap());
        assert!(
            paths
                .iter()
                .zip(command.get_args())
                .all(|(&expected, actual)| expected == actual.to_string_lossy())
        );
    }

    #[test]
    fn flatpak_style_exec() {
        // Tests args before field codes
        let exec = "/usr/bin/flatpak run --branch=stable --command=ferris --file-forwarding org.joshfake.ferris @@u %U";
        let args = [
            "run",
            "--branch=stable",
            "--command=ferris",
            "--file-forwarding",
            "org.joshfake.ferris",
            "@@u",
        ];
        let paths = ["file1.rs", "file2.rs"];
        let commands = exec_to_command(exec, "flatpak_style_exec", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();
        assert_eq!(args.len() + paths.len(), command.get_args().count());

        assert_eq!("/usr/bin/flatpak", command.get_program().to_str().unwrap());
        assert!(
            args.iter()
                .chain(paths.iter())
                .zip(command.get_args())
                .all(|(&expected, actual)| expected == actual.to_string_lossy())
        );
    }

    #[test]
    fn multiple_field_codes() {
        // Tests that only one field code is used rather than passing paths to each field code
        let exec = "/usr/games/roguelike %U %f";
        let paths = [
            "file:///usr/share/games/roguelike/mods/mod1",
            "file:///usr/share/games/roguelike/mods/mod2",
        ];
        let commands = exec_to_command(exec, "multiple_field_codes", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();

        assert_eq!(
            "/usr/games/roguelike",
            command.get_program().to_str().unwrap()
        );
        assert!(
            paths
                .iter()
                .zip(command.get_args())
                .all(|(&expected, actual)| expected == actual.to_string_lossy())
        );
    }

    #[test]
    fn sandwiched_field_code() {
        // Tests that arguments before and after the field code works
        // (Borrowed from KDE because someone had this exact line in an issue)
        let exec = "/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=okular --file-forwarding org.kde.okular @@u %U @@";
        let args_leading = [
            "run",
            "--branch=stable",
            "--arch=x86_64",
            "--command=okular",
            "--file-forwarding",
            "org.kde.okular",
            "@@u",
        ];
        let paths = ["rust_game_dev.pdf", "superhero_ferris.epub"];
        let args_trailing = ["@@"];
        let commands = exec_to_command(exec, "sandwiched_field_code", None, &paths)
            .expect("Should parse valid exec");

        assert_eq!(1, commands.len());
        let command = commands.first().unwrap();
        assert_eq!(
            args_leading.len() + paths.len() + args_trailing.len(),
            command.get_args().len()
        );

        assert_eq!("/usr/bin/flatpak", command.get_program().to_str().unwrap());
        assert!(
            args_leading
                .iter()
                .chain(paths.iter())
                .chain(args_trailing.iter())
                .zip(command.get_args())
                .all(|(&expected, actual)| expected == actual.to_string_lossy())
        );
    }
}
