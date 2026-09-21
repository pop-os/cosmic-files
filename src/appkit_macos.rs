// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Startup fixes that only AppKit can make.
//!
//! Two of them, both applied once at launch:
//!
//! * [`disable_autofill_heuristics`] registers a user default before the event loop starts, so
//!   AppKit never attaches its autofill heuristics to the rename, search and path-bar text
//!   fields. Left on, they make typing slower the longer the app runs on macOS 26.
//! * [`pin_srgb_color_space`] pins the window to sRGB. MacBook panels are Display P3 and
//!   libcosmic's palettes are authored in sRGB, so greys and the accent colour come out
//!   oversaturated until the window is told which space its colours are in.
//!
//! Reaching the `NSWindow` means going out through `raw_window_handle` to the `NSView` iced
//! draws into, which is what [`with_ns_window`] wraps: it hands a live `NSWindow` to a closure
//! on the main thread, or yields nothing if there is no window to hand over. AppKit calls back
//! into us synchronously, so nothing in here may touch application state.

use std::sync::atomic::{AtomicBool, Ordering};

use cosmic::iced::Task;
use cosmic::iced::runtime::window::raw_window_handle::RawWindowHandle;
use cosmic::iced::runtime::window::run_with_handle;
use cosmic::iced::window::Id as WindowId;
use objc2::MainThreadMarker;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSColorSpace, NSView, NSWindow};
use objc2_foundation::{NSDictionary, NSUserDefaults, ns_string};

/// Turn off AppKit's autofill heuristics. Call once, before the event loop starts; a default
/// registered afterwards would not be read by the text fields that are already alive.
pub fn disable_autofill_heuristics() {
    let key = ns_string!("NSAutoFillHeuristicControllerEnabled");
    let value: &AnyObject = ns_string!("NO");
    let defaults = NSUserDefaults::standardUserDefaults();

    // SAFETY: the dictionary is the `NSString` → `NSString` registration dictionary the method
    // documents; both are static strings that outlive the call.
    unsafe { defaults.registerDefaults(&NSDictionary::from_slices(&[key], &[value])) };

    log::info!(
        "registered NSAutoFillHeuristicControllerEnabled = NO (now {})",
        defaults.boolForKey(key)
    );
}

/// Set once the window's colour space has been pinned, so the repeat calls that come with
/// every resize cost nothing.
static PINNED_SRGB: AtomicBool = AtomicBool::new(false);

/// Pin `window_id`'s colour space to sRGB. Safe to call whenever the window reports a size:
/// the work happens on the first call that finds a window, and every later call is a no-op.
///
/// A task queued before the window exists is dropped by iced, which is why this is driven from
/// a window event rather than from `init`.
pub fn pin_srgb_color_space<M: Send + 'static>(window_id: WindowId) -> Task<M> {
    if PINNED_SRGB.load(Ordering::Relaxed) {
        return Task::none();
    }

    with_ns_window(window_id, |window, _mtm| {
        let was = color_space_name(window);
        window.setColorSpace(Some(&NSColorSpace::sRGBColorSpace()));
        PINNED_SRGB.store(true, Ordering::Relaxed);
        log::info!(
            "window colour space {was} is now {}",
            color_space_name(window)
        );
    })
    .discard()
}

fn color_space_name(window: &NSWindow) -> String {
    window
        .colorSpace()
        .and_then(|color_space| color_space.localizedName())
        .map_or_else(|| "unnamed".to_string(), |name| name.to_string())
}

/// Run `f` against the `NSWindow` behind an iced window, on the main thread.
///
/// Yields nothing when the window has already closed, when AppKit has not put the view in a
/// window yet, or when the callback somehow runs off the main thread. Do not touch application
/// state from `f`: iced holds the app borrowed while the task runs.
pub fn with_ns_window<T: Send + 'static>(
    window_id: WindowId,
    f: impl FnOnce(&NSWindow, MainThreadMarker) -> T + Send + 'static,
) -> Task<Option<T>> {
    run_with_handle(window_id, move |handle| {
        let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
            log::warn!("no AppKit window handle for window {window_id:?}");
            return None;
        };
        let Some(mtm) = MainThreadMarker::new() else {
            log::warn!("skipping AppKit window call for {window_id:?}: not on the main thread");
            return None;
        };

        // SAFETY: the handle borrows the view for the duration of this call, and the main
        // thread marker proves the thread AppKit requires for an `NSView`.
        let view: &NSView = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
        let Some(window) = view.window() else {
            log::warn!("AppKit view for window {window_id:?} has no window");
            return None;
        };
        Some(f(&window, mtm))
    })
}
