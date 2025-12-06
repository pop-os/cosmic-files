// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Integrations with XDG portals.

use ashpd::{
    desktop::inhibit::{InhibitFlags, InhibitProxy, SessionState},
    enumflags2::{BitFlags, make_bitflags},
};
use futures::StreamExt;
use log::warn;
use tokio::sync::watch::Receiver;

const INHIBIT_FLAGS: BitFlags<InhibitFlags> =
    make_bitflags!(InhibitFlags::{Logout | UserSwitch | Suspend});

/// Inhibit suspension and shutdown while file operations are in progress.
///
/// # Usage
/// Enable the inhibitor by setting the watcher to `true`. Disable the inhibitor by sending a
/// `false`. Sending multiple consecutive trues/falses is safe and guarded internally.
///
/// Portal:
/// https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Inhibit.html
pub async fn inhibit(mut signal: Receiver<bool>) -> ashpd::Result<()> {
    let proxy = InhibitProxy::new().await?;
    let session = proxy.create_monitor(None).await?;
    // Mark the watcher's value as unseen so we don't need a temporary bool and branch for the
    // initial state.
    signal.mark_changed();

    let mut states = proxy.receive_state_changed().await?;
    while let Some(SessionState::QueryEnd) = states.next().await.map(|state| state.session_state())
    {
        // Copying the bool is important or else we would needlessly hold the lock below.
        let should_inhibit = *signal.borrow_and_update();

        if should_inhibit {
            // XXX: Better message (if the message even matters).
            let _ = proxy
                .inhibit(None, INHIBIT_FLAGS, "File operations in progress")
                .await
                .inspect_err(|e| warn!("Failed to call inhibit endpoint: {e}"));
        }

        let _ = proxy
            .query_end_response(&session)
            .await
            .inspect_err(|e| warn!("Error sending QueryEnd to the XDG portal: {e}"));
    }

    Ok(())
}
