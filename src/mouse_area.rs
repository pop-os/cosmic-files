//! A container for capturing mouse events.

use std::time::Instant;

use cosmic::{
    iced_core::{
        border::Border,
        event::{self, Event},
        keyboard::{
            self,
            key::{self, Key},
            Event::{KeyPressed, KeyReleased},
            Modifiers,
        },
        layout,
        mouse::{self, click},
        overlay,
        renderer::{self, Quad, Renderer as _},
        touch,
        widget::{tree, Operation, Tree},
        Clipboard, Color, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget,
    },
    widget::Id,
    Element, Renderer, Theme,
};

use crate::tab::DOUBLE_CLICK_DURATION;

/// Emit messages on mouse events.
#[allow(missing_debug_implementations)]
pub struct MouseArea<'a, Message> {
    id: Id,
    content: Element<'a, Message>,
    on_drag: Option<Box<dyn Fn(Option<Rectangle>) -> Message + 'a>>,
    on_double_click: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_drag_end: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_resize: Option<Box<dyn Fn(Size) -> Message + 'a>>,
    on_right_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_right_press_no_capture: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_right_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_middle_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_middle_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_back_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_back_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_forward_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_forward_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_scroll: Option<Box<dyn Fn(mouse::ScrollDelta, Modifiers) -> Option<Message> + 'a>>,
    on_enter: Option<Box<dyn Fn() -> Message + 'a>>,
    on_exit: Option<Box<dyn Fn() -> Message + 'a>>,
    show_drag_rect: bool,
}

impl<'a, Message> MouseArea<'a, Message> {
    /// The message to emit when a drag is initiated.
    #[must_use]
    pub fn on_drag(mut self, message: impl Fn(Option<Rectangle>) -> Message + 'a) -> Self {
        self.on_drag = Some(Box::new(message));
        self
    }

    /// The message to emit when a drag ends.
    #[must_use]
    pub fn on_drag_end(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_drag_end = Some(Box::new(message));
        self
    }

    /// The message to emit on a double click.
    #[must_use]
    pub fn on_double_click(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_double_click = Some(Box::new(message));
        self
    }

    /// The message to emit on a left button press.
    #[must_use]
    pub fn on_press(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a left button release.
    #[must_use]
    pub fn on_release(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_release = Some(Box::new(message));
        self
    }

    /// The message to emit on resizing.
    #[must_use]
    pub fn on_resize(mut self, message: impl Fn(Size) -> Message + 'a) -> Self {
        self.on_resize = Some(Box::new(message));
        self
    }

    /// The message to emit on a right button press.
    #[must_use]
    pub fn on_right_press(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_right_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a right button press without capturing.
    #[must_use]
    pub fn on_right_press_no_capture(
        mut self,
        message: impl Fn(Option<Point>) -> Message + 'a,
    ) -> Self {
        self.on_right_press_no_capture = Some(Box::new(message));
        self
    }

    /// The message to emit on a right button release.
    #[must_use]
    pub fn on_right_release(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_right_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a middle button press.
    #[must_use]
    pub fn on_middle_press(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_middle_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a middle button release.
    #[must_use]
    pub fn on_middle_release(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_middle_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a back button press.
    #[must_use]
    pub fn on_back_press(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_back_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a back button release.
    #[must_use]
    pub fn on_back_release(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_back_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a forward button press.
    #[must_use]
    pub fn on_forward_press(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_forward_press = Some(Box::new(message));
        self
    }

    /// The message to emit on a forward button release.
    #[must_use]
    pub fn on_forward_release(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
        self.on_forward_release = Some(Box::new(message));
        self
    }

    /// The message to emit on a scroll.
    #[must_use]
    pub fn on_scroll(
        mut self,
        message: impl Fn(mouse::ScrollDelta, Modifiers) -> Option<Message> + 'a,
    ) -> Self {
        self.on_scroll = Some(Box::new(message));
        self
    }

    /// The message to emit when a mouse enters the area.
    #[must_use]
    pub fn on_enter(mut self, message: impl Fn() -> Message + 'a) -> Self {
        self.on_enter = Some(Box::new(message));
        self
    }

    /// The message to emit when a mouse exits the area.
    #[must_use]
    pub fn on_exit(mut self, message: impl Fn() -> Message + 'a) -> Self {
        self.on_exit = Some(Box::new(message));
        self
    }

    #[must_use]
    pub fn show_drag_rect(mut self, show_drag_rect: bool) -> Self {
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

/// Local state of the [`MouseArea`].
#[derive(Default)]
struct State {
    last_position: Option<Point>,
    drag_initiated: Option<Point>,
    modifiers: Modifiers,
    prev_click: Option<(mouse::Click, Instant)>,
    size: Option<Size>,
}

impl State {
    fn drag_rect(&self, cursor: mouse::Cursor) -> Option<Rectangle> {
        if let Some(drag_source) = self.drag_initiated {
            if let Some(position) = cursor.position() {
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
        self.prev_click = Some((new.clone(), now));
        new
    }
}

impl<'a, Message> MouseArea<'a, Message> {
    /// Creates a [`MouseArea`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        MouseArea {
            id: Id::unique(),
            content: content.into(),
            on_drag: None,
            on_drag_end: None,
            on_double_click: None,
            on_press: None,
            on_release: None,
            on_resize: None,
            on_right_press: None,
            on_right_press_no_capture: None,
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

impl<'a, Message> Widget<Message, Theme, Renderer> for MouseArea<'a, Message>
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
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }

        update(
            self,
            &event,
            layout,
            cursor,
            shell,
            tree.state.downcast_mut::<State>(),
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
    fn from(area: MouseArea<'a, Message>) -> Element<'a, Message> {
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
) -> event::Status {
    let layout_bounds = layout.bounds();

    if let Some(message) = widget.on_resize.as_ref() {
        let size = layout_bounds.size();
        if state.size != Some(size) {
            state.size = Some(size);
            shell.publish(message(size));
        }
    }

    if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
        let position_in = cursor.position_in(layout_bounds);
        match (position_in, state.last_position) {
            (None, Some(_)) => {
                if let Some(message) = widget.on_exit.as_ref() {
                    shell.publish(message())
                }
            }
            (Some(new), None) => {
                if let Some(message) = widget.on_enter.as_ref() {
                    shell.publish(message())
                }
            }
            _ => {}
        }
        state.last_position = position_in;
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
        .map(|(_, i)| Instant::now().duration_since(*i) <= DOUBLE_CLICK_DURATION)
        .unwrap_or_default();
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
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_press_no_capture.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Ignored;
        }
    }

    if let Some(message) = widget.on_right_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_back_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Back)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_back_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Back)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_forward_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Forward)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_forward_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Forward)) = event {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Some(on_scroll) = widget.on_scroll.as_ref() {
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            if let Some(message) = on_scroll(delta.clone(), state.modifiers) {
                shell.publish(message);
                return event::Status::Captured;
            }
        }
    }

    if let Event::Keyboard(key_event) = event {
        handle_key_event(key_event, state)
    };

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

fn handle_key_event(key_event: &keyboard::Event, state: &mut State) {
    if let KeyPressed {
        key: Key::Named(key::Named::Control),
        ..
    } = key_event
    {
        state.modifiers.insert(Modifiers::CTRL);
    }

    if let KeyReleased {
        key: Key::Named(key::Named::Control),
        ..
    } = key_event
    {
        state.modifiers.remove(Modifiers::CTRL);
    }

    if let KeyPressed {
        key: Key::Named(key::Named::Shift),
        ..
    } = key_event
    {
        state.modifiers.insert(Modifiers::SHIFT);
    }

    if let KeyReleased {
        key: Key::Named(key::Named::Shift),
        ..
    } = key_event
    {
        state.modifiers.remove(Modifiers::SHIFT);
    }

    if let KeyPressed {
        key: Key::Named(key::Named::Alt),
        ..
    } = key_event
    {
        state.modifiers.insert(Modifiers::ALT);
    }

    if let KeyReleased {
        key: Key::Named(key::Named::Alt),
        ..
    } = key_event
    {
        state.modifiers.remove(Modifiers::ALT);
    }
}
