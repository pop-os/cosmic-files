// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "wayland")]
use cosmic::iced::{
    event::wayland::{Event as WaylandEvent, OutputEvent},
    wayland::{
        actions::layer_surface::{IcedMargin, IcedOutput, SctkLayerSurfaceSettings},
        layer_surface::{
            destroy_layer_surface, get_layer_surface, Anchor, KeyboardInteractivity, Layer,
        },
    },
    Limits,
};
use cosmic::{
    app::{self, message, Command, Core},
    cosmic_config, cosmic_theme, executor,
    iced::{
        clipboard::dnd::DndAction,
        event,
        futures::{self, SinkExt},
        keyboard::{Event as KeyEvent, Key, Modifiers},
        subscription::{self, Subscription},
        window::{self, Event as WindowEvent, Id as WindowId},
        Alignment, Event, Length,
    },
    iced_runtime::clipboard,
    style, theme,
    widget::{
        self,
        dnd_destination::DragId,
        menu::{action::MenuAction, key_bind::KeyBind},
        segmented_button::{self, Entity},
    },
    Application, ApplicationExt, Element,
};
use notify_debouncer_full::{
    new_debouncer,
    notify::{self, RecommendedWatcher, Watcher},
    DebouncedEvent, Debouncer, FileIdMap,
};
use slotmap::Key as SlotMapKey;
use std::{
    any::TypeId, collections::{BTreeMap, HashMap, HashSet, VecDeque}, env, ffi::OsStr, fmt, fs, future::pending, io::{BufRead, BufReader}, num::NonZeroU16, os::unix::fs::PermissionsExt, path::PathBuf, process, sync::{Arc, Mutex}, time::{self, Instant}
};
use tokio::sync::mpsc;
use trash::TrashItem;
#[cfg(feature = "wayland")]
use wayland_client::{protocol::wl_output::WlOutput, Proxy};

use crate::{
    clipboard::{ClipboardCopy, ClipboardKind, ClipboardPaste},
    config::{AppTheme, Config, Favorite, IconSizes, TabConfig},
    fl, home_dir,
    key_bind::key_binds,
    localize::LANGUAGE_SORTER,
    menu, mime_app, mime_icon,
    mounter::{
        mounters, MounterAuth, MounterItem, MounterItems, MounterKey, MounterMessage, Mounters,
    },
    operation::{Operation, ReplaceResult},
    spawn_detached::spawn_detached,
    tab::{self, HeadingOptions, ItemMetadata, Location, Tab, HOVER_DURATION},
};

#[derive(Clone, Debug)]
pub enum Mode {
    App,
    Desktop,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
    pub mode: Mode,
    pub locations: Vec<Location>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    About,
    AddToSidebar,
    Compress,
    Copy,
    Cut,
    EditHistory,
    EditLocation,
    ExtractHere,
    HistoryNext,
    HistoryPrevious,
    ItemDown,
    ItemLeft,
    ItemRight,
    ItemUp,
    LocationUp,
    MoveToTrash,
    NewFile,
    NewFolder,
    Open,
    OpenInNewTab,
    OpenInNewWindow,
    OpenItemLocation,
    OpenTerminal,
    OpenWith,
    Paste,
    Preview,
    Rename,
    RestoreFromTrash,
    SearchActivate,
    SelectAll,
    SetSort(HeadingOptions, bool),
    Settings,
    TabClose,
    TabNew,
    TabNext,
    TabPrev,
    TabViewGrid,
    TabViewList,
    ToggleFoldersFirst,
    ToggleShowHidden,
    ToggleSort(HeadingOptions),
    WindowClose,
    WindowNew,
    ZoomDefault,
    ZoomIn,
    ZoomOut,
    Recents,
}

impl Action {
    fn message(&self, entity_opt: Option<Entity>) -> Message {
        match self {
            Action::About => Message::ToggleContextPage(ContextPage::About),
            Action::AddToSidebar => Message::AddToSidebar(entity_opt),
            Action::Compress => Message::Compress(entity_opt),
            Action::Copy => Message::Copy(entity_opt),
            Action::Cut => Message::Cut(entity_opt),
            Action::EditHistory => Message::ToggleContextPage(ContextPage::EditHistory),
            Action::EditLocation => Message::EditLocation(entity_opt),
            Action::ExtractHere => Message::ExtractHere(entity_opt),
            Action::HistoryNext => Message::TabMessage(entity_opt, tab::Message::GoNext),
            Action::HistoryPrevious => Message::TabMessage(entity_opt, tab::Message::GoPrevious),
            Action::ItemDown => Message::TabMessage(entity_opt, tab::Message::ItemDown),
            Action::ItemLeft => Message::TabMessage(entity_opt, tab::Message::ItemLeft),
            Action::ItemRight => Message::TabMessage(entity_opt, tab::Message::ItemRight),
            Action::ItemUp => Message::TabMessage(entity_opt, tab::Message::ItemUp),
            Action::LocationUp => Message::TabMessage(entity_opt, tab::Message::LocationUp),
            Action::MoveToTrash => Message::MoveToTrash(entity_opt),
            Action::NewFile => Message::NewItem(entity_opt, false),
            Action::NewFolder => Message::NewItem(entity_opt, true),
            Action::Open => Message::TabMessage(entity_opt, tab::Message::Open(None)),
            Action::OpenInNewTab => Message::OpenInNewTab(entity_opt),
            Action::OpenInNewWindow => Message::OpenInNewWindow(entity_opt),
            Action::OpenItemLocation => Message::OpenItemLocation(entity_opt),
            Action::OpenTerminal => Message::OpenTerminal(entity_opt),
            Action::OpenWith => Message::ToggleContextPage(ContextPage::OpenWith),
            Action::Paste => Message::Paste(entity_opt),
            Action::Preview => {
                Message::ToggleContextPage(ContextPage::Preview(entity_opt, PreviewKind::Selected))
            }
            Action::Rename => Message::Rename(entity_opt),
            Action::RestoreFromTrash => Message::RestoreFromTrash(entity_opt),
            Action::SearchActivate => Message::SearchActivate,
            Action::SelectAll => Message::TabMessage(entity_opt, tab::Message::SelectAll),
            Action::SetSort(sort, dir) => {
                Message::TabMessage(entity_opt, tab::Message::SetSort(*sort, *dir))
            }
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
            Action::ToggleFoldersFirst => {
                Message::TabMessage(entity_opt, tab::Message::ToggleFoldersFirst)
            }
            Action::ToggleShowHidden => {
                Message::TabMessage(entity_opt, tab::Message::ToggleShowHidden)
            }
            Action::ToggleSort(sort) => {
                Message::TabMessage(entity_opt, tab::Message::ToggleSort(*sort))
            }
            Action::WindowClose => Message::WindowClose,
            Action::WindowNew => Message::WindowNew,
            Action::ZoomDefault => Message::TabMessage(entity_opt, tab::Message::ZoomDefault),
            Action::ZoomIn => Message::TabMessage(entity_opt, tab::Message::ZoomIn),
            Action::ZoomOut => Message::TabMessage(entity_opt, tab::Message::ZoomOut),
            Action::Recents => Message::Recents,
        }
    }
}

impl MenuAction for Action {
    type Message = Message;

    fn message(&self) -> Message {
        self.message(None)
    }
}

#[derive(Clone, Debug)]
pub struct PreviewItem(pub tab::Item);

impl PartialEq for PreviewItem {
    fn eq(&self, other: &Self) -> bool {
        self.0.location_opt == other.0.location_opt
    }
}

impl Eq for PreviewItem {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreviewKind {
    Custom(PreviewItem),
    Location(Location),
    Selected,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NavMenuAction {
    OpenInNewTab(segmented_button::Entity),
    OpenInNewWindow(segmented_button::Entity),
    Preview(segmented_button::Entity),
    RemoveFromSidebar(segmented_button::Entity),
    EmptyTrash,
}

impl MenuAction for NavMenuAction {
    type Message = cosmic::app::Message<Message>;

    fn message(&self) -> Self::Message {
        cosmic::app::Message::App(Message::NavMenuAction(*self))
    }
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    AddToSidebar(Option<Entity>),
    AppTheme(AppTheme),
    CloseToast(widget::ToastId),
    Compress(Option<Entity>),
    Config(Config),
    Copy(Option<Entity>),
    Cut(Option<Entity>),
    DialogCancel,
    DialogComplete,
    DialogPush(DialogPage),
    DialogUpdate(DialogPage),
    DialogUpdateComplete(DialogPage),
    EditLocation(Option<Entity>),
    ExtractHere(Option<Entity>),
    Key(Modifiers, Key),
    LaunchUrl(String),
    MaybeExit,
    Modifiers(Modifiers),
    MoveToTrash(Option<Entity>),
    MounterItems(MounterKey, MounterItems),
    NavBarClose(Entity),
    NavBarContext(Entity),
    NavMenuAction(NavMenuAction),
    NetworkAuth(MounterKey, String, MounterAuth, mpsc::Sender<MounterAuth>),
    NetworkDriveInput(String),
    NetworkDriveSubmit,
    NetworkResult(MounterKey, String, Result<bool, String>),
    NewItem(Option<Entity>, bool),
    #[cfg(feature = "notify")]
    Notification(Arc<Mutex<notify_rust::NotificationHandle>>),
    NotifyEvents(Vec<DebouncedEvent>),
    NotifyWatcher(WatcherWrapper),
    OpenTerminal(Option<Entity>),
    OpenWith(PathBuf, mime_app::MimeApp),
    OpenInNewTab(Option<Entity>),
    OpenInNewWindow(Option<Entity>),
    OpenItemLocation(Option<Entity>),
    Paste(Option<Entity>),
    PasteContents(PathBuf, ClipboardPaste),
    PendingComplete(u64),
    PendingError(u64, String),
    PendingProgress(u64, f32),
    Preview(Entity, PreviewKind, time::Duration),
    RescanTrash,
    Rename(Option<Entity>),
    ReplaceResult(ReplaceResult),
    RestoreFromTrash(Option<Entity>),
    SearchActivate,
    SearchClear,
    SearchInput(String),
    SearchSubmit,
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    TabActivate(Entity),
    TabNext,
    TabPrev,
    TabClose(Option<Entity>),
    TabConfig(TabConfig),
    TabMessage(Option<Entity>, tab::Message),
    TabNew,
    TabRescan(Entity, Location, Vec<tab::Item>, Option<PathBuf>),
    ToggleContextPage(ContextPage),
    Undo(usize),
    UndoTrash(widget::ToastId, Arc<[PathBuf]>),
    UndoTrashStart(Vec<TrashItem>),
    WindowClose,
    WindowNew,
    DndHoverLocTimeout(Location),
    DndHoverTabTimeout(Entity),
    DndEnterNav(Entity),
    DndExitNav,
    DndEnterTab(Entity),
    DndExitTab,
    DndDropTab(Entity, Option<ClipboardPaste>, DndAction),
    DndDropNav(Entity, Option<ClipboardPaste>, DndAction),
    Recents,
    #[cfg(feature = "wayland")]
    OutputEvent(OutputEvent, WlOutput),
    Cosmic(app::cosmic::Message),
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    EditHistory,
    NetworkDrive,
    OpenWith,
    Preview(Option<Entity>, PreviewKind),
    Settings,
}

impl ContextPage {
    pub fn title(&self) -> String {
        match self {
            Self::About => String::new(),
            Self::EditHistory => fl!("edit-history"),
            Self::NetworkDrive => fl!("add-network-drive"),
            Self::OpenWith => fl!("open-with"),
            Self::Preview(..) => String::default(),
            Self::Settings => fl!("settings"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ArchiveType {
    Tgz,
    #[default]
    Zip,
}

impl ArchiveType {
    pub fn all() -> &'static [Self] {
        &[Self::Tgz, Self::Zip]
    }

    pub fn extension(&self) -> &str {
        match self {
            ArchiveType::Tgz => ".tgz",
            ArchiveType::Zip => ".zip",
        }
    }
}

impl AsRef<str> for ArchiveType {
    fn as_ref(&self) -> &str {
        self.extension()
    }
}

#[derive(Clone, Debug)]
pub enum DialogPage {
    Compress {
        paths: Vec<PathBuf>,
        to: PathBuf,
        name: String,
        archive_type: ArchiveType,
    },
    EmptyTrash,
    FailedOperation(u64),
    NetworkAuth {
        mounter_key: MounterKey,
        uri: String,
        auth: MounterAuth,
        auth_tx: mpsc::Sender<MounterAuth>,
    },
    NetworkError {
        mounter_key: MounterKey,
        uri: String,
        error: String,
    },
    NewItem {
        parent: PathBuf,
        name: String,
        dir: bool,
    },
    RenameItem {
        from: PathBuf,
        parent: PathBuf,
        name: String,
        dir: bool,
    },
    Replace {
        from: tab::Item,
        to: tab::Item,
        multiple: bool,
        apply_to_all: bool,
        tx: mpsc::Sender<ReplaceResult>,
    },
}

pub struct FavoriteIndex(usize);

pub struct MounterData(MounterKey, MounterItem);

pub struct WatcherWrapper {
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
pub struct App {
    core: Core,
    nav_bar_context_id: segmented_button::Entity,
    nav_model: segmented_button::SingleSelectModel,
    tab_model: segmented_button::Model<segmented_button::SingleSelect>,
    config_handler: Option<cosmic_config::Config>,
    config: Config,
    mode: Mode,
    app_themes: Vec<String>,
    default_view: Vec<String>,
    sort_by_names: Vec<String>,
    sort_direction: Vec<String>,
    context_page: ContextPage,
    dialog_pages: VecDeque<DialogPage>,
    dialog_text_input: widget::Id,
    key_binds: HashMap<KeyBind, Action>,
    modifiers: Modifiers,
    mounters: Mounters,
    mounter_items: HashMap<MounterKey, MounterItems>,
    network_drive_connecting: Option<(MounterKey, String)>,
    network_drive_input: String,
    #[cfg(feature = "notify")]
    notification_opt: Option<Arc<Mutex<notify_rust::NotificationHandle>>>,
    pending_operation_id: u64,
    pending_operations: BTreeMap<u64, (Operation, f32)>,
    complete_operations: BTreeMap<u64, Operation>,
    failed_operations: BTreeMap<u64, (Operation, String)>,
    preview_opt: Option<(Entity, PreviewKind, time::Instant)>,
    search_active: bool,
    search_id: widget::Id,
    search_input: String,
    #[cfg(feature = "wayland")]
    surface_ids: HashMap<WlOutput, WindowId>,
    #[cfg(feature = "wayland")]
    surface_names: HashMap<WindowId, String>,
    toasts: widget::toaster::Toasts<Message>,
    watcher_opt: Option<(Debouncer<RecommendedWatcher, FileIdMap>, HashSet<PathBuf>)>,
    window_id_opt: Option<window::Id>,
    nav_dnd_hover: Option<(Location, Instant)>,
    tab_dnd_hover: Option<(Entity, Instant)>,
    nav_drag_id: DragId,
    tab_drag_id: DragId,
}

impl App {
    fn open_tab(
        &mut self,
        location: Location,
        activate: bool,
        selection_path: Option<PathBuf>,
    ) -> Command<Message> {
        let mut tab = Tab::new(location.clone(), self.config.tab);
        tab.mode = match self.mode {
            Mode::App => tab::Mode::App,
            Mode::Desktop => {
                tab.config.view = tab::View::Grid;
                tab::Mode::Desktop
            }
        };
        let entity = self
            .tab_model
            .insert()
            .text(tab.title())
            .data(tab)
            .closable();

        let entity = if activate {
            entity.activate().id()
        } else {
            entity.id()
        };

        Command::batch([
            self.update_title(),
            self.update_watcher(),
            self.rescan_tab(entity, location, selection_path),
        ])
    }

    fn operation(&mut self, operation: Operation) {
        let id = self.pending_operation_id;
        self.pending_operation_id += 1;
        self.pending_operations.insert(id, (operation, 0.0));
    }

    fn rescan_tab(
        &mut self,
        entity: Entity,
        location: Location,
        selection_path: Option<PathBuf>,
    ) -> Command<Message> {
        let mounters = self.mounters.clone();
        let icon_sizes = self.config.tab.icon_sizes;
        Command::perform(
            async move {
                let location2 = location.clone();
                match tokio::task::spawn_blocking(move || location2.scan(mounters, icon_sizes))
                    .await
                {
                    Ok(items) => {
                        message::app(Message::TabRescan(entity, location, items, selection_path))
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

    fn rescan_trash(&mut self) -> Command<Message> {
        let mut needs_reload = Vec::new();
        for entity in self.tab_model.iter() {
            if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                if let Location::Trash = &tab.location {
                    needs_reload.push((entity, Location::Trash));
                }
            }
        }

        let mut commands = Vec::with_capacity(needs_reload.len());
        for (entity, location) in needs_reload {
            commands.push(self.rescan_tab(entity, location, None));
        }
        Command::batch(commands)
    }

    fn search(&mut self) -> Command<Message> {
        let entity = self.tab_model.active();
        let mut title_location_opt = None;
        if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
            match &tab.location {
                Location::Path(path) | Location::Search(path, ..) => {
                    let location = if !self.search_input.is_empty() {
                        Location::Search(path.clone(), self.search_input.clone())
                    } else {
                        Location::Path(path.clone())
                    };
                    tab.change_location(&location, None);
                    title_location_opt = Some((tab.title(), tab.location.clone()));
                }
                _ => {}
            }
        }
        if let Some((title, location)) = title_location_opt {
            self.tab_model.text_set(entity, title);
            return Command::batch([
                self.update_title(),
                self.update_watcher(),
                self.rescan_tab(entity, location, None),
            ]);
        }
        Command::none()
    }

    fn selected_paths(&self, entity_opt: Option<Entity>) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
        if let Some(tab) = self.tab_model.data::<Tab>(entity) {
            for location in tab.selected_locations() {
                if let Some(path) = location.path_opt() {
                    paths.push(path.to_path_buf());
                }
            }
        }
        paths
    }

    fn update_config(&mut self) -> Command<Message> {
        self.update_nav_model();
        cosmic::app::command::set_theme(self.config.app_theme.theme())
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

        for (favorite_i, favorite) in self.config.favorites.iter().enumerate() {
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
                        .data(FavoriteIndex(favorite_i))
                });
            }
        }

        nav_model = nav_model.insert(|b| {
            b.text(fl!("trash"))
                .icon(widget::icon::icon(tab::trash_icon_symbolic(16)))
                .data(Location::Trash)
                .divider_above()
        });

        if !self.mounters.is_empty() {
            nav_model = nav_model.insert(|b| {
                b.text(fl!("networks"))
                    .icon(widget::icon::icon(
                        widget::icon::from_name("network-workgroup-symbolic")
                            .size(16)
                            .handle(),
                    ))
                    .data(Location::Network(
                        "network:///".to_string(),
                        fl!("networks"),
                    ))
                    .divider_above()
            });
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
                if let Some(icon) = item.icon() {
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

        let tab_entity = self.tab_model.active();
        if let Some(tab) = self.tab_model.data::<Tab>(tab_entity) {
            self.activate_nav_model_location(&tab.location.clone());
        }
    }

    fn update_notification(&mut self) -> Command<Message> {
        // Handle closing notification if there are no operations
        if self.pending_operations.is_empty() {
            #[cfg(feature = "notify")]
            if let Some(notification_arc) = self.notification_opt.take() {
                return Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            //TODO: this is nasty
                            let notification_mutex = Arc::try_unwrap(notification_arc).unwrap();
                            let notification = notification_mutex.into_inner().unwrap();
                            notification.close();
                        })
                        .await
                        .unwrap();
                        message::app(Message::MaybeExit)
                    },
                    |x| x,
                );
            }
        }

        Command::none()
    }

    fn update_title(&mut self) -> Command<Message> {
        let window_title = match self.tab_model.text(self.tab_model.active()) {
            Some(tab_title) => format!("{tab_title} — {}", fl!("cosmic-files")),
            None => fl!("cosmic-files"),
        };
        self.set_window_title(window_title, window::Id::MAIN)
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

    fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;
        let repository = "https://github.com/pop-os/cosmic-files";
        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");
        widget::column::with_children(vec![
                widget::svg(widget::svg::Handle::from_memory(
                    &include_bytes!(
                        "../res/icons/hicolor/128x128/apps/com.system76.CosmicFiles.svg"
                    )[..],
                ))
                .into(),
                widget::text::title3(fl!("cosmic-files")).into(),
                widget::button::link(repository)
                    .on_press(Message::LaunchUrl(repository.to_string()))
                    .padding(0)
                    .into(),
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                    .on_press(Message::LaunchUrl(format!("{}/commits/{}", repository, hash)))
                    .padding(0)
                .into(),
            ])
        .align_items(Alignment::Center)
        .spacing(space_xxs)
        .into()
    }

    fn network_drive(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_m, ..
        } = theme::active().cosmic().spacing;
        let mut text_input =
            widget::text_input(fl!("enter-server-address"), &self.network_drive_input);
        let button = if self.network_drive_connecting.is_some() {
            widget::button::standard(fl!("connecting"))
        } else {
            text_input = text_input
                .on_input(Message::NetworkDriveInput)
                .on_submit(Message::NetworkDriveSubmit);
            widget::button::standard(fl!("connect")).on_press(Message::NetworkDriveSubmit)
        };
        let mut table = widget::column::with_capacity(8);
        for (i, line) in fl!("network-drive-schemes").lines().enumerate() {
            let mut row = widget::row::with_capacity(2);
            for part in line.split(',') {
                row = row.push(
                    widget::container(if i == 0 {
                        widget::text::heading(part.to_string())
                    } else {
                        widget::text::body(part.to_string())
                    })
                    .width(Length::Fill)
                    .padding(space_xxs),
                );
            }
            table = table.push(row);
            if i == 0 {
                table = table.push(widget::divider::horizontal::light());
            }
        }
        widget::column::with_children(vec![
            text_input.into(),
            widget::text(fl!("network-drive-description")).into(),
            table.into(),
            widget::row::with_children(vec![
                widget::horizontal_space(Length::Fill).into(),
                button.into(),
            ])
            .into(),
        ])
        .spacing(space_m)
        .into()
    }

    fn open_with(&self) -> Element<Message> {
        let mut children = Vec::new();
        let entity = self.tab_model.active();
        if let Some(tab) = self.tab_model.data::<Tab>(entity) {
            if let Some(items) = tab.items_opt() {
                for item in items.iter() {
                    if item.selected {
                        children.push(item.open_with_view(tab.config.icon_sizes));
                        // Only show one property view to avoid issues like hangs when generating
                        // preview images on thousands of files
                        break;
                    }
                }
            }
        }
        widget::settings::view_column(children).into()
    }

    fn edit_history(&self) -> Element<Message> {
        let mut children = Vec::new();

        //TODO: get height from theme?
        let progress_bar_height = Length::Fixed(4.0);

        if !self.pending_operations.is_empty() {
            let mut section = widget::settings::view_section(fl!("pending"));
            for (_id, (op, progress)) in self.pending_operations.iter().rev() {
                section = section.add(widget::column::with_children(vec![
                    widget::text(op.pending_text()).into(),
                    widget::progress_bar(0.0..=100.0, *progress)
                        .height(progress_bar_height)
                        .into(),
                ]));
            }
            children.push(section.into());
        }

        if !self.failed_operations.is_empty() {
            let mut section = widget::settings::view_section(fl!("failed"));
            for (_id, (op, error)) in self.failed_operations.iter().rev() {
                section = section.add(widget::column::with_children(vec![
                    widget::text(op.pending_text()).into(),
                    widget::text(error).into(),
                ]));
            }
            children.push(section.into());
        }

        if !self.complete_operations.is_empty() {
            let mut section = widget::settings::view_section(fl!("complete"));
            for (_id, op) in self.complete_operations.iter().rev() {
                section = section.add(widget::text(op.completed_text()));
            }
            children.push(section.into());
        }

        if children.is_empty() {
            children.push(widget::text::body(fl!("no-history")).into());
        }

        widget::settings::view_column(children).into()
    }

    fn preview(&self, entity_opt: &Option<Entity>, kind: &PreviewKind) -> Element<Message> {
        let mut children = Vec::with_capacity(1);
        let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
        match kind {
            PreviewKind::Custom(PreviewItem(item)) => {
                children.push(item.preview_view(IconSizes::default()));
            }
            PreviewKind::Location(location) => {
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        for item in items.iter() {
                            if item.location_opt.as_ref() == Some(location) {
                                children.push(item.preview_view(tab.config.icon_sizes));
                                // Only show one property view to avoid issues like hangs when generating
                                // preview images on thousands of files
                                break;
                            }
                        }
                    }
                }
            }
            PreviewKind::Selected => {
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        for item in items.iter() {
                            if item.selected {
                                children.push(item.preview_view(tab.config.icon_sizes));
                                // Only show one property view to avoid issues like hangs when generating
                                // preview images on thousands of files
                                break;
                            }
                        }
                    }
                }
            }
        }
        widget::settings::view_column(children).into()
    }

    fn settings(&self) -> Element<Message> {
        // TODO: Should dialog be updated here too?
        widget::settings::view_column(vec![
            widget::settings::view_section(fl!("appearance"))
                .add({
                    let app_theme_selected = match self.config.app_theme {
                        AppTheme::Dark => 1,
                        AppTheme::Light => 2,
                        AppTheme::System => 0,
                    };
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
                    ))
                })
                .add({
                    let tab_config = self.config.tab.clone();
                    widget::settings::item::builder(fl!("default-view")).control(widget::dropdown(
                        &self.default_view,
                        match tab_config.view {
                            tab::View::Grid => Some(0),
                            tab::View::List => Some(1),
                        },
                        move |index| {
                            Message::TabConfig(TabConfig {
                                view: match index {
                                    0 => tab::View::Grid,
                                    _ => tab::View::List,
                                },
                                ..tab_config
                            })
                        },
                    ))
                })
                .add({
                    let tab_config = self.config.tab.clone();
                    let list: u16 = tab_config.icon_sizes.list.into();
                    widget::settings::item::builder(fl!("icon-size-list"))
                        .description(format!("{}%", list))
                        .control(
                            widget::slider(50..=500, list, move |list| {
                                Message::TabConfig(TabConfig {
                                    icon_sizes: IconSizes {
                                        list: NonZeroU16::new(list).unwrap(),
                                        ..tab_config.icon_sizes
                                    },
                                    ..tab_config
                                })
                            })
                            .step(25u16),
                        )
                })
                .add({
                    let tab_config = self.config.tab.clone();
                    let grid: u16 = tab_config.icon_sizes.grid.into();
                    widget::settings::item::builder(fl!("icon-size-grid"))
                        .description(format!("{}%", grid))
                        .control(
                            widget::slider(50..=500, grid, move |grid| {
                                Message::TabConfig(TabConfig {
                                    icon_sizes: IconSizes {
                                        grid: NonZeroU16::new(grid).unwrap(),
                                        ..tab_config.icon_sizes
                                    },
                                    ..tab_config
                                })
                            })
                            .step(25u16),
                        )
                })
                .add({
                    let tab_config = self.config.tab;
                    let sort_by_selected = tab_config.sort_name as _;

                    widget::settings::item::builder(fl!("sorting-name")).control(widget::dropdown(
                        &self.sort_by_names,
                        Some(sort_by_selected),
                        move |index| {
                            Message::TabConfig(TabConfig {
                                sort_name: match index {
                                    0 => HeadingOptions::Name,
                                    1 => HeadingOptions::Modified,
                                    2 => HeadingOptions::Size,
                                    _ => HeadingOptions::Name,
                                },
                                ..tab_config
                            })
                        },
                    ))
                })
                .add({
                    let tab_config = self.config.tab;
                    // Ascending is true. Descending is false
                    let direction = tab_config.sort_direction.into();

                    widget::settings::item::builder(fl!("direction")).control(widget::dropdown(
                        &self.sort_direction,
                        Some(direction),
                        move |index| {
                            Message::TabConfig(TabConfig {
                                sort_direction: index == 1,
                                ..tab_config
                            })
                        },
                    ))
                })
                .into(),
            widget::settings::view_section(fl!("settings-tab"))
                .add({
                    let tab_config = self.config.tab.clone();
                    widget::settings::item::builder(fl!("settings-show-hidden")).toggler(
                        tab_config.show_hidden,
                        move |show_hidden| {
                            Message::TabConfig(TabConfig {
                                show_hidden,
                                ..tab_config
                            })
                        },
                    )
                })
                .into(),
        ])
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
    fn init(mut core: Core, flags: Self::Flags) -> (Self, Command<Self::Message>) {
        core.window.context_is_overlay = false;
        match flags.mode {
            Mode::App => {}
            Mode::Desktop => {
                core.window.content_container = false;
                core.window.show_window_menu = false;
                core.window.show_headerbar = false;
                core.window.sharp_corners = false;
                core.window.show_maximize = false;
                core.window.show_minimize = false;
                core.window.use_template = true;
            }
        }

        let app_themes = vec![fl!("match-desktop"), fl!("dark"), fl!("light")];

        let mut app = App {
            core,
            nav_bar_context_id: segmented_button::Entity::null(),
            nav_model: segmented_button::ModelBuilder::default().build(),
            tab_model: segmented_button::ModelBuilder::default().build(),
            config_handler: flags.config_handler,
            config: flags.config,
            mode: flags.mode,
            app_themes,
            default_view: vec![fl!("grid-view"), fl!("list-view")],
            sort_by_names: HeadingOptions::names(),
            sort_direction: vec![fl!("descending"), fl!("ascending")],
            context_page: ContextPage::Settings,
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            mounters: mounters(),
            mounter_items: HashMap::new(),
            network_drive_connecting: None,
            network_drive_input: String::new(),
            #[cfg(feature = "notify")]
            notification_opt: None,
            pending_operation_id: 0,
            pending_operations: BTreeMap::new(),
            complete_operations: BTreeMap::new(),
            failed_operations: BTreeMap::new(),
            preview_opt: None,
            search_active: false,
            search_id: widget::Id::unique(),
            search_input: String::new(),
            #[cfg(feature = "wayland")]
            surface_ids: HashMap::new(),
            #[cfg(feature = "wayland")]
            surface_names: HashMap::new(),
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
            watcher_opt: None,
            window_id_opt: Some(window::Id::MAIN),
            nav_dnd_hover: None,
            tab_dnd_hover: None,
            nav_drag_id: DragId::new(),
            tab_drag_id: DragId::new(),
        };

        let mut commands = vec![app.update_config()];

        for location in flags.locations {
            commands.push(app.open_tab(location, true, None));
        }

        if app.tab_model.iter().next().is_none() {
            if let Ok(current_dir) = env::current_dir() {
                commands.push(app.open_tab(Location::Path(current_dir), true, None));
            } else {
                commands.push(app.open_tab(Location::Path(home_dir()), true, None));
            }
        }

        (app, Command::batch(commands))
    }

    fn main_window_id(&self) -> window::Id {
        self.window_id_opt.unwrap_or(window::Id::MAIN)
    }

    fn nav_bar(&self) -> Option<Element<message::Message<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = cosmic::widget::nav_bar(nav_model, |entity| {
            cosmic::app::Message::Cosmic(cosmic::app::cosmic::Message::NavBar(entity))
        })
        .drag_id(self.nav_drag_id)
        .on_dnd_enter(|entity, _| cosmic::app::Message::App(Message::DndEnterNav(entity)))
        .on_dnd_leave(|_| cosmic::app::Message::App(Message::DndExitNav))
        .on_dnd_drop(|entity, data, action| {
            cosmic::app::Message::App(Message::DndDropNav(entity, data, action))
        })
        .on_context(|entity| cosmic::app::Message::App(Message::NavBarContext(entity)))
        .on_close(|entity| cosmic::app::Message::App(Message::NavBarClose(entity)))
        .on_middle_press(|entity| {
            cosmic::app::Message::App(Message::NavMenuAction(NavMenuAction::OpenInNewTab(entity)))
        })
        .context_menu(self.nav_context_menu(self.nav_bar_context_id))
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

    fn nav_context_menu(
        &self,
        id: widget::nav_bar::Id,
    ) -> Option<Vec<widget::menu::Tree<cosmic::app::Message<Self::Message>>>> {
        let maybe_trash_entity = self.nav_model.iter().find(|&entity| {
            self.nav_model
                .data::<Location>(entity)
                .map(|loc| *loc == Location::Trash)
                .unwrap_or_default()
        });
        let mut is_context_trash = false;
        if let Some(trash_id) = maybe_trash_entity {
            is_context_trash = trash_id == id;
        }
        Some(cosmic::widget::menu::items(
            &HashMap::new(),
            vec![
                cosmic::widget::menu::Item::Button(
                    fl!("open-in-new-tab"),
                    NavMenuAction::OpenInNewTab(id),
                ),
                cosmic::widget::menu::Item::Button(
                    fl!("open-in-new-window"),
                    NavMenuAction::OpenInNewWindow(id),
                ),
                cosmic::widget::menu::Item::Divider,
                cosmic::widget::menu::Item::Button(fl!("show-details"), NavMenuAction::Preview(id)),
                cosmic::widget::menu::Item::Divider,
                if is_context_trash {
                    cosmic::widget::menu::Item::Button(
                        fl!("empty-trash"),
                        NavMenuAction::EmptyTrash,
                    )
                } else {
                    cosmic::widget::menu::Item::Button(
                        fl!("remove-from-sidebar"),
                        NavMenuAction::RemoveFromSidebar(id),
                    )
                },
            ],
        ))
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        match self.mode {
            Mode::App => Some(&self.nav_model),
            Mode::Desktop => None,
        }
    }

    fn on_nav_select(&mut self, entity: Entity) -> Command<Self::Message> {
        self.search_active = false;
        self.search_input.clear();

        self.nav_model.activate(entity);
        if let Some(location) = self.nav_model.data::<Location>(entity) {
            let message = Message::TabMessage(None, tab::Message::Location(location.clone()));
            return self.update(message);
        }

        if let Some(data) = self.nav_model.data::<MounterData>(entity).clone() {
            if let Some(mounter) = self.mounters.get(&data.0) {
                return mounter.mount(data.1.clone()).map(|_| message::none());
            }
        }
        Command::none()
    }

    fn on_app_exit(&mut self) -> Option<Message> {
        Some(Message::WindowClose)
    }

    fn on_escape(&mut self) -> Command<Self::Message> {
        let entity = self.tab_model.active();

        // Close dialog if open
        if self.dialog_pages.pop_front().is_some() {
            return Command::none();
        }

        // Close gallery mode if open
        if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
            if tab.gallery {
                tab.gallery = false;
                return Command::none();
            }
        }

        // Close menus and context panes in order per message
        // Why: It'd be weird to close everything all at once
        // Usually, the Escape key (for example) closes menus and panes one by one instead
        // of closing everything on one press
        if self.core.window.show_context {
            self.set_show_context(false);
            return Command::none();
        }
        if self.search_active {
            // Close search if open
            self.search_active = false;
            return Command::none();
        }
        if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
            if tab.context_menu.is_some() {
                tab.context_menu = None;
                return Command::none();
            }

            let had_focused_button = tab.select_focus_id().is_some();
            if tab.select_none() {
                if had_focused_button {
                    // Unfocus if there was a focused button
                    return widget::button::focus(widget::Id::unique());
                }
                return Command::none();
            }
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
            Message::AddToSidebar(entity_opt) => {
                let mut favorites = self.config.favorites.clone();
                for path in self.selected_paths(entity_opt) {
                    favorites.push(Favorite::from_path(path));
                }
                config_set!(favorites, favorites);
                return self.update_config();
            }
            Message::AppTheme(app_theme) => {
                config_set!(app_theme, app_theme);
                return self.update_config();
            }
            Message::Compress(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                if let Some(current_path) = paths.first() {
                    if let Some(destination) = current_path.parent().zip(current_path.file_stem()) {
                        let to = destination.0.to_path_buf();
                        let name = destination.1.to_str().unwrap_or_default().to_string();
                        let archive_type = ArchiveType::default();
                        self.dialog_pages.push_back(DialogPage::Compress {
                            paths,
                            to,
                            name,
                            archive_type,
                        });
                        return widget::text_input::focus(self.dialog_text_input.clone());
                    }
                }
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
                let paths = self.selected_paths(entity_opt);
                let contents = ClipboardCopy::new(ClipboardKind::Copy, &paths);
                return clipboard::write_data(contents);
            }
            Message::Cut(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                let contents = ClipboardCopy::new(ClipboardKind::Cut, &paths);
                return clipboard::write_data(contents);
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
            Message::DialogCancel => {
                self.dialog_pages.pop_front();
            }
            Message::DialogComplete => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::Compress {
                            paths,
                            to,
                            name,
                            archive_type,
                        } => {
                            let extension = archive_type.extension();
                            let name = format!("{}{}", name, extension);
                            let to = to.join(name);
                            self.operation(Operation::Compress {
                                paths,
                                to,
                                archive_type,
                            })
                        }
                        DialogPage::EmptyTrash => {
                            self.operation(Operation::EmptyTrash);
                        }
                        DialogPage::FailedOperation(id) => {
                            log::warn!("TODO: retry operation {}", id);
                        }
                        DialogPage::NetworkAuth {
                            mounter_key,
                            uri,
                            auth,
                            auth_tx,
                        } => {
                            return Command::perform(
                                async move {
                                    auth_tx.send(auth).await.unwrap();
                                    message::none()
                                },
                                |x| x,
                            );
                        }
                        DialogPage::NetworkError {
                            mounter_key,
                            uri,
                            error,
                        } => {
                            //TODO: re-use mounter_key?
                            return Command::batch([
                                self.update(Message::NetworkDriveInput(uri)),
                                self.update(Message::NetworkDriveSubmit),
                            ]);
                        }
                        DialogPage::NewItem { parent, name, dir } => {
                            let path = parent.join(name);
                            self.operation(if dir {
                                Operation::NewFolder { path }
                            } else {
                                Operation::NewFile { path }
                            });
                        }
                        DialogPage::RenameItem {
                            from, parent, name, ..
                        } => {
                            let to = parent.join(name);
                            self.operation(Operation::Rename { from, to });
                        }
                        DialogPage::Replace { .. } => {
                            log::warn!("replace dialog should be completed with replace result");
                        }
                    }
                }
            }
            Message::DialogPush(dialog_page) => {
                self.dialog_pages.push_back(dialog_page);
            }
            Message::DialogUpdate(dialog_page) => {
                if !self.dialog_pages.is_empty() {
                    self.dialog_pages[0] = dialog_page;
                }
            }
            Message::DialogUpdateComplete(dialog_page) => {
                return Command::batch([
                    self.update(Message::DialogUpdate(dialog_page)),
                    self.update(Message::DialogComplete),
                ]);
            }
            Message::EditLocation(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(location) = self.tab_model.data::<Tab>(entity).and_then(|tab| {
                    if tab.edit_location.is_none() {
                        Some(tab.location.clone())
                    } else {
                        None
                    }
                }) {
                    return self.update(Message::TabMessage(
                        Some(entity),
                        tab::Message::EditLocation(Some(location)),
                    ));
                }
            }
            Message::ExtractHere(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                if let Some(current_path) = paths.get(0) {
                    if let Some(destination) = current_path.parent().zip(current_path.file_stem()) {
                        let destination_path = destination.0.to_path_buf();
                        self.operation(Operation::Extract {
                            paths,
                            to: destination_path,
                        });
                    }
                }
            }
            Message::Key(modifiers, key) => {
                let entity = self.tab_model.active();
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message(Some(entity)));
                    }
                }
            }
            Message::MaybeExit => {
                if self.window_id_opt.is_none() && self.pending_operations.is_empty() {
                    // Exit if window is closed and there are no pending operations
                    process::exit(0);
                }
            }
            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    log::warn!("failed to open {:?}: {}", url, err);
                }
            },
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::MoveToTrash(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                if !paths.is_empty() {
                    self.operation(Operation::Delete { paths });
                }
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
                    let entities: Vec<_> = self.tab_model.iter().collect();
                    for entity in entities {
                        let title_opt = match self.tab_model.data_mut::<Tab>(entity) {
                            Some(tab) => {
                                if unmounted.contains(&tab.location) {
                                    tab.change_location(&home_location, None);
                                    Some(tab.title())
                                } else {
                                    None
                                }
                            }
                            None => None,
                        };
                        if let Some(title) = title_opt {
                            self.tab_model.text_set(entity, title);
                            commands.push(self.rescan_tab(entity, home_location.clone(), None));
                        }
                    }
                    if !commands.is_empty() {
                        commands.push(self.update_title());
                        commands.push(self.update_watcher());
                    }
                }

                // Insert new items
                self.mounter_items.insert(mounter_key, mounter_items);

                // Update nav bar
                //TODO: this could change favorites IDs while they are in use
                self.update_nav_model();

                return Command::batch(commands);
            }
            Message::NetworkAuth(mounter_key, uri, auth, auth_tx) => {
                self.dialog_pages.push_back(DialogPage::NetworkAuth {
                    mounter_key,
                    uri,
                    auth,
                    auth_tx,
                });
            }
            Message::NetworkDriveInput(input) => {
                self.network_drive_input = input;
            }
            Message::NetworkDriveSubmit => {
                //TODO: know which mounter to use for network drives
                for (mounter_key, mounter) in self.mounters.iter() {
                    self.network_drive_connecting =
                        Some((*mounter_key, self.network_drive_input.clone()));
                    return mounter
                        .network_drive(self.network_drive_input.clone())
                        .map(|_| message::none());
                }
                log::warn!(
                    "no mounter found for connecting to {:?}",
                    self.network_drive_input
                );
            }
            Message::NetworkResult(mounter_key, uri, res) => {
                if self.network_drive_connecting == Some((mounter_key, uri.clone())) {
                    self.network_drive_connecting = None;
                }
                match res {
                    Ok(true) => {
                        log::info!("connected to {:?}", uri);
                        if matches!(self.context_page, ContextPage::NetworkDrive) {
                            self.set_show_context(false);
                        }
                    }
                    Ok(false) => {
                        log::info!("cancelled connection to {:?}", uri);
                    }
                    Err(error) => {
                        log::warn!("failed to connect to {:?}: {}", uri, error);
                        self.dialog_pages.push_back(DialogPage::NetworkError {
                            mounter_key,
                            uri,
                            error,
                        });
                    }
                }
            }
            Message::NewItem(entity_opt, dir) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Location::Path(path) = &tab.location {
                        self.dialog_pages.push_back(DialogPage::NewItem {
                            parent: path.clone(),
                            name: String::new(),
                            dir,
                        });
                        return widget::text_input::focus(self.dialog_text_input.clone());
                    }
                }
            }
            #[cfg(feature = "notify")]
            Message::Notification(notification) => {
                self.notification_opt = Some(notification);
            }
            Message::NotifyEvents(events) => {
                log::debug!("{:?}", events);

                let mut needs_reload = Vec::new();
                let entities: Vec<_> = self.tab_model.iter().collect();
                for entity in entities {
                    if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                        if let Location::Path(path) = &tab.location {
                            let mut contains_change = false;
                            for event in events.iter() {
                                for event_path in event.paths.iter() {
                                    if event_path.starts_with(&path) {
                                        match event.kind {
                                            notify::EventKind::Modify(
                                                notify::event::ModifyKind::Metadata(_),
                                            )
                                            | notify::EventKind::Modify(
                                                notify::event::ModifyKind::Data(_),
                                            ) => {
                                                // If metadata or data changed, find the matching item and reload it
                                                //TODO: this could be further optimized by looking at what exactly changed
                                                if let Some(items) = &mut tab.items_opt {
                                                    for item in items.iter_mut() {
                                                        if item.path_opt() == Some(event_path) {
                                                            //TODO: reload more, like mime types?
                                                            match fs::metadata(&event_path) {
                                                                Ok(new_metadata) => match &mut item
                                                                    .metadata
                                                                {
                                                                    ItemMetadata::Path {
                                                                        metadata,
                                                                        ..
                                                                    } => *metadata = new_metadata,
                                                                    _ => {}
                                                                },
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
                                needs_reload.push((entity, tab.location.clone()));
                            }
                        }
                    }
                }

                let mut commands = Vec::with_capacity(needs_reload.len());
                for (entity, location) in needs_reload {
                    commands.push(self.rescan_tab(entity, location, None));
                }
                return Command::batch(commands);
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
            Message::OpenTerminal(entity_opt) => {
                if let Some(terminal) = mime_app::terminal() {
                    let mut paths = Vec::new();
                    let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                    if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                        if let Location::Path(path) = &tab.location {
                            if let Some(items) = tab.items_opt() {
                                for item in items.iter() {
                                    if item.selected {
                                        if let Some(Location::Path(path)) = &item.location_opt {
                                            paths.push(path.clone());
                                        }
                                    }
                                }
                            }
                            if paths.is_empty() {
                                paths.push(path.clone());
                            }
                        }
                    }
                    for path in paths {
                        if let Some(mut command) = terminal.command(None) {
                            command.current_dir(&path);
                            match spawn_detached(&mut command) {
                                Ok(()) => {}
                                Err(err) => {
                                    log::warn!(
                                        "failed to open {:?} with terminal {:?}: {}",
                                        path,
                                        terminal.id,
                                        err
                                    )
                                }
                            }
                        } else {
                            log::warn!("failed to get command for {:?}", terminal.id);
                        }
                    }
                }
            }
            Message::OpenWith(path, app) => {
                if let Some(mut command) = app.command(Some(path.clone())) {
                    match spawn_detached(&mut command) {
                        Ok(()) => {
                            let _ = recently_used_xbel::update_recently_used(
                                &path,
                                App::APP_ID.to_string(),
                                "cosmic-files".to_string(),
                                None,
                            );
                        }
                        Err(err) => {
                            log::warn!("failed to open {:?} with {:?}: {}", path, app.id, err)
                        }
                    }
                } else {
                    log::warn!("failed to get command for {:?}", app.id);
                }

                // Close Open With context view
                self.set_show_context(false);
            }
            Message::OpenInNewTab(entity_opt) => {
                return Command::batch(self.selected_paths(entity_opt).into_iter().filter_map(
                    |path| {
                        if path.is_dir() {
                            Some(self.open_tab(Location::Path(path), false, None))
                        } else {
                            None
                        }
                    },
                ))
            }
            Message::OpenInNewWindow(entity_opt) => match env::current_exe() {
                Ok(exe) => self
                    .selected_paths(entity_opt)
                    .into_iter()
                    .filter(|p| p.is_dir())
                    .for_each(|path| match process::Command::new(&exe).arg(path).spawn() {
                        Ok(_child) => {}
                        Err(err) => {
                            log::error!("failed to execute {:?}: {}", exe, err);
                        }
                    }),
                Err(err) => {
                    log::error!("failed to get current executable path: {}", err);
                }
            },
            Message::OpenItemLocation(entity_opt) => {
                return Command::batch(self.selected_paths(entity_opt).into_iter().filter_map(
                    |path| {
                        if let Some(parent) = path.parent() {
                            Some(self.open_tab(
                                Location::Path(parent.to_path_buf()),
                                true,
                                Some(path),
                            ))
                        } else {
                            None
                        }
                    },
                ))
            }
            Message::Paste(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Location::Path(path) = &tab.location {
                        let to = path.clone();
                        return clipboard::read_data::<ClipboardPaste, _>(move |contents_opt| {
                            match contents_opt {
                                Some(contents) => {
                                    message::app(Message::PasteContents(to.clone(), contents))
                                }
                                None => message::none(),
                            }
                        });
                    }
                }
            }
            Message::PasteContents(to, mut contents) => {
                contents.paths.retain(|p| p != &to);
                if !contents.paths.is_empty() {
                    match contents.kind {
                        ClipboardKind::Copy => {
                            self.operation(Operation::Copy {
                                paths: contents.paths,
                                to,
                            });
                        }
                        ClipboardKind::Cut => {
                            //TODO: determine ability to move on non-Unix systems
                            let mut can_move = true;
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::MetadataExt;
                                //TODO: better error handling, fall back to not moving?
                                if let Ok(to_meta) = fs::metadata(&to) {
                                    for path in contents.paths.iter() {
                                        if let Ok(meta) = fs::metadata(path) {
                                            if meta.dev() != to_meta.dev() {
                                                can_move = false;
                                            }
                                        }
                                    }
                                }
                            }

                            if can_move {
                                self.operation(Operation::Move {
                                    paths: contents.paths,
                                    to,
                                });
                            } else {
                                self.operation(Operation::Copy {
                                    paths: contents.paths,
                                    to,
                                });
                            }
                        }
                    }
                }
            }
            Message::PendingComplete(id) => {
                let mut commands = Vec::with_capacity(3);

                if let Some((op, _)) = self.pending_operations.remove(&id) {
                    if let Some(description) = op.toast() {
                        if let Operation::Delete { ref paths } = op {
                            let paths: Arc<[PathBuf]> = Arc::from(paths.as_slice());
                            commands.push(
                                self.toasts
                                    .push(
                                        widget::toaster::Toast::new(description)
                                            .action(fl!("undo"), move |tid| {
                                                Message::UndoTrash(tid, paths.clone())
                                            }),
                                    )
                                    .map(cosmic::app::Message::App),
                            );
                        }
                    }
                    self.complete_operations.insert(id, op);
                }
                // Potentially show a notification
                commands.push(self.update_notification());
                // Manually rescan any trash tabs after any operation is completed
                commands.push(self.rescan_trash());

                // if search is active, update "search" tab view
                if !self.search_input.is_empty() {
                    commands.push(self.search());
                }
                return Command::batch(commands);
            }
            Message::PendingError(id, err) => {
                if let Some((op, _)) = self.pending_operations.remove(&id) {
                    self.failed_operations.insert(id, (op, err));
                    self.dialog_pages.push_back(DialogPage::FailedOperation(id));
                }
                // Manually rescan any trash tabs after any operation is completed
                return self.rescan_trash();
            }
            Message::PendingProgress(id, new_progress) => {
                if let Some((_, progress)) = self.pending_operations.get_mut(&id) {
                    *progress = new_progress;
                }
                return self.update_notification();
            }
            Message::Preview(entity, kind, timeout) => {
                if self
                    .preview_opt
                    .as_ref()
                    .is_some_and(|(e, k, i)| *e == entity && *k == kind && i.elapsed() > timeout)
                {
                    self.context_page = ContextPage::Preview(Some(entity), kind);
                    self.set_show_context(true);
                    self.set_context_title(self.context_page.title());
                }
            }
            Message::RescanTrash => {
                // Update trash icon if empty/full
                let maybe_entity = self.nav_model.iter().find(|&entity| {
                    self.nav_model
                        .data::<Location>(entity)
                        .map(|loc| *loc == Location::Trash)
                        .unwrap_or_default()
                });
                if let Some(entity) = maybe_entity {
                    self.nav_model
                        .icon_set(entity, widget::icon::icon(tab::trash_icon_symbolic(16)));
                }

                return self.rescan_trash();
            }

            Message::Rename(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Location::Path(parent) = &tab.location {
                        if let Some(items) = tab.items_opt() {
                            let mut selected = Vec::new();
                            for item in items.iter() {
                                if item.selected {
                                    if let Some(Location::Path(path)) = &item.location_opt {
                                        selected.push(path.clone());
                                    }
                                }
                            }
                            if !selected.is_empty() {
                                //TODO: batch rename
                                for path in selected {
                                    let name = match path.file_name().and_then(|x| x.to_str()) {
                                        Some(some) => some.to_string(),
                                        None => continue,
                                    };
                                    let dir = path.is_dir();
                                    self.dialog_pages.push_back(DialogPage::RenameItem {
                                        from: path,
                                        parent: parent.clone(),
                                        name,
                                        dir,
                                    });
                                }
                                return widget::text_input::focus(self.dialog_text_input.clone());
                            }
                        }
                    }
                }
            }
            Message::ReplaceResult(replace_result) => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::Replace { tx, .. } => {
                            return Command::perform(
                                async move {
                                    let _ = tx.send(replace_result).await;
                                    message::none()
                                },
                                |x| x,
                            );
                        }
                        other => {
                            log::warn!("tried to send replace result to the wrong dialog");
                            self.dialog_pages.push_front(other);
                        }
                    }
                }
            }
            Message::RestoreFromTrash(entity_opt) => {
                let mut paths = Vec::new();
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        for item in items.iter() {
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
            Message::SearchActivate => {
                self.search_active = true;
                return widget::text_input::focus(self.search_id.clone());
            }
            Message::SearchClear => {
                self.search_active = false;
                self.search_input.clear();
            }
            Message::SearchInput(input) => {
                if input != self.search_input {
                    self.search_input = input;
                    /*TODO: live search? (probably needs subscription for streaming results)
                    // This performs live search
                    if !self.search_input.is_empty() {
                        return self.search();
                    }
                    */
                }
            }
            Message::SearchSubmit => {
                if !self.search_input.is_empty() {
                    return self.search();
                } else {
                    // rescan the tab to get the contents back
                    // and exit search
                    self.search_active = false;
                    return self.search();
                }
            }
            Message::SystemThemeModeChange(_theme_mode) => {
                return self.update_config();
            }
            Message::TabActivate(entity) => {
                self.tab_model.activate(entity);

                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    self.activate_nav_model_location(&tab.location.clone());
                }
                let mut commands = vec![];
                commands.push(self.update_title());
                // if the tab was in an active search mode
                // search again in case files were modified/deleted
                if !self.search_input.is_empty() {
                    commands.push(self.search());
                }
                return Command::batch(commands);
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
                    let new_position = if position > 0 {
                        position - 1
                    } else {
                        position + 1
                    };

                    if self.tab_model.activate_position(new_position) {
                        if let Some(new_entity) = self.tab_model.entity_at(new_position) {
                            if let Some(tab) = self.tab_model.data::<Tab>(new_entity) {
                                self.activate_nav_model_location(&tab.location.clone());
                            }
                        }
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
            Message::TabConfig(config) => {
                if config != self.config.tab {
                    // Tabs are collected first to placate the borrowck
                    let tabs: Vec<_> = self.tab_model.iter().collect();
                    // Update main conf and each tab with the new config
                    let commands: Vec<_> = std::iter::once(self.update_config())
                        .chain(tabs.into_iter().map(|entity| {
                            let config = config.clone();
                            self.update(Message::TabMessage(
                                Some(entity),
                                tab::Message::Config(config),
                            ))
                        }))
                        .collect();

                    config_set!(tab, config);
                    return Command::batch(commands);
                }
            }
            Message::TabMessage(entity_opt, tab_message) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());

                //TODO: move to Command?
                if let tab::Message::ContextMenu(_point_opt) = tab_message {
                    // Disable side context page
                    self.set_show_context(false);
                }

                let tab_commands = match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => tab.update(tab_message, self.modifiers),
                    _ => Vec::new(),
                };

                let mut commands = Vec::new();
                for tab_command in tab_commands {
                    match tab_command {
                        tab::Command::Action(action) => {
                            commands.push(self.update(action.message(Some(entity))));
                        }
                        tab::Command::AddNetworkDrive => {
                            self.context_page = ContextPage::NetworkDrive;
                            self.set_show_context(true);
                            self.set_context_title(self.context_page.title());
                        }
                        tab::Command::ChangeLocation(tab_title, tab_path, selection_path) => {
                            self.activate_nav_model_location(&tab_path);

                            self.tab_model.text_set(entity, tab_title);
                            commands.push(Command::batch([
                                self.update_title(),
                                self.update_watcher(),
                                self.rescan_tab(entity, tab_path, selection_path),
                            ]));
                        }
                        tab::Command::DropFiles(to, from) => {
                            commands.push(self.update(Message::PasteContents(to, from)));
                        }
                        tab::Command::EmptyTrash => {
                            self.dialog_pages.push_back(DialogPage::EmptyTrash);
                        }
                        tab::Command::Iced(iced_command) => {
                            commands.push(iced_command.map(move |tab_message| {
                                message::app(Message::TabMessage(Some(entity), tab_message))
                            }));
                        }
                        tab::Command::MoveToTrash(paths) => {
                            self.operation(Operation::Delete { paths });
                        }
                        tab::Command::OpenFile(path) => {
                            let mut found_desktop_exec = false;
                            if mime_icon::mime_for_path(&path) == "application/x-desktop" {
                                match freedesktop_entry_parser::parse_entry(&path) {
                                    Ok(entry) => {
                                        match entry.section("Desktop Entry").attr("Exec") {
                                            Some(exec) => {
                                                match mime_app::exec_to_command(exec, None) {
                                                    Some(mut command) => {
                                                        match spawn_detached(&mut command) {
                                                            Ok(()) => {
                                                                found_desktop_exec = true;
                                                            }
                                                            Err(err) => {
                                                                log::warn!(
                                                                    "failed to execute {:?}: {}",
                                                                    path,
                                                                    err
                                                                );
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        log::warn!("failed to parse {:?}: invalid Desktop Entry/Exec", path);
                                                    }
                                                }
                                            }
                                            None => {
                                                log::warn!("failed to parse {:?}: missing Desktop Entry/Exec", path);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        log::warn!("failed to parse {:?}: {}", path, err);
                                    }
                                };
                            }
                            if !found_desktop_exec {
                                let file_extension = path.extension();
                                match file_extension {
                                    Some(ext) if ext == OsStr::new("AppImage") => {
                                        // Set the executable permission to the file
                                        let mut perms = fs::metadata(&path)
                                            .expect("Failed to get metadata")
                                            .permissions();
                                        // Set the executable permission to the file
                                        perms.set_mode(0o755);
                                        fs::set_permissions(&path, perms).expect("Failed to set permissions");

                                        log::info!("running app: {:?}", ext);
                                        let cmd = std::process::Command::new(path).spawn();
                                        match cmd {
                                            Ok(mut res) => {
                                                if let Some(stderr) = res.stderr.take() {
                                                    let reader = BufReader::new(stderr);
                                                    for line in reader.lines() {
                                                        if let Ok(line) = line {
                                                            log::error!(
                                                                "running app error: {}",
                                                                line
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                            Err(error) => {
                                                log::error!("error: {:?}", error);
                                            }
                                        }
                                    }
                                    _ => match open::that_detached(&path) {
                                        Ok(()) => {
                                            let _ = recently_used_xbel::update_recently_used(
                                                &path,
                                                App::APP_ID.to_string(),
                                                "cosmic-files".to_string(),
                                                None,
                                            );
                                        }
                                        Err(err) => {
                                            log::warn!("failed to open {:?}: {}", path, err);
                                        }
                                    },
                                }
                            }
                        }
                        tab::Command::OpenInNewTab(path) => {
                            commands.push(self.open_tab(Location::Path(path.clone()), false, None));
                        }
                        tab::Command::OpenInNewWindow(path) => match env::current_exe() {
                            Ok(exe) => match process::Command::new(&exe).arg(path).spawn() {
                                Ok(_child) => {}
                                Err(err) => {
                                    log::error!("failed to execute {:?}: {}", exe, err);
                                }
                            },
                            Err(err) => {
                                log::error!("failed to get current executable path: {}", err);
                            }
                        },
                        tab::Command::Preview(kind, mut timeout) => {
                            self.preview_opt = Some((entity, kind.clone(), Instant::now()));
                            if self.core.window.show_context {
                                // If the context window is already open, immediately show the preview
                                timeout = time::Duration::new(0, 0)
                            };
                            commands.push(Command::perform(
                                async move {
                                    tokio::time::sleep(timeout).await;
                                    message::app(Message::Preview(entity, kind, timeout))
                                },
                                |x| x,
                            ));
                        }
                        tab::Command::PreviewCancel => {
                            self.preview_opt = None;
                        }
                        tab::Command::WindowDrag => {
                            commands.push(window::drag(self.main_window_id()));
                        }
                        tab::Command::WindowToggleMaximize => {
                            commands.push(window::toggle_maximize(self.main_window_id()));
                        }
                    }
                }
                return Command::batch(commands);
            }
            Message::TabNew => {
                let active = self.tab_model.active();
                let location = match self.tab_model.data::<Tab>(active) {
                    Some(tab) => tab.location.clone(),
                    None => Location::Path(home_dir()),
                };
                return self.open_tab(location, true, None);
            }
            Message::TabRescan(entity, location, items, selection_path) => {
                match self.tab_model.data_mut::<Tab>(entity) {
                    Some(tab) => {
                        if location == tab.location {
                            tab.set_items(items);
                            if let Some(selection_path) = selection_path {
                                tab.select_path(selection_path);
                            }
                        }
                    }
                    _ => (),
                }
            }
            Message::ToggleContextPage(context_page) => {
                //TODO: ensure context menus are closed
                if self.context_page == context_page {
                    self.set_show_context(!self.core.window.show_context);
                } else {
                    self.set_show_context(true);
                }
                self.context_page = context_page;
                self.set_context_title(self.context_page.title());
            }
            Message::Undo(id) => {
                // TODO;
            }
            Message::UndoTrash(id, recently_trashed) => {
                self.toasts.remove(id);

                let mut paths = Vec::with_capacity(recently_trashed.len());
                let mounters = self.mounters.clone();
                let icon_sizes = self.config.tab.icon_sizes;

                return cosmic::command::future(async move {
                    match tokio::task::spawn_blocking(move || {
                        Location::Trash.scan(mounters, icon_sizes)
                    })
                    .await
                    {
                        Ok(items) => {
                            for path in &*recently_trashed {
                                for item in &items {
                                    if let ItemMetadata::Trash { ref entry, .. } = item.metadata {
                                        let original_path = entry.original_path();
                                        if &original_path == path {
                                            paths.push(entry.clone());
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("failed to rescan: {}", err);
                        }
                    }

                    Message::UndoTrashStart(paths)
                });
            }
            Message::UndoTrashStart(paths) => {
                self.operation(Operation::Restore { paths });
            }
            Message::WindowClose => {
                if let Some(window_id) = self.window_id_opt.take() {
                    return Command::batch([
                        window::close(window_id),
                        Command::perform(async move { message::app(Message::MaybeExit) }, |x| x),
                    ]);
                }
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
            Message::DndEnterNav(entity) => {
                if let Some(location) = self.nav_model.data::<Location>(entity) {
                    self.nav_dnd_hover = Some((location.clone(), Instant::now()));
                    let location = location.clone();
                    return Command::perform(tokio::time::sleep(HOVER_DURATION), move |_| {
                        cosmic::app::Message::App(Message::DndHoverLocTimeout(location))
                    });
                }
            }
            Message::DndExitNav => {
                self.nav_dnd_hover = None;
            }
            Message::DndDropNav(entity, data, action) => {
                self.nav_dnd_hover = None;
                if let Some((location, data)) = self.nav_model.data::<Location>(entity).zip(data) {
                    let kind = match action {
                        DndAction::Move => ClipboardKind::Cut,
                        _ => ClipboardKind::Copy,
                    };
                    let ret = match location {
                        Location::Path(p) => self.update(Message::PasteContents(
                            p.clone(),
                            ClipboardPaste {
                                kind,
                                paths: data.paths,
                            },
                        )),
                        Location::Trash if matches!(action, DndAction::Move) => {
                            self.operation(Operation::Delete { paths: data.paths });
                            Command::none()
                        }
                        _ => {
                            log::warn!("Copy to trash is not supported.");
                            Command::none()
                        }
                    };
                    return ret;
                }
            }
            Message::DndHoverLocTimeout(location) => {
                if self
                    .nav_dnd_hover
                    .as_ref()
                    .is_some_and(|(loc, i)| *loc == location && i.elapsed() >= HOVER_DURATION)
                {
                    self.nav_dnd_hover = None;
                    let entity = self.tab_model.active();
                    let title_opt = match self.tab_model.data_mut::<Tab>(entity) {
                        Some(tab) => {
                            tab.change_location(&location, None);
                            Some(tab.title())
                        }
                        None => None,
                    };
                    if let Some(title) = title_opt {
                        self.tab_model.text_set(entity, title);
                        return Command::batch([
                            self.update_title(),
                            self.update_watcher(),
                            self.rescan_tab(entity, location, None),
                        ]);
                    }
                }
            }
            Message::DndEnterTab(entity) => {
                self.tab_dnd_hover = Some((entity, Instant::now()));
                return Command::perform(tokio::time::sleep(HOVER_DURATION), move |_| {
                    cosmic::app::Message::App(Message::DndHoverTabTimeout(entity))
                });
            }
            Message::DndExitTab => {
                self.nav_dnd_hover = None;
            }
            Message::DndDropTab(entity, data, action) => {
                self.nav_dnd_hover = None;
                if let Some((tab, data)) = self.tab_model.data::<Tab>(entity).zip(data) {
                    let kind = match action {
                        DndAction::Move => ClipboardKind::Cut,
                        _ => ClipboardKind::Copy,
                    };
                    let ret = match &tab.location {
                        Location::Path(p) => self.update(Message::PasteContents(
                            p.clone(),
                            ClipboardPaste {
                                kind,
                                paths: data.paths,
                            },
                        )),
                        Location::Trash if matches!(action, DndAction::Move) => {
                            self.operation(Operation::Delete { paths: data.paths });
                            Command::none()
                        }
                        _ => {
                            log::warn!("Copy to trash is not supported.");
                            Command::none()
                        }
                    };
                    return ret;
                }
            }
            Message::DndHoverTabTimeout(entity) => {
                if self
                    .tab_dnd_hover
                    .as_ref()
                    .is_some_and(|(e, i)| *e == entity && i.elapsed() >= HOVER_DURATION)
                {
                    self.tab_dnd_hover = None;
                    return self.update(Message::TabActivate(entity));
                }
            }

            Message::NavBarClose(entity) => {
                if let Some(data) = self.nav_model.data::<MounterData>(entity) {
                    if let Some(mounter) = self.mounters.get(&data.0) {
                        return mounter.unmount(data.1.clone()).map(|_| message::none());
                    }
                }
            }

            // Tracks which nav bar item to show a context menu for.
            Message::NavBarContext(entity) => {
                self.nav_bar_context_id = entity;
            }

            // Applies selected nav bar context menu operation.
            Message::NavMenuAction(action) => match action {
                NavMenuAction::OpenInNewTab(entity) => {
                    match self.nav_model.data::<Location>(entity) {
                        Some(Location::Path(ref path)) => {
                            return self.open_tab(Location::Path(path.clone()), false, None);
                        }
                        Some(Location::Trash) => {
                            return self.open_tab(Location::Trash, false, None);
                        }
                        _ => {}
                    }
                }

                // Open the selected path in a new cosmic-files window.
                NavMenuAction::OpenInNewWindow(entity) => {
                    if let Some(&Location::Path(ref path)) = self.nav_model.data::<Location>(entity)
                    {
                        match env::current_exe() {
                            Ok(exe) => match process::Command::new(&exe).arg(path).spawn() {
                                Ok(_child) => {}
                                Err(err) => {
                                    log::error!("failed to execute {:?}: {}", exe, err);
                                }
                            },
                            Err(err) => {
                                log::error!("failed to get current executable path: {}", err);
                            }
                        }
                    }
                }

                NavMenuAction::Preview(entity) => {
                    if let Some(Location::Path(path)) = self.nav_model.data::<Location>(entity) {
                        match tab::item_from_path(path, IconSizes::default()) {
                            Ok(item) => {
                                self.context_page = ContextPage::Preview(
                                    None,
                                    PreviewKind::Custom(PreviewItem(item)),
                                );
                                self.set_show_context(true);
                                self.set_context_title(self.context_page.title());
                            }
                            Err(err) => {
                                log::warn!("failed to get item from path {:?}: {}", path, err);
                            }
                        }
                    }
                }

                NavMenuAction::RemoveFromSidebar(entity) => {
                    if let Some(FavoriteIndex(favorite_i)) =
                        self.nav_model.data::<FavoriteIndex>(entity)
                    {
                        let mut favorites = self.config.favorites.clone();
                        favorites.remove(*favorite_i);
                        config_set!(favorites, favorites);
                        return self.update_config();
                    }
                }

                NavMenuAction::EmptyTrash => {
                    self.dialog_pages.push_front(DialogPage::EmptyTrash);
                }
            },
            Message::Recents => {
                return self.open_tab(Location::Recents, false, None);
            }
            #[cfg(feature = "wayland")]
            Message::OutputEvent(output_event, output) => {
                match output_event {
                    OutputEvent::Created(output_info_opt) => {
                        log::info!("output {}: created", output.id());

                        let surface_id = WindowId::unique();
                        match self.surface_ids.insert(output.clone(), surface_id) {
                            Some(old_surface_id) => {
                                //TODO: remove old surface?
                                log::warn!(
                                    "output {}: already had surface ID {:?}",
                                    output.id(),
                                    old_surface_id
                                );
                            }
                            None => {}
                        }

                        match output_info_opt {
                            Some(output_info) => match output_info.name {
                                Some(output_name) => {
                                    self.surface_names.insert(surface_id, output_name.clone());
                                }
                                None => {
                                    log::warn!("output {}: no output name", output.id());
                                }
                            },
                            None => {
                                log::warn!("output {}: no output info", output.id());
                            }
                        }

                        return Command::batch([get_layer_surface(SctkLayerSurfaceSettings {
                            id: surface_id,
                            layer: Layer::Bottom,
                            keyboard_interactivity: KeyboardInteractivity::OnDemand,
                            pointer_interactivity: true,
                            anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                            output: IcedOutput::Output(output),
                            namespace: "cosmic-files-applet".into(),
                            size: Some((None, None)),
                            margin: IcedMargin {
                                top: 0,
                                bottom: 0,
                                left: 0,
                                right: 0,
                            },
                            exclusive_zone: -1,
                            size_limits: Limits::NONE.min_width(1.0).min_height(1.0),
                        })]);
                    }
                    OutputEvent::Removed => {
                        log::info!("output {}: removed", output.id());
                        match self.surface_ids.remove(&output) {
                            Some(surface_id) => {
                                self.surface_names.remove(&surface_id);
                                return destroy_layer_surface(surface_id);
                            }
                            None => {
                                log::warn!("output {}: no surface found", output.id());
                            }
                        }
                    }
                    OutputEvent::InfoUpdate(_output_info) => {
                        log::info!("output {}: info update", output.id());
                    }
                }
            }
            Message::Cosmic(cosmic) => {
                // Forward cosmic messages
                return Command::perform(async move { cosmic }, |cosmic| message::cosmic(cosmic));
            }
            Message::None => {}
        }

        Command::none()
    }

    fn context_drawer(&self) -> Option<Element<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match &self.context_page {
            ContextPage::About => self.about(),
            ContextPage::EditHistory => self.edit_history(),
            ContextPage::NetworkDrive => self.network_drive(),
            ContextPage::OpenWith => self.open_with(),
            ContextPage::Preview(entity_opt, kind) => self.preview(entity_opt, kind),
            ContextPage::Settings => self.settings(),
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        //TODO: should gallery view just be a dialog?
        let entity = self.tab_model.active();
        if let Some(tab) = self.tab_model.data::<Tab>(entity) {
            if tab.gallery {
                return Some(
                    tab.gallery_view()
                        .map(move |tab_message| Message::TabMessage(Some(entity), tab_message)),
                );
            }
        }

        let dialog_page = match self.dialog_pages.front() {
            Some(some) => some,
            None => return None,
        };

        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;

        let dialog = match dialog_page {
            DialogPage::Compress {
                paths,
                to,
                name,
                archive_type,
            } => {
                let mut dialog = widget::dialog(fl!("create-archive"));

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
                    let extension = archive_type.extension();
                    let name = format!("{}{}", name, extension);
                    let path = to.join(&name);
                    if path.exists() {
                        dialog =
                            dialog.tertiary_action(widget::text::body(fl!("file-already-exists")));
                        None
                    } else {
                        if name.starts_with('.') {
                            dialog = dialog.tertiary_action(widget::text::body(fl!("name-hidden")));
                        }
                        Some(Message::DialogComplete)
                    }
                };

                let archive_types = ArchiveType::all();
                let selected = archive_types.iter().position(|&x| x == *archive_type);
                dialog
                    .primary_action(
                        widget::button::suggested(fl!("create"))
                            .on_press_maybe(complete_maybe.clone()),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(
                        widget::column::with_children(vec![
                            widget::text::body(fl!("file-name")).into(),
                            widget::row::with_children(vec![
                                widget::text_input("", name.as_str())
                                    .id(self.dialog_text_input.clone())
                                    .on_input(move |name| {
                                        Message::DialogUpdate(DialogPage::Compress {
                                            paths: paths.clone(),
                                            to: to.clone(),
                                            name: name.clone(),
                                            archive_type: *archive_type,
                                        })
                                    })
                                    .on_submit_maybe(complete_maybe)
                                    .into(),
                                widget::dropdown(archive_types, selected, move |index| {
                                    Message::DialogUpdate(DialogPage::Compress {
                                        paths: paths.clone(),
                                        to: to.clone(),
                                        name: name.clone(),
                                        archive_type: archive_types[index],
                                    })
                                })
                                .into(),
                            ])
                            .align_items(Alignment::Center)
                            .spacing(space_xxs)
                            .into(),
                        ])
                        .spacing(space_xxs),
                    )
            }
            DialogPage::EmptyTrash => widget::dialog(fl!("empty-trash"))
                .body(fl!("empty-trash-warning"))
                .primary_action(
                    widget::button::suggested(fl!("empty-trash")).on_press(Message::DialogComplete),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),
            DialogPage::FailedOperation(id) => {
                //TODO: try next dialog page (making sure index is used by Dialog messages)?
                let (operation, err) = self.failed_operations.get(id)?;

                //TODO: nice description of error
                widget::dialog("Failed operation")
                    .body(format!("{:#?}\n{}", operation, err))
                    .icon(widget::icon::from_name("dialog-error").size(64))
                    //TODO: retry action
                    .primary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
            }
            DialogPage::NetworkAuth {
                mounter_key,
                uri,
                auth,
                auth_tx,
            } => {
                //TODO: use URI!
                let mut controls = Vec::with_capacity(4);
                if let Some(username) = &auth.username_opt {
                    //TODO: what should submit do?
                    controls.push(
                        widget::text_input(fl!("username"), username)
                            .on_input(move |value| {
                                Message::DialogUpdate(DialogPage::NetworkAuth {
                                    mounter_key: *mounter_key,
                                    uri: uri.clone(),
                                    auth: MounterAuth {
                                        username_opt: Some(value),
                                        ..auth.clone()
                                    },
                                    auth_tx: auth_tx.clone(),
                                })
                            })
                            .into(),
                    );
                }
                if let Some(domain) = &auth.domain_opt {
                    //TODO: what should submit do?
                    controls.push(
                        widget::text_input(fl!("domain"), domain)
                            .on_input(move |value| {
                                Message::DialogUpdate(DialogPage::NetworkAuth {
                                    mounter_key: *mounter_key,
                                    uri: uri.clone(),
                                    auth: MounterAuth {
                                        domain_opt: Some(value),
                                        ..auth.clone()
                                    },
                                    auth_tx: auth_tx.clone(),
                                })
                            })
                            .into(),
                    );
                }
                if let Some(password) = &auth.password_opt {
                    //TODO: what should submit do?
                    //TODO: button for showing password
                    controls.push(
                        widget::secure_input(fl!("password"), password, None, true)
                            .on_input(move |value| {
                                Message::DialogUpdate(DialogPage::NetworkAuth {
                                    mounter_key: *mounter_key,
                                    uri: uri.clone(),
                                    auth: MounterAuth {
                                        password_opt: Some(value),
                                        ..auth.clone()
                                    },
                                    auth_tx: auth_tx.clone(),
                                })
                            })
                            .into(),
                    );
                }
                if let Some(remember) = &auth.remember_opt {
                    //TODO: what should submit do?
                    //TODO: button for showing password
                    controls.push(
                        widget::checkbox(fl!("remember-password"), *remember, move |value| {
                            Message::DialogUpdate(DialogPage::NetworkAuth {
                                mounter_key: *mounter_key,
                                uri: uri.clone(),
                                auth: MounterAuth {
                                    remember_opt: Some(value),
                                    ..auth.clone()
                                },
                                auth_tx: auth_tx.clone(),
                            })
                        })
                        .into(),
                    );
                }

                let mut parts = auth.message.splitn(2, '\n');
                let title = parts.next().unwrap_or_default();
                let body = parts.next().unwrap_or_default();
                widget::dialog(title)
                    .body(body)
                    .control(widget::column::with_children(controls).spacing(space_s))
                    .primary_action(
                        widget::button::suggested(fl!("connect")).on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .tertiary_action(widget::button::text(fl!("connect-anonymously")).on_press(
                        Message::DialogUpdateComplete(DialogPage::NetworkAuth {
                            mounter_key: *mounter_key,
                            uri: uri.clone(),
                            auth: MounterAuth {
                                anonymous_opt: Some(true),
                                ..auth.clone()
                            },
                            auth_tx: auth_tx.clone(),
                        }),
                    ))
            }
            DialogPage::NetworkError {
                mounter_key,
                uri,
                error,
            } => widget::dialog(fl!("network-drive-error"))
                .body(error)
                .icon(widget::icon::from_name("dialog-error").size(64))
                .primary_action(
                    widget::button::standard(fl!("try-again")).on_press(Message::DialogComplete),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),
            DialogPage::NewItem { parent, name, dir } => {
                let mut dialog = widget::dialog(if *dir {
                    fl!("create-new-folder")
                } else {
                    fl!("create-new-file")
                });

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
                            widget::text::body(if *dir {
                                fl!("folder-name")
                            } else {
                                fl!("file-name")
                            })
                            .into(),
                            widget::text_input("", name.as_str())
                                .id(self.dialog_text_input.clone())
                                .on_input(move |name| {
                                    Message::DialogUpdate(DialogPage::NewItem {
                                        parent: parent.clone(),
                                        name,
                                        dir: *dir,
                                    })
                                })
                                .on_submit_maybe(complete_maybe)
                                .into(),
                        ])
                        .spacing(space_xxs),
                    )
            }
            DialogPage::RenameItem {
                from,
                parent,
                name,
                dir,
            } => {
                //TODO: combine logic with NewItem
                let mut dialog = widget::dialog(if *dir {
                    fl!("rename-folder")
                } else {
                    fl!("rename-file")
                });

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
                        widget::button::suggested(fl!("rename"))
                            .on_press_maybe(complete_maybe.clone()),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(
                        widget::column::with_children(vec![
                            widget::text::body(if *dir {
                                fl!("folder-name")
                            } else {
                                fl!("file-name")
                            })
                            .into(),
                            widget::text_input("", name.as_str())
                                .id(self.dialog_text_input.clone())
                                .on_input(move |name| {
                                    Message::DialogUpdate(DialogPage::RenameItem {
                                        from: from.clone(),
                                        parent: parent.clone(),
                                        name,
                                        dir: *dir,
                                    })
                                })
                                .on_submit_maybe(complete_maybe)
                                .into(),
                        ])
                        .spacing(space_xxs),
                    )
            }
            DialogPage::Replace {
                from,
                to,
                multiple,
                apply_to_all,
                tx,
            } => {
                let dialog = widget::dialog(fl!("replace-title", filename = to.name.as_str()))
                    .body(fl!("replace-warning-operation"))
                    .control(to.replace_view(fl!("original-file"), IconSizes::default()))
                    .control(from.replace_view(fl!("replace-with"), IconSizes::default()))
                    .primary_action(widget::button::suggested(fl!("replace")).on_press(
                        Message::ReplaceResult(ReplaceResult::Replace(*apply_to_all)),
                    ));
                if *multiple {
                    dialog
                        .control(widget::checkbox(
                            fl!("apply-to-all"),
                            *apply_to_all,
                            |apply_to_all| {
                                Message::DialogUpdate(DialogPage::Replace {
                                    from: from.clone(),
                                    to: to.clone(),
                                    multiple: *multiple,
                                    apply_to_all,
                                    tx: tx.clone(),
                                })
                            },
                        ))
                        .secondary_action(
                            widget::button::standard(fl!("skip")).on_press(Message::ReplaceResult(
                                ReplaceResult::Skip(*apply_to_all),
                            )),
                        )
                        .tertiary_action(
                            widget::button::text(fl!("cancel"))
                                .on_press(Message::ReplaceResult(ReplaceResult::Cancel)),
                        )
                } else {
                    dialog
                        .secondary_action(
                            widget::button::standard(fl!("cancel"))
                                .on_press(Message::ReplaceResult(ReplaceResult::Cancel)),
                        )
                        .tertiary_action(
                            widget::button::text(fl!("keep-both"))
                                .on_press(Message::ReplaceResult(ReplaceResult::KeepBoth)),
                        )
                }
            }
        };

        Some(dialog.into())
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(self.tab_model.active_data::<Tab>(), &self.key_binds).into()]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        let mut elements = Vec::with_capacity(2);

        if !self.pending_operations.is_empty() {
            elements.push(
                widget::button::text(format!("{}", self.pending_operations.len()))
                    .on_press(Message::ToggleContextPage(ContextPage::EditHistory))
                    .into(),
            );
        }

        if self.search_active {
            elements.push(
                widget::text_input::search_input("", &self.search_input)
                    .width(Length::Fixed(240.0))
                    .id(self.search_id.clone())
                    .on_clear(Message::SearchClear)
                    .on_input(Message::SearchInput)
                    .on_submit(Message::SearchSubmit)
                    .into(),
            )
        } else {
            elements.push(
                widget::button::icon(widget::icon::from_name("system-search-symbolic"))
                    .padding(8)
                    .on_press(Message::SearchActivate)
                    .into(),
            )
        }

        elements
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;

        let mut tab_column = widget::column::with_capacity(3);

        if self.tab_model.iter().count() > 1 {
            tab_column = tab_column.push(
                widget::container(
                    widget::tab_bar::horizontal(&self.tab_model)
                        .button_height(32)
                        .button_spacing(space_xxs)
                        .on_activate(Message::TabActivate)
                        .on_close(|entity| Message::TabClose(Some(entity)))
                        .on_dnd_enter(|entity, _| Message::DndEnterTab(entity))
                        .on_dnd_leave(|_| Message::DndExitTab)
                        .on_dnd_drop(|entity, data, action| {
                            Message::DndDropTab(entity, data, action)
                        })
                        .drag_id(self.tab_drag_id),
                )
                .style(style::Container::Background)
                .width(Length::Fill)
                .padding([0, space_s]),
            );
        }

        let entity = self.tab_model.active();
        match self.tab_model.data::<Tab>(entity) {
            Some(tab) => {
                let tab_view = tab
                    .view(&self.key_binds)
                    .map(move |message| Message::TabMessage(Some(entity), message));
                tab_column = tab_column.push(tab_view);
            }
            None => {
                //TODO
            }
        }

        // The toaster is added on top of an empty element to ensure that it does not override context menus
        tab_column = tab_column.push(widget::toaster(
            &self.toasts,
            widget::horizontal_space(Length::Fill),
        ));

        let content: Element<_> = tab_column.into();

        // Uncomment to debug layout:
        //content.explain(cosmic::iced::Color::WHITE)
        content
    }

    fn view_window(&self, id: WindowId) -> Element<Self::Message> {
        //TODO: distinct views per window?
        self.view_main().map(|message| match message {
            app::Message::App(app) => app,
            app::Message::Cosmic(cosmic) => Message::Cosmic(cosmic),
            app::Message::None => Message::None,
        })
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ThemeSubscription;
        struct WatcherSubscription;
        struct TrashWatcherSubscription;

        let mut subscriptions = vec![
            event::listen_with(|event, status| match event {
                Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                Event::Window(_id, WindowEvent::CloseRequested) => Some(Message::WindowClose),
                #[cfg(feature = "wayland")]
                Event::PlatformSpecific(event::PlatformSpecific::Wayland(wayland_event)) => {
                    match wayland_event {
                        WaylandEvent::Output(output_event, output) => {
                            Some(Message::OutputEvent(output_event, output))
                        }
                        _ => None,
                    }
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
                        new_debouncer(
                            time::Duration::from_millis(250),
                            Some(time::Duration::from_millis(250)),
                            move |events_res: notify_debouncer_full::DebounceEventResult| {
                                match events_res {
                                    Ok(mut events) => {
                                        log::debug!("{:?}", events);

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
            subscription::channel(
                TypeId::of::<TrashWatcherSubscription>(),
                25,
                |mut output| async move {
                    let watcher_res = new_debouncer(
                        time::Duration::from_millis(250),
                        Some(time::Duration::from_millis(250)),
                        move |event_res: notify_debouncer_full::DebounceEventResult| match event_res
                        {
                            Ok(mut events) => {
                                events.retain(|event| {
                                    matches!(
                                        event.kind,
                                        notify::EventKind::Create(_) | notify::EventKind::Remove(_)
                                    )
                                });

                                if !events.is_empty() {
                                    if let Err(e) = futures::executor::block_on(async {
                                        output.send(Message::RescanTrash).await
                                    }) {
                                        log::warn!("trash needs to be rescanned but sending message failed: {e:?}");
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("failed to watch trash bin for changes: {e:?}")
                            }
                        },
                    );

                    // TODO: Trash watching support for Windows, macOS, and other OSes
                    #[cfg(all(
                        unix,
                        not(target_os = "macos"),
                        not(target_os = "ios"),
                        not(target_os = "android")
                    ))]
                    match (watcher_res, trash::os_limited::trash_folders()) {
                        (Ok(mut watcher), Ok(trash_bins)) => {
                            for path in trash_bins {
                                if let Err(e) = watcher
                                    .watcher()
                                    .watch(&path, notify::RecursiveMode::Recursive)
                                {
                                    log::warn!(
                                        "failed to add trash bin `{}` to watcher: {e:?}",
                                        path.display()
                                    );
                                }
                            }

                            // Don't drop the watcher
                            std::future::pending().await
                        }
                        (Err(e), _) => {
                            log::warn!("failed to create new watcher for trash bin: {e:?}")
                        }
                        (_, Err(e)) => {
                            log::warn!("could not find any valid trash bins to watch: {e:?}")
                        }
                    }

                    std::future::pending().await
                },
            ),
        ];

        for (key, mounter) in self.mounters.iter() {
            let key = *key;
            subscriptions.push(mounter.subscription().map(move |mounter_message| {
                match mounter_message {
                    MounterMessage::Items(items) => Message::MounterItems(key, items),
                    MounterMessage::NetworkAuth(uri, auth, auth_tx) => {
                        Message::NetworkAuth(key, uri, auth, auth_tx)
                    }
                    MounterMessage::NetworkResult(uri, res) => {
                        Message::NetworkResult(key, uri, res)
                    }
                }
            }));
        }

        if !self.pending_operations.is_empty() {
            //TODO: inhibit suspend/shutdown?

            if self.window_id_opt.is_none() {
                #[cfg(feature = "notify")]
                {
                    struct NotificationSubscription;
                    subscriptions.push(subscription::channel(
                        TypeId::of::<NotificationSubscription>(),
                        1,
                        move |msg_tx| async move {
                            let msg_tx = Arc::new(tokio::sync::Mutex::new(msg_tx));
                            tokio::task::spawn_blocking(move || {
                                match notify_rust::Notification::new()
                                    .summary(&fl!("notification-in-progress"))
                                    .timeout(notify_rust::Timeout::Never)
                                    .show()
                                {
                                    Ok(notification) => {
                                        let _ = futures::executor::block_on(async {
                                            msg_tx
                                                .lock()
                                                .await
                                                .send(Message::Notification(Arc::new(Mutex::new(
                                                    notification,
                                                ))))
                                                .await
                                        });
                                    }
                                    Err(err) => {
                                        log::warn!("failed to create notification: {}", err);
                                    }
                                }
                            })
                            .await
                            .unwrap();

                            pending().await
                        },
                    ));
                }
            }
        }

        for (id, (pending_operation, _)) in self.pending_operations.iter() {
            //TODO: use recipe?
            let id = *id;
            let pending_operation = pending_operation.clone();
            subscriptions.push(subscription::channel(id, 16, move |msg_tx| async move {
                let msg_tx = Arc::new(tokio::sync::Mutex::new(msg_tx));
                match pending_operation.perform(id, &msg_tx).await {
                    Ok(()) => {
                        let _ = msg_tx.lock().await.send(Message::PendingComplete(id)).await;
                    }
                    Err(err) => {
                        let _ = msg_tx
                            .lock()
                            .await
                            .send(Message::PendingError(id, err.to_string()))
                            .await;
                    }
                }

                std::future::pending().await
            }));
        }

        for entity in self.tab_model.iter() {
            if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                subscriptions.push(
                    tab.subscription()
                        .with(entity)
                        .map(|(entity, tab_msg)| Message::TabMessage(Some(entity), tab_msg)),
                );
            }
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

    use crate::{
        config::{IconSizes, TabConfig},
        mounter::MounterMap,
        tab::Item,
    };

    use super::*;

    // Default number of files, directories, and nested directories for test file system
    pub const NUM_FILES: usize = 2;
    pub const NUM_HIDDEN: usize = 1;
    pub const NUM_DIRS: usize = 2;
    pub const NUM_NESTED: usize = 1;
    pub const NAME_LEN: usize = 5;

    /// Add `n` temporary files in `dir`
    ///
    /// Each file is assigned a numeric name from [0, n) with a prefix.
    pub fn file_flat_hier<D: AsRef<Path>>(dir: D, n: usize, prefix: &str) -> io::Result<Vec<File>> {
        let dir = dir.as_ref();
        (0..n)
            .map(|i| -> io::Result<File> {
                let name = format!("{prefix}{i}");
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
    ///
    /// # Arguments
    ///
    /// * `files` - Number of files to create in temp directories
    /// * `hidden` - Number of hidden files to create
    /// * `dirs` - Number of directories to create
    /// * `nested` - Number of nested directories to create in new dirs
    /// * `name_len` - Length of randomized directory names
    pub fn simple_fs(
        files: usize,
        hidden: usize,
        dirs: usize,
        nested: usize,
        name_len: usize,
    ) -> io::Result<TempDir> {
        // Files created inside of a TempDir are deleted with the directory
        // TempDir won't leak resources as long as the destructor runs
        let root = tempdir()?;
        debug!("Root temp directory: {}", root.as_ref().display());
        trace!("Creating {files} files and {hidden} hidden files in {dirs} temp dirs with {nested} nested temp dirs");

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

            // Normal files
            file_flat_hier(&path, files, "")?;
            // Hidden files
            file_flat_hier(&path, hidden, ".")?;

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
            _ => LANGUAGE_SORTER.compare(
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

    /// Read directory entries from `path` and sort.
    pub fn read_dir_sorted(path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut entries: Vec<_> = path
            .read_dir()?
            .map(|maybe_entry| maybe_entry.map(|entry| entry.path()))
            .collect::<io::Result<_>>()?;
        entries.sort_by(|a, b| sort_files(a, b));

        Ok(entries)
    }

    /// Filter `path` for directories
    pub fn filter_dirs(path: &Path) -> io::Result<impl Iterator<Item = PathBuf>> {
        Ok(path.read_dir()?.filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    Some(path)
                } else {
                    None
                }
            })
        }))
    }

    // Filter `path` for files
    pub fn filter_files(path: &Path) -> io::Result<impl Iterator<Item = PathBuf>> {
        Ok(path.read_dir()?.filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                path.is_file().then_some(path)
            })
        }))
    }

    /// Boiler plate for Tab tests
    pub fn tab_click_new(
        files: usize,
        hidden: usize,
        dirs: usize,
        nested: usize,
        name_len: usize,
    ) -> io::Result<(TempDir, Tab)> {
        let fs = simple_fs(files, hidden, dirs, nested, name_len)?;
        let path = fs.path();

        // New tab with items
        let location = Location::Path(path.to_owned());
        let items = location.scan(Mounters::new(MounterMap::new()), IconSizes::default());
        let mut tab = Tab::new(location, TabConfig::default());
        tab.set_items(items);

        // Ensure correct number of directories as a sanity check
        let items = tab.items_opt().expect("tab should be populated with Items");
        assert_eq!(NUM_DIRS, items.len());

        Ok((fs, tab))
    }

    /// Equality for [Path] and [Item].
    pub fn eq_path_item(path: &Path, item: &Item) -> bool {
        let name = path
            .file_name()
            .expect("temp entries should have names")
            .to_str()
            .expect("temp entries should be valid UTF-8");
        let is_dir = path.is_dir();

        // NOTE: I don't want to change `tab::hidden_attribute` to `pub(crate)` for
        // tests without asking
        #[cfg(not(target_os = "windows"))]
        let is_hidden = name.starts_with('.');

        #[cfg(target_os = "windows")]
        let is_hidden = {
            use std::os::windows::fs::MetadataExt;
            const FILE_ATTRIBUTE_HIDDEN: u32 = 2;
            let metadata = path.metadata().expect("fetching file metadata");
            metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN == FILE_ATTRIBUTE_HIDDEN
        };

        name == item.name
            && is_dir == item.metadata.is_dir()
            && path == item.path_opt().expect("item should have path")
            && is_hidden == item.hidden
    }

    /// Asserts `tab`'s location changed to `path`
    pub fn assert_eq_tab_path(tab: &Tab, path: &Path) {
        // Paths should be the same
        let Location::Path(ref tab_path) = tab.location else {
            panic!("Expected tab's location to be a path");
        };

        assert_eq!(
            path,
            tab_path,
            "Tab's path is {} instead of being updated to {}",
            tab_path.display(),
            path.display()
        );
    }

    pub fn assert_zoom_affects_item_size(tab: &mut Tab, message: tab::Message, should_zoom: bool) {
        let grid_icon_size = tab.config.icon_sizes.grid;
        let list_icon_size = tab.config.icon_sizes.list;

        debug!("Emitting {:?}", message);
        tab.update(message, Modifiers::empty());

        let grid_size_changed = grid_icon_size != tab.config.icon_sizes.grid;
        let list_size_changed = list_icon_size != tab.config.icon_sizes.list;

        assert_eq!(grid_size_changed || list_size_changed, should_zoom);
    }

    /// Assert that tab's items are equal to a path's entries.
    pub fn assert_eq_tab_path_contents(tab: &Tab, path: &Path) {
        let Location::Path(ref tab_path) = tab.location else {
            panic!("Expected tab's location to be a path");
        };

        // Tab items are sorted so paths from read_dir must be too
        let entries = read_dir_sorted(path).expect("should be able to read paths from temp dir");

        // Check lengths.
        // `items_opt` is optional and the directory at `path` may have zero entries
        // Therefore, this doesn't panic if `items_opt` is None
        let items_len = tab.items_opt().map(|items| items.len()).unwrap_or_default();
        assert_eq!(entries.len(), items_len);

        let empty = Vec::new();
        assert!(
            entries
                .into_iter()
                .zip(tab.items_opt().clone().unwrap_or(&empty))
                .all(|(a, b)| eq_path_item(&a, &b)),
            "Path ({}) and Tab path ({}) don't have equal contents",
            path.display(),
            tab_path.display()
        );
    }
}
