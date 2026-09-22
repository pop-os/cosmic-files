// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Turn trackpad gestures into whole zoom steps and navigations.
//!
//! The platform layer reports a pinch as a stream of small magnification deltas between a
//! begin and an end. Zoom is quantised into steps, so the deltas are banked until they add
//! up to a step's worth of spread, and the remainder carries over to the next event.
//!
//! A two-finger swipe arrives the same way, as scroll deltas carrying the phase of the
//! fingers, and [`Swipe`] banks the horizontal ones until they amount to a navigation.
//!
//! Ctrl+scroll banks the vertical ones instead, and [`Zoom`] turns them into the same
//! steps a pinch makes.

/// Where an event sits in a gesture's lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Began,
    Changed,
    Ended,
}

/// Pinch spread that makes up one zoom step. `NSEvent.magnification` sums to about 1.0
/// for a spread that doubles the content, so this walks the 50%-500% zoom range in a
/// little over one and a half full spreads.
const MAGNIFICATION_PER_ZOOM_STEP: f64 = 0.1;

/// Banked pinch spread that has not yet amounted to a zoom step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pinch {
    accum: f64,
}

impl Pinch {
    /// Feed one event and return the zoom steps it completes: positive to zoom in,
    /// negative to zoom out.
    pub fn feed(&mut self, phase: Phase, magnification: f64) -> i32 {
        match phase {
            Phase::Began | Phase::Ended => {
                self.accum = 0.0;
                return 0;
            }
            Phase::Changed => {}
        }
        // Reversing direction should zoom back immediately, not spend the bank first.
        if magnification != 0.0 && self.accum != 0.0 && (self.accum > 0.0) != (magnification > 0.0)
        {
            self.accum = 0.0;
        }
        self.accum += magnification;
        let steps = (self.accum / MAGNIFICATION_PER_ZOOM_STEP) as i32;
        self.accum -= f64::from(steps) * MAGNIFICATION_PER_ZOOM_STEP;
        steps
    }
}

/// Which way a completed swipe navigates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Back,
    Forward,
}

/// One scroll event, as the platform layer reports it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Scroll {
    /// Where the fingers are in the gesture, or `None` for an event with no phase at
    /// all, such as a mouse wheel notch.
    pub phase: Option<Phase>,
    /// Whether the event is inertia thrown off after the fingers lifted.
    pub momentum: bool,
    /// Horizontal travel in logical points, positive when the fingers move right.
    pub delta_x: f64,
    /// Vertical travel in logical points.
    pub delta_y: f64,
    /// Whether the deltas are precise, as a trackpad's are and a mouse wheel's are not.
    pub precise: bool,
}

/// Horizontal travel that commits a two-finger swipe to a navigation, in logical
/// points. `NSEvent.scrollingDeltaX` is already logical, so unlike the physical pixels
/// winit reports this threshold means the same finger distance on every display. Fifty
/// points is an ordinary flick: near enough that the fingers cross it while still on the
/// glass, before the momentum this reducer ignores begins, and far enough that the
/// sideways drift left over from a vertical flick never adds up to it.
const SWIPE_POINTS: f64 = 50.0;

/// How straight a movement has to be to count as horizontal: the cosine of its angle to
/// the x axis, about 25 degrees. macOS locks no axis for you and trackpad deltas are
/// always a little diagonal, so without this a vertical flick drifts into a navigation.
/// The value is alacritty's, tested against real hardware.
const HORIZONTAL_COSINE: f64 = 0.9;

/// Banked horizontal travel of a two-finger swipe that has not yet navigated.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Swipe {
    travel: f64,
    fired: bool,
}

impl Swipe {
    /// Feed one scroll event and return the navigation it completes, at most once per
    /// physical gesture.
    pub fn feed(&mut self, scroll: Scroll) -> Option<Direction> {
        // Inertia is not a gesture. It arrives with no finger phase of its own and
        // carries most of a flick's travel, so it is dropped before anything is banked.
        if scroll.momentum {
            return None;
        }
        // A wheel reports coarse notches and no gesture at all; swiping is a trackpad's.
        if !scroll.precise {
            return None;
        }
        match scroll.phase {
            Some(Phase::Began) | Some(Phase::Ended) => {
                self.travel = 0.0;
                self.fired = false;
                return None;
            }
            Some(Phase::Changed) => {}
            // An event that belongs to no gesture has nothing to add to one.
            None => return None,
        }
        // One navigation per gesture: the fingers keep moving after it fires.
        if self.fired {
            return None;
        }
        // Only movement along the x axis is swiping; the rest is scrolling.
        let distance = scroll.delta_x.hypot(scroll.delta_y);
        if distance == 0.0 || scroll.delta_x.abs() / distance <= HORIZONTAL_COSINE {
            return None;
        }
        // Turning back means the swipe that was building never happened, so its travel
        // is dropped rather than spent slowing the new direction down.
        if self.travel != 0.0 && (self.travel > 0.0) != (scroll.delta_x > 0.0) {
            self.travel = 0.0;
        }
        self.travel += scroll.delta_x;
        if self.travel.abs() < SWIPE_POINTS {
            return None;
        }
        self.fired = true;
        // Fingers to the right reveal what came before, as in Safari and Finder.
        Some(if self.travel > 0.0 {
            Direction::Back
        } else {
            Direction::Forward
        })
    }
}

/// Vertical travel that makes up one zoom step while Ctrl is held, in logical points.
/// The same magnitude as the pixel threshold the other platforms bank against, so a
/// Ctrl+scroll covers the zoom range at the same rate it always did.
const POINTS_PER_ZOOM_STEP: f64 = 50.0;

/// Banked Ctrl+scroll travel that has not yet amounted to a zoom step.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Zoom {
    travel: f64,
}

impl Zoom {
    /// Feed one scroll event and return the zoom steps it completes: positive to zoom in,
    /// negative to zoom out.
    pub fn feed(&mut self, scroll: Scroll) -> i32 {
        // Inertia is not the user zooming. The platform says outright which events it
        // threw off, so they are dropped rather than guessed at from a lull in the
        // stream, and they bank nothing for the next gesture to inherit.
        if scroll.momentum {
            return 0;
        }
        match scroll.phase {
            Some(Phase::Began) | Some(Phase::Ended) => {
                self.travel = 0.0;
                return 0;
            }
            // Precise deltas can arrive belonging to no gesture, as a Magic Mouse's do;
            // there are still fingers behind them, so they bank like any other.
            Some(Phase::Changed) | None => {}
        }
        // A wheel reports coarse notches far smaller than a step's worth of points, but
        // each notch is a whole movement of the user's, so each one is a whole step.
        if !scroll.precise {
            self.travel = 0.0;
            return if scroll.delta_y > 0.0 {
                1
            } else if scroll.delta_y < 0.0 {
                -1
            } else {
                0
            };
        }
        // Reversing direction should zoom back immediately, not spend the bank first.
        if scroll.delta_y != 0.0
            && self.travel != 0.0
            && (self.travel > 0.0) != (scroll.delta_y > 0.0)
        {
            self.travel = 0.0;
        }
        self.travel += scroll.delta_y;
        let steps = (self.travel / POINTS_PER_ZOOM_STEP) as i32;
        self.travel -= f64::from(steps) * POINTS_PER_ZOOM_STEP;
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spread needed for one step, as the porting notes prescribe (0.05-0.1 per step).
    const STEP: f64 = 0.1;

    #[test]
    fn small_spreads_bank_until_they_make_a_step() {
        let mut pinch = Pinch::default();
        pinch.feed(Phase::Began, 0.0);
        assert_eq!(pinch.feed(Phase::Changed, 0.04), 0);
        assert_eq!(pinch.feed(Phase::Changed, 0.04), 0);
        assert_eq!(pinch.feed(Phase::Changed, 0.04), 1);
        // 0.12 spread minus one step leaves 0.02 banked, not enough for another.
        assert_eq!(pinch.feed(Phase::Changed, STEP - 0.03), 0);
    }

    #[test]
    fn a_fast_spread_completes_several_steps_at_once() {
        let mut pinch = Pinch::default();
        pinch.feed(Phase::Began, 0.0);
        assert_eq!(pinch.feed(Phase::Changed, 0.35), 3);
    }

    #[test]
    fn pinching_in_zooms_out() {
        let mut pinch = Pinch::default();
        pinch.feed(Phase::Began, 0.0);
        assert_eq!(pinch.feed(Phase::Changed, -0.06), 0);
        assert_eq!(pinch.feed(Phase::Changed, -0.06), -1);
    }

    #[test]
    fn a_new_gesture_starts_from_an_empty_bank() {
        let mut pinch = Pinch::default();
        pinch.feed(Phase::Began, 0.0);
        assert_eq!(pinch.feed(Phase::Changed, 0.09), 0);
        pinch.feed(Phase::Ended, 0.0);
        // Lifting the fingers throws the banked 0.09 away.
        pinch.feed(Phase::Began, 0.0);
        assert_eq!(pinch.feed(Phase::Changed, 0.04), 0);
        assert_eq!(pinch.feed(Phase::Changed, 0.04), 0);
        assert_eq!(pinch.feed(Phase::Changed, 0.04), 1);
    }

    #[test]
    fn ending_never_completes_a_step() {
        let mut pinch = Pinch::default();
        pinch.feed(Phase::Began, 0.0);
        assert_eq!(pinch.feed(Phase::Changed, 0.09), 0);
        // AppKit reports the end event with a magnification of its own.
        assert_eq!(pinch.feed(Phase::Ended, 0.05), 0);
    }

    #[test]
    fn reversing_direction_responds_immediately() {
        let mut pinch = Pinch::default();
        pinch.feed(Phase::Began, 0.0);
        // Bank most of a step outward, then pinch inward: the outward bank must not
        // absorb the first part of the inward motion.
        assert_eq!(pinch.feed(Phase::Changed, 0.09), 0);
        assert_eq!(pinch.feed(Phase::Changed, -0.06), 0);
        assert_eq!(pinch.feed(Phase::Changed, -0.06), -1);
    }

    /// A precise scroll event with the fingers still on the glass.
    fn finger(phase: Phase, delta_x: f64, delta_y: f64) -> Scroll {
        Scroll {
            phase: Some(phase),
            momentum: false,
            delta_x,
            delta_y,
            precise: true,
        }
    }

    #[test]
    fn a_horizontal_flick_navigates_once_per_gesture() {
        let mut swipe = Swipe::default();
        assert_eq!(swipe.feed(finger(Phase::Began, 0.0, 0.0)), None);
        assert_eq!(swipe.feed(finger(Phase::Changed, -30.0, 0.0)), None);
        assert_eq!(
            swipe.feed(finger(Phase::Changed, -30.0, 0.0)),
            Some(Direction::Forward)
        );
        // The rest of the same swipe must not navigate again.
        assert_eq!(swipe.feed(finger(Phase::Changed, -30.0, 0.0)), None);
        assert_eq!(swipe.feed(finger(Phase::Changed, -30.0, 0.0)), None);
        assert_eq!(swipe.feed(finger(Phase::Ended, 0.0, 0.0)), None);
    }

    #[test]
    fn momentum_after_the_fingers_lift_never_navigates() {
        let mut swipe = Swipe::default();
        swipe.feed(finger(Phase::Began, 0.0, 0.0));
        assert_eq!(swipe.feed(finger(Phase::Changed, -20.0, 0.0)), None);
        swipe.feed(finger(Phase::Ended, 0.0, 0.0));
        // The tail of a flick carries far more travel than the fingers did.
        for _ in 0..8 {
            let inertia = Scroll {
                phase: None,
                momentum: true,
                delta_x: -40.0,
                delta_y: 0.0,
                precise: true,
            };
            assert_eq!(swipe.feed(inertia), None);
        }
        // Momentum is ignored for being momentum, whatever finger phase it claims.
        let inertia = Scroll {
            phase: Some(Phase::Changed),
            momentum: true,
            delta_x: -60.0,
            delta_y: 0.0,
            precise: true,
        };
        assert_eq!(swipe.feed(inertia), None);
    }

    #[test]
    fn a_diagonal_drag_never_navigates() {
        let mut swipe = Swipe::default();
        swipe.feed(finger(Phase::Began, 0.0, 0.0));
        // Half a trackpad's worth of travel at 45 degrees is not a swipe.
        for _ in 0..10 {
            assert_eq!(swipe.feed(finger(Phase::Changed, 30.0, 30.0)), None);
        }
    }

    #[test]
    fn scrolling_a_list_never_navigates() {
        let mut swipe = Swipe::default();
        swipe.feed(finger(Phase::Began, 0.0, 0.0));
        // Fingers never run exactly straight down: the drift must not add up.
        for _ in 0..20 {
            assert_eq!(swipe.feed(finger(Phase::Changed, 4.0, 40.0)), None);
        }
    }

    #[test]
    fn reversing_before_the_threshold_cancels_the_banked_travel() {
        let mut swipe = Swipe::default();
        swipe.feed(finger(Phase::Began, 0.0, 0.0));
        // Drift right, then change your mind and swipe decisively left: the right-hand
        // travel must not pay for part of the left-hand swipe.
        assert_eq!(swipe.feed(finger(Phase::Changed, 40.0, 0.0)), None);
        assert_eq!(swipe.feed(finger(Phase::Changed, -30.0, 0.0)), None);
        assert_eq!(
            swipe.feed(finger(Phase::Changed, -30.0, 0.0)),
            Some(Direction::Forward)
        );
    }

    #[test]
    fn a_second_swipe_navigates_again() {
        let mut swipe = Swipe::default();
        swipe.feed(finger(Phase::Began, 0.0, 0.0));
        assert_eq!(
            swipe.feed(finger(Phase::Changed, 60.0, 0.0)),
            Some(Direction::Back)
        );
        swipe.feed(finger(Phase::Ended, 0.0, 0.0));
        swipe.feed(finger(Phase::Began, 0.0, 0.0));
        assert_eq!(
            swipe.feed(finger(Phase::Changed, 60.0, 0.0)),
            Some(Direction::Back)
        );
    }

    #[test]
    fn a_mouse_wheel_never_navigates() {
        let mut swipe = Swipe::default();
        // A wheel reports whole notches and no gesture phase at all.
        for _ in 0..20 {
            let notch = Scroll {
                phase: None,
                momentum: false,
                delta_x: 10.0,
                delta_y: 0.0,
                precise: false,
            };
            assert_eq!(swipe.feed(notch), None);
        }
        // Only a precise device swipes, whatever phase the coarse one reports.
        swipe.feed(finger(Phase::Began, 0.0, 0.0));
        let coarse = Scroll {
            phase: Some(Phase::Changed),
            momentum: false,
            delta_x: 60.0,
            delta_y: 0.0,
            precise: false,
        };
        assert_eq!(swipe.feed(coarse), None);
    }

    /// One coarse notch of a mouse wheel, which belongs to no gesture.
    fn notch(delta_y: f64) -> Scroll {
        Scroll {
            phase: None,
            momentum: false,
            delta_x: 0.0,
            delta_y,
            precise: false,
        }
    }

    #[test]
    fn momentum_after_the_fingers_lift_never_zooms() {
        let mut zoom = Zoom::default();
        zoom.feed(finger(Phase::Began, 0.0, 0.0));
        // The tail of a flick carries several steps' worth of travel on its own.
        let inertia = Scroll {
            phase: None,
            momentum: true,
            delta_x: 0.0,
            delta_y: 120.0,
            precise: true,
        };
        assert_eq!(zoom.feed(inertia), 0);
        // It banked nothing either: the fingers still owe the whole step.
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 40.0)), 0);
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 10.0)), 1);
    }

    #[test]
    fn small_scroll_deltas_bank_until_they_make_a_step() {
        let mut zoom = Zoom::default();
        zoom.feed(finger(Phase::Began, 0.0, 0.0));
        // Forty-eight points of travel is a step short.
        for _ in 0..4 {
            assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 12.0)), 0);
        }
        // Sixty points is one step, and leaves ten banked rather than spending them.
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 12.0)), 1);
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 39.0)), 0);
    }

    #[test]
    fn a_fast_scroll_completes_several_steps_at_once() {
        let mut zoom = Zoom::default();
        zoom.feed(finger(Phase::Began, 0.0, 0.0));
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 160.0)), 3);
    }

    #[test]
    fn a_wheel_notch_is_one_zoom_step() {
        let mut zoom = Zoom::default();
        // A wheel's deltas are coarse and nowhere near a step's worth of points, but a
        // notch is the whole gesture the user made.
        assert_eq!(zoom.feed(notch(3.0)), 1);
        assert_eq!(zoom.feed(notch(-3.0)), -1);
    }

    #[test]
    fn a_new_scroll_gesture_starts_from_an_empty_bank() {
        let mut zoom = Zoom::default();
        zoom.feed(finger(Phase::Began, 0.0, 0.0));
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 40.0)), 0);
        zoom.feed(finger(Phase::Ended, 0.0, 0.0));
        // Lifting the fingers throws the banked forty points away.
        zoom.feed(finger(Phase::Began, 0.0, 0.0));
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 40.0)), 0);
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 10.0)), 1);
    }

    #[test]
    fn reversing_scroll_direction_responds_immediately() {
        let mut zoom = Zoom::default();
        zoom.feed(finger(Phase::Began, 0.0, 0.0));
        // Bank most of a step of zooming in, then scroll the other way: the bank must not
        // absorb the first part of the motion back out.
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, 40.0)), 0);
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, -40.0)), 0);
        assert_eq!(zoom.feed(finger(Phase::Changed, 0.0, -10.0)), -1);
    }
}
