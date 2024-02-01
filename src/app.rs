// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{message, Command, Core},
    cosmic_config::{self, CosmicConfigEntry},
    cosmic_theme, executor,
    iced::{
        event,
        futures::{self, SinkExt},
        keyboard::{Event as KeyEvent, KeyCode, Modifiers},
        subscription::{self, Subscription},
        window, Event, Length, Point,
    },
    style,
    widget::{self, segmented_button},
    Application, ApplicationExt, Element,
};
use notify::Watcher;
use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap, HashSet},
    env, fs,
    path::PathBuf,
    process, time,
};

use crate::{
    config::{AppTheme, Config, CONFIG_VERSION},
    fl, home_dir,
    key_bind::{key_binds, KeyBind},
    menu, mouse_area,
    operation::Operation,
    tab::{self, ItemMetadata, Location, Tab},
};

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Copy,
    Cut,
    MoveToTrash,
    NewFile,
    NewFolder,
    Paste,
    Properties,
    RestoreFromTrash,
    SelectAll,
    Settings,
    TabClose,
    TabNew,
    TabNext,
    TabPrev,
    TabViewGrid,
    TabViewList,
    WindowClose,
    WindowNew,
}

impl Action {
    pub fn message(self, entity_opt: Option<segmented_button::Entity>) -> Message {
        match self {
            Action::Copy => Message::Copy(entity_opt),
            Action::Cut => Message::Cut(entity_opt),
            Action::MoveToTrash => Message::MoveToTrash(entity_opt),
            Action::NewFile => Message::NewFile(entity_opt),
            Action::NewFolder => Message::NewFolder(entity_opt),
            Action::Paste => Message::Paste(entity_opt),
            Action::Properties => Message::ToggleContextPage(ContextPage::Properties),
            Action::RestoreFromTrash => Message::RestoreFromTrash(entity_opt),
            Action::SelectAll => Message::SelectAll(entity_opt),
            Action::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Action::TabClose => Message::TabClose(entity_opt),
            Action::TabNew => Message::TabNew,
            Action::TabNext => Message::TabNext,
            Action::TabPrev => Message::TabPrev,
            Action::TabViewGrid => {
                Message::TabMessage(entity_opt, tab::Message::View(tab::View::Grid))
            }
            Action::TabViewList => {
                Message::TabMessage(entity_opt, tab::Message::View(tab::View::List))
            }
            Action::WindowClose => Message::WindowClose,
            Action::WindowNew => Message::WindowNew,
        }
    }
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    AppTheme(AppTheme),
    Config(Config),
    Copy(Option<segmented_button::Entity>),
    Cut(Option<segmented_button::Entity>),
    Key(Modifiers, KeyCode),
    Modifiers(Modifiers),
    MoveToTrash(Option<segmented_button::Entity>),
    NewFile(Option<segmented_button::Entity>),
    NewFolder(Option<segmented_button::Entity>),
    NotifyEvent(notify::Event),
    NotifyWatcher(WatcherWrapper),
    Paste(Option<segmented_button::Entity>),
    PendingComplete(u64),
    PendingError(u64, String),
    PendingProgress(u64, f32),
    RestoreFromTrash(Option<segmented_button::Entity>),
    SelectAll(Option<segmented_button::Entity>),
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    TabActivate(segmented_button::Entity),
    TabNext,
    TabPrev,
    TabClose(Option<segmented_button::Entity>),
    TabContextAction(segmented_button::Entity, Action),
    TabContextMenu(segmented_button::Entity, Option<Point>),
    TabMessage(Option<segmented_button::Entity>, tab::Message),
    TabNew,
    TabRescan(segmented_button::Entity, Vec<tab::Item>),
    ToggleContextPage(ContextPage),
    WindowClose,
    WindowNew,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextPage {
    Operations,
    Properties,
    Settings,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::Operations => fl!("operations"),
            Self::Properties => fl!("properties"),
            Self::Settings => fl!("settings"),
        }
    }
}

#[derive(Debug)]
pub struct WatcherWrapper {
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
pub struct App {
    core: Core,
    nav_model: segmented_button::SingleSelectModel,
    tab_model: segmented_button::Model<segmented_button::SingleSelect>,
    config_handler: Option<cosmic_config::Config>,
    config: Config,
    app_themes: Vec<String>,
    context_page: ContextPage,
    key_binds: HashMap<KeyBind, Action>,
    modifiers: Modifiers,
    pending_operation_id: u64,
    pending_operations: BTreeMap<u64, (Operation, f32)>,
    complete_operations: BTreeMap<u64, Operation>,
    failed_operations: BTreeMap<u64, (Operation, String)>,
    watcher_opt: Option<(notify::RecommendedWatcher, HashSet<PathBuf>)>,
}

impl App {
    fn open_tab(&mut self, location: Location) -> Command<Message> {
        let tab = Tab::new(location.clone());
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

    fn operation(&mut self, operation: Operation) {
        let id = self.pending_operation_id;
        self.pending_operation_id += 1;
        self.pending_operations.insert(id, (operation, 0.0));
        //TODO: have some button to show current status
        self.core.window.show_context = true;
        self.context_page = ContextPage::Operations;
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

    fn update_config(&mut self) -> Command<Message> {
        cosmic::app::command::set_theme(self.config.app_theme.theme())
    }

    fn save_config(&mut self) -> Command<Message> {
        match self.config_handler {
            Some(ref config_handler) => match self.config.write_entry(&config_handler) {
                Ok(()) => {}
                Err(err) => {
                    log::error!("failed to save config: {}", err);
                }
            },
            None => {}
        }
        self.update_config()
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
        self.set_window_title(window_title)
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

    fn operations(&self) -> Element<Message> {
        let mut children = Vec::new();

        //TODO: get height from theme?
        let progress_bar_height = Length::Fixed(4.0);

        if !self.pending_operations.is_empty() {
            let mut section = widget::settings::view_section(fl!("pending"));
            for (id, (op, progress)) in self.pending_operations.iter().rev() {
                section = section.add(widget::column::with_children(vec![
                    widget::text(format!("{:?}", op)).into(),
                    widget::progress_bar(0.0..=100.0, *progress)
                        .height(progress_bar_height)
                        .into(),
                ]));
            }
            children.push(section.into());
        }

        if !self.failed_operations.is_empty() {
            let mut section = widget::settings::view_section(fl!("failed"));
            for (id, (op, error)) in self.failed_operations.iter().rev() {
                section = section.add(widget::column::with_children(vec![
                    widget::text(format!("{:?}", op)).into(),
                    widget::text(error).into(),
                ]));
            }
            children.push(section.into());
        }

        if !self.complete_operations.is_empty() {
            let mut section = widget::settings::view_section(fl!("complete"));
            for (id, op) in self.complete_operations.iter().rev() {
                section = section.add(widget::text(format!("{:?}", op)));
            }
            children.push(section.into());
        }

        widget::settings::view_column(children).into()
    }

    fn properties(&self) -> Element<Message> {
        let mut children = Vec::new();
        let entity = self.tab_model.active();
        if let Some(tab) = self.tab_model.data::<Tab>(entity) {
            if let Some(ref items) = tab.items_opt {
                for item in items.iter() {
                    if item.selected {
                        children.push(item.property_view(&self.core));
                    }
                }
            }
        }
        widget::settings::view_column(children).into()
    }

    fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        widget::settings::view_column(vec![widget::settings::view_section(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    move |index| {
                        Message::AppTheme(match index {
                            1 => AppTheme::Dark,
                            2 => AppTheme::Light,
                            _ => AppTheme::System,
                        })
                    },
                )),
            )
            .into()])
        .into()
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
    const APP_ID: &'static str = "com.system76.CosmicFiles";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let app_themes = vec![fl!("match-desktop"), fl!("dark"), fl!("light")];

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
        nav_model = nav_model.insert(|b| {
            b.text(fl!("trash"))
                .icon(widget::icon::icon(tab::trash_icon_symbolic(16)))
                .data(Location::Trash)
        });

        let mut app = App {
            core,
            nav_model: nav_model.build(),
            tab_model: segmented_button::ModelBuilder::default().build(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes,
            context_page: ContextPage::Settings,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            pending_operation_id: 0,
            pending_operations: BTreeMap::new(),
            complete_operations: BTreeMap::new(),
            failed_operations: BTreeMap::new(),
            watcher_opt: None,
        };

        let mut commands = Vec::new();

        for arg in env::args().skip(1) {
            let location = match fs::canonicalize(&arg) {
                Ok(absolute) => Location::Path(absolute),
                Err(err) => {
                    log::warn!("failed to canonicalize {:?}: {}", arg, err);
                    continue;
                }
            };
            commands.push(app.open_tab(location));
        }

        if app.tab_model.iter().next().is_none() {
            commands.push(app.open_tab(Location::Path(home_dir())));
        }

        (app, Command::batch(commands))
    }

    // The default nav_bar widget needs to have its width reduced for cosmic-files
    fn nav_bar(&self) -> Option<Element<message::Message<Self::Message>>> {
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

    fn on_nav_select(&mut self, entity: segmented_button::Entity) -> Command<Self::Message> {
        let location_opt = self.nav_model.data::<Location>(entity).clone();

        if let Some(location) = location_opt {
            let message = Message::TabMessage(None, tab::Message::Location(location.clone()));
            return self.update(message);
        }

        Command::none()
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        // Helper for updating config values efficiently
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                log::warn!(
                                    "failed to save config {:?}: {}",
                                    stringify!($name),
                                    err
                                );
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        log::warn!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name)
                        );
                    }
                }
            };
        }

        match message {
            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            Message::Config(config) => {
                if config != self.config {
                    log::info!("update config");
                    //TODO: update syntax theme by clearing tabs, only if needed
                    self.config = config;
                    return self.update_config();
                }
            }
            Message::Copy(entity_opt) => {
                log::warn!("TODO: COPY");
            }
            Message::Cut(entity_opt) => {
                log::warn!("TODO: CUT");
            }
            Message::Key(modifiers, key_code) => {
                let entity = self.tab_model.active();
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, key_code) {
                        return self.update(action.message(Some(entity)));
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::MoveToTrash(entity_opt) => {
                let mut paths = Vec::new();
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(ref mut items) = tab.items_opt {
                        for item in items.iter_mut() {
                            if item.selected {
                                paths.push(item.path.clone());
                            }
                        }
                    }
                }
                if !paths.is_empty() {
                    self.operation(Operation::Delete { paths });
                }
            }
            Message::NewFile(entity_opt) => {
                log::warn!("TODO: NEW FILE");
            }
            Message::NewFolder(entity_opt) => {
                log::warn!("TODO: NEW FOLDER");
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
            Message::Paste(entity_opt) => {
                log::warn!("TODO: PASTE");
            }
            Message::PendingComplete(id) => {
                if let Some((op, _)) = self.pending_operations.remove(&id) {
                    self.complete_operations.insert(id, op);
                }
            }
            Message::PendingError(id, err) => {
                if let Some((op, _)) = self.pending_operations.remove(&id) {
                    self.failed_operations.insert(id, (op, err));
                }
            }
            Message::PendingProgress(id, new_progress) => {
                if let Some((_, progress)) = self.pending_operations.get_mut(&id) {
                    *progress = new_progress;
                }
            }
            Message::RestoreFromTrash(entity_opt) => {
                let mut paths = Vec::new();
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(ref mut items) = tab.items_opt {
                        for item in items.iter_mut() {
                            if item.selected {
                                match &item.metadata {
                                    ItemMetadata::Trash { entry, .. } => {
                                        paths.push(entry.clone());
                                    }
                                    _ => {
                                        //TODO: error on trying to restore non-trash file?
                                    }
                                }
                            }
                        }
                    }
                }
                if !paths.is_empty() {
                    self.operation(Operation::Restore { paths });
                }
            }
            Message::SelectAll(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(ref mut items) = tab.items_opt {
                        for item in items.iter_mut() {
                            if item.hidden {
                                //TODO: option to show hidden files
                                continue;
                            }
                            item.selected = true;
                            item.click_time = None;
                        }
                    }
                }
            }
            Message::SystemThemeModeChange(_theme_mode) => {
                return self.update_config();
            }
            Message::TabActivate(entity) => {
                self.tab_model.activate(entity);
                return self.update_title();
            }
            Message::TabNext => {
                let len = self.tab_model.iter().count();
                let pos = self
                    .tab_model
                    .position(self.tab_model.active())
                    // Wraparound to 0 if i + 1 > num of tabs
                    .map(|i| (i as usize + 1) % len)
                    .expect("should always be at least one tab open");

                let entity = self.tab_model.iter().nth(pos);
                if let Some(entity) = entity {
                    return self.update(Message::TabActivate(entity));
                }
            }
            Message::TabPrev => {
                let pos = self
                    .tab_model
                    .position(self.tab_model.active())
                    .and_then(|i| (i as usize).checked_sub(1))
                    // Subtraction underflow => last tab; i.e. it wraps around
                    .unwrap_or_else(|| {
                        self.tab_model
                            .iter()
                            .count()
                            .checked_sub(1)
                            .unwrap_or_default()
                    });

                let entity = self.tab_model.iter().nth(pos);
                if let Some(entity) = entity {
                    return self.update(Message::TabActivate(entity));
                }
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
                    return window::close(window::Id::MAIN);
                }

                return Command::batch([self.update_title(), self.update_watcher()]);
            }
            Message::TabContextAction(entity, action) => {
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        // Close context menu
                        {
                            tab.context_menu = None;
                        }
                        // Run action's message
                        return self.update(action.message(Some(entity)));
                    }
                    _ => {}
                }
            }
            Message::TabContextMenu(entity, position_opt) => {
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        // Update context menu position
                        tab.context_menu = position_opt;
                    }
                    _ => {}
                }
                // Disable side context page
                self.core.window.show_context = false;
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
            Message::TabNew => {
                let active = self.tab_model.active();
                let location = match self.tab_model.data::<Tab>(active) {
                    Some(tab) => tab.location.clone(),
                    None => Location::Path(home_dir()),
                };
                return self.open_tab(location);
            }
            Message::TabRescan(entity, items) => match self.tab_model.data_mut::<Tab>(entity) {
                Some(tab) => {
                    tab.items_opt = Some(items);
                }
                _ => (),
            },
            //TODO: TABRELOAD
            Message::ToggleContextPage(context_page) => {
                //TODO: ensure context menus are closed
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                self.set_context_title(context_page.title());
            }
            Message::WindowClose => {
                return window::close(window::Id::MAIN);
            }
            Message::WindowNew => match env::current_exe() {
                Ok(exe) => match process::Command::new(&exe).spawn() {
                    Ok(_child) => {}
                    Err(err) => {
                        log::error!("failed to execute {:?}: {}", exe, err);
                    }
                },
                Err(err) => {
                    log::error!("failed to get current executable path: {}", err);
                }
            },
        }

        Command::none()
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::Operations => self.operations(),
            ContextPage::Properties => self.properties(),
            ContextPage::Settings => self.settings(),
        })
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(&self.key_binds).into()]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        vec![]
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
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
                let mut mouse_area = mouse_area::MouseArea::new(
                    tab.view(self.core())
                        .map(move |message| Message::TabMessage(Some(entity), message)),
                )
                .on_press(move |_point_opt| {
                    Message::TabMessage(Some(entity), tab::Message::Click(None))
                });
                if tab.context_menu.is_some() {
                    mouse_area = mouse_area
                        .on_right_press(move |_point_opt| Message::TabContextMenu(entity, None));
                } else {
                    mouse_area = mouse_area.on_right_press(move |point_opt| {
                        Message::TabContextMenu(entity, point_opt)
                    });
                }
                let mut popover = widget::popover(mouse_area, menu::context_menu(entity, &tab));
                match tab.context_menu {
                    Some(point) => {
                        let rounded = Point::new(point.x.round(), point.y.round());
                        popover = popover.position(rounded);
                    }
                    None => {
                        popover = popover.show_popup(false);
                    }
                }
                tab_column = tab_column.push(popover);
            }
            None => {
                //TODO
            }
        }

        let content: Element<_> = tab_column.into();

        // Uncomment to debug layout:
        //content.explain(cosmic::iced::Color::WHITE)
        content
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ConfigSubscription;
        struct ThemeSubscription;
        struct WatcherSubscription;

        let mut subscriptions = vec![
            event::listen_with(|event, _status| match event {
                Event::Keyboard(KeyEvent::KeyPressed {
                    key_code,
                    modifiers,
                }) => Some(Message::Key(modifiers, key_code)),
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            cosmic_config::config_subscription(
                TypeId::of::<ConfigSubscription>(),
                Self::APP_ID.into(),
                CONFIG_VERSION,
            )
            .map(|update| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::Config(update.config)
            }),
            cosmic_config::config_subscription::<_, cosmic_theme::ThemeMode>(
                TypeId::of::<ThemeSubscription>(),
                cosmic_theme::THEME_MODE_ID.into(),
                cosmic_theme::ThemeMode::version(),
            )
            .map(|update| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading theme mode {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::SystemThemeModeChange(update.config)
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
        ];

        for (id, (pending_operation, _)) in self.pending_operations.iter() {
            //TODO: use recipe?
            let id = *id;
            let pending_operation = pending_operation.clone();
            subscriptions.push(subscription::channel(
                id,
                16,
                move |mut msg_tx| async move {
                    match pending_operation.perform(id, &mut msg_tx).await {
                        Ok(()) => {
                            msg_tx.send(Message::PendingComplete(id)).await;
                        }
                        Err(err) => {
                            msg_tx
                                .send(Message::PendingError(id, err.to_string()))
                                .await;
                        }
                    }

                    loop {
                        tokio::time::sleep(time::Duration::new(1, 0)).await;
                    }
                },
            ));
        }

        Subscription::batch(subscriptions)
    }
}

// Utilities to build a temporary file hierarchy for tests.
//
// Ideally, tests would use the cap-std crate which limits path traversal.
#[cfg(test)]
pub(crate) mod test_utils {
    use std::{
        cmp::Ordering,
        fs::File,
        io::{self, Write},
        iter,
        path::Path,
    };

    use log::{debug, trace};
    use tempfile::{tempdir, TempDir};

    use crate::tab::Item;

    use super::*;

    // Default number of files, directories, and nested directories for test file system
    pub const NUM_FILES: usize = 2;
    pub const NUM_DIRS: usize = 2;
    pub const NUM_NESTED: usize = 1;
    pub const NAME_LEN: usize = 5;

    /// Add `n` temporary files in `dir`
    ///
    /// Each file is assigned a numeric name from [0, n).
    pub fn file_flat_hier<D: AsRef<Path>>(dir: D, n: usize) -> io::Result<Vec<File>> {
        let dir = dir.as_ref();
        (0..n)
            .map(|i| -> io::Result<File> {
                let name = i.to_string();
                let path = dir.join(&name);

                let mut file = File::create(path)?;
                file.write_all(name.as_bytes())?;

                Ok(file)
            })
            .collect()
    }

    // Random alphanumeric String of length `len`
    fn rand_string(len: usize) -> String {
        (0..len).map(|_| fastrand::alphanumeric()).collect()
    }

    /// Create a small, temporary file hierarchy.
    pub fn simple_fs(
        files: usize,
        dirs: usize,
        nested: usize,
        name_len: usize,
    ) -> io::Result<TempDir> {
        // Files created inside of a TempDir are deleted with the directory
        // TempDir won't leak resources as long as the destructor runs
        let root = tempdir()?;
        debug!("Root temp directory: {}", root.as_ref().display());

        // All paths for directories and nested directories
        let paths = (0..dirs).flat_map(|_| {
            let root = root.as_ref();
            let current = rand_string(name_len);

            iter::once(root.join(&current)).chain(
                (0..nested).map(move |_| root.join(format!("{current}/{}", rand_string(name_len)))),
            )
        });

        // Create directories from `paths` and add a few files
        for path in paths {
            fs::create_dir_all(&path)?;
            file_flat_hier(&path, files)?;

            for entry in path.read_dir()? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    trace!("Created file: {}", entry.path().display());
                }
            }
        }

        Ok(root)
    }

    /// Empty file hierarchy
    pub fn empty_fs() -> io::Result<TempDir> {
        tempdir()
    }

    /// Sort files.
    ///
    /// Directories are placed before files.
    /// Files are lexically sorted.
    /// This is more or less copied right from the [Tab] code
    pub fn sort_files(a: &Path, b: &Path) -> Ordering {
        match (a.is_dir(), b.is_dir()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => lexical_sort::natural_lexical_cmp(
                a.file_name()
                    .expect("temp entries should have names")
                    .to_str()
                    .expect("temp entries should be valid UTF-8"),
                b.file_name()
                    .expect("temp entries should have names")
                    .to_str()
                    .expect("temp entries should be valid UTF-8"),
            ),
        }
    }

    /// Equality for [Path] and [Item].
    pub fn eq_path_item(path: &Path, item: &Item) -> bool {
        let name = path
            .file_name()
            .expect("temp entries should have names")
            .to_str()
            .expect("temp entries should be valid UTF-8");
        let metadata = path.is_dir();

        name == item.name && metadata == item.metadata.is_dir() && path == item.path
    }
}
