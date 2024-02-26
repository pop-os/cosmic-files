use cosmic::{
    app::Core,
    cosmic_theme,
    iced::{
        alignment::{Horizontal, Vertical},
        futures::SinkExt,
        keyboard::Modifiers,
        subscription::{self, Subscription},
        //TODO: export in cosmic::widget
        widget::{horizontal_rule, scrollable::Viewport},
        Alignment,
        ContentFit,
        Length,
        Point,
        Rectangle,
        Size,
    },
    theme, widget, Element,
};
use mime_guess::MimeGuess;
use once_cell::sync::Lazy;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    fs::{self, Metadata},
    path::PathBuf,
    process,
    time::{Duration, Instant},
};

use crate::{
    app::Action,
    config::{IconSizes, TabConfig},
    dialog::DialogKind,
    fl, menu,
    mime_icon::mime_icon,
    mouse_area,
};

const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
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
fn button_style(selected: bool) -> theme::Button {
    //TODO: move to libcosmic
    theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            let mut appearance =
                widget::button::StyleSheet::active(theme, focused, &theme::Button::MenuItem);
            if !selected {
                appearance.background = None;
            }
            appearance
        }),
        disabled: Box::new(move |theme| {
            let mut appearance =
                widget::button::StyleSheet::disabled(theme, &theme::Button::MenuItem);
            if !selected {
                appearance.background = None;
            }
            appearance
        }),
        hovered: Box::new(move |focused, theme| {
            widget::button::StyleSheet::hovered(theme, focused, &theme::Button::MenuItem)
        }),
        pressed: Box::new(move |focused, theme| {
            widget::button::StyleSheet::pressed(theme, focused, &theme::Button::MenuItem)
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

#[cfg(target_os = "linux")]
fn open_command(path: &PathBuf) -> process::Command {
    let mut command = process::Command::new("xdg-open");
    command.arg(path);
    command
}

#[cfg(target_os = "macos")]
fn open_command(path: &PathBuf) -> process::Command {
    let mut command = process::Command::new("open");
    command.arg(path);
    command
}

#[cfg(target_os = "redox")]
fn open_command(path: &PathBuf) -> process::Command {
    let mut command = process::Command::new("launcher");
    command.arg(path);
    command
}

#[cfg(target_os = "windows")]
fn open_command(path: &PathBuf) -> process::Command {
    use std::os::windows::process::CommandExt;

    let mut command = process::Command::new("cmd");

    command
        .arg("/c")
        .arg("start")
        .raw_arg("\"\"")
        .arg(path)
        .creation_flags(0x08000000);
    command
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

                let mime_guess = MimeGuess::from_path(&path);

                let (icon_handle_grid, icon_handle_list) = if metadata.is_dir() {
                    (
                        folder_icon(&path, sizes.grid()),
                        folder_icon(&path, sizes.list()),
                    )
                } else {
                    (
                        mime_icon(&path, sizes.grid()),
                        mime_icon(&path, sizes.list()),
                    )
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
                    path,
                    mime_guess,
                    icon_handle_grid,
                    icon_handle_list,
                    thumbnail_res_opt: match mime_guess.first() {
                        Some(mime) if mime.type_() == "image" => None,
                        _ => Some(Err(())),
                    },
                    selected: false,
                    click_time: None,
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
pub fn scan_trash() -> Vec<Item> {
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

                let path = entry.original_path();
                let name = entry.name.clone();

                let mime_guess = MimeGuess::from_path(&path);

                let (icon_handle_grid, icon_handle_list) = match metadata.size {
                    trash::TrashItemSize::Entries(_) => (
                        folder_icon(&path, sizes.grid()),
                        folder_icon(&path, sizes.list()),
                    ),
                    trash::TrashItemSize::Bytes(_) => (
                        mime_icon(&path, sizes.grid()),
                        mime_icon(&path, sizes.list()),
                    ),
                };

                items.push(Item {
                    name,
                    metadata: ItemMetadata::Trash { metadata, entry },
                    hidden: false,
                    path,
                    mime_guess,
                    icon_handle_grid,
                    icon_handle_list,
                    thumbnail_res_opt: Some(Err(())),
                    selected: false,
                    click_time: None,
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
    None,
    Action(Action),
    ChangeLocation(String, Location),
}

#[derive(Clone, Debug)]
pub enum Message {
    Click(Option<usize>),
    Config(TabConfig),
    ContextAction(Action),
    ContextMenu(Option<Point>),
    Drag(Option<Point>),
    EditLocation(Option<Location>),
    GoNext,
    GoPrevious,
    Location(Location),
    LocationUp,
    RightClick(usize),
    Scroll(Viewport),
    Thumbnail(PathBuf, Result<image::RgbaImage, ()>),
    ToggleShowHidden,
    View(View),
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

#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub metadata: ItemMetadata,
    pub hidden: bool,
    pub path: PathBuf,
    pub mime_guess: MimeGuess,
    pub icon_handle_grid: widget::icon::Handle,
    pub icon_handle_list: widget::icon::Handle,
    pub thumbnail_res_opt: Option<Result<image::RgbaImage, ()>>,
    pub selected: bool,
    pub click_time: Option<Instant>,
}

impl Item {
    pub fn property_view(&self, core: &Core, sizes: IconSizes) -> Element<crate::app::Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = core.system_theme().cosmic().spacing;

        let mut column = widget::column().spacing(space_xxxs);

        let is_image = if let Some(mime) = self.mime_guess.first() {
            mime.type_() == "image" && self.path.is_file()
        } else {
            false
        };

        column = column.push(widget::row::with_children(vec![
            widget::horizontal_space(Length::Fill).into(),
            if is_image {
                widget::image::viewer(widget::image::Handle::from_path(&self.path))
                    .min_scale(1.0)
                    .into()
            } else {
                widget::icon::icon(self.icon_handle_grid.clone())
                    .content_fit(ContentFit::Contain)
                    .size(sizes.grid())
                    .into()
            },
            widget::horizontal_space(Length::Fill).into(),
        ]));

        column = column.push(widget::text::heading(self.name.clone()));

        if let Some(mime) = self.mime_guess.first() {
            column = column.push(widget::text(format!("Type: {}", mime)));
        }

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

impl fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Item")
            .field("name", &self.name)
            .field("metadata", &self.metadata)
            .field("hidden", &self.hidden)
            .field("path", &self.path)
            // icon_handles
            .field("selected", &self.selected)
            .field("click_time", &self.click_time)
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum View {
    Grid,
    List,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub location: Location,
    pub context_menu: Option<Point>,
    pub items_opt: Option<Vec<Item>>,
    pub view: View,
    pub dialog: Option<DialogKind>,
    pub drag_opt: Option<Point>,
    pub scroll_opt: Option<Viewport>,
    pub edit_location: Option<Location>,
    pub history_i: usize,
    pub history: Vec<Location>,
    pub config: TabConfig,
}

impl Tab {
    pub fn new(location: Location, config: TabConfig) -> Self {
        let history = vec![location.clone()];
        Self {
            location,
            context_menu: None,
            items_opt: None,
            view: View::Grid,
            dialog: None,
            drag_opt: None,
            scroll_opt: None,
            edit_location: None,
            history_i: 0,
            history,
            config,
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

    fn select_by_drag(&mut self, rect: Rectangle) {
        let items = match &mut self.items_opt {
            Some(some) => some,
            None => return,
        };

        println!("{:?}", rect);
        let (row, col) = match self.view {
            View::Grid => (0, 0),
            View::List => (0, 0),
        };
        for (i, item) in items.iter_mut().enumerate() {
            item.selected = false;
            //TODO
        }
    }

    pub fn update(&mut self, message: Message, modifiers: Modifiers) -> Command {
        let mut cd = None;
        let mut history_i_opt = None;
        match message {
            Message::Click(click_i_opt) => {
                if let Some(ref mut items) = self.items_opt {
                    for (i, item) in items.iter_mut().enumerate() {
                        if Some(i) == click_i_opt {
                            item.selected = true;
                            if let Some(click_time) = item.click_time {
                                if click_time.elapsed() < DOUBLE_CLICK_DURATION {
                                    match self.location {
                                        Location::Path(_) => {
                                            if item.path.is_dir() {
                                                cd = Some(Location::Path(item.path.clone()));
                                            } else if !self.dialog.is_some() {
                                                let mut command = open_command(&item.path);
                                                match command.spawn() {
                                                    Ok(_) => (),
                                                    Err(err) => {
                                                        log::warn!(
                                                            "failed to open {:?}: {}",
                                                            item.path,
                                                            err
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                        Location::Trash => {
                                            //TODO: open properties?
                                        }
                                    }
                                }
                            }
                            //TODO: prevent triple-click and beyond from opening file?
                            item.click_time = Some(Instant::now());
                        } else if modifiers.contains(Modifiers::CTRL)
                            && self.dialog.as_ref().map_or(true, |x| x.multiple())
                        {
                            // Holding control allows multiple selection
                            item.click_time = None;
                        } else {
                            item.selected = false;
                            item.click_time = None;
                        }
                    }
                }
                self.context_menu = None;
            }
            Message::Config(config) => {
                self.config = config;
            }
            Message::ContextAction(action) => {
                // Close context menu
                self.context_menu = None;

                return Command::Action(action);
            }
            Message::ContextMenu(point_opt) => {
                self.context_menu = point_opt;
            }
            Message::Drag(point_opt) => match point_opt {
                Some(point) => {
                    let drag = match self.drag_opt {
                        Some(some) => some,
                        None => {
                            self.drag_opt = Some(point);
                            point
                        }
                    };
                    let min_x = drag.x.min(point.x);
                    let max_x = drag.x.max(point.x);
                    let min_y = drag.y.min(point.y);
                    let max_y = drag.y.max(point.y);
                    let offset_y = self
                        .scroll_opt
                        .map(|x| x.absolute_offset().y)
                        .unwrap_or_default();
                    let rect = Rectangle::new(
                        Point::new(min_x, min_y + offset_y),
                        Size::new(max_x - min_x, max_y - min_y),
                    );
                    self.select_by_drag(rect);
                }
                None => {
                    self.drag_opt = None;
                }
            },
            Message::EditLocation(edit_location) => {
                self.edit_location = edit_location;
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
            Message::Location(location) => {
                cd = Some(location);
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
            Message::RightClick(click_i) => {
                if let Some(ref mut items) = self.items_opt {
                    if !items.get(click_i).map_or(false, |x| x.selected) {
                        // If item not selected, clear selection on other items
                        for (i, item) in items.iter_mut().enumerate() {
                            if i == click_i {
                                item.selected = true;
                            } else if modifiers.contains(Modifiers::CTRL)
                                && self.dialog.as_ref().map_or(true, |x| x.multiple())
                            {
                                // Holding control allows multiple selection
                            } else {
                                item.selected = false;
                            }
                            item.click_time = None;
                        }
                    }
                }
            }
            Message::Scroll(viewport) => {
                self.scroll_opt = Some(viewport);
            }
            Message::Thumbnail(path, thumbnail_res) => {
                if let Some(ref mut items) = self.items_opt {
                    for item in items.iter_mut() {
                        if item.path == path {
                            if let Ok(thumbnail) = &thumbnail_res {
                                //TODO: pass handles already generated to avoid blocking main thread
                                let handle = widget::icon::from_raster_pixels(
                                    thumbnail.width(),
                                    thumbnail.height(),
                                    thumbnail.as_raw().clone(),
                                );
                                item.icon_handle_grid = handle.clone();
                                item.icon_handle_list = handle;
                            }
                            item.thumbnail_res_opt = Some(thumbnail_res);
                            break;
                        }
                    }
                }
            }
            Message::ToggleShowHidden => self.config.show_hidden = !self.config.show_hidden,
            Message::View(view) => {
                self.view = view;
            }
        }
        if let Some(location) = cd {
            if location != self.location {
                self.location = location.clone();
                self.items_opt = None;
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
                Command::ChangeLocation(self.title(), location)
            } else {
                Command::None
            }
        } else {
            Command::None
        }
    }

    pub fn location_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_xxs,
            space_s,
            ..
        } = core.system_theme().cosmic().spacing;

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

    pub fn empty_view(&self, has_hidden: bool, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

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

    pub fn grid_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        //TODO: get from config
        let item_width = Length::Fixed(96.0);
        let item_height = Length::Fixed(116.0);
        let TabConfig {
            show_hidden,
            icon_sizes,
        } = self.config;

        let mut children: Vec<Element<_>> = Vec::new();
        if let Some(ref items) = self.items_opt {
            let mut count = 0;
            let mut hidden = 0;
            for (i, item) in items.iter().enumerate() {
                if !show_hidden && item.hidden {
                    hidden += 1;
                    continue;
                }

                let button = widget::button(
                    widget::column::with_children(vec![
                        widget::icon::icon(item.icon_handle_grid.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_sizes.grid())
                            .into(),
                        widget::text(item.name.clone()).into(),
                    ])
                    .align_items(Alignment::Center)
                    .spacing(space_xxs)
                    .height(item_height)
                    .width(item_width),
                )
                .padding(0)
                .style(button_style(item.selected))
                .on_press(Message::Click(Some(i)));
                if self.context_menu.is_some() {
                    children.push(button.into());
                } else {
                    children.push(
                        mouse_area::MouseArea::new(button)
                            .on_right_press_no_capture(move |_point_opt| Message::RightClick(i))
                            .into(),
                    );
                }
                count += 1;
            }

            if count == 0 {
                return self.empty_view(hidden > 0, core);
            }
        }
        widget::scrollable(widget::flex_row(children))
            .on_scroll(Message::Scroll)
            .width(Length::Fill)
            .into()
    }

    pub fn list_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        //TODO: make adaptive?
        let modified_width = Length::Fixed(200.0);
        let size_width = Length::Fixed(100.0);

        let mut children: Vec<Element<_>> = Vec::new();

        children.push(
            widget::row::with_children(vec![
                widget::text::heading(fl!("name"))
                    .width(Length::Fill)
                    .into(),
                //TODO: do not show modified column when in the trash
                widget::text::heading(fl!("modified"))
                    .width(modified_width)
                    .into(),
                widget::text::heading(fl!("size")).width(size_width).into(),
            ])
            .align_items(Alignment::Center)
            .padding(space_xxs)
            .spacing(space_xxs)
            .into(),
        );

        children.push(horizontal_rule(1).into());

        if let Some(ref items) = self.items_opt {
            let mut count = 0;
            let mut hidden = 0;
            let TabConfig {
                show_hidden,
                icon_sizes,
            } = self.config;
            for (i, item) in items.iter().enumerate() {
                if !show_hidden && item.hidden {
                    hidden += 1;
                    continue;
                }

                if count > 0 {
                    children.push(horizontal_rule(1).into());
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

                //TODO: align columns
                let button = widget::button(
                    widget::row::with_children(vec![
                        widget::icon::icon(item.icon_handle_list.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_sizes.list())
                            .into(),
                        widget::text(item.name.clone()).width(Length::Fill).into(),
                        widget::text(modified_text).width(modified_width).into(),
                        widget::text(size_text).width(size_width).into(),
                    ])
                    .align_items(Alignment::Center)
                    .spacing(space_xxs),
                )
                .padding(space_xxs)
                .style(button_style(item.selected))
                .on_press(Message::Click(Some(i)));
                if self.context_menu.is_some() {
                    children.push(button.into());
                } else {
                    children.push(
                        mouse_area::MouseArea::new(button)
                            .on_right_press_no_capture(move |_point_opt| Message::RightClick(i))
                            .into(),
                    );
                }
                count += 1;
            }

            if count == 0 {
                return self.empty_view(hidden > 0, core);
            }
        }

        widget::scrollable(
            widget::column::with_children(children)
                // Hack to make room for scroll bar
                .padding([0, space_xxs, 0, 0]),
        )
        .on_scroll(Message::Scroll)
        .width(Length::Fill)
        .into()
    }

    pub fn view(&self, core: &Core) -> Element<Message> {
        let location_view = self.location_view(core);
        let item_view = match self.view {
            View::Grid => self.grid_view(core),
            View::List => self.list_view(core),
        };
        let mut mouse_area =
            mouse_area::MouseArea::new(widget::container(item_view).height(Length::Fill))
                .on_drag(move |point_opt| Message::Drag(point_opt))
                .on_press(move |_point_opt| Message::Click(None))
                .on_release(move |point_opt| Message::Drag(None))
                .on_back_press(move |_point_opt| Message::GoPrevious)
                .on_forward_press(move |_point_opt| Message::GoNext)
                .show_drag_box(true);
        if self.context_menu.is_some() {
            mouse_area = mouse_area.on_right_press(move |_point_opt| Message::ContextMenu(None));
        } else {
            mouse_area =
                mouse_area.on_right_press(move |point_opt| Message::ContextMenu(point_opt));
        }
        let mut popover = widget::popover(mouse_area, menu::context_menu(&self));
        match self.context_menu {
            Some(point) => {
                let rounded = Point::new(point.x.round(), point.y.round());
                popover = popover.position(rounded);
            }
            None => {
                popover = popover.show_popup(false);
            }
        }
        widget::container(widget::column::with_children(vec![
            location_view,
            popover.into(),
        ]))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if let Some(items) = &self.items_opt {
            //TODO: how many thumbnail loads should be in flight at once?
            let jobs = 8;
            let mut subscriptions = Vec::with_capacity(jobs);
            for item in items.iter() {
                match item.thumbnail_res_opt {
                    Some(_) => continue,
                    None => {
                        let path = item.path.clone();
                        subscriptions.push(subscription::channel(
                            path.clone(),
                            1,
                            |mut output| async move {
                                let (path, thumbnail_res) =
                                    tokio::task::spawn_blocking(move || {
                                        let start = std::time::Instant::now();
                                        let thumbnail_res = match image::io::Reader::open(&path) {
                                            Ok(reader) => match reader.decode() {
                                                Ok(image) => {
                                                    //TODO: configurable thumbnail size
                                                    let thumbnail = image.thumbnail(64, 64);
                                                    Ok(thumbnail.to_rgba8())
                                                }
                                                Err(err) => {
                                                    log::warn!(
                                                        "failed to decode {:?}: {}",
                                                        path,
                                                        err
                                                    );
                                                    Err(())
                                                }
                                            },
                                            Err(err) => {
                                                log::warn!("failed to read {:?}: {}", path, err);
                                                Err(())
                                            }
                                        };
                                        log::info!(
                                            "thumbnailed {:?} in {:?}",
                                            path,
                                            start.elapsed()
                                        );
                                        (path, thumbnail_res)
                                    })
                                    .await
                                    .unwrap();

                                match output
                                    .send(Message::Thumbnail(path.clone(), thumbnail_res))
                                    .await
                                {
                                    Ok(()) => {}
                                    Err(err) => {
                                        log::warn!(
                                            "failed to send thumbnail for {:?}: {}",
                                            path,
                                            err
                                        );
                                    }
                                }

                                //TODO: how to properly kill this task?
                                loop {
                                    tokio::time::sleep(std::time::Duration::new(1, 0)).await;
                                }
                            },
                        ));
                    }
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

    use super::{scan_path, Item, Location, Message, Tab};
    use crate::{
        app::test_utils::{
            assert_eq_tab_path, assert_eq_tab_path_contents, empty_fs, eq_path_item, filter_dirs,
            read_dir_sorted, simple_fs, sort_files, tab_click_new, NAME_LEN, NUM_DIRS, NUM_FILES,
            NUM_HIDDEN, NUM_NESTED,
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
        debug!("Emitting first Message::Click(Some(1))");
        tab.update(Message::Click(Some(1)), Modifiers::empty());
        debug!("Emitting second Message::Click(Some(1))");
        tab.update(Message::Click(Some(1)), Modifiers::empty());

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
