// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Trackpad gestures from AppKit.
//!
//! winit turns `magnifyWithEvent:` into a `PinchGesture` window event, but the iced fork that
//! libcosmic builds on never converts it, so the gesture would otherwise never reach the app.
//! Swipes and `smartMagnifyWithEvent:` fare worse still: iced drops the scroll phase winit does
//! deliver, and winit reports no smart magnify at all. Instead an `NSEvent` local monitor
//! watches the application's own event stream, which AppKit hands to the monitor before
//! dispatching to the view. Each interesting event is forwarded over a channel as a
//! [`GestureEvent`], and [`subscription`] turns that channel into messages.
//!
//! What AppKit reports is passed on as it comes, phases and all; deciding when a stream of
//! deltas amounts to a zoom step or a navigation belongs to the reducers in [`crate::gesture`].
//!
//! The monitor passes every event through untouched, so nothing winit does changes.

use std::ptr::NonNull;
use std::sync::Mutex;

use block2::RcBlock;
use cosmic::iced::Subscription;
use cosmic::iced::futures::{self, StreamExt, channel::mpsc};
use objc2::MainThreadMarker;
use objc2_app_kit::{NSEvent, NSEventMask, NSEventPhase, NSEventType};

use crate::gesture::{Phase, Scroll};

/// A gesture event worth telling the app about.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GestureEvent {
    /// A pinch: where it sits in the gesture, and how far the fingers spread.
    Pinch(Phase, f64),
    /// A scroll, which is also how a two-finger swipe arrives.
    Scroll(Scroll),
    /// A two-finger double tap.
    SmartMagnify,
}

/// The receiving end of the monitor's channel, parked here until the subscription starts.
static RECEIVER: Mutex<Option<mpsc::UnboundedReceiver<GestureEvent>>> = Mutex::new(None);

/// Start watching for trackpad gestures. Call once, on the main thread, before the first
/// [`subscription`] runs; the monitor lives for the rest of the process.
pub fn install() {
    let Some(_mtm) = MainThreadMarker::new() else {
        log::warn!("gesture monitor not installed: not on the main thread");
        return;
    };

    let (tx, rx) = mpsc::unbounded();
    *RECEIVER.lock().unwrap() = Some(rx);

    let handler = RcBlock::new(move |event: NonNull<NSEvent>| -> *mut NSEvent {
        // SAFETY: AppKit hands the monitor a live event for the duration of the call.
        let event_ref = unsafe { event.as_ref() };
        if let Some(gesture) = gesture_event(event_ref) {
            // The subscription may not have started yet, or may have been dropped at
            // shutdown; either way there is nobody to tell, and that is fine.
            let _ = tx.unbounded_send(gesture);
        }
        event.as_ptr()
    });

    // Scrolling is watched for the horizontal swipe that navigates history. The events are
    // only observed: they go on to the view as they always did, so scrolling is unaffected.
    let mask = NSEventMask::Magnify | NSEventMask::ScrollWheel | NSEventMask::SmartMagnify;
    // SAFETY: the block returns the same event pointer it was given, which is valid.
    let monitor = unsafe { NSEvent::addLocalMonitorForEventsMatchingMask_handler(mask, &handler) };
    match monitor {
        // The monitor is removed only through `removeMonitor:`, never by dropping the token,
        // so the token is left to live for the process rather than tracked.
        Some(token) => std::mem::forget(token),
        None => log::warn!("gesture monitor not installed: AppKit refused it"),
    }
}

/// Deliver the monitor's gesture events as messages. Yields nothing if [`install`] did not run.
pub fn subscription() -> Subscription<GestureEvent> {
    Subscription::run(|| {
        // The receiver is taken once; a restarted subscription gets an empty stream.
        let rx = RECEIVER.lock().unwrap().take();
        futures::stream::iter(rx).flatten()
    })
}

fn gesture_event(event: &NSEvent) -> Option<GestureEvent> {
    match event.r#type() {
        NSEventType::Magnify => Some(GestureEvent::Pinch(
            phase(event.phase())?,
            event.magnification(),
        )),
        NSEventType::ScrollWheel => Some(GestureEvent::Scroll(Scroll {
            phase: phase(event.phase()),
            // A momentum phase of its own means the fingers have already lifted.
            momentum: !event.momentumPhase().is_empty(),
            // These are logical points, unlike the physical pixels winit reports.
            delta_x: event.scrollingDeltaX(),
            delta_y: event.scrollingDeltaY(),
            precise: event.hasPreciseScrollingDeltas(),
        })),
        NSEventType::SmartMagnify => Some(GestureEvent::SmartMagnify),
        _ => None,
    }
}

/// Where an event sits in its gesture's lifetime, or `None` if it belongs to no gesture: a
/// mouse wheel notch, or the MayBegin and Stationary phases, which report no movement.
fn phase(phase: NSEventPhase) -> Option<Phase> {
    if phase.contains(NSEventPhase::Began) {
        Some(Phase::Began)
    } else if phase.contains(NSEventPhase::Ended) || phase.contains(NSEventPhase::Cancelled) {
        Some(Phase::Ended)
    } else if phase.contains(NSEventPhase::Changed) {
        Some(Phase::Changed)
    } else {
        None
    }
}
