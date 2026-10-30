use cosmic::iced::Rectangle;
use cosmic::widget;
use std::cell::Cell;

use crate::item::ChecksumState;

#[derive(Clone, Debug)]
pub struct ItemState {
    pub button_id: widget::Id,
    pub checksums: ChecksumState,
    pub cut: bool,
    pub highlighted: bool,
    pub overlaps_drag_rect: bool,
    pub pos_opt: Cell<Option<(usize, usize)>>,
    pub rect_opt: Cell<Option<Rectangle>>,
    pub selected: bool,
}

impl Default for ItemState {
    fn default() -> Self {
        Self {
            button_id: widget::Id::unique(),
            checksums: ChecksumState::default(),
            cut: false,
            highlighted: false,
            overlaps_drag_rect: false,
            pos_opt: Cell::new(None),
            rect_opt: Cell::new(None),
            selected: false,
        }
    }
}
