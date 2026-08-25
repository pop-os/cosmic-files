// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "desktop")]
use cosmic::desktop::fde::GenericEntry;
use mime_guess::Mime;
use rustc_hash::FxHashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;
use std::{fs, process};

use crate::exec::{Field, for_each_field_code};

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
        let thumbnail_size = thumbnail_size.to_string();
        for arg in args {
            let mut new_arg = std::ffi::OsString::new();
            for_each_field_code(arg, |field| match field {
                Field::Literal(literal) => {
                    new_arg.push(literal);
                    Ok(())
                }
                Field::Code(code) => match code {
                    None => Ok(()),
                    Some('%') => {
                        new_arg.push("%");
                        Ok(())
                    }
                    Some('i' | 'u') => {
                        new_arg.push(input);
                        Ok(())
                    }
                    Some('o') => {
                        new_arg.push(output);
                        Ok(())
                    }
                    Some('s') => {
                        new_arg.push(&thumbnail_size);
                        Ok(())
                    }
                    Some(code) => {
                        log::warn!(
                            "unsupported thumbnailer Exec code '%{code}' in {:?}",
                            self.exec
                        );
                        Err(())
                    }
                },
            })
            .ok()?;
            command.arg(new_arg);
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
                    });
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

#[cfg(test)]
mod tests {
    use super::Thumbnailer;
    use std::path::{Path, PathBuf};

    fn args_of(exec: &str) -> Vec<String> {
        let input = PathBuf::from("/home/user/input.stl");
        let output = PathBuf::from("/home/user/.cache/thumbnails/out.png");
        let command = Thumbnailer { exec: exec.into() }
            .command(&input, &output, 512)
            .expect("should build command");
        std::iter::once(command.get_program().to_string_lossy().into_owned())
            .chain(
                command
                    .get_args()
                    .map(|arg| arg.to_string_lossy().into_owned()),
            )
            .collect()
    }

    #[test]
    fn substitutes_codes_embedded_in_arguments() {
        // f3d thumbnailer from /usr/share/thumbnailers/f3d-plugin-native.thumbnailer
        let args = args_of(
            "f3d --config=thumbnail --load-plugins=native --verbose=quiet --output=%o --resolution=%s,%s %i",
        );
        assert_eq!(
            args,
            [
                "f3d",
                "--config=thumbnail",
                "--load-plugins=native",
                "--verbose=quiet",
                "--output=/home/user/.cache/thumbnails/out.png",
                "--resolution=512,512",
                "/home/user/input.stl",
            ]
        );
    }

    #[test]
    fn substitutes_input_code_with_suffix() {
        // %i glued to a suffix, e.g. ImageMagick page selector (issue #503)
        let args = args_of("convert %i[0] -background \"#FFFFFF\" -flatten -thumbnail %s %o");
        assert_eq!(
            args,
            [
                "convert",
                "/home/user/input.stl[0]",
                "-background",
                "#FFFFFF",
                "-flatten",
                "-thumbnail",
                "512",
                "/home/user/.cache/thumbnails/out.png",
            ]
        );
    }

    #[test]
    fn double_percent_is_literal_percent() {
        let args = args_of("foo --scale=100%% %i");
        assert_eq!(args, ["foo", "--scale=100%", "/home/user/input.stl"]);
    }

    #[test]
    fn trailing_percent_is_ignored() {
        let args = args_of("foo --label=% %i");
        assert_eq!(args, ["foo", "--label=", "/home/user/input.stl"]);
    }

    #[test]
    fn rejects_unknown_codes_embedded_in_arguments() {
        let command = Thumbnailer {
            exec: "foo --output=%o --color=%z %i".into(),
        }
        .command(Path::new("/in"), Path::new("/out"), 512);
        assert!(command.is_none());
    }
}
