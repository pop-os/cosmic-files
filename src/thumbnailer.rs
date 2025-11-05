// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use mime_guess::Mime;
use rustc_hash::FxHashMap;
use std::{
    fs,
    path::Path,
    process,
    sync::{LazyLock, Mutex},
    time::Instant,
};

#[derive(Clone, Debug)]
pub struct Thumbnailer {
    pub exec: String,
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
        let mut command = process::Command::new(args.next()?);
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

    #[cfg(not(feature = "desktop"))]
    pub fn reload(&mut self) {}

    #[cfg(feature = "desktop")]
    pub fn reload(&mut self) {
        let start = Instant::now();

        self.cache.clear();

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
            let entry = match freedesktop_entry_parser::parse_entry(&path) {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!("failed to parse {}: {}", path.display(), err);
                    continue;
                }
            };

            let Some(section) = entry.section("Thumbnailer Entry") else {
                log::warn!(
                    "missing Thumbnailer Entry section for thumbnailer {}",
                    path.display()
                );
                continue;
            };

            //TODO: use TryExec?
            let Some(exec) = section.attr("Exec").first() else {
                log::warn!("missing Exec attribute for thumbnailer {}", path.display());
                continue;
            };

            let mime_types = section.attr("MimeType");
            if mime_types.is_empty() {
                log::warn!(
                    "missing MimeType attribute for thumbnailer {}",
                    path.display()
                );
                continue;
            }

            for mime_type in mime_types {
                if let Ok(mime) = mime_type.parse::<Mime>() {
                    log::trace!("thumbnailer {}={}", mime, path.display());
                    let apps = self
                        .cache
                        .entry(mime)
                        .or_insert_with(|| Vec::with_capacity(1));
                    apps.push(Thumbnailer { exec: exec.clone() });
                }
            }
        }

        let elapsed = start.elapsed();
        log::info!("loaded thumbnailer cache in {elapsed:?}");
    }

    pub fn get(&self, key: &Mime) -> Vec<Thumbnailer> {
        self.cache.get(key).map_or_else(Vec::new, Vec::clone)
    }
}

static THUMBNAILER_CACHE: LazyLock<Mutex<ThumbnailerCache>> =
    LazyLock::new(|| Mutex::new(ThumbnailerCache::new()));

pub fn thumbnailer(mime: &Mime) -> Vec<Thumbnailer> {
    let thumbnailer_cache = THUMBNAILER_CACHE.lock().unwrap();
    thumbnailer_cache.get(mime)
}
