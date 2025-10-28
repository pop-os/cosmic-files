use std::num::NonZeroU16;

use crate::{config::IconSizes, tab::View};

static DEFAULT_ZOOM: NonZeroU16 = NonZeroU16::new(100).unwrap();
static MIN_ZOOM: NonZeroU16 = NonZeroU16::new(50).unwrap();
static MAX_ZOOM: NonZeroU16 = NonZeroU16::new(500).unwrap();
const ZOOM_STEP: u16 = 25;

pub(crate) const fn zoom_to_default(view: View, icon_sizes: &mut IconSizes) {
    let icon_size = select_resized_icon(view, icon_sizes);
    *icon_size = DEFAULT_ZOOM;
}

pub(crate) fn zoom_in_view(view: View, icon_sizes: &mut IconSizes) {
    let icon_size = select_resized_icon(view, icon_sizes);

    let mut step = MIN_ZOOM;
    while step <= MAX_ZOOM {
        if *icon_size < step {
            *icon_size = step;
            break;
        }
        step = step.saturating_add(ZOOM_STEP);
    }
    if *icon_size > step {
        *icon_size = step;
    }
}

pub(crate) fn zoom_out_view(view: View, icon_sizes: &mut IconSizes) {
    let icon_size = select_resized_icon(view, icon_sizes);

    let mut step = MAX_ZOOM;
    while step >= MIN_ZOOM {
        if *icon_size > step {
            *icon_size = step;
            break;
        }
        step = NonZeroU16::new(step.get().saturating_sub(ZOOM_STEP)).unwrap();
    }
    if *icon_size < step {
        *icon_size = step;
    }
}

const fn select_resized_icon(view: View, icon_sizes: &mut IconSizes) -> &mut NonZeroU16 {
    match view {
        View::Grid => &mut icon_sizes.grid,
        View::List => &mut icon_sizes.list,
    }
}
