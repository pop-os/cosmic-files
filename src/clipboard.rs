// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::iced::clipboard::mime::{AllowedMimeTypes, AsMimeTypes};
use std::{
    borrow::Cow,
    error::Error,
    fs,
    path::{Path, PathBuf},
    str,
    sync::LazyLock,
};
use url::Url;

//TODO: do we have to use \r\n?
const CR_NL: &'static str = "\r\n";

#[derive(Clone, Copy, Debug)]
pub enum ClipboardKind {
    Copy,
    Cut { is_dnd: bool },
}

#[derive(Clone, Debug)]
pub struct ClipboardCopy {
    kind: ClipboardKind,
    paths: Vec<PathBuf>,
}

impl ClipboardCopy {
    pub fn new<P: AsRef<Path>>(kind: ClipboardKind, paths: impl IntoIterator<Item = P>) -> Self {
        let paths: Vec<PathBuf> = paths.into_iter().map(|x| x.as_ref().to_owned()).collect();
        Self { kind, paths }
    }
}

impl AsMimeTypes for ClipboardCopy {
    fn available(&self) -> Cow<'static, [String]> {
        static AVAILABLE: LazyLock<Vec<String>> = LazyLock::new(|| {
            vec![
                "text/plain".to_string(),
                "text/plain;charset=utf-8".to_string(),
                "UTF8_STRING".to_string(),
                "text/uri-list".to_string(),
                "x-special/gnome-copied-files".to_string(),
                "application/vnd.portal.filetransfer".to_string(),
                "application/vnd.portal.files".to_string(),
            ]
        });
        Cow::Borrowed(&AVAILABLE)
    }

    fn as_bytes(&self, mime_type: &str) -> Option<Cow<'static, [u8]>> {
        match mime_type {
            "text/plain" | "text/plain;charset=utf-8" | "UTF8_STRING" => {
                let mut text_plain = String::new();
                for path in &self.paths {
                    match path.to_str() {
                        Some(path_str) => {
                            if !text_plain.is_empty() {
                                text_plain.push_str(CR_NL);
                            }
                            //TODO: what if the path contains CR or NL?
                            text_plain.push_str(path_str);
                        }
                        None => {
                            //TODO: allow non-UTF-8?
                            log::warn!(
                                "{} is not valid UTF-8, not adding to text/plain clipboard",
                                path.display()
                            );
                        }
                    }
                }
                Some(Cow::from(text_plain.into_bytes()))
            }
            "text/uri-list" => {
                let mut text_uri_list = String::new();
                for path in &self.paths {
                    match Url::from_file_path(path) {
                        Ok(url) => {
                            text_uri_list.push_str(url.as_ref());
                            text_uri_list.push_str(CR_NL);
                        }
                        Err(()) => {
                            log::warn!(
                                "{} cannot be turned into a URL, not adding to text/uri-list clipboard",
                                path.display()
                            );
                        }
                    }
                }
                Some(Cow::from(text_uri_list.into_bytes()))
            }
            "x-special/gnome-copied-files" => {
                let mut x_special_gnome_copied_files = match self.kind {
                    ClipboardKind::Copy => "copy",
                    ClipboardKind::Cut { .. } => "cut",
                }
                .to_string();
                for path in &self.paths {
                    match Url::from_file_path(path) {
                        Ok(url) => {
                            x_special_gnome_copied_files.push('\n');
                            x_special_gnome_copied_files.push_str(url.as_ref());
                        }
                        Err(()) => {
                            log::warn!(
                                "{} cannot be turned into a URL, not adding to text/uri-list clipboard",
                                path.display()
                            );
                        }
                    }
                }
                Some(Cow::from(x_special_gnome_copied_files.into_bytes()))
            }
            "application/vnd.portal.filetransfer" | "application/vnd.portal.files" => {
                let res: ashpd::Result<String> = futures::executor::block_on(async {
                    let mut files = Vec::new();
                    for path in &self.paths {
                        match fs::File::open(path) {
                            Ok(file) => files.push(file),
                            Err(err) => log::warn!(
                                "{} cannot be opened: {}, not adding to portal file transfer clipboard",
                                path.display(),
                                err
                            ),
                        }
                    }
                    let file_transfer = ashpd::documents::FileTransfer::new().await?;
                    let key = file_transfer.start_transfer(false, true).await?; // XXX args
                    file_transfer.add_files(&key, &files).await?;
                    Ok(key)
                });
                match res {
                    Ok(key) => Some(Cow::from(key.into_bytes())),
                    Err(err) => {
                        log::warn!("failed to use file transfer portal: {}", err);
                        None
                    }
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClipboardPaste {
    pub kind: ClipboardKind,
    pub paths: Vec<PathBuf>,
}

impl AllowedMimeTypes for ClipboardPaste {
    fn allowed() -> Cow<'static, [String]> {
        static ALLOWED: LazyLock<Vec<String>> = LazyLock::new(|| {
            vec![
                "x-special/gnome-copied-files".to_string(),
                "text/uri-list".to_string(),
            ]
        });
        Cow::Borrowed(&ALLOWED)
    }
}

impl TryFrom<(Vec<u8>, String)> for ClipboardPaste {
    type Error = Box<dyn Error>;
    fn try_from(value: (Vec<u8>, String)) -> Result<Self, Self::Error> {
        let (data, mime) = value;
        // Assume the kind is Copy if not provided by the mime type
        let mut kind = ClipboardKind::Copy;
        let mut paths = Vec::new();
        match mime.as_str() {
            "text/uri-list" => {
                let text = str::from_utf8(&data)?;
                for line in text.lines() {
                    let url = Url::parse(line)?;
                    match url.to_file_path() {
                        Ok(path) => paths.push(path),
                        Err(()) => Err(format!("invalid file URL {url:?}"))?,
                    }
                }
            }
            "x-special/gnome-copied-files" => {
                let text = str::from_utf8(&data)?;
                for (i, line) in text.lines().enumerate() {
                    if i == 0 {
                        kind = match line {
                            "copy" => ClipboardKind::Copy,
                            "cut" => ClipboardKind::Cut { is_dnd: false },
                            _ => Err(format!("unsupported clipboard operation {line:?}"))?,
                        };
                    } else {
                        let url = Url::parse(line)?;
                        match url.to_file_path() {
                            Ok(path) => paths.push(path),
                            Err(()) => Err(format!("invalid file URL {url:?}"))?,
                        }
                    }
                }
            }
            _ => Err(format!("unsupported mime type {mime:?}"))?,
        }
        Ok(Self { kind, paths })
    }
}
