use cosmic::{
    app::Core,
    cosmic_theme,
    iced::{
        alignment::{Horizontal, Vertical},
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

fn folder_icon(path: &PathBuf, icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(SPECIAL_DIRS.get(path).map_or("folder", |x| *x))
        .size(icon_size)
        .handle()
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

pub fn rescan(tab_path: PathBuf) -> Vec<Item> {
    let mut items = Vec::new();
    match fs::read_dir(&tab_path) {
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

                items.push(Item {
                    name,
                    metadata,
                    hidden,
                    path,
                    icon_handle_grid,
                    icon_handle_list,
                    select_time: None,
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

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Click(Option<usize>),
    Home,
    Parent,
}

#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub metadata: Metadata,
    pub hidden: bool,
    pub path: PathBuf,
    pub icon_handle_grid: widget::icon::Handle,
    pub icon_handle_list: widget::icon::Handle,
    pub select_time: Option<Instant>,
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
        if !self.metadata.is_dir() {
            const KIB: u64 = 1024;
            const MIB: u64 = 1024 * KIB;
            const GIB: u64 = 1024 * MIB;
            const TIB: u64 = 1024 * GIB;

            fn format_size(size: u64) -> String {
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

            section = section.add(widget::settings::item::item(
                "Size",
                widget::text(format_size(self.metadata.len())),
            ));
        }

        if let Ok(time) = self.metadata.accessed() {
            section = section.add(widget::settings::item(
                "Accessed",
                widget::text(
                    chrono::DateTime::<chrono::Local>::from(time)
                        .format("%c")
                        .to_string(),
                ),
            ));
        }

        if let Ok(time) = self.metadata.modified() {
            section = section.add(widget::settings::item(
                "Modified",
                widget::text(
                    chrono::DateTime::<chrono::Local>::from(time)
                        .format("%c")
                        .to_string(),
                ),
            ));
        }

        if let Ok(time) = self.metadata.created() {
            section = section.add(widget::settings::item(
                "Created",
                widget::text(
                    chrono::DateTime::<chrono::Local>::from(time)
                        .format("%c")
                        .to_string(),
                ),
            ));
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
            .field("select_time", &self.select_time)
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
    pub path: PathBuf,
    //TODO
    pub context_menu: Option<Point>,
    pub items_opt: Option<Vec<Item>>,
    pub view: View,
}

impl Tab {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: match fs::canonicalize(&path) {
                Ok(absolute) => absolute,
                Err(err) => {
                    log::warn!("failed to canonicalize {:?}: {}", path, err);
                    path
                }
            },
            context_menu: None,
            items_opt: None,
            view: View::Grid,
        }
    }

    pub fn title(&self) -> String {
        //TODO: better title
        format!("{}", self.path.display())
    }

    pub fn update(&mut self, message: Message) -> bool {
        let mut cd = None;
        match message {
            Message::Click(click_i_opt) => {
                if let Some(ref mut items) = self.items_opt {
                    for (i, item) in items.iter_mut().enumerate() {
                        if Some(i) == click_i_opt {
                            if let Some(select_time) = item.select_time {
                                if select_time.elapsed() < DOUBLE_CLICK_DURATION {
                                    if item.metadata.is_dir() {
                                        cd = Some(item.path.clone());
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
                            }
                            //TODO: prevent triple-click and beyond from opening file
                            item.select_time = Some(Instant::now());
                        } else {
                            item.select_time = None;
                        }
                    }
                }
                self.context_menu = None;
            }
            Message::Home => {
                cd = Some(crate::home_dir());
            }
            Message::Parent => {
                if let Some(parent) = self.path.parent() {
                    cd = Some(parent.to_owned());
                }
            }
        }
        if let Some(path) = cd {
            self.path = path;
            self.items_opt = None;
            true
        } else {
            false
        }
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
        .height(Length::Fill)
        .width(Length::Fill)
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

                children.push(
                    widget::button(
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
                    .style(button_style(item.select_time.is_some()))
                    .on_press(Message::Click(Some(i)))
                    .into(),
                );
                count += 1;
            }

            if count == 0 {
                return self.empty_view(hidden > 0, core);
            }
        }
        widget::flex_row(children).into()
    }

    pub fn list_view(&self, core: &Core) -> Element<Message> {
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

                children.push(
                    widget::button(
                        widget::row::with_children(vec![
                            widget::icon::icon(item.icon_handle_list.clone())
                                .size(ICON_SIZE_LIST)
                                .into(),
                            widget::text(item.name.clone()).into(),
                        ])
                        .align_items(Alignment::Center)
                        .spacing(space_xxs),
                    )
                    .style(button_style(item.select_time.is_some()))
                    .width(Length::Fill)
                    .on_press(Message::Click(Some(i)))
                    .into(),
                );
                count += 1;
            }

            if count == 0 {
                return self.empty_view(hidden > 0, core);
            }
        }
        widget::column::with_children(children)
            .width(Length::Fill)
            .into()
    }

    pub fn view(&self, core: &Core) -> Element<Message> {
        widget::container(
            widget::scrollable(match self.view {
                View::Grid => self.grid_view(core),
                View::List => self.list_view(core),
            })
            .width(Length::Fill),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}
