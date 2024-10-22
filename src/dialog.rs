// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{self, cosmic::Cosmic, message, Core, Task},
    cosmic_config, cosmic_theme, executor,
    iced::{
        event,
        futures::{self, SinkExt},
        keyboard::{Event as KeyEvent, Key, Modifiers},
        stream, window, Alignment, Event, Length, Size, Subscription,
    },
    theme,
    widget::{
        self,
        menu::{Action as MenuAction, KeyBind},
        segmented_button,
    },
    Application, ApplicationExt, Element,
};
use notify_debouncer_full::{
    new_debouncer,
    notify::{self, RecommendedWatcher, Watcher},
    DebouncedEvent, Debouncer, FileIdMap,
};
use recently_used_xbel::update_recently_used;
use std::{
    any::TypeId,
    collections::{HashMap, HashSet, VecDeque},
    env, fmt, fs,
    num::NonZeroU16,
    path::PathBuf,
    str::FromStr,
    time::{self, Instant},
};

use crate::{
    app::{Action, ContextPage, Message as AppMessage, PreviewItem, PreviewKind},
    config::{Config, Favorite, IconSizes, TabConfig},
    fl, home_dir,
    key_bind::key_binds,
    localize::LANGUAGE_SORTER,
    menu,
    mounter::{MounterItem, MounterItems, MounterKey, MounterMessage, MOUNTERS},
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
    ) -> (Self, Task<M>) {
        //TODO: only do this once somehow?
        crate::localize::localize();

        let (config_handler, config) = Config::load();

        let mut settings = window::Settings::default();
        settings.decorations = false;
        settings.exit_on_close_request = false;
        settings.min_size = Some(Size::new(360.0, 180.0));
        settings.resizable = true;
        settings.size = Size::new(1024.0, 640.0);
        settings.transparent = true;

        #[cfg(target_os = "linux")]
        {
            settings.platform_specific.application_id = App::APP_ID.to_string();
        }

        let (window_id, window_command) = window::open(settings.clone());

        let mut core = Core::default();
        core.set_main_window_id(Some(window_id));
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
            config_handler,
            config,
        };

        let (cosmic, cosmic_command) = Cosmic::<App>::init((core, flags));
        (
            Self {
                cosmic,
                mapper,
                on_result: Box::new(on_result),
            },
            Task::batch([
                window_command.map(|_id| message::none()),
                cosmic_command
                    .map(DialogMessage)
                    .map(move |message| app::Message::App(mapper(message))),
            ]),
        )
    }

    pub fn set_title(&mut self, title: impl Into<String>) -> Task<M> {
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
    ) -> Task<M> {
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

    pub fn update(&mut self, message: DialogMessage) -> Task<M> {
        let mapper = self.mapper;
        let command = self
            .cosmic
            .update(message.0)
            .map(DialogMessage)
            .map(move |message| app::Message::App(mapper(message)));
        if let Some(result) = self.cosmic.app.result_opt.take() {
            let on_result_message = (self.on_result)(result);
            Task::batch([
                command,
                Task::perform(async move { app::Message::App(on_result_message) }, |x| x),
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
        self.cosmic.app.flags.window_id
    }
}

#[derive(Clone, Debug)]
enum DialogPage {
    NewFolder { parent: PathBuf, name: String },
    Replace { filename: String },
}

#[derive(Clone, Debug)]
struct Flags {
    kind: DialogKind,
    path_opt: Option<PathBuf>,
    window_id: window::Id,
    config_handler: Option<cosmic_config::Config>,
    config: Config,
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
enum Message {
    None,
    Cancel,
    Choice(usize, usize),
    Config(Config),
    DialogCancel,
    DialogComplete,
    DialogUpdate(DialogPage),
    Filename(String),
    Filter(usize),
    Key(Modifiers, Key),
    Modifiers(Modifiers),
    MounterItems(MounterKey, MounterItems),
    NewFolder,
    NotifyEvents(Vec<DebouncedEvent>),
    NotifyWatcher(WatcherWrapper),
    Open,
    Preview,
    Save(bool),
    SearchActivate,
    SearchClear,
    SearchInput(String),
    TabMessage(tab::Message),
    TabRescan(Location, Option<tab::Item>, Vec<tab::Item>),
    TabView(tab::View),
    ToggleFoldersFirst,
    ZoomDefault,
    ZoomIn,
    ZoomOut,
}

impl From<AppMessage> for Message {
    fn from(app_message: AppMessage) -> Message {
        match app_message {
            AppMessage::None => Message::None,
            AppMessage::Preview(_entity_opt) => Message::Preview,
            AppMessage::SearchActivate => Message::SearchActivate,
            AppMessage::TabMessage(_entity_opt, tab_message) => Message::TabMessage(tab_message),
            AppMessage::TabView(_entity_opt, view) => Message::TabView(view),
            AppMessage::ToggleFoldersFirst => Message::ToggleFoldersFirst,
            AppMessage::ZoomDefault(_entity_opt) => Message::ZoomDefault,
            AppMessage::ZoomIn(_entity_opt) => Message::ZoomIn,
            AppMessage::ZoomOut(_entity_opt) => Message::ZoomOut,
            unsupported => {
                log::warn!("{unsupported:?} not supported in dialog mode");
                Message::None
            }
        }
    }
}

pub struct MounterData(MounterKey, MounterItem);

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
    context_page: ContextPage,
    dialog_pages: VecDeque<DialogPage>,
    dialog_text_input: widget::Id,
    filters: Vec<DialogFilter>,
    filter_selected: Option<usize>,
    filename_id: widget::Id,
    modifiers: Modifiers,
    mounter_items: HashMap<MounterKey, MounterItems>,
    nav_model: segmented_button::SingleSelectModel,
    result_opt: Option<DialogResult>,
    search_id: widget::Id,
    tab: Tab,
    key_binds: HashMap<KeyBind, Action>,
    watcher_opt: Option<(Debouncer<RecommendedWatcher, FileIdMap>, HashSet<PathBuf>)>,
}

impl App {
    fn button_view(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            ..
        } = theme::active().cosmic().spacing;

        let mut col = widget::column::with_capacity(2).spacing(space_xxs);
        if let DialogKind::SaveFile { filename } = &self.flags.kind {
            col = col.push(
                widget::text_input("", filename)
                    .id(self.filename_id.clone())
                    .on_input(Message::Filename)
                    .on_submit(Message::Save(false)),
            );
        }

        let mut row = widget::row::with_capacity(
            if !self.filters.is_empty() { 1 } else { 0 } + self.choices.len() * 2 + 3,
        )
        .align_y(Alignment::Center)
        .spacing(space_xxs);
        if !self.filters.is_empty() {
            row = row.push(widget::dropdown(
                &self.filters,
                self.filter_selected,
                Message::Filter,
            ));
        }
        for (choice_i, choice) in self.choices.iter().enumerate() {
            match choice {
                DialogChoice::CheckBox { label, value, .. } => {
                    row = row.push(widget::checkbox(label, *value).on_toggle(move |checked| {
                        Message::Choice(choice_i, if checked { 1 } else { 0 })
                    }));
                }
                DialogChoice::ComboBox {
                    label,
                    options,
                    selected,
                    ..
                } => {
                    row = row.push(widget::text::heading(label));
                    row = row.push(widget::dropdown(options, *selected, move |option_i| {
                        Message::Choice(choice_i, option_i)
                    }));
                }
            }
        }
        row = row.push(widget::horizontal_space());
        row = row.push(widget::button::standard(fl!("cancel")).on_press(Message::Cancel));
        row = row.push(if self.flags.kind.save() {
            widget::button::suggested(&self.accept_label).on_press(Message::Save(false))
        } else {
            widget::button::suggested(&self.accept_label).on_press(Message::Open)
        });

        col = col.push(row);

        widget::layer_container(col)
            .layer(cosmic_theme::Layer::Primary)
            .padding([space_xxs, space_xs])
            .into()
    }

    fn preview<'a>(&'a self, kind: &'a PreviewKind) -> Element<'a, AppMessage> {
        let mut children = Vec::with_capacity(1);
        match kind {
            PreviewKind::Custom(PreviewItem(item)) => {
                children.push(item.preview_view(IconSizes::default(), true));
            }
            PreviewKind::Location(location) => {
                if let Some(items) = self.tab.items_opt() {
                    for item in items.iter() {
                        if item.location_opt.as_ref() == Some(location) {
                            children.push(item.preview_view(self.tab.config.icon_sizes, true));
                            // Only show one property view to avoid issues like hangs when generating
                            // preview images on thousands of files
                            break;
                        }
                    }
                }
            }
            PreviewKind::Selected => {
                if let Some(items) = self.tab.items_opt() {
                    for item in items.iter() {
                        if item.selected {
                            children.push(item.preview_view(self.tab.config.icon_sizes, true));
                            // Only show one property view to avoid issues like hangs when generating
                            // preview images on thousands of files
                            break;
                        }
                    }
                    if children.is_empty() {
                        if let Some(item) = &self.tab.parent_item_opt {
                            children.push(item.preview_view(self.tab.config.icon_sizes, true));
                        }
                    }
                }
            }
        }
        widget::settings::view_column(children).into()
    }

    fn rescan_tab(&self) -> Task<Message> {
        let location = self.tab.location.clone();
        let icon_sizes = self.tab.config.icon_sizes;
        Task::perform(
            async move {
                let location2 = location.clone();
                match tokio::task::spawn_blocking(move || location2.scan(icon_sizes)).await {
                    Ok((parent_item_opt, items)) => {
                        message::app(Message::TabRescan(location, parent_item_opt, items))
                    }
                    Err(err) => {
                        log::warn!("failed to rescan: {}", err);
                        message::none()
                    }
                }
            },
            |x| x,
        )
    }

    fn search_get(&self) -> Option<&str> {
        match &self.tab.location {
            Location::Search(_, term, ..) => Some(term),
            _ => None,
        }
    }

    fn search_set(&mut self, term_opt: Option<String>) -> Task<Message> {
        let location_opt = match term_opt {
            Some(term) => match &self.tab.location {
                Location::Path(path) | Location::Search(path, ..) => Some((
                    Location::Search(
                        path.to_path_buf(),
                        term,
                        self.tab.config.show_hidden,
                        Instant::now(),
                    ),
                    true,
                )),
                _ => None,
            },
            None => match &self.tab.location {
                Location::Search(path, ..) => Some((Location::Path(path.to_path_buf()), false)),
                _ => None,
            },
        };
        if let Some((location, focus_search)) = location_opt {
            self.tab.change_location(&location, None);
            return Task::batch([
                self.update_title(),
                self.update_watcher(),
                self.rescan_tab(),
                if focus_search {
                    widget::text_input::focus(self.search_id.clone())
                } else {
                    Task::none()
                },
            ]);
        }
        Task::none()
    }

    fn update_config(&mut self) -> Task<Message> {
        self.update_nav_model();
        Task::none()
    }

    fn activate_nav_model_location(&mut self, location: &Location) {
        let nav_bar_id = self.nav_model.iter().find(|&id| {
            self.nav_model
                .data::<Location>(id)
                .map(|l| l == location)
                .unwrap_or_default()
        });

        if let Some(id) = nav_bar_id {
            self.nav_model.activate(id);
        } else {
            let active = self.nav_model.active();
            segmented_button::Selectable::deactivate(&mut self.nav_model, active);
        }
    }

    fn update_nav_model(&mut self) {
        let mut nav_model = segmented_button::ModelBuilder::default();

        nav_model = nav_model.insert(|b| {
            b.text(fl!("recents"))
                .icon(widget::icon::from_name("document-open-recent-symbolic"))
                .data(Location::Recents)
        });

        for (_favorite_i, favorite) in self.flags.config.favorites.iter().enumerate() {
            if let Some(path) = favorite.path_opt() {
                let name = if matches!(favorite, Favorite::Home) {
                    fl!("home")
                } else if let Some(file_name) = path.file_name().and_then(|x| x.to_str()) {
                    file_name.to_string()
                } else {
                    continue;
                };
                nav_model = nav_model.insert(move |b| {
                    b.text(name.clone())
                        .icon(
                            widget::icon::icon(if path.is_dir() {
                                tab::folder_icon_symbolic(&path, 16)
                            } else {
                                widget::icon::from_name("text-x-generic-symbolic")
                                    .size(16)
                                    .handle()
                            })
                            .size(16),
                        )
                        .data(Location::Path(path.clone()))
                });
            }
        }

        // Collect all mounter items
        let mut nav_items = Vec::new();
        for (key, items) in self.mounter_items.iter() {
            for item in items.iter() {
                nav_items.push((*key, item));
            }
        }
        // Sort by name lexically
        nav_items.sort_by(|a, b| LANGUAGE_SORTER.compare(&a.1.name(), &b.1.name()));
        // Add items to nav model
        for (i, (key, item)) in nav_items.into_iter().enumerate() {
            nav_model = nav_model.insert(|mut b| {
                b = b.text(item.name()).data(MounterData(key, item.clone()));
                if let Some(path) = item.path() {
                    b = b.data(Location::Path(path.clone()));
                }
                if let Some(icon) = item.icon(true) {
                    b = b.icon(widget::icon::icon(icon).size(16));
                }
                if item.is_mounted() {
                    b = b.closable();
                }
                if i == 0 {
                    b = b.divider_above();
                }
                b
            });
        }

        self.nav_model = nav_model.build();

        self.activate_nav_model_location(&self.tab.location.clone());
    }

    fn update_title(&mut self) -> Task<Message> {
        self.set_header_title(self.title.clone());
        self.set_window_title(self.title.clone(), self.flags.window_id)
    }

    fn update_watcher(&mut self) -> Task<Message> {
        if let Some((mut watcher, old_paths)) = self.watcher_opt.take() {
            let mut new_paths = HashSet::new();
            if let Some(path) = &self.tab.location.path_opt() {
                new_paths.insert(path.to_path_buf());
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
        Task::none()
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
    fn init(mut core: Core, flags: Self::Flags) -> (Self, Task<Message>) {
        core.window.context_is_overlay = false;
        core.window.show_close = false;
        core.window.show_maximize = false;
        core.window.show_minimize = false;
        // Only show details context drawer by default in open dialog
        core.window.show_context = !flags.kind.save();

        let title = flags.kind.title();
        let accept_label = flags.kind.accept_label();

        let location = Location::Path(match &flags.path_opt {
            Some(path) => path.to_path_buf(),
            None => match env::current_dir() {
                Ok(path) => path,
                Err(_) => home_dir(),
            },
        });

        let tab_config = TabConfig {
            view: tab::View::List,
            folders_first: false,
            ..Default::default()
        };
        let mut tab = Tab::new(location, tab_config);
        tab.mode = tab::Mode::Dialog(flags.kind.clone());
        tab.sort_name = tab::HeadingOptions::Modified;
        tab.sort_direction = false;

        let key_binds = key_binds(&tab.mode);

        let mut app = App {
            core,
            flags,
            title,
            accept_label,
            choices: Vec::new(),
            context_page: ContextPage::Preview(None, PreviewKind::Selected),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
            filters: Vec::new(),
            filter_selected: None,
            filename_id: widget::Id::unique(),
            modifiers: Modifiers::empty(),
            mounter_items: HashMap::new(),
            nav_model: segmented_button::ModelBuilder::default().build(),
            result_opt: None,
            search_id: widget::Id::unique(),
            tab,
            key_binds,
            watcher_opt: None,
        };

        let commands = Task::batch([
            app.update_config(),
            app.update_title(),
            app.update_watcher(),
            app.rescan_tab(),
        ]);

        (app, commands)
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        match &self.context_page {
            ContextPage::Preview(_, kind) => Some(self.preview(kind).map(Message::from)),
            _ => None,
        }
    }

    fn dialog(&self) -> Option<Element<Message>> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        //TODO: should gallery view just be a dialog?
        if self.tab.gallery {
            return Some(
                widget::column::with_children(vec![
                    self.tab.gallery_view().map(Message::TabMessage),
                    // Draw button row as part of the overlay
                    widget::container(self.button_view())
                        .width(Length::Fill)
                        .padding(space_xxs)
                        .class(theme::Container::WindowBackground)
                        .into(),
                ])
                .into(),
            );
        }

        let dialog_page = match self.dialog_pages.front() {
            Some(some) => some,
            None => return None,
        };

        let dialog = match dialog_page {
            DialogPage::NewFolder { parent, name } => {
                let mut dialog = widget::dialog(fl!("create-new-folder"));

                let complete_maybe = if name.is_empty() {
                    None
                } else if name == "." || name == ".." {
                    dialog = dialog.tertiary_action(widget::text::body(fl!(
                        "name-invalid",
                        filename = name.as_str()
                    )));
                    None
                } else if name.contains('/') {
                    dialog = dialog.tertiary_action(widget::text::body(fl!("name-no-slashes")));
                    None
                } else {
                    let path = parent.join(name);
                    if path.exists() {
                        if path.is_dir() {
                            dialog = dialog
                                .tertiary_action(widget::text::body(fl!("folder-already-exists")));
                        } else {
                            dialog = dialog
                                .tertiary_action(widget::text::body(fl!("file-already-exists")));
                        }
                        None
                    } else {
                        if name.starts_with('.') {
                            dialog = dialog.tertiary_action(widget::text::body(fl!("name-hidden")));
                        }
                        Some(Message::DialogComplete)
                    }
                };

                dialog
                    .primary_action(
                        widget::button::suggested(fl!("save"))
                            .on_press_maybe(complete_maybe.clone()),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(
                        widget::column::with_children(vec![
                            widget::text::body(fl!("folder-name")).into(),
                            widget::text_input("", name.as_str())
                                .id(self.dialog_text_input.clone())
                                .on_input(move |name| {
                                    Message::DialogUpdate(DialogPage::NewFolder {
                                        parent: parent.clone(),
                                        name,
                                    })
                                })
                                .on_submit_maybe(complete_maybe)
                                .into(),
                        ])
                        .spacing(space_xxs),
                    )
            }
            DialogPage::Replace { filename } => {
                widget::dialog(fl!("replace-title", filename = filename.as_str()))
                    .icon(widget::icon::from_name("dialog-question").size(64))
                    .body(fl!("replace-warning"))
                    .primary_action(
                        widget::button::suggested(fl!("replace")).on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
            }
        };

        Some(dialog.into())
    }

    fn footer(&self) -> Option<Element<Message>> {
        Some(self.button_view())
    }

    fn header_end(&self) -> Vec<Element<Message>> {
        let mut elements = Vec::with_capacity(3);

        if let Some(term) = self.search_get() {
            if self.core.is_condensed() {
                elements.push(
                    //TODO: selected state is not appearing different
                    widget::button::icon(widget::icon::from_name("system-search-symbolic"))
                        .on_press(Message::SearchClear)
                        .padding(8)
                        .selected(true)
                        .into(),
                );
            } else {
                elements.push(
                    widget::text_input::search_input("", term)
                        .width(Length::Fixed(240.0))
                        .id(self.search_id.clone())
                        .on_clear(Message::SearchClear)
                        .on_input(Message::SearchInput)
                        .into(),
                );
            }
        } else {
            elements.push(
                widget::button::icon(widget::icon::from_name("system-search-symbolic"))
                    .on_press(Message::SearchActivate)
                    .padding(8)
                    .into(),
            );
        }

        if self.flags.kind.save() {
            elements.push(
                widget::button::icon(widget::icon::from_name("folder-new-symbolic"))
                    .on_press(Message::NewFolder)
                    .padding(8)
                    .into(),
            );
        }

        let show_details = match self.context_page {
            ContextPage::Preview(..) => self.core.window.show_context,
            _ => false,
        };
        elements.push(
            menu::dialog_menu(&self.tab, &self.key_binds, show_details)
                .map(Message::from)
                .into(),
        );

        elements
    }

    fn nav_bar(&self) -> Option<Element<message::Message<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = cosmic::widget::nav_bar(nav_model, |entity| {
            cosmic::app::Message::Cosmic(cosmic::app::cosmic::Message::NavBar(entity))
        })
        //TODO .on_close(|entity| cosmic::app::Message::App(Message::NavBarClose(entity)))
        .close_icon(
            widget::icon::from_name("media-eject-symbolic")
                .size(16)
                .icon(),
        )
        .into_container();

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        Some(Element::from(
            // XXX both must be shrink to avoid flex layout from ignoring it
            nav.width(Length::Shrink).height(Length::Shrink),
        ))
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        Some(&self.nav_model)
    }

    fn on_app_exit(&mut self) -> Option<Message> {
        self.result_opt = Some(DialogResult::Cancel);
        None
    }

    fn on_nav_select(&mut self, entity: segmented_button::Entity) -> Task<Message> {
        self.nav_model.activate(entity);
        if let Some(location) = self.nav_model.data::<Location>(entity) {
            let message = Message::TabMessage(tab::Message::Location(location.clone()));
            return self.update(message);
        }

        if let Some(data) = self.nav_model.data::<MounterData>(entity).clone() {
            if let Some(mounter) = MOUNTERS.get(&data.0) {
                return mounter.mount(data.1.clone()).map(|_| message::none());
            }
        }
        Task::none()
    }

    fn on_escape(&mut self) -> Task<Message> {
        if self.tab.gallery {
            // Close gallery if open
            self.tab.gallery = false;
            return Task::none();
        }

        if self.search_get().is_some() {
            // Close search if open
            return self.search_set(None);
        }

        if self.tab.context_menu.is_some() {
            self.tab.context_menu = None;
            return Task::none();
        }

        if self.tab.edit_location.is_some() {
            // Close location editing if enabled
            self.tab.edit_location = None;
            return Task::none();
        }

        let had_focused_button = self.tab.select_focus_id().is_some();
        if self.tab.select_none() {
            if had_focused_button {
                // Unfocus if there was a focused button
                return widget::button::focus(widget::Id::unique());
            }
            return Task::none();
        }

        self.update(Message::Cancel)
    }

    /// Handle application events here.
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::None => {}
            Message::Cancel => {
                self.result_opt = Some(DialogResult::Cancel);
                return window::close(self.flags.window_id);
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
            Message::Config(config) => {
                if config != self.flags.config {
                    log::info!("update config");
                    self.flags.config = config;
                    return self.update_config();
                }
            }
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            Message::DialogComplete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::NewFolder { parent, name } => {
                            let path = parent.join(name);
                            match fs::create_dir(&path) {
                                Ok(()) => {
                                    // cd to directory
                                    let message = Message::TabMessage(tab::Message::Location(
                                        Location::Path(path.clone()),
                                    ));
                                    return self.update(message);
                                }
                                Err(err) => {
                                    log::warn!("failed to create {:?}: {}", path, err);
                                }
                            }
                        }
                        DialogPage::Replace { .. } => {
                            return self.update(Message::Save(true));
                        }
                    }
                }
            }
            Message::DialogUpdate(dialog_page) => {
                if !self.dialog_pages.is_empty() {
                    self.dialog_pages[0] = dialog_page;
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
            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(Message::from(action.message()));
                    }
                }
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::MounterItems(mounter_key, mounter_items) => {
                // Check for unmounted folders
                let mut unmounted = Vec::new();
                if let Some(old_items) = self.mounter_items.get(&mounter_key) {
                    for old_item in old_items.iter() {
                        if let Some(old_path) = old_item.path() {
                            if old_item.is_mounted() {
                                let mut still_mounted = false;
                                for item in mounter_items.iter() {
                                    if let Some(path) = item.path() {
                                        if path == old_path {
                                            if item.is_mounted() {
                                                still_mounted = true;
                                                break;
                                            }
                                        }
                                    }
                                }
                                if !still_mounted {
                                    unmounted.push(Location::Path(old_path));
                                }
                            }
                        }
                    }
                }

                // Go back to home in any tabs that were unmounted
                let mut commands = Vec::new();
                {
                    let home_location = Location::Path(home_dir());
                    if unmounted.contains(&self.tab.location) {
                        self.tab.change_location(&home_location, None);
                        commands.push(self.update_watcher());
                        commands.push(self.rescan_tab());
                    }
                }

                // Insert new items
                self.mounter_items.insert(mounter_key, mounter_items);

                // Update nav bar
                //TODO: this could change favorites IDs while they are in use
                self.update_nav_model();

                return Task::batch(commands);
            }
            Message::NewFolder => {
                if let Some(path) = self.tab.location.path_opt() {
                    self.dialog_pages.push_back(DialogPage::NewFolder {
                        parent: path.to_path_buf(),
                        name: String::new(),
                    });
                    return widget::text_input::focus(self.dialog_text_input.clone());
                }
            }
            Message::NotifyEvents(events) => {
                log::debug!("{:?}", events);

                if let Some(path) = self.tab.location.path_opt() {
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
                                                if item.path_opt() == Some(event_path) {
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
                            if let Some(path) = item.path_opt() {
                                paths.push(path.clone());
                                let _ = update_recently_used(
                                    &path.clone(),
                                    App::APP_ID.to_string(),
                                    "cosmic-files".to_string(),
                                    None,
                                );
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
                            return Task::none();
                        }
                    }
                }

                // If there are proper matching items, return them
                if !paths.is_empty() {
                    self.result_opt = Some(DialogResult::Open(paths));
                    return window::close(self.flags.window_id);
                }

                // If we are in directory mode, return the current directory
                if self.flags.kind.is_dir() {
                    match &self.tab.location {
                        Location::Path(tab_path) => {
                            self.result_opt = Some(DialogResult::Open(vec![tab_path.clone()]));
                            return window::close(self.flags.window_id);
                        }
                        _ => {}
                    }
                }
            }
            Message::Preview => match self.context_page {
                ContextPage::Preview(..) => {
                    self.core.window.show_context = !self.core.window.show_context;
                }
                _ => {
                    self.context_page = ContextPage::Preview(None, PreviewKind::Selected);
                    self.core.window.show_context = true;
                }
            },
            Message::Save(replace) => {
                if let DialogKind::SaveFile { filename } = &self.flags.kind {
                    if !filename.is_empty() {
                        if let Some(tab_path) = self.tab.location.path_opt() {
                            let path = tab_path.join(&filename);
                            if path.is_dir() {
                                // cd to directory
                                let message = Message::TabMessage(tab::Message::Location(
                                    Location::Path(path.clone()),
                                ));
                                return self.update(message);
                            } else if !replace && path.exists() {
                                self.dialog_pages.push_back(DialogPage::Replace {
                                    filename: filename.clone(),
                                });
                            } else {
                                self.result_opt = Some(DialogResult::Open(vec![path]));
                                return window::close(self.flags.window_id);
                            }
                        }
                    }
                }
            }
            Message::SearchActivate => {
                return if self.search_get().is_none() {
                    self.search_set(Some(String::new()))
                } else {
                    widget::text_input::focus(self.search_id.clone())
                };
            }
            Message::SearchClear => {
                return self.search_set(None);
            }
            Message::SearchInput(input) => {
                return self.search_set(Some(input));
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
                            commands.push(self.update(Message::from(action.message())));
                        }
                        tab::Command::ChangeLocation(_tab_title, _tab_path, _selection_path) => {
                            commands.push(Task::batch([self.update_watcher(), self.rescan_tab()]));
                        }
                        tab::Command::Iced(iced_command) => {
                            commands.push(
                                iced_command.0.map(|tab_message| {
                                    message::app(Message::TabMessage(tab_message))
                                }),
                            );
                        }
                        tab::Command::OpenFile(_item_path) => {
                            if self.flags.kind.save() {
                                commands.push(self.update(Message::Save(false)));
                            } else {
                                commands.push(self.update(Message::Open));
                            }
                        }
                        tab::Command::Preview(kind) => {
                            self.context_page = ContextPage::Preview(None, kind);
                            self.set_show_context(true);
                            self.set_context_title(self.context_page.title());
                        }
                        tab::Command::WindowDrag => {
                            commands.push(window::drag(self.flags.window_id));
                        }
                        tab::Command::WindowToggleMaximize => {
                            commands.push(window::toggle_maximize(self.flags.window_id));
                        }
                        unsupported => {
                            log::warn!("{unsupported:?} not supported in dialog mode");
                        }
                    }
                }
                return Task::batch(commands);
            }
            Message::TabRescan(location, parent_item_opt, mut items) => {
                if location == self.tab.location {
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
                                                log::warn!(
                                                    "failed to parse glob {:?}: {}",
                                                    value,
                                                    err
                                                );
                                            }
                                        }
                                    }
                                    DialogFilterPattern::Mime(value) => {
                                        match mime_guess::Mime::from_str(value) {
                                            Ok(mime) => parsed_mimes.push(mime),
                                            Err(err) => {
                                                log::warn!(
                                                    "failed to parse mime {:?}: {}",
                                                    value,
                                                    err
                                                );
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

                    self.tab.parent_item_opt = parent_item_opt;
                    self.tab.set_items(items);

                    // Reset focus on location change
                    if self.search_get().is_some() {
                        return widget::text_input::focus(self.search_id.clone());
                    } else {
                        return widget::text_input::focus(self.filename_id.clone());
                    }
                }
            }
            Message::TabView(view) => {
                self.tab.config.view = view;
            }
            Message::ToggleFoldersFirst => {
                self.tab.config.folders_first = !self.tab.config.folders_first;
            }
            Message::ZoomDefault => match self.tab.config.view {
                tab::View::List => self.tab.config.icon_sizes.list = 100.try_into().unwrap(),
                tab::View::Grid => self.tab.config.icon_sizes.grid = 100.try_into().unwrap(),
            },
            Message::ZoomIn => {
                let zoom_in = |size: &mut NonZeroU16, min: u16, max: u16| {
                    let mut step = min;
                    while step <= max {
                        if size.get() < step {
                            *size = step.try_into().unwrap();
                            break;
                        }
                        step += 25;
                    }
                    if size.get() > step {
                        *size = step.try_into().unwrap();
                    }
                };
                match self.tab.config.view {
                    tab::View::List => zoom_in(&mut self.tab.config.icon_sizes.list, 50, 500),
                    tab::View::Grid => zoom_in(&mut self.tab.config.icon_sizes.grid, 50, 500),
                }
            }
            Message::ZoomOut => {
                let zoom_out = |size: &mut NonZeroU16, min: u16, max: u16| {
                    let mut step = max;
                    while step >= min {
                        if size.get() > step {
                            *size = step.try_into().unwrap();
                            break;
                        }
                        step -= 25;
                    }
                    if size.get() < step {
                        *size = step.try_into().unwrap();
                    }
                };
                match self.tab.config.view {
                    tab::View::List => zoom_out(&mut self.tab.config.icon_sizes.list, 50, 500),
                    tab::View::Grid => zoom_out(&mut self.tab.config.icon_sizes.grid, 50, 500),
                }
            }
        }

        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let mut col = widget::column::with_capacity(2);

        if self.core.is_condensed() {
            if let Some(term) = self.search_get() {
                col = col.push(
                    widget::container(
                        widget::text_input::search_input("", term)
                            .width(Length::Fill)
                            .id(self.search_id.clone())
                            .on_clear(Message::SearchClear)
                            .on_input(Message::SearchInput),
                    )
                    .padding(space_xxs),
                )
            }
        }

        col = col.push(
            self.tab
                .view(&self.key_binds)
                .map(move |message| Message::TabMessage(message)),
        );

        col.into()
    }

    fn subscription(&self) -> Subscription<Message> {
        struct WatcherSubscription;
        let mut subscriptions = vec![
            event::listen_with(|event, status, _window_id| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            Config::subscription().map(|update| {
                if !update.errors.is_empty() {
                    log::info!(
                        "errors loading config {:?}: {:?}",
                        update.keys,
                        update.errors
                    );
                }
                Message::Config(update.config)
            }),
            Subscription::run_with_id(
                TypeId::of::<WatcherSubscription>(),
                stream::channel(100, |mut output| async move {
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
                }),
            ),
            self.tab.subscription().map(Message::TabMessage),
        ];

        for (key, mounter) in MOUNTERS.iter() {
            let key = *key;
            subscriptions.push(mounter.subscription().map(move |mounter_message| {
                match mounter_message {
                    MounterMessage::Items(items) => Message::MounterItems(key, items),
                    _ => {
                        log::warn!("{:?} not supported in dialog mode", mounter_message);
                        Message::None
                    }
                }
            }));
        }

        Subscription::batch(subscriptions)
    }
}
