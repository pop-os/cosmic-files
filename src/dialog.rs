// Copyright 2023 System76 <inflist_o@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{
        self,
        cosmic::{Cosmic, Message as CosmicMessage},
        message, Command, Core,
    },
    cosmic_theme, executor,
    iced::{
        event,
        futures::{self, SinkExt},
        keyboard::{Event as KeyEvent, Modifiers},
        multi_window::Application as IcedApplication,
        subscription::{self, Subscription},
        window, Event, Length, Size,
    },
    style,
    widget::{self, segmented_button},
    Application, ApplicationExt, Element,
};
use notify::Watcher;
use std::{
    any::TypeId,
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex},
    time,
};

use crate::{
    config::TabConfig,
    fl, home_dir,
    tab::{self, Location, Tab},
};

#[derive(Clone, Debug)]
pub struct DialogMessage(app::Message<Message>);

#[derive(Clone, Debug)]
pub enum DialogResult {
    Cancel,
    Open(Vec<PathBuf>),
}

pub struct Dialog<M> {
    cosmic: Cosmic<App>,
    mapper: fn(DialogMessage) -> M,
    on_result: fn(DialogResult) -> M,
}

impl<M: 'static> Dialog<M> {
    pub fn new(
        mapper: fn(DialogMessage) -> M,
        on_result: fn(DialogResult) -> M,
    ) -> (Self, Command<M>) {
        let mut settings = window::Settings::default();
        settings.decorations = false;
        settings.exit_on_close_request = false;
        settings.transparent = true;
        settings.platform_specific.application_id = App::APP_ID.to_string();
        {
            //TODO: allow resize!
            settings.size = Size::new(800.0, 600.0);
            settings.resizable = false;
        }
        let (window_id, window_command) = window::spawn(settings);

        let core = Core::default();
        let flags = Flags { window_id };
        let (cosmic, cosmic_command) = <Cosmic<App> as IcedApplication>::new((core, flags));

        (
            Self {
                cosmic,
                mapper,
                on_result,
            },
            Command::batch([window_command, cosmic_command])
                .map(DialogMessage)
                .map(move |message| app::Message::App(mapper(message))),
        )
    }

    pub fn subscription(&self) -> Subscription<M> {
        self.cosmic
            .subscription()
            .map(DialogMessage)
            .map(self.mapper)
    }

    pub fn update(&mut self, message: DialogMessage) -> Command<M> {
        let mapper = self.mapper;
        let command = self
            .cosmic
            .update(message.0)
            .map(DialogMessage)
            .map(move |message| app::Message::App(mapper(message)));
        if let Some(result) = self.cosmic.app.result_opt.take() {
            let on_result = self.on_result;
            Command::batch([
                command,
                Command::perform(async move { app::Message::App(on_result(result)) }, |x| x),
            ])
        } else {
            command
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<M> {
        self.cosmic
            .view(window_id)
            .map(DialogMessage)
            .map(self.mapper)
    }
}

#[derive(Clone, Debug)]
struct Flags {
    window_id: window::Id,
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
enum Message {
    Cancel,
    Modifiers(Modifiers),
    NotifyEvent(notify::Event),
    NotifyWatcher(WatcherWrapper),
    Open,
    SelectAll(Option<segmented_button::Entity>),
    TabActivate(segmented_button::Entity),
    TabClose(Option<segmented_button::Entity>),
    TabMessage(Option<segmented_button::Entity>, tab::Message),
    TabRescan(segmented_button::Entity, Vec<tab::Item>),
}

#[derive(Debug)]
struct WatcherWrapper {
    watcher_opt: Option<notify::RecommendedWatcher>,
}

impl Clone for WatcherWrapper {
    fn clone(&self) -> Self {
        Self { watcher_opt: None }
    }
}

impl PartialEq for WatcherWrapper {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

/// The [`App`] stores application-specific state.
struct App {
    core: Core,
    flags: Flags,
    modifiers: Modifiers,
    nav_model: segmented_button::SingleSelectModel,
    result_opt: Option<DialogResult>,
    tab_model: segmented_button::Model<segmented_button::SingleSelect>,
    watcher_opt: Option<(notify::RecommendedWatcher, HashSet<PathBuf>)>,
}

impl App {
    fn open_tab(&mut self, location: Location) -> Command<Message> {
        let mut tab = Tab::new(location.clone(), TabConfig::default());
        tab.dialog = true;
        let entity = self
            .tab_model
            .insert()
            .text(tab.title())
            .data(tab)
            .closable()
            .activate()
            .id();
        Command::batch([
            self.update_title(),
            self.update_watcher(),
            self.rescan_tab(entity, location),
        ])
    }

    fn rescan_tab(
        &mut self,
        entity: segmented_button::Entity,
        location: Location,
    ) -> Command<Message> {
        Command::perform(
            async move {
                match tokio::task::spawn_blocking(move || location.scan()).await {
                    Ok(items) => message::app(Message::TabRescan(entity, items)),
                    Err(err) => {
                        log::warn!("failed to rescan: {}", err);
                        message::none()
                    }
                }
            },
            |x| x,
        )
    }

    fn update_title(&mut self) -> Command<Message> {
        let (header_title, window_title) = match self.tab_model.text(self.tab_model.active()) {
            Some(tab_title) => (
                tab_title.to_string(),
                format!("{tab_title} — COSMIC File Manager"),
            ),
            None => (String::new(), "COSMIC File Manager".to_string()),
        };
        self.set_header_title(header_title);
        self.set_window_title(window_title, self.main_window_id())
    }

    fn update_watcher(&mut self) -> Command<Message> {
        if let Some((mut watcher, old_paths)) = self.watcher_opt.take() {
            let mut new_paths = HashSet::new();
            for entity in self.tab_model.iter() {
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    if let Location::Path(path) = &tab.location {
                        new_paths.insert(path.clone());
                    }
                }
            }

            // Unwatch paths no longer used
            for path in old_paths.iter() {
                if !new_paths.contains(path) {
                    match watcher.unwatch(path) {
                        Ok(()) => {
                            log::debug!("unwatching {:?}", path);
                        }
                        Err(err) => {
                            log::debug!("failed to unwatch {:?}: {}", path, err);
                        }
                    }
                }
            }

            // Watch new paths
            for path in new_paths.iter() {
                if !old_paths.contains(path) {
                    //TODO: should this be recursive?
                    match watcher.watch(path, notify::RecursiveMode::NonRecursive) {
                        Ok(()) => {
                            log::debug!("watching {:?}", path);
                        }
                        Err(err) => {
                            log::debug!("failed to watch {:?}: {}", path, err);
                        }
                    }
                }
            }

            self.watcher_opt = Some((watcher, new_paths));
        }

        //TODO: should any of this run in a command?
        Command::none()
    }
}

/// Implement [`Application`] to integrate with COSMIC.
impl Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received
    type Flags = Flags;

    /// Message type specific to our [`App`].
    type Message = Message;

    /// The unique application ID to supply to the window manager.
    const APP_ID: &'static str = "com.system76.CosmicFilesDialog";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(mut core: Core, flags: Self::Flags) -> (Self, Command<Message>) {
        core.window.show_maximize = false;
        core.window.show_minimize = false;

        let mut nav_model = segmented_button::ModelBuilder::default();
        if let Some(dir) = dirs::home_dir() {
            nav_model = nav_model.insert(move |b| {
                b.text(fl!("home"))
                    .icon(widget::icon::icon(tab::folder_icon_symbolic(&dir, 16)).size(16))
                    .data(Location::Path(dir.clone()))
            });
        }
        //TODO: Sort by name?
        for dir_opt in &[
            dirs::document_dir(),
            dirs::download_dir(),
            dirs::audio_dir(),
            dirs::picture_dir(),
            dirs::video_dir(),
        ] {
            if let Some(dir) = dir_opt {
                if let Some(file_name) = dir.file_name().and_then(|x| x.to_str()) {
                    nav_model = nav_model.insert(move |b| {
                        b.text(file_name.to_string())
                            .icon(widget::icon::icon(tab::folder_icon_symbolic(&dir, 16)).size(16))
                            .data(Location::Path(dir.clone()))
                    });
                }
            }
        }

        let mut app = App {
            core,
            flags,
            modifiers: Modifiers::empty(),
            nav_model: nav_model.build(),
            result_opt: None,
            tab_model: segmented_button::ModelBuilder::default().build(),
            watcher_opt: None,
        };

        let mut commands = Vec::new();

        if app.tab_model.iter().next().is_none() {
            commands.push(app.open_tab(Location::Path(home_dir())));
        }

        (app, Command::batch(commands))
    }

    fn main_window_id(&self) -> window::Id {
        self.flags.window_id
    }

    // The default nav_bar widget needs to have its width reduced for cosmic-files
    fn nav_bar(&self) -> Option<Element<message::Message<Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = widget::nav_bar(nav_model, |entity| {
            message::cosmic(cosmic::app::cosmic::Message::NavBar(entity))
        });

        if !self.core().is_condensed() {
            nav = nav.max_width(200);
        }

        Some(Element::from(nav))
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_app_exit(&mut self) {
        self.result_opt = Some(DialogResult::Cancel);
    }

    fn on_nav_select(&mut self, entity: segmented_button::Entity) -> Command<Message> {
        let location_opt = self.nav_model.data::<Location>(entity).clone();

        if let Some(location) = location_opt {
            let message = Message::TabMessage(None, tab::Message::Location(location.clone()));
            return self.update(message);
        }

        Command::none()
    }

    /// Handle application events here.
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Cancel => {
                self.result_opt = Some(DialogResult::Cancel);
                return window::close(self.main_window_id());
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::NotifyEvent(event) => {
                log::debug!("{:?}", event);

                let mut needs_reload = Vec::new();
                for entity in self.tab_model.iter() {
                    if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                        //TODO: support reloading trash, somehow
                        if let Location::Path(path) = &tab.location {
                            let mut contains_change = false;
                            for event_path in event.paths.iter() {
                                if event_path.starts_with(&path) {
                                    contains_change = true;
                                    break;
                                }
                            }
                            if contains_change {
                                needs_reload.push((entity, tab.location.clone()));
                            }
                        }
                    }
                }

                let mut commands = Vec::with_capacity(needs_reload.len());
                for (entity, location) in needs_reload {
                    commands.push(self.rescan_tab(entity, location));
                }
                return Command::batch(commands);
            }
            Message::NotifyWatcher(mut watcher_wrapper) => match watcher_wrapper.watcher_opt.take()
            {
                Some(mut watcher) => {
                    self.watcher_opt = Some((watcher, HashSet::new()));
                    return self.update_watcher();
                }
                None => {
                    log::warn!("message did not contain notify watcher");
                }
            },
            Message::Open => {
                let mut paths = Vec::new();
                let entity = self.tab_model.active();
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(ref mut items) = tab.items_opt {
                        for item in items.iter_mut() {
                            if item.selected {
                                paths.push(item.path.clone());
                            }
                        }
                    }
                }
                self.result_opt = Some(DialogResult::Open(paths));
                return window::close(self.main_window_id());
            }
            Message::SelectAll(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(ref mut items) = tab.items_opt {
                        for item in items.iter_mut() {
                            if !tab.config.show_hidden && item.hidden {
                                continue;
                            }
                            item.selected = true;
                            item.click_time = None;
                        }
                    }
                }
            }
            Message::TabActivate(entity) => {
                self.tab_model.activate(entity);
                return self.update_title();
            }
            Message::TabClose(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());

                // Activate closest item
                if let Some(position) = self.tab_model.position(entity) {
                    if position > 0 {
                        self.tab_model.activate_position(position - 1);
                    } else {
                        self.tab_model.activate_position(position + 1);
                    }
                }

                // Remove item
                self.tab_model.remove(entity);

                // If that was the last tab, close window
                if self.tab_model.iter().next().is_none() {
                    return window::close(self.main_window_id());
                }

                return Command::batch([self.update_title(), self.update_watcher()]);
            }
            Message::TabMessage(entity_opt, tab_message) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());

                let mut update_opt = None;
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        if tab.update(tab_message, self.modifiers) {
                            update_opt = Some((tab.title(), tab.location.clone()));
                        }
                    }
                    _ => (),
                }
                if let Some((tab_title, tab_path)) = update_opt {
                    self.tab_model.text_set(entity, tab_title);
                    return Command::batch([
                        self.update_title(),
                        self.update_watcher(),
                        self.rescan_tab(entity, tab_path),
                    ]);
                }
            }
            Message::TabRescan(entity, items) => match self.tab_model.data_mut::<Tab>(entity) {
                Some(tab) => {
                    tab.items_opt = Some(items);
                }
                _ => (),
            },
        }

        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        let mut tab_column = widget::column::with_capacity(1);

        if self.tab_model.iter().count() > 1 {
            tab_column = tab_column.push(
                widget::container(
                    widget::view_switcher::horizontal(&self.tab_model)
                        .button_height(32)
                        .button_spacing(space_xxs)
                        .on_activate(Message::TabActivate)
                        .on_close(|entity| Message::TabClose(Some(entity))),
                )
                .style(style::Container::Background)
                .width(Length::Fill),
            );
        }

        let entity = self.tab_model.active();
        match self.tab_model.data::<Tab>(entity) {
            Some(tab) => {
                tab_column = tab_column.push(
                    tab.view(self.core())
                        .map(move |message| Message::TabMessage(Some(entity), message)),
                );
            }
            None => {
                //TODO
            }
        }

        tab_column = tab_column.push(
            widget::row::with_children(vec![
                widget::horizontal_space(Length::Fill).into(),
                widget::button(widget::text(fl!("cancel")))
                    .on_press(Message::Cancel)
                    .into(),
                widget::button(widget::text(fl!("open")))
                    .on_press(Message::Open)
                    .into(),
            ])
            .padding(space_xxs)
            .spacing(space_xxs),
        );

        let content: Element<_> = tab_column.into();

        // Uncomment to debug layout:
        //content.explain(cosmic::iced::Color::WHITE)
        content
    }

    fn subscription(&self) -> Subscription<Message> {
        struct WatcherSubscription;

        Subscription::batch([
            event::listen_with(|event, _status| match event {
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            subscription::channel(
                TypeId::of::<WatcherSubscription>(),
                100,
                |mut output| async move {
                    let watcher_res = {
                        let mut output = output.clone();
                        //TODO: debounce
                        notify::recommended_watcher(
                            move |event_res: Result<notify::Event, notify::Error>| match event_res {
                                Ok(event) => {
                                    match &event.kind {
                                        notify::EventKind::Access(_)
                                        | notify::EventKind::Modify(
                                            notify::event::ModifyKind::Metadata(_),
                                        ) => {
                                            // Data not mutated
                                            return;
                                        }
                                        _ => {}
                                    }

                                    match futures::executor::block_on(async {
                                        output.send(Message::NotifyEvent(event)).await
                                    }) {
                                        Ok(()) => {}
                                        Err(err) => {
                                            log::warn!("failed to send notify event: {:?}", err);
                                        }
                                    }
                                }
                                Err(err) => {
                                    log::warn!("failed to watch files: {:?}", err);
                                }
                            },
                        )
                    };

                    match watcher_res {
                        Ok(watcher) => {
                            match output
                                .send(Message::NotifyWatcher(WatcherWrapper {
                                    watcher_opt: Some(watcher),
                                }))
                                .await
                            {
                                Ok(()) => {}
                                Err(err) => {
                                    log::warn!("failed to send notify watcher: {:?}", err);
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("failed to create file watcher: {:?}", err);
                        }
                    }

                    //TODO: how to properly kill this task?
                    loop {
                        tokio::time::sleep(time::Duration::new(1, 0)).await;
                    }
                },
            ),
        ])
    }
}
