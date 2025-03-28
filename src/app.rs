// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "wayland")]
use cosmic::iced::{
    event::wayland::{Event as WaylandEvent, OutputEvent, OverlapNotifyEvent},
    platform_specific::runtime::wayland::layer_surface::{
        IcedMargin, IcedOutput, SctkLayerSurfaceSettings,
    },
    platform_specific::shell::wayland::commands::layer_surface::{
        destroy_layer_surface, get_layer_surface, Anchor, KeyboardInteractivity, Layer,
    },
    Limits,
};
#[cfg(feature = "wayland")]
use cosmic::iced_winit::commands::overlap_notify::overlap_notify;
use cosmic::{
    app::{self, context_drawer, Core, Task},
    cosmic_config, cosmic_theme, executor,
    iced::{
        self,
        clipboard::dnd::DndAction,
        core::SmolStr,
        event,
        futures::{self, SinkExt},
        keyboard::{Event as KeyEvent, Key, Modifiers},
        stream,
        window::{self, Event as WindowEvent, Id as WindowId},
        Alignment, Event, Length, Point, Rectangle, Size, Subscription,
    },
    iced_runtime::clipboard,
    style, theme,
    widget::{
        self,
        dnd_destination::DragId,
        horizontal_space, icon,
        menu::{action::MenuAction, key_bind::KeyBind},
        segmented_button::{self, Entity},
        vertical_space,
    },
    Application, ApplicationExt, Element,
};
use cosmic::{iced::mouse::Event::CursorMoved, surface};
use mime_guess::Mime;
use notify_debouncer_full::{
    new_debouncer,
    notify::{self, RecommendedWatcher, Watcher},
    DebouncedEvent, Debouncer, FileIdMap,
};
use slotmap::Key as SlotMapKey;
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    env, fmt, fs, io,
    num::NonZeroU16,
    path::{Path, PathBuf},
    process,
    sync::{Arc, Mutex},
    time::{self, Instant},
};
use tokio::sync::mpsc;
use trash::TrashItem;
#[cfg(feature = "wayland")]
use wayland_client::{protocol::wl_output::WlOutput, Proxy};

use crate::{
    clipboard::{ClipboardCopy, ClipboardKind, ClipboardPaste},
    config::{AppTheme, Config, DesktopConfig, Favorite, IconSizes, TabConfig, TypeToSearch},
    dialog::{DialogKind, DialogMessage},
    fl, home_dir,
    key_bind::key_binds,
    localize::LANGUAGE_SORTER,
    menu, mime_app, mime_icon,
    mounter::{MounterAuth, MounterItem, MounterItems, MounterKey, MounterMessage, MOUNTERS},
    operation::{Controller, Operation, OperationSelection, ReplaceResult},
    spawn_detached::spawn_detached,
    tab::{self, HeadingOptions, ItemMetadata, Location, Tab, HOVER_DURATION},
};
use crate::{
    dialog::Dialog,
    operation::{OperationError, OperationErrorType},
};
use crate::{dialog::DialogResult, mime_app::MimeApp};

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
    CosmicSettingsAppearance,
    CosmicSettingsDisplays,
    CosmicSettingsWallpaper,
    DesktopViewOptions,
    Delete,
    EditHistory,
    EditLocation,
    EmptyTrash,
    #[cfg(feature = "desktop")]
    ExecEntryAction(usize),
    ExtractHere,
    ExtractTo,
    Gallery,
    HistoryNext,
    HistoryPrevious,
    ItemDown,
    ItemLeft,
    ItemRight,
    ItemUp,
    LocationUp,
    NewFile,
    NewFolder,
    Open,
    OpenInNewTab,
    OpenInNewWindow,
    OpenItemLocation,
    OpenTerminal,
    OpenWith,
    Paste,
    PermanentlyDelete,
    Preview,
    Rename,
    RestoreFromTrash,
    SearchActivate,
    SelectFirst,
    SelectLast,
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
            Action::CosmicSettingsAppearance => Message::CosmicSettings("appearance"),
            Action::CosmicSettingsDisplays => Message::CosmicSettings("displays"),
            Action::CosmicSettingsWallpaper => Message::CosmicSettings("wallpaper"),
            Action::Delete => Message::Delete(entity_opt),
            Action::DesktopViewOptions => Message::DesktopViewOptions,
            Action::EditHistory => Message::ToggleContextPage(ContextPage::EditHistory),
            Action::EditLocation => {
                Message::TabMessage(entity_opt, tab::Message::EditLocationEnable)
            }
            Action::EmptyTrash => Message::TabMessage(None, tab::Message::EmptyTrash),
            Action::ExtractHere => Message::ExtractHere(entity_opt),
            Action::ExtractTo => Message::ExtractTo(entity_opt),
            #[cfg(feature = "desktop")]
            Action::ExecEntryAction(action) => {
                Message::TabMessage(entity_opt, tab::Message::ExecEntryAction(None, *action))
            }
            Action::Gallery => Message::TabMessage(entity_opt, tab::Message::GalleryToggle),
            Action::HistoryNext => Message::TabMessage(entity_opt, tab::Message::GoNext),
            Action::HistoryPrevious => Message::TabMessage(entity_opt, tab::Message::GoPrevious),
            Action::ItemDown => Message::TabMessage(entity_opt, tab::Message::ItemDown),
            Action::ItemLeft => Message::TabMessage(entity_opt, tab::Message::ItemLeft),
            Action::ItemRight => Message::TabMessage(entity_opt, tab::Message::ItemRight),
            Action::ItemUp => Message::TabMessage(entity_opt, tab::Message::ItemUp),
            Action::LocationUp => Message::TabMessage(entity_opt, tab::Message::LocationUp),
            Action::NewFile => Message::NewItem(entity_opt, false),
            Action::NewFolder => Message::NewItem(entity_opt, true),
            Action::Open => Message::TabMessage(entity_opt, tab::Message::Open(None)),
            Action::OpenInNewTab => Message::OpenInNewTab(entity_opt),
            Action::OpenInNewWindow => Message::OpenInNewWindow(entity_opt),
            Action::OpenItemLocation => Message::OpenItemLocation(entity_opt),
            Action::OpenTerminal => Message::OpenTerminal(entity_opt),
            Action::OpenWith => Message::OpenWithDialog(entity_opt),
            Action::Paste => Message::Paste(entity_opt),
            Action::PermanentlyDelete => Message::PermanentlyDelete(entity_opt),
            Action::Preview => Message::Preview(entity_opt),
            Action::Rename => Message::Rename(entity_opt),
            Action::RestoreFromTrash => Message::RestoreFromTrash(entity_opt),
            Action::SearchActivate => Message::SearchActivate,
            Action::SelectAll => Message::TabMessage(entity_opt, tab::Message::SelectAll),
            Action::SelectFirst => Message::TabMessage(entity_opt, tab::Message::SelectFirst),
            Action::SelectLast => Message::TabMessage(entity_opt, tab::Message::SelectLast),
            Action::SetSort(sort, dir) => {
                Message::TabMessage(entity_opt, tab::Message::SetSort(*sort, *dir))
            }
            Action::Settings => Message::ToggleContextPage(ContextPage::Settings),
            Action::TabClose => Message::TabClose(entity_opt),
            Action::TabNew => Message::TabNew,
            Action::TabNext => Message::TabNext,
            Action::TabPrev => Message::TabPrev,
            Action::TabViewGrid => Message::TabView(entity_opt, tab::View::Grid),
            Action::TabViewList => Message::TabView(entity_opt, tab::View::List),
            Action::ToggleFoldersFirst => Message::ToggleFoldersFirst,
            Action::ToggleShowHidden => {
                Message::TabMessage(entity_opt, tab::Message::ToggleShowHidden)
            }
            Action::ToggleSort(sort) => {
                Message::TabMessage(entity_opt, tab::Message::ToggleSort(*sort))
            }
            Action::WindowClose => Message::WindowClose,
            Action::WindowNew => Message::WindowNew,
            Action::ZoomDefault => Message::ZoomDefault(entity_opt),
            Action::ZoomIn => Message::ZoomIn(entity_opt),
            Action::ZoomOut => Message::ZoomOut(entity_opt),
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
    Open(segmented_button::Entity),
    OpenWith(segmented_button::Entity),
    OpenInNewTab(segmented_button::Entity),
    OpenInNewWindow(segmented_button::Entity),
    Preview(segmented_button::Entity),
    RemoveFromSidebar(segmented_button::Entity),
    EmptyTrash,
}

impl MenuAction for NavMenuAction {
    type Message = cosmic::Action<Message>;

    fn message(&self) -> Self::Message {
        cosmic::Action::App(Message::NavMenuAction(*self))
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
    CosmicSettings(&'static str),
    CursorMoved(Point),
    Cut(Option<Entity>),
    Delete(Option<Entity>),
    DesktopConfig(DesktopConfig),
    DesktopViewOptions,
    DialogCancel,
    DialogComplete,
    FileDialogMessage(DialogMessage),
    DialogPush(DialogPage),
    DialogUpdate(DialogPage),
    DialogUpdateComplete(DialogPage),
    ExtractHere(Option<Entity>),
    ExtractTo(Option<Entity>),
    ExtractToResult(DialogResult),
    #[cfg(all(feature = "desktop", feature = "wayland"))]
    Focused(window::Id),
    Key(Modifiers, Key, Option<SmolStr>),
    LaunchUrl(String),
    MaybeExit,
    ModifiersChanged(Modifiers),
    MounterItems(MounterKey, MounterItems),
    MountResult(MounterKey, MounterItem, Result<bool, String>),
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
    OpenInNewTab(Option<Entity>),
    OpenInNewWindow(Option<Entity>),
    OpenItemLocation(Option<Entity>),
    OpenWithBrowse,
    OpenWithDialog(Option<Entity>),
    OpenWithSelection(usize),
    #[cfg(all(feature = "desktop", feature = "wayland"))]
    Overlap(OverlapNotifyEvent, window::Id),
    Paste(Option<Entity>),
    PasteContents(PathBuf, ClipboardPaste),
    PendingCancel(u64),
    PendingCancelAll,
    PendingComplete(u64, OperationSelection),
    PendingDismiss,
    PendingError(u64, OperationError),
    PendingPause(u64, bool),
    PendingPauseAll(bool),
    PermanentlyDelete(Option<Entity>),
    Preview(Option<Entity>),
    RescanTrash,
    Rename(Option<Entity>),
    ReplaceResult(ReplaceResult),
    RestoreFromTrash(Option<Entity>),
    ScrollTab(i16),
    SearchActivate,
    SearchClear,
    SearchInput(String),
    SetShowDetails(bool),
    SetTypeToSearch(TypeToSearch),
    SystemThemeModeChange(cosmic_theme::ThemeMode),
    Size(Size),
    TabActivate(Entity),
    TabNext,
    TabPrev,
    TabClose(Option<Entity>),
    TabConfig(TabConfig),
    TabMessage(Option<Entity>, tab::Message),
    TabNew,
    TabRescan(
        Entity,
        Location,
        Option<tab::Item>,
        Vec<tab::Item>,
        Option<Vec<PathBuf>>,
    ),
    TabView(Option<Entity>, tab::View),
    ToggleContextPage(ContextPage),
    ToggleFoldersFirst,
    Undo(usize),
    UndoTrash(widget::ToastId, Arc<[PathBuf]>),
    UndoTrashStart(Vec<TrashItem>),
    WindowClose,
    WindowCloseRequested(window::Id),
    WindowNew,
    WindowUnfocus,
    ZoomDefault(Option<Entity>),
    ZoomIn(Option<Entity>),
    ZoomOut(Option<Entity>),
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
    Cosmic(app::Action),
    None,
    Surface(surface::Action),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextPage {
    About,
    EditHistory,
    NetworkDrive,
    Preview(Option<Entity>, PreviewKind),
    Settings,
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
        password: Option<String>,
    },
    EmptyTrash,
    FailedOperation(u64),
    ExtractPassword {
        id: u64,
        password: String,
    },
    MountError {
        mounter_key: MounterKey,
        item: MounterItem,
        error: String,
    },
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
    OpenWith {
        path: PathBuf,
        mime: mime_guess::Mime,
        selected: usize,
        store_opt: Option<mime_app::MimeApp>,
    },
    PermanentlyDelete {
        paths: Vec<PathBuf>,
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
    SetExecutableAndLaunch {
        path: PathBuf,
    },
    FavoritePathError {
        path: PathBuf,
        entity: Entity,
    },
}

pub struct FavoriteIndex(usize);

pub struct MounterData(MounterKey, MounterItem);

#[derive(Clone, Debug)]
pub enum WindowKind {
    Desktop(Entity),
    DesktopViewOptions,
    Preview(Option<Entity>, PreviewKind),
    FileDialog(Option<Vec<PathBuf>>),
}

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
    context_page: ContextPage,
    dialog_pages: VecDeque<DialogPage>,
    dialog_text_input: widget::Id,
    key_binds: HashMap<KeyBind, Action>,
    margin: HashMap<window::Id, (f32, f32, f32, f32)>,
    mime_app_cache: mime_app::MimeAppCache,
    modifiers: Modifiers,
    mounter_items: HashMap<MounterKey, MounterItems>,
    network_drive_connecting: Option<(MounterKey, String)>,
    network_drive_input: String,
    #[cfg(feature = "notify")]
    notification_opt: Option<Arc<Mutex<notify_rust::NotificationHandle>>>,
    overlap: HashMap<String, (window::Id, Rectangle)>,
    pending_operation_id: u64,
    pending_operations: BTreeMap<u64, (Operation, Controller)>,
    progress_operations: BTreeSet<u64>,
    complete_operations: BTreeMap<u64, Operation>,
    failed_operations: BTreeMap<u64, (Operation, Controller, String)>,
    search_id: widget::Id,
    size: Option<Size>,
    #[cfg(feature = "wayland")]
    surface_ids: HashMap<WlOutput, WindowId>,
    #[cfg(feature = "wayland")]
    surface_names: HashMap<WindowId, String>,
    toasts: widget::toaster::Toasts<Message>,
    watcher_opt: Option<(Debouncer<RecommendedWatcher, FileIdMap>, HashSet<PathBuf>)>,
    window_id_opt: Option<window::Id>,
    windows: HashMap<window::Id, WindowKind>,
    nav_dnd_hover: Option<(Location, Instant)>,
    tab_dnd_hover: Option<(Entity, Instant)>,
    nav_drag_id: DragId,
    tab_drag_id: DragId,
    auto_scroll_speed: Option<i16>,
    file_dialog_opt: Option<Dialog<Message>>,
}

impl App {
    fn open_file(&mut self, path: &PathBuf) {
        let mime = mime_icon::mime_for_path(path);

        if mime == "application/x-desktop" {
            // Try opening desktop application
            match freedesktop_entry_parser::parse_entry(path) {
                Ok(entry) => match entry.section("Desktop Entry").attr("Exec") {
                    Some(exec) => match mime_app::exec_to_command(exec, None) {
                        Some(mut command) => match spawn_detached(&mut command) {
                            Ok(()) => {
                                return;
                            }
                            Err(err) => {
                                log::warn!("failed to execute {:?}: {}", path, err);
                            }
                        },
                        None => {
                            log::warn!("failed to parse {:?}: invalid Desktop Entry/Exec", path);
                        }
                    },
                    None => {
                        log::warn!("failed to parse {:?}: missing Desktop Entry/Exec", path);
                    }
                },
                Err(err) => {
                    log::warn!("failed to parse {:?}: {}", path, err);
                }
            }
        } else if mime == "application/x-executable" || mime == "application/vnd.appimage" {
            // Try opening executable
            let mut command = std::process::Command::new(path);
            match spawn_detached(&mut command) {
                Ok(()) => {}
                Err(err) => match err.kind() {
                    io::ErrorKind::PermissionDenied => {
                        // If permission is denied, try marking as executable, then running
                        self.dialog_pages
                            .push_back(DialogPage::SetExecutableAndLaunch {
                                path: path.to_path_buf(),
                            });
                    }
                    _ => {
                        log::warn!("failed to execute {:?}: {}", path, err);
                    }
                },
            }
            return;
        }

        // Try mime apps, which should be faster than xdg-open
        for app in self.mime_app_cache.get(&mime) {
            let Some(mut command) = app.command(Some(path.clone().into())) else {
                continue;
            };
            match spawn_detached(&mut command) {
                Ok(()) => {
                    let _ = recently_used_xbel::update_recently_used(
                        path,
                        App::APP_ID.to_string(),
                        "cosmic-files".to_string(),
                        None,
                    );
                    return;
                }
                Err(err) => {
                    log::warn!("failed to open {:?} with {:?}: {}", path, app.id, err);
                }
            }
        }

        // loop through subclasses if available
        if let Some(mime_sub_classes) = mime_icon::parent_mime_types(&mime) {
            for sub_class in mime_sub_classes {
                for app in self.mime_app_cache.get(&sub_class) {
                    let Some(mut command) = app.command(Some(path.clone().into())) else {
                        continue;
                    };
                    match spawn_detached(&mut command) {
                        Ok(()) => {
                            let _ = recently_used_xbel::update_recently_used(
                                path,
                                App::APP_ID.to_string(),
                                "cosmic-files".to_string(),
                                None,
                            );
                            return;
                        }
                        Err(err) => {
                            log::warn!("failed to open {:?} with {:?}: {}", path, app.id, err);
                        }
                    }
                }
            }
        }

        // Fall back to using open crate
        match open::that_detached(path) {
            Ok(()) => {
                let _ = recently_used_xbel::update_recently_used(
                    path,
                    App::APP_ID.to_string(),
                    "cosmic-files".to_string(),
                    None,
                );
            }
            Err(err) => {
                log::warn!("failed to open {:?}: {}", path, err);
            }
        }
    }

    #[cfg(feature = "desktop")]
    fn exec_entry_action(entry: cosmic::desktop::DesktopEntryData, action: usize) {
        if let Some(action) = entry.desktop_actions.get(action) {
            // Largely copied from COSMIC app library
            let mut exec = shlex::Shlex::new(&action.exec);
            match exec.next() {
                Some(cmd) if !cmd.contains('=') => {
                    let mut proc = tokio::process::Command::new(cmd);
                    for arg in exec {
                        if !arg.starts_with('%') {
                            proc.arg(arg);
                        }
                    }
                    let _ = proc.spawn();
                }
                _ => (),
            }
        } else {
            log::warn!(
                "Invalid actions index `{action}` for desktop entry {}",
                entry.name
            );
        }
    }

    fn handle_overlap(&mut self) {
        let Some((bl, br, tl, tr, mut size)) = self.size.as_ref().map(|s| {
            (
                Rectangle::new(
                    Point::new(0., s.height / 2.),
                    Size::new(s.width / 2., s.height / 2.),
                ),
                Rectangle::new(
                    Point::new(s.width / 2., s.height / 2.),
                    Size::new(s.width / 2., s.height / 2.),
                ),
                Rectangle::new(Point::new(0., 0.), Size::new(s.width / 2., s.height / 2.)),
                Rectangle::new(
                    Point::new(s.width / 2., 0.),
                    Size::new(s.width / 2., s.height / 2.),
                ),
                *s,
            )
        }) else {
            return;
        };

        let mut overlaps: HashMap<_, _> = self
            .windows
            .keys()
            .map(|k| (*k, (0., 0., 0., 0.)))
            .collect();
        let mut sorted_overlaps: Vec<_> = self.overlap.values().collect();
        sorted_overlaps
            .sort_by(|a, b| (b.1.width * b.1.height).total_cmp(&(a.1.width * b.1.height)));

        for (w_id, overlap) in sorted_overlaps {
            let tl = tl.intersects(overlap);
            let tr = tr.intersects(overlap);
            let bl = bl.intersects(overlap);
            let br = br.intersects(overlap);
            let Some((top, left, bottom, right)) = overlaps.get_mut(w_id) else {
                continue;
            };
            if tl && tr {
                *top += overlap.height;
            }
            if tl && bl {
                *left += overlap.width;
            }
            if bl && br {
                *bottom += overlap.height;
            }
            if tr && br {
                *right += overlap.width;
            }

            let min_dim =
                if overlap.width / size.width.max(1.) > overlap.height / size.height.max(1.) {
                    (0., overlap.height)
                } else {
                    (overlap.width, 0.)
                };
            // just one quadrant with overlap
            if tl && !(tr || bl) {
                *top += min_dim.1;
                *left += min_dim.0;

                size.height -= min_dim.1;
                size.width -= min_dim.0;
            }
            if tr && !(tl || br) {
                *top += min_dim.1;
                *right += min_dim.0;

                size.height -= min_dim.1;
                size.width -= min_dim.0;
            }
            if bl && !(br || tl) {
                *bottom += min_dim.1;
                *left += min_dim.0;

                size.height -= min_dim.1;
                size.width -= min_dim.0;
            }
            if br && !(bl || tr) {
                *bottom += min_dim.1;
                *right += min_dim.0;

                size.height -= min_dim.1;
                size.width -= min_dim.0;
            }
        }
        self.margin = overlaps;
    }

    fn open_tab_entity(
        &mut self,
        location: Location,
        activate: bool,
        selection_paths: Option<Vec<PathBuf>>,
    ) -> (Entity, Task<Message>) {
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

        (
            entity,
            Task::batch([
                self.update_title(),
                self.update_watcher(),
                self.update_tab(entity, location, selection_paths),
            ]),
        )
    }

    fn open_tab(
        &mut self,
        location: Location,
        activate: bool,
        selection_paths: Option<Vec<PathBuf>>,
    ) -> Task<Message> {
        self.open_tab_entity(location, activate, selection_paths).1
    }

    fn operation(&mut self, operation: Operation) {
        let id = self.pending_operation_id;
        self.pending_operation_id += 1;
        if operation.show_progress_notification() {
            self.progress_operations.insert(id);
        }
        self.pending_operations
            .insert(id, (operation, Controller::default()));
    }

    fn remove_window(&mut self, id: &window::Id) {
        if let Some(WindowKind::Desktop(entity)) = self.windows.remove(id) {
            // Remove the tab from the tab model
            self.tab_model.remove(entity);
        }
    }

    fn rescan_operation_selection(&mut self, op_sel: OperationSelection) -> Task<Message> {
        log::info!("rescan_operation_selection {:?}", op_sel);
        let entity = self.tab_model.active();
        let Some(tab) = self.tab_model.data::<Tab>(entity) else {
            return Task::none();
        };
        let Some(items) = tab.items_opt() else {
            return Task::none();
        };
        for item in items.iter() {
            if item.selected {
                if let Some(path) = item.path_opt() {
                    if op_sel.selected.contains(path) || op_sel.ignored.contains(path) {
                        // Ignore if path in selected or ignored paths
                        continue;
                    }
                }

                // Return if there is a previous selection not matching
                return Task::none();
            }
        }
        self.update_tab(entity, tab.location.clone(), Some(op_sel.selected))
    }

    fn update_tab(
        &mut self,
        entity: Entity,
        location: Location,
        selection_paths: Option<Vec<PathBuf>>,
    ) -> Task<Message> {
        if let Location::Search(_, term, ..) = location {
            self.search_set(entity, Some(term), selection_paths)
        } else {
            self.rescan_tab(entity, location, selection_paths)
        }
    }

    fn rescan_tab(
        &mut self,
        entity: Entity,
        location: Location,
        selection_paths: Option<Vec<PathBuf>>,
    ) -> Task<Message> {
        log::info!("rescan_tab {entity:?} {location:?} {selection_paths:?}");
        let icon_sizes = self.config.tab.icon_sizes;
        Task::perform(
            async move {
                let location2 = location.clone();
                match tokio::task::spawn_blocking(move || location2.scan(icon_sizes)).await {
                    Ok((parent_item_opt, items)) => cosmic::action::app(Message::TabRescan(
                        entity,
                        location,
                        parent_item_opt,
                        items,
                        selection_paths,
                    )),
                    Err(err) => {
                        log::warn!("failed to rescan: {}", err);
                        cosmic::action::none()
                    }
                }
            },
            |x| x,
        )
    }

    fn rescan_trash(&mut self) -> Task<Message> {
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
            commands.push(self.update_tab(entity, location, None));
        }
        Task::batch(commands)
    }

    fn search_get(&self) -> Option<&str> {
        let entity = self.tab_model.active();
        let tab = self.tab_model.data::<Tab>(entity)?;
        match &tab.location {
            Location::Search(_, term, ..) => Some(term),
            _ => None,
        }
    }

    fn search_set_active(&mut self, term_opt: Option<String>) -> Task<Message> {
        let entity = self.tab_model.active();
        self.search_set(entity, term_opt, None)
    }

    fn search_set(
        &mut self,
        tab: Entity,
        term_opt: Option<String>,
        selection_paths: Option<Vec<PathBuf>>,
    ) -> Task<Message> {
        let mut title_location_opt = None;
        if let Some(tab) = self.tab_model.data_mut::<Tab>(tab) {
            let location_opt = match term_opt {
                Some(term) => match &tab.location {
                    Location::Path(path) | Location::Search(path, ..) => Some((
                        Location::Search(
                            path.to_path_buf(),
                            term,
                            tab.config.show_hidden,
                            Instant::now(),
                        ),
                        true,
                    )),
                    _ => None,
                },
                None => match &tab.location {
                    Location::Search(path, ..) => Some((Location::Path(path.to_path_buf()), false)),
                    _ => None,
                },
            };
            if let Some((location, focus_search)) = location_opt {
                tab.change_location(&location, None);
                title_location_opt = Some((tab.title(), tab.location.clone(), focus_search));
            }
        }
        if let Some((title, location, focus_search)) = title_location_opt {
            self.tab_model.text_set(tab, title);
            return Task::batch([
                self.update_title(),
                self.update_watcher(),
                self.rescan_tab(tab, location, selection_paths),
                if focus_search {
                    widget::text_input::focus(self.search_id.clone())
                } else {
                    Task::none()
                },
            ]);
        }
        Task::none()
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

    fn update_config(&mut self) -> Task<Message> {
        self.update_nav_model();
        // Tabs are collected first to placate the borrowck
        let tabs: Vec<_> = self.tab_model.iter().collect();
        // Update main conf and each tab with the new config
        let commands: Vec<_> =
            std::iter::once(cosmic::command::set_theme(self.config.app_theme.theme()))
                .chain(tabs.into_iter().map(|entity| {
                    self.update(Message::TabMessage(
                        Some(entity),
                        tab::Message::Config(self.config.tab),
                    ))
                }))
                .collect();
        Task::batch(commands)
    }

    fn update_desktop(&mut self) -> Task<Message> {
        let mut needs_reload = Vec::new();
        for entity in self.tab_model.iter() {
            if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                if let Location::Desktop(path, output, _) = &tab.location {
                    needs_reload.push((
                        entity,
                        Location::Desktop(path.clone(), output.clone(), self.config.desktop),
                    ));
                };
            }
        }
        let mut commands = Vec::with_capacity(needs_reload.len());
        for (entity, location) in needs_reload {
            if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                tab.location = location.clone();
            }
            commands.push(self.update_tab(entity, location, None));
        }
        Task::batch(commands)
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
                    fl!("filesystem")
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

        if !MOUNTERS.is_empty() {
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

        let tab_entity = self.tab_model.active();
        if let Some(tab) = self.tab_model.data::<Tab>(tab_entity) {
            self.activate_nav_model_location(&tab.location.clone());
        }
    }

    fn update_notification(&mut self) -> Task<Message> {
        // Handle closing notification if there are no operations
        if self.pending_operations.is_empty() {
            #[cfg(feature = "notify")]
            if let Some(notification_arc) = self.notification_opt.take() {
                return Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            //TODO: this is nasty
                            let notification_mutex = Arc::try_unwrap(notification_arc).unwrap();
                            let notification = notification_mutex.into_inner().unwrap();
                            notification.close();
                        })
                        .await
                        .unwrap();
                        cosmic::action::app(Message::MaybeExit)
                    },
                    |x| x,
                );
            }
        }

        Task::none()
    }

    fn update_title(&mut self) -> Task<Message> {
        let window_title = match self.tab_model.text(self.tab_model.active()) {
            Some(tab_title) => format!("{tab_title} — {}", fl!("cosmic-files")),
            None => fl!("cosmic-files"),
        };
        if let Some(window_id) = &self.window_id_opt {
            self.set_window_title(window_title, *window_id)
        } else {
            Task::none()
        }
    }

    fn update_watcher(&mut self) -> Task<Message> {
        if let Some((mut watcher, old_paths)) = self.watcher_opt.take() {
            let mut new_paths = HashSet::new();
            for entity in self.tab_model.iter() {
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    if let Some(path) = tab.location.path_opt() {
                        new_paths.insert(path.to_path_buf());
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
        Task::none()
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
        .align_x(Alignment::Center)
        .spacing(space_xxs)
        .into()
    }

    fn network_drive(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_m, ..
        } = theme::active().cosmic().spacing;
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
            widget::text::body(fl!("network-drive-description")).into(),
            table.into(),
        ])
        .spacing(space_m)
        .into()
    }

    fn desktop_view_options(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_m, space_l, ..
        } = theme::active().cosmic().spacing;
        let config = self.config.desktop;

        let mut children = Vec::new();

        let mut section = widget::settings::section().title(fl!("show-on-desktop"));
        section = section.add(
            widget::settings::item::builder(fl!("desktop-folder-content")).toggler(
                config.show_content,
                move |show_content| {
                    Message::DesktopConfig(DesktopConfig {
                        show_content,
                        ..config
                    })
                },
            ),
        );
        section = section.add(
            widget::settings::item::builder(fl!("mounted-drives")).toggler(
                config.show_mounted_drives,
                move |show_mounted_drives| {
                    Message::DesktopConfig(DesktopConfig {
                        show_mounted_drives,
                        ..config
                    })
                },
            ),
        );
        section = section.add(
            widget::settings::item::builder(fl!("trash-folder-icon")).toggler(
                config.show_trash,
                move |show_trash| {
                    Message::DesktopConfig(DesktopConfig {
                        show_trash,
                        ..config
                    })
                },
            ),
        );
        children.push(section.into());

        let mut section = widget::settings::section().title(fl!("icon-size-and-spacing"));
        let icon_size: u16 = config.icon_size.into();
        section = section.add(
            widget::settings::item::builder(fl!("icon-size"))
                .description(format!("{}%", icon_size))
                .control(
                    widget::slider(50..=500, icon_size, move |icon_size| {
                        Message::DesktopConfig(DesktopConfig {
                            icon_size: NonZeroU16::new(icon_size).unwrap(),
                            ..config
                        })
                    })
                    .step(25u16),
                ),
        );

        let grid_spacing: u16 = config.grid_spacing.into();
        section = section.add(
            widget::settings::item::builder(fl!("grid-spacing"))
                .description(format!("{}%", grid_spacing))
                .control(
                    widget::slider(50..=500, grid_spacing, move |grid_spacing| {
                        Message::DesktopConfig(DesktopConfig {
                            grid_spacing: NonZeroU16::new(grid_spacing).unwrap(),
                            ..config
                        })
                    })
                    .step(25u16),
                ),
        );
        children.push(section.into());

        widget::column::with_children(children)
            .padding([0, space_l, space_l, space_l])
            .spacing(space_m)
            .into()
    }

    fn edit_history(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_m, .. } = theme::active().cosmic().spacing;

        let mut children = Vec::new();

        //TODO: get height from theme?
        let progress_bar_height = Length::Fixed(4.0);

        if !self.pending_operations.is_empty() {
            let mut section = widget::settings::section().title(fl!("pending"));
            for (id, (op, controller)) in self.pending_operations.iter().rev() {
                let progress = controller.progress();
                section = section.add(widget::column::with_children(vec![
                    widget::row::with_children(vec![
                        widget::progress_bar(0.0..=1.0, progress)
                            .height(progress_bar_height)
                            .into(),
                        if controller.is_paused() {
                            widget::tooltip(
                                widget::button::icon(widget::icon::from_name(
                                    "media-playback-start-symbolic",
                                ))
                                .on_press(Message::PendingPause(*id, false))
                                .padding(8),
                                widget::text::body(fl!("resume")),
                                widget::tooltip::Position::Top,
                            )
                            .into()
                        } else {
                            widget::tooltip(
                                widget::button::icon(widget::icon::from_name(
                                    "media-playback-pause-symbolic",
                                ))
                                .on_press(Message::PendingPause(*id, true))
                                .padding(8),
                                widget::text::body(fl!("pause")),
                                widget::tooltip::Position::Top,
                            )
                            .into()
                        },
                        widget::tooltip(
                            widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                                .on_press(Message::PendingCancel(*id))
                                .padding(8),
                            widget::text::body(fl!("cancel")),
                            widget::tooltip::Position::Top,
                        )
                        .into(),
                    ])
                    .align_y(Alignment::Center)
                    .into(),
                    widget::text::body(op.pending_text(progress, controller.state())).into(),
                ]));
            }
            children.push(section.into());
        }

        if !self.failed_operations.is_empty() {
            let mut section = widget::settings::section().title(fl!("failed"));
            for (_id, (op, controller, error)) in self.failed_operations.iter().rev() {
                let progress = controller.progress();
                section = section.add(widget::column::with_children(vec![
                    widget::text::body(op.pending_text(progress, controller.state())).into(),
                    widget::text::body(error).into(),
                ]));
            }
            children.push(section.into());
        }

        if !self.complete_operations.is_empty() {
            let mut section = widget::settings::section().title(fl!("complete"));
            for (_id, op) in self.complete_operations.iter().rev() {
                section = section.add(widget::text::body(op.completed_text()));
            }
            children.push(section.into());
        }

        if children.is_empty() {
            children.push(widget::text::body(fl!("no-history")).into());
        }

        widget::column::with_children(children)
            .spacing(space_m)
            .into()
    }

    fn preview<'a>(
        &'a self,
        entity_opt: &Option<Entity>,
        kind: &'a PreviewKind,
        context_drawer: bool,
    ) -> Element<'a, tab::Message> {
        let cosmic_theme::Spacing { space_l, .. } = theme::active().cosmic().spacing;

        let mut children = Vec::with_capacity(1);
        let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
        let military_time = self.config.tab.military_time;
        match kind {
            PreviewKind::Custom(PreviewItem(item)) => {
                children.push(item.preview_view(
                    Some(&self.mime_app_cache),
                    IconSizes::default(),
                    military_time,
                ));
            }
            PreviewKind::Location(location) => {
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        for item in items.iter() {
                            if item.location_opt.as_ref() == Some(location) {
                                children.push(item.preview_view(
                                    Some(&self.mime_app_cache),
                                    tab.config.icon_sizes,
                                    military_time,
                                ));
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
                                children.push(item.preview_view(
                                    Some(&self.mime_app_cache),
                                    tab.config.icon_sizes,
                                    military_time,
                                ));
                                // Only show one property view to avoid issues like hangs when generating
                                // preview images on thousands of files
                                break;
                            }
                        }
                        if children.is_empty() {
                            if let Some(item) = &tab.parent_item_opt {
                                children.push(item.preview_view(
                                    Some(&self.mime_app_cache),
                                    tab.config.icon_sizes,
                                    military_time,
                                ));
                            }
                        }
                    }
                }
            }
        }
        widget::column::with_children(children)
            .padding(if context_drawer {
                [0, 0, 0, 0]
            } else {
                [0, space_l, space_l, space_l]
            })
            .into()
    }

    fn settings(&self) -> Element<Message> {
        let tab_config = self.config.tab;

        // TODO: Should dialog be updated here too?
        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("appearance"))
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
                .into(),
            widget::settings::section()
                .title(fl!("type-to-search"))
                .add(widget::radio(
                    widget::text::body(fl!("type-to-search-recursive")),
                    TypeToSearch::Recursive,
                    Some(self.config.type_to_search),
                    Message::SetTypeToSearch,
                ))
                .add(widget::radio(
                    widget::text::body(fl!("type-to-search-enter-path")),
                    TypeToSearch::EnterPath,
                    Some(self.config.type_to_search),
                    Message::SetTypeToSearch,
                ))
                .into(),
            widget::settings::section()
                .title(fl!("other"))
                .add({
                    widget::settings::item::builder(fl!("single-click")).toggler(
                        tab_config.single_click,
                        move |single_click| {
                            Message::TabConfig(TabConfig {
                                single_click,
                                ..tab_config
                            })
                        },
                    )
                })
                .into(),
        ])
        .into()
    }

    fn get_programs_for_mime(&self, mime_type: &Mime) -> Vec<&MimeApp> {
        let mut results = Vec::new();

        let mut dedupe = HashSet::new();

        // start with exact matches
        for mime_app in self.mime_app_cache.get(mime_type) {
            let app_id = &mime_app.id;
            if !dedupe.contains(app_id) {
                results.push(mime_app);
                dedupe.insert(app_id);
            }
        }

        // grab matches based off of subclass / parent mime type
        if let Some(parent_types) = mime_icon::parent_mime_types(mime_type) {
            for parent_type in parent_types {
                for mime_app in self.mime_app_cache.get(&parent_type) {
                    let app_id = &mime_app.id;
                    if !dedupe.contains(app_id) {
                        results.push(mime_app);
                        dedupe.insert(app_id);
                    }
                }
            }
        }

        results
    }

    // Update favorites based on renaming or moving dirs.
    fn update_favorites(&mut self, path_changes: &[(PathBuf, PathBuf)]) -> bool {
        let mut favorites_changed = false;
        let favorites = self
            .config
            .favorites
            .iter()
            .cloned()
            .map(|favorite| {
                if let Favorite::Path(ref path) = favorite {
                    for (from, to) in path_changes {
                        if path.starts_with(from) {
                            if let Ok(relative) = path.strip_prefix(from) {
                                favorites_changed = true;
                                return Favorite::from_path(to.join(relative));
                            }
                        }
                    }
                }
                favorite
            })
            .collect();

        if favorites_changed {
            if let Some(config_handler) = &self.config_handler {
                match self.config.set_favorites(config_handler, favorites) {
                    Ok(updated) => {
                        if updated {
                            return true;
                        }
                    }
                    Err(err) => {
                        log::warn!(
                            "failed to update favorites after moving directories: {:?}",
                            err,
                        );
                    }
                };
            } else {
                self.config.favorites = favorites;
                log::warn!(
                    "failed to update favorites after moving directories: no config handler",
                );
            }
        }

        return false;
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
    fn init(mut core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        core.window.context_is_overlay = false;
        match flags.mode {
            Mode::App => {
                core.window.show_context = flags.config.show_details;
            }
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

        let key_binds = key_binds(&match flags.mode {
            Mode::App => tab::Mode::App,
            Mode::Desktop => tab::Mode::Desktop,
        });

        let window_id_opt = core.main_window_id();

        let mut app = App {
            core,
            nav_bar_context_id: segmented_button::Entity::null(),
            nav_model: segmented_button::ModelBuilder::default().build(),
            tab_model: segmented_button::ModelBuilder::default().build(),
            config_handler: flags.config_handler,
            config: flags.config,
            mode: flags.mode,
            app_themes,
            context_page: ContextPage::Preview(None, PreviewKind::Selected),
            dialog_pages: VecDeque::new(),
            dialog_text_input: widget::Id::unique(),
            key_binds,
            margin: HashMap::new(),
            mime_app_cache: mime_app::MimeAppCache::new(),
            modifiers: Modifiers::empty(),
            mounter_items: HashMap::new(),
            network_drive_connecting: None,
            network_drive_input: String::new(),
            #[cfg(feature = "notify")]
            notification_opt: None,
            overlap: HashMap::new(),
            pending_operation_id: 0,
            pending_operations: BTreeMap::new(),
            progress_operations: BTreeSet::new(),
            complete_operations: BTreeMap::new(),
            failed_operations: BTreeMap::new(),
            search_id: widget::Id::unique(),
            size: None,
            #[cfg(feature = "wayland")]
            surface_ids: HashMap::new(),
            #[cfg(feature = "wayland")]
            surface_names: HashMap::new(),
            toasts: widget::toaster::Toasts::new(Message::CloseToast),
            watcher_opt: None,
            window_id_opt,
            windows: HashMap::new(),
            nav_dnd_hover: None,
            tab_dnd_hover: None,
            nav_drag_id: DragId::new(),
            tab_drag_id: DragId::new(),
            auto_scroll_speed: None,
            file_dialog_opt: None,
        };

        let mut commands = vec![app.update_config()];

        for location in flags.locations {
            if let Some(path) = location.path_opt() {
                if path.is_file() {
                    if let Some(parent) = path.parent() {
                        commands.push(app.open_tab(
                            Location::Path(parent.to_path_buf()),
                            true,
                            Some(vec![path.to_path_buf()]),
                        ));
                        continue;
                    }
                }
            }
            commands.push(app.open_tab(location, true, None));
        }

        if app.tab_model.iter().next().is_none() {
            if let Ok(current_dir) = env::current_dir() {
                commands.push(app.open_tab(Location::Path(current_dir), true, None));
            } else {
                commands.push(app.open_tab(Location::Path(home_dir()), true, None));
            }
        }

        (app, Task::batch(commands))
    }

    fn nav_bar(&self) -> Option<Element<cosmic::Action<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav = cosmic::widget::nav_bar(nav_model, |entity| {
            cosmic::Action::Cosmic(cosmic::app::Action::NavBar(entity))
        })
        .drag_id(self.nav_drag_id)
        .on_dnd_enter(|entity, _| cosmic::Action::App(Message::DndEnterNav(entity)))
        .on_dnd_leave(|_| cosmic::Action::App(Message::DndExitNav))
        .on_dnd_drop(|entity, data, action| {
            cosmic::Action::App(Message::DndDropNav(entity, data, action))
        })
        .on_context(|entity| cosmic::Action::App(Message::NavBarContext(entity)))
        .on_close(|entity| cosmic::Action::App(Message::NavBarClose(entity)))
        .on_middle_press(|entity| {
            cosmic::Action::App(Message::NavMenuAction(NavMenuAction::OpenInNewTab(entity)))
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
        entity: widget::nav_bar::Id,
    ) -> Option<Vec<widget::menu::Tree<cosmic::Action<Self::Message>>>> {
        let favorite_index_opt = self.nav_model.data::<FavoriteIndex>(entity);
        let location_opt = self.nav_model.data::<Location>(entity);

        let mut items = Vec::new();

        if location_opt
            .and_then(|x| x.path_opt())
            .map_or(false, |x| x.is_file())
        {
            items.push(cosmic::widget::menu::Item::Button(
                fl!("open"),
                None,
                NavMenuAction::Open(entity),
            ));
            items.push(cosmic::widget::menu::Item::Button(
                fl!("menu-open-with"),
                None,
                NavMenuAction::OpenWith(entity),
            ));
        } else {
            items.push(cosmic::widget::menu::Item::Button(
                fl!("open-in-new-tab"),
                None,
                NavMenuAction::OpenInNewTab(entity),
            ));
            items.push(cosmic::widget::menu::Item::Button(
                fl!("open-in-new-window"),
                None,
                NavMenuAction::OpenInNewWindow(entity),
            ));
        }
        items.push(cosmic::widget::menu::Item::Divider);
        if matches!(location_opt, Some(Location::Path(..))) {
            items.push(cosmic::widget::menu::Item::Button(
                fl!("show-details"),
                None,
                NavMenuAction::Preview(entity),
            ));
        }
        items.push(cosmic::widget::menu::Item::Divider);
        if favorite_index_opt.is_some() {
            items.push(cosmic::widget::menu::Item::Button(
                fl!("remove-from-sidebar"),
                None,
                NavMenuAction::RemoveFromSidebar(entity),
            ));
        }
        if matches!(location_opt, Some(Location::Trash)) {
            if tab::trash_entries() > 0 {
                items.push(cosmic::widget::menu::Item::Button(
                    fl!("empty-trash"),
                    None,
                    NavMenuAction::EmptyTrash,
                ));
            }
        }

        Some(cosmic::widget::menu::items(&HashMap::new(), items))
    }

    fn nav_model(&self) -> Option<&segmented_button::SingleSelectModel> {
        match self.mode {
            Mode::App => Some(&self.nav_model),
            Mode::Desktop => None,
        }
    }

    fn on_nav_select(&mut self, entity: Entity) -> Task<Self::Message> {
        self.nav_model.activate(entity);
        if let Some(location) = self.nav_model.data::<Location>(entity) {
            let should_open = match location {
                Location::Path(path) => match path.try_exists() {
                    Ok(true) => true,
                    Ok(false) => {
                        log::warn!("failed to open favorite, path does not exist: {:?}", path);
                        self.dialog_pages.push_back(DialogPage::FavoritePathError {
                            path: path.clone(),
                            entity,
                        });
                        false
                    }
                    Err(err) => {
                        log::warn!("failed to open favorite for path: {:?}, {}", path, err);
                        self.dialog_pages.push_back(DialogPage::FavoritePathError {
                            path: path.clone(),
                            entity,
                        });
                        false
                    }
                },
                _ => true,
            };

            if should_open {
                let message = Message::TabMessage(None, tab::Message::Location(location.clone()));
                return self.update(message);
            }
        }

        if let Some(data) = self.nav_model.data::<MounterData>(entity) {
            if let Some(mounter) = MOUNTERS.get(&data.0) {
                return mounter
                    .mount(data.1.clone())
                    .map(|_| cosmic::action::none());
            }
        }
        Task::none()
    }

    fn on_app_exit(&mut self) -> Option<Message> {
        Some(Message::WindowClose)
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        Some(Message::WindowCloseRequested(id))
    }

    fn on_context_drawer(&mut self) -> Task<Self::Message> {
        if let ContextPage::Preview(..) = self.context_page {
            // Persist state of preview page
            if self.core.window.show_context != self.config.show_details {
                return self.update(Message::Preview(None));
            }
        }
        Task::none()
    }

    fn on_escape(&mut self) -> Task<Self::Message> {
        let entity = self.tab_model.active();

        // Close dialog if open
        if self.dialog_pages.pop_front().is_some() {
            return Task::none();
        }

        // Close gallery mode if open
        if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
            if tab.gallery {
                tab.gallery = false;
                return Task::none();
            }
        }

        // Close menus and context panes in order per message
        // Why: It'd be weird to close everything all at once
        // Usually, the Escape key (for example) closes menus and panes one by one instead
        // of closing everything on one press
        if self.core.window.show_context {
            self.set_show_context(false);
            return cosmic::task::message(cosmic::action::app(Message::SetShowDetails(false)));
        }
        if self.search_get().is_some() {
            // Close search if open
            return self.search_set_active(None);
        }
        if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
            if tab.context_menu.is_some() {
                tab.context_menu = None;
                return Task::none();
            }

            if tab.edit_location.is_some() {
                tab.edit_location = None;
                return Task::none();
            }

            let had_focused_button = tab.select_focus_id().is_some();
            if tab.select_none() {
                if had_focused_button {
                    // Unfocus if there was a focused button
                    return widget::button::focus(widget::Id::unique());
                }
                return Task::none();
            }
        }

        Task::none()
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
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
                    let favorite = Favorite::from_path(path);
                    if !favorites.iter().any(|f| f == &favorite) {
                        favorites.push(favorite);
                    }
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
                            password: None,
                        });
                        return widget::text_input::focus(self.dialog_text_input.clone());
                    }
                }
            }
            Message::Config(config) => {
                if config != self.config {
                    log::info!("update config");
                    // Show details is preserved for existing instances
                    let show_details = self.config.show_details;
                    self.config = config;
                    self.config.show_details = show_details;
                    return self.update_config();
                }
            }
            Message::Copy(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                let contents = ClipboardCopy::new(ClipboardKind::Copy, &paths);
                return clipboard::write_data(contents);
            }
            Message::CursorMoved(pos) => {
                let entity = self.tab_model.active();
                return self.update(Message::TabMessage(
                    Some(entity),
                    tab::Message::CursorMoved(pos),
                ));
            }
            Message::Cut(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                let contents = ClipboardCopy::new(ClipboardKind::Cut, &paths);
                return clipboard::write_data(contents);
            }
            Message::CloseToast(id) => {
                self.toasts.remove(id);
            }
            Message::CosmicSettings(arg) => {
                //TODO: use special settings URL scheme instead?
                let mut command = process::Command::new("cosmic-settings");
                command.arg(arg);
                match spawn_detached(&mut command) {
                    Ok(()) => {}
                    Err(err) => {
                        log::warn!("failed to run cosmic-settings {}: {}", arg, err)
                    }
                }
            }
            Message::Delete(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    match &tab.location {
                        Location::Trash => {
                            if let Some(items) = tab.items_opt() {
                                let mut trash_items = Vec::new();
                                for item in items.iter() {
                                    if item.selected {
                                        match &item.metadata {
                                            ItemMetadata::Trash { entry, .. } => {
                                                trash_items.push(entry.clone());
                                            }
                                            _ => {
                                                //TODO: error on trying to permanently delete non-trash file?
                                            }
                                        }
                                    }
                                }
                                if !trash_items.is_empty() {
                                    self.operation(Operation::DeleteTrash { items: trash_items });
                                }
                            }
                        }
                        _ => {
                            let paths = dbg!(self.selected_paths(entity_opt));
                            if !paths.is_empty() {
                                self.operation(Operation::Delete { paths });
                            }
                        }
                    }
                }
            }
            Message::DesktopConfig(config) => {
                if config != self.config.desktop {
                    config_set!(desktop, config);
                    return self.update_desktop();
                }
            }
            Message::DesktopViewOptions => {
                let mut settings = window::Settings {
                    decorations: true,
                    min_size: Some(Size::new(360.0, 180.0)),
                    resizable: true,
                    size: Size::new(480.0, 444.0),
                    transparent: true,
                    ..Default::default()
                };

                #[cfg(target_os = "linux")]
                {
                    // Use the dialog ID to make it float
                    settings.platform_specific.application_id =
                        "com.system76.CosmicFilesDialog".to_string();
                }

                let (id, command) = window::open(settings);
                self.windows.insert(id, WindowKind::DesktopViewOptions);
                return command.map(|_id| cosmic::action::none());
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
                            password,
                        } => {
                            let extension = archive_type.extension();
                            let name = format!("{}{}", name, extension);
                            let to = to.join(name);
                            self.operation(Operation::Compress {
                                paths,
                                to,
                                archive_type,
                                password,
                            })
                        }
                        DialogPage::EmptyTrash => {
                            self.operation(Operation::EmptyTrash);
                        }
                        DialogPage::FailedOperation(id) => {
                            log::warn!("TODO: retry operation {}", id);
                        }
                        DialogPage::ExtractPassword { id, password } => {
                            let (operation, _, _err) = self.failed_operations.get(&id).unwrap();
                            let new_op = match &operation {
                                Operation::Extract { to, paths, .. } => Operation::Extract {
                                    to: to.clone(),
                                    paths: paths.clone(),
                                    password: Some(password),
                                },
                                _ => unreachable!(),
                            };
                            self.operation(new_op);
                        }
                        DialogPage::MountError {
                            mounter_key,
                            item,
                            error: _,
                        } => {
                            if let Some(mounter) = MOUNTERS.get(&mounter_key) {
                                return mounter.mount(item).map(|_| cosmic::action::none());
                            }
                        }
                        DialogPage::NetworkAuth {
                            mounter_key: _,
                            uri: _,
                            auth,
                            auth_tx,
                        } => {
                            return Task::perform(
                                async move {
                                    auth_tx.send(auth).await.unwrap();
                                    cosmic::action::none()
                                },
                                |x| x,
                            );
                        }
                        DialogPage::NetworkError {
                            mounter_key: _,
                            uri,
                            error: _,
                        } => {
                            //TODO: re-use mounter_key?
                            return Task::batch([
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
                        DialogPage::OpenWith {
                            path,
                            mime,
                            selected,
                            ..
                        } => {
                            let all_apps = self.get_programs_for_mime(&mime);

                            if let Some(app) = all_apps.get(selected) {
                                if let Some(mut command) = app.command(Some(path.clone().into())) {
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
                                            log::warn!(
                                                "failed to open {:?} with {:?}: {}",
                                                path,
                                                app.id,
                                                err
                                            )
                                        }
                                    }
                                } else {
                                    log::warn!(
                                        "failed to open {:?} with {:?}: failed to get command",
                                        path,
                                        app.id
                                    );
                                }
                            }
                        }
                        DialogPage::PermanentlyDelete { paths } => {
                            self.operation(Operation::PermanentlyDelete { paths });
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
                        DialogPage::SetExecutableAndLaunch { path } => {
                            self.operation(Operation::SetExecutableAndLaunch { path });
                        }
                        DialogPage::FavoritePathError { entity, .. } => {
                            if let Some(FavoriteIndex(favorite_i)) =
                                self.nav_model.data::<FavoriteIndex>(entity)
                            {
                                let mut favorites = self.config.favorites.clone();
                                favorites.remove(*favorite_i);
                                config_set!(favorites, favorites);
                                return self.update_config();
                            }
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
                return Task::batch([
                    self.update(Message::DialogUpdate(dialog_page)),
                    self.update(Message::DialogComplete),
                ]);
            }
            Message::ExtractHere(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                if let Some(destination) = paths
                    .first()
                    .and_then(|first| first.parent())
                    .map(|parent| parent.to_path_buf())
                {
                    self.operation(Operation::Extract {
                        paths,
                        to: destination,
                        password: None,
                    });
                }
            }
            Message::ExtractTo(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                if let Some(destination) = paths
                    .first()
                    .and_then(|first| first.parent())
                    .map(|parent| parent.to_path_buf())
                {
                    let (mut dialog, dialog_task) = Dialog::new(
                        DialogKind::OpenFolder,
                        Some(destination),
                        Message::FileDialogMessage,
                        Message::ExtractToResult,
                    );
                    let set_title_task = dialog.set_title(fl!("extract-to-title"));
                    dialog.set_accept_label(fl!("extract-here"));
                    self.windows
                        .insert(dialog.window_id(), WindowKind::FileDialog(Some(paths)));
                    self.file_dialog_opt = Some(dialog);
                    return Task::batch([set_title_task, dialog_task]);
                };
            }
            Message::ExtractToResult(result) => {
                match result {
                    DialogResult::Cancel => {}
                    DialogResult::Open(selected_paths) => {
                        let mut archive_paths = None;
                        if let Some(file_dialog) = &self.file_dialog_opt {
                            let window = self.windows.remove(&file_dialog.window_id());
                            if let Some(WindowKind::FileDialog(paths)) = window {
                                archive_paths = paths;
                            }
                        }
                        if let Some(archive_paths) = archive_paths {
                            if !selected_paths.is_empty() {
                                self.operation(Operation::Extract {
                                    paths: archive_paths,
                                    to: selected_paths[0].clone(),
                                    password: None,
                                });
                            }
                        }
                    }
                }
                self.file_dialog_opt = None;
            }
            Message::FileDialogMessage(dialog_message) => {
                if let Some(dialog) = &mut self.file_dialog_opt {
                    return dialog.update(dialog_message);
                }
            }
            Message::Key(modifiers, key, text) => {
                let entity = self.tab_model.active();
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message(Some(entity)));
                    }
                }

                // Uncaptured keys with only shift modifiers go to the search or location box
                if !modifiers.logo()
                    && !modifiers.control()
                    && !modifiers.alt()
                    && matches!(key, Key::Character(_))
                {
                    if let Some(text) = text {
                        match self.config.type_to_search {
                            TypeToSearch::Recursive => {
                                let mut term = self.search_get().unwrap_or_default().to_string();
                                term.push_str(&text);
                                return self.search_set_active(Some(term));
                            }
                            TypeToSearch::EnterPath => {
                                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                                    let location = tab.edit_location.as_ref().map_or_else(
                                        || tab.location.clone(),
                                        |x| x.location.clone(),
                                    );
                                    // Try to add text to end of location
                                    if let Some(path) = location.path_opt() {
                                        let mut path_string = path.to_string_lossy().to_string();
                                        path_string.push_str(&text);
                                        tab.edit_location = Some(
                                            location.with_path(PathBuf::from(path_string)).into(),
                                        );
                                    }
                                }
                            }
                        }
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
            Message::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
                let entity = self.tab_model.active();
                return self.update(Message::TabMessage(
                    Some(entity),
                    tab::Message::ModifiersChanged(modifiers),
                ));
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
                                        if path == old_path && item.is_mounted() {
                                            still_mounted = true;
                                            break;
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
                            commands.push(self.update_tab(entity, home_location.clone(), None));
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

                // Update desktop tabs
                commands.push(self.update_desktop());

                return Task::batch(commands);
            }
            Message::MountResult(mounter_key, item, res) => match res {
                Ok(true) => {
                    log::info!("connected to {:?}", item);
                }
                Ok(false) => {
                    log::info!("cancelled connection to {:?}", item);
                }
                Err(error) => {
                    log::warn!("failed to connect to {:?}: {}", item, error);
                    self.dialog_pages.push_back(DialogPage::MountError {
                        mounter_key,
                        item,
                        error,
                    });
                }
            },
            Message::NetworkAuth(mounter_key, uri, auth, auth_tx) => {
                self.dialog_pages.push_back(DialogPage::NetworkAuth {
                    mounter_key,
                    uri,
                    auth,
                    auth_tx,
                });
                return widget::text_input::focus(self.dialog_text_input.clone());
            }
            Message::NetworkDriveInput(input) => {
                self.network_drive_input = input;
            }
            Message::NetworkDriveSubmit => {
                //TODO: know which mounter to use for network drives
                for (mounter_key, mounter) in MOUNTERS.iter() {
                    self.network_drive_connecting =
                        Some((*mounter_key, self.network_drive_input.clone()));
                    return mounter
                        .network_drive(self.network_drive_input.clone())
                        .map(|_| cosmic::action::none());
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
                    if let Some(path) = &tab.location.path_opt() {
                        self.dialog_pages.push_back(DialogPage::NewItem {
                            parent: path.to_path_buf(),
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
                        if let Some(path) = &tab.location.path_opt() {
                            let mut contains_change = false;
                            for event in events.iter() {
                                for event_path in event.paths.iter() {
                                    if event_path.starts_with(path) {
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
                                                            match fs::metadata(event_path) {
                                                                Ok(new_metadata) => {
                                                                    if let ItemMetadata::Path {
                                                                        metadata,
                                                                        ..
                                                                    } = &mut item.metadata
                                                                    {
                                                                        *metadata = new_metadata
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
                                needs_reload.push((entity, tab.location.clone()));
                            }
                        }
                    }
                }

                let mut commands = Vec::with_capacity(needs_reload.len());
                for (entity, location) in needs_reload {
                    commands.push(self.update_tab(entity, location, None));
                }
                return Task::batch(commands);
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
                if let Some(terminal) = self.mime_app_cache.terminal() {
                    let mut paths = Vec::new();
                    let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                    if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                        if let Some(path) = &tab.location.path_opt() {
                            if let Some(items) = tab.items_opt() {
                                for item in items.iter() {
                                    if item.selected {
                                        if let Some(path) = item.path_opt() {
                                            paths.push(path.to_path_buf());
                                        }
                                    }
                                }
                            }
                            if paths.is_empty() {
                                paths.push(path.to_path_buf());
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
            Message::OpenInNewTab(entity_opt) => {
                return Task::batch(self.selected_paths(entity_opt).into_iter().filter_map(
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
                return Task::batch(self.selected_paths(entity_opt).into_iter().filter_map(
                    |path| {
                        path.parent().map(Path::to_path_buf).map(|parent| {
                            self.open_tab(Location::Path(parent), true, Some(vec![path]))
                        })
                    },
                ))
            }
            Message::OpenWithBrowse => match self.dialog_pages.pop_front() {
                Some(DialogPage::OpenWith {
                    mime,
                    store_opt: Some(app),
                    ..
                }) => {
                    let url = format!("mime:///{mime}");
                    if let Some(mut command) = app.command(Some(url.clone().into())) {
                        match spawn_detached(&mut command) {
                            Ok(()) => {}
                            Err(err) => {
                                log::warn!("failed to open {:?} with {:?}: {}", url, app.id, err)
                            }
                        }
                    } else {
                        log::warn!(
                            "failed to open {:?} with {:?}: failed to get command",
                            url,
                            app.id
                        );
                    }
                }
                Some(dialog_page) => {
                    self.dialog_pages.push_front(dialog_page);
                }
                None => {}
            },
            Message::OpenWithDialog(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        for item in items {
                            if !item.selected {
                                continue;
                            }
                            let Some(path) = item.path_opt() else {
                                continue;
                            };
                            return self.update(Message::DialogPush(DialogPage::OpenWith {
                                path: path.to_path_buf(),
                                mime: item.mime.clone(),
                                selected: 0,
                                store_opt: "x-scheme-handler/mime"
                                    .parse::<mime_guess::Mime>()
                                    .ok()
                                    .and_then(|mime| {
                                        self.mime_app_cache.get(&mime).first().cloned()
                                    }),
                            }));
                        }
                    }
                }
            }
            Message::OpenWithSelection(index) => {
                if let Some(DialogPage::OpenWith { selected, .. }) = self.dialog_pages.front_mut() {
                    *selected = index;
                }
            }
            Message::Paste(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(path) = tab.location.path_opt() {
                        let to = path.clone();
                        return clipboard::read_data::<ClipboardPaste>().map(move |contents_opt| {
                            match contents_opt {
                                Some(contents) => cosmic::action::app(Message::PasteContents(
                                    to.clone(),
                                    contents,
                                )),
                                None => cosmic::action::none(),
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
                            self.operation(Operation::Move {
                                paths: contents.paths,
                                to,
                            });
                        }
                    }
                }
            }
            Message::PendingCancel(id) => {
                if let Some((_, controller)) = self.pending_operations.get(&id) {
                    controller.cancel();
                    self.progress_operations.remove(&id);
                }
            }
            Message::PendingCancelAll => {
                for (id, (_, controller)) in self.pending_operations.iter() {
                    controller.cancel();
                    self.progress_operations.remove(id);
                }
            }
            Message::PendingComplete(id, op_sel) => {
                let mut commands = Vec::with_capacity(4);
                if let Some((op, _)) = self.pending_operations.remove(&id) {
                    // Show toast for some operations
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
                                    .map(cosmic::Action::App),
                            );
                        } else {
                            commands.push(
                                self.toasts
                                    .push(widget::toaster::Toast::new(description))
                                    .map(cosmic::Action::App),
                            );
                        }
                    }

                    // If a favorite for a path has been renamed or moved, update it.
                    if let Operation::Rename { ref from, ref to } = op {
                        if self.update_favorites(&[(from.clone(), to.clone())]) {
                            commands.push(self.update_config());
                        }
                    } else if let Operation::Move { ref paths, ref to } = op {
                        let path_changes: Vec<_> = paths
                            .iter()
                            .filter_map(|from| {
                                from.file_name().map(|name| (from.clone(), to.join(name)))
                            })
                            .collect();
                        if self.update_favorites(&path_changes) {
                            commands.push(self.update_config());
                        }
                    }

                    self.complete_operations.insert(id, op);
                }
                // Close progress notification if all relavent operations are finished
                if !self
                    .pending_operations
                    .iter()
                    .any(|(_id, (op, _))| op.show_progress_notification())
                {
                    self.progress_operations.clear();
                }
                // Potentially show a notification
                commands.push(self.update_notification());
                // Rescan and select based on operation
                commands.push(self.rescan_operation_selection(op_sel));
                // Manually rescan any trash tabs after any operation is completed
                commands.push(self.rescan_trash());
                return Task::batch(commands);
            }
            Message::PendingDismiss => {
                self.progress_operations.clear();
            }
            Message::PendingError(id, err) => {
                if let Some((op, controller)) = self.pending_operations.remove(&id) {
                    // Only show dialog if not cancelled
                    if !controller.is_cancelled() {
                        self.dialog_pages.push_back(match err.kind {
                            OperationErrorType::Generic(_) => DialogPage::FailedOperation(id),
                            OperationErrorType::PasswordRequired => DialogPage::ExtractPassword {
                                id,
                                password: String::from(""),
                            },
                        });
                    }
                    // Remove from progress
                    self.progress_operations.remove(&id);
                    self.failed_operations
                        .insert(id, (op, controller, err.to_string()));
                }
                // Close progress notification if all relavent operations are finished
                if !self
                    .pending_operations
                    .iter()
                    .any(|(_id, (op, _))| op.show_progress_notification())
                {
                    self.progress_operations.clear();
                }
                // Manually rescan any trash tabs after any operation is completed
                return self.rescan_trash();
            }
            Message::PendingPause(id, pause) => {
                if let Some((_, controller)) = self.pending_operations.get(&id) {
                    if pause {
                        controller.pause();
                    } else {
                        controller.unpause();
                    }
                }
            }
            Message::PendingPauseAll(pause) => {
                for (_id, (_, controller)) in self.pending_operations.iter() {
                    if pause {
                        controller.pause();
                    } else {
                        controller.unpause();
                    }
                }
            }
            Message::PermanentlyDelete(entity_opt) => {
                let paths = self.selected_paths(entity_opt);
                if !paths.is_empty() {
                    self.dialog_pages
                        .push_back(DialogPage::PermanentlyDelete { paths });
                }
            }
            Message::Preview(entity_opt) => {
                match self.mode {
                    Mode::App => {
                        let show_details = !self.config.show_details;
                        self.context_page = ContextPage::Preview(None, PreviewKind::Selected);
                        self.core.window.show_context = show_details;
                        return cosmic::task::message(Message::SetShowDetails(show_details));
                    }
                    Mode::Desktop => {
                        let selected_paths = self.selected_paths(entity_opt);
                        let mut commands = Vec::with_capacity(selected_paths.len());
                        for path in selected_paths {
                            let mut settings = window::Settings {
                                decorations: true,
                                min_size: Some(Size::new(360.0, 180.0)),
                                resizable: true,
                                size: Size::new(480.0, 600.0),
                                transparent: true,
                                ..Default::default()
                            };

                            #[cfg(target_os = "linux")]
                            {
                                // Use the dialog ID to make it float
                                settings.platform_specific.application_id =
                                    "com.system76.CosmicFilesDialog".to_string();
                            }

                            let (id, command) = window::open(settings);
                            self.windows.insert(
                                id,
                                WindowKind::Preview(
                                    entity_opt,
                                    PreviewKind::Location(Location::Path(path)),
                                ),
                            );
                            commands.push(command.map(|_id| cosmic::action::none()));
                        }
                        return Task::batch(commands);
                    }
                }
            }
            Message::RescanTrash => {
                // Update trash icon if empty/full
                let maybe_entity = self.nav_model.iter().find(|&entity| {
                    self.nav_model
                        .data::<Location>(entity)
                        .map(|loc| matches!(loc, Location::Trash))
                        .unwrap_or_default()
                });
                if let Some(entity) = maybe_entity {
                    self.nav_model
                        .icon_set(entity, widget::icon::icon(tab::trash_icon_symbolic(16)));
                }

                return Task::batch([self.rescan_trash(), self.update_desktop()]);
            }
            Message::Rename(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        let mut selected = Vec::new();
                        for item in items.iter() {
                            if item.selected {
                                if let Some(path) = item.path_opt() {
                                    selected.push(path.to_path_buf());
                                }
                            }
                        }
                        if !selected.is_empty() {
                            //TODO: batch rename
                            for path in selected {
                                let parent = match path.parent() {
                                    Some(some) => some.to_path_buf(),
                                    None => continue,
                                };
                                let name = match path.file_name().and_then(|x| x.to_str()) {
                                    Some(some) => some.to_string(),
                                    None => continue,
                                };
                                let dir = path.is_dir();
                                self.dialog_pages.push_back(DialogPage::RenameItem {
                                    from: path,
                                    parent,
                                    name,
                                    dir,
                                });
                            }
                            return widget::text_input::focus(self.dialog_text_input.clone());
                        }
                    }
                }
            }
            Message::ReplaceResult(replace_result) => {
                if let Some(dialog_page) = self.dialog_pages.pop_front() {
                    match dialog_page {
                        DialogPage::Replace { tx, .. } => {
                            return Task::perform(
                                async move {
                                    let _ = tx.send(replace_result).await;
                                    cosmic::action::none()
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
                let mut trash_items = Vec::new();
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        for item in items.iter() {
                            if item.selected {
                                match &item.metadata {
                                    ItemMetadata::Trash { entry, .. } => {
                                        trash_items.push(entry.clone());
                                    }
                                    _ => {
                                        //TODO: error on trying to restore non-trash file?
                                    }
                                }
                            }
                        }
                    }
                }
                if !trash_items.is_empty() {
                    self.operation(Operation::Restore { items: trash_items });
                }
            }
            Message::ScrollTab(scroll_speed) => {
                let entity = self.tab_model.active();
                return self.update(Message::TabMessage(
                    Some(entity),
                    tab::Message::ScrollTab((scroll_speed as f32) / 10.0),
                ));
            }
            Message::SearchActivate => {
                return if self.search_get().is_none() {
                    self.search_set_active(Some(String::new()))
                } else {
                    widget::text_input::focus(self.search_id.clone())
                };
            }
            Message::SearchClear => {
                return self.search_set_active(None);
            }
            Message::SearchInput(input) => {
                return self.search_set_active(Some(input));
            }
            Message::SetShowDetails(show_details) => {
                config_set!(show_details, show_details);
                return self.update_config();
            }
            Message::SetTypeToSearch(type_to_search) => {
                config_set!(type_to_search, type_to_search);
                return self.update_config();
            }
            Message::SystemThemeModeChange(_theme_mode) => {
                return self.update_config();
            }
            Message::TabActivate(entity) => {
                self.tab_model.activate(entity);

                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    self.activate_nav_model_location(&tab.location.clone());
                }
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
                    if let Some(window_id) = &self.window_id_opt {
                        return window::close(*window_id);
                    }
                }

                return Task::batch([self.update_title(), self.update_watcher()]);
            }
            Message::TabConfig(config) => {
                if config != self.config.tab {
                    config_set!(tab, config);
                    return self.update_config();
                }
            }
            Message::ToggleFoldersFirst => {
                let mut config = self.config.tab;
                config.folders_first = !config.folders_first;
                return self.update(Message::TabConfig(config));
            }
            Message::TabMessage(entity_opt, tab_message) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());

                //TODO: move to Task?
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
                        }
                        tab::Command::AddToSidebar(path) => {
                            let mut favorites = self.config.favorites.clone();
                            let favorite = Favorite::from_path(path);
                            if !favorites.iter().any(|f| f == &favorite) {
                                favorites.push(favorite);
                            }
                            config_set!(favorites, favorites);
                            commands.push(self.update_config());
                        }
                        tab::Command::AutoScroll(scroll_speed) => {
                            // converting an f32 to an i16 here by multiplying by 10 and casting to i16
                            // further resolution isn't necessary
                            if let Some(scroll_speed_float) = scroll_speed {
                                self.auto_scroll_speed = Some((scroll_speed_float * 10.0) as i16);
                            } else {
                                self.auto_scroll_speed = None;
                            }
                        }
                        tab::Command::ChangeLocation(tab_title, tab_path, selection_paths) => {
                            self.activate_nav_model_location(&tab_path);

                            self.tab_model.text_set(entity, tab_title);
                            commands.push(Task::batch([
                                self.update_title(),
                                self.update_watcher(),
                                self.update_tab(entity, tab_path, selection_paths),
                            ]));
                        }
                        tab::Command::Delete(paths) => {
                            self.operation(Operation::Delete { paths });
                        }
                        tab::Command::DropFiles(to, from) => {
                            commands.push(self.update(Message::PasteContents(to, from)));
                        }
                        tab::Command::EmptyTrash => {
                            self.dialog_pages.push_back(DialogPage::EmptyTrash);
                        }
                        #[cfg(feature = "desktop")]
                        tab::Command::ExecEntryAction(entry, action) => {
                            App::exec_entry_action(entry, action);
                        }
                        tab::Command::Iced(iced_command) => {
                            commands.push(iced_command.0.map(move |x| {
                                cosmic::action::app(Message::TabMessage(Some(entity), x))
                            }));
                        }
                        tab::Command::OpenFile(path) => self.open_file(&path),
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
                        tab::Command::OpenTrash => {
                            //TODO: use handler for x-scheme-handler/trash and open trash:///
                            let mut command = process::Command::new("cosmic-files");
                            command.arg("--trash");
                            match spawn_detached(&mut command) {
                                Ok(()) => {}
                                Err(err) => {
                                    log::warn!("failed to run cosmic-files --trash: {}", err)
                                }
                            }
                        }
                        tab::Command::Preview(kind) => {
                            self.context_page = ContextPage::Preview(Some(entity), kind);
                            self.set_show_context(true);
                        }
                        tab::Command::SetOpenWith(mime, id) => {
                            //TODO: this will block for a few ms, run in background?
                            self.mime_app_cache.set_default(mime, id);
                        }
                        tab::Command::WindowDrag => {
                            if let Some(window_id) = &self.window_id_opt {
                                commands.push(window::drag(*window_id));
                            }
                        }
                        tab::Command::WindowToggleMaximize => {
                            if let Some(window_id) = &self.window_id_opt {
                                commands.push(window::toggle_maximize(*window_id));
                            }
                        }
                    }
                }
                return Task::batch(commands);
            }
            Message::TabNew => {
                let active = self.tab_model.active();
                let location = match self.tab_model.data::<Tab>(active) {
                    Some(tab) => tab.location.clone(),
                    None => Location::Path(home_dir()),
                };
                return self.open_tab(location, true, None);
            }
            Message::TabRescan(entity, location, parent_item_opt, items, selection_paths) => {
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    if location == tab.location {
                        tab.parent_item_opt = parent_item_opt;
                        tab.set_items(items);
                        if let Some(selection_paths) = selection_paths {
                            tab.select_paths(selection_paths);
                        }
                    }
                }
            }
            Message::TabView(entity_opt, view) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data_mut::<Tab>(entity) {
                    tab.config.view = view;
                }
                let mut config = self.config.tab;
                config.view = view;
                return self.update(Message::TabConfig(config));
            }
            Message::ToggleContextPage(context_page) => {
                //TODO: ensure context menus are closed
                if self.context_page == context_page
                    || matches!(self.context_page, ContextPage::Preview(_, _))
                {
                    self.set_show_context(!self.core.window.show_context);
                } else {
                    self.set_show_context(true);
                }
                self.context_page = context_page;
                // Preview status is preserved across restarts
                if matches!(self.context_page, ContextPage::Preview(_, _)) {
                    return cosmic::task::message(cosmic::action::app(Message::SetShowDetails(
                        self.core.window.show_context,
                    )));
                }
            }
            Message::Undo(_id) => {
                // TODO: undo
            }
            Message::UndoTrash(id, recently_trashed) => {
                self.toasts.remove(id);

                let mut paths = Vec::with_capacity(recently_trashed.len());
                let icon_sizes = self.config.tab.icon_sizes;

                return cosmic::task::future(async move {
                    match tokio::task::spawn_blocking(move || Location::Trash.scan(icon_sizes))
                        .await
                    {
                        Ok((_parent_item_opt, items)) => {
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
            Message::UndoTrashStart(items) => {
                self.operation(Operation::Restore { items });
            }
            Message::WindowClose => {
                if let Some(window_id) = self.window_id_opt.take() {
                    return Task::batch([
                        window::close(window_id),
                        Task::perform(
                            async move { cosmic::action::app(Message::MaybeExit) },
                            |x| x,
                        ),
                    ]);
                }
            }
            Message::WindowUnfocus => {
                let tab_entity = self.tab_model.active();
                if let Some(tab) = self.tab_model.data_mut::<Tab>(tab_entity) {
                    tab.context_menu = None;
                }
            }
            Message::WindowCloseRequested(id) => {
                self.remove_window(&id);
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
            Message::ZoomDefault(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                let mut config = self.config.tab;
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    match tab.config.view {
                        tab::View::List => config.icon_sizes.list = 100.try_into().unwrap(),
                        tab::View::Grid => config.icon_sizes.grid = 100.try_into().unwrap(),
                    }
                }
                return self.update(Message::TabConfig(config));
            }
            Message::ZoomIn(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
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
                let mut config = self.config.tab;
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    match tab.config.view {
                        tab::View::List => zoom_in(&mut config.icon_sizes.list, 50, 500),
                        tab::View::Grid => zoom_in(&mut config.icon_sizes.grid, 50, 500),
                    }
                }
                return self.update(Message::TabConfig(config));
            }
            Message::ZoomOut(entity_opt) => {
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
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
                let mut config = self.config.tab;
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    match tab.config.view {
                        tab::View::List => zoom_out(&mut config.icon_sizes.list, 50, 500),
                        tab::View::Grid => zoom_out(&mut config.icon_sizes.grid, 50, 500),
                    }
                }
                return self.update(Message::TabConfig(config));
            }
            Message::DndEnterNav(entity) => {
                if let Some(location) = self.nav_model.data::<Location>(entity) {
                    self.nav_dnd_hover = Some((location.clone(), Instant::now()));
                    let location = location.clone();
                    return Task::perform(tokio::time::sleep(HOVER_DURATION), move |_| {
                        cosmic::Action::App(Message::DndHoverLocTimeout(location.clone()))
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
                            Task::none()
                        }
                        _ => {
                            log::warn!("Copy to trash is not supported.");
                            Task::none()
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
                        return Task::batch([
                            self.update_title(),
                            self.update_watcher(),
                            self.update_tab(entity, location, None),
                        ]);
                    }
                }
            }
            Message::DndEnterTab(entity) => {
                self.tab_dnd_hover = Some((entity, Instant::now()));
                return Task::perform(tokio::time::sleep(HOVER_DURATION), move |_| {
                    cosmic::Action::App(Message::DndHoverTabTimeout(entity))
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
                            Task::none()
                        }
                        _ => {
                            log::warn!("Copy to trash is not supported.");
                            Task::none()
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
                    if let Some(mounter) = MOUNTERS.get(&data.0) {
                        return mounter
                            .unmount(data.1.clone())
                            .map(|_| cosmic::action::none());
                    }
                }
            }
            Message::NavBarContext(entity) => {
                // Close location editing if enabled
                let tab_entity = self.tab_model.active();
                if let Some(tab) = self.tab_model.data_mut::<Tab>(tab_entity) {
                    tab.edit_location = None;
                }

                self.nav_bar_context_id = entity;
            }
            Message::NavMenuAction(action) => match action {
                NavMenuAction::Open(entity) => {
                    if let Some(path) = self
                        .nav_model
                        .data::<Location>(entity)
                        .and_then(|x| x.path_opt())
                        .map(|x| x.to_path_buf())
                    {
                        self.open_file(&path);
                    }
                }
                NavMenuAction::OpenWith(entity) => {
                    if let Some(path) = self
                        .nav_model
                        .data::<Location>(entity)
                        .and_then(|x| x.path_opt())
                        .map(|x| x.to_path_buf())
                    {
                        match tab::item_from_path(&path, IconSizes::default()) {
                            Ok(item) => {
                                return self.update(Message::DialogPush(DialogPage::OpenWith {
                                    path: path.to_path_buf(),
                                    mime: item.mime.clone(),
                                    selected: 0,
                                    store_opt: "x-scheme-handler/mime"
                                        .parse::<mime_guess::Mime>()
                                        .ok()
                                        .and_then(|mime| {
                                            self.mime_app_cache.get(&mime).first().cloned()
                                        }),
                                }));
                            }
                            Err(err) => {
                                log::warn!("failed to get item for path {:?}: {}", path, err);
                            }
                        }
                    }
                }
                NavMenuAction::OpenInNewTab(entity) => {
                    match self.nav_model.data::<Location>(entity) {
                        Some(Location::Network(ref uri, ref display_name)) => {
                            return self.open_tab(
                                Location::Network(uri.clone(), display_name.clone()),
                                false,
                                None,
                            );
                        }
                        Some(Location::Path(ref path)) => {
                            return self.open_tab(Location::Path(path.clone()), false, None);
                        }
                        Some(Location::Recents) => {
                            return self.open_tab(Location::Recents, false, None);
                        }
                        Some(Location::Trash) => {
                            return self.open_tab(Location::Trash, false, None);
                        }
                        _ => {}
                    }
                }

                // Open the selected path in a new cosmic-files window.
                NavMenuAction::OpenInNewWindow(entity) => 'open_in_new_window: {
                    if let Some(location) = self.nav_model.data::<Location>(entity) {
                        match env::current_exe() {
                            Ok(exe) => {
                                let mut command = process::Command::new(&exe);
                                match location {
                                    Location::Path(path) => {
                                        command.arg(path);
                                    }
                                    Location::Trash => {
                                        command.arg("--trash");
                                    }
                                    Location::Network(..) => {
                                        command.arg("--network");
                                    }
                                    Location::Recents => {
                                        command.arg("--recents");
                                    }
                                    _ => {
                                        log::error!(
                                            "unsupported location for open in new window: {:?}",
                                            location
                                        );
                                        break 'open_in_new_window;
                                    }
                                };
                                match command.spawn() {
                                    Ok(_child) => {}
                                    Err(err) => {
                                        log::error!("failed to execute {:?}: {}", exe, err);
                                    }
                                };
                            }
                            Err(err) => {
                                log::error!("failed to get current executable path: {}", err);
                            }
                        }
                    }
                }

                NavMenuAction::Preview(entity) => {
                    if let Some(path) = self
                        .nav_model
                        .data::<Location>(entity)
                        .and_then(|location| location.path_opt())
                    {
                        match tab::item_from_path(path, IconSizes::default()) {
                            Ok(item) => {
                                self.context_page = ContextPage::Preview(
                                    None,
                                    PreviewKind::Custom(PreviewItem(item)),
                                );
                                self.set_show_context(true);
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
                        if let Some(old_surface_id) =
                            self.surface_ids.insert(output.clone(), surface_id)
                        {
                            //TODO: remove old surface?
                            log::warn!(
                                "output {}: already had surface ID {:?}",
                                output.id(),
                                old_surface_id
                            );
                        }

                        let display = match output_info_opt {
                            Some(output_info) => match output_info.name {
                                Some(output_name) => {
                                    self.surface_names.insert(surface_id, output_name.clone());
                                    output_name
                                }
                                None => {
                                    log::warn!("output {}: no output name", output.id());
                                    String::new()
                                }
                            },
                            None => {
                                log::warn!("output {}: no output info", output.id());
                                String::new()
                            }
                        };

                        let (entity, command) = self.open_tab_entity(
                            Location::Desktop(crate::desktop_dir(), display, self.config.desktop),
                            false,
                            None,
                        );
                        self.windows.insert(surface_id, WindowKind::Desktop(entity));
                        return Task::batch([
                            command,
                            get_layer_surface(SctkLayerSurfaceSettings {
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
                                exclusive_zone: 0,
                                size_limits: Limits::NONE.min_width(1.0).min_height(1.0),
                            }),
                            #[cfg(feature = "wayland")]
                            overlap_notify(surface_id, true),
                        ]);
                    }
                    OutputEvent::Removed => {
                        log::info!("output {}: removed", output.id());
                        match self.surface_ids.remove(&output) {
                            Some(surface_id) => {
                                self.remove_window(&surface_id);
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
                return Task::perform(async move { cosmic }, cosmic::action::cosmic);
            }
            Message::None => {}
            #[cfg(all(feature = "desktop", feature = "wayland"))]
            Message::Overlap(overlap_notify_event, w_id) => match overlap_notify_event {
                OverlapNotifyEvent::OverlapLayerAdd {
                    identifier,
                    namespace,
                    logical_rect,
                    exclusive,
                    ..
                } => {
                    if exclusive > 0 || namespace == "Dock" || namespace == "Panel" {
                        self.overlap.insert(identifier, (w_id, logical_rect));
                        self.handle_overlap();
                    }
                }
                OverlapNotifyEvent::OverlapLayerRemove { identifier } => {
                    self.overlap.remove(&identifier);
                    self.handle_overlap();
                }
                _ => {}
            },
            Message::Size(size) => {
                self.size = Some(size);
                self.handle_overlap();
            }
            #[cfg(all(feature = "desktop", feature = "wayland"))]
            Message::Focused(id) => {
                if let Some(w) = self.windows.get(&id) {
                    match w {
                        WindowKind::Desktop(entity) => self.tab_model.activate(*entity),
                        _ => {}
                    };
                }
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
        }

        Task::none()
    }

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match &self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            ),
            ContextPage::EditHistory => context_drawer::context_drawer(
                self.edit_history(),
                Message::ToggleContextPage(ContextPage::EditHistory),
            )
            .title(fl!("edit-history")),
            ContextPage::NetworkDrive => {
                let mut text_input =
                    widget::text_input(fl!("enter-server-address"), &self.network_drive_input);
                let button = if self.network_drive_connecting.is_some() {
                    widget::button::standard(fl!("connecting"))
                } else {
                    text_input = text_input
                        .on_input(Message::NetworkDriveInput)
                        .on_submit(|_| Message::NetworkDriveSubmit);
                    widget::button::standard(fl!("connect")).on_press(Message::NetworkDriveSubmit)
                };
                context_drawer::context_drawer(
                    self.network_drive(),
                    Message::ToggleContextPage(ContextPage::NetworkDrive),
                )
                .title(fl!("add-network-drive"))
                .header(text_input)
                .footer(widget::row::with_children(vec![
                    widget::horizontal_space().into(),
                    button.into(),
                ]))
            }
            ContextPage::Preview(entity_opt, kind) => {
                let mut actions = Vec::with_capacity(3);
                let entity = entity_opt.unwrap_or_else(|| self.tab_model.active());
                if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                    if let Some(items) = tab.items_opt() {
                        for item in items.iter() {
                            if item.selected {
                                actions.extend(item.preview_header().into_iter().map(|element| {
                                    element.map(move |x| Message::TabMessage(Some(entity), x))
                                }));
                                break;
                            }
                        }
                    }
                };
                context_drawer::context_drawer(
                    self.preview(entity_opt, kind, true)
                        .map(move |x| Message::TabMessage(Some(entity), x)),
                    Message::ToggleContextPage(ContextPage::Preview(Some(entity), kind.clone())),
                )
                .header_actions(actions)
            }
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
        })
    }

    fn dialog(&self) -> Option<Element<Message>> {
        //TODO: should gallery view just be a dialog?
        let entity = self.tab_model.active();
        if let Some(tab) = self.tab_model.data::<Tab>(entity) {
            if tab.gallery {
                return Some(
                    tab.gallery_view()
                        .map(move |x| Message::TabMessage(Some(entity), x)),
                );
            }
        }

        let Some(dialog_page) = self.dialog_pages.front() else {
            return None;
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
                password,
            } => {
                let mut dialog = widget::dialog().title(fl!("create-archive"));

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
                dialog = dialog
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
                                            password: password.clone(),
                                        })
                                    })
                                    .on_submit_maybe(
                                        complete_maybe.clone().map(|maybe| move |_| maybe.clone()),
                                    )
                                    .into(),
                                Element::from(widget::dropdown(
                                    archive_types,
                                    selected,
                                    move |index| index,
                                ))
                                .map(|index| {
                                    Message::DialogUpdate(DialogPage::Compress {
                                        paths: paths.clone(),
                                        to: to.clone(),
                                        name: name.clone(),
                                        archive_type: archive_types[index],
                                        password: password.clone(),
                                    })
                                })
                                .into(),
                            ])
                            .align_y(Alignment::Center)
                            .spacing(space_xxs)
                            .into(),
                        ])
                        .spacing(space_xxs),
                    );

                if *archive_type == ArchiveType::Zip {
                    let password_unwrapped = password.clone().unwrap_or_else(String::default);
                    dialog = dialog.control(widget::column::with_children(vec![
                        widget::text::body(fl!("password")).into(),
                        widget::text_input("", password_unwrapped)
                            .password()
                            .on_input(move |password_unwrapped| {
                                Message::DialogUpdate(DialogPage::Compress {
                                    paths: paths.clone(),
                                    to: to.clone(),
                                    name: name.clone(),
                                    archive_type: *archive_type,
                                    password: Some(password_unwrapped),
                                })
                            })
                            .on_submit_maybe(
                                complete_maybe.clone().map(|maybe| move |_| maybe.clone()),
                            )
                            .into(),
                    ]));
                }

                dialog
            }
            DialogPage::EmptyTrash => widget::dialog()
                .title(fl!("empty-trash"))
                .body(fl!("empty-trash-warning"))
                .primary_action(
                    widget::button::suggested(fl!("empty-trash")).on_press(Message::DialogComplete),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),
            DialogPage::FailedOperation(id) => {
                //TODO: try next dialog page (making sure index is used by Dialog messages)?
                let (operation, _, err) = self.failed_operations.get(id)?;

                //TODO: nice description of error
                widget::dialog()
                    .title("Failed operation")
                    .body(format!("{:#?}\n{}", operation, err))
                    .icon(widget::icon::from_name("dialog-error").size(64))
                    //TODO: retry action
                    .primary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
            }
            DialogPage::ExtractPassword { id, password } => {
                widget::dialog()
                    .title(fl!("extract-password-required"))
                    .icon(widget::icon::from_name("dialog-error").size(64))
                    .control(widget::text_input("", password).password().on_input(
                        move |password| {
                            Message::DialogUpdate(DialogPage::ExtractPassword { id: *id, password })
                        },
                    ))
                    .primary_action(
                        widget::button::suggested(fl!("extract-here"))
                            .on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
            }
            DialogPage::MountError {
                mounter_key: _,
                item: _,
                error,
            } => widget::dialog()
                .title(fl!("mount-error"))
                .body(error)
                .icon(widget::icon::from_name("dialog-error").size(64))
                .primary_action(
                    widget::button::standard(fl!("try-again")).on_press(Message::DialogComplete),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),
            DialogPage::NetworkAuth {
                mounter_key,
                uri,
                auth,
                auth_tx,
            } => {
                //TODO: use URI!
                let mut controls = Vec::with_capacity(4);
                let mut id_assigned = false;

                if let Some(username) = &auth.username_opt {
                    //TODO: what should submit do?
                    let mut input = widget::text_input(fl!("username"), username)
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
                        .on_submit(|_| Message::DialogComplete);
                    if !id_assigned {
                        input = input.id(self.dialog_text_input.clone());
                        id_assigned = true;
                    }
                    controls.push(input.into());
                }

                if let Some(domain) = &auth.domain_opt {
                    //TODO: what should submit do?
                    let mut input = widget::text_input(fl!("domain"), domain)
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
                        .on_submit(|_| Message::DialogComplete);
                    if !id_assigned {
                        input = input.id(self.dialog_text_input.clone());
                        id_assigned = true;
                    }
                    controls.push(input.into());
                }

                if let Some(password) = &auth.password_opt {
                    //TODO: what should submit do?
                    //TODO: button for showing password
                    let mut input = widget::secure_input(fl!("password"), password, None, true)
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
                        .on_submit(|_| Message::DialogComplete);
                    if !id_assigned {
                        input = input.id(self.dialog_text_input.clone());
                    }
                    controls.push(input.into());
                }

                if let Some(remember) = &auth.remember_opt {
                    //TODO: what should submit do?
                    //TODO: button for showing password
                    controls.push(
                        widget::checkbox(fl!("remember-password"), *remember)
                            .on_toggle(move |value| {
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

                let mut widget = widget::dialog()
                    .title(title)
                    .body(body)
                    .control(widget::column::with_children(controls).spacing(space_s))
                    .primary_action(
                        widget::button::suggested(fl!("connect")).on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    );

                if let Some(_anonymous) = &auth.anonymous_opt {
                    widget = widget.tertiary_action(
                        widget::button::text(fl!("connect-anonymously")).on_press(
                            Message::DialogUpdateComplete(DialogPage::NetworkAuth {
                                mounter_key: *mounter_key,
                                uri: uri.clone(),
                                auth: MounterAuth {
                                    anonymous_opt: Some(true),
                                    ..auth.clone()
                                },
                                auth_tx: auth_tx.clone(),
                            }),
                        ),
                    );
                }

                widget
            }
            DialogPage::NetworkError {
                mounter_key: _,
                uri: _,
                error,
            } => widget::dialog()
                .title(fl!("network-drive-error"))
                .body(error)
                .icon(widget::icon::from_name("dialog-error").size(64))
                .primary_action(
                    widget::button::standard(fl!("try-again")).on_press(Message::DialogComplete),
                )
                .secondary_action(
                    widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                ),
            DialogPage::NewItem { parent, name, dir } => {
                let mut dialog = widget::dialog().title(if *dir {
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
                                .on_submit_maybe(
                                    complete_maybe.clone().map(|maybe| move |_| maybe.clone()),
                                )
                                .into(),
                        ])
                        .spacing(space_xxs),
                    )
            }
            DialogPage::OpenWith {
                path,
                mime,
                selected,
                store_opt,
                ..
            } => {
                let name = match path.file_name() {
                    Some(file_name) => file_name.to_str(),
                    None => path.as_os_str().to_str(),
                };

                let mut column = widget::list_column();
                let available_programs = self.get_programs_for_mime(&mime);
                let item_height = 32.0;
                let mut displayed_default = false;

                for (i, app) in available_programs.iter().enumerate() {
                    column = column.add(
                        widget::button::custom(
                            widget::row::with_children(vec![
                                widget::icon(app.icon.clone()).size(32).into(),
                                if app.is_default && !displayed_default {
                                    displayed_default = true;
                                    widget::text::body(fl!(
                                        "default-app",
                                        name = Some(app.name.as_str())
                                    ))
                                    .into()
                                } else {
                                    widget::text::body(app.name.to_string()).into()
                                },
                                widget::horizontal_space().into(),
                                if *selected == i {
                                    widget::icon::from_name("checkbox-checked-symbolic")
                                        .size(16)
                                        .into()
                                } else {
                                    widget::Space::with_width(Length::Fixed(16.0)).into()
                                },
                            ])
                            .spacing(space_s)
                            .height(Length::Fixed(item_height))
                            .align_y(Alignment::Center),
                        )
                        .width(Length::Fill)
                        .class(theme::Button::MenuItem)
                        .on_press(Message::OpenWithSelection(i)),
                    );
                }

                let mut dialog = widget::dialog()
                    .title(fl!("open-with-title", name = name))
                    .primary_action(
                        widget::button::suggested(fl!("open")).on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(
                        widget::scrollable(column).height(if let Some(size) = self.size {
                            let max_size = size.height - 256.0;
                            // (32 (item_height) + 5.0 (custom button padding)) + (space_xxs (list item spacing) * 2)
                            let scrollable_height = available_programs.len() as f32
                                * (item_height + 5.0 + (2.0 * space_xxs as f32));

                            if scrollable_height > max_size {
                                Length::Fill
                            } else {
                                Length::Shrink
                            }
                        } else {
                            Length::Fill
                        }),
                    );

                if let Some(app) = store_opt {
                    dialog = dialog.tertiary_action(
                        widget::button::text(fl!("browse-store", store = app.name.as_str()))
                            .on_press(Message::OpenWithBrowse),
                    );
                }

                dialog
            }
            DialogPage::PermanentlyDelete { paths } => {
                let target = if paths.len() == 1 {
                    format!(
                        "« {} »",
                        paths[0]
                            .file_name()
                            .map(std::ffi::OsStr::to_string_lossy)
                            .unwrap_or_else(|| paths[0].to_string_lossy())
                    )
                } else {
                    fl!("selected-items", items = paths.len())
                };

                widget::dialog()
                    .title(fl!("permanently-delete-question"))
                    .icon(icon::from_name("dialog-warning").size(32))
                    .primary_action(
                        widget::button::destructive(fl!("delete"))
                            .on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::standard(fl!("cancel")).on_press(Message::DialogCancel),
                    )
                    .control(widget::text(fl!(
                        "permanently-delete-warning",
                        nb_items = paths.len(),
                        target = target
                    )))
            }
            DialogPage::RenameItem {
                from,
                parent,
                name,
                dir,
            } => {
                //TODO: combine logic with NewItem
                let mut dialog = widget::dialog().title(if *dir {
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
                    if from != &path && path.exists() {
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
                                .on_submit_maybe(
                                    complete_maybe.clone().map(|maybe| move |_| maybe.clone()),
                                )
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
                let military_time = self.config.tab.military_time;
                let dialog = widget::dialog()
                    .title(fl!("replace-title", filename = to.name.as_str()))
                    .body(fl!("replace-warning-operation"))
                    .control(
                        to.replace_view(fl!("original-file"), IconSizes::default(), military_time)
                            .map(|x| Message::TabMessage(None, x)),
                    )
                    .control(
                        from.replace_view(fl!("replace-with"), IconSizes::default(), military_time)
                            .map(|x| Message::TabMessage(None, x)),
                    )
                    .primary_action(widget::button::suggested(fl!("replace")).on_press(
                        Message::ReplaceResult(ReplaceResult::Replace(*apply_to_all)),
                    ));
                if *multiple {
                    dialog
                        .control(
                            widget::checkbox(fl!("apply-to-all"), *apply_to_all).on_toggle(
                                |apply_to_all| {
                                    Message::DialogUpdate(DialogPage::Replace {
                                        from: from.clone(),
                                        to: to.clone(),
                                        multiple: *multiple,
                                        apply_to_all,
                                        tx: tx.clone(),
                                    })
                                },
                            ),
                        )
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
            DialogPage::SetExecutableAndLaunch { path } => {
                let name = match path.file_name() {
                    Some(file_name) => file_name.to_str(),
                    None => path.as_os_str().to_str(),
                };
                widget::dialog()
                    .title(fl!("set-executable-and-launch"))
                    .primary_action(
                        widget::button::text(fl!("set-and-launch"))
                            .class(theme::Button::Suggested)
                            .on_press(Message::DialogComplete),
                    )
                    .secondary_action(
                        widget::button::text(fl!("cancel"))
                            .class(theme::Button::Standard)
                            .on_press(Message::DialogCancel),
                    )
                    .control(widget::text::text(fl!(
                        "set-executable-and-launch-description",
                        name = name
                    )))
            }
            DialogPage::FavoritePathError { path, .. } => widget::dialog()
                .title(fl!("favorite-path-error"))
                .body(fl!(
                    "favorite-path-error-description",
                    path = path.as_os_str().to_str()
                ))
                .icon(widget::icon::from_name("dialog-error").size(64))
                .primary_action(
                    widget::button::destructive(fl!("remove")).on_press(Message::DialogComplete),
                )
                .secondary_action(
                    widget::button::standard(fl!("keep")).on_press(Message::DialogCancel),
                ),
        };
        Some(dialog.into())
    }

    fn footer(&self) -> Option<Element<Message>> {
        if self.progress_operations.is_empty() {
            return None;
        }

        let cosmic_theme::Spacing {
            space_xs, space_s, ..
        } = theme::active().cosmic().spacing;

        let mut title = String::new();
        let mut total_progress = 0.0;
        let mut count = 0;
        let mut all_paused = true;
        for (_id, (op, controller)) in self.pending_operations.iter() {
            if !controller.is_paused() {
                all_paused = false;
            }
            if op.show_progress_notification() {
                let progress = controller.progress();
                if title.is_empty() {
                    title = op.pending_text(progress, controller.state());
                }
                total_progress += progress;
                count += 1;
            }
        }
        let running = count;
        // Adjust the progress bar so it does not jump around when operations finish
        for id in self.progress_operations.iter() {
            if self.complete_operations.contains_key(id) {
                total_progress += 1.0;
                count += 1;
            }
        }
        let finished = count - running;
        total_progress /= count as f32;
        if running > 1 {
            if finished > 0 {
                title = fl!(
                    "operations-running-finished",
                    running = running,
                    finished = finished,
                    percent = ((total_progress * 100.0) as i32)
                );
            } else {
                title = fl!(
                    "operations-running",
                    running = running,
                    percent = ((total_progress * 100.0) as i32)
                );
            }
        }

        //TODO: get height from theme?
        let progress_bar_height = Length::Fixed(4.0);
        let progress_bar =
            widget::progress_bar(0.0..=1.0, total_progress).height(progress_bar_height);

        let container = widget::layer_container(widget::column::with_children(vec![
            widget::row::with_children(vec![
                progress_bar.into(),
                if all_paused {
                    widget::tooltip(
                        widget::button::icon(widget::icon::from_name(
                            "media-playback-start-symbolic",
                        ))
                        .on_press(Message::PendingPauseAll(false))
                        .padding(8),
                        widget::text::body(fl!("resume")),
                        widget::tooltip::Position::Top,
                    )
                    .into()
                } else {
                    widget::tooltip(
                        widget::button::icon(widget::icon::from_name(
                            "media-playback-pause-symbolic",
                        ))
                        .on_press(Message::PendingPauseAll(true))
                        .padding(8),
                        widget::text::body(fl!("pause")),
                        widget::tooltip::Position::Top,
                    )
                    .into()
                },
                widget::tooltip(
                    widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                        .on_press(Message::PendingCancelAll)
                        .padding(8),
                    widget::text::body(fl!("cancel")),
                    widget::tooltip::Position::Top,
                )
                .into(),
            ])
            .align_y(Alignment::Center)
            .into(),
            widget::text::body(title).into(),
            widget::Space::with_height(space_s).into(),
            widget::row::with_children(vec![
                widget::button::link(fl!("details"))
                    .on_press(Message::ToggleContextPage(ContextPage::EditHistory))
                    .padding(0)
                    .trailing_icon(true)
                    .into(),
                widget::horizontal_space().into(),
                widget::button::standard(fl!("dismiss"))
                    .on_press(Message::PendingDismiss)
                    .into(),
            ])
            .align_y(Alignment::Center)
            .into(),
        ]))
        .padding([8, space_xs])
        .layer(cosmic_theme::Layer::Primary);

        Some(container.into())
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu::menu_bar(
            self.tab_model.active_data::<Tab>(),
            &self.config,
            &self.modifiers,
            &self.key_binds,
        )]
    }

    fn header_end(&self) -> Vec<Element<Self::Message>> {
        let mut elements = Vec::with_capacity(2);

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

        elements
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let cosmic_theme::Spacing {
            space_xxs, space_s, ..
        } = theme::active().cosmic().spacing;

        let mut tab_column = widget::column::with_capacity(4);

        if self.core.is_condensed() {
            if let Some(term) = self.search_get() {
                tab_column = tab_column.push(
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
                .class(style::Container::Background)
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
        tab_column = tab_column.push(widget::toaster(&self.toasts, widget::horizontal_space()));

        let content: Element<_> = tab_column.into();

        // Uncomment to debug layout:
        //content.explain(cosmic::iced::Color::WHITE)
        content
    }

    fn view_window(&self, id: WindowId) -> Element<Self::Message> {
        let content = match self.windows.get(&id) {
            Some(WindowKind::Desktop(entity)) => {
                let mut tab_column = widget::column::with_capacity(3);

                let tab_view = match self.tab_model.data::<Tab>(*entity) {
                    Some(tab) => tab
                        .view(&self.key_binds)
                        .map(move |message| Message::TabMessage(Some(*entity), message)),
                    None => widget::vertical_space().into(),
                };

                let mut popover = widget::popover(tab_view);
                if let Some(dialog) = self.dialog() {
                    popover = popover.popup(dialog);
                }
                tab_column = tab_column.push(popover);

                // The toaster is added on top of an empty element to ensure that it does not override context menus
                tab_column =
                    tab_column.push(widget::toaster(&self.toasts, widget::horizontal_space()));
                return if let Some(margin) = self.margin.get(&id) {
                    if margin.0 >= 0. || margin.2 >= 0. {
                        tab_column = widget::column::with_children(vec![
                            vertical_space().height(margin.0).into(),
                            tab_column.into(),
                            vertical_space().height(margin.2).into(),
                        ])
                    }
                    if margin.1 >= 0. || margin.3 >= 0. {
                        Element::from(widget::row::with_children(vec![
                            horizontal_space().width(margin.1).into(),
                            tab_column.into(),
                            horizontal_space().width(margin.3).into(),
                        ]))
                    } else {
                        tab_column.into()
                    }
                } else {
                    tab_column.into()
                };
            }
            Some(WindowKind::DesktopViewOptions) => self.desktop_view_options(),
            Some(WindowKind::Preview(entity_opt, kind)) => self
                .preview(entity_opt, kind, false)
                .map(|x| Message::TabMessage(*entity_opt, x)),
            Some(WindowKind::FileDialog(..)) => match &self.file_dialog_opt {
                Some(dialog) => return dialog.view(id),
                None => widget::text("Unknown window ID").into(),
            },
            None => {
                //TODO: distinct views per monitor in desktop mode
                return self.view_main().map(|message| match message {
                    cosmic::Action::App(app) => app,
                    cosmic::Action::Cosmic(cosmic) => Message::Cosmic(cosmic),
                    cosmic::Action::None => Message::None,
                });
            }
        };

        widget::container(widget::scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .class(theme::Container::WindowBackground)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        struct ThemeSubscription;
        struct WatcherSubscription;
        struct TrashWatcherSubscription;

        let mut subscriptions = vec![
            event::listen_with(|event, status, window_id| match event {
                Event::Keyboard(KeyEvent::KeyPressed {
                    key,
                    modifiers,
                    text,
                    ..
                }) => match status {
                    event::Status::Ignored => Some(Message::Key(modifiers, key, text)),
                    event::Status::Captured => None,
                },
                Event::Keyboard(KeyEvent::ModifiersChanged(modifiers)) => {
                    Some(Message::ModifiersChanged(modifiers))
                }
                Event::Window(WindowEvent::Unfocused) => Some(Message::WindowUnfocus),
                #[cfg(all(feature = "desktop", feature = "wayland"))]
                Event::Window(WindowEvent::Focused) => Some(Message::Focused(window_id)),
                Event::Window(WindowEvent::CloseRequested) => Some(Message::WindowClose),
                Event::Window(WindowEvent::Opened { position: _, size }) => {
                    Some(Message::Size(size))
                }
                Event::Window(WindowEvent::Resized(s)) => Some(Message::Size(s)),
                #[cfg(feature = "wayland")]
                Event::PlatformSpecific(event::PlatformSpecific::Wayland(wayland_event)) => {
                    match wayland_event {
                        WaylandEvent::Output(output_event, output) => {
                            Some(Message::OutputEvent(output_event, output))
                        }
                        #[cfg(feature = "desktop")]
                        WaylandEvent::OverlapNotify(event) => {
                            Some(Message::Overlap(event, window_id))
                        }
                        _ => None,
                    }
                }
                Event::Mouse(CursorMoved { position: pos }) => Some(Message::CursorMoved(pos)),
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
                }),
            ),
            Subscription::run_with_id(
                TypeId::of::<TrashWatcherSubscription>(),
                stream::channel(25, |mut output| async move {
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
                }),
            ),
        ];

        if let Some(scroll_speed) = self.auto_scroll_speed {
            subscriptions.push(
                iced::time::every(time::Duration::from_millis(10))
                    .with(scroll_speed)
                    .map(|(scroll_speed, _)| Message::ScrollTab(scroll_speed)),
            );
        }

        for (key, mounter) in MOUNTERS.iter() {
            subscriptions.push(
                mounter.subscription().with(*key).map(
                    |(key, mounter_message)| match mounter_message {
                        MounterMessage::Items(items) => Message::MounterItems(key, items),
                        MounterMessage::MountResult(item, res) => {
                            Message::MountResult(key, item, res)
                        }
                        MounterMessage::NetworkAuth(uri, auth, auth_tx) => {
                            Message::NetworkAuth(key, uri, auth, auth_tx)
                        }
                        MounterMessage::NetworkResult(uri, res) => {
                            Message::NetworkResult(key, uri, res)
                        }
                    },
                ),
            );
        }

        if !self.pending_operations.is_empty() {
            //TODO: inhibit suspend/shutdown?

            if self.window_id_opt.is_some() {
                // Refresh progress when window is open and operations are in progress
                subscriptions.push(window::frames().map(|_| Message::None));
            } else {
                // Handle notification when window is closed and operations are in progress
                #[cfg(feature = "notify")]
                {
                    struct NotificationSubscription;
                    subscriptions.push(Subscription::run_with_id(
                        TypeId::of::<NotificationSubscription>(),
                        stream::channel(1, move |msg_tx| async move {
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

                            std::future::pending().await
                        }),
                    ));
                }
            }
        }

        for (id, (pending_operation, controller)) in self.pending_operations.iter() {
            //TODO: use recipe?
            let id = *id;
            let pending_operation = pending_operation.clone();
            let controller = controller.clone();
            subscriptions.push(Subscription::run_with_id(
                id,
                stream::channel(16, move |msg_tx| async move {
                    let msg_tx = Arc::new(tokio::sync::Mutex::new(msg_tx));
                    match pending_operation.perform(&msg_tx, controller).await {
                        Ok(result_paths) => {
                            let _ = msg_tx
                                .lock()
                                .await
                                .send(Message::PendingComplete(id, result_paths))
                                .await;
                        }
                        Err(err) => {
                            let _ = msg_tx
                                .lock()
                                .await
                                .send(Message::PendingError(id, err))
                                .await;
                        }
                    }

                    std::future::pending().await
                }),
            ));
        }

        let mut selected_preview = None;
        if self.core.window.show_context {
            if let ContextPage::Preview(entity_opt, PreviewKind::Selected) = self.context_page {
                selected_preview = Some(entity_opt.unwrap_or_else(|| self.tab_model.active()));
            }
        }
        for entity in self.tab_model.iter() {
            if let Some(tab) = self.tab_model.data::<Tab>(entity) {
                subscriptions.push(
                    tab.subscription(selected_preview == Some(entity))
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
        let (parent_item_opt, items) = location.scan(IconSizes::default());
        let mut tab = Tab::new(location, TabConfig::default());
        tab.parent_item_opt = parent_item_opt;
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
        let Some(tab_path) = tab.location.path_opt() else {
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

    /// Assert that tab's items are equal to a path's entries.
    pub fn assert_eq_tab_path_contents(tab: &Tab, path: &Path) {
        let Some(tab_path) = tab.location.path_opt() else {
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
                .zip(tab.items_opt().unwrap_or(&empty))
                .all(|(a, b)| eq_path_item(&a, b)),
            "Path ({}) and Tab path ({}) don't have equal contents",
            path.display(),
            tab_path.display()
        );
    }
}
