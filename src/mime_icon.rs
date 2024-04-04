// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{config::CosmicTk, cosmic_config::ConfigGet, widget::icon};
use mime_guess::Mime;
use once_cell::sync::Lazy;
use std::{collections::HashMap, path::Path, sync::Mutex};

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
                for icon_name in self.shared_mime_info.lookup_icon_names(&key.mime) {
                    if let Some(path) = icon::from_name(icon_name)
                        .prefer_svg(true)
                        .size(key.size)
                        .path()
                    {
                        return Some(icon::from_path(path));
                    }
                }
                None
            })
            .clone()
    }
}
static MIME_ICON_CACHE: Lazy<Mutex<MimeIconCache>> = Lazy::new(|| Mutex::new(MimeIconCache::new()));

pub fn mime_for_path<P: AsRef<Path>>(path: P) -> Mime {
    let mime_icon_cache = MIME_ICON_CACHE.lock().unwrap();
    // Try the shared mime info cache first
    let guess = mime_icon_cache
        .shared_mime_info
        .guess_mime_type()
        .path(&path)
        .guess();
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

pub fn fallback_theme() -> Option<String> {
    // Icon precedence
    // 1. User's preferred theme (automatic via libcosmic)
    // 2. COSMIC default (automatic)
    // 3. GTK default
    if CosmicTk::config()
        .and_then(|config| config.get::<String>("icon_theme"))
        .is_err()
        && icon::from_name("folder").path().is_none()
    {
        log::warn!("COSMIC's default theme is missing, but it should be available as part of a correct installation");

        // Try GTK as a last resort.
        let icon_theme = gtk_icon_theme();
        if icon_theme.is_none() {
            log::warn!("Could not find a valid icon theme. Install COSMIC's icon theme or set a default with cosmic-settings.");
        }
        icon_theme
    } else {
        None
    }
}

fn gtk_icon_theme() -> Option<String> {
    let gsettings = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "icon-theme"])
        .output()
        .ok()?;

    if gsettings.status.success() {
        let name = String::from_utf8(gsettings.stdout)
            .ok()?
            .trim()
            .trim_matches('\'')
            .to_owned();
        Some(name)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use cosmic::widget;

    use super::fallback_theme;

    #[test]
    fn basic_icon_lookup_never_fails() {
        if let Some(gtk_theme) = fallback_theme() {
            cosmic::icon_theme::set_default(gtk_theme);
        }

        // Fallback
        widget::icon::from_name("text-x-generic")
            .path()
            .expect("`text-x-generic` should always be found");

        // Trash
        widget::icon::from_name("user-trash-full-symbolic")
            .path()
            .expect("`user-trash-full-symbolic` should always be found");
        widget::icon::from_name("user-trash-symbolic")
            .path()
            .expect("`user-trash-symbolic` should always be found");

        // Folders
        widget::icon::from_name("folder")
            .path()
            .expect("`folder` should always be found");
        widget::icon::from_name("folder-documents-symbolic")
            .path()
            .expect("`folder-documents-symbolic` should always be found");
        widget::icon::from_name("folder-download-symbolic")
            .path()
            .expect("`folder-documents-symbolic` should always be found");
        widget::icon::from_name("folder-music-symbolic")
            .path()
            .expect("`folder-music-symbolic` should always be found");
        widget::icon::from_name("folder-pictures-symbolic")
            .path()
            .expect("`folder-pictures-symbolic` should always be found");
        widget::icon::from_name("folder-publicshare-symbolic")
            .path()
            .expect("`folder-publicshare-symbolic` should always be found");
        widget::icon::from_name("folder-templates-symbolic")
            .path()
            .expect("`folder-templates-symbolic` should always be found");
        widget::icon::from_name("folder-videos-symbolic")
            .path()
            .expect("`folder-videos-symbolic` should always be found");
        widget::icon::from_name("user-desktop-symbolic")
            .path()
            .expect("`user-desktop-symbolic` should always be found");
        widget::icon::from_name("user-home-symbolic")
            .path()
            .expect("`user-home-symblic` should always be found");
    }
}
