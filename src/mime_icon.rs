// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use mime_guess::Mime;
use once_cell::sync::Lazy;
use std::{collections::HashMap, fs, path::Path, sync::Mutex};

pub const FALLBACK_MIME_ICON: &str = "text-x-generic";

#[derive(Debug, Eq, Hash, PartialEq)]
struct MimeIconKey {
    mime: Mime,
    size: u16,
}

struct MimeIconCache {
    cache: HashMap<MimeIconKey, Option<icon::Handle>>,
    shared_mime_info: xdg_mime::SharedMimeInfo,
}

impl MimeIconCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            shared_mime_info: xdg_mime::SharedMimeInfo::new(),
        }
    }

    pub fn get(&mut self, key: MimeIconKey) -> Option<icon::Handle> {
        self.cache
            .entry(key)
            .or_insert_with_key(|key| {
                let mut icon_names = self.shared_mime_info.lookup_icon_names(&key.mime);
                if icon_names.is_empty() {
                    return None;
                }
                let icon_name = icon_names.remove(0);
                let mut named = icon::from_name(icon_name).size(key.size);
                if !icon_names.is_empty() {
                    let mut fallback_names = Vec::with_capacity(icon_names.len());
                    for fallback_name in icon_names {
                        fallback_names.push(fallback_name.into());
                    }
                    named = named.fallback(Some(icon::IconFallback::Names(fallback_names)));
                }
                Some(named.handle())
            })
            .clone()
    }
}
static MIME_ICON_CACHE: Lazy<Mutex<MimeIconCache>> = Lazy::new(|| Mutex::new(MimeIconCache::new()));

pub fn mime_for_path<P: AsRef<Path>>(
    path: P,
    metadata_opt: Option<&fs::Metadata>,
    remote: bool,
) -> Mime {
    let path = path.as_ref();
    let mime_icon_cache = MIME_ICON_CACHE.lock().unwrap();
    // Try the shared mime info cache first
    let mut gb = mime_icon_cache.shared_mime_info.guess_mime_type();
    if remote {
        if let Some(file_name) = path.file_name().and_then(|x| x.to_str()) {
            gb.file_name(file_name);
        }
    } else {
        gb.path(&path);
    }
    if let Some(metadata) = metadata_opt {
        gb.metadata(metadata.clone());
    }
    let guess = gb.guess();
    if guess.uncertain() {
        // If uncertain, try mime_guess. This could happen on platforms without shared-mime-info
        mime_guess::from_path(&path).first_or_octet_stream()
    } else {
        guess.mime_type().clone()
    }
}

pub fn mime_icon(mime: Mime, size: u16) -> icon::Handle {
    let mut mime_icon_cache = MIME_ICON_CACHE.lock().unwrap();
    match mime_icon_cache.get(MimeIconKey { mime, size }) {
        Some(handle) => handle,
        None => icon::from_name(FALLBACK_MIME_ICON).size(size).handle(),
    }
}

pub fn parent_mime_types(mime: &Mime) -> Option<Vec<Mime>> {
    let mime_icon_cache = MIME_ICON_CACHE.lock().unwrap();

    mime_icon_cache.shared_mime_info.get_parents_aliased(mime)
}
