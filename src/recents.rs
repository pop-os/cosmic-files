// SPDX-License-Identifier: GPL-3.0-only

//! Updates of `~/.local/share/recently-used.xbel`.
//!
//! Every update reads, parses and rewrites the whole file, which grows with each opened file, so
//! updates never run on the UI thread. Updates made by this process are serialized so that two
//! of them cannot overwrite each other's changes.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;

static LOCK: Mutex<()> = Mutex::new(());

/// Must be held while reading, modifying and writing back the recents file.
pub fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Records in the background that `app_id` opened `paths`.
pub fn add(paths: Vec<PathBuf>, app_id: &'static str) {
    if paths.is_empty() {
        return;
    }
    spawn(move || {
        let _lock = lock();
        for path in &paths {
            // Errors are ignored, as they were when this ran on the UI thread: the file may not
            // exist, and opening the file must not depend on it.
            let _ = recently_used_xbel::update_recently_used(
                path,
                app_id.to_string(),
                "cosmic-files".to_string(),
                None,
            );
        }
    });
}

/// Clears the recents history in the background.
pub fn clear() {
    spawn(|| {
        let _lock = lock();
        if let Err(err) = recently_used_xbel::clear_recently_used() {
            log::warn!("failed to clear recents history: {err}");
        }
    });
}

fn spawn(f: impl FnOnce() + Send + 'static) {
    if let Err(err) = thread::Builder::new().name("recents".into()).spawn(f) {
        log::warn!("failed to spawn recents update thread: {err}");
    }
}
