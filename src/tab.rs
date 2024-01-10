use cosmic::{
    app::Core,
    cosmic_theme,
    iced::{
        alignment::{Horizontal, Vertical},
        keyboard::Modifiers,
        Alignment, Length, Point,
    },
    theme, widget, Element,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    fs::{self, Metadata},
    path::PathBuf,
    process,
    time::{Duration, Instant},
};

use crate::{fl, mime_icon::mime_icon};

const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
//TODO: configurable
const ICON_SIZE_LIST: u16 = 32;
const ICON_SIZE_GRID: u16 = 64;

lazy_static::lazy_static! {
    static ref SPECIAL_DIRS: HashMap<PathBuf, &'static str> = {
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
    };
}

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

fn trash_icon_symbolic(icon_size: u16) -> widget::icon::Handle {
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
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    const GIB: u64 = 1024 * MIB;
    const TIB: u64 = 1024 * GIB;

    if size >= 4 * TIB {
        format!("{:.1} TiB", size as f64 / TIB as f64)
    } else if size >= GIB {
        format!("{:.1} GiB", size as f64 / GIB as f64)
    } else if size >= MIB {
        format!("{:.1} MiB", size as f64 / MIB as f64)
    } else if size >= KIB {
        format!("{:.1} KiB", size as f64 / KIB as f64)
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
    let mut command = process::Command::new("cmd");
    command.arg("/c");
    command.arg("start");
    command.arg(path);
    command
}

pub fn scan_path(tab_path: &PathBuf) -> Vec<Item> {
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

                //TODO: configurable size
                let (icon_handle_grid, icon_handle_list) = if metadata.is_dir() {
                    (
                        folder_icon(&path, ICON_SIZE_GRID),
                        folder_icon(&path, ICON_SIZE_LIST),
                    )
                } else {
                    (
                        mime_icon(&path, ICON_SIZE_GRID),
                        mime_icon(&path, ICON_SIZE_LIST),
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
                    metadata: ItemMetadata::Path(metadata, children),
                    hidden,
                    path,
                    icon_handle_grid,
                    icon_handle_list,
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
pub fn scan_trash() -> Vec<Item> {
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
                let name = entry.name;

                //TODO: configurable size
                let (icon_handle_grid, icon_handle_list) = match metadata.size {
                    trash::TrashItemSize::Entries(_) => (
                        folder_icon(&path, ICON_SIZE_GRID),
                        folder_icon(&path, ICON_SIZE_LIST),
                    ),
                    trash::TrashItemSize::Bytes(_) => (
                        mime_icon(&path, ICON_SIZE_GRID),
                        mime_icon(&path, ICON_SIZE_LIST),
                    ),
                };

                items.push(Item {
                    name,
                    metadata: ItemMetadata::Trash(metadata),
                    hidden: false,
                    path,
                    icon_handle_grid,
                    icon_handle_list,
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
    pub fn scan(&self) -> Vec<Item> {
        match self {
            Self::Path(path) => scan_path(path),
            Self::Trash => scan_trash(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Click(Option<usize>),
    Location(Location),
    RightClick(usize),
    View(View),
}

#[derive(Clone, Debug)]
pub enum ItemMetadata {
    Path(Metadata, usize),
    Trash(trash::TrashItemMetadata),
}

impl ItemMetadata {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Path(metadata, _) => metadata.is_dir(),
            Self::Trash(metadata) => match metadata.size {
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
    pub icon_handle_grid: widget::icon::Handle,
    pub icon_handle_list: widget::icon::Handle,
    pub selected: bool,
    pub click_time: Option<Instant>,
}

impl Item {
    pub fn property_view(&self, core: &Core) -> Element<crate::Message> {
        let mut section = widget::settings::view_section("");
        section = section.add(widget::settings::item::item_row(vec![
            widget::icon::icon(self.icon_handle_list.clone())
                .size(ICON_SIZE_LIST)
                .into(),
            widget::text(self.name.clone()).into(),
        ]));

        //TODO: translate!
        //TODO: correct display of folder size?
        match &self.metadata {
            ItemMetadata::Path(metadata, children) => {
                if metadata.is_dir() {
                    section = section.add(widget::settings::item::item(
                        "Items",
                        widget::text(format!("{}", children)),
                    ));
                } else {
                    section = section.add(widget::settings::item::item(
                        "Size",
                        widget::text(format_size(metadata.len())),
                    ));
                }

                if let Ok(time) = metadata.accessed() {
                    section = section.add(widget::settings::item(
                        "Accessed",
                        widget::text(
                            chrono::DateTime::<chrono::Local>::from(time)
                                .format("%c")
                                .to_string(),
                        ),
                    ));
                }

                if let Ok(time) = metadata.modified() {
                    section = section.add(widget::settings::item(
                        "Modified",
                        widget::text(
                            chrono::DateTime::<chrono::Local>::from(time)
                                .format("%c")
                                .to_string(),
                        ),
                    ));
                }

                if let Ok(time) = metadata.created() {
                    section = section.add(widget::settings::item(
                        "Created",
                        widget::text(
                            chrono::DateTime::<chrono::Local>::from(time)
                                .format("%c")
                                .to_string(),
                        ),
                    ));
                }
            }
            ItemMetadata::Trash(_metadata) => {
                //TODO: trash metadata
            }
        }

        section.into()
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
    //TODO
    pub context_menu: Option<Point>,
    pub items_opt: Option<Vec<Item>>,
    pub view: View,
}

impl Tab {
    pub fn new(location: Location) -> Self {
        Self {
            location,
            context_menu: None,
            items_opt: None,
            view: View::List,
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

    pub fn update(&mut self, message: Message, modifiers: Modifiers) -> bool {
        let mut cd = None;
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
                                            } else {
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
                        } else if modifiers.contains(Modifiers::CTRL) {
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
            Message::Location(location) => {
                cd = Some(location);
            }
            Message::RightClick(click_i) => {
                if let Some(ref mut items) = self.items_opt {
                    if !items.get(click_i).map_or(false, |x| x.selected) {
                        // If item not selected, clear selection on other items
                        for (i, item) in items.iter_mut().enumerate() {
                            if i == click_i {
                                item.selected = true;
                            } else if modifiers.contains(Modifiers::CTRL) {
                                // Holding control allows multiple selection
                            } else {
                                item.selected = false;
                            }
                            item.click_time = None;
                        }
                    }
                }
            }
            Message::View(view) => {
                self.view = view;
            }
        }
        if let Some(location) = cd {
            if location != self.location {
                self.location = location;
                self.items_opt = None;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn breadcrumbs_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = core.system_theme().cosmic().spacing;

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
                    match ancestor.file_name() {
                        Some(name) => {
                            if ancestor == home_dir {
                                row = row.push(
                                    widget::icon::icon(folder_icon_symbolic(&ancestor, 16))
                                        .size(16),
                                );
                                found_home = true;
                            }
                            row = row.push(widget::text(name.to_string_lossy().to_string()));
                        }
                        None => {
                            row = row.push(
                                widget::icon::from_name("drive-harddisk-system-symbolic")
                                    .size(16)
                                    .icon(),
                            );
                            row = row.push(widget::text(fl!("filesystem")));
                        }
                    }

                    if !children.is_empty() {
                        children.push(widget::text("/").into());
                    }

                    children.push(
                        widget::button(row)
                            .padding(space_xxxs)
                            .on_press(Message::Location(Location::Path(ancestor)))
                            .style(theme::Button::Text)
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
                row = row.push(widget::text(fl!("trash")));

                children.push(
                    widget::button(row)
                        .padding(space_xxxs)
                        .on_press(Message::Location(Location::Trash))
                        .style(theme::Button::Text)
                        .into(),
                );
            }
        }
        children.push(widget::horizontal_space(Length::Fill).into());

        widget::container(widget::row::with_children(children).align_items(Alignment::Center))
            .style(theme::Container::Primary)
            .into()
    }

    pub fn empty_view(&self, has_hidden: bool, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        widget::container(
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
        .into()
    }

    pub fn grid_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        let mut children: Vec<Element<_>> = Vec::new();
        if let Some(ref items) = self.items_opt {
            let mut count = 0;
            let mut hidden = 0;
            for (i, item) in items.iter().enumerate() {
                if item.hidden {
                    hidden += 1;
                    //TODO: SHOW HIDDEN OPTION
                    continue;
                }

                let button = widget::button(
                    widget::column::with_children(vec![
                        widget::icon::icon(item.icon_handle_grid.clone())
                            .size(ICON_SIZE_GRID)
                            .into(),
                        widget::text(item.name.clone()).into(),
                    ])
                    .align_items(Alignment::Center)
                    .spacing(space_xxs)
                    //TODO: get from config
                    .height(Length::Fixed(128.0))
                    .width(Length::Fixed(128.0)),
                )
                .style(button_style(item.selected))
                .on_press(Message::Click(Some(i)));
                if self.context_menu.is_some() {
                    children.push(button.into());
                } else {
                    children.push(
                        crate::mouse_area::MouseArea::new(button)
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
            .width(Length::Fill)
            .into()
    }

    pub fn list_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        let mut children: Vec<Element<_>> = Vec::new();

        children.push(
            //TODO: translate
            widget::row::with_children(vec![
                widget::text("Name").into(),
                widget::horizontal_space(Length::Fill).into(),
                widget::text("Size").into(),
                // Hack to make room for scroll bar
                widget::horizontal_space(Length::Fixed(space_xxs as f32)).into(),
            ])
            .align_items(Alignment::Center)
            .padding(space_xxs)
            .spacing(space_xxs)
            .into(),
        );

        //TODO: export in cosmic::widget
        children.push(cosmic::iced::widget::horizontal_rule(1).into());

        if let Some(ref items) = self.items_opt {
            let mut count = 0;
            let mut hidden = 0;
            for (i, item) in items.iter().enumerate() {
                if item.hidden {
                    hidden += 1;
                    //TODO: SHOW HIDDEN OPTION
                    continue;
                }

                //TODO: align columns
                let button = widget::button(
                    widget::row::with_children(vec![
                        widget::icon::icon(item.icon_handle_list.clone())
                            .size(ICON_SIZE_LIST)
                            .into(),
                        widget::text(item.name.clone()).into(),
                        widget::horizontal_space(Length::Fill).into(),
                        widget::text(match &item.metadata {
                            ItemMetadata::Path(metadata, children) => {
                                if metadata.is_dir() {
                                    format!("{} items", children)
                                } else {
                                    format_size(metadata.len())
                                }
                            }
                            ItemMetadata::Trash(metadata) => match metadata.size {
                                trash::TrashItemSize::Entries(entries) => {
                                    format!("{} items", entries)
                                }
                                trash::TrashItemSize::Bytes(bytes) => format_size(bytes),
                            },
                        })
                        .into(),
                        // Hack to make room for scroll bar
                        widget::horizontal_space(Length::Fixed(space_xxs as f32)).into(),
                    ])
                    .align_items(Alignment::Center)
                    .spacing(space_xxs),
                )
                .style(button_style(item.selected))
                .on_press(Message::Click(Some(i)));
                if self.context_menu.is_some() {
                    children.push(button.into());
                } else {
                    children.push(
                        crate::mouse_area::MouseArea::new(button)
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
        widget::scrollable(widget::column::with_children(children))
            .width(Length::Fill)
            .into()
    }

    pub fn view(&self, core: &Core) -> Element<Message> {
        widget::container(match self.view {
            View::Grid => self.grid_view(core),
            View::List => self.list_view(core),
        })
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}
