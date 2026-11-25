// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;

static LOCK: Mutex<()> = Mutex::new(());

pub fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn add(paths: Vec<PathBuf>, app_id: &'static str) {
    if paths.is_empty() {
        return;
    }
    spawn(move || {
        let _lock = lock();
        for path in &paths {
            let _ = recently_used_xbel::update_recently_used(
                path,
                app_id.to_string(),
                "cosmic-files".to_string(),
                None,
            );
        }
    });
}

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
