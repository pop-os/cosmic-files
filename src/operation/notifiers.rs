// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::Notify;

/// Monitor files which are being written to.
pub struct FileWritingNotifier {
    data: Vec<PathBuf>,
    notify: Arc<Notify>,
}

static ACTIVELY_WRITING: LazyLock<Mutex<FileWritingNotifier>> = LazyLock::new(|| {
    Mutex::new(FileWritingNotifier {
        data: Vec::new(),
        notify: Arc::new(Notify::new()),
    })
});

/// Append path that is being written to.
pub fn actively_writing_add(path: PathBuf) {
    ACTIVELY_WRITING.lock().unwrap().data.push(path);
}

/// Remove path to file that has finished writing and notify waiters.
pub fn actively_writing_remove(path: &Path) {
    let mut guard = ACTIVELY_WRITING.lock().unwrap();
    guard.data.retain(|p| p != path);
    guard.notify.notify_waiters();
}

/// Wait until the actively-writing queue is empty or a file has been removed.
pub async fn actively_writing_tick() {
    let notify = (|| {
        let guard = ACTIVELY_WRITING.lock().unwrap();

        if !guard.data.is_empty() {
            return Some(guard.notify.clone());
        }

        None
    })();

    if let Some(notify) = notify {
        notify.notified().await
    }
}

/// Check if a file is being written to. Avoid thumbnail generation until after it is finished.
pub fn is_actively_writing_to(path: &Path) -> bool {
    ACTIVELY_WRITING
        .lock()
        .unwrap()
        .data
        .iter()
        .any(|p| p == path)
}
