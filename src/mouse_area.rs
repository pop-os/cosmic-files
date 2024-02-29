//! A container for capturing mouse events.

use cosmic::{
    iced_core::{
        border::Border,
        event::{self, Event},
        layout, mouse, overlay,
        renderer::{self, Quad, Renderer as _},
        touch,
        widget::{tree, Operation, OperationOutputWrapper, Tree},
        Clipboard, Color, Layout, Length, Point, Rectangle, Shell, Size, Widget,
    },
    Element, Renderer, Theme,
};

/// Emit messages on mouse events.
#[allow(missing_debug_implementations)]
pub struct MouseArea<'a, Message> {
    content: Element<'a, Message>,
    on_drag: Option<Box<dyn Fn(Option<Rectangle>) -> Message + 'a>>,
    on_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
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
    show_drag_rect: bool,
}

impl<'a, Message> MouseArea<'a, Message> {
    /// The message to emit when a drag is initiated.
    #[must_use]
    pub fn on_drag(mut self, message: impl Fn(Option<Rectangle>) -> Message + 'a) -> Self {
        self.on_drag = Some(Box::new(message));
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

    #[must_use]
    pub fn show_drag_rect(mut self, show_drag_rect: bool) -> Self {
        self.show_drag_rect = show_drag_rect;
        self
    }
}

/// Local state of the [`MouseArea`].
#[derive(Default)]
struct State {
    last_size: Option<Size>,
    // TODO: Support on_mouse_enter and on_mouse_exit
    drag_initiated: Option<Point>,
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
}

impl<'a, Message> MouseArea<'a, Message> {
    /// Creates a [`MouseArea`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        MouseArea {
            content: content.into(),
            on_drag: None,
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
        operation: &mut dyn Operation<OperationOutputWrapper<Message>>,
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
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer)
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
        if state.last_size != Some(size) {
            shell.publish(message(size));
            state.last_size = Some(size);
        }
    }

    if state.drag_initiated.is_none() && !cursor.is_over(layout_bounds) {
        return event::Status::Ignored;
    }

    if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
    | Event::Touch(touch::Event::FingerPressed { .. }) = event
    {
        state.drag_initiated = cursor.position();
        if let Some(message) = widget.on_press.as_ref() {
            shell.publish(message(cursor.position_in(layout_bounds)));

            return event::Status::Captured;
        }
    }

    if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
    | Event::Touch(touch::Event::FingerLifted { .. }) = event
    {
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
