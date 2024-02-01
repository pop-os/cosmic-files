use cosmic::{
    app::Core,
    cosmic_theme,
    iced::{
        alignment::{Horizontal, Vertical},
        keyboard::Modifiers,
        //TODO: export in cosmic::widget
        widget::horizontal_rule,
        Alignment,
        Length,
        Point,
    },
    theme, widget, Element,
};
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

use crate::{fl, mime_icon::mime_icon};

const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
//TODO: configurable
const ICON_SIZE_LIST: u16 = 32;
const ICON_SIZE_GRID: u16 = 64;
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
                    metadata: ItemMetadata::Path { metadata, children },
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
                let name = entry.name.clone();

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
                    metadata: ItemMetadata::Trash { metadata, entry },
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
    EditLocation(Option<Location>),
    GoNext,
    GoPrevious,
    Location(Location),
    RightClick(usize),
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
    pub icon_handle_grid: widget::icon::Handle,
    pub icon_handle_list: widget::icon::Handle,
    pub selected: bool,
    pub click_time: Option<Instant>,
}

impl Item {
    pub fn property_view(&self, core: &Core) -> Element<crate::app::Message> {
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
            ItemMetadata::Path { metadata, children } => {
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
            ItemMetadata::Trash { .. } => {
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
    pub edit_location: Option<Location>,
    pub history_i: usize,
    pub history: Vec<Location>,
}

impl Tab {
    pub fn new(location: Location) -> Self {
        let history = vec![location.clone()];
        Self {
            location,
            context_menu: None,
            items_opt: None,
            view: View::List,
            edit_location: None,
            history_i: 0,
            history,
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
        if let Some(mut location) = cd {
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
                    self.history.push(location);
                }
                true
            } else {
                false
            }
        } else {
            false
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

        widget::column::with_children(vec![
            self.location_view(core),
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
            .into(),
        ])
        .into()
    }

    pub fn grid_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        //TODO: get from config
        let item_width = Length::Fixed(96.0);
        let item_height = Length::Fixed(116.0);

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
                    .height(item_height)
                    .width(item_width),
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
        widget::scrollable(widget::column::with_children(vec![
            self.location_view(core),
            widget::flex_row(children).into(),
        ]))
        .width(Length::Fill)
        .into()
    }

    pub fn list_view(&self, core: &Core) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = core.system_theme().cosmic().spacing;

        //TODO: make adaptive?
        let column_width = Length::Fixed(200.0);

        let mut children: Vec<Element<_>> = Vec::new();

        children.push(self.location_view(core));

        children.push(
            widget::row::with_children(vec![
                widget::text::heading(fl!("name"))
                    .width(Length::Fill)
                    .into(),
                //TODO: do not show modified column when in the trash
                widget::text::heading(fl!("modified"))
                    .width(column_width)
                    .into(),
                widget::text::heading(fl!("size"))
                    .width(column_width)
                    .into(),
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
            for (i, item) in items.iter().enumerate() {
                if item.hidden {
                    hidden += 1;
                    //TODO: SHOW HIDDEN OPTION
                    continue;
                }

                if count > 0 {
                    children.push(horizontal_rule(1).into());
                }

                let modified_text = match &item.metadata {
                    ItemMetadata::Path { metadata, .. } => match metadata.modified() {
                        Ok(time) => chrono::DateTime::<chrono::Local>::from(time)
                            .format("%c")
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
                            .size(ICON_SIZE_LIST)
                            .into(),
                        widget::text(item.name.clone()).width(Length::Fill).into(),
                        widget::text(modified_text).width(column_width).into(),
                        widget::text(size_text).width(column_width).into(),
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
        widget::scrollable(
            widget::column::with_children(children)
                // Hack to make room for scroll bar
                .padding([0, space_xxs, 0, 0]),
        )
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

#[cfg(test)]
mod tests {
    use std::io;

    use log::debug;
    use test_log::test;

    use super::scan_path;
    use crate::app::test_utils::{
        empty_fs, eq_path_item, simple_fs, sort_files, NAME_LEN, NUM_DIRS, NUM_FILES, NUM_NESTED,
    };

    #[test]
    fn scan_path_succeeds_on_valid_path() -> io::Result<()> {
        let fs = simple_fs(NUM_FILES, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        let mut entries: Vec<_> = path
            .read_dir()?
            .map(|maybe_entry| maybe_entry.map(|entry| entry.path()))
            .collect::<io::Result<_>>()?;
        entries.sort_by(|a, b| sort_files(a, b));

        debug!("Calling scan_path(\"{}\")", path.display());
        let actual = scan_path(&path.to_owned());

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
        let fs = simple_fs(NUM_FILES, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        // A nonexisting path within the temp dir
        let invalid_path = path.join("ferris");
        assert!(!invalid_path.exists());

        debug!("Calling scan_path(\"{}\")", invalid_path.display());
        let actual = scan_path(&invalid_path);

        assert!(actual.is_empty());

        Ok(())
    }

    #[test]
    fn scan_path_empty_dir_returns_empty_vec() -> io::Result<()> {
        let fs = empty_fs()?;
        let path = fs.path();

        debug!("Calling scan_path(\"{}\")", path.display());
        let actual = scan_path(&path.to_owned());

        assert_eq!(0, path.read_dir()?.count());
        assert_eq!(0, actual.len());

        Ok(())
    }
}
