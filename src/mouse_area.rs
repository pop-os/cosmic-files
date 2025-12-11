//! A container for capturing mouse events.

use std::time::Instant;

use crate::tab::DOUBLE_CLICK_DURATION;
use cosmic::{
    Element, Renderer, Theme,
    iced_core::{
        Clipboard, Color, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
        border::Border,
        event::{self, Event},
        layout,
        mouse::{self, click},
        overlay,
        renderer::{self, Quad, Renderer as _},
        touch,
        widget::{Operation, Tree, tree},
    },
    widget::Id,
};

/// Emit messages on mouse events.
#[allow(missing_debug_implementations)]
pub struct MouseArea<'a, Message> {
    id: Id,
    content: Element<'a, Message>,
    on_auto_scroll: Option<Box<dyn OnAutoScroll<'a, Message>>>,
    on_drag: Option<Box<dyn OnDrag<'a, Message>>>,
    on_double_click: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_press: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_drag_end: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_release: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_resize: Option<Box<dyn OnResize<'a, Message>>>,
    on_right_press: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_right_press_no_capture: bool,
    on_right_press_window_position: bool,
    on_right_release: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_middle_press: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_middle_release: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_back_press: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_back_release: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_forward_press: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_forward_release: Option<Box<dyn OnMouseButton<'a, Message>>>,
    on_scroll: Option<Box<dyn OnScroll<'a, Message>>>,
    on_enter: Option<Box<dyn OnEnterExit<'a, Message>>>,
    on_exit: Option<Box<dyn OnEnterExit<'a, Message>>>,
    show_drag_rect: bool,
}

impl<'a, Message> MouseArea<'a, Message> {
    /// The message to emit when auto scroll changes.
    #[must_use]
    pub fn on_auto_scroll(mut self, message: impl OnAutoScroll<'a, Message>) -> Self {
        self.on_auto_scroll = Some(Box::new(message));
        self
    }

    /// The message to emit when a drag is initiated.
    #[must_use]
    pub fn on_drag(mut self, message: impl OnDrag<'a, Message>) -> Self {
        self.on_drag = Some(Box::new(message));
        self
    }

    /// The message to emit when a drag ends.
    #[must_use]
    pub fn on_drag_end(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_drag_end = Some(Box::new(message));
        self
    }

    /// The message to emit on a double click.
    #[must_use]
    pub fn on_double_click(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_double_click = Some(Box::new(message));
        self
    }

    /// The message to emit on a left button press.
    #[must_use]
    pub fn on_press(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a left button release.
    #[must_use]
    pub fn on_release(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_release = Some(Box::new(message));
        self
    }

    /// The message to emit on resizing.
    #[must_use]
    pub fn on_resize(mut self, message: impl OnResize<'a, Message>) -> Self {
        self.on_resize = Some(Box::new(message));
        self
    }

    /// The message to emit on a right button press.
    #[must_use]
    pub fn on_right_press(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_right_press = Some(Box::new(message));
        self
    }

    /// on_right_press will not capture input
    #[must_use]
    pub fn on_right_press_no_capture(mut self) -> Self {
        self.on_right_press_no_capture = true;
        self
    }

    /// Only on wayland, on_right_press will provide window position instead of widget relative
    #[must_use]
    pub fn wayland_on_right_press_window_position(mut self) -> Self {
        #[cfg(feature = "wayland")]
        {
            self.on_right_press_window_position = true;
        }
        self
    }

    /// on_right_press will provide window position instead of widget relative
    #[must_use]
    pub fn on_right_press_window_position(mut self) -> Self {
        self.on_right_press_window_position = true;
        self
    }

    /// The message to emit on a right button release.
    #[must_use]
    pub fn on_right_release(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_right_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a middle button press.
    #[must_use]
    pub fn on_middle_press(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_middle_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a middle button release.
    #[must_use]
    pub fn on_middle_release(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_middle_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a back button press.
    #[must_use]
    pub fn on_back_press(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_back_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a back button release.
    #[must_use]
    pub fn on_back_release(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_back_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a forward button press.
    #[must_use]
    pub fn on_forward_press(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_forward_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a forward button release.
    #[must_use]
    pub fn on_forward_release(mut self, message: impl OnMouseButton<'a, Message>) -> Self {
        self.on_forward_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a scroll.
    #[must_use]
    pub fn on_scroll(mut self, message: impl OnScroll<'a, Message>) -> Self {
        self.on_scroll = Some(Box::new(message));
        self
    }

    /// The message to emit when a mouse enters the area.
    #[must_use]
    pub fn on_enter(mut self, message: impl OnEnterExit<'a, Message>) -> Self {
        self.on_enter = Some(Box::new(message));
        self
    }

    /// The message to emit when a mouse exits the area.
    #[must_use]
    pub fn on_exit(mut self, message: impl OnEnterExit<'a, Message>) -> Self {
        self.on_exit = Some(Box::new(message));
        self
    }

    #[must_use]
    pub const fn show_drag_rect(mut self, show_drag_rect: bool) -> Self {
        self.show_drag_rect = show_drag_rect;
        self
    }

    /// Sets the widget's unique identifier.
    #[must_use]
    pub fn with_id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }
}

pub trait OnAutoScroll<'a, Message>: Fn(Option<f32>) -> Message + 'a {}
impl<'a, Message, F> OnAutoScroll<'a, Message> for F where F: Fn(Option<f32>) -> Message + 'a {}

pub trait OnMouseButton<'a, Message>: Fn(Option<Point>) -> Message + 'a {}
impl<'a, Message, F> OnMouseButton<'a, Message> for F where F: Fn(Option<Point>) -> Message + 'a {}

pub trait OnDrag<'a, Message>: Fn(Option<Rectangle>) -> Message + 'a {}
impl<'a, Message, F> OnDrag<'a, Message> for F where F: Fn(Option<Rectangle>) -> Message + 'a {}

pub trait OnResize<'a, Message>: Fn(Rectangle) -> Message + 'a {}
impl<'a, Message, F> OnResize<'a, Message> for F where F: Fn(Rectangle) -> Message + 'a {}

pub trait OnScroll<'a, Message>: Fn(mouse::ScrollDelta) -> Option<Message> + 'a {}
impl<'a, Message, F> OnScroll<'a, Message> for F where
    F: Fn(mouse::ScrollDelta) -> Option<Message> + 'a
{
}

pub trait OnEnterExit<'a, Message>: Fn() -> Message + 'a {}
impl<'a, Message, F> OnEnterExit<'a, Message> for F where F: Fn() -> Message + 'a {}

/// Local state of the [`MouseArea`].
#[derive(Default)]
struct State {
    last_auto_scroll: Option<f32>,
    last_position: Option<Point>,
    last_virtual_position: Option<Point>,
    drag_initiated: Option<Point>,
    prev_click: Option<(mouse::Click, Instant)>,
    viewport: Option<Rectangle>,
}

impl State {
    fn drag_rect(&self, cursor: mouse::Cursor) -> Option<Rectangle> {
        if let Some(drag_source) = self.drag_initiated {
            if let Some(position) = cursor.position().or(self.last_virtual_position) {
                if position.distance(drag_source) > 1.0 {
                    let min_x = drag_source.x.min(position.x);
                    let max_x = drag_source.x.max(position.x);
                    let min_y = drag_source.y.min(position.y);
                    let max_y = drag_source.y.max(position.y);
                    return Some(Rectangle::new(
                        Point::new(min_x, min_y),
                        Size::new(max_x - min_x, max_y - min_y),
                    ));
                }
            }
        }
        None
    }

    fn click(&mut self, pos: Point) -> mouse::Click {
        let now = Instant::now();

        let new = if let Some((prev_click, prev_time)) = self.prev_click.take() {
            if now.duration_since(prev_time) < DOUBLE_CLICK_DURATION {
                match prev_click.kind() {
                    mouse::click::Kind::Single => {
                        mouse::Click::new(pos, mouse::Button::Left, Some(prev_click))
                    }
                    mouse::click::Kind::Double => {
                        mouse::Click::new(pos, mouse::Button::Left, Some(prev_click))
                    }
                    mouse::click::Kind::Triple => {
                        mouse::Click::new(pos, mouse::Button::Left, Some(prev_click))
                    }
                }
            } else {
                mouse::Click::new(pos, mouse::Button::Left, None)
            }
        } else {
            mouse::Click::new(pos, mouse::Button::Left, None)
        };
        self.prev_click = Some((new, now));
        new
    }
}

impl<'a, Message> MouseArea<'a, Message> {
    /// Creates a [`MouseArea`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        MouseArea {
            id: Id::unique(),
            content: content.into(),
            on_auto_scroll: None,
            on_drag: None,
            on_drag_end: None,
            on_double_click: None,
            on_press: None,
            on_release: None,
            on_resize: None,
            on_right_press: None,
            on_right_press_no_capture: false,
            on_right_press_window_position: false,
            on_right_release: None,
            on_middle_press: None,
            on_middle_release: None,
            on_back_press: None,
            on_back_release: None,
            on_forward_press: None,
            on_forward_release: None,
            on_enter: None,
            on_exit: None,
            on_scroll: None,
            show_drag_rect: false,
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for MouseArea<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) == event::Status::Captured
        {
            return event::Status::Captured;
        }

        update(
            self,
            &event,
            layout,
            cursor,
            shell,
            tree.state.downcast_mut::<State>(),
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );

        if self.show_drag_rect {
            let state = tree.state.downcast_ref::<State>();
            if let Some(bounds) = state.drag_rect(cursor) {
                let cosmic = theme.cosmic();
                let mut bg_color = cosmic.accent_color();
                //TODO: get correct alpha
                bg_color.alpha = 0.2;
                renderer.start_layer(*viewport);
                renderer.fill_quad(
                    Quad {
                        bounds,
                        border: Border {
                            color: cosmic.accent_color().into(),
                            width: 1.0,
                            radius: cosmic.radius_xs().into(),
                        },
                        ..Default::default()
                    },
                    Color::from(bg_color),
                );
                renderer.end_layer();
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer, translation)
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut cosmic::iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.content.as_widget().drag_destinations(
            &state.children[0],
            layout,
            renderer,
            dnd_rectangles,
        );
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<'a, Message> From<MouseArea<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
    Renderer: 'a + renderer::Renderer,
    Theme: 'a,
{
    fn from(area: MouseArea<'a, Message>) -> Self {
        Element::new(area)
    }
}

/// Processes the given [`Event`] and updates the [`State`] of an [`MouseArea`]
/// accordingly.
fn update<Message: Clone>(
    widget: &mut MouseArea<'_, Message>,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    state: &mut State,
    viewport: &Rectangle,
) -> event::Status {
    let offset = layout.virtual_offset();
    let layout_bounds = layout.bounds();

    let viewport_changed = state.viewport.map_or(true, |v| v != *viewport);

    if let Some(message) = widget.on_resize.as_ref() {
        if viewport_changed {
            shell.publish(message(*viewport));
        }
    }

    state.viewport = Some(*viewport);

    let should_check_hover = viewport_changed
        || matches!(
            event,
            Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Mouse(mouse::Event::WheelScrolled { .. })
        );

    if should_check_hover {
        let position_in = cursor.position_in(layout_bounds);
        match (position_in, state.last_position) {
            (None, Some(_)) => {
                if let Some(message) = widget.on_exit.as_ref() {
                    shell.publish(message());
                }
            }
            (Some(_), None) => {
                if let Some(message) = widget.on_enter.as_ref() {
                    shell.publish(message());
                }
            }
            _ => {}
        }
        state.last_position = position_in;
    }

    if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
        let virtual_position = Point::new(
            viewport.x - layout_bounds.x + position.x,
            viewport.y - layout_bounds.y + position.y,
        );
        state.last_virtual_position = Some(virtual_position);

        if let Some(message) = widget.on_auto_scroll.as_ref() {
            let auto_scroll = if state.drag_initiated.is_some() {
                let bottom = viewport.y;
                let top = viewport.y + viewport.height;
                if virtual_position.y < bottom {
                    Some(virtual_position.y - bottom)
                } else if virtual_position.y > top {
                    Some(virtual_position.y - top)
                } else {
                    None
                }
            } else {
                None
            };
            if state.last_auto_scroll != auto_scroll {
                shell.publish(message(auto_scroll));
                state.last_auto_scroll = auto_scroll;
            }
        }
    }

    if state.drag_initiated.is_none() && !cursor.is_over(layout_bounds) {
        return event::Status::Ignored;
    }

    if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
    | Event::Touch(touch::Event::FingerPressed { .. }) = event
    {
        let click = state.click(cursor.position_in(layout_bounds).unwrap_or_default());
        match click.kind() {
            click::Kind::Single => {
                if let Some(message) = widget.on_press.as_ref() {
                    shell.publish(message(cursor.position_in(layout_bounds)));
                }
            }
            click::Kind::Double => {
                if let Some(message) = widget.on_double_click.as_ref() {
                    shell.publish(message(cursor.position_in(layout_bounds)));
                }
            }
            click::Kind::Triple => {
                // TODO what to do here
                if let Some(message) = widget.on_press.as_ref() {
                    shell.publish(message(cursor.position_in(layout_bounds)));
                }
            }
        }
        if widget.on_drag.is_some() {
            state.drag_initiated = cursor.position();
        }

        if widget.on_press.is_some() {
            return event::Status::Captured;
        }
    }

    let distance_dragged = state
        .drag_initiated
        .map(|initiated| initiated.distance(cursor.position().unwrap_or_default()))
        .unwrap_or_default();
    if matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
    ) && distance_dragged > 1.0
    {
        state.drag_initiated = None;
        state.prev_click = None;
        if let Some(message) = widget.on_drag_end.as_ref() {
            shell.publish(message(cursor.position_in(layout_bounds)));
        }
    }

    let recent_click = state
        .prev_click
        .as_ref()
        .is_some_and(|(_, i)| Instant::now().duration_since(*i) <= DOUBLE_CLICK_DURATION);
    if matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
    ) && state.prev_click.is_some()
    {
        if !recent_click {
            state.prev_click = None;
            return event::Status::Ignored;
        }
        state.drag_initiated = None;
        if let Some(message) = widget.on_release.as_ref() {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_press.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
        ) {
            let point_opt = if widget.on_right_press_window_position {
                cursor.position_over(layout_bounds).map(|mut p| {
                    p.x -= offset.x;
                    p.y -= offset.y;
                    p
                })
            } else {
                cursor.position_in(layout_bounds)
            };
            shell.publish(message(point_opt));

            if widget.on_right_press_no_capture {
                return event::Status::Ignored;
            }
            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_release.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right))
        ) {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_press.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle))
        ) {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_release.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle))
        ) {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_back_press.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Back))
        ) {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_back_release.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Back))
        ) {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_forward_press.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward))
        ) {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_forward_release.as_ref() {
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Forward))
        ) {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(on_scroll) = widget.on_scroll.as_ref() {
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            if let Some(message) = on_scroll(*delta) {
                shell.publish(message);
                return event::Status::Captured;
            }
        }
    }

    if let Some((message, drag_rect)) = widget.on_drag.as_ref().zip(state.drag_rect(cursor)) {
        shell.publish(message(drag_rect.intersection(&layout_bounds).map(
            |mut rect| {
                rect.x -= layout_bounds.x;
                rect.y -= layout_bounds.y;
                rect
            },
        )));
    }

    event::Status::Ignored
}
