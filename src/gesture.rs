// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Turn a trackpad pinch into whole zoom steps.
//!
//! The platform layer reports a pinch as a stream of small magnification deltas between a
//! begin and an end. Zoom is quantised into steps, so the deltas are banked until they add
//! up to a step's worth of spread, and the remainder carries over to the next event.

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
}
