// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::iced::clipboard::mime::{AllowedMimeTypes, AsMimeTypes};
use std::{
    borrow::Cow,
    error::Error,
    path::{Path, PathBuf},
    str,
};
use url::Url;

#[derive(Clone, Copy, Debug)]
pub enum ClipboardKind {
    Copy,
    Cut,
}

pub struct ClipboardCopy {
    pub available: Cow<'static, [String]>,
    pub text_plain: Cow<'static, [u8]>,
    pub text_uri_list: Cow<'static, [u8]>,
    pub x_special_gnome_copied_files: Cow<'static, [u8]>,
}

impl ClipboardCopy {
    pub fn new<P: AsRef<Path>>(kind: ClipboardKind, paths: &[P]) -> Self {
        let available = vec![
            "text/plain".to_string(),
            "text/uri-list".to_string(),
            "x-special/gnome-copied-files".to_string(),
        ];
        let mut text_plain = String::new();
        let mut text_uri_list = String::new();
        let mut x_special_gnome_copied_files = match kind {
            ClipboardKind::Copy => "copy",
            ClipboardKind::Cut => "cut",
        }
        .to_string();
        //TODO: do we have to use \r\n?
        let cr_nl = "\r\n";
        for path in paths.iter() {
            let path = path.as_ref();

            match path.to_str() {
                Some(path_str) => {
                    if !text_plain.is_empty() {
                        text_plain.push_str(cr_nl);
                    }
                    //TOOD: what if the path contains CR or NL?
                    text_plain.push_str(path_str);
                }
                None => {
                    //TODO: allow non-UTF-8?
                    log::warn!(
                        "{:?} is not valid UTF-8, not adding to text/plain clipboard",
                        path
                    );
                }
            }

            match Url::from_file_path(path) {
                Ok(url) => {
                    let url_str = url.as_ref();

                    text_uri_list.push_str(url_str);
                    text_uri_list.push_str(cr_nl);

                    x_special_gnome_copied_files.push('\n');
                    x_special_gnome_copied_files.push_str(url_str);
                }
                Err(()) => {
                    log::warn!(
                        "{:?} cannot be turned into a URL, not adding to text/uri-list clipboard",
                        path
                    );
                }
            }
        }
        Self {
            available: Cow::from(available),
            text_plain: Cow::from(text_plain.into_bytes()),
            text_uri_list: Cow::from(text_uri_list.into_bytes()),
            x_special_gnome_copied_files: Cow::from(x_special_gnome_copied_files.into_bytes()),
        }
    }
}

impl AsMimeTypes for ClipboardCopy {
    fn available(&self) -> Cow<'static, [String]> {
        self.available.clone()
    }

    fn as_bytes(&self, mime_type: &str) -> Option<Cow<'static, [u8]>> {
        match mime_type {
            "text/plain" => Some(self.text_plain.clone()),
            "text/uri-list" => Some(self.text_uri_list.clone()),
            "x-special/gnome-copied-files" => Some(self.x_special_gnome_copied_files.clone()),
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
        Cow::from(vec![
            "x-special/gnome-copied-files".to_string(),
            "text/uri-list".to_string(),
        ])
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
                        Err(()) => Err(format!("invalid file URL {:?}", url))?,
                    }
                }
            }
            "x-special/gnome-copied-files" => {
                let text = str::from_utf8(&data)?;
                for (i, line) in text.lines().enumerate() {
                    if i == 0 {
                        kind = match line {
                            "copy" => ClipboardKind::Copy,
                            "cut" => ClipboardKind::Cut,
                            _ => Err(format!("unsupported clipboard operation {:?}", line))?,
                        };
                    } else {
                        let url = Url::parse(line)?;
                        match url.to_file_path() {
                            Ok(path) => paths.push(path),
                            Err(()) => Err(format!("invalid file URL {:?}", url))?,
                        }
                    }
                }
            }
            _ => Err(format!("unsupported mime type {:?}", mime))?,
        }
        Ok(Self { kind, paths })
    }
}
