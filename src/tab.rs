use cosmic::iced::clipboard::dnd::DndAction;
use cosmic::iced::Border;
use cosmic::iced_core::widget::tree;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::{vertical_space, Id, Widget};
use cosmic::{
    cosmic_theme,
    iced::{
        alignment::{Horizontal, Vertical},
        futures::SinkExt,
        keyboard::Modifiers,
        subscription::{self, Subscription},
        //TODO: export in cosmic::widget
        widget::{
            horizontal_rule,
            scrollable::{AbsoluteOffset, Viewport},
        },
        Alignment,
        Color,
        ContentFit,
        Length,
        Point,
        Rectangle,
        Size,
    },
    theme, widget, Element,
};
use mime_guess::{mime, Mime};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::{
    cell::Cell,
    cmp::Ordering,
    collections::HashMap,
    fmt,
    fs::{self, Metadata},
    num::NonZeroU16,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::clipboard::{ClipboardCopy, ClipboardKind, ClipboardPaste};
use crate::{
    app::{self, Action},
    config::{IconSizes, TabConfig, ICON_SCALE_MAX, ICON_SIZE_GRID},
    dialog::DialogKind,
    fl, menu,
    mime_app::{mime_apps, MimeApp},
    mime_icon::{mime_for_path, mime_icon},
    mouse_area,
};
use cosmic::widget::{DndDestination, DndSource};

pub const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
pub const HOVER_DURATION: Duration = Duration::from_millis(1600);

//TODO: adjust for locales?
const TIME_FORMAT: &'static str = "%a %-d %b %-Y %r";
static SPECIAL_DIRS: Lazy<HashMap<PathBuf, &'static str>> = Lazy::new(|| {
    let mut special_dirs = HashMap::new();
    if let Some(dir) = dirs::document_dir() {
        special_dirs.insert(dir, "folder-documents");
    }
    if let Some(dir) = dirs::download_dir() {
        special_dirs.insert(dir, "folder-download");
    }
    if let Some(dir) = dirs::audio_dir() {
        special_dirs.insert(dir, "folder-music");
    }
    if let Some(dir) = dirs::picture_dir() {
        special_dirs.insert(dir, "folder-pictures");
    }
    if let Some(dir) = dirs::public_dir() {
        special_dirs.insert(dir, "folder-publicshare");
    }
    if let Some(dir) = dirs::template_dir() {
        special_dirs.insert(dir, "folder-templates");
    }
    if let Some(dir) = dirs::video_dir() {
        special_dirs.insert(dir, "folder-videos");
    }
    if let Some(dir) = dirs::desktop_dir() {
        special_dirs.insert(dir, "user-desktop");
    }
    if let Some(dir) = dirs::home_dir() {
        special_dirs.insert(dir, "user-home");
    }
    special_dirs
});

fn button_appearance(
    theme: &theme::Theme,
    selected: bool,
    focused: bool,
    accent: bool,
) -> widget::button::Appearance {
    let cosmic = theme.cosmic();
    let mut appearance = widget::button::Appearance::new();
    if selected {
        if accent {
            appearance.background = Some(Color::from(cosmic.accent_color()).into());
            appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
            appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
        } else {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        }
    }
    if focused && accent {
        appearance.outline_width = 1.0;
        appearance.outline_color = Color::from(cosmic.accent_color());
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }
    appearance.border_radius = cosmic.radius_s().into();
    appearance
}

fn button_style(selected: bool, accent: bool) -> theme::Button {
    //TODO: move to libcosmic?
    theme::Button::Custom {
        active: Box::new(move |focused, theme| button_appearance(theme, selected, focused, accent)),
        disabled: Box::new(move |theme| button_appearance(theme, selected, false, accent)),
        hovered: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent)
        }),
        pressed: Box::new(move |focused, theme| {
            button_appearance(theme, selected, focused, accent)
        }),
    }
}

pub fn folder_icon(path: &PathBuf, icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(SPECIAL_DIRS.get(path).map_or("folder", |x| *x))
        .size(icon_size)
        .handle()
}

pub fn folder_icon_symbolic(path: &PathBuf, icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(format!(
        "{}-symbolic",
        SPECIAL_DIRS.get(path).map_or("folder", |x| *x)
    ))
    .size(icon_size)
    .handle()
}

pub fn trash_icon_symbolic(icon_size: u16) -> widget::icon::Handle {
    let full = match trash::os_limited::list() {
        Ok(entries) => !entries.is_empty(),
        Err(_err) => false,
    };
    widget::icon::from_name(if full {
        "user-trash-full-symbolic"
    } else {
        "user-trash-symbolic"
    })
    .size(icon_size)
    .handle()
}

//TODO: translate, add more levels?
fn format_size(size: u64) -> String {
    const KB: u64 = 1000;
    const MB: u64 = 1000 * KB;
    const GB: u64 = 1000 * MB;
    const TB: u64 = 1000 * GB;

    if size >= TB {
        format!("{:.1} TB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

#[cfg(not(target_os = "windows"))]
fn hidden_attribute(_metadata: &Metadata) -> bool {
    false
}

#[cfg(target_os = "windows")]
fn hidden_attribute(metadata: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    // https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants
    const FILE_ATTRIBUTE_HIDDEN: u32 = 2;
    metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN == FILE_ATTRIBUTE_HIDDEN
}

pub fn scan_path(tab_path: &PathBuf, sizes: IconSizes) -> Vec<Item> {
    let mut items = Vec::new();
    match fs::read_dir(tab_path) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!("failed to read entry in {:?}: {}", tab_path, err);
                        continue;
                    }
                };

                let name = match entry.file_name().into_string() {
                    Ok(ok) => ok,
                    Err(name_os) => {
                        log::warn!(
                            "failed to parse entry in {:?}: {:?} is not valid UTF-8",
                            tab_path,
                            name_os,
                        );
                        continue;
                    }
                };

                let metadata = match entry.metadata() {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!(
                            "failed to read metadata for entry in {:?}: {}",
                            tab_path,
                            err
                        );
                        continue;
                    }
                };

                let hidden = name.starts_with(".") || hidden_attribute(&metadata);

                let path = entry.path();

                let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) =
                    if metadata.is_dir() {
                        (
                            //TODO: make this a static
                            "inode/directory".parse().unwrap(),
                            folder_icon(&path, sizes.grid()),
                            folder_icon(&path, sizes.list()),
                            folder_icon(&path, sizes.list_condensed()),
                        )
                    } else {
                        let mime = mime_for_path(&path);
                        (
                            mime.clone(),
                            mime_icon(mime.clone(), sizes.grid()),
                            mime_icon(mime.clone(), sizes.list()),
                            mime_icon(mime, sizes.list_condensed()),
                        )
                    };

                let open_with = mime_apps(&mime);

                let thumbnail_opt = if mime.type_() == mime::IMAGE {
                    if mime.subtype() == mime::SVG {
                        Some(ItemThumbnail::Svg)
                    } else {
                        None
                    }
                } else {
                    Some(ItemThumbnail::NotImage)
                };

                let children = if metadata.is_dir() {
                    //TODO: calculate children in the background (and make it cancellable?)
                    match fs::read_dir(&path) {
                        Ok(entries) => entries.count(),
                        Err(err) => {
                            log::warn!("failed to read directory {:?}: {}", path, err);
                            0
                        }
                    }
                } else {
                    0
                };

                items.push(Item {
                    name,
                    metadata: ItemMetadata::Path { metadata, children },
                    hidden,
                    path_opt: Some(path),
                    mime,
                    icon_handle_grid,
                    icon_handle_list,
                    icon_handle_list_condensed,
                    open_with,
                    thumbnail_opt,
                    button_id: widget::Id::unique(),
                    pos_opt: Cell::new(None),
                    rect_opt: Cell::new(None),
                    selected: false,
                    overlaps_drag_rect: false,
                });
            }
        }
        Err(err) => {
            log::warn!("failed to read directory {:?}: {}", tab_path, err);
        }
    }
    items.sort_by(|a, b| match (a.metadata.is_dir(), b.metadata.is_dir()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => lexical_sort::natural_lexical_cmp(&a.name, &b.name),
    });
    items
}

// This config statement is from trash::os_limited, inverted
#[cfg(not(any(
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
)))]
pub fn scan_trash(_sizes: IconSizes) -> Vec<Item> {
    log::warn!("viewing trash not supported on this platform");
    Vec::new()
}

// This config statement is from trash::os_limited
#[cfg(any(
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
))]
pub fn scan_trash(sizes: IconSizes) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    match trash::os_limited::list() {
        Ok(entries) => {
            for entry in entries {
                let metadata = match trash::os_limited::metadata(&entry) {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!("failed to get metadata for trash item {:?}: {}", entry, err);
                        continue;
                    }
                };

                let original_path = entry.original_path();
                let name = entry.name.clone();

                let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) =
                    match metadata.size {
                        trash::TrashItemSize::Entries(_) => (
                            //TODO: make this a static
                            "inode/directory".parse().unwrap(),
                            folder_icon(&original_path, sizes.grid()),
                            folder_icon(&original_path, sizes.list()),
                            folder_icon(&original_path, sizes.list_condensed()),
                        ),
                        trash::TrashItemSize::Bytes(_) => {
                            //TODO: do not use original path
                            let mime = mime_for_path(&original_path);
                            (
                                mime.clone(),
                                mime_icon(mime.clone(), sizes.grid()),
                                mime_icon(mime.clone(), sizes.list()),
                                mime_icon(mime, sizes.list_condensed()),
                            )
                        }
                    };

                items.push(Item {
                    name,
                    metadata: ItemMetadata::Trash { metadata, entry },
                    hidden: false,
                    path_opt: None,
                    mime,
                    icon_handle_grid,
                    icon_handle_list,
                    icon_handle_list_condensed,
                    open_with: Vec::new(),
                    thumbnail_opt: Some(ItemThumbnail::NotImage),
                    button_id: widget::Id::unique(),
                    pos_opt: Cell::new(None),
                    rect_opt: Cell::new(None),
                    selected: false,
                    overlaps_drag_rect: false,
                });
            }
        }
        Err(err) => {
            log::warn!("failed to read trash items: {}", err);
        }
    }
    items.sort_by(|a, b| match (a.metadata.is_dir(), b.metadata.is_dir()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => lexical_sort::natural_lexical_cmp(&a.name, &b.name),
    });
    items
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Location {
    Path(PathBuf),
    Trash,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::Trash => write!(f, "trash"),
        }
    }
}

impl Location {
    pub fn scan(&self, sizes: IconSizes) -> Vec<Item> {
        match self {
            Self::Path(path) => scan_path(path, sizes),
            Self::Trash => scan_trash(sizes),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Command {
    Action(Action),
    ChangeLocation(String, Location),
    EmptyTrash,
    FocusButton(widget::Id),
    FocusTextInput(widget::Id),
    OpenFile(PathBuf),
    Scroll(widget::Id, AbsoluteOffset),
    DropFiles(PathBuf, ClipboardPaste),
    Timeout(Duration, Message),
    MoveToTrash(Vec<PathBuf>),
}

#[derive(Clone, Debug)]
pub enum Message {
    Click(Option<usize>),
    DoubleClick(Option<usize>),
    ClickRelease(Option<usize>),
    DragEnd(Option<usize>),
    Config(TabConfig),
    ContextAction(Action),
    ContextMenu(Option<Point>),
    Drag(Option<Rectangle>),
    EditLocation(Option<Location>),
    EmptyTrash,
    GoNext,
    GoPrevious,
    ItemDown,
    ItemLeft,
    ItemRight,
    ItemUp,
    Location(Location),
    LocationUp,
    Open,
    Resize(Size),
    RightClick(usize),
    Scroll(Viewport),
    SelectAll,
    Thumbnail(PathBuf, ItemThumbnail),
    ToggleFoldersFirst,
    ToggleShowHidden,
    View(View),
    ToggleSort(HeadingOptions),
    Drop(Option<(Location, ClipboardPaste)>),
    DndHover(Location),
    DndEnter(Location),
    DndLeave(Location),
    ZoomDefault,
    ZoomIn,
    ZoomOut,
}

#[derive(Clone, Debug)]
pub enum ItemMetadata {
    Path {
        metadata: Metadata,
        children: usize,
    },
    Trash {
        metadata: trash::TrashItemMetadata,
        entry: trash::TrashItem,
    },
}

impl ItemMetadata {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Path { metadata, .. } => metadata.is_dir(),
            Self::Trash { metadata, .. } => match metadata.size {
                trash::TrashItemSize::Entries(_) => true,
                trash::TrashItemSize::Bytes(_) => false,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum ItemThumbnail {
    NotImage,
    Rgba(image::RgbaImage),
    Svg,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub metadata: ItemMetadata,
    pub hidden: bool,
    pub path_opt: Option<PathBuf>,
    pub mime: Mime,
    pub icon_handle_grid: widget::icon::Handle,
    pub icon_handle_list: widget::icon::Handle,
    pub icon_handle_list_condensed: widget::icon::Handle,
    pub open_with: Vec<MimeApp>,
    pub thumbnail_opt: Option<ItemThumbnail>,
    pub button_id: widget::Id,
    pub pos_opt: Cell<Option<(usize, usize)>>,
    pub rect_opt: Cell<Option<Rectangle>>,
    pub selected: bool,
    pub overlaps_drag_rect: bool,
}

impl Item {
    fn preview(&self, sizes: IconSizes) -> Element<'static, app::Message> {
        // This loads the image only if thumbnailing worked
        let icon = widget::icon::icon(self.icon_handle_grid.clone())
            .content_fit(ContentFit::Contain)
            .size(sizes.grid())
            .into();
        match self
            .thumbnail_opt
            .as_ref()
            .unwrap_or(&ItemThumbnail::NotImage)
        {
            ItemThumbnail::NotImage => icon,
            ItemThumbnail::Rgba(_) => {
                if let Some(path) = &self.path_opt {
                    widget::image::viewer(widget::image::Handle::from_path(path))
                        .min_scale(1.0)
                        .into()
                } else {
                    icon
                }
            }
            ItemThumbnail::Svg => {
                if let Some(path) = &self.path_opt {
                    widget::Svg::from_path(path).into()
                } else {
                    icon
                }
            }
        }
    }

    pub fn open_with_view(&self, sizes: IconSizes) -> Element<app::Message> {
        let cosmic_theme::Spacing {
            space_xs,
            space_xxxs,
            ..
        } = theme::active().cosmic().spacing;

        let mut column = widget::column().spacing(space_xxxs);

        column = column.push(widget::row::with_children(vec![
            widget::horizontal_space(Length::Fill).into(),
            self.preview(sizes),
            widget::horizontal_space(Length::Fill).into(),
        ]));

        column = column.push(widget::text::heading(&self.name));

        column = column.push(widget::text(format!("Type: {}", self.mime)));

        if let Some(path) = &self.path_opt {
            for app in self.open_with.iter() {
                column = column.push(
                    widget::button(
                        widget::row::with_children(vec![
                            widget::icon(app.icon.clone()).into(),
                            if app.is_default {
                                widget::text(fl!("default-app", name = app.name.as_str())).into()
                            } else {
                                widget::text(&app.name).into()
                            },
                        ])
                        .spacing(space_xs),
                    )
                    //TODO: do not clone so much?
                    .on_press(app::Message::OpenWith(path.clone(), app.clone()))
                    .padding(space_xs)
                    .width(Length::Fill),
                );
            }
        }

        column.into()
    }

    pub fn property_view(&self, sizes: IconSizes) -> Element<'static, app::Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;

        let mut column = widget::column().spacing(space_xxxs);

        column = column.push(widget::row::with_children(vec![
            widget::horizontal_space(Length::Fill).into(),
            self.preview(sizes),
            widget::horizontal_space(Length::Fill).into(),
        ]));

        column = column.push(widget::text::heading(self.name.clone()));

        column = column.push(widget::text(format!("Type: {}", self.mime)));

        //TODO: translate!
        //TODO: correct display of folder size?
        match &self.metadata {
            ItemMetadata::Path { metadata, children } => {
                if metadata.is_dir() {
                    column = column.push(widget::text(format!("Items: {}", children)));
                } else {
                    column = column.push(widget::text(format!(
                        "Size: {}",
                        format_size(metadata.len())
                    )));
                }

                if let Ok(time) = metadata.created() {
                    column = column.push(widget::text(format!(
                        "Created: {}",
                        chrono::DateTime::<chrono::Local>::from(time).format(TIME_FORMAT)
                    )));
                }

                if let Ok(time) = metadata.modified() {
                    column = column.push(widget::text(format!(
                        "Modified: {}",
                        chrono::DateTime::<chrono::Local>::from(time).format(TIME_FORMAT)
                    )));
                }

                if let Ok(time) = metadata.accessed() {
                    column = column.push(widget::text(format!(
                        "Accessed: {}",
                        chrono::DateTime::<chrono::Local>::from(time).format(TIME_FORMAT)
                    )));
                }
            }
            ItemMetadata::Trash { .. } => {
                //TODO: trash metadata
            }
        }

        column.into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum View {
    Grid,
    List,
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, PartialOrd, Ord, Eq, Deserialize, Serialize)]
pub enum HeadingOptions {
    Name = 0,
    Modified,
    Size,
}

impl fmt::Display for HeadingOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeadingOptions::Name => write!(f, "{}", fl!("name")),
            HeadingOptions::Modified => write!(f, "{}", fl!("modified")),
            HeadingOptions::Size => write!(f, "{}", fl!("size")),
        }
    }
}

impl HeadingOptions {
    pub fn names() -> Vec<String> {
        vec![
            HeadingOptions::Name.to_string(),
            HeadingOptions::Modified.to_string(),
            HeadingOptions::Size.to_string(),
        ]
    }
}

// TODO when creating items, pass <Arc<SelectedItems>> to each item
// as a drag data, so that when dnd is initiated, they are all included
#[derive(Clone)]
pub struct Tab {
    //TODO: make more items private
    pub location: Location,
    pub context_menu: Option<Point>,
    pub dialog: Option<DialogKind>,
    pub scroll_opt: Option<AbsoluteOffset>,
    pub size_opt: Option<Size>,
    pub edit_location: Option<Location>,
    pub edit_location_id: widget::Id,
    pub history_i: usize,
    pub history: Vec<Location>,
    pub config: TabConfig,
    pub(crate) items_opt: Option<Vec<Item>>,
    pub dnd_hovered: Option<(Location, Instant)>,
    scrollable_id: widget::Id,
    select_focus: Option<usize>,
    select_range: Option<(usize, usize)>,
    cached_selected: RefCell<Option<bool>>,
    clicked: Option<usize>,
    selected_clicked: bool,
}

impl Tab {
    pub fn new(location: Location, config: TabConfig) -> Self {
        let history = vec![location.clone()];
        Self {
            location,
            context_menu: None,
            dialog: None,
            scroll_opt: None,
            size_opt: None,
            edit_location: None,
            edit_location_id: widget::Id::unique(),
            history_i: 0,
            history,
            config,
            items_opt: None,
            scrollable_id: widget::Id::unique(),
            select_focus: None,
            select_range: None,
            cached_selected: RefCell::new(None),
            clicked: None,
            dnd_hovered: None,
            selected_clicked: false,
        }
    }

    pub fn title(&self) -> String {
        //TODO: better title
        match &self.location {
            Location::Path(path) => {
                format!("{}", path.display())
            }
            Location::Trash => {
                fl!("trash")
            }
        }
    }

    pub fn items_opt(&self) -> Option<&Vec<Item>> {
        self.items_opt.as_ref()
    }

    pub fn set_items(&mut self, items: Vec<Item>) {
        self.items_opt = Some(items);
    }

    pub fn select_all(&mut self) {
        *self.cached_selected.borrow_mut() = None;
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                if !self.config.show_hidden && item.hidden {
                    item.selected = false;
                    continue;
                }
                item.selected = true;
            }
        }
    }

    pub fn select_none(&mut self) -> bool {
        *self.cached_selected.borrow_mut() = None;
        self.select_focus = None;
        let mut had_selection = false;
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                if item.selected {
                    item.selected = false;
                    had_selection = true;
                }
            }
        }
        had_selection
    }

    pub fn select_name(&mut self, name: &str) {
        *self.cached_selected.borrow_mut() = None;
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                if item.name == name {
                    item.selected = true;
                } else {
                    item.selected = false;
                }
            }
        }
    }

    fn select_position(&mut self, row: usize, col: usize, mod_shift: bool) -> bool {
        *self.cached_selected.borrow_mut() = None;
        let mut start = (row, col);
        let mut end = (row, col);
        if mod_shift {
            if self.select_focus.is_none() || self.select_range.is_none() {
                // Set select range to initial state if necessary
                self.select_range = self.select_focus.map(|i| (i, i));
            }

            if let Some(pos) = self.select_range_start_pos_opt() {
                if pos.0 < row || (pos.0 == row && pos.1 < col) {
                    start = pos;
                } else {
                    end = pos;
                }
            }
        }

        let mut found = false;
        if let Some(ref mut items) = self.items_opt {
            for (i, item) in items.iter_mut().enumerate() {
                item.selected = false;
                let pos = match item.pos_opt.get() {
                    Some(some) => some,
                    None => continue,
                };
                if pos.0 < start.0 || (pos.0 == start.0 && pos.1 < start.1) {
                    // Before start
                    continue;
                }
                if pos.0 > end.0 || (pos.0 == end.0 && pos.1 > end.1) {
                    // After end
                    continue;
                }
                if pos == (row, col) {
                    // Update focus if this is what we wanted to select
                    self.select_focus = Some(i);
                    self.select_range = if mod_shift {
                        self.select_range.map(|r| (r.0, i))
                    } else {
                        Some((i, i))
                    };
                    found = true;
                }
                item.selected = true;
            }
        }
        found
    }

    pub fn select_rect(&mut self, rect: Rectangle, mod_ctrl: bool, mod_shift: bool) {
        *self.cached_selected.borrow_mut() = None;
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                let was_overlapped = item.overlaps_drag_rect;
                item.overlaps_drag_rect = item
                    .rect_opt
                    .get()
                    .map(|r| r.intersects(&rect))
                    .unwrap_or(false);

                item.selected = if mod_ctrl || mod_shift {
                    if was_overlapped == item.overlaps_drag_rect {
                        item.selected
                    } else {
                        !item.selected
                    }
                } else {
                    item.overlaps_drag_rect
                };
            }
        }
    }

    pub fn select_focus_id(&self) -> Option<widget::Id> {
        let items = self.items_opt.as_ref()?;
        let item = items.get(self.select_focus?)?;
        Some(item.button_id.clone())
    }

    fn select_focus_pos_opt(&self) -> Option<(usize, usize)> {
        let items = self.items_opt.as_ref()?;
        let item = items.get(self.select_focus?)?;
        item.pos_opt.get()
    }

    fn select_focus_scroll(&mut self) -> Option<AbsoluteOffset> {
        let items = self.items_opt.as_ref()?;
        let item = items.get(self.select_focus?)?;
        let rect = item.rect_opt.get()?;

        //TODO: move to function
        let visible_rect = {
            let point = match self.scroll_opt {
                Some(offset) => Point::new(0.0, offset.y),
                None => Point::new(0.0, 0.0),
            };
            let size = self.size_opt.unwrap_or_else(|| Size::new(0.0, 0.0));
            Rectangle::new(point, size)
        };

        if rect.y < visible_rect.y {
            // Scroll up to rect
            self.scroll_opt = Some(AbsoluteOffset { x: 0.0, y: rect.y });
            self.scroll_opt
        } else if (rect.y + rect.height) > (visible_rect.y + visible_rect.height) {
            // Scroll down to rect
            self.scroll_opt = Some(AbsoluteOffset {
                x: 0.0,
                y: rect.y + rect.height - visible_rect.height,
            });
            self.scroll_opt
        } else {
            // Do not scroll
            None
        }
    }

    fn select_range_start_pos_opt(&self) -> Option<(usize, usize)> {
        let items = self.items_opt.as_ref()?;
        let item = items.get(self.select_range.map(|r| r.0)?)?;
        item.pos_opt.get()
    }

    fn select_first_pos_opt(&self) -> Option<(usize, usize)> {
        *self.cached_selected.borrow_mut() = None;
        let items = self.items_opt.as_ref()?;
        let mut first = None;
        for item in items.iter() {
            if !item.selected {
                continue;
            }

            let (row, col) = match item.pos_opt.get() {
                Some(some) => some,
                None => continue,
            };

            first = Some(match first {
                Some((first_row, first_col)) => {
                    if row < first_row {
                        (row, col)
                    } else if row == first_row {
                        (row, col.min(first_row))
                    } else {
                        (first_row, first_col)
                    }
                }
                None => (row, col),
            });
        }
        first
    }

    fn select_last_pos_opt(&self) -> Option<(usize, usize)> {
        *self.cached_selected.borrow_mut() = None;
        let items = self.items_opt.as_ref()?;
        let mut last = None;
        for item in items.iter() {
            if !item.selected {
                continue;
            }

            let (row, col) = match item.pos_opt.get() {
                Some(some) => some,
                None => continue,
            };

            last = Some(match last {
                Some((last_row, last_col)) => {
                    if row > last_row {
                        (row, col)
                    } else if row == last_row {
                        (row, col.max(last_row))
                    } else {
                        (last_row, last_col)
                    }
                }
                None => (row, col),
            });
        }
        last
    }

    pub fn change_location(&mut self, location: &Location, history_i_opt: Option<usize>) {
        self.location = location.clone();
        self.items_opt = None;
        self.select_focus = None;
        self.edit_location = None;
        if let Some(history_i) = history_i_opt {
            // Navigating in history
            self.history_i = history_i;
        } else {
            // Truncate history to remove next entries
            self.history.truncate(self.history_i + 1);

            // Push to the front of history
            self.history_i = self.history.len();
            self.history.push(location.clone());
        }
    }

    pub fn update(&mut self, message: Message, modifiers: Modifiers) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut cd = None;
        let mut history_i_opt = None;
        let mod_ctrl = modifiers.contains(Modifiers::CTRL)
            && self.dialog.as_ref().map_or(true, |x| x.multiple());
        let mod_shift = modifiers.contains(Modifiers::SHIFT)
            && self.dialog.as_ref().map_or(true, |x| x.multiple());
        match message {
            Message::ClickRelease(click_i_opt) => {
                if click_i_opt == self.clicked.take() {
                    return commands;
                }
                self.context_menu = None;
                if let Some(ref mut items) = self.items_opt {
                    for (i, item) in items.iter_mut().enumerate() {
                        if mod_ctrl {
                            if Some(i) == click_i_opt && item.selected {
                                item.selected = false;
                                self.select_range = None;
                            }
                        } else if Some(i) != click_i_opt {
                            item.selected = false;
                        }
                    }
                }
            }
            Message::DragEnd(_) => {
                self.clicked = None;
                if let Some(ref mut items) = self.items_opt {
                    for item in items.iter_mut() {
                        item.overlaps_drag_rect = false;
                    }
                }
            }
            Message::DoubleClick(click_i_opt) => {
                if let Some(clicked_item) = self
                    .items_opt
                    .as_ref()
                    .and_then(|items| click_i_opt.and_then(|click_i| items.get(click_i)))
                {
                    if let Some(path) = &clicked_item.path_opt {
                        if clicked_item.metadata.is_dir() {
                            cd = Some(Location::Path(path.clone()));
                        } else {
                            commands.push(Command::OpenFile(path.clone()));
                        }
                    } else {
                        log::warn!("no path for item {:?}", clicked_item);
                    }
                } else {
                    log::warn!("no item for click index {:?}", click_i_opt);
                }
            }
            Message::Click(click_i_opt) => {
                self.selected_clicked = false;
                self.context_menu = None;
                if click_i_opt.is_none() {
                    self.clicked = click_i_opt;
                }

                if mod_shift {
                    if let Some(click_i) = click_i_opt {
                        self.select_range = self
                            .select_range
                            .map_or(Some((click_i, click_i)), |r| Some((r.0, click_i)));
                        if let Some(range) = self.select_range {
                            let min = range.0.min(range.1);
                            let max = range.0.max(range.1);
                            if let Some(ref mut items) = self.items_opt {
                                for (i, item) in items.iter_mut().enumerate() {
                                    item.selected = i >= min && i <= max;
                                }
                            }
                        }
                        self.clicked = click_i_opt;
                        self.select_focus = click_i_opt;
                        self.selected_clicked = true;
                    }
                } else {
                    let dont_unset = mod_ctrl
                        || self.column_sort().is_some_and(|l| {
                            l.iter()
                                .any(|(e_i, e)| Some(e_i) == click_i_opt.as_ref() && e.selected)
                        });
                    *self.cached_selected.borrow_mut() = None;
                    if let Some(ref mut items) = self.items_opt {
                        for (i, item) in items.iter_mut().enumerate() {
                            if Some(i) == click_i_opt {
                                // Filter out selection if it does not match dialog kind
                                if let Some(dialog) = &self.dialog {
                                    let item_is_dir = item.metadata.is_dir();
                                    if item_is_dir != dialog.is_dir() {
                                        // Allow selecting folder if dialog is for files to make it
                                        // possible to double click
                                        //TODO: clear any other selection when selecting a folder
                                        if !item_is_dir {
                                            continue;
                                        }
                                    }
                                }
                                if !item.selected {
                                    self.clicked = click_i_opt;
                                    item.selected = true;
                                    self.select_range = Some((i, i));
                                }

                                self.selected_clicked = true;
                            } else if !dont_unset && item.selected {
                                self.clicked = click_i_opt;
                                item.selected = false;
                            }
                        }
                    }
                    if self.select_focus.take().is_some() {
                        // Unfocus currently focused button
                        commands.push(Command::FocusButton(widget::Id::unique()));
                    }
                }
            }
            Message::Config(config) => {
                self.config = config;
            }
            Message::ContextAction(action) => {
                // Close context menu
                self.context_menu = None;

                commands.push(Command::Action(action));
            }
            Message::ContextMenu(point_opt) => {
                self.context_menu = point_opt;
            }
            Message::Drag(rect_opt) => match rect_opt {
                Some(rect) => {
                    self.context_menu = None;
                    self.select_rect(rect, mod_ctrl, mod_shift);
                    if self.select_focus.take().is_some() {
                        // Unfocus currently focused button
                        commands.push(Command::FocusButton(widget::Id::unique()));
                    }
                }
                None => {}
            },
            Message::EditLocation(edit_location) => {
                if self.edit_location.is_none() && edit_location.is_some() {
                    commands.push(Command::FocusTextInput(self.edit_location_id.clone()));
                }
                self.edit_location = edit_location;
            }
            Message::EmptyTrash => {
                commands.push(Command::EmptyTrash);
            }
            Message::GoNext => {
                if let Some(history_i) = self.history_i.checked_add(1) {
                    if let Some(location) = self.history.get(history_i) {
                        cd = Some(location.clone());
                        history_i_opt = Some(history_i);
                    }
                }
            }
            Message::GoPrevious => {
                if let Some(history_i) = self.history_i.checked_sub(1) {
                    if let Some(location) = self.history.get(history_i) {
                        cd = Some(location.clone());
                        history_i_opt = Some(history_i);
                    }
                }
            }
            Message::ItemDown => {
                if let Some((row, col)) = self.select_focus_pos_opt().or(self.select_last_pos_opt())
                {
                    if self.select_focus.is_none() {
                        // Select last item in current selection to focus it.
                        self.select_position(row, col, mod_shift);
                    }

                    //TODO: Shift modifier should select items in between
                    // Try to select item in next row
                    if !self.select_position(row + 1, col, mod_shift) {
                        // Ensure current item is still selected if there are no other items
                        self.select_position(row, col, mod_shift);
                    }
                } else {
                    // Select first item
                    //TODO: select first in scroll
                    self.select_position(0, 0, mod_shift);
                }
                if let Some(offset) = self.select_focus_scroll() {
                    commands.push(Command::Scroll(self.scrollable_id.clone(), offset));
                }
                if let Some(id) = self.select_focus_id() {
                    commands.push(Command::FocusButton(id));
                }
            }
            Message::ItemLeft => {
                if let Some((row, col)) =
                    self.select_focus_pos_opt().or(self.select_first_pos_opt())
                {
                    if self.select_focus.is_none() {
                        // Select first item in current selection to focus it.
                        self.select_position(row, col, mod_shift);
                    }

                    // Try to select previous item in current row
                    if !col
                        .checked_sub(1)
                        .map_or(false, |col| self.select_position(row, col, mod_shift))
                    {
                        // Try to select last item in previous row
                        if !row.checked_sub(1).map_or(false, |row| {
                            let mut col = 0;
                            if let Some(ref items) = self.items_opt {
                                for item in items.iter() {
                                    match item.pos_opt.get() {
                                        Some((item_row, item_col)) if item_row == row => {
                                            col = col.max(item_col);
                                        }
                                        _ => continue,
                                    }
                                }
                            }
                            self.select_position(row, col, mod_shift)
                        }) {
                            // Ensure current item is still selected if there are no other items
                            self.select_position(row, col, mod_shift);
                        }
                    }
                } else {
                    // Select first item
                    //TODO: select first in scroll
                    self.select_position(0, 0, mod_shift);
                }
                if let Some(offset) = self.select_focus_scroll() {
                    commands.push(Command::Scroll(self.scrollable_id.clone(), offset));
                }
                if let Some(id) = self.select_focus_id() {
                    commands.push(Command::FocusButton(id));
                }
            }
            Message::ItemRight => {
                if let Some((row, col)) = self.select_focus_pos_opt().or(self.select_last_pos_opt())
                {
                    if self.select_focus.is_none() {
                        // Select last item in current selection to focus it.
                        self.select_position(row, col, mod_shift);
                    }
                    // Try to select next item in current row
                    if !self.select_position(row, col + 1, mod_shift) {
                        // Try to select first item in next row
                        if !self.select_position(row + 1, 0, mod_shift) {
                            // Ensure current item is still selected if there are no other items
                            self.select_position(row, col, mod_shift);
                        }
                    }
                } else {
                    // Select first item
                    //TODO: select first in scroll
                    self.select_position(0, 0, mod_shift);
                }
                if let Some(offset) = self.select_focus_scroll() {
                    commands.push(Command::Scroll(self.scrollable_id.clone(), offset));
                }
                if let Some(id) = self.select_focus_id() {
                    commands.push(Command::FocusButton(id));
                }
            }
            Message::ItemUp => {
                if let Some((row, col)) =
                    self.select_focus_pos_opt().or(self.select_first_pos_opt())
                {
                    if self.select_focus.is_none() {
                        // Select first item in current selection to focus it.
                        self.select_position(row, col, mod_shift);
                    }

                    //TODO: Shift modifier should select items in between
                    // Try to select item in last row
                    if !row
                        .checked_sub(1)
                        .map_or(false, |row| self.select_position(row, col, mod_shift))
                    {
                        // Ensure current item is still selected if there are no other items
                        self.select_position(row, col, mod_shift);
                    }
                } else {
                    // Select first item
                    //TODO: select first in scroll
                    self.select_position(0, 0, mod_shift);
                }
                if let Some(offset) = self.select_focus_scroll() {
                    commands.push(Command::Scroll(self.scrollable_id.clone(), offset));
                }
                if let Some(id) = self.select_focus_id() {
                    commands.push(Command::FocusButton(id));
                }
            }
            Message::Location(location) => {
                // Workaround to support favorited files
                match &location {
                    Location::Path(path) => {
                        if path.is_dir() {
                            cd = Some(location);
                        } else {
                            commands.push(Command::OpenFile(path.clone()));
                        }
                    }
                    Location::Trash => {
                        cd = Some(location);
                    }
                }
            }
            Message::LocationUp => {
                // Sets location to the path's parent
                // Does nothing if path is root or location is Trash
                if let Location::Path(ref path) = self.location {
                    if let Some(parent) = path.parent() {
                        cd = Some(Location::Path(parent.to_owned()));
                    }
                }
            }
            Message::Open => {
                if let Some(ref mut items) = self.items_opt {
                    for item in items.iter() {
                        if item.selected {
                            if let Some(path) = &item.path_opt {
                                if path.is_dir() {
                                    //TODO: allow opening multiple tabs?
                                    cd = Some(Location::Path(path.clone()));
                                } else {
                                    commands.push(Command::OpenFile(path.clone()));
                                }
                            } else {
                                //TODO: open properties?
                            }
                        }
                    }
                }
            }
            Message::Resize(size) => {
                self.size_opt = Some(size);
            }
            Message::RightClick(click_i) => {
                *self.cached_selected.borrow_mut() = None;
                if let Some(ref mut items) = self.items_opt {
                    if !items.get(click_i).map_or(false, |x| x.selected) {
                        // If item not selected, clear selection on other items
                        for (i, item) in items.iter_mut().enumerate() {
                            if i == click_i {
                                item.selected = true;
                            } else if mod_ctrl {
                                // Holding control allows multiple selection
                            } else {
                                item.selected = false;
                            }
                        }
                    }
                }
            }
            Message::Scroll(viewport) => {
                self.scroll_opt = Some(viewport.absolute_offset());
            }
            Message::SelectAll => {
                self.select_all();
                if self.select_focus.take().is_some() {
                    // Unfocus currently focused button
                    commands.push(Command::FocusButton(widget::Id::unique()));
                }
            }
            Message::Thumbnail(path, thumbnail) => {
                if let Some(ref mut items) = self.items_opt {
                    for item in items.iter_mut() {
                        if item.path_opt.as_ref() == Some(&path) {
                            if let ItemThumbnail::Rgba(rgba) = &thumbnail {
                                //TODO: pass handles already generated to avoid blocking main thread
                                let handle = widget::icon::from_raster_pixels(
                                    rgba.width(),
                                    rgba.height(),
                                    rgba.as_raw().clone(),
                                );
                                item.icon_handle_grid = handle.clone();
                                item.icon_handle_list = handle;
                            }
                            item.thumbnail_opt = Some(thumbnail);
                            break;
                        }
                    }
                }
            }
            Message::ToggleFoldersFirst => self.config.folders_first = !self.config.folders_first,
            Message::ToggleShowHidden => self.config.show_hidden = !self.config.show_hidden,

            Message::View(view) => {
                self.config.view = view;
            }
            Message::ToggleSort(heading_option) => {
                let heading_sort = if self.config.sort_name == heading_option {
                    !self.config.sort_direction
                } else {
                    true
                };
                self.config.sort_direction = heading_sort;
                self.config.sort_name = heading_option;
            }
            Message::Drop(Some((to, mut from))) => {
                self.dnd_hovered = None;
                match to {
                    Location::Path(to) => {
                        if let Ok(entries) = fs::read_dir(&to) {
                            for i in entries.into_iter().filter_map(|e| e.ok()) {
                                let i = i.path();
                                from.paths.retain(|p| &i != p);
                                if from.paths.is_empty() {
                                    log::info!("All dropped files already in target directory.");
                                    return commands;
                                }
                            }
                        }
                        commands.push(Command::DropFiles(to, from))
                    }
                    Location::Trash if matches!(from.kind, ClipboardKind::Cut) => {
                        commands.push(Command::MoveToTrash(from.paths))
                    }
                    Location::Trash => {
                        log::warn!("Copy to trash is not supported.");
                    }
                };
            }
            Message::Drop(None) => {
                self.dnd_hovered = None;
            }
            Message::DndHover(loc) => {
                if self.dnd_hovered.as_ref().is_some_and(|(l, i)| {
                    *l == loc && Instant::now().duration_since(*i) > HOVER_DURATION
                }) {
                    cd = Some(loc);
                }
            }
            Message::DndEnter(loc) => {
                self.dnd_hovered = Some((loc.clone(), Instant::now()));
                if loc != self.location {
                    commands.push(Command::Timeout(HOVER_DURATION, Message::DndHover(loc)));
                }
            }
            Message::DndLeave(loc) => {
                if Some(&loc) == self.dnd_hovered.as_ref().map(|(l, _)| l) {
                    self.dnd_hovered = None;
                }
            }
            Message::ZoomDefault => match self.config.view {
                View::List => self.config.icon_sizes.list = 100.try_into().unwrap(),
                View::Grid => self.config.icon_sizes.grid = 100.try_into().unwrap(),
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
                match self.config.view {
                    View::List => zoom_in(&mut self.config.icon_sizes.list, 100, 500),
                    View::Grid => zoom_in(&mut self.config.icon_sizes.grid, 50, 500),
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
                match self.config.view {
                    View::List => zoom_out(&mut self.config.icon_sizes.list, 100, 500),
                    View::Grid => zoom_out(&mut self.config.icon_sizes.grid, 50, 500),
                }
            }
        }
        if let Some(location) = cd {
            if location != self.location {
                if match &location {
                    Location::Path(path) => path.is_dir(),
                    Location::Trash => true,
                } {
                    self.change_location(&location, history_i_opt);
                    commands.push(Command::ChangeLocation(self.title(), location));
                } else {
                    log::warn!("tried to cd to {:?} which is not a directory", location);
                }
            }
        }
        commands
    }

    fn column_sort(&self) -> Option<Vec<(usize, &Item)>> {
        let check_reverse = |ord: Ordering, sort: bool| {
            if sort {
                ord
            } else {
                ord.reverse()
            }
        };
        let mut items: Vec<_> = self.items_opt.as_ref()?.iter().enumerate().collect();
        let heading_sort = self.config.sort_direction;
        match self.config.sort_name {
            HeadingOptions::Size => {
                items.sort_by(|a, b| {
                    // entries take precedence over size
                    let get_size = |x: &Item| match &x.metadata {
                        ItemMetadata::Path { metadata, children } => {
                            if metadata.is_dir() {
                                (true, *children as u64)
                            } else {
                                (false, metadata.len())
                            }
                        }
                        ItemMetadata::Trash { metadata, .. } => match metadata.size {
                            trash::TrashItemSize::Entries(entries) => (true, entries as u64),
                            trash::TrashItemSize::Bytes(bytes) => (false, bytes),
                        },
                    };
                    let (a_is_entry, a_size) = get_size(a.1);
                    let (b_is_entry, b_size) = get_size(b.1);

                    //TODO: use folders_first?
                    match (a_is_entry, b_is_entry) {
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                        _ => check_reverse(a_size.cmp(&b_size), heading_sort),
                    }
                })
            }
            HeadingOptions::Name => items.sort_by(|a, b| {
                if self.config.folders_first {
                    match (a.1.metadata.is_dir(), b.1.metadata.is_dir()) {
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                        _ => check_reverse(
                            lexical_sort::natural_lexical_cmp(&a.1.name, &b.1.name),
                            heading_sort,
                        ),
                    }
                } else {
                    check_reverse(
                        lexical_sort::natural_lexical_cmp(&a.1.name, &b.1.name),
                        heading_sort,
                    )
                }
            }),
            HeadingOptions::Modified => {
                items.sort_by(|a, b| {
                    let get_modified = |x: &Item| match &x.metadata {
                        ItemMetadata::Path { metadata, .. } => metadata.modified().ok(),
                        ItemMetadata::Trash { .. } => None,
                    };

                    let a_modified = get_modified(a.1);
                    let b_modified = get_modified(b.1);
                    if self.config.folders_first {
                        match (a.1.metadata.is_dir(), b.1.metadata.is_dir()) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => check_reverse(a_modified.cmp(&b_modified), heading_sort),
                        }
                    } else {
                        check_reverse(a_modified.cmp(&b_modified), heading_sort)
                    }
                });
            }
        }
        Some(items)
    }

    pub fn location_view(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_xxs,
            space_s,
            ..
        } = theme::active().cosmic().spacing;

        let mut row = widget::row::with_capacity(5).align_items(Alignment::Center);

        let mut prev_button =
            widget::button(widget::icon::from_name("go-previous-symbolic").size(16))
                .padding(space_xxs)
                .style(theme::Button::Icon);
        if self.history_i > 0 && !self.history.is_empty() {
            prev_button = prev_button.on_press(Message::GoPrevious);
        }
        row = row.push(prev_button);

        let mut next_button = widget::button(widget::icon::from_name("go-next-symbolic").size(16))
            .padding(space_xxs)
            .style(theme::Button::Icon);
        if self.history_i + 1 < self.history.len() {
            next_button = next_button.on_press(Message::GoNext);
        }
        row = row.push(next_button);

        row = row.push(widget::horizontal_space(Length::Fixed(space_s.into())));

        if let Some(location) = &self.edit_location {
            match location {
                Location::Path(path) => {
                    row = row.push(
                        widget::button(widget::icon::from_name("window-close-symbolic").size(16))
                            .on_press(Message::EditLocation(None))
                            .padding(space_xxs)
                            .style(theme::Button::Icon),
                    );
                    row = row.push(
                        widget::text_input("", path.to_string_lossy())
                            .id(self.edit_location_id.clone())
                            .on_input(|input| {
                                Message::EditLocation(Some(Location::Path(PathBuf::from(input))))
                            })
                            .on_submit(Message::Location(location.clone())),
                    );
                    return row.into();
                }
                _ => {
                    //TODO: allow editing other locations
                }
            }
        } else if let Location::Path(_) = &self.location {
            row = row.push(
                widget::button(widget::icon::from_name("edit-symbolic").size(16))
                    .on_press(Message::EditLocation(Some(self.location.clone())))
                    .padding(space_xxs)
                    .style(theme::Button::Icon),
            );
        }

        let mut children: Vec<Element<_>> = Vec::new();
        match &self.location {
            Location::Path(path) => {
                let home_dir = crate::home_dir();
                for ancestor in path.ancestors() {
                    let ancestor = ancestor.to_path_buf();
                    let mut found_home = false;
                    let mut row = widget::row::with_capacity(2)
                        .align_items(Alignment::Center)
                        .spacing(space_xxxs);

                    let name = match ancestor.file_name() {
                        Some(name) => {
                            if ancestor == home_dir {
                                row = row.push(
                                    widget::icon::icon(folder_icon_symbolic(&ancestor, 16))
                                        .size(16),
                                );
                                found_home = true;
                                fl!("home")
                            } else {
                                name.to_string_lossy().to_string()
                            }
                        }
                        None => {
                            row = row.push(
                                widget::icon::from_name("drive-harddisk-system-symbolic")
                                    .size(16)
                                    .icon(),
                            );
                            fl!("filesystem")
                        }
                    };

                    if children.is_empty() {
                        row = row.push(widget::text::heading(name));
                    } else {
                        children.push(
                            widget::icon::from_name("go-next-symbolic")
                                .size(16)
                                .icon()
                                .into(),
                        );
                        row = row.push(widget::text(name));
                    }

                    children.push(
                        widget::button(row)
                            .padding(space_xxxs)
                            .on_press(Message::Location(Location::Path(ancestor)))
                            .style(theme::Button::Link)
                            .into(),
                    );

                    if found_home {
                        break;
                    }
                }
                children.reverse();
            }
            Location::Trash => {
                let mut row = widget::row::with_capacity(2)
                    .align_items(Alignment::Center)
                    .spacing(space_xxxs);
                row = row.push(widget::icon::icon(trash_icon_symbolic(16)).size(16));
                row = row.push(widget::text::heading(fl!("trash")));

                children.push(
                    widget::button(row)
                        .padding(space_xxxs)
                        .on_press(Message::Location(Location::Trash))
                        .style(theme::Button::Text)
                        .into(),
                );
            }
        }

        for child in children {
            row = row.push(child);
        }
        row.into()
    }

    pub fn empty_view(&self, has_hidden: bool) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        widget::column::with_children(vec![widget::container(
            widget::column::with_children(vec![
                widget::icon::from_name("folder-symbolic")
                    .size(64)
                    .icon()
                    .into(),
                widget::text(if has_hidden {
                    fl!("empty-folder-hidden")
                } else {
                    fl!("empty-folder")
                })
                .into(),
            ])
            .align_items(Alignment::Center)
            .spacing(space_xxs),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()])
        .into()
    }

    pub fn grid_view(
        &self,
    ) -> (
        Option<Element<'static, cosmic::app::Message<crate::app::Message>>>,
        Element<Message>,
        bool,
    ) {
        let cosmic_theme::Spacing {
            space_m,
            space_xxs,
            space_xxxs,
            ..
        } = theme::active().cosmic().spacing;

        let TabConfig {
            show_hidden,
            icon_sizes,
            ..
        } = self.config;

        let text_height = 40; // Height of two lines of text
        let item_width = (2 * space_xxs + icon_sizes.grid() + 2 * space_xxs) as usize;
        let item_height =
            (space_xxxs + icon_sizes.grid() + space_xxxs + text_height + space_xxxs) as usize;

        let (width, height) = match self.size_opt {
            Some(size) => (
                (size.width.floor() as usize)
                    .checked_sub(2 * (space_m as usize))
                    .unwrap_or(0)
                    .max(item_width),
                (size.height.floor() as usize).max(item_height),
            ),
            None => (item_width, item_height),
        };

        let (cols, column_spacing) = {
            let width_m1 = width.checked_sub(item_width).unwrap_or(0);
            let cols_m1 = width_m1 / (item_width + space_xxs as usize);
            let cols = cols_m1 + 1;
            let spacing = width_m1
                .checked_div(cols_m1)
                .unwrap_or(0)
                .checked_sub(item_width)
                .unwrap_or(0);
            (cols, spacing as u16)
        };

        let mut grid = widget::grid()
            .column_spacing(column_spacing)
            .row_spacing(space_xxs)
            .padding([0, space_m].into());
        let mut dnd_items: Vec<(usize, (usize, usize), &Item)> = Vec::new();
        let mut drag_w_i = usize::MAX;
        let mut drag_n_i = usize::MAX;
        let mut drag_e_i = 0;
        let mut drag_s_i = 0;

        let mut children = Vec::new();

        if let Some(items) = self.column_sort() {
            let mut count = 0;
            let mut col = 0;
            let mut row = 0;
            let mut hidden = 0;
            for &(i, item) in items.iter() {
                if !show_hidden && item.hidden {
                    item.pos_opt.set(None);
                    item.rect_opt.set(None);
                    hidden += 1;
                    continue;
                }
                item.pos_opt.set(Some((row, col)));
                item.rect_opt.set(Some(Rectangle::new(
                    Point::new(
                        (col * (item_width + column_spacing as usize) + space_m as usize) as f32,
                        (row * (item_height + space_xxs as usize)) as f32,
                    ),
                    Size::new(item_width as f32, item_height as f32),
                )));

                //TODO: one focus group per grid item (needs custom widget)
                let buttons = vec![
                    widget::button(
                        widget::icon::icon(item.icon_handle_grid.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_sizes.grid()),
                    )
                    .style(button_style(item.selected, false))
                    .padding(space_xxxs),
                    widget::button(widget::text(item.name.clone()))
                        .id(item.button_id.clone())
                        .style(button_style(item.selected, true))
                        .padding([0, space_xxs]),
                ];

                let mut column = widget::column::with_capacity(buttons.len())
                    .align_items(Alignment::Center)
                    .height(Length::Fixed(item_height as f32))
                    .width(Length::Fixed(item_width as f32));
                for button in buttons {
                    if self.context_menu.is_some() {
                        column = column.push(button)
                    } else {
                        column = column.push(
                            mouse_area::MouseArea::new(button).on_right_press_no_capture(
                                move |_point_opt| Message::RightClick(i),
                            ),
                        );
                    }
                }

                let column: Element<Message> = if item.metadata.is_dir() && item.path_opt.is_some()
                {
                    let tab_location = Location::Path(item.path_opt.clone().unwrap());
                    let tab_location_enter = tab_location.clone();
                    let tab_location_leave = tab_location.clone();
                    let is_dnd_hovered =
                        self.dnd_hovered.as_ref().map(|(l, _)| l) == Some(&tab_location);
                    cosmic::widget::container(
                        DndDestination::for_data::<ClipboardPaste>(column, move |data, action| {
                            if let Some(mut data) = data {
                                if action == DndAction::Copy {
                                    Message::Drop(Some((tab_location.clone(), data)))
                                } else if action == DndAction::Move {
                                    data.kind = ClipboardKind::Cut;
                                    Message::Drop(Some((tab_location.clone(), data)))
                                } else {
                                    log::warn!("unsupported action: {:?}", action);
                                    Message::Drop(None)
                                }
                            } else {
                                Message::Drop(None)
                            }
                        })
                        .on_enter(move |_, _, _| Message::DndEnter(tab_location_enter.clone()))
                        .on_leave(move || Message::DndLeave(tab_location_leave.clone())),
                    )
                    .style(if is_dnd_hovered {
                        theme::Container::custom(|t| {
                            let mut a = cosmic::iced_style::container::StyleSheet::appearance(
                                t,
                                &theme::Container::default(),
                            );
                            let t = t.cosmic();
                            // todo use theme drop target color
                            let mut bg = t.accent_color();
                            bg.alpha = 0.2;
                            a.background = Some(Color::from(bg).into());
                            a.border = Border {
                                color: t.accent_color().into(),
                                width: 1.0,
                                radius: t.radius_s().into(),
                            };
                            a
                        })
                    } else {
                        theme::Container::default()
                    })
                    .into()
                } else {
                    column.into()
                };

                if item.selected {
                    dnd_items.push((i, (row, col), item));
                    drag_w_i = drag_w_i.min(col);
                    drag_n_i = drag_n_i.min(row);
                    drag_e_i = drag_e_i.max(col);
                    drag_s_i = drag_s_i.max(row);
                }
                let mouse_area = crate::mouse_area::MouseArea::new(column)
                    .on_press(move |_| Message::Click(Some(i)))
                    .on_double_click(move |_| Message::DoubleClick(Some(i)))
                    .on_release(move |_| Message::ClickRelease(Some(i)));
                grid = grid.push(mouse_area);

                count += 1;
                col += 1;
                if col >= cols {
                    col = 0;
                    row += 1;
                    grid = grid.insert_row();
                }
            }

            if count == 0 {
                return (None, self.empty_view(hidden > 0), false);
            }

            children.push(grid.into());

            //TODO: HACK If we don't reach the bottom of the view, go ahead and add a spacer to do that
            {
                let mut max_bottom = 0;
                for (_, item) in items {
                    if let Some(rect) = item.rect_opt.get() {
                        let bottom = (rect.y + rect.height).ceil() as usize;
                        if bottom > max_bottom {
                            max_bottom = bottom;
                        }
                    }
                }
                let spacer_height = height
                    .checked_sub(max_bottom + 2 * (space_xxs as usize))
                    .unwrap_or(0);
                if spacer_height > 0 {
                    children.push(
                        widget::container(vertical_space(Length::Fixed(spacer_height as f32)))
                            .into(),
                    )
                }
            }
        }

        (
            (!dnd_items.is_empty()).then(|| {
                let mut dnd_grid = widget::grid()
                    .column_spacing(column_spacing)
                    .row_spacing(space_xxs)
                    .padding([0, space_m].into());

                let mut dnd_item_i = 0;
                for r in drag_n_i..=drag_s_i {
                    dnd_grid = dnd_grid.insert_row();
                    for c in drag_w_i..=drag_e_i {
                        let Some((i, (row, col), item)) = dnd_items.get(dnd_item_i) else {
                            break;
                        };
                        if *row == r && *col == c {
                            let buttons = vec![
                                widget::button(
                                    widget::icon::icon(item.icon_handle_grid.clone())
                                        .content_fit(ContentFit::Contain)
                                        .size(icon_sizes.grid()),
                                )
                                .on_press(Message::Click(Some(*i)))
                                .padding(space_xxxs)
                                .style(button_style(item.selected, false)),
                                widget::button(widget::text(item.name.clone()))
                                    .id(item.button_id.clone())
                                    .on_press(Message::Click(Some(*i)))
                                    .padding([0, space_xxs])
                                    .style(button_style(item.selected, true)),
                            ];

                            let mut column = widget::column::with_capacity(buttons.len())
                                .align_items(Alignment::Center)
                                .height(Length::Fixed(item_height as f32))
                                .width(Length::Fixed(item_width as f32));
                            for button in buttons {
                                column = column.push(button)
                            }

                            dnd_grid = dnd_grid.push(column);
                            dnd_item_i += 1;
                        } else {
                            dnd_grid = dnd_grid.push(
                                widget::container(vertical_space(item_width as f32))
                                    .height(Length::Fixed(item_height as f32)),
                            );
                        }
                    }
                }
                Element::from(dnd_grid)
                    .map(|m| cosmic::app::Message::App(crate::app::Message::TabMessage(None, m)))
            }),
            mouse_area::MouseArea::new(widget::column::with_children(children))
                .on_press(|_| Message::Click(None))
                .on_drag(Message::Drag)
                .on_drag_end(|_| Message::DragEnd(None))
                .show_drag_rect(true)
                .on_release(|_| Message::ClickRelease(None))
                .into(),
            true,
        )
    }

    pub fn list_view(
        &self,
    ) -> (
        Option<Element<'static, cosmic::app::Message<crate::app::Message>>>,
        Element<Message>,
        bool,
    ) {
        let cosmic_theme::Spacing {
            space_m, space_xxs, ..
        } = theme::active().cosmic().spacing;

        let TabConfig {
            show_hidden,
            icon_sizes,
            sort_name,
            sort_direction,
            ..
        } = self.config;

        let size = self.size_opt.unwrap_or_else(|| Size::new(0.0, 0.0));
        //TODO: allow resizing?
        let name_width = 300.0;
        let modified_width = 200.0;
        let size_width = 100.0;
        let condensed = size.width < (name_width + modified_width + size_width);
        let icon_size = if condensed {
            icon_sizes.list_condensed()
        } else {
            icon_sizes.list()
        };
        let row_height = icon_size + 2 * space_xxs;

        let heading_item = |name, width, msg| {
            let mut row = widget::row::with_capacity(2)
                .align_items(Alignment::Center)
                .spacing(space_xxs)
                .width(width);
            row = row.push(widget::text::heading(name));
            match (sort_name == msg, sort_direction) {
                (true, true) => {
                    row = row.push(widget::icon::from_name("pan-down-symbolic").size(16));
                }
                (true, false) => {
                    row = row.push(widget::icon::from_name("pan-up-symbolic").size(16));
                }
                _ => {}
            }
            //TODO: make it possible to resize with the mouse
            mouse_area::MouseArea::new(row)
                .on_press(move |_point_opt| Message::ToggleSort(msg))
                .into()
        };

        let mut children: Vec<Element<_>> = Vec::new();
        let mut y = 0;
        if !condensed {
            children.push(
                widget::row::with_children(vec![
                    heading_item(fl!("name"), Length::Fill, HeadingOptions::Name),
                    //TODO: do not show modified column when in the trash
                    heading_item(
                        fl!("modified"),
                        Length::Fixed(modified_width),
                        HeadingOptions::Modified,
                    ),
                    heading_item(fl!("size"), Length::Fixed(size_width), HeadingOptions::Size),
                ])
                .align_items(Alignment::Center)
                .height(Length::Fixed(row_height as f32))
                .padding(space_xxs)
                .spacing(space_xxs)
                .into(),
            );
            y += row_height;
            children.push(horizontal_rule(1).into());
            y += 1;
        }

        let items = self.column_sort();
        let mut drag_items = Vec::new();
        if let Some(items) = items {
            let mut count = 0;
            let mut hidden = 0;
            for (i, item) in items {
                if !show_hidden && item.hidden {
                    item.pos_opt.set(None);
                    item.rect_opt.set(None);
                    hidden += 1;
                    continue;
                }
                item.pos_opt.set(Some((count, 0)));
                item.rect_opt.set(Some(Rectangle::new(
                    Point::new(space_m as f32, y as f32),
                    Size::new(size.width - (2 * space_m) as f32, row_height as f32),
                )));

                if count > 0 {
                    children.push(horizontal_rule(1).into());
                    y += 1;
                }

                let modified_text = match &item.metadata {
                    ItemMetadata::Path { metadata, .. } => match metadata.modified() {
                        Ok(time) => chrono::DateTime::<chrono::Local>::from(time)
                            .format(TIME_FORMAT)
                            .to_string(),
                        Err(_) => String::new(),
                    },
                    ItemMetadata::Trash { .. } => String::new(),
                };

                let size_text = match &item.metadata {
                    ItemMetadata::Path { metadata, children } => {
                        if metadata.is_dir() {
                            format!("{} items", children)
                        } else {
                            format_size(metadata.len())
                        }
                    }
                    ItemMetadata::Trash { metadata, .. } => match metadata.size {
                        trash::TrashItemSize::Entries(entries) => {
                            //TODO: translate
                            if entries == 1 {
                                format!("{} item", entries)
                            } else {
                                format!("{} items", entries)
                            }
                        }
                        trash::TrashItemSize::Bytes(bytes) => format_size(bytes),
                    },
                };

                let row = if condensed {
                    widget::row::with_children(vec![
                        widget::icon::icon(item.icon_handle_list_condensed.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_size)
                            .into(),
                        widget::column::with_children(vec![
                            widget::text(item.name.clone()).into(),
                            //TODO: translate?
                            widget::text(format!("{} - {}", modified_text, size_text)).into(),
                        ])
                        .into(),
                    ])
                    .align_items(Alignment::Center)
                    .spacing(space_xxs)
                } else {
                    widget::row::with_children(vec![
                        widget::icon::icon(item.icon_handle_list.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_size)
                            .into(),
                        widget::text(item.name.clone()).width(Length::Fill).into(),
                        widget::text(modified_text.clone())
                            .width(Length::Fixed(modified_width))
                            .into(),
                        widget::text(size_text.clone())
                            .width(Length::Fixed(size_width))
                            .into(),
                    ])
                    .align_items(Alignment::Center)
                    .spacing(space_xxs)
                };

                let button = |row| {
                    crate::mouse_area::MouseArea::new(
                        widget::button(row)
                            .width(Length::Fill)
                            .height(Length::Fixed(row_height as f32))
                            .id(item.button_id.clone())
                            .padding(space_xxs)
                            .style(button_style(item.selected, true)),
                    )
                    .on_press(move |_| Message::Click(Some(i)))
                    .on_double_click(move |_| Message::DoubleClick(Some(i)))
                    .on_release(move |_| Message::ClickRelease(Some(i)))
                };

                let mut button_row = button(row.into());
                if self.context_menu.is_some() {
                    button_row =
                        button_row.on_right_press(move |_point_opt| Message::RightClick(i));
                }
                let button_row: Element<_> = if item.metadata.is_dir() && item.path_opt.is_some() {
                    let tab_location = Location::Path(item.path_opt.clone().unwrap());
                    let tab_location_enter = tab_location.clone();
                    let tab_location_leave = tab_location.clone();
                    let is_dnd_hovered =
                        self.dnd_hovered.as_ref().map(|(l, _)| l) == Some(&tab_location);
                    cosmic::widget::container(
                        DndDestination::for_data(button_row, move |data, action| {
                            if let Some(mut data) = data {
                                if action == DndAction::Copy {
                                    Message::Drop(Some((tab_location.clone(), data)))
                                } else if action == DndAction::Move {
                                    data.kind = ClipboardKind::Cut;
                                    Message::Drop(Some((tab_location.clone(), data)))
                                } else {
                                    log::warn!("unsupported action: {:?}", action);
                                    Message::Drop(None)
                                }
                            } else {
                                log::warn!("No data for drop.");
                                Message::Drop(None)
                            }
                        })
                        .on_enter(move |_, _, _| Message::DndEnter(tab_location_enter.clone()))
                        .on_leave(move || Message::DndLeave(tab_location_leave.clone())),
                    )
                    // todo refactor into the dnd destination wrapper
                    .style(if is_dnd_hovered {
                        theme::Container::custom(|t| {
                            let mut a = cosmic::iced_style::container::StyleSheet::appearance(
                                t,
                                &theme::Container::default(),
                            );
                            let t = t.cosmic();
                            // todo use theme drop target color
                            let mut bg = t.accent_color();
                            bg.alpha = 0.2;
                            a.background = Some(Color::from(bg).into());
                            a.border = Border {
                                color: t.accent_color().into(),
                                width: 1.0,
                                radius: t.radius_s().into(),
                            };
                            a
                        })
                    } else {
                        theme::Container::default()
                    })
                    .into()
                } else {
                    button_row.into()
                };

                if item.selected || !drag_items.is_empty() {
                    let dnd_row = if !item.selected {
                        Element::from(vertical_space(Length::Fixed(row_height as f32)))
                    } else if condensed {
                        widget::row::with_children(vec![
                            widget::icon::icon(item.icon_handle_list_condensed.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::column::with_children(vec![
                                widget::text(item.name.clone()).into(),
                                //TODO: translate?
                                widget::text(format!("{} - {}", modified_text, size_text)).into(),
                            ])
                            .into(),
                        ])
                        .align_items(Alignment::Center)
                        .spacing(space_xxs)
                        .into()
                    } else {
                        widget::row::with_children(vec![
                            widget::icon::icon(item.icon_handle_list.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::text(item.name.clone()).width(Length::Fill).into(),
                            widget::text(modified_text)
                                .width(Length::Fixed(modified_width))
                                .into(),
                            widget::text(size_text)
                                .width(Length::Fixed(size_width))
                                .into(),
                        ])
                        .align_items(Alignment::Center)
                        .spacing(space_xxs)
                        .into()
                    };
                    if item.selected {
                        drag_items.push(
                            widget::container(button(dnd_row))
                                .width(Length::Shrink)
                                .into(),
                        );
                    } else {
                        drag_items.push(dnd_row);
                    }
                }

                count += 1;
                y += row_height;
                children.push(button_row);
            }

            if count == 0 {
                return (None, self.empty_view(hidden > 0), false);
            }
        }
        //TODO: HACK If we don't reach the bottom of the view, go ahead and add a spacer to do that
        {
            let spacer_height = size.height as i32 - y as i32;
            if spacer_height > 0 {
                children.push(
                    widget::container(vertical_space(Length::Fixed(spacer_height as f32))).into(),
                );
            }
        }
        let drag_col = (!drag_items.is_empty()).then(|| {
            Element::from(widget::column::with_children(drag_items))
                .map(|m| cosmic::app::Message::App(crate::app::Message::TabMessage(None, m)))
        });

        (
            drag_col,
            mouse_area::MouseArea::new(
                widget::column::with_children(children).padding([0, space_m]),
            )
            .with_id(Id::new("list-view"))
            .on_press(|_| Message::Click(None))
            .on_drag(Message::Drag)
            .on_drag_end(|_| Message::DragEnd(None))
            .show_drag_rect(true)
            .on_release(|_| Message::ClickRelease(None))
            .into(),
            true,
        )
    }

    pub fn view(&self, key_binds: &HashMap<KeyBind, Action>) -> Element<Message> {
        let location_view = self.location_view();
        let (drag_list, mut item_view, can_scroll) = match self.config.view {
            View::Grid => self.grid_view(),
            View::List => self.list_view(),
        };
        item_view = widget::container(item_view).width(Length::Fill).into();
        let files = self
            .items_opt
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .filter(|item| item.selected)
                    .filter_map(|item| item.path_opt.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let item_view = DndSource::<_, cosmic::app::Message<app::Message>, ClipboardCopy>::with_id(
            item_view,
            Id::new("tab-view"),
        );

        let item_view = match drag_list {
            Some(drag_list) if self.selected_clicked => {
                let drag_list = ArcElementWrapper(Arc::new(Mutex::new(drag_list)));
                item_view
                    .drag_content(move || {
                        ClipboardCopy::new(crate::clipboard::ClipboardKind::Copy, &files)
                    })
                    .drag_icon(move || {
                        let state: tree::State =
                            Widget::<cosmic::app::Message<app::Message>, _, _>::state(&drag_list);
                        (drag_list.clone().into(), state)
                    })
            }
            _ => item_view,
        };

        let tab_location = self.location.clone();
        let mut mouse_area = mouse_area::MouseArea::new(item_view)
            .on_press(move |_point_opt| Message::Click(None))
            .on_release(|_| Message::ClickRelease(None))
            .on_back_press(move |_point_opt| Message::GoPrevious)
            .on_forward_press(move |_point_opt| Message::GoNext);

        if self.context_menu.is_some() {
            mouse_area = mouse_area.on_right_press(move |_point_opt| Message::ContextMenu(None));
        } else {
            mouse_area =
                mouse_area.on_right_press(move |point_opt| Message::ContextMenu(point_opt));
        }

        let mut popover = widget::popover(mouse_area);

        if let Some(point) = self.context_menu {
            popover = popover
                .popup(menu::context_menu(&self, &key_binds))
                .position(widget::popover::Position::Point(point));
        }
        let mut tab_column = widget::column::with_capacity(3);
        tab_column = tab_column.push(location_view);
        if can_scroll {
            tab_column = tab_column.push(
                mouse_area::MouseArea::new(
                    widget::scrollable(popover)
                        .id(self.scrollable_id.clone())
                        .on_scroll(Message::Scroll)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .on_resize(Message::Resize),
            );
        } else {
            tab_column = tab_column.push(popover);
        }
        if let Location::Trash = self.location {
            if let Some(items) = self.items_opt() {
                if !items.is_empty() {
                    let cosmic_theme::Spacing {
                        space_xxs,
                        space_xs,
                        ..
                    } = theme::active().cosmic().spacing;

                    tab_column = tab_column.push(
                        widget::layer_container(widget::row::with_children(vec![
                            widget::horizontal_space(Length::Fill).into(),
                            widget::button::standard(fl!("empty-trash"))
                                .on_press(Message::EmptyTrash)
                                .into(),
                        ]))
                        .padding([space_xxs, space_xs])
                        .layer(cosmic_theme::Layer::Primary),
                    );
                }
            }
        }
        let mut tab_view = widget::container(tab_column)
            .height(Length::Fill)
            .width(Length::Fill);

        if self.dnd_hovered.as_ref().map(|(l, _)| l) == Some(&tab_location) {
            tab_view = tab_view.style(cosmic::theme::Container::custom(|t| {
                let mut a = cosmic::iced_style::container::StyleSheet::appearance(
                    t,
                    &cosmic::theme::Container::default(),
                );
                let c = t.cosmic();
                a.border = cosmic::iced_core::Border {
                    color: (c.accent_color()).into(),
                    width: 1.,
                    radius: c.radius_0().into(),
                };
                a
            }));
        }

        let tab_location_2 = self.location.clone();
        let tab_location_3 = self.location.clone();
        let dnd_dest = DndDestination::for_data(tab_view, move |data, action| {
            if let Some(mut data) = data {
                if action == DndAction::Copy {
                    Message::Drop(Some((tab_location.clone(), data)))
                } else if action == DndAction::Move {
                    data.kind = ClipboardKind::Cut;
                    Message::Drop(Some((tab_location.clone(), data)))
                } else {
                    log::warn!("unsupported action: {:?}", action);
                    Message::Drop(None)
                }
            } else {
                Message::Drop(None)
            }
        })
        .on_enter(move |_, _, _| Message::DndEnter(tab_location_2.clone()))
        .on_leave(move || Message::DndLeave(tab_location_3.clone()));

        dnd_dest.into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if let Some(items) = &self.items_opt {
            //TODO: how many thumbnail loads should be in flight at once?
            let jobs = 8;
            let mut subscriptions = Vec::with_capacity(jobs);

            //TODO: move to function
            let visible_rect = {
                let point = match self.scroll_opt {
                    Some(offset) => Point::new(0.0, offset.y),
                    None => Point::new(0.0, 0.0),
                };
                let size = self.size_opt.unwrap_or_else(|| Size::new(0.0, 0.0));
                Rectangle::new(point, size)
            };

            //TODO: HACK to ensure positions are up to date since subscription runs before view
            match self.config.view {
                View::Grid => _ = self.grid_view(),
                View::List => _ = self.list_view(),
            };

            for item in items.iter() {
                if item.thumbnail_opt.is_some() {
                    // Skip items that already have a thumbnail
                    continue;
                }

                match item.rect_opt.get() {
                    Some(rect) => {
                        if !rect.intersects(&visible_rect) {
                            // Skip items that are not visible
                            continue;
                        }
                    }
                    None => {
                        // Skip items with no determined rect (this should include hidden items)
                        continue;
                    }
                }

                if let Some(path) = item.path_opt.clone() {
                    subscriptions.push(subscription::channel(
                        path.clone(),
                        1,
                        |mut output| async move {
                            let (path, thumbnail) = tokio::task::spawn_blocking(move || {
                                let start = std::time::Instant::now();
                                let thumbnail = match image::io::Reader::open(&path)
                                    .and_then(|img| img.with_guessed_format())
                                {
                                    Ok(reader) => match reader.decode() {
                                        Ok(image) => {
                                            //TODO: configurable thumbnail size?
                                            let thumbnail_size =
                                                (ICON_SIZE_GRID * ICON_SCALE_MAX) as u32;
                                            let thumbnail =
                                                image.thumbnail(thumbnail_size, thumbnail_size);
                                            ItemThumbnail::Rgba(thumbnail.to_rgba8())
                                        }
                                        Err(err) => {
                                            log::warn!("failed to decode {:?}: {}", path, err);
                                            ItemThumbnail::NotImage
                                        }
                                    },
                                    Err(err) => {
                                        log::warn!("failed to read {:?}: {}", path, err);
                                        ItemThumbnail::NotImage
                                    }
                                };
                                log::info!("thumbnailed {:?} in {:?}", path, start.elapsed());
                                (path, thumbnail)
                            })
                            .await
                            .unwrap();

                            match output
                                .send(Message::Thumbnail(path.clone(), thumbnail))
                                .await
                            {
                                Ok(()) => {}
                                Err(err) => {
                                    log::warn!("failed to send thumbnail for {:?}: {}", path, err);
                                }
                            }

                            //TODO: how to properly kill this task?
                            loop {
                                tokio::time::sleep(std::time::Duration::new(1, 0)).await;
                            }
                        },
                    ));
                }

                if subscriptions.len() >= jobs {
                    break;
                }
            }
            Subscription::batch(subscriptions)
        } else {
            Subscription::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io, path::PathBuf};

    use cosmic::iced_runtime::keyboard::Modifiers;
    use log::{debug, trace};
    use tempfile::TempDir;
    use test_log::test;

    use super::{scan_path, Location, Message, Tab};
    use crate::{
        app::test_utils::{
            assert_eq_tab_path, empty_fs, eq_path_item, filter_dirs, read_dir_sorted, simple_fs,
            tab_click_new, NAME_LEN, NUM_DIRS, NUM_FILES, NUM_HIDDEN, NUM_NESTED,
        },
        config::{IconSizes, TabConfig},
    };

    // Boilerplate for tab tests. Checks if simulated clicks selected items.
    fn tab_selects_item(
        clicks: &[usize],
        modifiers: Modifiers,
        expected_selected: &[bool],
    ) -> io::Result<()> {
        let (_fs, mut tab) = tab_click_new(NUM_FILES, NUM_NESTED, NUM_DIRS, NUM_NESTED, NAME_LEN)?;

        // Simulate clicks by triggering Message::Click
        for &click in clicks {
            debug!("Emitting Message::Click(Some({click})) with modifiers: {modifiers:?}");
            tab.update(Message::Click(Some(click)), modifiers);
        }

        let items = tab
            .items_opt
            .as_deref()
            .expect("tab should be populated with items");

        for (i, (&expected, actual)) in expected_selected.into_iter().zip(items).enumerate() {
            assert_eq!(
                expected,
                actual.selected,
                "expected index {i} to be {}",
                if expected {
                    "selected but it was deselected"
                } else {
                    "deselected but it was selected"
                }
            );
        }

        Ok(())
    }

    fn tab_history() -> io::Result<(TempDir, Tab, Vec<PathBuf>)> {
        let fs = simple_fs(NUM_FILES, NUM_NESTED, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();
        let mut tab = Tab::new(Location::Path(path.into()), TabConfig::default());

        // All directories (simple_fs only produces one nested layer)
        let dirs: Vec<PathBuf> = filter_dirs(path)?
            .flat_map(|dir| {
                filter_dirs(&dir).map(|nested_dirs| std::iter::once(dir).chain(nested_dirs))
            })
            .flatten()
            .collect();
        assert!(
            dirs.len() == NUM_DIRS + NUM_DIRS * NUM_NESTED,
            "Sanity check: Have {} dirs instead of {}",
            dirs.len(),
            NUM_DIRS + NUM_DIRS * NUM_NESTED
        );

        debug!("Building history by emitting Message::Location");
        for dir in &dirs {
            debug!(
                "Emitting Message::Location(Location::Path(\"{}\"))",
                dir.display()
            );
            tab.update(
                Message::Location(Location::Path(dir.clone())),
                Modifiers::empty(),
            );
        }
        trace!("Tab history: {:?}", tab.history);

        Ok((fs, tab, dirs))
    }

    #[test]
    fn scan_path_succeeds_on_valid_path() -> io::Result<()> {
        let fs = simple_fs(NUM_FILES, NUM_HIDDEN, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        // Read directory entries and sort as cosmic-files does
        let entries = read_dir_sorted(path)?;

        debug!("Calling scan_path(\"{}\")", path.display());
        let actual = scan_path(&path.to_owned(), IconSizes::default());

        // scan_path shouldn't skip any entries
        assert_eq!(entries.len(), actual.len());

        // Correct files should be scanned
        assert!(entries
            .into_iter()
            .zip(actual.into_iter())
            .all(|(path, item)| eq_path_item(&path, &item)));

        Ok(())
    }

    #[test]
    fn scan_path_returns_empty_vec_for_invalid_path() -> io::Result<()> {
        let fs = simple_fs(NUM_FILES, NUM_NESTED, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        // A nonexisting path within the temp dir
        let invalid_path = path.join("ferris");
        assert!(!invalid_path.exists());

        debug!("Calling scan_path(\"{}\")", invalid_path.display());
        let actual = scan_path(&invalid_path, IconSizes::default());

        assert!(actual.is_empty());

        Ok(())
    }

    #[test]
    fn scan_path_empty_dir_returns_empty_vec() -> io::Result<()> {
        let fs = empty_fs()?;
        let path = fs.path();

        debug!("Calling scan_path(\"{}\")", path.display());
        let actual = scan_path(&path.to_owned(), IconSizes::default());

        assert_eq!(0, path.read_dir()?.count());
        assert!(actual.is_empty());

        Ok(())
    }

    #[test]
    fn tab_location_changes_location() -> io::Result<()> {
        let fs = simple_fs(NUM_FILES, NUM_NESTED, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        // Next directory in temp directory
        // This does not have to be sorted
        let next_dir = filter_dirs(path)?
            .next()
            .expect("temp directory should have at least one directory");

        let mut tab = Tab::new(Location::Path(path.to_owned()), TabConfig::default());
        debug!(
            "Emitting Message::Location(Location::Path(\"{}\"))",
            next_dir.display()
        );
        tab.update(
            Message::Location(Location::Path(next_dir.clone())),
            Modifiers::empty(),
        );

        // Validate that the tab's path updated
        // NOTE: `items_opt` is set to None with Message::Location so this ONLY checks for equal paths
        // If item contents are NOT None then this needs to be reevaluated for correctness
        assert_eq_tab_path(&tab, &next_dir);
        assert!(
            tab.items_opt.is_none(),
            "Tab's `items` is not None which means this test needs to be updated"
        );

        Ok(())
    }

    #[test]
    fn tab_click_single_selects_item() -> io::Result<()> {
        // Select the second directory with no keys held down
        tab_selects_item(&[1], Modifiers::empty(), &[false, true])
    }

    #[test]
    fn tab_click_double_opens_folder() -> io::Result<()> {
        let (fs, mut tab) = tab_click_new(NUM_FILES, NUM_NESTED, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        // Simulate double clicking second directory
        debug!("Emitting double click Message::DoubleClick(Some(1))");
        tab.update(Message::DoubleClick(Some(1)), Modifiers::empty());

        // Path to second directory
        let second_dir = read_dir_sorted(path)?
            .into_iter()
            .filter(|p| p.is_dir())
            .nth(1)
            .expect("should be at least two directories");

        // Location should have changed to second_dir
        assert_eq_tab_path(&tab, &second_dir);

        Ok(())
    }

    #[test]
    fn tab_click_ctrl_selects_multiple() -> io::Result<()> {
        // Select the first and second directory by holding down ctrl
        tab_selects_item(&[0, 1], Modifiers::CTRL, &[true, true])
    }

    #[test]
    fn tab_gonext_moves_forward_in_history() -> io::Result<()> {
        let (fs, mut tab, dirs) = tab_history()?;
        let path = fs.path();

        // Rewind to the start
        for _ in 0..dirs.len() {
            debug!("Emitting Message::GoPrevious to rewind to the start",);
            tab.update(Message::GoPrevious, Modifiers::empty());
        }
        assert_eq_tab_path(&tab, path);

        // Back to the future. Directories should be in the order they were opened.
        for dir in dirs {
            debug!("Emitting Message::GoNext",);
            tab.update(Message::GoNext, Modifiers::empty());
            assert_eq_tab_path(&tab, &dir);
        }

        Ok(())
    }

    #[test]
    fn tab_goprev_moves_backward_in_history() -> io::Result<()> {
        let (fs, mut tab, dirs) = tab_history()?;
        let path = fs.path();

        for dir in dirs.into_iter().rev() {
            assert_eq_tab_path(&tab, &dir);
            debug!("Emitting Message::GoPrevious",);
            tab.update(Message::GoPrevious, Modifiers::empty());
        }
        assert_eq_tab_path(&tab, path);

        Ok(())
    }

    #[test]
    fn tab_empty_history_does_nothing_on_prev_next() -> io::Result<()> {
        let fs = simple_fs(0, NUM_NESTED, NUM_DIRS, 0, NAME_LEN)?;
        let path = fs.path();
        let mut tab = Tab::new(Location::Path(path.into()), TabConfig::default());

        // Tab's location shouldn't change if GoPrev or GoNext is triggered
        debug!("Emitting Message::GoPrevious",);
        tab.update(Message::GoPrevious, Modifiers::empty());
        assert_eq_tab_path(&tab, path);

        debug!("Emitting Message::GoNext",);
        tab.update(Message::GoNext, Modifiers::empty());
        assert_eq_tab_path(&tab, path);

        Ok(())
    }

    #[test]
    fn tab_locationup_moves_up_hierarchy() -> io::Result<()> {
        let fs = simple_fs(0, NUM_NESTED, NUM_DIRS, 0, NAME_LEN)?;
        let path = fs.path();
        let mut next_dir = filter_dirs(path)?
            .next()
            .expect("should be at least one directory");

        let mut tab = Tab::new(Location::Path(next_dir.clone()), TabConfig::default());
        // This will eventually yield false once root is hit
        while next_dir.pop() {
            debug!("Emitting Message::LocationUp",);
            tab.update(Message::LocationUp, Modifiers::empty());
            assert_eq_tab_path(&tab, &next_dir);
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct ArcElementWrapper<M>(pub Arc<Mutex<Element<'static, M>>>);

impl<M> Widget<M, cosmic::Theme, cosmic::Renderer> for ArcElementWrapper<M> {
    fn size(&self) -> Size<Length> {
        self.0.lock().unwrap().as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.0.lock().unwrap().as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        renderer: &cosmic::Renderer,
        limits: &cosmic::iced_core::layout::Limits,
    ) -> cosmic::iced_core::layout::Node {
        self.0
            .lock()
            .unwrap()
            .as_widget_mut()
            .layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut cosmic::Renderer,
        theme: &cosmic::Theme,
        style: &cosmic::iced_core::renderer::Style,
        layout: cosmic::iced_core::Layout<'_>,
        cursor: cosmic::iced_core::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.0
            .lock()
            .unwrap()
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport)
    }

    fn tag(&self) -> tree::Tag {
        self.0.lock().unwrap().as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.0.lock().unwrap().as_widget().state()
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.0.lock().unwrap().as_widget().children()
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        self.0.lock().unwrap().as_widget_mut().diff(tree)
    }

    fn operate(
        &self,
        state: &mut tree::Tree,
        layout: cosmic::iced_core::Layout<'_>,
        renderer: &cosmic::Renderer,
        operation: &mut dyn widget::Operation<cosmic::iced_core::widget::OperationOutputWrapper<M>>,
    ) {
        self.0
            .lock()
            .unwrap()
            .as_widget()
            .operate(state, layout, renderer, operation)
    }

    fn on_event(
        &mut self,
        _state: &mut tree::Tree,
        _event: cosmic::iced::Event,
        _layout: cosmic::iced_core::Layout<'_>,
        _cursor: cosmic::iced_core::mouse::Cursor,
        _renderer: &cosmic::Renderer,
        _clipboard: &mut dyn cosmic::iced_core::Clipboard,
        _shell: &mut cosmic::iced_core::Shell<'_, M>,
        _viewport: &Rectangle,
    ) -> cosmic::iced_core::event::Status {
        self.0.lock().unwrap().as_widget_mut().on_event(
            _state, _event, _layout, _cursor, _renderer, _clipboard, _shell, _viewport,
        )
    }

    fn mouse_interaction(
        &self,
        _state: &tree::Tree,
        _layout: cosmic::iced_core::Layout<'_>,
        _cursor: cosmic::iced_core::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &cosmic::Renderer,
    ) -> cosmic::iced_core::mouse::Interaction {
        self.0
            .lock()
            .unwrap()
            .as_widget()
            .mouse_interaction(_state, _layout, _cursor, _viewport, _renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        _state: &'a mut tree::Tree,
        _layout: cosmic::iced_core::Layout<'_>,
        _renderer: &cosmic::Renderer,
    ) -> Option<cosmic::iced_core::overlay::Element<'a, M, cosmic::Theme, cosmic::Renderer>> {
        // TODO
        None
    }

    fn id(&self) -> Option<Id> {
        self.0.lock().unwrap().as_widget().id()
    }

    fn set_id(&mut self, _id: Id) {
        self.0.lock().unwrap().as_widget_mut().set_id(_id)
    }

    fn drag_destinations(
        &self,
        _state: &tree::Tree,
        _layout: cosmic::iced_core::Layout<'_>,
        _dnd_rectangles: &mut cosmic::iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.0
            .lock()
            .unwrap()
            .as_widget()
            .drag_destinations(_state, _layout, _dnd_rectangles)
    }
}

impl<Message: 'static> From<ArcElementWrapper<Message>> for Element<'static, Message> {
    fn from(wrapper: ArcElementWrapper<Message>) -> Self {
        Element::new(wrapper)
    }
}
