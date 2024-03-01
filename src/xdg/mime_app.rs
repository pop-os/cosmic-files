// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

pub use cosmic::desktop::DesktopEntryData;
use cosmic::desktop::{load_applications, Mime};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex, time::Instant};

pub struct MimeAppCache {
    cache: HashMap<Mime, Vec<DesktopEntryData>>,
    empty: Vec<DesktopEntryData>,
}

impl MimeAppCache {
    pub fn new() -> Self {
        let mut mime_app_cache = Self {
            cache: HashMap::new(),
            empty: Vec::new(),
        };
        mime_app_cache.reload();
        mime_app_cache
    }

    pub fn reload(&mut self) {
        let start = Instant::now();

        self.cache.clear();

        //TODO: get proper locale?
        let locale = None;
        for app in load_applications(locale, false) {
            for mime_type in app.mime_types.iter() {
                self.cache
                    .entry(mime_type.clone())
                    .or_insert_with(|| Vec::with_capacity(1))
                    .push(app.clone());
            }
        }

        for apps in self.cache.values_mut() {
            apps.sort_by(|a, b| lexical_sort::natural_lexical_cmp(&a.name, &b.name));
        }

        let elapsed = start.elapsed();
        log::info!("loaded mime app cache in {:?}", elapsed);
    }

    pub fn get(&self, key: &Mime) -> &Vec<DesktopEntryData> {
        self.cache.get(&key).unwrap_or_else(|| &self.empty)
    }
}

static MIME_APP_CACHE: Lazy<Mutex<MimeAppCache>> = Lazy::new(|| Mutex::new(MimeAppCache::new()));

pub fn mime_apps(mime: &Mime) -> Vec<DesktopEntryData> {
    let mime_app_cache = MIME_APP_CACHE.lock().unwrap();
    mime_app_cache.get(mime).clone()
}
