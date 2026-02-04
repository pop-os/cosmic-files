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
    Cut { is_dnd: bool },
}

#[derive(Clone, Debug)]
pub struct ClipboardCopy {
    pub available: Cow<'static, [String]>,
    pub text_plain: Cow<'static, [u8]>,
    pub text_uri_list: Cow<'static, [u8]>,
    pub x_special_gnome_copied_files: Cow<'static, [u8]>,
}

impl ClipboardCopy {
    pub fn new<P: AsRef<Path>>(kind: ClipboardKind, paths: impl IntoIterator<Item = P>) -> Self {
        let available = vec![
            "text/plain".to_string(),
            "text/plain;charset=utf-8".to_string(),
            "UTF8_STRING".to_string(),
            "text/uri-list".to_string(),
            "x-special/gnome-copied-files".to_string(),
        ];
        let mut text_plain = String::new();
        let mut text_uri_list = String::new();
        let mut x_special_gnome_copied_files = match kind {
            ClipboardKind::Copy => "copy",
            ClipboardKind::Cut { .. } => "cut",
        }
        .to_string();
        //TODO: do we have to use \r\n?
        let cr_nl = "\r\n";
        for path in paths {
            let path = path.as_ref();

            match path.to_str() {
                Some(path_str) => {
                    if !text_plain.is_empty() {
                        text_plain.push_str(cr_nl);
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
                        "{} cannot be turned into a URL, not adding to text/uri-list clipboard",
                        path.display()
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
            "text/plain" | "text/plain;charset=utf-8" | "UTF8_STRING" => {
                Some(self.text_plain.clone())
            }
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

/// Image data from clipboard for pasting as a new file.
#[derive(Clone, Debug)]
pub struct ClipboardPasteImage {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl AllowedMimeTypes for ClipboardPasteImage {
    fn allowed() -> Cow<'static, [String]> {
        Cow::from(vec![
            "image/png".to_string(),
            "image/jpeg".to_string(),
            "image/gif".to_string(),
            "image/bmp".to_string(),
            "image/webp".to_string(),
            "image/tiff".to_string(),
            "image/x-tiff".to_string(),
            "image/svg+xml".to_string(),
            "image/x-icon".to_string(),
            "image/vnd.microsoft.icon".to_string(),
            "image/x-bmp".to_string(),
            "image/x-ms-bmp".to_string(),
            "image/pjpeg".to_string(),
            "image/x-png".to_string(),
            "image/avif".to_string(),
            "image/heic".to_string(),
            "image/heif".to_string(),
            "image/jxl".to_string(),
        ])
    }
}

impl TryFrom<(Vec<u8>, String)> for ClipboardPasteImage {
    type Error = Box<dyn Error>;
    fn try_from(value: (Vec<u8>, String)) -> Result<Self, Self::Error> {
        let (data, mime) = value;
        if data.is_empty() {
            return Err("Empty image data".into());
        }
        Ok(Self {
            data,
            mime_type: mime,
        })
    }
}

impl ClipboardPasteImage {
    /// Get the file extension for the image based on MIME type.
    /// Returns None if the MIME type is not recognized.
    pub fn extension(&self) -> Option<&'static str> {
        match self.mime_type.as_str() {
            "image/png" | "image/x-png" => Some("png"),
            "image/jpeg" | "image/pjpeg" => Some("jpg"),
            "image/gif" => Some("gif"),
            "image/bmp" | "image/x-bmp" | "image/x-ms-bmp" => Some("bmp"),
            "image/webp" => Some("webp"),
            "image/tiff" | "image/x-tiff" => Some("tiff"),
            "image/svg+xml" => Some("svg"),
            "image/x-icon" | "image/vnd.microsoft.icon" => Some("ico"),
            "image/avif" => Some("avif"),
            "image/heic" => Some("heic"),
            "image/heif" => Some("heif"),
            "image/jxl" => Some("jxl"),
            _ => None,
        }
    }
}

/// Video data from clipboard for pasting as a new file.
#[derive(Clone, Debug)]
pub struct ClipboardPasteVideo {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl AllowedMimeTypes for ClipboardPasteVideo {
    fn allowed() -> Cow<'static, [String]> {
        Cow::from(vec![
            "video/mp4".to_string(),
            "video/webm".to_string(),
            "video/ogg".to_string(),
            "video/mpeg".to_string(),
            "video/quicktime".to_string(),
            "video/x-msvideo".to_string(),
            "video/x-matroska".to_string(),
            "video/x-flv".to_string(),
            "video/3gpp".to_string(),
            "video/3gpp2".to_string(),
            "video/x-ms-wmv".to_string(),
            "video/avi".to_string(),
        ])
    }
}

impl TryFrom<(Vec<u8>, String)> for ClipboardPasteVideo {
    type Error = Box<dyn Error>;
    fn try_from(value: (Vec<u8>, String)) -> Result<Self, Self::Error> {
        let (data, mime) = value;
        if data.is_empty() {
            return Err("Empty video data".into());
        }
        Ok(Self {
            data,
            mime_type: mime,
        })
    }
}

impl ClipboardPasteVideo {
    /// Get the file extension for the video based on MIME type.
    /// Returns None if the MIME type is not recognized.
    pub fn extension(&self) -> Option<&'static str> {
        match self.mime_type.as_str() {
            "video/mp4" => Some("mp4"),
            "video/webm" => Some("webm"),
            "video/ogg" => Some("ogv"),
            "video/mpeg" => Some("mpeg"),
            "video/quicktime" => Some("mov"),
            "video/x-msvideo" | "video/avi" => Some("avi"),
            "video/x-matroska" => Some("mkv"),
            "video/x-flv" => Some("flv"),
            "video/3gpp" => Some("3gp"),
            "video/3gpp2" => Some("3g2"),
            "video/x-ms-wmv" => Some("wmv"),
            _ => None,
        }
    }
}

/// Text data from clipboard for pasting as a new text file.
#[derive(Clone, Debug)]
pub struct ClipboardPasteText {
    pub data: String,
}

impl AllowedMimeTypes for ClipboardPasteText {
    fn allowed() -> Cow<'static, [String]> {
        Cow::from(vec![
            "text/plain".to_string(),
            "text/plain;charset=utf-8".to_string(),
            "UTF8_STRING".to_string(),
            "STRING".to_string(),
            "TEXT".to_string(),
        ])
    }
}

impl TryFrom<(Vec<u8>, String)> for ClipboardPasteText {
    type Error = Box<dyn Error>;
    fn try_from(value: (Vec<u8>, String)) -> Result<Self, Self::Error> {
        let (data, _mime) = value;
        if data.is_empty() {
            return Err("Empty text data".into());
        }
        // Use lossy conversion to handle clipboard data that may contain
        // invalid UTF-8 (e.g., Latin-1 encoded special characters from browsers)
        let text = String::from_utf8_lossy(&data);
        Ok(Self {
            data: text.into_owned(),
        })
    }
}
