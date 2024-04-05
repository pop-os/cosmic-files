use std::borrow::Cow;

use cosmic::{
    iced::{
        clipboard::{
            dnd::{self, DndAction, DndDestinationRectangle, DndEvent, OfferEvent},
            mime::AllowedMimeTypes,
        },
        event,
        id::Internal,
        mouse, overlay, Event, Length, Rectangle,
    },
    iced_core::{
        self, layout,
        widget::{tree, Tree},
        Clipboard, Shell,
    },
    widget::{Id, Widget},
    Element,
};

pub struct DndDestinationWrapper<'a, Message> {
    id: Id,
    drag_id: Option<u64>,
    preferred_action: DndAction,
    action: DndAction,
    container: Element<'a, Message>,
    mime_types: Vec<Cow<'static, str>>,
    forward_drag_as_cursor: bool,
    on_hold: Option<Box<dyn Fn(u32, u32) -> Message>>,
    on_drop: Option<Box<dyn Fn(u32, u32) -> Message>>,
    on_enter: Option<Box<dyn Fn(u32, u32, Vec<String>) -> Message>>,
    on_leave: Option<Box<dyn Fn() -> Message>>,
    on_motion: Option<Box<dyn Fn(u32, u32) -> Message>>,
    on_action_selected: Option<Box<dyn Fn(DndAction) -> Message>>,
    on_data_received: Option<Box<dyn Fn(String, Vec<u8>) -> Message>>,
    on_finish: Option<Box<dyn Fn(String, Vec<u8>, DndAction) -> Message>>,
}

impl<'a, Message: 'static> DndDestinationWrapper<'a, Message> {
    pub fn new(child: impl Into<Element<'a, Message>>, mimes: Vec<Cow<'static, str>>) -> Self {
        Self {
            id: Id::unique(),
            drag_id: None,
            mime_types: mimes,
            preferred_action: DndAction::Move,
            action: DndAction::Copy | DndAction::Move,
            container: child.into(),
            forward_drag_as_cursor: false,
            on_hold: None,
            on_drop: None,
            on_enter: None,
            on_leave: None,
            on_motion: None,
            on_action_selected: None,
            on_data_received: None,
            on_finish: None,
        }
    }

    pub fn with_data<T: AllowedMimeTypes>(
        child: impl Into<Element<'a, Message>>,
        on_finish: impl Fn(Option<T>, DndAction) -> Message + 'static,
    ) -> Self {
        Self {
            id: Id::unique(),
            drag_id: None,
            mime_types: T::allowed().into_iter().cloned().map(Cow::Owned).collect(),
            preferred_action: DndAction::Move,
            action: DndAction::Copy | DndAction::Move,
            container: child.into(),
            forward_drag_as_cursor: false,
            on_hold: None,
            on_drop: None,
            on_enter: None,
            on_leave: None,
            on_motion: None,
            on_action_selected: None,
            on_data_received: None,
            on_finish: Some(Box::new(move |mime, data, action| {
                on_finish(T::try_from((data, mime)).ok(), action)
            })),
        }
    }

    pub fn data_received_for<T: AllowedMimeTypes>(
        mut self,
        f: impl Fn(Option<T>) -> Message + 'static,
    ) -> Self {
        self.on_data_received = Some(Box::new(
            move |mime, data| f(T::try_from((data, mime)).ok()),
        ));
        self
    }

    pub fn with_id(
        child: impl Into<Element<'a, Message>>,
        id: Id,
        mimes: Vec<Cow<'static, str>>,
    ) -> Self {
        Self {
            id,
            drag_id: None,
            mime_types: mimes,
            preferred_action: DndAction::Move,
            action: DndAction::Copy | DndAction::Move,
            container: child.into(),
            forward_drag_as_cursor: false,
            on_hold: None,
            on_drop: None,
            on_enter: None,
            on_leave: None,
            on_motion: None,
            on_action_selected: None,
            on_data_received: None,
            on_finish: None,
        }
    }

    pub fn with_drag_id(mut self, id: u64) -> Self {
        self.drag_id = Some(id);
        self
    }

    pub fn with_action(mut self, action: DndAction) -> Self {
        self.action = action;
        self
    }

    pub fn with_preferred_action(mut self, action: DndAction) -> Self {
        self.preferred_action = action;
        self
    }

    pub fn with_forward_drag_as_cursor(mut self, forward: bool) -> Self {
        self.forward_drag_as_cursor = forward;
        self
    }

    pub fn on_hold(mut self, f: impl Fn(u32, u32) -> Message + 'static) -> Self {
        self.on_hold = Some(Box::new(f));
        self
    }

    pub fn on_drop(mut self, f: impl Fn(u32, u32) -> Message + 'static) -> Self {
        self.on_drop = Some(Box::new(f));
        self
    }

    pub fn on_enter(mut self, f: impl Fn(u32, u32, Vec<String>) -> Message + 'static) -> Self {
        self.on_enter = Some(Box::new(f));
        self
    }

    pub fn on_leave(mut self, m: impl Fn() -> Message + 'static) -> Self {
        self.on_leave = Some(Box::new(m));
        self
    }

    pub fn on_finish(
        mut self,
        f: impl Fn(String, Vec<u8>, DndAction) -> Message + 'static,
    ) -> Self {
        self.on_finish = Some(Box::new(f));
        self
    }

    pub fn on_motion(mut self, f: impl Fn(u32, u32) -> Message + 'static) -> Self {
        self.on_motion = Some(Box::new(f));
        self
    }

    pub fn on_action_selected(mut self, f: impl Fn(DndAction) -> Message + 'static) -> Self {
        self.on_action_selected = Some(Box::new(f));
        self
    }

    pub fn on_data_received(mut self, f: impl Fn(String, Vec<u8>) -> Message + 'static) -> Self {
        self.on_data_received = Some(Box::new(f));
        self
    }

    pub fn drag_id(&self) -> u128 {
        self.drag_id.unwrap_or_else(|| match &self.id.0 {
            Internal::Unique(id) => *id,
            Internal::Custom(id, _) => *id,
            Internal::Set(_) => panic!("Invalid Id assigned to dnd destination."),
        }) as u128
    }
}

impl<'a, Message: 'static> Widget<Message, cosmic::Theme, cosmic::Renderer>
    for DndDestinationWrapper<'a, Message>
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.container)]
    }

    fn tag(&self) -> iced_core::widget::tree::Tag {
        tree::Tag::of::<State>()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.container.as_widget_mut().diff(&mut tree.children[0]);
    }

    fn state(&self) -> iced_core::widget::tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> iced_core::Size<Length> {
        self.container.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &cosmic::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.container
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: layout::Layout<'_>,
        renderer: &cosmic::Renderer,
        operation: &mut dyn iced_core::widget::Operation<
            iced_core::widget::OperationOutputWrapper<Message>,
        >,
    ) {
        self.container
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &cosmic::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let s = self.container.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if matches!(s, event::Status::Captured) {
            return event::Status::Captured;
        }

        let state = tree.state.downcast_mut::<State>();

        let my_id = self.drag_id();

        match event {
            Event::Dnd(DndEvent::Offer(
                id,
                OfferEvent::Enter {
                    x, y, mime_types, ..
                },
            )) if id == Some(my_id) => {
                if let Some(f) = &self.on_enter {
                    shell.publish(f(x as u32, y as u32, mime_types));
                }
                state.drag_offer = Some(DragOffer {
                    x,
                    y,
                    dropped: false,
                    selected_action: DndAction::empty(),
                });
                if self.forward_drag_as_cursor {
                    let drag_cursor = mouse::Cursor::Available((x as f32, y as f32).into());
                    let event = Event::Mouse(mouse::Event::CursorMoved {
                        position: drag_cursor.position().unwrap(),
                    });
                    self.container.as_widget_mut().on_event(
                        &mut tree.children[0],
                        event,
                        layout,
                        drag_cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(_, OfferEvent::Leave)) => {
                if let Some(f) = &self.on_leave {
                    state.drag_offer = None;
                    shell.publish(f());
                }

                // If the offer was dropped, we don't want to send a leave event.
                if !state.drag_offer.as_ref().is_some_and(|d| d.dropped) {
                    state.drag_offer = None;
                }

                if self.forward_drag_as_cursor {
                    let drag_cursor = mouse::Cursor::Unavailable;
                    let event = Event::Mouse(mouse::Event::CursorLeft);
                    self.container.as_widget_mut().on_event(
                        &mut tree.children[0],
                        event,
                        layout,
                        drag_cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::Motion { x, y })) if id == Some(my_id) => {
                if let Some(s) = state.drag_offer.as_mut() {
                    s.x = x;
                    s.y = y;
                } else {
                    state.drag_offer = Some(DragOffer {
                        x,
                        y,
                        dropped: false,
                        selected_action: DndAction::empty(),
                    });
                    if let Some(f) = &self.on_enter {
                        shell.publish(f(x as u32, y as u32, vec![]));
                    }
                }

                if let Some(f) = &self.on_motion {
                    shell.publish(f(x as u32, y as u32));
                }

                if self.forward_drag_as_cursor {
                    let drag_cursor = mouse::Cursor::Available((x as f32, y as f32).into());
                    let event = Event::Mouse(mouse::Event::CursorMoved {
                        position: drag_cursor.position().unwrap(),
                    });
                    self.container.as_widget_mut().on_event(
                        &mut tree.children[0],
                        event,
                        layout,
                        drag_cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    );
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::LeaveDestination)) if id == Some(my_id) => {
                if state.drag_offer.take().is_some() {
                    if let Some(f) = &self.on_leave {
                        shell.publish(f());
                    }
                }
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::Drop)) if id == Some(my_id) => {
                if let Some(offer) = &state.drag_offer {
                    if let Some(f) = &self.on_drop {
                        shell.publish(f(offer.x as u32, offer.y as u32));
                    }
                }
                if let Some(s) = state.drag_offer.as_mut() {
                    s.dropped = true;
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::SelectedAction(action)))
                if id == Some(my_id) =>
            {
                if let Some(f) = &self.on_action_selected {
                    shell.publish(f(action));
                }
                if let Some(offer) = state.drag_offer.as_mut() {
                    offer.selected_action = action;
                }
                return event::Status::Captured;
            }
            Event::Dnd(DndEvent::Offer(id, OfferEvent::Data { data, mime_type }))
                if id == Some(my_id) =>
            {
                let Some(dnd) = state.drag_offer.take() else {
                    return event::Status::Ignored;
                };

                if !dnd.dropped {
                    if let Some(f) = &self.on_data_received {
                        shell.publish(f(mime_type, data));
                    }
                    state.drag_offer = Some(dnd);
                } else {
                    // send finish message
                    if let Some(f) = &self.on_finish {
                        shell.publish(f(mime_type, data, dnd.selected_action));
                    }
                }
                return event::Status::Captured;
            }
            _ => {}
        }
        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: layout::Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &cosmic::Renderer,
    ) -> mouse::Interaction {
        self.container.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut cosmic::Renderer,
        theme: &cosmic::Theme,
        renderer_style: &iced_core::renderer::Style,
        layout: layout::Layout<'_>,
        cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.container.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: layout::Layout<'_>,
        renderer: &cosmic::Renderer,
    ) -> Option<overlay::Element<'b, Message, cosmic::Theme, cosmic::Renderer>> {
        self.container
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer)
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: layout::Layout<'_>,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        let bounds = layout.bounds();
        let my_id = self.drag_id();
        let my_dest = DndDestinationRectangle {
            id: my_id as u128,
            rectangle: dnd::Rectangle {
                x: bounds.x as f64,
                y: bounds.y as f64,
                width: bounds.width as f64,
                height: bounds.height as f64,
            },
            mime_types: self.mime_types.clone(),
            actions: self.action,
            preferred: self.preferred_action,
        };
        dnd_rectangles.push(my_dest);

        self.container
            .as_widget()
            .drag_destinations(&state.children[0], layout, dnd_rectangles);
    }

    fn id(&self) -> Option<Id> {
        Some(self.id.clone())
    }

    fn set_id(&mut self, id: Id) {
        self.id = id;
    }
}

pub struct State {
    pub drag_offer: Option<DragOffer>,
}

pub struct DragOffer {
    pub x: f64,
    pub y: f64,
    pub dropped: bool,
    pub selected_action: DndAction,
}

impl State {
    pub fn new() -> Self {
        Self { drag_offer: None }
    }
}

impl<'a, Message: 'static> From<DndDestinationWrapper<'a, Message>> for Element<'a, Message> {
    fn from(wrapper: DndDestinationWrapper<'a, Message>) -> Self {
        Element::new(wrapper)
    }
}
