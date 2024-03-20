// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::iced::clipboard::mime::AsMimeTypes;
use std::{borrow::Cow, path::Path};
use url::Url;

pub struct ClipboardContents {
    pub available: Cow<'static, [String]>,
    pub text_plain: Cow<'static, [u8]>,
    pub text_uri_list: Cow<'static, [u8]>,
}

impl ClipboardContents {
    pub fn new<P: AsRef<Path>>(paths: &[P]) -> Self {
        let available = vec!["text/plain".to_string(), "text/uri-list".to_string()];
        let mut text_plain = String::new();
        let mut text_uri_list = String::new();
        //TODO: do we have to use \r\n?
        let newline = "\r\n";
        for path in paths.iter() {
            let path = path.as_ref();

            match path.to_str() {
                Some(path_str) => {
                    if !text_plain.is_empty() {
                        text_plain.push_str(newline);
                    }

                    //TOOD: what if the path contains a newline?
                    text_plain.push_str(path_str);
                }
                None => {
                    log::warn!(
                        "{:?} is not valid UTF-8, not adding to text/plain clipboard",
                        path
                    );
                }
            }

            match Url::from_file_path(path) {
                Ok(url) => {
                    text_uri_list.push_str(&url.to_string());
                    text_uri_list.push_str(newline);
                }
                Err(err) => {
                    log::warn!(
                        "{:?} cannot be turned into a URL, not adding to text/uri-list clipboard: {}",
                        path, err
                    );
                }
            }
        }
        Self {
            available: Cow::from(available),
            text_plain: Cow::from(text_plain.into_bytes()),
            text_uri_list: Cow::from(text_uri_list.into_bytes()),
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
            _ => None,
        }
    }
}
