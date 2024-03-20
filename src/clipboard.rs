// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::iced::clipboard::mime::AsMimeTypes;
use std::{borrow::Cow, path::Path};
use url::Url;

pub enum ClipboardKind {
    Copy,
    Cut,
}

pub struct ClipboardContents {
    pub available: Cow<'static, [String]>,
    pub text_plain: Cow<'static, [u8]>,
    pub text_uri_list: Cow<'static, [u8]>,
    pub x_special_gnome_copied_files: Cow<'static, [u8]>,
}

impl ClipboardContents {
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
                    let url_str = url.to_string();

                    text_uri_list.push_str(&url_str);
                    text_uri_list.push_str(cr_nl);

                    x_special_gnome_copied_files.push('\n');
                    x_special_gnome_copied_files.push_str(&url_str);
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

impl AsMimeTypes for ClipboardContents {
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
