// Copyright 2023 System76 <inflist_o@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{self, cosmic::Cosmic, message, Command, Core},
    cosmic_theme, executor,
    iced::{
        event,
        futures::{self, SinkExt},
        keyboard::{Event as KeyEvent, Modifiers},
        multi_window::Application as IcedApplication,
        subscription::{self, Subscription},
        window, Event, Length, Size,
    },
    widget::{self, segmented_button},
    Application, ApplicationExt, Element,
};
use notify::Watcher;
use std::{any::TypeId, collections::HashSet, env, fs, path::PathBuf, time};

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

#[derive(Clone, Debug)]
pub enum DialogKind {
    OpenFile,
    OpenFolder,
    OpenMultipleFiles,
    OpenMultipleFolders,
    SaveFile { filename: String },
}

impl DialogKind {
    pub fn title(&self) -> String {
        match self {
            Self::OpenFile => fl!("open-file"),
            Self::OpenFolder => fl!("open-folder"),
            Self::OpenMultipleFiles => fl!("open-multiple-files"),
            Self::OpenMultipleFolders => fl!("open-multiple-folders"),
            Self::SaveFile { .. } => fl!("save-file"),
        }
    }

    pub fn multiple(&self) -> bool {
        matches!(self, Self::OpenMultipleFiles | Self::OpenMultipleFolders)
    }

    pub fn save(&self) -> bool {
        matches!(self, Self::SaveFile { .. })
    }
}

pub struct Dialog<M> {
    cosmic: Cosmic<App>,
    mapper: fn(DialogMessage) -> M,
    on_result: Box<dyn Fn(DialogResult) -> M>,
}

impl<M: Send + 'static> Dialog<M> {
    pub fn new(
        kind: DialogKind,
        path_opt: Option<PathBuf>,
        mapper: fn(DialogMessage) -> M,
        on_result: impl Fn(DialogResult) -> M + 'static,
    ) -> (Self, Command<M>) {
        //TODO: only do this once somehow?
        crate::localize::localize();

        let mut settings = window::Settings::default();
        settings.decorations = false;
        settings.exit_on_close_request = false;
        settings.transparent = true;

        //TODO: allow resize!
        settings.size = Size::new(800.0, 600.0);
        settings.resizable = false;

        #[cfg(target_os = "linux")]
        {
            settings.platform_specific.application_id = App::APP_ID.to_string();
        }

        let (window_id, window_command) = window::spawn(settings);

        let core = Core::default();
        let flags = Flags {
            kind,
            path_opt: path_opt
                .as_ref()
                .and_then(|path| match fs::canonicalize(path) {
                    Ok(ok) => Some(ok),
                    Err(err) => {
                        log::warn!("failed to canonicalize {:?}: {}", path, err);
                        None
                    }
                }),
            window_id,
        };
        let (cosmic, cosmic_command) = <Cosmic<App> as IcedApplication>::new((core, flags));

        (
            Self {
                cosmic,
                mapper,
                on_result: Box::new(on_result),
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
            let on_result_message = (self.on_result)(result);
            Command::batch([
                command,
                Command::perform(async move { app::Message::App(on_result_message) }, |x| x),
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
    kind: DialogKind,
    path_opt: Option<PathBuf>,
    window_id: window::Id,
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
enum Message {
    Cancel,
    Filename(String),
    Modifiers(Modifiers),
    NotifyEvent(notify::Event),
    NotifyWatcher(WatcherWrapper),
    Open,
    Save,
    TabMessage(tab::Message),
    TabRescan(Vec<tab::Item>),
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
    filename_id: widget::Id,
    modifiers: Modifiers,
    nav_model: segmented_button::SingleSelectModel,
    result_opt: Option<DialogResult>,
    tab: Tab,
    watcher_opt: Option<(notify::RecommendedWatcher, HashSet<PathBuf>)>,
}

impl App {
    fn rescan_tab(&self) -> Command<Message> {
        let location = self.tab.location.clone();
        let icon_sizes = self.tab.config.icon_sizes;
        Command::perform(
            async move {
                match tokio::task::spawn_blocking(move || location.scan(icon_sizes)).await {
                    Ok(items) => message::app(Message::TabRescan(items)),
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
        let title = self.flags.kind.title();
        self.set_header_title(title.clone());
        self.set_window_title(title, self.main_window_id())
    }

    fn update_watcher(&mut self) -> Command<Message> {
        if let Some((mut watcher, old_paths)) = self.watcher_opt.take() {
            let mut new_paths = HashSet::new();
            if let Location::Path(path) = &self.tab.location {
                new_paths.insert(path.clone());
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

        let location = Location::Path(match &flags.path_opt {
            Some(path) => path.to_path_buf(),
            None => match env::current_dir() {
                Ok(path) => path,
                Err(_) => home_dir(),
            },
        });

        let mut tab = Tab::new(location, TabConfig::default());
        tab.dialog = Some(flags.kind.clone());

        let mut app = App {
            core,
            flags,
            filename_id: widget::Id::unique(),
            modifiers: Modifiers::empty(),
            nav_model: nav_model.build(),
            result_opt: None,
            tab,
            watcher_opt: None,
        };

        let commands = Command::batch([app.update_title(), app.update_watcher(), app.rescan_tab()]);

        (app, commands)
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
            let message = Message::TabMessage(tab::Message::Location(location.clone()));
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
            Message::Filename(new_filename) => {
                if let DialogKind::SaveFile { filename } = &mut self.flags.kind {
                    *filename = new_filename.clone();
                }

                // Select based on filename
                if let Some(items) = &mut self.tab.items_opt {
                    for item in items.iter_mut() {
                        item.selected = item.name == new_filename;
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::NotifyEvent(event) => {
                log::debug!("{:?}", event);

                if let Location::Path(path) = &self.tab.location {
                    let mut contains_change = false;
                    for event_path in event.paths.iter() {
                        if event_path.starts_with(&path) {
                            contains_change = true;
                            break;
                        }
                    }
                    if contains_change {
                        return self.rescan_tab();
                    }
                }
            }
            Message::NotifyWatcher(mut watcher_wrapper) => match watcher_wrapper.watcher_opt.take()
            {
                Some(watcher) => {
                    self.watcher_opt = Some((watcher, HashSet::new()));
                    return self.update_watcher();
                }
                None => {
                    log::warn!("message did not contain notify watcher");
                }
            },
            Message::Open => {
                let mut paths = Vec::new();
                if let Some(ref mut items) = self.tab.items_opt {
                    for item in items.iter_mut() {
                        if item.selected {
                            paths.push(item.path.clone());
                        }
                    }
                }
                if !paths.is_empty() {
                    self.result_opt = Some(DialogResult::Open(paths));
                    return window::close(self.main_window_id());
                }
            }
            Message::Save => {
                if let DialogKind::SaveFile { filename } = &self.flags.kind {
                    if !filename.is_empty() {
                        if let Location::Path(tab_path) = &self.tab.location {
                            let path = tab_path.join(&filename);
                            if path.exists() {
                                //TODO: dialog or something?
                                log::warn!("{:?} exists", path);
                            }
                            self.result_opt = Some(DialogResult::Open(vec![path]));
                            return window::close(self.main_window_id());
                        }
                    }
                }
            }
            Message::TabMessage(tab_message) => {
                let click_i_opt = match tab_message {
                    tab::Message::Click(click_i_opt) => click_i_opt,
                    _ => None,
                };

                let tab_command = self.tab.update(tab_message, self.modifiers);

                // Update filename box when anything is selected
                if let DialogKind::SaveFile { filename } = &mut self.flags.kind {
                    if let Some(click_i) = click_i_opt {
                        if let Some(items) = &self.tab.items_opt {
                            if let Some(item) = items.get(click_i) {
                                if item.selected {
                                    *filename = item.name.clone();
                                }
                            }
                        }
                    }
                }

                match tab_command {
                    tab::Command::None => {}
                    tab::Command::Action(action) => {
                        log::warn!("Action {:?} not supported in dialog", action);
                    }
                    tab::Command::ChangeLocation(_tab_title, _tab_path) => {
                        return Command::batch([self.update_watcher(), self.rescan_tab()]);
                    }
                }
            }
            Message::TabRescan(mut items) => {
                // Select based on filename
                if let DialogKind::SaveFile { filename } = &self.flags.kind {
                    for item in items.iter_mut() {
                        item.selected = &item.name == filename;
                    }
                }

                self.tab.items_opt = Some(items);

                // Reset focus on location change
                return widget::text_input::focus(self.filename_id.clone());
            }
        }

        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = self.core().system_theme().cosmic().spacing;

        let mut tab_column = widget::column::with_capacity(2);

        tab_column = tab_column.push(
            self.tab
                .view(self.core())
                .map(move |message| Message::TabMessage(message)),
        );

        tab_column = tab_column.push(
            widget::row::with_children(vec![
                if let DialogKind::SaveFile { filename } = &self.flags.kind {
                    widget::text_input("", filename)
                        .id(self.filename_id.clone())
                        .on_input(Message::Filename)
                        .on_submit(Message::Save)
                        .into()
                } else {
                    widget::horizontal_space(Length::Fill).into()
                },
                widget::button::standard(fl!("cancel"))
                    .on_press(Message::Cancel)
                    .into(),
                if self.flags.kind.save() {
                    widget::button::standard(fl!("save")).on_press(Message::Save)
                } else {
                    widget::button::standard(fl!("open")).on_press(Message::Open)
                }
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
            self.tab.subscription().map(Message::TabMessage),
        ])
    }
}
