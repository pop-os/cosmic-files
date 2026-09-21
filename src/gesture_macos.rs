// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Trackpad gestures from AppKit.
//!
//! winit turns `magnifyWithEvent:` into a `PinchGesture` window event, but the iced fork that
//! libcosmic builds on never converts it, so the gesture would otherwise never reach the app.
//! Instead an `NSEvent` local monitor watches the application's own event stream for magnify
//! events, which AppKit hands to the monitor before dispatching them to the view. Each one is
//! forwarded over a channel as a [`Phase`] and magnification delta, and [`subscription`] turns
//! that channel into messages.
//!
//! The monitor passes every event through untouched, so nothing winit does changes.

use std::ptr::NonNull;
use std::sync::Mutex;

use block2::RcBlock;
use cosmic::iced::Subscription;
use cosmic::iced::futures::{self, StreamExt, channel::mpsc};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSEventMask, NSEventPhase};

use crate::gesture::Phase;

/// One magnify event: where it sits in the gesture and how much the fingers spread.
pub type PinchEvent = (Phase, f64);

/// The receiving end of the monitor's channel, parked here until the subscription starts.
static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<PinchEvent>>> = Mutex::new(None);

/// Start watching for pinch gestures. Call once, on the main thread, before the first
/// [`subscription`] runs; the monitor lives for the rest of the process.
pub fn install() {
    let Some(_mtm) = MainThreadMarker::new() else {
        log::warn!("pinch monitor not installed: not on the main thread");
        return;
    };

    let (tx, rx) = mpsc::unbounded();
    *RECEIVER.lock().unwrap() = Some(rx);

    let handler = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        // SAFETY: AppKit hands the monitor a live event for the duration of the call.
        let event_ref = unsafe { event.as_ref() };
        if let Some(pinch) = pinch_event(event_ref) {
            // The subscription may not have started yet, or may have been dropped at
            // shutdown; either way there is nobody to tell, and that is fine.
            let _ = tx.unbounded_send(pinch);
        }
        event.as_ptr()
    });

    // SAFETY: the block returns the same event pointer it was given, which is valid.
    let monitor = unsafe {
        NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::Magnify, &handler)
    };
    match monitor {
        // The monitor is removed only through `removeMonitor:`, never by dropping the token,
        // so the token is left to live for the process rather than tracked.
        Some(token) => std::mem::forget(token),
        None => log::warn!("pinch monitor not installed: AppKit refused it"),
    }
}

/// Deliver the monitor's pinch events as messages. Yields nothing if [`install`] did not run.
pub fn subscription() -> Subscription<PinchEvent> {
    Subscription::run(|| {
        // The receiver is taken once; a restarted subscription gets an empty stream.
        let rx = RECEIVER.lock().unwrap().take();
        futures::stream::iter(rx).flatten()
    })
}

fn pinch_event(event: &NSEvent) -> Option<PinchEvent> {
    let phase = event.phase();
    let phase = if phase.contains(NSEventPhase::Began) {
        Phase::Began
    } else if phase.contains(NSEventPhase::Ended) || phase.contains(NSEventPhase::Cancelled) {
        Phase::Ended
    } else if phase.contains(NSEventPhase::Changed) {
        Phase::Changed
    } else {
        // MayBegin and Stationary carry no spread to act on.
        return None;
    };
    Some((phase, event.magnification()))
}
