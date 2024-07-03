// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "winit")]
use cosmic::iced::multi_window::Application as IcedApplication;
#[cfg(feature = "wayland")]
use cosmic::iced::Application as IcedApplication;
use cosmic::{
    app::{self, cosmic::Cosmic, message, Command, Core},
    cosmic_theme, executor,
    iced::{
        event,
        futures::{self, SinkExt},
        keyboard::{Event as KeyEvent, Modifiers},
        subscription::{self, Subscription},
        widget::scrollable,
        window, Alignment, Event, Length, Size,
    },
    theme,
    widget::{self, menu::KeyBind, segmented_button},
    Application, ApplicationExt, Element,
};
use notify_debouncer_full::{
    new_debouncer,
    notify::{self, RecommendedWatcher, Watcher},
    DebouncedEvent, Debouncer, FileIdMap,
};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    env, fmt, fs,
    path::PathBuf,
    str::FromStr,
    time,
};

use crate::{
    app::Action,
    config::TabConfig,
    fl, home_dir,
    tab::{self, ItemMetadata, Location, Tab},
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

    pub fn accept_label(&self) -> String {
        match self {
            Self::SaveFile { .. } => fl!("save"),
            _ => fl!("open"),
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Self::OpenFolder | Self::OpenMultipleFolders)
    }

    pub fn multiple(&self) -> bool {
        matches!(self, Self::OpenMultipleFiles | Self::OpenMultipleFolders)
    }

    pub fn save(&self) -> bool {
        matches!(self, Self::SaveFile { .. })
    }
}

#[derive(Clone, Debug)]
pub struct DialogChoiceOption {
    pub id: String,
    pub label: String,
}

impl AsRef<str> for DialogChoiceOption {
    fn as_ref(&self) -> &str {
        &self.label
    }
}

#[derive(Clone, Debug)]
pub enum DialogChoice {
    CheckBox {
        id: String,
        label: String,
        value: bool,
    },
    ComboBox {
        id: String,
        label: String,
        options: Vec<DialogChoiceOption>,
        selected: Option<usize>,
    },
}

#[derive(Clone, Debug)]
pub enum DialogFilterPattern {
    Glob(String),
    Mime(String),
}

#[derive(Clone, Debug)]
pub struct DialogFilter {
    pub label: String,
    pub patterns: Vec<DialogFilterPattern>,
}

impl AsRef<str> for DialogFilter {
    fn as_ref(&self) -> &str {
        &self.label
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
        settings.size = Size::new(1024.0, 640.0);
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

    pub fn set_title(&mut self, title: impl Into<String>) -> Command<M> {
        let mapper = self.mapper;
        self.cosmic.app.title = title.into();
        self.cosmic
            .app
            .update_title()
            .map(DialogMessage)
            .map(move |message| app::Message::App(mapper(message)))
    }

    pub fn set_accept_label(&mut self, accept_label: impl Into<String>) {
        self.cosmic.app.accept_label = accept_label.into();
    }

    pub fn choices(&self) -> &[DialogChoice] {
        &self.cosmic.app.choices
    }

    pub fn set_choices(&mut self, choices: impl Into<Vec<DialogChoice>>) {
        self.cosmic.app.choices = choices.into();
    }

    pub fn filters(&self) -> (&[DialogFilter], Option<usize>) {
        (&self.cosmic.app.filters, self.cosmic.app.filter_selected)
    }

    pub fn set_filters(
        &mut self,
        filters: impl Into<Vec<DialogFilter>>,
        filter_selected: Option<usize>,
    ) -> Command<M> {
        let mapper = self.mapper;
        self.cosmic.app.filters = filters.into();
        self.cosmic.app.filter_selected = filter_selected;
        self.cosmic
            .app
            .rescan_tab()
            .map(DialogMessage)
            .map(move |message| app::Message::App(mapper(message)))
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

    pub fn window_id(&self) -> window::Id {
        self.cosmic.app.main_window_id()
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
    Choice(usize, usize),
    Filename(String),
    Filter(usize),
    Modifiers(Modifiers),
    NotifyEvents(Vec<DebouncedEvent>),
    NotifyWatcher(WatcherWrapper),
    Open,
    Save(bool),
    TabMessage(tab::Message),
    TabRescan(Vec<tab::Item>),
}

struct WatcherWrapper {
    watcher_opt: Option<Debouncer<RecommendedWatcher, FileIdMap>>,
}

impl Clone for WatcherWrapper {
    fn clone(&self) -> Self {
        Self { watcher_opt: None }
    }
}

impl fmt::Debug for WatcherWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WatcherWrapper").finish()
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
    title: String,
    accept_label: String,
    choices: Vec<DialogChoice>,
    filters: Vec<DialogFilter>,
    filter_selected: Option<usize>,
    filename_id: widget::Id,
    modifiers: Modifiers,
    nav_model: segmented_button::SingleSelectModel,
    result_opt: Option<DialogResult>,
    replace_dialog: bool,
    tab: Tab,
    key_binds: HashMap<KeyBind, Action>,
    watcher_opt: Option<(Debouncer<RecommendedWatcher, FileIdMap>, HashSet<PathBuf>)>,
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
        self.set_header_title(self.title.clone());
        self.set_window_title(self.title.clone(), self.main_window_id())
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
                    match watcher.watcher().unwatch(path) {
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
                    match watcher
                        .watcher()
                        .watch(path, notify::RecursiveMode::NonRecursive)
                    {
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
        //TODO: make set_nav_bar_toggle_condensed pub
        core.nav_bar_toggle_condensed();

        let title = flags.kind.title();
        let accept_label = flags.kind.accept_label();

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
        tab.config.view = tab::View::List;

        let mut app = App {
            core,
            flags,
            title,
            accept_label,
            choices: Vec::new(),
            filters: Vec::new(),
            filter_selected: None,
            filename_id: widget::Id::unique(),
            modifiers: Modifiers::empty(),
            nav_model: nav_model.build(),
            result_opt: None,
            replace_dialog: false,
            tab,
            key_binds: HashMap::new(),
            watcher_opt: None,
        };

        let commands = Command::batch([app.update_title(), app.update_watcher(), app.rescan_tab()]);

        (app, commands)
    }

    fn main_window_id(&self) -> window::Id {
        self.flags.window_id
    }

    fn dialog(&self) -> Option<Element<Message>> {
        if self.replace_dialog {
            if let DialogKind::SaveFile { filename } = &self.flags.kind {
                return Some(
                    widget::dialog(fl!("replace-title", filename = filename.as_str()))
                        .icon(widget::icon::from_name("dialog-question").size(64))
                        .body(fl!("replace-warning"))
                        .primary_action(
                            widget::button::suggested(fl!("replace")).on_press(Message::Save(true)),
                        )
                        .secondary_action(
                            widget::button::standard(fl!("cancel")).on_press(Message::Cancel),
                        )
                        .into(),
                );
            }
        }
        None
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_app_exit(&mut self) -> Option<Message> {
        self.result_opt = Some(DialogResult::Cancel);
        None
    }

    fn on_nav_select(&mut self, entity: segmented_button::Entity) -> Command<Message> {
        let location_opt = self.nav_model.data::<Location>(entity).clone();

        if let Some(location) = location_opt {
            let message = Message::TabMessage(tab::Message::Location(location.clone()));
            return self.update(message);
        }

        Command::none()
    }

    fn on_escape(&mut self) -> Command<Message> {
        self.update(Message::Cancel)
    }

    /// Handle application events here.
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Cancel => {
                if self.replace_dialog {
                    self.replace_dialog = false;
                } else {
                    self.result_opt = Some(DialogResult::Cancel);
                    return window::close(self.main_window_id());
                }
            }
            Message::Choice(choice_i, option_i) => {
                if let Some(choice) = self.choices.get_mut(choice_i) {
                    match choice {
                        DialogChoice::CheckBox { value, .. } => *value = option_i > 0,
                        DialogChoice::ComboBox {
                            options, selected, ..
                        } => {
                            if option_i < options.len() {
                                *selected = Some(option_i);
                            } else {
                                *selected = None;
                            }
                        }
                    }
                }
            }
            Message::Filename(new_filename) => {
                // Select based on filename
                self.tab.select_name(&new_filename);

                if let DialogKind::SaveFile { filename } = &mut self.flags.kind {
                    *filename = new_filename;
                }
            }
            Message::Filter(filter_i) => {
                if filter_i < self.filters.len() {
                    self.filter_selected = Some(filter_i);
                } else {
                    self.filter_selected = None;
                }
                return self.rescan_tab();
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::NotifyEvents(events) => {
                log::debug!("{:?}", events);

                if let Location::Path(path) = &self.tab.location {
                    let mut contains_change = false;
                    for event in events.iter() {
                        for event_path in event.paths.iter() {
                            if event_path.starts_with(&path) {
                                match event.kind {
                                    notify::EventKind::Modify(
                                        notify::event::ModifyKind::Metadata(_),
                                    )
                                    | notify::EventKind::Modify(notify::event::ModifyKind::Data(
                                        _,
                                    )) => {
                                        // If metadata or data changed, find the matching item and reload it
                                        //TODO: this could be further optimized by looking at what exactly changed
                                        if let Some(items) = &mut self.tab.items_opt {
                                            for item in items.iter_mut() {
                                                if item.path_opt.as_ref() == Some(event_path) {
                                                    //TODO: reload more, like mime types?
                                                    match fs::metadata(&event_path) {
                                                        Ok(new_metadata) => {
                                                            match &mut item.metadata {
                                                                ItemMetadata::Path {
                                                                    metadata,
                                                                    ..
                                                                } => *metadata = new_metadata,
                                                                _ => {}
                                                            }
                                                        }
                                                        Err(err) => {
                                                            log::warn!("failed to reload metadata for {:?}: {}", path, err);
                                                        }
                                                    }
                                                    //TODO item.thumbnail_opt =
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        // Any other events reload the whole tab
                                        contains_change = true;
                                        break;
                                    }
                                }
                            }
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
                if let Some(items) = self.tab.items_opt() {
                    for item in items.iter() {
                        if item.selected {
                            if let Some(path) = &item.path_opt {
                                paths.push(path.clone());
                            }
                        }
                    }
                }

                // Ensure selection is allowed
                //TODO: improve tab logic so this doesn't block the open button so often
                for path in paths.iter() {
                    let path_is_dir = path.is_dir();
                    if path_is_dir != self.flags.kind.is_dir() {
                        if path_is_dir && paths.len() == 1 {
                            // If the only selected item is a directory and we are selecting files, cd to it
                            let message = Message::TabMessage(tab::Message::Location(
                                Location::Path(path.clone()),
                            ));
                            return self.update(message);
                        } else {
                            // Otherwise, this is not a legal selection
                            return Command::none();
                        }
                    }
                }

                // If there are proper matching items, return them
                if !paths.is_empty() {
                    self.result_opt = Some(DialogResult::Open(paths));
                    return window::close(self.main_window_id());
                }

                // If we are in directory mode, return the current directory
                if self.flags.kind.is_dir() {
                    match &self.tab.location {
                        Location::Path(tab_path) => {
                            self.result_opt = Some(DialogResult::Open(vec![tab_path.clone()]));
                            return window::close(self.main_window_id());
                        }
                        _ => {}
                    }
                }
            }
            Message::Save(replace) => {
                if let DialogKind::SaveFile { filename } = &self.flags.kind {
                    if !filename.is_empty() {
                        if let Location::Path(tab_path) = &self.tab.location {
                            let path = tab_path.join(&filename);
                            if path.is_dir() {
                                // cd to directory
                                let message = Message::TabMessage(tab::Message::Location(
                                    Location::Path(path.clone()),
                                ));
                                return self.update(message);
                            } else if !replace && path.exists() {
                                self.replace_dialog = true;
                            } else {
                                self.result_opt = Some(DialogResult::Open(vec![path]));
                                return window::close(self.main_window_id());
                            }
                        }
                    }
                }
            }
            Message::TabMessage(tab_message) => {
                let click_i_opt = match tab_message {
                    tab::Message::Click(click_i_opt) => click_i_opt,
                    _ => None,
                };

                let tab_commands = self.tab.update(tab_message, self.modifiers);

                // Update filename box when anything is selected
                if let DialogKind::SaveFile { filename } = &mut self.flags.kind {
                    if let Some(click_i) = click_i_opt {
                        if let Some(items) = self.tab.items_opt() {
                            if let Some(item) = items.get(click_i) {
                                if item.selected && !item.metadata.is_dir() {
                                    *filename = item.name.clone();
                                }
                            }
                        }
                    }
                }

                let mut commands = Vec::new();
                for tab_command in tab_commands {
                    match tab_command {
                        tab::Command::Action(action) => {
                            log::warn!("Action {:?} not supported in dialog", action);
                        }
                        tab::Command::ChangeLocation(_tab_title, _tab_path) => {
                            commands
                                .push(Command::batch([self.update_watcher(), self.rescan_tab()]));
                        }
                        tab::Command::DropFiles(_, _) => {
                            log::warn!("DropFiles not supported in dialog");
                        }
                        tab::Command::EmptyTrash => {
                            log::warn!("EmptyTrash not supported in dialog");
                        }
                        tab::Command::FocusButton(id) => {
                            commands.push(widget::button::focus(id));
                        }
                        tab::Command::FocusTextInput(id) => {
                            commands.push(widget::text_input::focus(id));
                        }
                        tab::Command::OpenFile(_item_path) => {
                            if self.flags.kind.save() {
                                commands.push(self.update(Message::Save(false)));
                            } else {
                                commands.push(self.update(Message::Open));
                            }
                        }
                        tab::Command::OpenInNewTab(_path) => {
                            log::warn!("OpenInNewTab not supported in dialog");
                        }
                        tab::Command::OpenInNewWindow(_path) => {
                            log::warn!("OpenInNewWindow not supported in dialog");
                        }
                        tab::Command::LocationProperties(_path) => {
                            log::warn!("LocationProperties not supported in dialog");
                        }
                        tab::Command::Scroll(id, offset) => {
                            commands.push(scrollable::scroll_to(id, offset));
                        }
                        tab::Command::Timeout(_, _) => {
                            log::warn!("Timeout not supported in dialog");
                        }
                        tab::Command::MoveToTrash(_) => {
                            log::warn!("MoveToTrash not supported in dialog");
                        }
                    }
                }
                return Command::batch(commands);
            }
            Message::TabRescan(mut items) => {
                // Filter
                if let Some(filter_i) = self.filter_selected {
                    if let Some(filter) = self.filters.get(filter_i) {
                        // Parse filters
                        let mut parsed_globs = Vec::new();
                        let mut parsed_mimes = Vec::new();
                        for pattern in filter.patterns.iter() {
                            match pattern {
                                DialogFilterPattern::Glob(value) => {
                                    match glob::Pattern::new(value) {
                                        Ok(glob) => parsed_globs.push(glob),
                                        Err(err) => {
                                            log::warn!("failed to parse glob {:?}: {}", value, err);
                                        }
                                    }
                                }
                                DialogFilterPattern::Mime(value) => {
                                    match mime_guess::Mime::from_str(value) {
                                        Ok(mime) => parsed_mimes.push(mime),
                                        Err(err) => {
                                            log::warn!("failed to parse mime {:?}: {}", value, err);
                                        }
                                    }
                                }
                            }
                        }

                        items.retain(|item| {
                            if item.metadata.is_dir() {
                                // Directories are always shown
                                return true;
                            }

                            // Check for mime type match (first because it is faster)
                            for mime in parsed_mimes.iter() {
                                if mime == &item.mime {
                                    return true;
                                }
                            }

                            // Check for glob match (last because it is slower)
                            for glob in parsed_globs.iter() {
                                if glob.matches(&item.name) {
                                    return true;
                                }
                            }

                            // No filters matched
                            false
                        });
                    }
                }

                // Select based on filename
                if let DialogKind::SaveFile { filename } = &self.flags.kind {
                    for item in items.iter_mut() {
                        item.selected = &item.name == filename;
                    }
                }

                self.tab.set_items(items);

                // Reset focus on location change
                return widget::text_input::focus(self.filename_id.clone());
            }
        }

        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let mut tab_column = widget::column::with_capacity(2);
        tab_column = tab_column.push(
            //TODO: key binds for dialog
            self.tab
                .view(&self.key_binds)
                .map(move |message| Message::TabMessage(message)),
        );

        let mut row = widget::row::with_capacity(
            if !self.filters.is_empty() { 1 } else { 0 } + self.choices.len() * 2 + 3,
        )
        .align_items(Alignment::Center)
        .padding(space_xxs)
        .spacing(space_xxs);
        if !self.filters.is_empty() {
            row = row.push(widget::dropdown(
                &self.filters,
                self.filter_selected,
                Message::Filter,
            ));
        }
        if let DialogKind::SaveFile { filename } = &self.flags.kind {
            row = row.push(
                widget::text_input("", filename)
                    .id(self.filename_id.clone())
                    .on_input(Message::Filename)
                    .on_submit(Message::Save(false)),
            );
        } else {
            row = row.push(widget::horizontal_space(Length::Fill));
        }
        for (choice_i, choice) in self.choices.iter().enumerate() {
            match choice {
                DialogChoice::CheckBox { label, value, .. } => {
                    row = row.push(widget::text::body(label));
                    row = row.push(widget::checkbox("", *value, move |checked| {
                        Message::Choice(choice_i, if checked { 1 } else { 0 })
                    }));
                }
                DialogChoice::ComboBox {
                    label,
                    options,
                    selected,
                    ..
                } => {
                    row = row.push(widget::text::body(label));
                    row = row.push(widget::dropdown(options, *selected, move |option_i| {
                        Message::Choice(choice_i, option_i)
                    }));
                }
            }
        }
        row = row.push(widget::button::standard(fl!("cancel")).on_press(Message::Cancel));
        row = row.push(if self.flags.kind.save() {
            widget::button::suggested(&self.accept_label).on_press(Message::Save(false))
        } else {
            widget::button::suggested(&self.accept_label).on_press(Message::Open)
        });

        tab_column = tab_column.push(row);

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
                        new_debouncer(
                            time::Duration::from_millis(250),
                            Some(time::Duration::from_millis(250)),
                            move |events_res: notify_debouncer_full::DebounceEventResult| {
                                match events_res {
                                    Ok(mut events) => {
                                        events.retain(|event| {
                                            match &event.kind {
                                                notify::EventKind::Access(_) => {
                                                    // Data not mutated
                                                    false
                                                }
                                                notify::EventKind::Modify(
                                                    notify::event::ModifyKind::Metadata(e),
                                                ) if (*e != notify::event::MetadataKind::Any
                                                    && *e
                                                        != notify::event::MetadataKind::WriteTime) =>
                                                {
                                                    // Data not mutated nor modify time changed
                                                    false
                                                }
                                                _ => true
                                            }
                                        });

                                        if !events.is_empty() {
                                            match futures::executor::block_on(async {
                                                output.send(Message::NotifyEvents(events)).await
                                            }) {
                                                Ok(()) => {}
                                                Err(err) => {
                                                    log::warn!(
                                                        "failed to send notify events: {:?}",
                                                        err
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        log::warn!("failed to watch files: {:?}", err);
                                    }
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

                    std::future::pending().await
                },
            ),
            self.tab.subscription().map(Message::TabMessage),
        ])
    }
}
