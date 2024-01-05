//! A container for capturing mouse events.

use cosmic::iced_core::{
    event::{self, Event},
    layout, mouse, overlay, renderer, touch,
    widget::{tree, Operation, OperationOutputWrapper, Tree},
    {Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Widget},
};

/// Emit messages on mouse events.
#[allow(missing_debug_implementations)]
pub struct MouseArea<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
    on_drag: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_right_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_right_press_no_capture: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_right_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_middle_press: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
    on_middle_release: Option<Box<dyn Fn(Option<Point>) -> Message + 'a>>,
}

impl<'a, Message, Renderer> MouseArea<'a, Message, Renderer> {
    /// The message to emit when a drag is initiated.
    #[must_use]
    pub fn on_drag(mut self, message: impl Fn(Option<Point>) -> Message + 'a) -> Self {
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
}

/// Local state of the [`MouseArea`].
#[derive(Default)]
struct State {
    // TODO: Support on_mouse_enter and on_mouse_exit
    drag_initiated: Option<Point>,
}

impl<'a, Message, Renderer> MouseArea<'a, Message, Renderer> {
    /// Creates a [`MouseArea`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        MouseArea {
            content: content.into(),
            on_drag: None,
            on_press: None,
            on_release: None,
            on_right_press: None,
            on_right_press_no_capture: None,
            on_right_release: None,
            on_middle_press: None,
            on_middle_release: None,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for MouseArea<'a, Message, Renderer>
where
    Renderer: renderer::Renderer,
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

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
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
        theme: &Renderer::Theme,
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
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer)
    }
}

impl<'a, Message, Renderer> From<MouseArea<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + renderer::Renderer,
{
    fn from(area: MouseArea<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(area)
    }
}

/// Processes the given [`Event`] and updates the [`State`] of an [`MouseArea`]
/// accordingly.
fn update<Message: Clone, Renderer>(
    widget: &mut MouseArea<'_, Message, Renderer>,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    state: &mut State,
) -> event::Status {
    if !cursor.is_over(layout.bounds()) {
        return event::Status::Ignored;
    }

    if let Some(message) = widget.on_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) = event
        {
            state.drag_initiated = cursor.position();
            shell.publish(message(cursor.position_in(layout.bounds())));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) = event
        {
            state.drag_initiated = None;
            shell.publish(message(cursor.position_in(layout.bounds())));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            shell.publish(message(cursor.position_in(layout.bounds())));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_press_no_capture.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            shell.publish(message(cursor.position_in(layout.bounds())));

            return event::Status::Ignored;
        }
    }

    if let Some(message) = widget.on_right_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) = event {
            shell.publish(message(cursor.position_in(layout.bounds())));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) = event {
            shell.publish(message(cursor.position_in(layout.bounds())));

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) = event {
            shell.publish(message(cursor.position_in(layout.bounds())));

            return event::Status::Captured;
        }
    }

    if state.drag_initiated.is_none() && widget.on_drag.is_some() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) = event
        {
            state.drag_initiated = cursor.position();
        }
    } else if let Some((message, drag_source)) = widget.on_drag.as_ref().zip(state.drag_initiated) {
        if let Some(position) = cursor.position() {
            if position.distance(drag_source) > 1.0 {
                state.drag_initiated = None;
                shell.publish(message(cursor.position_in(layout.bounds())));

                return event::Status::Captured;
            }
        }
    }

    event::Status::Ignored
}
