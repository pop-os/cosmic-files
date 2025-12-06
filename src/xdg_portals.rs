// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Integrations with XDG portals.

use ashpd::{
    desktop::inhibit::{InhibitFlags, InhibitProxy},
    enumflags2::{BitFlags, make_bitflags},
};
use log::{debug, warn};
use tokio::sync::watch::Receiver;

// Actions to inhibit. Currently, COSMIC defaults to the GTK portal for Inhibit. That
// implementation only supports inhibiting idling and trying to inhibit anything else causes the
// D-Bus call to silently fail. We will only inhibit idling until COSMIC gets a bespoke Inhibit.
const INHIBIT_FLAGS: BitFlags<InhibitFlags> = make_bitflags!(InhibitFlags::{Idle});

/// Inhibit idle and user switching while media is played.
///
/// # Usage
/// Enable the inhibitor by setting the watcher to `true`. Disable the inhibitor by sending a
/// `false`. Sending multiple consecutive trues/falses is safe and guarded internally.
///
/// Portal:
/// https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Inhibit.html
pub async fn inhibit(mut signal: Receiver<bool>) -> ashpd::Result<()> {
    let proxy = InhibitProxy::new().await?;
    // Mark the watcher's value as unseen so a temporary or mutable bool isn't needed in the loop.
    signal.mark_changed();

    loop {
        if signal.wait_for(|&status| status).await.is_err() {
            // The watcher will likely only be closed when the app is closed.
            debug!("Inhibit task's watcher is closed");
            break;
        }
        // Copying the bool is important or else we would needlessly hold the lock below.
        let should_inhibit = *signal.borrow_and_update();

        if should_inhibit
            && let Some(inhibit_handle) = proxy
                .inhibit(None, INHIBIT_FLAGS, "")
                .await
                .inspect_err(|e| warn!("Failed to call inhibit portal endpoint: {e}"))
                .ok()
        {
            // We don't have to check the bool because it's already checked to be false in the
            // closure. We also don't have to break on error because the next iteration of the loop
            // would break anyway.
            let _ = signal.wait_for(|&status| !status).await;
            if let Err(e) = inhibit_handle.close().await {
                // This should only happen if the inhibit portal silently fails which GTK (and
                // others!) apparently do.
                warn!("Removing the inhibitor failed: {e}");
                break;
            }
        }
    }

    Ok(())
}
