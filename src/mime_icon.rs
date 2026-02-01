// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use mime_guess::Mime;
use rustc_hash::FxHashMap;
use std::{
    fs,
    io::Read,
    path::Path,
    sync::{LazyLock, Mutex},
};

pub const FALLBACK_MIME_ICON: &str = "text-x-generic";

#[derive(Debug, Eq, Hash, PartialEq)]
struct MimeIconKey {
    mime: Mime,
    size: u16,
}

struct MimeIconCache {
    cache: FxHashMap<MimeIconKey, Option<icon::Handle>>,
    shared_mime_info: xdg_mime::SharedMimeInfo,
}

impl MimeIconCache {
    pub fn new() -> Self {
        Self {
            cache: FxHashMap::default(),
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
                    let fallback_names =
                        icon_names.into_iter().map(std::borrow::Cow::from).collect();
                    named = named.fallback(Some(icon::IconFallback::Names(fallback_names)));
                }
                Some(named.handle())
            })
            .clone()
    }
}
static MIME_ICON_CACHE: LazyLock<Mutex<MimeIconCache>> =
    LazyLock::new(|| Mutex::new(MimeIconCache::new()));

pub fn mime_for_path(
    path: impl AsRef<Path>,
    metadata_opt: Option<&fs::Metadata>,
    remote: bool,
) -> Mime {
    let mime_icon_cache = MIME_ICON_CACHE.lock().unwrap();
    mime_for_path_with_cache(path.as_ref(), metadata_opt, remote, &mime_icon_cache)
}

fn mime_for_path_with_cache(
    path: &Path,
    metadata_opt: Option<&fs::Metadata>,
    remote: bool,
    mime_icon_cache: &std::sync::MutexGuard<MimeIconCache>,
) -> Mime {
    // Try the shared mime info cache first
    let mut gb = mime_icon_cache.shared_mime_info.guess_mime_type();
    if remote {
        if let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str) {
            gb.file_name(file_name);
        }
    } else {
        gb.path(path);
    }
    if let Some(metadata) = metadata_opt {
        gb.metadata(metadata.clone());
    }
    let guess = gb.guess();
    let guessed_mime = guess.mime_type();

    /// Checks if the `Mime` is a special variant returned by `xdg-mime`.
    /// This includes directories, symlinks and zerosize files, which are returned as uncertain.
    fn is_special_mime(mime: &Mime) -> bool {
        *mime == "inode/directory" || *mime == "inode/symlink" || *mime == "application/x-zerosize"
    }

    // `xdg-mime-rs` sets the guess to uncertain if it returns special mime types.
    // The guess could also be uncertain on platforms without shared-mime-info.
    // Try mime_guess, but only if it is not one of the special mime types.
    if guess.uncertain() && (remote || !is_special_mime(guessed_mime)) {
        // If uncertain, try mime_guess. This could happen on platforms without shared-mime-info
        mime_guess::from_path(path).first_or_octet_stream()
    } else {
        guessed_mime.clone()
    }
}

pub fn mime_for_bytes_or_path(path: impl AsRef<Path>) -> Mime {
    let path = path.as_ref();
    let mime_icon_cache = MIME_ICON_CACHE.lock().unwrap();
    let mime_from_data = mime_for_bytes_with_cache(path, &mime_icon_cache);
    mime_from_data.unwrap_or_else(|_| mime_for_path_with_cache(path, None, false, &mime_icon_cache))
}

fn mime_for_bytes_with_cache(
    path: &Path,
    mime_icon_cache: &std::sync::MutexGuard<MimeIconCache>,
) -> Result<Mime, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0u8; 1024];
    let bytes_read = file.read(&mut buffer)?;

    mime_icon_cache
        .shared_mime_info
        .get_mime_type_for_data(&buffer[0..bytes_read])
        // limit to certain matches which are defined as > 80 per `xdg-mime-rs` docs
        .filter(|(_mime, priority)| *priority > 80)
        .map(|(mime, _priority)| mime.clone())
        // If uncertain, try `infer`. This could happen on platforms without shared-mime-info.
        // We use `infer` instead of `mime_guess` because the former is for magic bytes, the
        // latter is for filenames.
        .or_else(|| {
            infer::get_from_path(path)
                .ok()
                .flatten()
                .and_then(|m| m.mime_type().parse::<mime_guess::Mime>().ok())
        })
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not determine MIME type from magic bytes",
            )
        })
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
