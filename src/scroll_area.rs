//! A container for capturing mouse wheel events.

use cosmic::{
    iced_core::{
        event::{self, Event},
        layout,
        mouse::{self},
        overlay,
        renderer::{self},
        widget::{Operation, OperationOutputWrapper, Tree},
        Clipboard, Layout, Length, Rectangle, Shell, Size, Widget,
    },
    widget::Id,
    Element, Renderer, Theme,
};

/// Emit messages on mouse wheel events.
#[allow(missing_debug_implementations)]
pub struct ScrollArea<'a, Message> {
    id: Id,
    content: Element<'a, Message>,
    on_scroll: Option<Box<dyn Fn(Option<mouse::ScrollDelta>) -> Option<Message> + 'a>>,
    should_propogate_events: bool,
}

impl<'a, Message> ScrollArea<'a, Message> {
    /// The message to emit on a forward button release.
    #[must_use]
    pub fn on_scroll(
        mut self,
        message: impl Fn(Option<mouse::ScrollDelta>) -> Option<Message> + 'a,
    ) -> Self {
        self.on_scroll = Some(Box::new(message));
        self
    }

    /// Sets the widget's unique identifier.
    #[must_use]
    pub fn with_id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }
}

impl<'a, Message> ScrollArea<'a, Message> {
    /// Creates a [`ScrollArea`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message>>, should_propogate_events: bool) -> Self {
        ScrollArea {
            id: Id::unique(),
            content: content.into(),
            on_scroll: None,
            should_propogate_events,
        }
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for ScrollArea<'a, Message>
where
    Message: Clone,
{
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

        update(self, &event, layout, cursor, shell)
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

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

impl<'a, Message> From<ScrollArea<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
    Renderer: 'a + renderer::Renderer,
    Theme: 'a,
{
    fn from(area: ScrollArea<'a, Message>) -> Element<'a, Message> {
        Element::new(area)
    }
}

/// Processes the given [`Event`] and updates the [`State`] of a [`ScrollArea`]
/// accordingly.
fn update<Message: Clone>(
    widget: &mut ScrollArea<'_, Message>,
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
) -> event::Status {
    let layout_bounds = layout.bounds();
    if !cursor.is_over(layout_bounds) {
        return event::Status::Ignored;
    }

    if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
        if let Some(message) = widget.on_scroll.as_ref() {
            if let Some(msg) = message(Some(delta.clone())) {
                shell.publish(msg);
                if !widget.should_propogate_events {
                    return event::Status::Captured;
                }
            }
        }
    }

    event::Status::Ignored
}
