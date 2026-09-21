// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! The parts of the application only AppKit can provide.
//!
//! Two fixes are applied once at launch:
//!
//! * [`disable_autofill_heuristics`] registers a user default before the event loop starts, so
//!   AppKit never attaches its autofill heuristics to the rename, search and path-bar text
//!   fields. Left on, they make typing slower the longer the app runs on macOS 26.
//! * [`pin_srgb_color_space`] pins the window to sRGB. MacBook panels are Display P3 and
//!   libcosmic's palettes are authored in sRGB, so greys and the accent colour come out
//!   oversaturated until the window is told which space its colours are in.
//!
//! The rest is the application lifecycle a Mac app is expected to have.
//! [`hide_application`] is what Cmd+H does everywhere else, and [`watch_activation`] notices
//! the application being brought to the front — a click on the Dock icon, most of all — so that
//! a window can be put back after the last one was closed.
//!
//! Reaching the `NSWindow` means going out through `raw_window_handle` to the `NSView` iced
//! draws into, which is what [`with_ns_window`] wraps: it hands a live `NSWindow` to a closure
//! on the main thread, or yields nothing if there is no window to hand over. AppKit calls back
//! into us synchronously, so nothing in here may touch application state.

use std::ptr::NonNull;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use block2::RcBlock;
use cosmic::iced::futures::{self, StreamExt, channel::mpsc};
use cosmic::iced::runtime::window::raw_window_handle::RawWindowHandle;
use cosmic::iced::runtime::window::run_with_handle;
use cosmic::iced::window::Id as WindowId;
use cosmic::iced::{Subscription, Task};
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationDidBecomeActiveNotification, NSColorSpace, NSMenu, NSView, NSWindow,
};
use objc2_foundation::{
    NSDictionary, NSNotification, NSNotificationCenter, NSUserDefaults, ns_string,
};

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

/// Hide the application, the way Cmd+H hides any other Mac app. Clicking the Dock icon, or
/// Cmd+Tabbing back, brings it out again; AppKit restores the windows it hid.
pub fn hide_application() {
    let Some(mtm) = MainThreadMarker::new() else {
        log::warn!("not hiding the application: not on the main thread");
        return;
    };
    // `nil` is the sender a programmatic hide passes, as opposed to the menu item that would
    // otherwise be validated against it.
    NSApplication::sharedApplication(mtm).hide(None);
    log::info!("hid the application");
}

/// The application was brought to the front: the Dock icon was clicked, it was Cmd+Tabbed to,
/// or it was unhidden.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Activated;

/// The receiving end of the activation observer's channel, parked here until the subscription
/// starts.
static ACTIVATIONS: Mutex<Option<mpsc::UnboundedReceiver<Activated>>> = Mutex::new(None);

/// Start watching for the application being activated. Call once, on the main thread, before
/// the first [`activation_subscription`] runs; the observer lives for the rest of the process.
///
/// This observes a notification rather than implementing `applicationShouldHandleReopen:`,
/// because winit owns the application delegate and replacing it would take the rest of the
/// delegate's work with it. The cost is that an activation is only reported when the
/// application was not already frontmost.
pub fn watch_activation() {
    let Some(_mtm) = MainThreadMarker::new() else {
        log::warn!("activation observer not installed: not on the main thread");
        return;
    };

    let (tx, rx) = mpsc::unbounded();
    *ACTIVATIONS.lock().unwrap() = Some(rx);

    let handler = RcBlock::new(move |_notification: NonNull<NSNotification>| {
        // AppKit posts this synchronously, while the application may already be borrowed, so
        // handing the news to the subscription is all this may do.
        let _ = tx.unbounded_send(Activated);
    });

    let center = NSNotificationCenter::defaultCenter();
    // SAFETY: the name is AppKit's own notification constant, the block only sends on a
    // channel, and a `None` queue asks for delivery on the posting thread, which is the main
    // thread for this notification.
    let token = unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(NSApplicationDidBecomeActiveNotification),
            None,
            None,
            &handler,
        )
    };
    // The observer is wanted for the life of the process, and dropping the token would remove
    // it, so the token is leaked rather than tracked.
    std::mem::forget(token);
    log::info!("watching for application activation");
}

/// Deliver the observer's activations as messages. Yields nothing if [`watch_activation`] did
/// not run.
pub fn activation_subscription() -> Subscription<Activated> {
    Subscription::run(|| {
        // The receiver is taken once; a restarted subscription gets an empty stream.
        let rx = ACTIVATIONS.lock().unwrap().take();
        futures::stream::iter(rx).flatten()
    })
}

/// Set once the Quit item has been unbound, so the repeat calls cost nothing.
static QUIT_KEY_RELEASED: AtomicBool = AtomicBool::new(false);

/// Take Cmd+Q off the Quit item in the application menu that winit installs.
///
/// That item sends `terminate:`, which ends the process where it stands; a copy or move still
/// running would be lost with it. AppKit matches a menu item's key equivalent before the key
/// reaches the window, so while the shortcut sits on that item the binding table never sees
/// Cmd+Q. Unbound, the key arrives as any other does and quits through
/// [`crate::app::Action::Quit`], which waits for the pending operations first.
///
/// Call once a window exists: winit builds the menu while the event loop is starting, which is
/// after anything `main` can do.
pub fn release_quit_key_equivalent() {
    if QUIT_KEY_RELEASED.load(Ordering::Relaxed) {
        return;
    }
    let Some(mtm) = MainThreadMarker::new() else {
        log::warn!("not unbinding Cmd+Q: not on the main thread");
        return;
    };
    let Some(menu) = NSApplication::sharedApplication(mtm).mainMenu() else {
        log::info!("no application menu, so nothing holds Cmd+Q");
        QUIT_KEY_RELEASED.store(true, Ordering::Relaxed);
        return;
    };
    let unbound = unbind_terminate(&menu);
    log::info!("unbound Cmd+Q from {unbound} Quit menu item(s)");
    QUIT_KEY_RELEASED.store(true, Ordering::Relaxed);
}

/// Clear the key equivalent of every `terminate:` item in `menu` and its submenus, and report
/// how many there were.
fn unbind_terminate(menu: &NSMenu) -> usize {
    let mut unbound = 0;
    for item in &menu.itemArray() {
        if let Some(submenu) = item.submenu() {
            unbound += unbind_terminate(&submenu);
        }
        log::debug!(
            "menu item {:?} action {:?} key {:?}",
            item.title(),
            item.action(),
            item.keyEquivalent()
        );
        if item.action() == Some(sel!(terminate:)) && !item.keyEquivalent().is_empty() {
            item.setKeyEquivalent(ns_string!(""));
            unbound += 1;
        }
    }
    unbound
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
