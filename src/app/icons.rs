// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(crate) static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: &'static str,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        macro_rules! bundle {
            ($name:expr, $size:expr) => {
                let data: &'static [u8] =
                    include_bytes!(concat!("../../res/icons/bundled/", $name, ".svg"));
                cache.insert(
                    IconCacheKey {
                        name: $name,
                        size: $size,
                    },
                    icon::from_svg_bytes(data).symbolic(true),
                );
            };
        }

        bundle!("tab-new-filled-symbolic", 14);
        bundle!("value-increase-symbolic", 14);
        bundle!("value-decrease-symbolic", 14);
        bundle!("loupe-symbolic", 14);
        bundle!("folder-symbolic", 14);
        bundle!("folder-new-symbolic", 14);
        bundle!("edit-copy-symbolic", 14);
        bundle!("paper-symbolic", 14);
        bundle!("document-open-symbolic", 14);
        bundle!("arrow-into-box-symbolic", 14);
        bundle!("edit-symbolic", 14);
        bundle!("user-trash-symbolic", 14);
        bundle!("cross-small-square-filled-symbolic", 14);
        bundle!("external-link-symbolic", 14);
        bundle!("cut-symbolic", 14);
        bundle!("copy-symbolic", 14);
        bundle!("clipboard-symbolic", 14);
        bundle!("edit-select-all-symbolic", 14);
        bundle!("history-undo-symbolic", 14);
        bundle!("grid-symbolic", 14);
        bundle!("list-large-symbolic", 14);
        bundle!("view-conceal-symbolic", 14);
        bundle!("settings-symbolic", 14);
        bundle!("info-outline-symbolic", 14);
        bundle!("dock-left-symbolic", 14);
        bundle!("arrow-into-box-symbolic", 14);
        bundle!("image-round-symbolic", 14);
        bundle!("terminal-symbolic", 14);
        bundle!("symbolic-link-symbolic", 14);
        bundle!("package-x-generic-symbolic", 14);
        bundle!("archive-extract-symbolic", 14);
        bundle!("brush-monitor-symbolic", 14);
        bundle!("display-symbolic", 14);
        bundle!("shell-overview-symbolic", 14);
        bundle!("empty-trash-bin-symbolic", 14);

        Self { cache }
    }

    pub fn get_icon(&mut self, name: &'static str, size: u16) -> icon::Icon {
        let handle = self
            .cache
            .entry(IconCacheKey { name, size })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        icon::icon(handle).size(size)
    }

    pub fn get_handle(&mut self, name: &'static str, size: u16) -> icon::Handle {
        let handle = self
            .cache
            .entry(IconCacheKey { name, size })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        handle
    }
}

pub fn get_icon(name: &'static str, size: u16) -> icon::Icon {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get_icon(name, size)
}

pub fn get_handle(name: &'static str, size: u16) -> icon::Handle {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get_handle(name, size)
}
