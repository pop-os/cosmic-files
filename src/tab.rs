use cosmic::{
    cosmic_theme, font,
    iced::{
        advanced::{
            graphics,
            text::{self, Paragraph},
        },
        alignment::{Horizontal, Vertical},
        clipboard::dnd::DndAction,
        event,
        futures,
        futures::SinkExt,
        keyboard::Modifiers,
        stream,
        //TODO: export in cosmic::widget
        widget::{
            container, horizontal_rule, rule,
            scrollable::{self, AbsoluteOffset, Viewport},
        },
        Alignment,
        Border,
        Color,
        ContentFit,
        Length,
        Point,
        Rectangle,
        Size,
        Subscription,
    },
    iced_core::{mouse::ScrollDelta, widget::tree},
    theme,
    widget::{
        self,
        menu::{action::MenuAction, key_bind::KeyBind},
        DndDestination, DndSource, Id, Space, Widget,
    },
    Element,
};

use chrono::{DateTime, Utc};
use mime_guess::{mime, Mime};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    cmp::Ordering,
    collections::HashMap,
    fmt::{self, Display},
    fs::{self, File, Metadata},
    io::{BufRead, BufReader},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{atomic, Arc, Mutex, RwLock},
    time::{Duration, Instant, SystemTime},
};
use tokio::sync::mpsc;

use crate::{
    app::{self, Action, PreviewItem, PreviewKind},
    clipboard::{ClipboardCopy, ClipboardKind, ClipboardPaste},
    config::{DesktopConfig, IconSizes, TabConfig, ICON_SCALE_MAX, ICON_SIZE_GRID},
    dialog::DialogKind,
    fl,
    localize::{LANGUAGE_CHRONO, LANGUAGE_SORTER},
    menu,
    mime_app::{mime_apps, MimeApp},
    mime_icon::{mime_for_path, mime_icon},
    mounter::MOUNTERS,
    mouse_area,
    thumbnailer::thumbnailer,
};
use unix_permissions_ext::UNIXPermissionsExt;
use uzers::{get_group_by_gid, get_user_by_uid};

pub const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
pub const HOVER_DURATION: Duration = Duration::from_millis(1600);
//TODO: best limit for search items
const MAX_SEARCH_LATENCY: Duration = Duration::from_millis(20);
const MAX_SEARCH_RESULTS: usize = 200;
//TODO: configurable thumbnail size?
const THUMBNAIL_SIZE: u32 = (ICON_SIZE_GRID as u32) * (ICON_SCALE_MAX as u32);

//TODO: adjust for locales?
const DATE_TIME_FORMAT: &'static str = "%b %-d, %-Y, %-I:%M %p";
const TIME_FORMAT: &'static str = "%-I:%M %p";
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
    highlighted: bool,
    focused: bool,
    accent: bool,
    condensed_radius: bool,
    desktop: bool,
) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut appearance = widget::button::Style::new();
    if selected {
        if accent {
            appearance.background = Some(Color::from(cosmic.accent_color()).into());
            appearance.icon_color = Some(Color::from(cosmic.on_accent_color()));
            appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
        } else {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        }
    } else if highlighted {
        if accent {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
            appearance.icon_color = Some(Color::from(cosmic.on_bg_component_color()));
            appearance.text_color = Some(Color::from(cosmic.on_bg_component_color()));
        } else {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        }
    } else if desktop {
        appearance.background = Some(Color::from(cosmic.bg_color()).into());
        appearance.icon_color = Some(Color::from(cosmic.on_bg_color()));
        appearance.text_color = Some(Color::from(cosmic.on_bg_color()));
    }
    if focused && accent {
        appearance.outline_width = 1.0;
        appearance.outline_color = Color::from(cosmic.accent_color());
        appearance.border_width = 2.0;
        appearance.border_color = Color::TRANSPARENT;
    }
    if condensed_radius {
        appearance.border_radius = cosmic.radius_xs().into();
    } else {
        appearance.border_radius = cosmic.radius_s().into();
    }
    appearance
}

fn button_style(
    selected: bool,
    highlighted: bool,
    accent: bool,
    condensed_radius: bool,
    desktop: bool,
) -> theme::Button {
    //TODO: move to libcosmic?
    theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            button_appearance(
                theme,
                selected,
                highlighted,
                focused,
                accent,
                condensed_radius,
                desktop,
            )
        }),
        disabled: Box::new(move |theme| {
            button_appearance(
                theme,
                selected,
                highlighted,
                false,
                accent,
                condensed_radius,
                desktop,
            )
        }),
        hovered: Box::new(move |focused, theme| {
            button_appearance(
                theme,
                selected,
                highlighted,
                focused,
                accent,
                condensed_radius,
                desktop,
            )
        }),
        pressed: Box::new(move |focused, theme| {
            button_appearance(
                theme,
                selected,
                highlighted,
                focused,
                accent,
                condensed_radius,
                desktop,
            )
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

#[cfg(target_os = "macos")]
pub fn trash_entries() -> usize {
    0
}

#[cfg(not(target_os = "macos"))]
pub fn trash_entries() -> usize {
    match trash::os_limited::list() {
        Ok(entries) => entries.len(),
        Err(_err) => 0,
    }
}

pub fn trash_icon(icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(if trash_entries() > 0 {
        "user-trash-full"
    } else {
        "user-trash"
    })
    .size(icon_size)
    .handle()
}

pub fn trash_icon_symbolic(icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(if trash_entries() > 0 {
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
enum PermissionOwner {
    Owner,
    Group,
    Other,
}

fn format_permissions_owner(metadata: &Metadata, owner: PermissionOwner) -> String {
    return match owner {
        PermissionOwner::Owner => get_user_by_uid(metadata.uid())
            .and_then(|user| user.name().to_str().map(ToOwned::to_owned))
            .unwrap_or_default(),
        PermissionOwner::Group => get_group_by_gid(metadata.gid())
            .and_then(|group| group.name().to_str().map(ToOwned::to_owned))
            .unwrap_or_default(),
        PermissionOwner::Other => String::from(""),
    };
}
fn format_permissions(metadata: &Metadata, owner: PermissionOwner) -> String {
    let mut perms: Vec<String> = Vec::new();
    if match owner {
        PermissionOwner::Owner => metadata.permissions().readable_by_owner(),
        PermissionOwner::Group => metadata.permissions().readable_by_group(),
        PermissionOwner::Other => metadata.permissions().readable_by_other(),
    } {
        perms.push(fl!("read"));
    }
    if match owner {
        PermissionOwner::Owner => metadata.permissions().writable_by_owner(),
        PermissionOwner::Group => metadata.permissions().writable_by_group(),
        PermissionOwner::Other => metadata.permissions().writable_by_other(),
    } {
        perms.push(fl!("write"));
    }
    if match owner {
        PermissionOwner::Owner => metadata.permissions().executable_by_owner(),
        PermissionOwner::Group => metadata.permissions().executable_by_group(),
        PermissionOwner::Other => metadata.permissions().executable_by_other(),
    } {
        perms.push(fl!("execute"));
    }

    perms.join(" ")
}

struct FormatTime(SystemTime);

impl FormatTime {
    fn from_secs(secs: i64) -> Option<Self> {
        // This looks convoluted because we need to ensure the units match up
        let secs: u64 = secs.try_into().ok()?;
        let now = SystemTime::now();
        let filetime_diff = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|from_epoch| from_epoch.as_secs())
            .ok()
            .and_then(|now_secs| now_secs.checked_sub(secs))
            .map(Duration::from_secs)?;
        now.checked_add(filetime_diff).map(FormatTime)
    }
}

impl Display for FormatTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let date_time = chrono::DateTime::<chrono::Local>::from(self.0);
        let now = chrono::Local::now();
        if date_time.date_naive() == now.date_naive() {
            write!(
                f,
                "{}, {}",
                fl!("today"),
                date_time.format_localized(TIME_FORMAT, *LANGUAGE_CHRONO)
            )
        } else {
            date_time
                .format_localized(DATE_TIME_FORMAT, *LANGUAGE_CHRONO)
                .fmt(f)
        }
    }
}

fn format_time(time: SystemTime) -> FormatTime {
    FormatTime(time)
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

pub fn parse_desktop_file(path: &Path) -> (Option<String>, Option<String>) {
    let entry = match freedesktop_entry_parser::parse_entry(path) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse {:?}: {}", path, err);
            return (None, None);
        }
    };
    (
        entry
            .section("Desktop Entry")
            .attr("Name")
            .map(|x| x.to_string()),
        entry
            .section("Desktop Entry")
            .attr("Icon")
            .map(|x| x.to_string()),
    )
}

pub fn item_from_entry(
    path: PathBuf,
    name: String,
    metadata: fs::Metadata,
    sizes: IconSizes,
) -> Item {
    let mut display_name = Item::display_name(&name);

    let hidden = name.starts_with(".") || hidden_attribute(&metadata);

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
            //TODO: clean this up, implement for trash
            let icon_name_opt = if mime == "application/x-desktop" {
                let (desktop_name_opt, icon_name_opt) = parse_desktop_file(&path);
                if let Some(desktop_name) = desktop_name_opt {
                    display_name = Item::display_name(&desktop_name);
                }
                icon_name_opt
            } else {
                None
            };
            if let Some(icon_name) = icon_name_opt {
                (
                    mime.clone(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.grid())
                        .handle(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.list())
                        .handle(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.list_condensed())
                        .handle(),
                )
            } else {
                (
                    mime.clone(),
                    mime_icon(mime.clone(), sizes.grid()),
                    mime_icon(mime.clone(), sizes.list()),
                    mime_icon(mime, sizes.list_condensed()),
                )
            }
        };

    let open_with = mime_apps(&mime);

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

    Item {
        name,
        display_name,
        metadata: ItemMetadata::Path { metadata, children },
        hidden,
        location_opt: Some(Location::Path(path)),
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        open_with,
        thumbnail_opt: None,
        button_id: widget::Id::unique(),
        pos_opt: Cell::new(None),
        rect_opt: Cell::new(None),
        selected: false,
        highlighted: false,
        overlaps_drag_rect: false,
    }
}

pub fn item_from_path<P: Into<PathBuf>>(path: P, sizes: IconSizes) -> Result<Item, String> {
    let path = path.into();
    let name = match path.file_name() {
        Some(name_os) => name_os
            .to_str()
            .ok_or_else(|| {
                format!(
                    "failed to parse file name for {:?}: {:?} is not valid UTF-8",
                    path, name_os
                )
            })?
            .to_string(),
        None => fl!("filesystem"),
    };
    let metadata = fs::metadata(&path)
        .map_err(|err| format!("failed to read metadata for {:?}: {}", path, err))?;
    Ok(item_from_entry(path, name, metadata, sizes))
}

pub fn scan_path(tab_path: &PathBuf, sizes: IconSizes) -> Vec<Item> {
    let mut items = Vec::new();
    let mut hidden_files = Vec::new();
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

                let path = entry.path();

                let name = match entry.file_name().into_string() {
                    Ok(ok) => ok,
                    Err(name_os) => {
                        log::warn!(
                            "failed to parse entry at {:?}: {:?} is not valid UTF-8",
                            path,
                            name_os,
                        );
                        continue;
                    }
                };

                if name == ".hidden" {
                    hidden_files = parse_hidden_file(&path);
                }

                let metadata = match fs::metadata(&path) {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!("failed to read metadata for entry at {:?}: {}", path, err);
                        continue;
                    }
                };

                items.push(item_from_entry(path, name, metadata, sizes));
            }
        }
        Err(err) => {
            log::warn!("failed to read directory {:?}: {}", tab_path, err);
        }
    }
    items.sort_by(|a, b| match (a.metadata.is_dir(), b.metadata.is_dir()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => LANGUAGE_SORTER.compare(&a.display_name, &b.display_name),
    });
    items.iter_mut().for_each(|item| {
        if hidden_files
            .iter()
            .find(|hidden| &&item.name == hidden)
            .is_some()
        {
            item.hidden = true;
        }
    });
    items
}

pub fn scan_search<F: Fn(&Path, &str, Metadata) -> bool + Sync>(
    tab_path: &PathBuf,
    term: &str,
    show_hidden: bool,
    callback: F,
) {
    if term.is_empty() {
        return;
    }

    let pattern = regex::escape(&term);
    let regex = match regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
    {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse regex {:?}: {}", pattern, err);
            return;
        }
    };

    ignore::WalkBuilder::new(tab_path)
        .standard_filters(false)
        .hidden(!show_hidden)
        //TODO: only use this on supported targets
        .same_file_system(true)
        .build_parallel()
        .run(|| {
            Box::new(|entry_res| {
                let Ok(entry) = entry_res else {
                    // Skip invalid entries
                    return ignore::WalkState::Skip;
                };

                let Some(file_name) = entry.file_name().to_str() else {
                    // Skip anything with an invalid name
                    return ignore::WalkState::Skip;
                };

                if regex.is_match(file_name) {
                    let path = entry.path();

                    let metadata = match entry.metadata() {
                        Ok(ok) => ok,
                        Err(err) => {
                            log::warn!("failed to read metadata for entry at {:?}: {}", path, err);
                            return ignore::WalkState::Continue;
                        }
                    };

                    //TODO: use entry.into_path?
                    if !callback(path, file_name, metadata) {
                        return ignore::WalkState::Quit;
                    }
                }

                ignore::WalkState::Continue
            })
        });
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
                let name = entry.name.to_string_lossy().to_string();
                let display_name = Item::display_name(&name);

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
                    display_name,
                    metadata: ItemMetadata::Trash { metadata, entry },
                    hidden: false,
                    location_opt: None,
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
                    highlighted: false,
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
        _ => LANGUAGE_SORTER.compare(&a.display_name, &b.display_name),
    });
    items
}

fn uri_to_path(uri: String) -> Option<PathBuf> {
    //TODO support for external drive or cloud?
    if uri.starts_with("file://") {
        let path_str = &uri[7..];
        Some(PathBuf::from(path_str))
    } else {
        None
    }
}

pub fn scan_recents(sizes: IconSizes) -> Vec<Item> {
    let mut recents = Vec::new();

    match recently_used_xbel::parse_file() {
        Ok(recent_files) => {
            for bookmark in recent_files.bookmarks {
                let uri = bookmark.href;
                let path = match uri_to_path(uri) {
                    None => continue,
                    Some(path) => path,
                };
                let last_edit = match bookmark.modified.parse::<DateTime<Utc>>() {
                    Ok(last_edit) => last_edit,
                    Err(_) => continue,
                };
                let last_visit = match bookmark.visited.parse::<DateTime<Utc>>() {
                    Ok(last_visit) => last_visit,
                    Err(_) => continue,
                };
                let path_buf = PathBuf::from(path);
                let path_exist = path_buf.exists();

                if path_exist {
                    let file_name = path_buf.file_name();

                    if let Some(name) = file_name {
                        let name = name.to_string_lossy().to_string();

                        let metadata = match path_buf.metadata() {
                            Ok(ok) => ok,
                            Err(err) => {
                                log::warn!(
                                    "failed to read metadata for entry at {:?}: {}",
                                    path_buf.clone(),
                                    err
                                );
                                continue;
                            }
                        };

                        let item = item_from_entry(path_buf, name, metadata, sizes);
                        recents.push((
                            item,
                            if last_edit.le(&last_visit) {
                                last_edit
                            } else {
                                last_visit
                            },
                        ))
                    }
                } else {
                    log::warn!("recent file path not exist: {:?}", path_buf);
                }
            }
        }
        Err(err) => {
            log::warn!("Error reading recent files: {:?}", err);
        }
    }

    recents.sort_by(|a, b| b.1.cmp(&a.1));

    recents.into_iter().take(50).map(|(item, _)| item).collect()
}

pub fn scan_network(uri: &str, sizes: IconSizes) -> Vec<Item> {
    for (_key, mounter) in MOUNTERS.iter() {
        match mounter.network_scan(uri, sizes) {
            Some(Ok(items)) => return items,
            Some(Err(err)) => {
                log::warn!("failed to scan {:?}: {}", uri, err);
            }
            None => {}
        }
    }
    Vec::new()
}

//TODO: organize desktop items based on display
pub fn scan_desktop(
    tab_path: &PathBuf,
    _display: &str,
    desktop_config: DesktopConfig,
    sizes: IconSizes,
) -> Vec<Item> {
    let mut items = Vec::new();

    if desktop_config.show_content {
        items.extend(scan_path(tab_path, sizes));
    }

    if desktop_config.show_mounted_drives {
        for (_mounter_key, mounter) in MOUNTERS.iter() {
            for mounter_item in mounter.items(sizes).unwrap_or_default() {
                let Some(path) = mounter_item.path() else {
                    continue;
                };

                // Get most item data from path
                let mut item = match item_from_path(&path, sizes) {
                    Ok(item) => item,
                    Err(err) => {
                        log::warn!("failed to get item from mounter item {:?}: {}", path, err);
                        continue;
                    }
                };

                //Override some data with mounter information
                item.name = mounter_item.name();
                item.display_name = Item::display_name(&item.name);

                //TODO: use icon size for mounter item icon
                if let Some(icon) = mounter_item.icon(false) {
                    item.icon_handle_grid = icon.clone();
                    item.icon_handle_list = icon.clone();
                    item.icon_handle_list_condensed = icon;
                }

                items.push(item);
            }
        }
    }

    if desktop_config.show_trash {
        let name = fl!("trash");
        let display_name = Item::display_name(&name);

        let metadata = ItemMetadata::SimpleDir {
            entries: trash_entries() as u64,
        };

        let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = {
            (
                "inode/directory".parse().unwrap(),
                trash_icon(sizes.grid()),
                trash_icon(sizes.list()),
                trash_icon(sizes.list_condensed()),
            )
        };

        items.push(Item {
            name,
            display_name,
            metadata,
            hidden: false,
            location_opt: Some(Location::Trash),
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
            highlighted: false,
            overlaps_drag_rect: false,
        })
    }

    items
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Location {
    Desktop(PathBuf, String, DesktopConfig),
    Network(String, String),
    Path(PathBuf),
    Recents,
    Search(PathBuf, String, bool, Instant),
    Trash,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Desktop(path, display, ..) => {
                write!(f, "{} on display {display}", path.display())
            }
            Self::Network(uri, ..) => write!(f, "{}", uri),
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::Recents => write!(f, "recents"),
            Self::Search(path, term, ..) => write!(f, "search {} for {}", path.display(), term),
            Self::Trash => write!(f, "trash"),
        }
    }
}

impl Location {
    pub fn path_opt(&self) -> Option<&PathBuf> {
        match self {
            Self::Desktop(path, ..) => Some(&path),
            Self::Path(path) => Some(&path),
            Self::Search(path, ..) => Some(&path),
            _ => None,
        }
    }

    pub fn with_path(&self, path: PathBuf) -> Self {
        match self {
            Self::Desktop(_, display, desktop_config) => {
                Self::Desktop(path, display.clone(), *desktop_config)
            }
            Self::Path(..) => Self::Path(path),
            Self::Search(_, term, show_hidden, _) => {
                Self::Search(path, term.clone(), *show_hidden, Instant::now())
            }
            other => other.clone(),
        }
    }

    pub fn scan(&self, sizes: IconSizes) -> (Option<Item>, Vec<Item>) {
        let items = match self {
            Self::Desktop(path, display, desktop_config) => {
                scan_desktop(path, display, *desktop_config, sizes)
            }
            Self::Path(path) => scan_path(path, sizes),
            Self::Search(..) => {
                // Search is done incrementally
                Vec::new()
            }
            Self::Trash => scan_trash(sizes),
            Self::Recents => scan_recents(sizes),
            Self::Network(uri, _) => scan_network(uri, sizes),
        };
        let parent_item_opt = match self.path_opt() {
            Some(path) => match item_from_path(path, sizes) {
                Ok(item) => Some(item),
                Err(err) => {
                    log::warn!("failed to get item for {:?}: {}", path, err);
                    None
                }
            },
            //TODO: support other locations?
            None => None,
        };
        (parent_item_opt, items)
    }
}

pub struct TaskWrapper(pub cosmic::Task<Message>);

impl From<cosmic::Task<Message>> for TaskWrapper {
    fn from(task: cosmic::Task<Message>) -> Self {
        Self(task)
    }
}

impl fmt::Debug for TaskWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TaskWrapper").finish()
    }
}

#[derive(Debug)]
pub enum Command {
    Action(Action),
    AddNetworkDrive,
    AddToSidebar(PathBuf),
    ChangeLocation(String, Location, Option<PathBuf>),
    DropFiles(PathBuf, ClipboardPaste),
    EmptyTrash,
    Iced(TaskWrapper),
    MoveToTrash(Vec<PathBuf>),
    OpenFile(PathBuf),
    OpenInNewTab(PathBuf),
    OpenInNewWindow(PathBuf),
    OpenTrash,
    Preview(PreviewKind),
    WindowDrag,
    WindowToggleMaximize,
}

#[derive(Clone, Debug)]
pub enum Message {
    AddNetworkDrive,
    Click(Option<usize>),
    DoubleClick(Option<usize>),
    ClickRelease(Option<usize>),
    DragEnd(Option<usize>),
    Config(TabConfig),
    ContextAction(Action),
    ContextMenu(Option<Point>),
    LocationContextMenuPoint(Option<Point>),
    LocationContextMenuIndex(Option<usize>),
    LocationMenuAction(LocationMenuAction),
    Drag(Option<Rectangle>),
    EditLocation(Option<Location>),
    EditLocationEnable,
    OpenInNewTab(PathBuf),
    EmptyTrash,
    Gallery(bool),
    GalleryPrevious,
    GalleryNext,
    GalleryToggle,
    GoNext,
    GoPrevious,
    ItemDown,
    ItemLeft,
    ItemRight,
    ItemUp,
    Location(Location),
    LocationUp,
    Open(Option<PathBuf>),
    RightClick(Option<usize>),
    MiddleClick(usize),
    Scroll(Viewport),
    ScrollToFocus,
    SearchContext(Location, SearchContextWrapper),
    SearchReady(bool),
    SelectAll,
    SetSort(HeadingOptions, bool),
    Thumbnail(PathBuf, ItemThumbnail),
    ToggleShowHidden,
    View(View),
    ToggleSort(HeadingOptions),
    Drop(Option<(Location, ClipboardPaste)>),
    DndHover(Location),
    DndEnter(Location),
    DndLeave(Location),
    WindowDrag,
    WindowToggleMaximize,
    ZoomIn,
    ZoomOut,
    HighlightDeactivate(usize),
    HighlightActivate(usize),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LocationMenuAction {
    OpenInNewTab(usize),
    OpenInNewWindow(usize),
    Preview(usize),
    AddToSidebar(usize),
}

impl MenuAction for LocationMenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::LocationMenuAction(*self)
    }
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
    SimpleDir {
        entries: u64,
    },
    SimpleFile {
        size: u64,
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
            Self::SimpleDir { .. } => true,
            Self::SimpleFile { .. } => false,
        }
    }

    pub fn modified(&self) -> Option<SystemTime> {
        match self {
            Self::Path { metadata, .. } => metadata.modified().ok(),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ItemThumbnail {
    NotImage,
    Image(widget::image::Handle, Option<(u32, u32)>),
    Svg(widget::svg::Handle),
    Text(widget::text_editor::Content),
}

impl Clone for ItemThumbnail {
    fn clone(&self) -> Self {
        match self {
            Self::NotImage => Self::NotImage,
            Self::Image(handle, size_opt) => Self::Image(handle.clone(), *size_opt),
            Self::Svg(handle) => Self::Svg(handle.clone()),
            // Content cannot be cloned simply
            Self::Text(content) => {
                Self::Text(widget::text_editor::Content::with_text(&content.text()))
            }
        }
    }
}

impl ItemThumbnail {
    pub fn new(path: &Path, metadata: fs::Metadata, mime: mime::Mime, thumbnail_size: u32) -> Self {
        let size = metadata.len();
        let check_size = |thumbnailer: &str, max_size| {
            if size <= max_size {
                true
            } else {
                log::warn!(
                    "skipping internal {} thumbnailer for {:?}: file size {} is larger than {}",
                    thumbnailer,
                    path,
                    format_size(size),
                    format_size(max_size)
                );
                false
            }
        };
        //TODO: adjust limits for internal thumbnailers as desired
        if mime.type_() == mime::IMAGE
            && mime.subtype() == mime::SVG
            && check_size("svg", 8 * 1000 * 1000)
        {
            // Try built-in svg thumbnailer
            match fs::read(&path) {
                Ok(data) => {
                    //TODO: validate SVG data
                    return ItemThumbnail::Svg(widget::svg::Handle::from_memory(data));
                }
                Err(err) => {
                    log::warn!("failed to read {:?}: {}", path, err);
                }
            }
        } else if mime.type_() == mime::IMAGE && check_size("image", 64 * 1000 * 1000) {
            // Try built-in image thumbnailer
            match image::io::Reader::open(&path).and_then(|img| img.with_guessed_format()) {
                Ok(reader) => match reader.decode() {
                    Ok(image) => {
                        let thumbnail =
                            image.thumbnail(thumbnail_size, thumbnail_size).into_rgba8();
                        return ItemThumbnail::Image(
                            widget::image::Handle::from_rgba(
                                thumbnail.width(),
                                thumbnail.height(),
                                thumbnail.into_raw(),
                            ),
                            Some((image.width(), image.height())),
                        );
                    }
                    Err(err) => {
                        log::warn!("failed to decode {:?}: {}", path, err);
                    }
                },
                Err(err) => {
                    log::warn!("failed to read {:?}: {}", path, err);
                }
            }
        } else if mime.type_() == mime::TEXT && check_size("text", 8 * 1000 * 1000) {
            match fs::read_to_string(&path) {
                Ok(data) => {
                    return ItemThumbnail::Text(widget::text_editor::Content::with_text(&data));
                }
                Err(err) => {
                    log::warn!("failed to read {:?}: {}", path, err);
                }
            }
        }

        // Try external thumbnailers
        for thumbnailer in thumbnailer(&mime) {
            let prefix = if thumbnailer.exec.starts_with("evince-thumbnailer ") {
                //TODO: apparmor config for evince-thumbnailer does not allow /tmp/cosmic-files*
                "gnome-desktop-"
            } else {
                "cosmic-files-"
            };
            let file = match tempfile::NamedTempFile::with_prefix(prefix) {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!(
                        "failed to create temporary file for thumbnail of {:?}: {}",
                        path,
                        err
                    );
                    continue;
                }
            };

            let Some(mut command) = thumbnailer.command(&path, file.path(), thumbnail_size) else {
                continue;
            };
            match command.status() {
                Ok(status) => {
                    if status.success() {
                        match image::io::Reader::open(file.path())
                            .and_then(|img| img.with_guessed_format())
                        {
                            Ok(reader) => match reader.decode().map(|image| image.into_rgba8()) {
                                Ok(image) => {
                                    return ItemThumbnail::Image(
                                        widget::image::Handle::from_rgba(
                                            image.width(),
                                            image.height(),
                                            image.into_raw(),
                                        ),
                                        None,
                                    );
                                }
                                Err(err) => {
                                    log::warn!("failed to decode {:?}: {}", path, err);
                                }
                            },
                            Err(err) => {
                                log::warn!("failed to read {:?}: {}", path, err);
                            }
                        }
                    } else {
                        log::warn!("failed to run {:?} for {:?}: {}", thumbnailer, path, status);
                    }
                }
                Err(err) => {
                    log::warn!("failed to run {:?} for {:?}: {}", thumbnailer, path, err);
                }
            }
        }

        ItemThumbnail::NotImage
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub display_name: String,
    pub metadata: ItemMetadata,
    pub hidden: bool,
    pub location_opt: Option<Location>,
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
    pub highlighted: bool,
    pub overlaps_drag_rect: bool,
}

impl Item {
    fn display_name(name: &str) -> String {
        // In order to wrap at periods and underscores, add a zero width space after each one
        name.replace(".", ".\u{200B}").replace("_", "_\u{200B}")
    }

    pub fn path_opt(&self) -> Option<&PathBuf> {
        self.location_opt.as_ref()?.path_opt()
    }

    pub fn can_gallery(&self) -> bool {
        self.mime.type_() == mime::IMAGE || self.mime.type_() == mime::TEXT
    }

    fn preview<'a>(&'a self, sizes: IconSizes) -> Element<'a, app::Message> {
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
            ItemThumbnail::Image(handle, _) => {
                if let Some(path) = self.path_opt() {
                    if self.mime.type_() == mime::IMAGE {
                        return widget::image(widget::image::Handle::from_path(path)).into();
                    }
                }
                widget::image(handle.clone()).into()
            }
            ItemThumbnail::Svg(handle) => widget::svg(handle.clone()).into(),
            ItemThumbnail::Text(content) => widget::container(widget::text_editor(content))
                .width(Length::Fixed(THUMBNAIL_SIZE as f32))
                .height(Length::Fixed(THUMBNAIL_SIZE as f32))
                .into(),
        }
    }

    pub fn preview_view<'a>(
        &'a self,
        sizes: IconSizes,
        nav_row: bool,
    ) -> Element<'a, app::Message> {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_xxs,
            space_m,
            ..
        } = theme::active().cosmic().spacing;

        let mut column = widget::column().spacing(space_m);

        if nav_row {
            let mut row = widget::row::with_capacity(3).spacing(space_xxs);
            row = row.push(
                widget::button::icon(widget::icon::from_name("go-previous-symbolic"))
                    .on_press(app::Message::TabMessage(None, Message::ItemLeft)),
            );
            row = row.push(
                widget::button::icon(widget::icon::from_name("go-next-symbolic"))
                    .on_press(app::Message::TabMessage(None, Message::ItemRight)),
            );

            if self.can_gallery() {
                if let Some(_path) = self.path_opt() {
                    row = row.push(
                        widget::button::icon(widget::icon::from_name("view-fullscreen-symbolic"))
                            .on_press(app::Message::TabMessage(None, Message::Gallery(true))),
                    );
                }
            }
            column = column.push(row);
        }

        column = column.push(widget::row::with_children(vec![
            widget::horizontal_space().into(),
            self.preview(sizes),
            widget::horizontal_space().into(),
        ]));

        let mut details = widget::column().spacing(space_xxxs);
        details = details.push(widget::text::heading(self.name.clone()));
        details = details.push(widget::text(format!("Type: {}", self.mime)));
        let mut settings = Vec::new();
        //TODO: translate!
        //TODO: correct display of folder size?
        match &self.metadata {
            ItemMetadata::Path { metadata, children } => {
                if metadata.is_dir() {
                    details = details.push(widget::text(format!("Items: {}", children)));
                } else {
                    details = details.push(widget::text(format!(
                        "Size: {}",
                        format_size(metadata.len())
                    )));
                }

                if let Ok(time) = metadata.created() {
                    details = details.push(widget::text(format!("Created: {}", format_time(time))));
                }

                if let Ok(time) = metadata.modified() {
                    details =
                        details.push(widget::text(format!("Modified: {}", format_time(time))));
                }

                if let Ok(time) = metadata.accessed() {
                    details =
                        details.push(widget::text(format!("Accessed: {}", format_time(time))));
                }

                #[cfg(not(target_os = "windows"))]
                {
                    settings.push(
                        widget::settings::item::builder(format_permissions_owner(
                            metadata,
                            PermissionOwner::Owner,
                        ))
                        .description(fl!("owner"))
                        .control(widget::text(format_permissions(
                            metadata,
                            PermissionOwner::Owner,
                        ))),
                    );

                    settings.push(
                        widget::settings::item::builder(format_permissions_owner(
                            metadata,
                            PermissionOwner::Group,
                        ))
                        .description(fl!("group"))
                        .control(widget::text(format_permissions(
                            metadata,
                            PermissionOwner::Group,
                        ))),
                    );

                    settings.push(widget::settings::item::builder(fl!("other")).control(
                        widget::text(format_permissions(metadata, PermissionOwner::Other)),
                    ));
                }
            }
            _ => {
                //TODO: other metadata types
            }
        }
        match self
            .thumbnail_opt
            .as_ref()
            .unwrap_or(&ItemThumbnail::NotImage)
        {
            ItemThumbnail::Image(_, Some((width, height))) => {
                details = details.push(widget::text(format!("{}x{}", width, height)));
            }
            _ => {}
        }
        column = column.push(details);

        if let Some(path) = self.path_opt() {
            column = column.push(widget::button::standard(fl!("open")).on_press(
                app::Message::TabMessage(None, Message::Open(Some(path.to_path_buf()))),
            ));
        }

        if !settings.is_empty() {
            let mut section = widget::settings::section();
            for setting in settings {
                section = section.add(setting);
            }
            column = column.push(section);
        }

        column.into()
    }

    pub fn replace_view<'a>(
        &'a self,
        heading: String,
        sizes: IconSizes,
    ) -> Element<'a, app::Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;

        let mut row = widget::row().spacing(space_xxxs);
        row = row.push(self.preview(sizes));

        let mut column = widget::column().spacing(space_xxxs);
        column = column.push(widget::text::heading(heading));

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
                if let Ok(time) = metadata.modified() {
                    column = column.push(widget::text(format!(
                        "Last modified: {}",
                        format_time(time)
                    )));
                }
            }
            _ => {
                //TODO: other metadata
            }
        }

        row = row.push(column);
        row.into()
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
    TrashedOn,
}

impl fmt::Display for HeadingOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeadingOptions::Name => write!(f, "{}", fl!("name")),
            HeadingOptions::Modified => write!(f, "{}", fl!("modified")),
            HeadingOptions::Size => write!(f, "{}", fl!("size")),
            HeadingOptions::TrashedOn => write!(f, "{}", fl!("trashed-on")),
        }
    }
}

impl HeadingOptions {
    pub fn names() -> Vec<String> {
        vec![
            HeadingOptions::Name.to_string(),
            HeadingOptions::Modified.to_string(),
            HeadingOptions::Size.to_string(),
            HeadingOptions::TrashedOn.to_string(),
        ]
    }
}

#[derive(Clone, Debug)]
pub enum Mode {
    App,
    Desktop,
    Dialog(DialogKind),
}

impl Mode {
    /// Whether multiple files can be selected in this mode
    pub fn multiple(&self) -> bool {
        match self {
            Mode::App | Mode::Desktop => true,
            Mode::Dialog(dialog) => dialog.multiple(),
        }
    }
}

struct SearchContext {
    results_rx: mpsc::Receiver<(PathBuf, String, Metadata)>,
    ready: Arc<atomic::AtomicBool>,
    last_modified_opt: Arc<RwLock<Option<SystemTime>>>,
}

pub struct SearchContextWrapper(Option<SearchContext>);

impl Clone for SearchContextWrapper {
    fn clone(&self) -> Self {
        Self(None)
    }
}

impl fmt::Debug for SearchContextWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchContextWrapper").finish()
    }
}

// TODO when creating items, pass <Arc<SelectedItems>> to each item
// as a drag data, so that when dnd is initiated, they are all included
pub struct Tab {
    //TODO: make more items private
    pub location: Location,
    pub location_context_menu_point: Option<Point>,
    pub location_context_menu_index: Option<usize>,
    pub context_menu: Option<Point>,
    pub mode: Mode,
    pub scroll_opt: Option<AbsoluteOffset>,
    pub size_opt: Cell<Option<Size>>,
    pub item_view_size_opt: Cell<Option<Size>>,
    pub edit_location: Option<Location>,
    pub edit_location_id: widget::Id,
    pub history_i: usize,
    pub history: Vec<Location>,
    pub config: TabConfig,
    pub sort_name: HeadingOptions,
    pub sort_direction: bool,
    pub gallery: bool,
    pub(crate) parent_item_opt: Option<Item>,
    pub(crate) items_opt: Option<Vec<Item>>,
    pub dnd_hovered: Option<(Location, Instant)>,
    scrollable_id: widget::Id,
    select_focus: Option<usize>,
    select_range: Option<(usize, usize)>,
    cached_selected: RefCell<Option<bool>>,
    clicked: Option<usize>,
    selected_clicked: bool,
    last_right_click: Option<usize>,
    search_context: Option<SearchContext>,
}

fn folder_name<P: AsRef<Path>>(path: P) -> (String, bool) {
    let path = path.as_ref();
    let mut found_home = false;
    let name = match path.file_name() {
        Some(name) => {
            if path == crate::home_dir() {
                found_home = true;
                fl!("home")
            } else {
                name.to_string_lossy().to_string()
            }
        }
        None => {
            fl!("filesystem")
        }
    };
    (name, found_home)
}

// parse .hidden file and return files path
fn parse_hidden_file(path: &PathBuf) -> Vec<String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut paths: Vec<String> = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if !line.is_empty() {
                paths.push(line.trim().to_string());
            }
        }
    }

    paths
}

impl Tab {
    pub fn new(location: Location, config: TabConfig) -> Self {
        let history = vec![location.clone()];
        Self {
            location,
            context_menu: None,
            location_context_menu_point: None,
            location_context_menu_index: None,
            mode: Mode::App,
            scroll_opt: None,
            size_opt: Cell::new(None),
            item_view_size_opt: Cell::new(None),
            edit_location: None,
            edit_location_id: widget::Id::unique(),
            history_i: 0,
            history,
            config,
            sort_name: HeadingOptions::Name,
            sort_direction: true,
            gallery: false,
            parent_item_opt: None,
            items_opt: None,
            scrollable_id: widget::Id::unique(),
            select_focus: None,
            select_range: None,
            cached_selected: RefCell::new(None),
            clicked: None,
            dnd_hovered: None,
            selected_clicked: false,
            last_right_click: None,
            search_context: None,
        }
    }

    pub fn title(&self) -> String {
        match &self.location {
            Location::Desktop(path, _, _) => {
                let (name, _) = folder_name(path);
                name
            }
            Location::Path(path) => {
                let (name, _) = folder_name(path);
                name
            }
            Location::Search(path, term, ..) => {
                //TODO: translate
                let (name, _) = folder_name(path);
                format!("Search \"{}\": {}", term, name)
            }
            Location::Trash => {
                fl!("trash")
            }
            Location::Recents => {
                fl!("recents")
            }
            Location::Network(_uri, display_name) => display_name.clone(),
        }
    }

    pub fn items_opt(&self) -> Option<&Vec<Item>> {
        self.items_opt.as_ref()
    }

    pub fn items_opt_mut(&mut self) -> Option<&mut Vec<Item>> {
        self.items_opt.as_mut()
    }

    pub fn set_items(&mut self, mut items: Vec<Item>) {
        let selected = self.selected_locations();
        for item in items.iter_mut() {
            item.selected = false;
            if let Some(location) = &item.location_opt {
                if selected.contains(location) {
                    item.selected = true;
                }
            }
        }
        self.items_opt = Some(items);
    }

    pub fn selected_locations(&self) -> Vec<Location> {
        let mut locations = Vec::new();
        if let Some(ref items) = self.items_opt {
            for item in items.iter() {
                if item.selected {
                    if let Some(location) = &item.location_opt {
                        locations.push(location.clone());
                    }
                }
            }
        }
        locations
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
                item.selected = item.name == name;
            }
        }
    }

    pub fn select_path(&mut self, path: PathBuf) {
        let location = Location::Path(path);
        *self.cached_selected.borrow_mut() = None;
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                item.selected = item.location_opt.as_ref() == Some(&location);
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
            let size = self
                .item_view_size_opt
                .get()
                .unwrap_or_else(|| Size::new(0.0, 0.0));
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
        self.context_menu = None;
        self.edit_location = None;
        self.items_opt = None;
        //TODO: remember scroll by location?
        self.scroll_opt = None;
        self.select_focus = None;
        self.search_context = None;
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
        let mod_ctrl = modifiers.contains(Modifiers::CTRL) && self.mode.multiple();
        let mod_shift = modifiers.contains(Modifiers::SHIFT) && self.mode.multiple();
        match message {
            Message::AddNetworkDrive => {
                commands.push(Command::AddNetworkDrive);
            }
            Message::ClickRelease(click_i_opt) => {
                if click_i_opt == self.clicked.take() {
                    return commands;
                }
                self.context_menu = None;
                self.location_context_menu_index = None;
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
                    if let Some(location) = &clicked_item.location_opt {
                        if clicked_item.metadata.is_dir() {
                            cd = Some(location.clone());
                        } else {
                            if let Some(path) = location.path_opt() {
                                commands.push(Command::OpenFile(path.to_path_buf()));
                            } else {
                                log::warn!("no path for item {:?}", clicked_item);
                            }
                        }
                    } else {
                        log::warn!("no location for item {:?}", clicked_item);
                    }
                } else {
                    log::warn!("no item for click index {:?}", click_i_opt);
                }
            }
            Message::Click(click_i_opt) => {
                self.selected_clicked = false;
                self.context_menu = None;
                self.edit_location = None;
                self.location_context_menu_index = None;
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
                            let (sort_name, sort_direction, _) = self.sort_options();
                            //TODO: this assumes the default sort order!
                            if sort_name == HeadingOptions::Name && sort_direction {
                                // A default/unsorted tab's view is consistent with how the
                                // Items are laid out internally (items_opt), so Items can be
                                // linearly selected
                                if let Some(ref mut items) = self.items_opt {
                                    for item in items.iter_mut().skip(min).take(max - min + 1) {
                                        item.selected = true;
                                    }
                                }
                            } else {
                                // A sorted tab's items can't be linearly selected
                                // Let's say we have:
                                // index | file
                                // 0     | file0
                                // 1     | file1
                                // 2     | file2
                                // This is both the default sort and internal ordering
                                // When sorted it may be displayed as:
                                // 1     | file1
                                // 0     | file0
                                // 2     | file2
                                // However, the internal ordering is still the same thus
                                // linearly selecting items doesn't work. Shift selecting
                                // file0 and file2 would select indices 0 to 2 when it should
                                // select indices 0 AND 2 from items_opt
                                let indices: Vec<_> = self
                                    .column_sort()
                                    .map(|sorted| sorted.into_iter().map(|(i, _)| i).collect())
                                    .unwrap_or_else(|| {
                                        let len = self
                                            .items_opt
                                            .as_deref()
                                            .map(|items| items.len())
                                            .unwrap_or_default();
                                        (0..len).collect()
                                    });

                                // Find the true indices for the min and max element w.r.t.
                                // a sorted tab.
                                let min = indices
                                    .iter()
                                    .position(|&offset| offset == min)
                                    .unwrap_or_default();
                                // We can't skip `min_real` elements here because the index of
                                // `max` may actually be before `min` in a sorted tab
                                let max = indices
                                    .iter()
                                    .position(|&offset| offset == max)
                                    .unwrap_or_else(|| indices.len());
                                let min_real = min.min(max);
                                let max_real = max.max(min);

                                if let Some(ref mut items) = self.items_opt {
                                    for index in indices
                                        .into_iter()
                                        .skip(min_real)
                                        .take(max_real - min_real + 1)
                                    {
                                        if let Some(item) = items.get_mut(index) {
                                            item.selected = true;
                                        }
                                    }
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
                                if let Mode::Dialog(dialog) = &self.mode {
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
                                self.select_focus = click_i_opt;
                                self.selected_clicked = true;
                            } else if !dont_unset && item.selected {
                                self.clicked = click_i_opt;
                                item.selected = false;
                            }
                        }
                    }
                }
            }
            Message::Config(config) => {
                // View is preserved for existing tabs
                let view = self.config.view;
                self.config = config;
                self.config.view = view;
            }
            Message::ContextAction(action) => {
                // Close context menu
                self.context_menu = None;

                commands.push(Command::Action(action));
            }
            Message::ContextMenu(point_opt) => {
                self.edit_location = None;
                if point_opt.is_none() || !mod_shift {
                    self.context_menu = point_opt;
                    //TODO: hack for clearing selecting when right clicking empty space
                    if self.context_menu.is_some() && self.last_right_click.take().is_none() {
                        if let Some(ref mut items) = self.items_opt {
                            for item in items.iter_mut() {
                                item.selected = false;
                            }
                        }
                    }
                }
            }
            Message::LocationContextMenuPoint(point_opt) => {
                self.location_context_menu_point = point_opt;
            }
            Message::LocationContextMenuIndex(index_opt) => {
                self.location_context_menu_index = index_opt;
            }
            Message::LocationMenuAction(action) => {
                self.location_context_menu_index = None;
                let path_for_index = |ancestor_index| {
                    self.location
                        .path_opt()
                        .and_then(|path| path.ancestors().nth(ancestor_index))
                        .map(|path| path.to_path_buf())
                };
                match action {
                    LocationMenuAction::OpenInNewTab(ancestor_index) => {
                        if let Some(path) = path_for_index(ancestor_index) {
                            commands.push(Command::OpenInNewTab(path));
                        }
                    }
                    LocationMenuAction::OpenInNewWindow(ancestor_index) => {
                        if let Some(path) = path_for_index(ancestor_index) {
                            commands.push(Command::OpenInNewWindow(path));
                        }
                    }
                    LocationMenuAction::Preview(ancestor_index) => {
                        if let Some(path) = path_for_index(ancestor_index) {
                            //TODO: blocking code, run in command
                            match item_from_path(&path, IconSizes::default()) {
                                Ok(item) => {
                                    commands.push(Command::Preview(PreviewKind::Custom(
                                        PreviewItem(item),
                                    )));
                                }
                                Err(err) => {
                                    log::warn!("failed to get item from path {:?}: {}", path, err);
                                }
                            }
                        }
                    }
                    LocationMenuAction::AddToSidebar(ancestor_index) => {
                        if let Some(path) = path_for_index(ancestor_index) {
                            commands.push(Command::AddToSidebar(path));
                        } else {
                            log::warn!(
                                "no ancestor {ancestor_index} for location {:?}",
                                self.location
                            );
                        }
                    }
                }
            }
            Message::Drag(rect_opt) => match rect_opt {
                Some(rect) => {
                    self.context_menu = None;
                    self.location_context_menu_index = None;
                    self.select_rect(rect, mod_ctrl, mod_shift);
                    if self.select_focus.take().is_some() {
                        // Unfocus currently focused button
                        commands.push(Command::Iced(
                            widget::button::focus(widget::Id::unique()).into(),
                        ));
                    }
                }
                None => {}
            },
            Message::EditLocation(edit_location) => {
                if self.edit_location.is_none() && edit_location.is_some() {
                    commands.push(Command::Iced(
                        widget::text_input::focus(self.edit_location_id.clone()).into(),
                    ));
                }
                self.edit_location = edit_location;
            }
            Message::EditLocationEnable => {
                commands.push(Command::Iced(
                    widget::text_input::focus(self.edit_location_id.clone()).into(),
                ));
                self.edit_location = Some(self.location.clone());
            }
            Message::OpenInNewTab(path) => {
                commands.push(Command::OpenInNewTab(path));
            }
            Message::EmptyTrash => {
                commands.push(Command::EmptyTrash);
            }
            Message::Gallery(gallery) => {
                self.gallery = gallery;
            }
            Message::GalleryPrevious | Message::GalleryNext => {
                let mut pos_opt = None;
                if let Some(mut indices) = self.column_sort() {
                    if matches!(message, Message::GalleryPrevious) {
                        indices.reverse();
                    }
                    let mut found = false;
                    for (index, item) in indices {
                        if self.select_focus == None {
                            found = true;
                        }
                        if self.select_focus == Some(index) {
                            found = true;
                            continue;
                        }
                        if found {
                            if item.can_gallery() {
                                pos_opt = item.pos_opt.get();
                                if pos_opt.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }
                if let Some((row, col)) = pos_opt {
                    // Should mod_shift be available?
                    self.select_position(row, col, mod_shift);
                }
                if let Some(offset) = self.select_focus_scroll() {
                    commands.push(Command::Iced(
                        scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
                    ));
                }
                if let Some(id) = self.select_focus_id() {
                    commands.push(Command::Iced(widget::button::focus(id).into()));
                }
            }
            Message::GalleryToggle => {
                if let Some(indices) = self.column_sort() {
                    for (_, item) in indices.iter() {
                        if item.selected && item.can_gallery() {
                            self.gallery = !self.gallery;
                            break;
                        }
                    }
                }
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
                if self.gallery {
                    for command in self.update(Message::GalleryNext, modifiers) {
                        commands.push(command);
                    }
                } else {
                    if let Some((row, col)) =
                        self.select_focus_pos_opt().or(self.select_last_pos_opt())
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
                        commands.push(Command::Iced(
                            scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
                        ));
                    }
                    if let Some(id) = self.select_focus_id() {
                        commands.push(Command::Iced(widget::button::focus(id).into()));
                    }
                }
            }
            Message::ItemLeft => {
                if self.gallery {
                    for command in self.update(Message::GalleryPrevious, modifiers) {
                        commands.push(command);
                    }
                } else {
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
                        commands.push(Command::Iced(
                            scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
                        ));
                    }
                    if let Some(id) = self.select_focus_id() {
                        commands.push(Command::Iced(widget::button::focus(id).into()));
                    }
                }
            }
            Message::ItemRight => {
                if self.gallery {
                    for command in self.update(Message::GalleryNext, modifiers) {
                        commands.push(command);
                    }
                } else {
                    if let Some((row, col)) =
                        self.select_focus_pos_opt().or(self.select_last_pos_opt())
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
                        commands.push(Command::Iced(
                            scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
                        ));
                    }
                    if let Some(id) = self.select_focus_id() {
                        commands.push(Command::Iced(widget::button::focus(id).into()));
                    }
                }
            }
            Message::ItemUp => {
                if self.gallery {
                    for command in self.update(Message::GalleryPrevious, modifiers) {
                        commands.push(command);
                    }
                } else {
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
                        commands.push(Command::Iced(
                            scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
                        ));
                    }
                    if let Some(id) = self.select_focus_id() {
                        commands.push(Command::Iced(widget::button::focus(id).into()));
                    }
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
                    _ => {
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
            Message::Open(path_opt) => {
                match path_opt {
                    Some(path) => {
                        if path.is_dir() {
                            cd = Some(Location::Path(path));
                        } else {
                            commands.push(Command::OpenFile(path));
                        }
                    }
                    None => {
                        if let Some(ref mut items) = self.items_opt {
                            for item in items.iter() {
                                if item.selected {
                                    if let Some(location) = &item.location_opt {
                                        if item.metadata.is_dir() {
                                            //TODO: allow opening multiple tabs?
                                            cd = Some(location.clone());
                                        } else {
                                            if let Some(path) = location.path_opt() {
                                                commands
                                                    .push(Command::OpenFile(path.to_path_buf()));
                                            }
                                        }
                                    } else {
                                        //TODO: open properties?
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Message::RightClick(click_i_opt) => {
                self.update(Message::Click(click_i_opt), modifiers);
                *self.cached_selected.borrow_mut() = None;
                if let Some(ref mut items) = self.items_opt {
                    if !click_i_opt.map_or(false, |click_i| {
                        items.get(click_i).map_or(false, |x| x.selected)
                    }) {
                        // If item not selected, clear selection on other items
                        for (i, item) in items.iter_mut().enumerate() {
                            item.selected = Some(i) == click_i_opt;
                        }
                    }
                }
                //TODO: hack for clearing selecting when right clicking empty space
                self.last_right_click = click_i_opt;
            }
            Message::MiddleClick(click_i) => {
                self.update(Message::Click(Some(click_i)), modifiers);
                if !mod_ctrl && !mod_shift {
                    if let Some(ref mut items) = self.items_opt {
                        for (i, item) in items.iter_mut().enumerate() {
                            item.selected = i == click_i;
                        }
                        self.select_range = Some((click_i, click_i));
                    }
                    if let Some(clicked_item) =
                        self.items_opt.as_ref().and_then(|items| items.get(click_i))
                    {
                        if let Some(path) = clicked_item.path_opt() {
                            if clicked_item.metadata.is_dir() {
                                //cd = Some(Location::Path(path.clone()));
                                commands.push(Command::OpenInNewTab(path.clone()))
                            } else {
                                commands.push(Command::OpenFile(path.clone()));
                            }
                        } else {
                            log::warn!("no path for item {:?}", clicked_item);
                        }
                    } else {
                        log::warn!("no item for click index {:?}", click_i);
                    }
                }
            }
            Message::HighlightDeactivate(i) => {
                if let Some(item) = self.items_opt.as_mut().and_then(|f| f.get_mut(i)) {
                    item.highlighted = false;
                }
            }
            Message::HighlightActivate(i) => {
                if let Some(item) = self.items_opt.as_mut().and_then(|f| f.get_mut(i)) {
                    item.highlighted = true;
                }
            }

            Message::Scroll(viewport) => {
                self.scroll_opt = Some(viewport.absolute_offset());
            }
            Message::ScrollToFocus => {
                if let Some(offset) = self.select_focus_scroll() {
                    commands.push(Command::Iced(
                        scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
                    ));
                }
            }
            Message::SearchContext(location, context) => {
                if location == self.location {
                    self.search_context = context.0;
                } else {
                    log::warn!(
                        "search context provided for {:?} instead of {:?}",
                        location,
                        self.location
                    );
                }
            }
            Message::SearchReady(finished) => {
                if let Some(context) = &mut self.search_context {
                    if let Some(items) = &mut self.items_opt {
                        if finished || context.ready.swap(false, atomic::Ordering::SeqCst) {
                            let duration = Instant::now();
                            while let Some((path, name, metadata)) =
                                context.results_rx.blocking_recv()
                            {
                                //TODO: combine this with column_sort logic, they must match!
                                let item_modified = metadata.modified().ok();
                                let index = match items.binary_search_by(|other| {
                                    item_modified.cmp(&other.metadata.modified())
                                }) {
                                    Ok(index) => index,
                                    Err(index) => index,
                                };
                                if index < MAX_SEARCH_RESULTS {
                                    //TODO: use correct IconSizes
                                    items.insert(
                                        index,
                                        item_from_entry(path, name, metadata, IconSizes::default()),
                                    );
                                }
                                // Ensure that updates make it to the GUI in a timely manner
                                if !finished && duration.elapsed() >= MAX_SEARCH_LATENCY {
                                    break;
                                }
                            }
                        }
                        if items.len() >= MAX_SEARCH_RESULTS {
                            items.truncate(MAX_SEARCH_RESULTS);
                            if let Some(last_modified) =
                                items.last().and_then(|item| item.metadata.modified())
                            {
                                *context.last_modified_opt.write().unwrap() = Some(last_modified);
                            }
                        }
                    } else {
                        log::warn!("search ready but items array is empty");
                    }
                }
                if finished {
                    self.search_context = None;
                }
            }
            Message::SelectAll => {
                self.select_all();
                if self.select_focus.take().is_some() {
                    // Unfocus currently focused button
                    commands.push(Command::Iced(
                        widget::button::focus(widget::Id::unique()).into(),
                    ));
                }
            }
            Message::SetSort(heading_option, dir) => {
                if !matches!(self.location, Location::Search(..)) {
                    self.sort_name = heading_option;
                    self.sort_direction = dir;
                }
            }
            Message::Thumbnail(path, thumbnail) => {
                if let Some(ref mut items) = self.items_opt {
                    let location = Location::Path(path);
                    for item in items.iter_mut() {
                        if item.location_opt.as_ref() == Some(&location) {
                            let handle_opt = match &thumbnail {
                                ItemThumbnail::NotImage => None,
                                ItemThumbnail::Image(handle, _) => Some(widget::icon::Handle {
                                    symbolic: false,
                                    data: widget::icon::Data::Image(handle.clone()),
                                }),
                                ItemThumbnail::Svg(handle) => Some(widget::icon::Handle {
                                    symbolic: false,
                                    data: widget::icon::Data::Svg(handle.clone()),
                                }),
                                //TODO: text thumbnails?
                                ItemThumbnail::Text(_text) => None,
                            };
                            if let Some(handle) = handle_opt {
                                item.icon_handle_grid = handle.clone();
                                item.icon_handle_list = handle.clone();
                                item.icon_handle_list_condensed = handle;
                            }
                            item.thumbnail_opt = Some(thumbnail);
                            break;
                        }
                    }
                }
            }
            Message::ToggleShowHidden => {
                self.config.show_hidden = !self.config.show_hidden;
                if let Location::Search(path, term, ..) = &self.location {
                    cd = Some(Location::Search(
                        path.clone(),
                        term.clone(),
                        self.config.show_hidden,
                        Instant::now(),
                    ));
                }
            }
            Message::View(view) => {
                self.config.view = view;
            }
            Message::ToggleSort(heading_option) => {
                if !matches!(self.location, Location::Search(..)) {
                    let heading_sort = if self.sort_name == heading_option {
                        !self.sort_direction
                    } else {
                        // Default modified to descending, and others to ascending.
                        heading_option != HeadingOptions::Modified
                    };
                    self.sort_direction = heading_sort;
                    self.sort_name = heading_option;
                }
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
                    _ => {
                        log::warn!("{:?} to {:?} is not supported.", from.kind, to);
                    }
                };
            }
            Message::Drop(None) => {
                self.dnd_hovered = None;
            }
            Message::DndHover(loc) => {
                if self
                    .dnd_hovered
                    .as_ref()
                    .is_some_and(|(l, i)| *l == loc && i.elapsed() > HOVER_DURATION)
                {
                    cd = Some(loc);
                }
            }
            Message::DndEnter(loc) => {
                self.dnd_hovered = Some((loc.clone(), Instant::now()));
                if loc != self.location {
                    commands.push(Command::Iced(
                        cosmic::Task::perform(
                            async move {
                                tokio::time::sleep(HOVER_DURATION).await;
                                Message::DndHover(loc)
                            },
                            |x| x,
                        )
                        .into(),
                    ));
                }
            }
            Message::DndLeave(loc) => {
                if Some(&loc) == self.dnd_hovered.as_ref().map(|(l, _)| l) {
                    self.dnd_hovered = None;
                }
            }
            Message::WindowDrag => {
                commands.push(Command::WindowDrag);
            }
            Message::WindowToggleMaximize => {
                commands.push(Command::WindowToggleMaximize);
            }
            Message::ZoomIn => {
                commands.push(Command::Action(Action::ZoomIn));
            }
            Message::ZoomOut => {
                commands.push(Command::Action(Action::ZoomOut));
            }
        }

        // Scroll to top if needed
        if self.scroll_opt.is_none() {
            let offset = AbsoluteOffset { x: 0.0, y: 0.0 };
            self.scroll_opt = Some(offset);
            commands.push(Command::Iced(
                scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
            ));
        }

        // Change directory if requested
        if let Some(location) = cd {
            if matches!(self.mode, Mode::Desktop) {
                match location {
                    Location::Path(path) => {
                        commands.push(Command::OpenFile(path));
                    }
                    Location::Trash => {
                        commands.push(Command::OpenTrash);
                    }
                    _ => {}
                }
            } else if location != self.location {
                if location.path_opt().map_or(true, |path| path.is_dir()) {
                    let prev_path = if let Some(path) = self.location.path_opt() {
                        Some(path.to_path_buf())
                    } else {
                        None
                    };
                    self.change_location(&location, history_i_opt);
                    commands.push(Command::ChangeLocation(self.title(), location, prev_path));
                } else {
                    log::warn!("tried to cd to {:?} which is not a directory", location);
                }
            }
        }

        commands
    }

    pub(crate) fn sort_options(&self) -> (HeadingOptions, bool, bool) {
        match self.location {
            Location::Search(..) => (HeadingOptions::Modified, false, false),
            _ => (
                self.sort_name,
                self.sort_direction,
                self.config.folders_first,
            ),
        }
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
        let (sort_name, sort_direction, folders_first) = self.sort_options();
        match sort_name {
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
                        ItemMetadata::SimpleDir { entries } => (true, *entries),
                        ItemMetadata::SimpleFile { size } => (false, *size),
                    };
                    let (a_is_entry, a_size) = get_size(a.1);
                    let (b_is_entry, b_size) = get_size(b.1);

                    //TODO: use folders_first?
                    match (a_is_entry, b_is_entry) {
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                        _ => check_reverse(a_size.cmp(&b_size), sort_direction),
                    }
                })
            }
            HeadingOptions::Name => items.sort_by(|a, b| {
                if folders_first {
                    match (a.1.metadata.is_dir(), b.1.metadata.is_dir()) {
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                        _ => check_reverse(
                            LANGUAGE_SORTER.compare(&a.1.display_name, &b.1.display_name),
                            sort_direction,
                        ),
                    }
                } else {
                    check_reverse(
                        LANGUAGE_SORTER.compare(&a.1.display_name, &b.1.display_name),
                        sort_direction,
                    )
                }
            }),
            HeadingOptions::Modified => {
                items.sort_by(|a, b| {
                    let a_modified = a.1.metadata.modified();
                    let b_modified = b.1.metadata.modified();
                    if folders_first {
                        match (a.1.metadata.is_dir(), b.1.metadata.is_dir()) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => check_reverse(a_modified.cmp(&b_modified), sort_direction),
                        }
                    } else {
                        check_reverse(a_modified.cmp(&b_modified), sort_direction)
                    }
                });
            }
            HeadingOptions::TrashedOn => {
                let time_deleted = |x: &Item| match &x.metadata {
                    ItemMetadata::Trash { entry, .. } => Some(entry.time_deleted),
                    _ => None,
                };

                items.sort_by(|a, b| {
                    let a_time_deleted = time_deleted(a.1);
                    let b_time_deleted = time_deleted(b.1);
                    if folders_first {
                        match (a.1.metadata.is_dir(), b.1.metadata.is_dir()) {
                            (true, false) => Ordering::Less,
                            (false, true) => Ordering::Greater,
                            _ => check_reverse(a_time_deleted.cmp(&b_time_deleted), sort_direction),
                        }
                    } else {
                        check_reverse(b_time_deleted.cmp(&a_time_deleted), sort_direction)
                    }
                });
            }
        }
        Some(items)
    }

    fn dnd_dest<'a>(
        &self,
        location: &Location,
        element: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let location1 = location.clone();
        let location2 = location.clone();
        let location3 = location.clone();
        let is_dnd_hovered = self.dnd_hovered.as_ref().map(|(l, _)| l) == Some(&location);
        let mut container = widget::container(
            DndDestination::for_data::<ClipboardPaste>(element, move |data, action| {
                if let Some(mut data) = data {
                    if action == DndAction::Copy {
                        Message::Drop(Some((location1.clone(), data)))
                    } else if action == DndAction::Move {
                        data.kind = ClipboardKind::Cut;
                        Message::Drop(Some((location1.clone(), data)))
                    } else {
                        log::warn!("unsupported action: {:?}", action);
                        Message::Drop(None)
                    }
                } else {
                    Message::Drop(None)
                }
            })
            .on_enter(move |_, _, _| Message::DndEnter(location2.clone()))
            .on_leave(move || Message::DndLeave(location3.clone())),
        );
        if is_dnd_hovered {
            container = container.style(|t| {
                let mut a = widget::container::Style::default();
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
            });
        }
        container.into()
    }

    pub fn gallery_view(&self) -> Element<Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            space_m,
            ..
        } = theme::active().cosmic().spacing;

        //TODO: display error messages when image not found?
        let mut name_opt = None;
        let mut element_opt: Option<Element<Message>> = None;
        if let Some(index) = self.select_focus {
            if let Some(items) = &self.items_opt {
                if let Some(item) = items.get(index) {
                    name_opt = Some(widget::text::heading(&item.display_name));
                    match item
                        .thumbnail_opt
                        .as_ref()
                        .unwrap_or(&ItemThumbnail::NotImage)
                    {
                        ItemThumbnail::NotImage => {}
                        ItemThumbnail::Image(handle, _) => {
                            if let Some(path) = item.path_opt() {
                                element_opt = Some(
                                    widget::container(
                                        //TODO: use widget::image::viewer, when its zoom can be reset
                                        widget::image(widget::image::Handle::from_path(path)),
                                    )
                                    .center(Length::Fill)
                                    .into(),
                                );
                            } else {
                                element_opt = Some(
                                    widget::container(
                                        //TODO: use widget::image::viewer, when its zoom can be reset
                                        widget::image(handle.clone()),
                                    )
                                    .center(Length::Fill)
                                    .into(),
                                );
                            }
                        }
                        ItemThumbnail::Svg(handle) => {
                            element_opt = Some(
                                widget::svg(handle.clone())
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .into(),
                            );
                        }
                        ItemThumbnail::Text(text) => {
                            element_opt = Some(widget::text_editor(text).into())
                        }
                    }
                }
            }
        }

        let mut column = widget::column::with_capacity(2);
        column = column.push(widget::Space::with_height(Length::Fixed(space_m.into())));
        {
            let mut row = widget::row::with_capacity(5).align_y(Alignment::Center);
            row = row.push(widget::horizontal_space());
            if let Some(name) = name_opt {
                row = row.push(name);
            }
            row = row.push(widget::horizontal_space());
            row = row.push(
                widget::button::icon(widget::icon::from_name("window-close-symbolic"))
                    .class(theme::Button::Standard)
                    .on_press(Message::Gallery(false)),
            );
            row = row.push(widget::Space::with_width(Length::Fixed(space_m.into())));
            // This mouse area provides window drag while the header bar is hidden
            let mouse_area = mouse_area::MouseArea::new(row)
                .on_drag(|_| Message::WindowDrag)
                .on_double_click(|_| Message::WindowToggleMaximize);
            column = column.push(mouse_area);
        }
        {
            let mut row = widget::row::with_capacity(7).align_y(Alignment::Center);
            row = row.push(widget::Space::with_width(Length::Fixed(space_m.into())));
            row = row.push(
                widget::button::icon(widget::icon::from_name("go-previous-symbolic"))
                    .padding(space_xs)
                    .class(theme::Button::Standard)
                    .on_press(Message::GalleryPrevious),
            );
            row = row.push(widget::Space::with_width(Length::Fixed(space_xxs.into())));
            if let Some(element) = element_opt {
                row = row.push(element);
            } else {
                //TODO: what to do when no image?
                row = row.push(widget::Space::new(Length::Fill, Length::Fill));
            }
            row = row.push(widget::Space::with_width(Length::Fixed(space_xxs.into())));
            row = row.push(
                widget::button::icon(widget::icon::from_name("go-next-symbolic"))
                    .padding(space_xs)
                    .class(theme::Button::Standard)
                    .on_press(Message::GalleryNext),
            );
            row = row.push(widget::Space::with_width(Length::Fixed(space_m.into())));
            column = column.push(row);
        }

        widget::container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme| {
                let cosmic = theme.cosmic();
                let mut bg = cosmic.bg_color();
                bg.alpha = 0.75;
                widget::container::Style {
                    background: Some(Color::from(bg).into()),
                    ..Default::default()
                }
            })
            .into()
    }

    pub fn location_view(&self) -> Element<Message> {
        //TODO: responsiveness is done in a hacky way, potentially move this to a custom widget?
        fn text_width<'a>(
            content: &'a str,
            font: font::Font,
            font_size: f32,
            line_height: f32,
        ) -> f32 {
            let text: text::Text<&'a str, font::Font> = text::Text {
                content,
                bounds: Size::INFINITY,
                size: font_size.into(),
                line_height: text::LineHeight::Absolute(line_height.into()),
                font,
                horizontal_alignment: Horizontal::Left,
                vertical_alignment: Vertical::Top,
                shaping: text::Shaping::default(),
                wrapping: text::Wrapping::None,
            };
            graphics::text::Paragraph::with_text(text)
                .min_bounds()
                .width
        }
        fn text_width_body<'a>(content: &'a str) -> f32 {
            //TODO: should libcosmic set the font when using widget::text::body?
            text_width(content, font::default(), 14.0, 20.0)
        }
        fn text_width_heading<'a>(content: &'a str) -> f32 {
            text_width(content, font::semibold(), 14.0, 20.0)
        }

        let cosmic_theme::Spacing {
            space_xxxs,
            space_xxs,
            space_s,
            space_m,
            ..
        } = theme::active().cosmic().spacing;

        let size = self.size_opt.get().unwrap_or(Size::new(0.0, 0.0));

        let mut row = widget::row::with_capacity(5)
            .align_y(Alignment::Center)
            .padding([space_xxxs, 0]);
        let mut w = 0.0;

        let mut prev_button =
            widget::button::custom(widget::icon::from_name("go-previous-symbolic").size(16))
                .padding(space_xxs)
                .class(theme::Button::Icon);
        if self.history_i > 0 && !self.history.is_empty() {
            prev_button = prev_button.on_press(Message::GoPrevious);
        }
        row = row.push(prev_button);
        w += 16.0 + 2.0 * space_xxs as f32;

        let mut next_button =
            widget::button::custom(widget::icon::from_name("go-next-symbolic").size(16))
                .padding(space_xxs)
                .class(theme::Button::Icon);
        if self.history_i + 1 < self.history.len() {
            next_button = next_button.on_press(Message::GoNext);
        }
        row = row.push(next_button);
        w += 16.0 + 2.0 * space_xxs as f32;

        row = row.push(widget::Space::with_width(Length::Fixed(space_s.into())));
        w += space_s as f32;

        //TODO: allow resizing?
        let name_width = 300.0;
        let modified_width = 200.0;
        let size_width = 100.0;
        let condensed = size.width < (name_width + modified_width + size_width);

        let (sort_name, sort_direction, _) = self.sort_options();
        let heading_item = |name, width, msg| {
            let mut row = widget::row::with_capacity(2)
                .align_y(Alignment::Center)
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
            return mouse_area::MouseArea::new(row)
                .on_press(move |_point_opt| Message::ToggleSort(msg))
                .into();
        };

        let heading_row = widget::row::with_children(vec![
            heading_item(fl!("name"), Length::Fill, HeadingOptions::Name),
            if self.location == Location::Trash {
                heading_item(
                    fl!("trashed-on"),
                    Length::Fixed(modified_width),
                    HeadingOptions::TrashedOn,
                )
            } else {
                heading_item(
                    fl!("modified"),
                    Length::Fixed(modified_width),
                    HeadingOptions::Modified,
                )
            },
            heading_item(fl!("size"), Length::Fixed(size_width), HeadingOptions::Size),
        ])
        .align_y(Alignment::Center)
        .height(Length::Fixed((space_m + 4).into()))
        .padding([0, space_xxs])
        .spacing(space_xxs);

        if let Some(location) = &self.edit_location {
            //TODO: allow editing other locations
            if let Some(path) = location.path_opt() {
                row = row.push(
                    widget::button::custom(
                        widget::icon::from_name("window-close-symbolic").size(16),
                    )
                    .on_press(Message::EditLocation(None))
                    .padding(space_xxs)
                    .class(theme::Button::Icon),
                );
                row = row.push(
                    widget::text_input("", path.to_string_lossy())
                        .id(self.edit_location_id.clone())
                        .on_input(|input| {
                            Message::EditLocation(Some(location.with_path(PathBuf::from(input))))
                        })
                        .on_submit(Message::Location(location.clone()))
                        .line_height(1.0),
                );
                let mut column = widget::column::with_capacity(4).padding([0, space_s]);
                column = column.push(row);
                column = column.push(horizontal_rule(1).class(theme::Rule::Custom(Box::new(
                    |theme| rule::Style {
                        color: theme.cosmic().accent_color().into(),
                        width: 1,
                        radius: 0.0.into(),
                        fill_mode: rule::FillMode::Full,
                    },
                ))));
                if self.config.view == View::List && !condensed {
                    column = column.push(heading_row);
                    column = column.push(widget::divider::horizontal::default());
                }
                return column.into();
            }
        } else if let Some(path) = self.location.path_opt() {
            row = row.push(
                crate::mouse_area::MouseArea::new(
                    widget::button::custom(widget::icon::from_name("edit-symbolic").size(16))
                        .padding(space_xxs)
                        .class(theme::Button::Icon)
                        .on_press(Message::EditLocation(Some(self.location.clone()))),
                )
                .on_middle_press(move |_| Message::OpenInNewTab(path.clone())),
            );
            w += 16.0 + 2.0 * space_xxs as f32;
        }

        let mut children: Vec<Element<_>> = Vec::new();
        match &self.location {
            Location::Desktop(path, ..) | Location::Path(path) | Location::Search(path, ..) => {
                let excess_str = "...";
                let excess_width = text_width_body(excess_str);
                for (index, ancestor) in path.ancestors().enumerate() {
                    let (name, found_home) = folder_name(&ancestor);
                    let (name_width, name_text) = if children.is_empty() {
                        (
                            text_width_heading(&name),
                            widget::text::heading(name).wrapping(text::Wrapping::None),
                        )
                    } else {
                        children.push(
                            widget::icon::from_name("go-next-symbolic")
                                .size(16)
                                .icon()
                                .into(),
                        );
                        w += 16.0;
                        (
                            text_width_body(&name),
                            widget::text::body(name).wrapping(text::Wrapping::None),
                        )
                    };

                    // Add padding for mouse area
                    w += 2.0 * space_xxxs as f32;

                    let mut row = widget::row::with_capacity(2)
                        .align_y(Alignment::Center)
                        .spacing(space_xxxs);
                    //TODO: figure out why this hardcoded offset is needed after the first item is ellipsed
                    let overflow_offset = 64.0;
                    let overflow = w + name_width + overflow_offset > size.width && index > 0;
                    if overflow {
                        row = row.push(widget::text::body(excess_str));
                        w += excess_width;
                    } else {
                        row = row.push(name_text);
                        w += name_width;
                    }

                    let location = self.location.with_path(ancestor.to_path_buf());
                    let mut mouse_area = crate::mouse_area::MouseArea::new(
                        widget::button::custom(row)
                            .padding(space_xxxs)
                            .class(theme::Button::Link)
                            .on_press(Message::Location(location.clone())),
                    );

                    if self.location_context_menu_index.is_some() {
                        mouse_area = mouse_area.on_right_press(move |_point_opt| {
                            Message::LocationContextMenuIndex(None)
                        })
                    } else {
                        mouse_area = mouse_area.on_right_press_no_capture(move |_point_opt| {
                            Message::LocationContextMenuIndex(Some(index))
                        })
                    }

                    let mouse_area = if let Location::Path(_) = &self.location {
                        mouse_area
                            .on_middle_press(move |_| Message::OpenInNewTab(ancestor.to_path_buf()))
                    } else {
                        mouse_area
                    };

                    children.push(self.dnd_dest(&location, mouse_area));

                    if found_home || overflow {
                        break;
                    }
                }
                children.reverse();
            }
            Location::Trash => {
                children.push(
                    widget::button::custom(widget::text::heading(fl!("trash")))
                        .padding(space_xxxs)
                        .on_press(Message::Location(Location::Trash))
                        .class(theme::Button::Text)
                        .into(),
                );
            }
            Location::Recents => {
                children.push(
                    widget::button::custom(widget::text::heading(fl!("recents")))
                        .padding(space_xxxs)
                        .on_press(Message::Location(Location::Recents))
                        .class(theme::Button::Text)
                        .into(),
                );
            }
            Location::Network(uri, display_name) => {
                children.push(
                    widget::button::custom(widget::text::heading(display_name))
                        .padding(space_xxxs)
                        .on_press(Message::Location(Location::Network(
                            uri.clone(),
                            display_name.clone(),
                        )))
                        .class(theme::Button::Text)
                        .into(),
                );
            }
        }

        for child in children {
            row = row.push(child);
        }
        let mut column = widget::column::with_capacity(4).padding([0, space_s]);
        column = column.push(row);
        column = column.push(
            horizontal_rule(1).class(theme::Rule::Custom(Box::new(|theme| rule::Style {
                color: theme.cosmic().accent_color().into(),
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            }))),
        );

        if self.config.view == View::List && !condensed {
            column = column.push(heading_row);
            column = column.push(widget::divider::horizontal::default());
        }

        let mouse_area = crate::mouse_area::MouseArea::new(column)
            .on_right_press(Message::LocationContextMenuPoint);

        let mut popover = widget::popover(mouse_area);
        if let (Some(point), Some(index)) = (
            self.location_context_menu_point,
            self.location_context_menu_index,
        ) {
            popover = popover
                .popup(menu::location_context_menu(index))
                .position(widget::popover::Position::Point(point))
        }

        popover.into()
    }

    pub fn empty_view(&self, has_hidden: bool) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        //TODO: left clicking on an empty folder does not clear context menu
        widget::column::with_children(vec![widget::container(
            widget::column::with_children(match self.mode {
                Mode::App | Mode::Dialog(_) => vec![
                    widget::icon::from_name("folder-symbolic")
                        .size(64)
                        .icon()
                        .into(),
                    widget::text(if has_hidden {
                        fl!("empty-folder-hidden")
                    } else if matches!(self.location, Location::Search(..)) {
                        fl!("no-results")
                    } else {
                        fl!("empty-folder")
                    })
                    .into(),
                ],
                Mode::Desktop => Vec::new(),
            })
            .align_x(Alignment::Center)
            .spacing(space_xxs),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()])
        .into()
    }

    pub fn grid_view(&self) -> (Option<Element<'static, Message>>, Element<Message>, bool) {
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

        let text_height = 3 * 20; // 3 lines of text
        let item_width = (3 * space_xxs + icon_sizes.grid() + 3 * space_xxs) as usize;
        let item_height =
            (space_xxxs + icon_sizes.grid() + space_xxxs + text_height + space_xxxs) as usize;

        let (width, height) = match self.size_opt.get() {
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

        let rows = {
            let height_m1 = height.checked_sub(item_height).unwrap_or(0);
            let rows_m1 = height_m1 / (item_height + space_xxs as usize);
            rows_m1 + 1
        };

        let mut grid = widget::grid()
            .column_spacing(column_spacing)
            .row_spacing(space_xxs)
            .padding(space_xxs.into());
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
            let mut page_row = 0;
            let mut hidden = 0;
            let mut grid_elements = Vec::new();
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
                let buttons: Vec<Element<Message>> = vec![
                    widget::button::custom(
                        widget::icon::icon(item.icon_handle_grid.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_sizes.grid()),
                    )
                    .padding(space_xxxs)
                    .class(button_style(
                        item.selected,
                        item.highlighted,
                        false,
                        false,
                        false,
                    ))
                    .into(),
                    widget::tooltip(
                        widget::button::custom(widget::text::body(&item.display_name))
                            .id(item.button_id.clone())
                            .padding([0, space_xxxs])
                            .class(button_style(
                                item.selected,
                                item.highlighted,
                                true,
                                true,
                                matches!(self.mode, Mode::Desktop),
                            )),
                        widget::text::body(&item.name),
                        widget::tooltip::Position::Bottom,
                    )
                    .into(),
                ];

                let mut column = widget::column::with_capacity(buttons.len())
                    .align_x(Alignment::Center)
                    .height(Length::Fixed(item_height as f32))
                    .width(Length::Fixed(item_width as f32));
                for button in buttons {
                    if self.context_menu.is_some() {
                        column = column.push(button)
                    } else {
                        column = column.push(
                            mouse_area::MouseArea::new(button).on_right_press_no_capture(
                                move |_point_opt| Message::RightClick(Some(i)),
                            ),
                        );
                    }
                }

                let column: Element<Message> =
                    if item.metadata.is_dir() && item.location_opt.is_some() {
                        self.dnd_dest(&item.location_opt.clone().unwrap(), column)
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
                    .on_release(move |_| Message::ClickRelease(Some(i)))
                    .on_middle_press(move |_| Message::MiddleClick(i))
                    .on_enter(move || Message::HighlightActivate(i))
                    .on_exit(move || Message::HighlightDeactivate(i));

                //TODO: error if the row or col is already set?
                while grid_elements.len() <= row {
                    grid_elements.push(Vec::new());
                }
                grid_elements[row].push(mouse_area);

                count += 1;
                if matches!(self.mode, Mode::Desktop) {
                    row += 1;
                    if row >= page_row + rows {
                        row = 0;
                        col += 1;
                    }
                    if col >= cols {
                        col = 0;
                        page_row += rows;
                        row = page_row;
                    }
                } else {
                    col += 1;
                    if col >= cols {
                        col = 0;
                        row += 1;
                    }
                }
            }

            for row_elements in grid_elements {
                for element in row_elements {
                    grid = grid.push(element);
                }
                grid = grid.insert_row();
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

                let top_deduct = 7 * (space_xxs as usize);

                self.item_view_size_opt
                    .set(self.size_opt.get().map(|s| Size {
                        width: s.width,
                        height: s.height - top_deduct as f32,
                    }));

                let spacer_height = height.checked_sub(max_bottom + top_deduct).unwrap_or(0);
                if spacer_height > 0 {
                    children.push(
                        widget::container(Space::with_height(Length::Fixed(spacer_height as f32)))
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
                    .padding(space_xxs.into());

                let mut dnd_item_i = 0;
                for r in drag_n_i..=drag_s_i {
                    dnd_grid = dnd_grid.insert_row();
                    for c in drag_w_i..=drag_e_i {
                        let Some((i, (row, col), item)) = dnd_items.get(dnd_item_i) else {
                            break;
                        };
                        if *row == r && *col == c {
                            let buttons = vec![
                                widget::button::custom(
                                    widget::icon::icon(item.icon_handle_grid.clone())
                                        .content_fit(ContentFit::Contain)
                                        .size(icon_sizes.grid()),
                                )
                                .on_press(Message::Click(Some(*i)))
                                .padding(space_xxxs)
                                .class(button_style(
                                    item.selected,
                                    item.highlighted,
                                    false,
                                    false,
                                    false,
                                )),
                                widget::button::custom(widget::text(item.display_name.clone()))
                                    .id(item.button_id.clone())
                                    .on_press(Message::Click(Some(*i)))
                                    .padding([0, space_xxxs])
                                    .class(button_style(
                                        item.selected,
                                        item.highlighted,
                                        true,
                                        true,
                                        false,
                                    )),
                            ];

                            let mut column = widget::column::with_capacity(buttons.len())
                                .align_x(Alignment::Center)
                                .height(Length::Fixed(item_height as f32))
                                .width(Length::Fixed(item_width as f32));
                            for button in buttons {
                                column = column.push(button)
                            }

                            dnd_grid = dnd_grid.push(column);
                            dnd_item_i += 1;
                        } else {
                            dnd_grid = dnd_grid.push(
                                widget::container(Space::with_height(item_width as f32))
                                    .height(Length::Fixed(item_height as f32)),
                            );
                        }
                    }
                }
                Element::from(dnd_grid)
            }),
            mouse_area::MouseArea::new(
                widget::container(widget::column::with_children(children)).width(Length::Fill),
            )
            .on_press(|_| Message::Click(None))
            .on_drag(Message::Drag)
            .on_drag_end(|_| Message::DragEnd(None))
            .show_drag_rect(true)
            .on_release(|_| Message::ClickRelease(None))
            .into(),
            true,
        )
    }

    pub fn list_view(&self) -> (Option<Element<'static, Message>>, Element<Message>, bool) {
        let cosmic_theme::Spacing {
            space_m,
            space_s,
            space_xxs,
            space_xxxs,
            ..
        } = theme::active().cosmic().spacing;

        let TabConfig {
            show_hidden,
            icon_sizes,
            ..
        } = self.config;

        let size = self.size_opt.get().unwrap_or_else(|| Size::new(0.0, 0.0));
        //TODO: allow resizing?
        let name_width = 300.0;
        let modified_width = 200.0;
        let size_width = 100.0;
        let condensed = size.width < (name_width + modified_width + size_width);
        let is_search = matches!(self.location, Location::Search(..));
        let icon_size = if condensed || is_search {
            icon_sizes.list_condensed()
        } else {
            icon_sizes.list()
        };
        let row_height = icon_size + 2 * space_xxs;

        let mut children: Vec<Element<_>> = Vec::new();
        let mut y = 0;

        let items = self.column_sort();
        let mut drag_items = Vec::new();
        if let Some(items) = items {
            let mut count = 0;
            let mut hidden = 0;
            for (i, item) in items {
                if item.hidden && !show_hidden {
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
                    children.push(
                        container(horizontal_rule(1))
                            .padding([0, space_xxxs])
                            .into(),
                    );
                    y += 1;
                }

                let modified_text = match &item.metadata {
                    ItemMetadata::Path { metadata, .. } => match metadata.modified() {
                        Ok(time) => format_time(time).to_string(),
                        Err(_) => String::new(),
                    },
                    ItemMetadata::Trash { entry, .. } => FormatTime::from_secs(entry.time_deleted)
                        .map(|t| t.to_string())
                        .unwrap_or_default(),
                    _ => String::new(),
                };

                let size_text = match &item.metadata {
                    ItemMetadata::Path { metadata, children } => {
                        if metadata.is_dir() {
                            //TODO: translate
                            if *children == 1 {
                                format!("{} item", children)
                            } else {
                                format!("{} items", children)
                            }
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
                    ItemMetadata::SimpleDir { entries } => {
                        //TODO: translate
                        if *entries == 1 {
                            format!("{} item", entries)
                        } else {
                            format!("{} items", entries)
                        }
                    }
                    ItemMetadata::SimpleFile { size } => format_size(*size),
                };

                let row = if condensed {
                    widget::row::with_children(vec![
                        widget::icon::icon(item.icon_handle_list_condensed.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_size)
                            .into(),
                        widget::column::with_children(vec![
                            widget::text(item.display_name.clone()).into(),
                            //TODO: translate?
                            widget::text::caption(format!("{} - {}", modified_text, size_text))
                                .into(),
                        ])
                        .into(),
                    ])
                    .height(Length::Fixed(row_height as f32))
                    .align_y(Alignment::Center)
                    .spacing(space_xxs)
                } else if is_search {
                    widget::row::with_children(vec![
                        widget::icon::icon(item.icon_handle_list_condensed.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_size)
                            .into(),
                        widget::column::with_children(vec![
                            widget::text(item.display_name.clone()).into(),
                            widget::text::caption(match item.path_opt() {
                                Some(path) => path.display().to_string(),
                                None => String::new(),
                            })
                            .into(),
                        ])
                        .width(Length::Fill)
                        .into(),
                        widget::text(modified_text.clone())
                            .width(Length::Fixed(modified_width))
                            .into(),
                        widget::text(size_text.clone())
                            .width(Length::Fixed(size_width))
                            .into(),
                    ])
                    .height(Length::Fixed(row_height as f32))
                    .align_y(Alignment::Center)
                    .spacing(space_xxs)
                } else {
                    widget::row::with_children(vec![
                        widget::icon::icon(item.icon_handle_list.clone())
                            .content_fit(ContentFit::Contain)
                            .size(icon_size)
                            .into(),
                        widget::text(item.display_name.clone())
                            .width(Length::Fill)
                            .into(),
                        widget::text(modified_text.clone())
                            .width(Length::Fixed(modified_width))
                            .into(),
                        widget::text(size_text.clone())
                            .width(Length::Fixed(size_width))
                            .into(),
                    ])
                    .height(Length::Fixed(row_height as f32))
                    .align_y(Alignment::Center)
                    .spacing(space_xxs)
                };

                let button = |row| {
                    let mouse_area = crate::mouse_area::MouseArea::new(
                        widget::button::custom(row)
                            .width(Length::Fill)
                            .id(item.button_id.clone())
                            .padding([0, space_xxs])
                            .class(button_style(
                                item.selected,
                                item.highlighted,
                                true,
                                false,
                                false,
                            )),
                    )
                    .on_press(move |_| Message::Click(Some(i)))
                    .on_double_click(move |_| Message::DoubleClick(Some(i)))
                    .on_release(move |_| Message::ClickRelease(Some(i)))
                    .on_middle_press(move |_| Message::MiddleClick(i))
                    .on_enter(move || Message::HighlightActivate(i))
                    .on_exit(move || Message::HighlightDeactivate(i));

                    if self.context_menu.is_some() {
                        mouse_area
                    } else {
                        mouse_area.on_right_press_no_capture(move |_point_opt| {
                            Message::RightClick(Some(i))
                        })
                    }
                };

                let button_row = button(row.into());
                let button_row: Element<_> =
                    if item.metadata.is_dir() && item.location_opt.is_some() {
                        self.dnd_dest(item.location_opt.as_ref().unwrap(), button_row)
                    } else {
                        button_row.into()
                    };

                if item.selected || !drag_items.is_empty() {
                    let dnd_row = if !item.selected {
                        Element::from(Space::with_height(Length::Fixed(row_height as f32)))
                    } else if condensed {
                        widget::row::with_children(vec![
                            widget::icon::icon(item.icon_handle_list_condensed.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::column::with_children(vec![
                                widget::text(item.display_name.clone()).into(),
                                //TODO: translate?
                                widget::text(format!("{} - {}", modified_text, size_text)).into(),
                            ])
                            .into(),
                        ])
                        .align_y(Alignment::Center)
                        .spacing(space_xxs)
                        .into()
                    } else if is_search {
                        widget::row::with_children(vec![
                            widget::icon::icon(item.icon_handle_list_condensed.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::column::with_children(vec![
                                widget::text(item.display_name.clone()).into(),
                                widget::text::caption(match item.path_opt() {
                                    Some(path) => path.display().to_string(),
                                    None => String::new(),
                                })
                                .into(),
                            ])
                            .width(Length::Fill)
                            .into(),
                            widget::text(modified_text.clone())
                                .width(Length::Fixed(modified_width))
                                .into(),
                            widget::text(size_text.clone())
                                .width(Length::Fixed(size_width))
                                .into(),
                        ])
                        .align_y(Alignment::Center)
                        .spacing(space_xxs)
                        .into()
                    } else {
                        widget::row::with_children(vec![
                            widget::icon::icon(item.icon_handle_list.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::text(item.display_name.clone())
                                .width(Length::Fill)
                                .into(),
                            widget::text(modified_text)
                                .width(Length::Fixed(modified_width))
                                .into(),
                            widget::text(size_text)
                                .width(Length::Fixed(size_width))
                                .into(),
                        ])
                        .align_y(Alignment::Center)
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
            let top_deduct = (if condensed || is_search { 6 } else { 9 }) * space_xxs;

            self.item_view_size_opt
                .set(self.size_opt.get().map(|s| Size {
                    width: s.width,
                    height: s.height - top_deduct as f32,
                }));

            let spacer_height = size.height - y as f32 - top_deduct as f32;
            if spacer_height > 0. {
                children.push(
                    widget::container(Space::with_height(Length::Fixed(spacer_height))).into(),
                );
            }
        }
        let drag_col = (!drag_items.is_empty())
            .then(|| Element::from(widget::column::with_children(drag_items)));

        (
            drag_col,
            mouse_area::MouseArea::new(
                widget::column::with_children(children).padding([0, space_s]),
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

    pub fn view_responsive(
        &self,
        key_binds: &HashMap<KeyBind, Action>,
        size: Size,
    ) -> Element<Message> {
        // Update cached size
        self.size_opt.set(Some(size));

        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            ..
        } = theme::active().cosmic().spacing;

        let location_view_opt = if matches!(self.mode, Mode::Desktop) {
            None
        } else {
            Some(self.location_view())
        };
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
                    .filter_map(|item| item.path_opt().map(|x| x.clone()))
                    .collect::<Vec<PathBuf>>()
            })
            .unwrap_or_default();
        let item_view =
            DndSource::<Message, ClipboardCopy>::with_id(item_view, Id::new("tab-view"));

        let item_view = match drag_list {
            Some(drag_list) if self.selected_clicked => {
                let drag_list = ArcElementWrapper::<Message>(Arc::new(Mutex::new(drag_list)));
                item_view
                    .drag_content(move || {
                        ClipboardCopy::new(crate::clipboard::ClipboardKind::Copy, &files)
                    })
                    .drag_icon(move || {
                        let state: tree::State = Widget::<Message, _, _>::state(&drag_list);
                        (Element::from(drag_list.clone()).map(|_m| ()), state)
                    })
            }
            _ => item_view,
        };

        let tab_location = self.location.clone();
        let mut mouse_area = mouse_area::MouseArea::new(item_view)
            .on_press(move |_point_opt| Message::Click(None))
            .on_release(|_| Message::ClickRelease(None))
            //TODO: better way to keep focused item in view
            .on_resize(|_| Message::ScrollToFocus)
            .on_back_press(move |_point_opt| Message::GoPrevious)
            .on_forward_press(move |_point_opt| Message::GoNext)
            .on_scroll(respond_to_scroll_direction);

        if self.context_menu.is_some() {
            mouse_area = mouse_area.on_right_press(move |_point_opt| Message::ContextMenu(None));
        } else {
            mouse_area = mouse_area.on_right_press(Message::ContextMenu);
        }

        let mut popover = widget::popover(mouse_area);

        if let Some(point) = self.context_menu {
            popover = popover
                .popup(menu::context_menu(&self, &key_binds))
                .position(widget::popover::Position::Point(point));
        }
        let mut tab_column = widget::column::with_capacity(3);
        if let Some(location_view) = location_view_opt {
            tab_column = tab_column.push(location_view);
        }
        if can_scroll {
            tab_column = tab_column.push(
                widget::scrollable(popover)
                    .id(self.scrollable_id.clone())
                    .on_scroll(Message::Scroll)
                    .width(Length::Fill)
                    .height(Length::Fill),
            );
        } else {
            tab_column = tab_column.push(popover);
        }
        match &self.location {
            Location::Trash => {
                if let Some(items) = self.items_opt() {
                    if !items.is_empty() {
                        tab_column = tab_column.push(
                            widget::layer_container(widget::row::with_children(vec![
                                widget::horizontal_space().into(),
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
            Location::Network(uri, _display_name) if uri == "network:///" => {
                tab_column = tab_column.push(
                    widget::layer_container(widget::row::with_children(vec![
                        widget::horizontal_space().into(),
                        widget::button::standard(fl!("add-network-drive"))
                            .on_press(Message::AddNetworkDrive)
                            .into(),
                    ]))
                    .padding([space_xxs, space_xs])
                    .layer(cosmic_theme::Layer::Primary),
                );
            }
            _ => {}
        }
        let mut tab_view = widget::container(tab_column)
            .height(Length::Fill)
            .width(Length::Fill);

        if self.dnd_hovered.as_ref().map(|(l, _)| l) == Some(&tab_location) {
            tab_view = tab_view.style(|t| {
                let mut a = widget::container::Style::default();
                let c = t.cosmic();
                a.border = cosmic::iced_core::Border {
                    color: (c.accent_color()).into(),
                    width: 1.,
                    radius: c.radius_0().into(),
                };
                a
            });
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

    pub fn view<'a>(&'a self, key_binds: &'a HashMap<KeyBind, Action>) -> Element<Message> {
        widget::responsive(|size| self.view_responsive(key_binds, size)).into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let Some(items) = &self.items_opt else {
            return Subscription::none();
        };

        //TODO: how many thumbnail loads should be in flight at once?
        let jobs = 8;
        let mut subscriptions = Vec::with_capacity(jobs + 1);

        //TODO: move to function
        let visible_rect = {
            let point = match self.scroll_opt {
                Some(offset) => Point::new(0.0, offset.y),
                None => Point::new(0.0, 0.0),
            };
            let size = self.size_opt.get().unwrap_or_else(|| Size::new(0.0, 0.0));
            Rectangle::new(point, size)
        };

        //TODO: HACK to ensure positions are up to date since subscription runs before view
        match self.config.view {
            View::Grid => _ = self.grid_view(),
            View::List => _ = self.list_view(),
        };

        for item in items.iter() {
            if item.thumbnail_opt.is_some() {
                // Skip items that already have a mime type and thumbnail
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

            let Some(path) = item.path_opt().map(|path| path.to_path_buf()) else {
                continue;
            };
            let ItemMetadata::Path { metadata, .. } = item.metadata.clone() else {
                continue;
            };
            let mime = item.mime.clone();
            subscriptions.push(Subscription::run_with_id(
                path.clone(),
                stream::channel(1, |mut output| async move {
                    let message = {
                        let path = path.clone();
                        tokio::task::spawn_blocking(move || {
                            let start = Instant::now();
                            let thumbnail =
                                ItemThumbnail::new(&path, metadata, mime, THUMBNAIL_SIZE);
                            log::debug!("thumbnailed {:?} in {:?}", path, start.elapsed());
                            Message::Thumbnail(path.clone(), thumbnail)
                        })
                        .await
                        .unwrap()
                    };

                    match output.send(message).await {
                        Ok(()) => {}
                        Err(err) => {
                            log::warn!("failed to send thumbnail for {:?}: {}", path, err);
                        }
                    }

                    std::future::pending().await
                }),
            ));

            if subscriptions.len() >= jobs {
                break;
            }
        }

        // Load search items incrementally
        if let Location::Search(path, term, show_hidden, start) = &self.location {
            let location = self.location.clone();
            let path = path.clone();
            let term = term.clone();
            let show_hidden = *show_hidden;
            let start = start.clone();
            subscriptions.push(Subscription::run_with_id(
                location.clone(),
                stream::channel(2, move |mut output| async move {
                    //TODO: optimal size?
                    let (results_tx, results_rx) = mpsc::channel(65536);

                    let ready = Arc::new(atomic::AtomicBool::new(false));
                    let last_modified_opt = Arc::new(RwLock::new(None));
                    output
                        .send(Message::SearchContext(
                            location.clone(),
                            SearchContextWrapper(Some(SearchContext {
                                results_rx,
                                ready: ready.clone(),
                                last_modified_opt: last_modified_opt.clone(),
                            })),
                        ))
                        .await
                        .unwrap();

                    let output = Arc::new(tokio::sync::Mutex::new(output));
                    {
                        let output = output.clone();
                        tokio::task::spawn_blocking(move || {
                            scan_search(
                                &path,
                                &term,
                                show_hidden,
                                move |path, name, metadata| -> bool {
                                    // Don't send if the result is too old
                                    if let Some(last_modified) = *last_modified_opt.read().unwrap()
                                    {
                                        if let Ok(modified) = metadata.modified() {
                                            if modified < last_modified {
                                                return true;
                                            }
                                        } else {
                                            return true;
                                        }
                                    }

                                    match results_tx.blocking_send((
                                        path.to_path_buf(),
                                        name.to_string(),
                                        metadata,
                                    )) {
                                        Ok(()) => {
                                            if !ready.swap(true, atomic::Ordering::SeqCst) {
                                                // Wake up update method
                                                futures::executor::block_on(async {
                                                    output
                                                        .lock()
                                                        .await
                                                        .send(Message::SearchReady(false))
                                                        .await
                                                })
                                                .is_ok()
                                            } else {
                                                true
                                            }
                                        }
                                        Err(_) => false,
                                    }
                                },
                            );
                            log::info!(
                                "searched for {:?} in {:?} in {:?}",
                                term,
                                path,
                                start.elapsed(),
                            );
                        })
                        .await
                        .unwrap();
                    }

                    // Send final ready
                    let _ = output.lock().await.send(Message::SearchReady(true)).await;

                    std::future::pending().await
                }),
            ));
        }

        Subscription::batch(subscriptions)
    }
}

pub fn respond_to_scroll_direction(delta: ScrollDelta, modifiers: Modifiers) -> Option<Message> {
    if !modifiers.control() {
        return None;
    }

    let delta_y = match delta {
        ScrollDelta::Lines { y, .. } => y,
        ScrollDelta::Pixels { y, .. } => y,
    };

    if delta_y > 0.0 {
        return Some(Message::ZoomIn);
    }

    if delta_y < 0.0 {
        return Some(Message::ZoomOut);
    }

    None
}

#[cfg(test)]
mod tests {
    use std::{fs, io, path::PathBuf};

    use cosmic::{iced::mouse::ScrollDelta, iced_runtime::keyboard::Modifiers};
    use log::{debug, trace};
    use tempfile::TempDir;
    use test_log::test;

    use super::{respond_to_scroll_direction, scan_path, Location, Message, Tab};
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
    fn tab_scroll_up_with_ctrl_modifier_zooms() -> io::Result<()> {
        let message_maybe =
            respond_to_scroll_direction(ScrollDelta::Pixels { x: 0.0, y: 1.0 }, Modifiers::CTRL);
        assert!(!message_maybe.is_none());
        assert!(matches!(message_maybe.unwrap(), Message::ZoomIn));
        Ok(())
    }

    #[test]
    fn tab_scroll_up_without_ctrl_modifier_does_not_zoom() -> io::Result<()> {
        let message_maybe =
            respond_to_scroll_direction(ScrollDelta::Pixels { x: 0.0, y: 1.0 }, Modifiers::empty());
        assert!(message_maybe.is_none());
        Ok(())
    }

    #[test]
    fn tab_scroll_down_with_ctrl_modifier_zooms() -> io::Result<()> {
        let message_maybe =
            respond_to_scroll_direction(ScrollDelta::Pixels { x: 0.0, y: -1.0 }, Modifiers::CTRL);
        assert!(!message_maybe.is_none());
        assert!(matches!(message_maybe.unwrap(), Message::ZoomOut));
        Ok(())
    }

    #[test]
    fn tab_scroll_down_without_ctrl_modifier_does_not_zoom() -> io::Result<()> {
        let message_maybe = respond_to_scroll_direction(
            ScrollDelta::Pixels { x: 0.0, y: -1.0 },
            Modifiers::empty(),
        );
        assert!(message_maybe.is_none());
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

    #[test]
    fn sort_long_number_file_names() -> io::Result<()> {
        let fs = empty_fs()?;
        let path = fs.path();

        // Create files with names 255 characters long that only contain a single number
        // Example: 0000...0 for 255 characters
        // https://en.wikipedia.org/wiki/Filename#Comparison_of_filename_limitations
        let mut base_nums: Vec<_> = ('0'..'9').collect();
        fastrand::shuffle(&mut base_nums);
        debug!("Shuffled numbers for paths: {base_nums:?}");
        let paths: Vec<_> = base_nums
            .iter()
            .map(|&base| path.join(std::iter::repeat(base).take(255).collect::<String>()))
            .collect();

        for (file, &base) in paths.iter().zip(base_nums.iter()) {
            trace!("Creating long file name for {base}");
            fs::File::create(file)?;
        }

        debug!("Creating tab for directory of long file names");
        Tab::new(Location::Path(path.into()), TabConfig::default());

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
        operation: &mut dyn widget::Operation,
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
    ) -> event::Status {
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
        _translation: cosmic::iced_core::Vector,
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
        renderer: &cosmic::Renderer,
        _dnd_rectangles: &mut cosmic::iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.0.lock().unwrap().as_widget().drag_destinations(
            _state,
            _layout,
            renderer,
            _dnd_rectangles,
        )
    }
}

impl<Message: 'static> From<ArcElementWrapper<Message>> for Element<'static, Message> {
    fn from(wrapper: ArcElementWrapper<Message>) -> Self {
        Element::new(wrapper)
    }
}
