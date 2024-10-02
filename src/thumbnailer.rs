// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use mime_guess::Mime;
use once_cell::sync::Lazy;
use std::{collections::HashMap, fs, path::Path, process, sync::Mutex, time::Instant};

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
                        command.arg(format!("{}", thumbnail_size));
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
    cache: HashMap<Mime, Vec<Thumbnailer>>,
}

impl ThumbnailerCache {
    pub fn new() -> Self {
        let mut thumbnailer_cache = Self {
            cache: HashMap::new(),
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
        match xdg::BaseDirectories::new() {
            Ok(xdg_dirs) => {
                search_dirs.push(xdg_dirs.get_data_home().join("thumbnailers"));
                for data_dir in xdg_dirs.get_data_dirs() {
                    search_dirs.push(data_dir.join("thumbnailers"));
                }
            }
            Err(err) => {
                log::warn!("failed to get xdg base directories: {}", err);
            }
        }

        let mut thumbnailer_paths = Vec::new();
        for dir in search_dirs {
            log::trace!("looking for thumbnailers in {:?}", dir);
            match fs::read_dir(&dir) {
                Ok(entries) => {
                    for entry_res in entries {
                        match entry_res {
                            Ok(entry) => thumbnailer_paths.push(entry.path()),
                            Err(err) => {
                                log::warn!("failed to read entry in directory {:?}: {}", dir, err);
                            }
                        }
                    }
                }
                Err(err) => {
                    log::warn!("failed to read directory {:?}: {}", dir, err);
                }
            }
        }

        //TODO: handle directory specific behavior
        for path in thumbnailer_paths {
            let entry = match freedesktop_entry_parser::parse_entry(&path) {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!("failed to parse {:?}: {}", path, err);
                    continue;
                }
            };

            //TODO: use TryExec?
            let section = entry.section("Thumbnailer Entry");
            let Some(exec) = section.attr("Exec") else {
                log::warn!("missing Exec attribute for thumbnailer {:?}", path);
                continue;
            };
            let Some(mime_types) = section.attr("MimeType") else {
                log::warn!("missing MimeType attribute for thumbnailer {:?}", path);
                continue;
            };

            for mime_type in mime_types.split_terminator(';') {
                if let Ok(mime) = mime_type.parse::<Mime>() {
                    log::trace!("thumbnailer {}={:?}", mime, path);
                    let apps = self
                        .cache
                        .entry(mime.clone())
                        .or_insert_with(|| Vec::with_capacity(1));
                    apps.push(Thumbnailer {
                        exec: exec.to_string(),
                    });
                }
            }
        }

        let elapsed = start.elapsed();
        log::info!("loaded thumbnailer cache in {:?}", elapsed);
    }

    pub fn get(&self, key: &Mime) -> Vec<Thumbnailer> {
        self.cache
            .get(&key)
            .map_or_else(|| Vec::new(), |x| x.clone())
    }
}

static THUMBNAILER_CACHE: Lazy<Mutex<ThumbnailerCache>> =
    Lazy::new(|| Mutex::new(ThumbnailerCache::new()));

pub fn thumbnailer(mime: &Mime) -> Vec<Thumbnailer> {
    let thumbnailer_cache = THUMBNAILER_CACHE.lock().unwrap();
    thumbnailer_cache.get(mime)
}
