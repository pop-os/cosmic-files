// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "desktop")]
use cosmic::desktop::fde::GenericEntry;
use mime_guess::Mime;
use rustc_hash::FxHashMap;
#[cfg(feature = "desktop")]
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::Instant;

const EXE_THUMBNAILER_EXEC: &str = "cosmic-files --thumbnail-exe %o --size %s %i";
const EXE_MIME_TYPES: [&str; 2] = [
    "application/vnd.microsoft.portable-executable",
    "application/x-msdownload",
];

static BUNDLED_EXE_THUMBNAILER: OnceLock<PathBuf> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Thumbnailer {
    pub exec: String,
    executable: Option<PathBuf>,
}

impl Thumbnailer {
    pub fn command(
        &self,
        input: &Path,
        output: &Path,
        thumbnail_size: u32,
    ) -> Option<process::Command> {
        let args_vec: Vec<String> = shlex::split(&self.exec)?;
        let mut args = args_vec.iter();
        let parsed_executable = args.next()?;
        let executable = self
            .executable
            .as_deref()
            .unwrap_or_else(|| Path::new(parsed_executable));
        let mut command = process::Command::new(executable);
        for arg in args {
            if arg.starts_with('%') {
                match arg.as_str() {
                    "%i" | "%u" => {
                        command.arg(input);
                    }
                    "%o" => {
                        command.arg(output);
                    }
                    "%s" => {
                        command.arg(format!("{thumbnail_size}"));
                    }
                    _ => {
                        log::warn!(
                            "unsupported thumbnailer Exec code {:?} in {:?}",
                            arg,
                            self.exec
                        );
                        return None;
                    }
                }
            } else {
                command.arg(arg);
            }
        }
        Some(command)
    }
}

pub struct ThumbnailerCache {
    cache: FxHashMap<Mime, Vec<Thumbnailer>>,
}

impl ThumbnailerCache {
    pub fn new() -> Self {
        let mut thumbnailer_cache = Self {
            cache: FxHashMap::default(),
        };
        thumbnailer_cache.reload();
        thumbnailer_cache
    }

    pub fn reload(&mut self) {
        let start = Instant::now();

        self.cache.clear();
        self.load_bundled_exe_thumbnailer();

        #[cfg(feature = "desktop")]
        self.load_external_thumbnailers();

        let elapsed = start.elapsed();
        log::info!("loaded thumbnailer cache in {elapsed:?}");
    }

    fn load_bundled_exe_thumbnailer(&mut self) {
        let Some(executable) = BUNDLED_EXE_THUMBNAILER.get() else {
            return;
        };
        let thumbnailer = Thumbnailer {
            exec: EXE_THUMBNAILER_EXEC.to_string(),
            executable: Some(executable.clone()),
        };

        for mime_type in EXE_MIME_TYPES {
            let mime = mime_type.parse::<Mime>().expect("valid static MIME type");
            self.cache
                .entry(mime)
                .or_insert_with(|| Vec::with_capacity(1))
                .push(thumbnailer.clone());
        }
    }

    #[cfg(feature = "desktop")]
    fn load_external_thumbnailers(&mut self) {
        let mut search_dirs = Vec::new();
        let xdg_dirs = xdg::BaseDirectories::new();

        if let Some(mut data_home) = xdg_dirs.get_data_home() {
            data_home.push("thumbnailers");
            search_dirs.push(data_home);
        }
        search_dirs.extend(xdg_dirs.get_data_dirs().into_iter().map(|mut data_dir| {
            data_dir.push("thumbnailers");
            data_dir
        }));

        let mut thumbnailer_paths = Vec::new();
        for dir in search_dirs {
            log::trace!("looking for thumbnailers in {}", dir.display());
            match fs::read_dir(&dir) {
                Ok(entries) => {
                    thumbnailer_paths.extend(entries.filter_map(|entry_res| {
                        entry_res
                            .inspect_err(|err| {
                                log::warn!(
                                    "failed to read entry in directory {}: {}",
                                    dir.display(),
                                    err
                                )
                            })
                            .ok()
                            .map(|entry| entry.path())
                    }));
                }
                Err(err) => {
                    log::warn!("failed to read directory {}: {}", dir.display(), err);
                }
            }
        }

        //TODO: handle directory specific behavior
        for path in thumbnailer_paths {
            let entry = match GenericEntry::from_path(&path) {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!("failed to parse {}: {}", path.display(), err);
                    continue;
                }
            };

            //TODO: use TryExec?
            let Some(section) = entry.group("Thumbnailer Entry") else {
                log::warn!(
                    "missing Thumbnailer Entry section for thumbnailer {}",
                    path.display()
                );
                continue;
            };
            let Some(exec) = section.entry("Exec") else {
                log::warn!("missing Exec attribute for thumbnailer {}", path.display());
                continue;
            };
            let Some(mime_types) = section.entry("MimeType") else {
                log::warn!(
                    "missing MimeType attribute for thumbnailer {}",
                    path.display()
                );
                continue;
            };

            for mime_type in mime_types.split_terminator(';') {
                if let Ok(mime) = mime_type.parse::<Mime>() {
                    log::trace!("thumbnailer {}={}", mime, path.display());
                    let apps = self
                        .cache
                        .entry(mime)
                        .or_insert_with(|| Vec::with_capacity(1));
                    apps.push(Thumbnailer {
                        exec: exec.to_string(),
                        executable: None,
                    });
                }
            }
        }
    }

    pub fn get(&self, key: &Mime) -> Vec<Thumbnailer> {
        self.cache.get(key).map_or_else(Vec::new, Vec::clone)
    }
}

static THUMBNAILER_CACHE: LazyLock<Mutex<ThumbnailerCache>> =
    LazyLock::new(|| Mutex::new(ThumbnailerCache::new()));

pub fn register_bundled_exe_thumbnailer(executable: PathBuf) {
    let _ = BUNDLED_EXE_THUMBNAILER.set(executable);
}

pub fn thumbnailer(mime: &Mime) -> Vec<Thumbnailer> {
    let thumbnailer_cache = THUMBNAILER_CACHE.lock().unwrap();
    thumbnailer_cache.get(mime)
}

#[cfg(test)]
mod tests {
    use super::{EXE_THUMBNAILER_EXEC, Thumbnailer};
    use std::path::{Path, PathBuf};

    #[test]
    fn command_uses_bundled_executable() {
        let executable = PathBuf::from("/tmp/cosmic-files-under-test");
        let thumbnailer = Thumbnailer {
            exec: EXE_THUMBNAILER_EXEC.to_string(),
            executable: Some(executable.clone()),
        };

        let command = thumbnailer
            .command(Path::new("input.exe"), Path::new("output.png"), 128)
            .unwrap();
        assert_eq!(command.get_program(), executable);
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "--thumbnail-exe",
                "output.png",
                "--size",
                "128",
                "input.exe"
            ]
        );
    }
}
