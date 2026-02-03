use chrono::{Datelike, Timelike, Utc};
use cosmic::{
    Apply, Element, cosmic_theme,
    desktop::fde::{DesktopEntry, get_languages_from_env},
    font,
    iced::{
        Alignment,
        Border,
        Color,
        ContentFit,
        Length,
        Point,
        Rectangle,
        Size,
        Subscription,
        Vector,
        advanced::{
            graphics,
            text::{self, Paragraph},
        },
        alignment::{Horizontal, Vertical},
        clipboard::dnd::DndAction,
        event,
        futures::{self, SinkExt},
        keyboard::Modifiers,
        padding,
        stream,
        //TODO: export in cosmic::widget
        widget::{
            horizontal_rule, rule,
            scrollable::{self, AbsoluteOffset, Viewport},
            stack,
        },
        window,
    },
    iced_core::{mouse::ScrollDelta, widget::tree},
    theme,
    widget::{
        self, DndDestination, DndSource, Id, Space, Widget,
        menu::{action::MenuAction, key_bind::KeyBind},
    },
};
use i18n_embed::LanguageLoader;
use icu::{
    datetime::{
        DateTimeFormatter, DateTimeFormatterPreferences, fieldsets,
        input::{Date, DateTime, Time},
        options::TimePrecision,
    },
    locale::preferences::extensions::unicode::keywords::HourCycle,
};
use image::{DynamicImage, ImageDecoder, ImageReader};
use jxl_oxide::integration::JxlDecoder;
use mime_guess::{Mime, mime};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    cmp::{Ordering, Reverse},
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::{self, Display},
    fs::{self, File, Metadata},
    hash::Hash,
    io::{BufRead, BufReader},
    os::unix::fs::MetadataExt,
    path::{self, Path, PathBuf},
    rc::Rc,
    sync::{Arc, LazyLock, RwLock, atomic},
    time::{Duration, Instant, SystemTime},
};
use tempfile::NamedTempFile;
use tokio::sync::mpsc;
use trash::TrashItemSize;
use walkdir::WalkDir;

use crate::{
    FxOrderMap,
    app::{Action, PreviewItem, PreviewKind},
    clipboard::{ClipboardCopy, ClipboardKind, ClipboardPaste},
    config::{DesktopConfig, ICON_SCALE_MAX, ICON_SIZE_GRID, IconSizes, TabConfig, ThumbCfg},
    dialog::DialogKind,
    fl,
    large_image::{
        LargeImageManager, decode_large_image, exceeds_memory_limit, should_use_dedicated_worker,
        should_use_tiling,
    },
    localize::{LANGUAGE_SORTER, LOCALE},
    menu, mime_app,
    mime_icon::{mime_for_path, mime_icon},
    mounter::MOUNTERS,
    mouse_area,
    operation::{Controller, OperationError},
    thumbnail_cacher::{CachedThumbnail, ThumbnailCacher, ThumbnailSize},
    thumbnailer::thumbnailer,
};
use uzers::{get_group_by_gid, get_user_by_uid};

pub const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
pub const HOVER_DURATION: Duration = Duration::from_millis(1600);
pub const TYPE_SELECT_TIMEOUT: Duration = Duration::from_millis(1000);
//TODO: best limit for search items
const MAX_SEARCH_LATENCY: Duration = Duration::from_millis(20);
const MAX_SEARCH_RESULTS: usize = 200;
//TODO: configurable thumbnail size?
const THUMBNAIL_SIZE: u32 = (ICON_SIZE_GRID as u32) * (ICON_SCALE_MAX as u32);

// Thumbnail generation semaphore - limits parallel thumbnail workers
// Uses 4 workers for balanced throughput and memory usage
pub static THUMB_SEMAPHORE: LazyLock<tokio::sync::Semaphore> =
    LazyLock::new(|| tokio::sync::Semaphore::const_new(num_cpus::get().min(4)));

pub(crate) static SORT_OPTION_FALLBACK: LazyLock<FxHashMap<String, (HeadingOptions, bool)>> =
    LazyLock::new(|| {
        FxHashMap::from_iter(dirs::download_dir().into_iter().map(|dir| {
            (
                Location::Path(dir).normalize().to_string(),
                (HeadingOptions::Modified, false),
            )
        }))
    });

static MODE_NAMES: LazyLock<Vec<String>> = LazyLock::new(|| {
    vec![
        // Mode 0
        fl!("none"),
        // Mode 1
        fl!("execute-only"),
        // Mode 2
        fl!("write-only"),
        // Mode 3
        fl!("write-execute"),
        // Mode 4
        fl!("read-only"),
        // Mode 5
        fl!("read-execute"),
        // Mode 6
        fl!("read-write"),
        // Mode 7
        fl!("read-write-execute"),
    ]
});

static SPECIAL_DIRS: LazyLock<FxHashMap<PathBuf, &'static str>> = LazyLock::new(|| {
    let mut special_dirs = FxHashMap::default();
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
    cut: bool,
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
            if cut {
                appearance.text_color = Some(Color::from(cosmic.accent.on_disabled));
            } else {
                appearance.text_color = Some(Color::from(cosmic.on_accent_color()));
            }
        } else {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        }
    } else if highlighted {
        if accent {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
            appearance.icon_color = Some(Color::from(cosmic.on_bg_component_color()));
            appearance.text_color = Some(Color::from(cosmic.on_bg_component_color()));
            if cut {
                appearance.text_color = Some(Color::from(cosmic.background.component.on_disabled));
            } else {
                appearance.text_color = Some(Color::from(cosmic.on_bg_component_color()));
            }
        } else {
            appearance.background = Some(Color::from(cosmic.bg_component_color()).into());
        }
    } else if desktop {
        appearance.background = Some(Color::from(cosmic.bg_color()).into());
        appearance.icon_color = Some(Color::from(cosmic.on_bg_color()));
        if cut {
            appearance.text_color = Some(Color::from(cosmic.background.component.disabled));
        } else {
            appearance.text_color = Some(Color::from(cosmic.on_bg_color()));
        }
    } else if cut {
        appearance.text_color = Some(Color::from(cosmic.background.component.on_disabled));
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
    cut: bool,
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
                cut,
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
                cut,
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
                cut,
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
                cut,
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

//TODO: replace with Path::has_trailing_sep when stable
fn has_trailing_sep(path: &Path) -> bool {
    path.as_os_str()
        .as_encoded_bytes()
        .last()
        .copied()
        .is_some_and(|b| path::is_separator(b as char))
}

fn tab_complete(path: &Path) -> Result<Vec<(String, PathBuf)>, Box<dyn Error>> {
    let parent = if has_trailing_sep(path) && path.is_dir() {
        // Show completions inside existing child directory instead of parent
        path
    } else {
        path.parent()
            .ok_or_else(|| format!("path has no parent {}", path.display()))?
    };

    let child_os = path.strip_prefix(parent)?;
    let child = child_os
        .to_str()
        .ok_or_else(|| format!("invalid UTF-8 {}", child_os.display()))?;

    let pattern = format!("^{}", regex::escape(child));
    let regex = regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()?;

    let mut completions = Vec::new();
    for entry_res in fs::read_dir(parent)? {
        let entry = entry_res?;
        let file_name_os = entry.file_name();
        let Some(file_name) = file_name_os.to_str() else {
            continue;
        };
        // Don't list hidden files before entering a pattern
        if pattern == "^" && file_name.starts_with('.') {
            continue;
        }
        if regex.is_match(file_name) {
            completions.push((file_name.to_string(), entry.path()));
        }
    }

    completions.sort_by(|a, b| LANGUAGE_SORTER.compare(&a.0, &b.0));
    //TODO: make the list scrollable?
    completions.truncate(8);
    Ok(completions)
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
    widget::icon::from_name(if trash::os_limited::is_empty().unwrap_or(true) {
        "user-trash"
    } else {
        "user-trash-full"
    })
    .size(icon_size)
    .handle()
}

pub fn trash_icon_symbolic(icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(if trash::os_limited::is_empty().unwrap_or(true) {
        "user-trash-symbolic"
    } else {
        "user-trash-full-symbolic"
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
        format!("{size} B")
    }
}

const MODE_SHIFT_USER: u32 = 6;
const MODE_SHIFT_GROUP: u32 = 3;
const MODE_SHIFT_OTHER: u32 = 0;

const fn get_mode_part(mode: u32, shift: u32) -> u32 {
    (mode >> shift) & 0o7
}

fn set_mode_part(mode: u32, shift: u32, bits: u32) -> u32 {
    assert!(bits <= 0o7);
    (mode & !(0o7 << shift)) | (bits << shift)
}

fn date_time_formatter(military_time: bool) -> DateTimeFormatter<fieldsets::YMDT> {
    let mut prefs = DateTimeFormatterPreferences::from(LOCALE.clone());
    prefs.hour_cycle = Some(if military_time {
        HourCycle::H23
    } else {
        HourCycle::H12
    });

    let mut fs = fieldsets::YMDT::medium();
    fs = fs.with_time_precision(TimePrecision::Minute);

    DateTimeFormatter::try_new(prefs, fs).expect("failed to create DateTimeFormatter")
}

fn time_formatter(military_time: bool) -> DateTimeFormatter<fieldsets::T> {
    let mut prefs = DateTimeFormatterPreferences::from(LOCALE.clone());
    prefs.hour_cycle = Some(if military_time {
        HourCycle::H23
    } else {
        HourCycle::H12
    });

    let mut fs = fieldsets::T::medium();
    fs = fs.with_time_precision(TimePrecision::Minute);

    DateTimeFormatter::try_new(prefs, fs).expect("failed to create DateTimeFormatter")
}

struct FormatTime<'a> {
    pub time: SystemTime,
    pub date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
    pub time_formatter: &'a DateTimeFormatter<fieldsets::T>,
}

impl<'a> FormatTime<'a> {
    fn from_secs(
        secs: i64,
        date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
        time_formatter: &'a DateTimeFormatter<fieldsets::T>,
    ) -> Option<Self> {
        // This looks convoluted because we need to ensure the units match up
        let secs: u64 = secs.try_into().ok()?;
        let now = SystemTime::now();
        let filetime_diff = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|from_epoch| from_epoch.as_secs())
            .ok()
            .and_then(|now_secs| now_secs.checked_sub(secs))
            .map(Duration::from_secs)?;
        now.checked_sub(filetime_diff).map(|time| Self {
            time,
            date_time_formatter,
            time_formatter,
        })
    }
}

impl Display for FormatTime<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let datetime = chrono::DateTime::<chrono::Local>::from(self.time);
        let now = chrono::Local::now();
        let icu_datetime = DateTime {
            date: Date::try_new_gregorian(
                datetime.year(),
                datetime.month() as u8,
                datetime.day() as u8,
            )
            .unwrap(),
            time: Time::try_new(
                datetime.hour() as u8,
                datetime.minute() as u8,
                datetime.second() as u8,
                0,
            )
            .unwrap(),
        };

        if datetime.date_naive() == now.date_naive() {
            f.write_str(fl!("today").as_str())?;
            f.write_str(", ")?;
            self.time_formatter.format(&icu_datetime).fmt(f)
        } else {
            self.date_time_formatter.format(&icu_datetime).fmt(f)
        }
    }
}

const fn format_time<'a>(
    time: SystemTime,
    date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
    time_formatter: &'a DateTimeFormatter<fieldsets::T>,
) -> FormatTime<'a> {
    FormatTime {
        time,
        date_time_formatter,
        time_formatter,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsKind {
    Local,
    Remote,
    Gvfs,
}

#[cfg(target_os = "linux")]
pub fn fs_kind(metadata: &Metadata) -> FsKind {
    //TODO: method to reload remote filesystems dynamically
    //TODO: fix for https://github.com/eminence/procfs/issues/262
    static DEVICES: LazyLock<FxHashMap<u64, FsKind>> = LazyLock::new(|| {
        let mut devices = FxHashMap::default();
        match procfs::process::Process::myself() {
            Ok(process) => match process.mountinfo() {
                Ok(mount_infos) => {
                    devices = FxHashMap::from_iter(mount_infos.iter().filter_map(|mount_info| {
                        let mut parts = mount_info.majmin.split(':');
                        let major_str = parts.next()?;
                        let minor_str = parts.next()?;
                        let major = major_str.parse::<libc::c_uint>().ok()?;
                        let minor = minor_str.parse::<libc::c_uint>().ok()?;
                        let dev = libc::makedev(major, minor);
                        // Network and distributed filesystem types
                        // Based on common remote filesystem types found in /proc/mounts
                        let kind = match mount_info.fs_type.as_str() {
                            // SMB/CIFS variants
                            "cifs" | "smb" | "smb2" | "smbfs" => FsKind::Remote,

                            // NFS variants
                            "nfs" | "nfs4" => FsKind::Remote,

                            // FUSE-based remote filesystems
                            "fuse.rclone" | "fuse.sshfs" | "fuse.davfs2" | "fuse.ceph"
                            | "fuse.glusterfs" | "fuse.s3fs" | "fuse.goofys" | "fuse.gcsfuse"
                            | "fuse.afp" | "fuse.afpfs" => FsKind::Remote,

                            // Other network protocols
                            "afs" | "coda" | "ncpfs" | "davfs" | "davfs2" | "shfs" => {
                                FsKind::Remote
                            }

                            // Cluster/distributed filesystems
                            "ceph" | "glusterfs" | "lustre" | "gfs" | "gfs2" | "ocfs2" => {
                                FsKind::Remote
                            }

                            // GVFS (GNOME Virtual File System)
                            "fuse.gvfsd-fuse" => FsKind::Gvfs,

                            // Everything else is local
                            _ => FsKind::Local,
                        };
                        Some((dev, kind))
                    }));
                }
                Err(err) => {
                    log::warn!("failed to get mount info: {err}");
                }
            },
            Err(err) => {
                log::warn!("failed to get process info: {err}");
            }
        }
        devices
    });
    DEVICES.get(&metadata.dev()).map_or(FsKind::Local, |x| *x)
}

#[cfg(not(target_os = "linux"))]
pub fn fs_kind(_metadata: &Metadata) -> FsKind {
    //TODO: support BSD, macOS, Windows?
    FsKind::Local
}

fn get_desktop_file_display_name(path: &Path) -> Option<String> {
    let locales = get_languages_from_env();
    let entry = match DesktopEntry::from_path(path, Some(&locales)) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse {}: {}", path.display(), err);
            return None;
        }
    };

    entry.name(&locales).map(|s| s.into_owned())
}

fn get_desktop_file_icon(path: &Path) -> Option<String> {
    let entry = match DesktopEntry::from_path::<&str>(path, None) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse {}: {}", path.display(), err);
            return None;
        }
    };

    entry.icon().map(str::to_string)
}

/// Creates an icon handle from a desktop file's Icon field value.
/// Supports both icon names (looked up in theme) and absolute paths (used directly).
fn desktop_icon_handle(icon: &str, size: u16) -> widget::icon::Handle {
    let icon_path = Path::new(icon);
    if icon_path.is_absolute() && icon_path.exists() {
        widget::icon::from_path(icon_path.to_path_buf())
    } else {
        widget::icon::from_name(icon).size(size).handle()
    }
}

pub fn parse_desktop_file(path: &Path) -> (Option<String>, Option<String>) {
    let locales = get_languages_from_env();
    let entry = match DesktopEntry::from_path(path, Some(&locales)) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse {}: {}", path.display(), err);
            return (None, None);
        }
    };
    (
        entry.name(&locales).map(|s| s.into_owned()),
        entry.icon().map(str::to_string),
    )
}

fn display_name_for_file(path: &Path, name: &str, get_from_gvfs: bool, is_desktop: bool) -> String {
    if is_desktop {
        return get_desktop_file_display_name(path).map_or_else(
            || Item::display_name(name),
            |desktop_name| Item::display_name(desktop_name.as_str()),
        );
    } else if get_from_gvfs {
        #[cfg(feature = "gvfs")]
        return Item::display_name(glib::filename_display_name(path).as_str());
    }
    Item::display_name(name)
}

#[cfg(feature = "gvfs")]
pub fn item_from_gvfs_info(path: PathBuf, file_info: gio::FileInfo, sizes: IconSizes) -> Item {
    let file_name = file_info
        .attribute_as_string(gio::FILE_ATTRIBUTE_STANDARD_NAME)
        .unwrap_or_default();
    let mtime = file_info.attribute_uint64(gio::FILE_ATTRIBUTE_TIME_MODIFIED);
    let mut is_desktop = false;
    let remote = file_info.boolean(gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE);
    let is_dir = matches!(file_info.file_type(), gio::FileType::Directory);

    let size_opt = (!is_dir).then_some(file_info.size() as u64);

    let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = if is_dir {
        (
            //TODO: make this a static
            "inode/directory".parse().unwrap(),
            folder_icon(&path, sizes.grid()),
            folder_icon(&path, sizes.list()),
            folder_icon(&path, sizes.list_condensed()),
        )
    } else {
        // ALWAYS assume we're remote for mime guessing here, since gvfs reading can be expensive
        // @todo - expose this as a config option?
        let mime = mime_for_path(&path, None, true);

        //TODO: clean this up, implement for trash
        let icon_name_opt = if mime == "application/x-desktop" {
            is_desktop = true;
            get_desktop_file_icon(&path)
        } else {
            None
        };
        if let Some(icon_name) = icon_name_opt {
            (
                mime,
                desktop_icon_handle(&icon_name, sizes.grid()),
                desktop_icon_handle(&icon_name, sizes.list()),
                desktop_icon_handle(&icon_name, sizes.list_condensed()),
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

    let mut children_opt = None;
    let mut dir_size = DirSize::NotDirectory;
    if is_dir && !remote {
        dir_size = DirSize::Calculating(Controller::default());
        //TODO: calculate children in the background (and make it cancellable?)
        match fs::read_dir(&path) {
            Ok(entries) => {
                children_opt = Some(entries.count());
            }
            Err(err) => {
                log::warn!("failed to read directory {}: {}", path.display(), err);
            }
        }
    }

    let display_name = display_name_for_file(&path, &file_info.display_name(), false, is_desktop);
    let hidden = file_name.starts_with('.');

    Item {
        name: file_name.into(),
        display_name,
        is_mount_point: false,
        metadata: ItemMetadata::GvfsPath {
            mtime,
            size_opt,
            children_opt,
        },
        hidden,
        location_opt: Some(Location::Path(path)),
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        thumbnail_opt: if remote {
            Some(ItemThumbnail::NotImage)
        } else {
            None
        },
        button_id: widget::Id::unique(),
        pos_opt: Cell::new(None),
        rect_opt: Cell::new(None),
        selected: false,
        highlighted: false,
        overlaps_drag_rect: false,
        dir_size,
        cut: false,
    }
}

pub fn item_from_entry(
    path: PathBuf,
    name: String,
    metadata: fs::Metadata,
    sizes: IconSizes,
) -> Item {
    let mut is_desktop = false;
    let mut is_gvfs = false;

    let hidden = name.starts_with('.') || hidden_attribute(&metadata);

    let remote = match fs_kind(&metadata) {
        FsKind::Local => false,
        FsKind::Remote => true,
        #[cfg(feature = "gvfs")]
        FsKind::Gvfs => {
            is_gvfs = true;
            let file = gio::File::for_path(&path);

            match gio::prelude::FileExt::query_filesystem_info(
                &file,
                gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE,
                gio::Cancellable::NONE,
            ) {
                Ok(info) => info.boolean(gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE),
                Err(err) => {
                    log::warn!(
                        "failed to get GIO filesystem info for {}: {}",
                        path.display(),
                        err
                    );
                    true
                }
            }
        }
        #[cfg(not(feature = "gvfs"))]
        FsKind::Gvfs => {
            log::info!(
                "gvfs feature not enabled, info may be inaccurate for {}",
                path.display()
            );
            true
        }
    };

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
            let mime = mime_for_path(&path, Some(&metadata), remote);
            //TODO: clean this up, implement for trash
            let icon_name_opt = if mime == "application/x-desktop" {
                is_desktop = true;
                get_desktop_file_icon(&path)
            } else {
                None
            };
            if let Some(icon_name) = icon_name_opt {
                (
                    mime,
                    desktop_icon_handle(&icon_name, sizes.grid()),
                    desktop_icon_handle(&icon_name, sizes.list()),
                    desktop_icon_handle(&icon_name, sizes.list_condensed()),
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

    let mut children_opt = None;
    let mut dir_size = DirSize::NotDirectory;
    if metadata.is_dir() && !remote {
        dir_size = DirSize::Calculating(Controller::default());
        //TODO: calculate children in the background (and make it cancellable?)
        match fs::read_dir(&path) {
            Ok(entries) => {
                children_opt = Some(entries.count());
            }
            Err(err) => {
                log::warn!("failed to read directory {}: {}", path.display(), err);
            }
        }
    }

    let display_name = display_name_for_file(&path, &name, is_gvfs, is_desktop);

    Item {
        name,
        display_name,
        is_mount_point: false,
        metadata: ItemMetadata::Path {
            metadata,
            children_opt,
        },
        hidden,
        location_opt: Some(Location::Path(path)),
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        thumbnail_opt: remote.then_some(ItemThumbnail::NotImage),
        button_id: widget::Id::unique(),
        pos_opt: Cell::new(None),
        rect_opt: Cell::new(None),
        selected: false,
        highlighted: false,
        overlaps_drag_rect: false,
        dir_size,
        cut: false,
    }
}

fn get_filename_from_path(path: &Path) -> Result<String, String> {
    Ok(match path.file_name() {
        Some(name_os) => name_os
            .to_str()
            .ok_or_else(|| {
                format!(
                    "failed to parse file name for {}: {name_os:?} is not valid UTF-8",
                    path.display()
                )
            })?
            .to_string(),
        None => fl!("filesystem"),
    })
}

pub fn item_from_path<P: Into<PathBuf>>(path: P, sizes: IconSizes) -> Result<Item, String> {
    let path = path.into();
    let name = get_filename_from_path(&path)?;
    let metadata = fs::metadata(&path)
        .map_err(|err| format!("failed to read metadata for {}: {}", path.display(), err))?;
    Ok(item_from_entry(path, name, metadata, sizes))
}

pub fn scan_path(tab_path: &PathBuf, sizes: IconSizes) -> Vec<Item> {
    let mut items = Vec::new();
    let mut hidden_files = Box::from([]);
    let mut remote_scannable = false;

    #[cfg(feature = "gvfs")]
    {
        if let Ok(path_meta) = fs::metadata(tab_path)
            && fs_kind(&path_meta) == FsKind::Gvfs
        {
            let file = gio::File::for_path(tab_path);

            // gio crate expects a comma delimited string
            let attr_string = [
                gio::FILE_ATTRIBUTE_STANDARD_DISPLAY_NAME.as_str(),
                gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE.as_str(),
                gio::FILE_ATTRIBUTE_TIME_MODIFIED.as_str(),
                gio::FILE_ATTRIBUTE_STANDARD_SIZE.as_str(),
                gio::FILE_ATTRIBUTE_STANDARD_TYPE.as_str(),
                gio::FILE_ATTRIBUTE_STANDARD_NAME.as_str(),
            ]
            .join(",");

            match gio::prelude::FileExt::enumerate_children(
                &file,
                attr_string.as_str(),
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            ) {
                Ok(res) => {
                    remote_scannable = true;
                    items = res
                        .filter_map(|file| {
                            let file = file.ok()?;
                            Some(item_from_gvfs_info(tab_path.join(file.name()), file, sizes))
                        })
                        .collect();
                }
                Err(err) => {
                    log::warn!(
                        "could not enumerate {} via gio: {}",
                        tab_path.display(),
                        err
                    );
                }
            }
        }
    }

    if !remote_scannable {
        match fs::read_dir(tab_path) {
            Ok(entries) => {
                items = entries
                    .filter_map(|entry_res| {
                        let entry = entry_res
                            .inspect_err(|err| {
                                log::warn!(
                                    "failed to read entry in {}: {}",
                                    tab_path.display(),
                                    err
                                )
                            })
                            .ok()?;

                        let path = entry.path();

                        let name = entry
                            .file_name()
                            .into_string()
                            .inspect_err(|name_os| {
                                log::warn!(
                                    "failed to parse entry at {}: {:?} is not valid UTF-8",
                                    path.display(),
                                    name_os
                                )
                            })
                            .ok()?;

                        if name == ".hidden" && path.is_file() {
                            hidden_files = parse_hidden_file(&path);
                        }

                        let metadata = fs::metadata(&path)
                            .inspect_err(|err| {
                                log::warn!(
                                    "failed to read metadata for entry at {}: {}",
                                    path.display(),
                                    err
                                )
                            })
                            .ok()?;

                        Some(item_from_entry(path, name, metadata, sizes))
                    })
                    .collect();
            }
            Err(err) => {
                log::warn!("failed to read directory {}: {}", tab_path.display(), err);
            }
        }
    }
    items.sort_unstable_by(|a, b| match (a.metadata.is_dir(), b.metadata.is_dir()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => LANGUAGE_SORTER.compare(&a.display_name, &b.display_name),
    });
    for item in &mut items {
        if hidden_files.contains(&item.name) {
            item.hidden = true;
        }
    }
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

    let pattern = regex::escape(term);
    let regex = match regex::RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
    {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to parse regex {pattern:?}: {err}");
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
                            log::warn!(
                                "failed to read metadata for entry at {}: {}",
                                path.display(),
                                err
                            );
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
    let entries = match trash::os_limited::list() {
        Ok(entry) => entry,
        Err(err) => {
            log::warn!("failed to read trash items: {err}");
            return Vec::new();
        }
    };
    let mut items: Vec<_> = entries
        .into_iter()
        .filter_map(|entry| {
            let metadata = trash::os_limited::metadata(&entry)
                .inspect_err(|err| {
                    log::warn!("failed to get metadata for trash item {entry:?}: {err}")
                })
                .ok()?;
            let original_path = entry.original_path();
            let name = entry.name.to_string_lossy().into_owned();
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
                        // This passes remote = true so it does not read from the original path
                        let mime = mime_for_path(&original_path, None, true);
                        (
                            mime.clone(),
                            mime_icon(mime.clone(), sizes.grid()),
                            mime_icon(mime.clone(), sizes.list()),
                            mime_icon(mime, sizes.list_condensed()),
                        )
                    }
                };

            Some(Item {
                name,
                display_name,
                is_mount_point: false,
                metadata: ItemMetadata::Trash { metadata, entry },
                hidden: false,
                location_opt: None,
                mime,
                icon_handle_grid,
                icon_handle_list,
                icon_handle_list_condensed,
                thumbnail_opt: Some(ItemThumbnail::NotImage),
                button_id: widget::Id::unique(),
                pos_opt: Cell::new(None),
                rect_opt: Cell::new(None),
                selected: false,
                highlighted: false,
                overlaps_drag_rect: false,
                dir_size: DirSize::NotDirectory,
                cut: false,
            })
        })
        .collect();
    items.sort_by(|a, b| match (a.metadata.is_dir(), b.metadata.is_dir()) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => LANGUAGE_SORTER.compare(&a.display_name, &b.display_name),
    });
    items
}

fn uri_to_path(uri: String) -> Option<PathBuf> {
    uri.parse::<url::Url>().ok().and_then(|url| {
        //TODO support for external drive or cloud?
        if url.scheme() == "file" {
            url.to_file_path().ok()
        } else {
            None
        }
    })
}

pub fn scan_recents(sizes: IconSizes) -> Vec<Item> {
    let recent_files = match recently_used_xbel::parse_file() {
        Ok(recent_files) => recent_files,
        Err(err) => {
            log::warn!("Error reading recent files: {err:?}");
            return Vec::new();
        }
    };
    let mut recents: Vec<_> = recent_files
        .bookmarks
        .into_iter()
        .filter_map(|bookmark| {
            let path = uri_to_path(bookmark.href)?;
            let last_edit = bookmark.modified.parse::<chrono::DateTime<Utc>>().ok()?;
            let last_visit = bookmark.visited.parse::<chrono::DateTime<Utc>>().ok()?;

            if path.exists() {
                let file_name = path.file_name()?;
                let name = file_name.to_string_lossy().to_string();

                let metadata = match path.metadata() {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!(
                            "failed to read metadata for entry at {}: {}",
                            path.display(),
                            err
                        );
                        return None;
                    }
                };

                let item = item_from_entry(path, name, metadata, sizes);
                Some((item, last_edit.min(last_visit)))
            } else {
                log::warn!("recent file path not exist: {}", path.display());
                None
            }
        })
        .collect();

    recents.sort_by_key(|recent| Reverse(recent.1));

    recents.into_iter().take(50).map(|(item, _)| item).collect()
}

pub fn scan_network(uri: &str, sizes: IconSizes) -> Vec<Item> {
    for mounter in MOUNTERS.values() {
        match mounter.network_scan(uri, sizes) {
            Some(Ok(items)) => return items,
            Some(Err(err)) => {
                log::warn!("failed to scan {uri:?}: {err}");
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
    mut sizes: IconSizes,
) -> Vec<Item> {
    sizes.grid = desktop_config.icon_size;

    let mut items = Vec::new();

    if desktop_config.show_content {
        items.extend(scan_path(tab_path, sizes));
    }

    if desktop_config.show_mounted_drives {
        for mounter in MOUNTERS.values() {
            let Some(mounter_items) = mounter.items(sizes) else {
                continue;
            };
            items.extend(mounter_items.into_iter().filter_map(|mounter_item| {
                let path = mounter_item.path()?;
                // Get most item data from path
                let mut item = match item_from_path(&path, sizes) {
                    Ok(item) => item,
                    Err(err) => {
                        log::warn!(
                            "failed to get item from mounter item {}: {}",
                            path.display(),
                            err
                        );
                        return None;
                    }
                };

                //Override some data with mounter information
                item.name = mounter_item.name();
                item.display_name = Item::display_name(&item.name);

                //TODO: use icon size for mounter item icon
                if let Some(icon) = mounter_item.icon(false) {
                    item.icon_handle_grid.clone_from(&icon);
                    item.icon_handle_list.clone_from(&icon);
                    item.icon_handle_list_condensed = icon;
                }

                Some(item)
            }));
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
            is_mount_point: false,
            metadata,
            hidden: false,
            location_opt: Some(Location::Trash),
            mime,
            icon_handle_grid,
            icon_handle_list,
            icon_handle_list_condensed,
            thumbnail_opt: Some(ItemThumbnail::NotImage),
            button_id: widget::Id::unique(),
            pos_opt: Cell::new(None),
            rect_opt: Cell::new(None),
            selected: false,
            highlighted: false,
            overlaps_drag_rect: false,
            dir_size: DirSize::NotDirectory,
            cut: false,
        });
    }

    items
}

#[derive(Clone, Debug)]
pub struct EditLocation {
    pub location: Location,
    pub completions: Option<Vec<(String, PathBuf)>>,
    pub selected: Option<usize>,
}

impl EditLocation {
    pub fn resolve(&self) -> Option<Location> {
        if let Location::Network(uri, ..) = &self.location {
            MOUNTERS
                .values()
                .find_map(|mounter| mounter.dir_info(uri))
                .map(|(uri, display_name, path_opt)| Location::Network(uri, display_name, path_opt))
        } else {
            let Some(selected) = self.selected else {
                return Some(self.location.clone());
            };
            let completions = self.completions.as_ref()?;
            let completion = completions.get(selected)?;
            Some(self.location.with_path(completion.1.clone()).normalize())
        }
    }

    pub fn select(&mut self, forwards: bool) {
        if let Some(completions) = &self.completions {
            if completions.is_empty() {
                self.selected = None;
            } else {
                let mut selected = if forwards {
                    self.selected.and_then(|x| x.checked_add(1)).unwrap_or(0)
                } else {
                    self.selected
                        .and_then(|x| x.checked_sub(1))
                        .unwrap_or(completions.len() - 1)
                };
                if selected >= completions.len() {
                    selected = 0;
                }
                self.selected = Some(selected);

                // Automatically resolve if there is only one completion
                if completions.len() == 1
                    && let Some(resolved) = self.resolve()
                {
                    self.location = resolved;
                    self.selected = None;
                }
            }
        } else {
            self.selected = None;
        }
    }
}

impl From<Location> for EditLocation {
    fn from(location: Location) -> Self {
        Self {
            location,
            completions: None,
            selected: None,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Location {
    Desktop(PathBuf, String, DesktopConfig),
    Network(String, String, Option<PathBuf>),
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
            Self::Network(uri, ..) => write!(f, "{uri}"),
            Self::Path(path) => write!(f, "{}", path.display()),
            Self::Recents => write!(f, "recents"),
            Self::Search(path, term, ..) => write!(f, "search {} for {}", path.display(), term),
            Self::Trash => write!(f, "trash"),
        }
    }
}

impl Location {
    pub fn normalize(&self) -> Self {
        if let Location::Network(uri, ..) = self {
            if !uri.ends_with('/') {
                let mut uri = uri.clone();
                uri.push('/');
                self.with_uri(uri)
            } else {
                self.clone()
            }
        } else if let Some(mut path) = self.path_opt().cloned() {
            // Canonicalize path, if possible
            if let Ok(canonical) = fs::canonicalize(&path) {
                path = canonical;
            }
            // Add trailing slash if location is a directory
            if path.is_dir() {
                path.push("");
            }
            self.with_path(path)
        } else {
            self.clone()
        }
    }

    pub fn ancestors(&self) -> Vec<(Self, String)> {
        self.path_opt().map_or_else(Default::default, |path| {
            path.ancestors()
                .scan(false, |found_home, ancestor| {
                    (!*found_home).then(|| {
                        let (name, is_home) = folder_name(ancestor);
                        *found_home = is_home;
                        (self.with_path(ancestor.to_path_buf()), name)
                    })
                })
                .collect()
        })
    }

    pub const fn path_opt(&self) -> Option<&PathBuf> {
        match self {
            Self::Desktop(path, ..) => Some(path),
            Self::Path(path) => Some(path),
            Self::Search(path, ..) => Some(path),
            Self::Network(_, _, path) => path.as_ref(),
            _ => None,
        }
    }

    pub(crate) fn into_path_opt(self) -> Option<PathBuf> {
        match self {
            Self::Desktop(path, ..) => Some(path),
            Self::Path(path) => Some(path),
            Self::Search(path, ..) => Some(path),
            Self::Network(_, _, path) => path,
            _ => None,
        }
    }

    pub fn with_path(&self, path: PathBuf) -> Self {
        let path = Self::expand_tilde(path);
        match self {
            Self::Desktop(_, display, desktop_config) => {
                Self::Desktop(path, display.clone(), *desktop_config)
            }
            Self::Path(..) => Self::Path(path),
            Self::Search(_, term, show_hidden, time) => {
                Self::Search(path, term.clone(), *show_hidden, *time)
            }

            other => other.clone(),
        }
    }

    pub fn with_uri(&self, uri: String) -> Self {
        if let Self::Network(_, name, path) = self {
            Self::Network(uri, name.clone(), path.clone())
        } else {
            self.clone()
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
            Self::Network(uri, _, _) => scan_network(uri, sizes),
        };
        let parent_item_opt = match self.path_opt() {
            Some(path) => match item_from_path(path, sizes) {
                Ok(item) => Some(item),
                Err(err) => {
                    log::warn!("failed to get item for {}: {}", path.display(), err);
                    None
                }
            },
            //TODO: support other locations?
            None => None,
        };
        (parent_item_opt, items)
    }

    pub fn title(&self) -> String {
        match self {
            Self::Desktop(path, _, _) => {
                let (name, _) = folder_name(path);
                name
            }
            Self::Path(path) => {
                let (name, _) = folder_name(path);
                name
            }
            Self::Search(path, term, ..) => {
                //TODO: translate
                let (name, _) = folder_name(path);
                format!("Search \"{term}\": {name}")
            }
            Self::Trash => {
                fl!("trash")
            }
            Self::Recents => {
                fl!("recents")
            }
            Self::Network(display_name, ..) => display_name.clone(),
        }
    }

    /// Expand a path that starts with "~" with the
    /// user's home directory
    pub fn expand_tilde(path: PathBuf) -> PathBuf {
        let mut components = path.components();
        match components.next() {
            Some(path::Component::Normal(os_str)) if os_str == "~" => {
                if let Some(home) = dirs::home_dir() {
                    home.join(components.as_path())
                } else {
                    path
                }
            }
            _ => path,
        }
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
    AutoScroll(Option<f32>),
    ChangeLocation(String, Location, Option<Vec<PathBuf>>),
    ContextMenu(Option<Point>, Option<window::Id>),
    Delete(Vec<PathBuf>),
    DropFiles(PathBuf, ClipboardPaste),
    EmptyTrash,
    #[cfg(feature = "desktop")]
    ExecEntryAction(cosmic::desktop::DesktopEntryData, usize),
    Iced(TaskWrapper),
    OpenFile(Vec<PathBuf>),
    OpenInNewTab(PathBuf),
    OpenInNewWindow(PathBuf),
    OpenTrash,
    Preview(PreviewKind),
    SetOpenWith(Mime, String),
    SetPermissions(PathBuf, u32),
    SetSort(String, HeadingOptions, bool),
    WindowDrag,
    WindowToggleMaximize,
}

#[derive(Clone, Debug)]
pub enum Message {
    AddNetworkDrive,
    AutoScroll(Option<f32>),
    Click(Option<usize>),
    DoubleClick(Option<usize>),
    ClickRelease(Option<usize>),
    Config(TabConfig),
    ContextAction(Action),
    ContextMenu(Option<Point>, Option<window::Id>),
    LocationContextMenuPoint(Option<Point>),
    LocationContextMenuIndex(Option<Point>, Option<usize>),
    LocationMenuAction(LocationMenuAction),
    Drag(Option<Rectangle>),
    DragEnd,
    EditLocation(Option<EditLocation>),
    EditLocationComplete(usize),
    EditLocationEnable,
    EditLocationSubmit,
    EditLocationTab,
    OpenInNewTab(PathBuf),
    EmptyTrash,
    #[cfg(feature = "desktop")]
    ExecEntryAction(Option<PathBuf>, usize),
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
    Reload,
    RightClick(Option<Point>, Option<usize>),
    MiddleClick(usize),
    Resize(Rectangle),
    Scroll(Viewport),
    ScrollTab(f32),
    ScrollToFocused,
    SearchContext(Location, SearchContextWrapper),
    SearchReady(bool),
    SelectAll,
    SelectFirst,
    SelectLast,
    SetOpenWith(Mime, String),
    SetPermissions(PathBuf, u32),
    SetSort(HeadingOptions, bool),
    TabComplete(PathBuf, Vec<(String, PathBuf)>),
    Thumbnail(PathBuf, ItemThumbnail),
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
    DirectorySize(PathBuf, DirSize),
    ImageDecoded(PathBuf, u32, u32, Vec<u8>, Option<(u32, u32)>, u64), // path, width, height, pixels, display_size, generation
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
pub enum DirSize {
    Calculating(Controller),
    Directory(u64),
    NotDirectory,
    Error(String),
}

#[derive(Clone, Debug)]
pub enum ItemMetadata {
    Path {
        metadata: Metadata,
        children_opt: Option<usize>,
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
    #[cfg(feature = "gvfs")]
    GvfsPath {
        mtime: u64,
        size_opt: Option<u64>,
        children_opt: Option<usize>,
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
            #[cfg(feature = "gvfs")]
            Self::GvfsPath { children_opt, .. } => children_opt.is_some(),
        }
    }

    pub fn modified(&self) -> Option<SystemTime> {
        match self {
            Self::Path { metadata, .. } => metadata.modified().ok(),
            #[cfg(feature = "gvfs")]
            Self::GvfsPath { mtime, .. } => {
                Some(SystemTime::UNIX_EPOCH + Duration::from_secs(*mtime))
            }
            _ => None,
        }
    }

    pub fn file_size(&self) -> Option<u64> {
        match self {
            Self::Path { metadata, .. } => (!metadata.is_dir()).then_some(metadata.len()),
            Self::Trash { metadata, .. } => match metadata.size {
                TrashItemSize::Bytes(size) => Some(size),
                TrashItemSize::Entries(_) => None,
            },
            #[cfg(feature = "gvfs")]
            Self::GvfsPath { size_opt, .. } => *size_opt,
            _ => None,
        }
    }

    pub fn children_count(&self) -> Option<&usize> {
        match &self {
            ItemMetadata::Path { children_opt, .. } => children_opt.as_ref(),
            #[cfg(feature = "gvfs")]
            ItemMetadata::GvfsPath { children_opt, .. } => children_opt.as_ref(),
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
    pub fn new(
        path: &Path,
        metadata: ItemMetadata,
        mime: mime::Mime,
        mut thumbnail_size: u32,
        max_mem: u64,
        jobs: usize,
        max_size_mb: u64,
    ) -> Self {
        let thumbnail_cacher =
            ThumbnailCacher::new(path, ThumbnailSize::from_pixel_size(thumbnail_size));
        match thumbnail_cacher.as_ref() {
            Ok(cache) => match cache.get_cached_thumbnail() {
                CachedThumbnail::Valid((thumbnail_path, size)) => {
                    // Check original image dimensions even when loading cached thumbnail
                    // This prevents trying to load huge images in preview mode
                    let original_dims = match image::image_dimensions(path) {
                        Ok((width, height)) => Some((width, height)),
                        Err(_) => size.map(|s| (s.pixel_size(), s.pixel_size())),
                    };

                    return Self::Image(
                        widget::image::Handle::from_path(thumbnail_path),
                        original_dims,
                    );
                }
                CachedThumbnail::Failed => {
                    if mime.type_() != mime::IMAGE {
                        return Self::NotImage;
                    }
                }
                CachedThumbnail::RequiresUpdate(size) => {
                    thumbnail_size = size.pixel_size();
                }
            },
            Err(err) => {
                log::warn!(
                    "failed to create ThumbnailCache for {}: {}",
                    path.display(),
                    err
                );
            }
        }

        let size = metadata.file_size().unwrap_or_default();
        let check_size = |thumbnailer: &str, max_size| {
            if size <= max_size {
                true
            } else {
                log::warn!(
                    "skipping internal {} thumbnailer for {}: file size {} is larger than {}",
                    thumbnailer,
                    path.display(),
                    format_size(size),
                    format_size(max_size)
                );
                false
            }
        };

        let mut tried_supported_file = false;
        // First try built-in image thumbnailer
        if mime.type_() == mime::IMAGE && check_size("image", max_size_mb * 1000 * 1000) {
            // Check if image dimensions would exceed available memory budget
            // The GPU tiling system can handle large images, but we still need to decode them first
            let dimensions_ok = match image::image_dimensions(path) {
                Ok((width, height)) => {
                    if exceeds_memory_limit(width, height, max_mem) {
                        log::warn!(
                            "skipping thumbnail generation for {}: {}x{} image would exceed {}MB memory budget",
                            path.display(),
                            width,
                            height,
                            max_mem
                        );
                        false
                    } else {
                        if should_use_tiling(width, height) {
                            log::info!(
                                "Large image {}x{} detected, will use GPU tiling for display",
                                width,
                                height
                            );
                        }
                        true
                    }
                }
                Err(err) => {
                    log::debug!(
                        "failed to read dimensions for {}: {}, will try decoding",
                        path.display(),
                        err
                    );
                    true // If we can't read dimensions, try anyway
                }
            };

            if !dimensions_ok {
                // Skip this image entirely since it is too large to safely decode
                return Self::NotImage;
            }

            tried_supported_file = true;
            let dyn_img: Option<image::DynamicImage> = match mime.subtype().as_str() {
                "jxl" => match File::open(path) {
                    Ok(file) => match JxlDecoder::new(file) {
                        Ok(mut decoder) => {
                            let mut limits = image::Limits::default();
                            let max_ram = max_mem * 1000 * 1000 / jobs as u64;
                            limits.max_alloc = Some(max_ram);
                            let _ = decoder.set_limits(limits);
                            match image::DynamicImage::from_decoder(decoder) {
                                Ok(img) => Some(img),
                                Err(err) => {
                                    log::warn!("failed to decode jxl {}: {}", path.display(), err);
                                    None
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("failed to create jxl decoder {}: {}", path.display(), err);
                            None
                        }
                    },
                    Err(err) => {
                        log::warn!("failed to open path {}: {}", path.display(), err);
                        None
                    }
                },
                _ => {
                    match image::ImageReader::open(path)
                        .and_then(image::ImageReader::with_guessed_format)
                    {
                        Ok(mut reader) => {
                            let mut limits = image::Limits::default();
                            let max_ram = max_mem * 1000 * 1000 / jobs as u64;
                            limits.max_alloc = Some(max_ram);
                            reader.limits(limits);
                            match reader.decode() {
                                Ok(reader) => Some(reader),
                                Err(err) => {
                                    log::warn!("failed to decode {}: {}", path.display(), err);
                                    None
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("failed to read {}: {}", path.display(), err);
                            None
                        }
                    }
                }
            };

            if let Some(dyn_img) = dyn_img {
                let (img_width, img_height) = (dyn_img.width(), dyn_img.height());

                if let Ok(cacher) = thumbnail_cacher.as_ref() {
                    match cacher.update_with_image(dyn_img) {
                        Ok(thumb_path) => {
                            return Self::Image(
                                widget::image::Handle::from_path(thumb_path),
                                Some((img_width, img_height)),
                            );
                        }
                        Err(err) => {
                            log::warn!("cacher failed to decode {}: {}", path.display(), err);
                        }
                    }
                } else {
                    // Fallback for when thumbnail cacher isn't available.
                    let thumbnail = dyn_img
                        .thumbnail(thumbnail_size, thumbnail_size)
                        .into_rgba8();
                    return Self::Image(
                        widget::image::Handle::from_rgba(
                            thumbnail.width(),
                            thumbnail.height(),
                            thumbnail.into_raw(),
                        ),
                        Some((img_width, img_height)),
                    );
                }
            }
        }

        // Try external thumbnailers.
        let thumbnail_dir = thumbnail_cacher
            .as_ref()
            .ok()
            .map(ThumbnailCacher::thumbnail_dir);
        if let Some((item_thumbnail, temp_file)) =
            Self::generate_thumbnail_external(path, &mime, thumbnail_size, thumbnail_dir)
        {
            if let Ok(cache) = thumbnail_cacher
                && let Err(err) = cache.update_with_temp_file(temp_file)
            {
                log::warn!("failed to update cache for {}: {}", path.display(), err);
            }
            return item_thumbnail;
        }

        tried_supported_file = tried_supported_file || !thumbnailer(&mime).is_empty();

        // Try internal thumbnailers that don't get cached.
        //TODO: adjust limits for internal thumbnailers as desired
        if mime.type_() == mime::IMAGE
            && mime.subtype() == mime::SVG
            && check_size("svg", 8 * 1000 * 1000)
        {
            tried_supported_file = true;
            // Try built-in svg thumbnailer
            match fs::read(path) {
                Ok(data) => {
                    //TODO: validate SVG data
                    return Self::Svg(widget::svg::Handle::from_memory(data));
                }
                Err(err) => {
                    log::warn!("failed to read {}: {}", path.display(), err);
                }
            }
        } else if mime.type_() == mime::TEXT && check_size("text", 8 * 1000 * 1000) {
            /*TODO: fix performance issues, widget::text_editor::Content::with_text forces all text to shape, which blocks rendering
            match fs::read_to_string(&path) {
                Ok(data) => {
                    return ItemThumbnail::Text(widget::text_editor::Content::with_text(&data));
                }
                Err(err) => {
                    log::warn!("failed to read {}: {}", path.display(), err);
                }
            }
            */
        }

        // If we weren't able to create a thumbnail, but we should have
        // been able to, create a fail marker so that it isn't tried the
        // next time.
        if let Ok(cacher) = thumbnail_cacher
            && tried_supported_file
            && let Err(err) = cacher.create_fail_marker()
        {
            log::warn!(
                "failed to create thumbnail fail marker for {}: {}",
                path.display(),
                err
            );
        }

        Self::NotImage
    }

    fn generate_thumbnail_external(
        path: &Path,
        mime: &mime::Mime,
        thumbnail_size: u32,
        thumbnail_dir: Option<&Path>,
    ) -> Option<(Self, NamedTempFile)> {
        // Try external thumbnailers
        for thumbnailer in thumbnailer(mime) {
            let is_evince = thumbnailer.exec.starts_with("evince-thumbnailer ");
            let prefix = if is_evince {
                //TODO: apparmor config for evince-thumbnailer does not allow /tmp/cosmic-files*
                "gnome-desktop-"
            } else {
                "cosmic-files-"
            };

            // It's preferable to create the tempfile in the same directory as the final cached
            // thumbnail to ensure that no copies across filesytems need to be made. However,
            // the apparmor config for evince-thumbnailer does not allow this, so we need to
            // fallback to the system tempdir.
            let dir = if is_evince { None } else { thumbnail_dir };
            let file = match dir {
                Some(d) => tempfile::Builder::new().prefix(prefix).tempfile_in(d),
                None => tempfile::Builder::new().prefix(prefix).tempfile(),
            };
            let file = match file {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!(
                        "failed to create temporary file for thumbnail of {}: {}",
                        path.display(),
                        err
                    );
                    continue;
                }
            };

            let Some(mut command) = thumbnailer.command(path, file.path(), thumbnail_size) else {
                continue;
            };
            match command.status() {
                Ok(status) => {
                    if status.success() {
                        match image::ImageReader::open(file.path())
                            .and_then(ImageReader::with_guessed_format)
                        {
                            Ok(reader) => match reader.decode().map(DynamicImage::into_rgba8) {
                                Ok(image) => {
                                    return Some((
                                        Self::Image(
                                            widget::image::Handle::from_rgba(
                                                image.width(),
                                                image.height(),
                                                image.into_raw(),
                                            ),
                                            None,
                                        ),
                                        file,
                                    ));
                                }
                                Err(err) => {
                                    log::warn!("failed to decode {}: {}", path.display(), err);
                                }
                            },
                            Err(err) => {
                                log::warn!("failed to read {}: {}", path.display(), err);
                            }
                        }
                    } else {
                        log::warn!(
                            "failed to run {:?} for {}: {}",
                            thumbnailer,
                            path.display(),
                            status
                        );
                    }
                }
                Err(err) => {
                    log::warn!(
                        "failed to run {thumbnailer:?} for {}: {}",
                        path.display(),
                        err
                    );
                }
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub is_mount_point: bool,
    pub display_name: String,
    pub metadata: ItemMetadata,
    pub hidden: bool,
    pub location_opt: Option<Location>,
    pub mime: Mime,
    pub icon_handle_grid: widget::icon::Handle,
    pub icon_handle_list: widget::icon::Handle,
    pub icon_handle_list_condensed: widget::icon::Handle,
    pub thumbnail_opt: Option<ItemThumbnail>,
    pub button_id: widget::Id,
    pub pos_opt: Cell<Option<(usize, usize)>>,
    pub rect_opt: Cell<Option<Rectangle>>,
    pub selected: bool,
    pub highlighted: bool,
    pub cut: bool,
    pub overlaps_drag_rect: bool,
    pub dir_size: DirSize,
}

impl Item {
    fn display_name(name: &str) -> String {
        // In order to wrap at periods and underscores, add a zero width space after each one
        name.replace('.', ".\u{200B}").replace('_', "_\u{200B}")
    }

    pub fn path_opt(&self) -> Option<&PathBuf> {
        self.location_opt.as_ref()?.path_opt()
    }

    pub fn can_gallery(&self) -> bool {
        self.mime.type_() == mime::IMAGE || self.mime.type_() == mime::TEXT
    }

    pub fn file_metadata(&self) -> Option<Metadata> {
        match &self.metadata {
            ItemMetadata::Path { metadata, .. } => Some(metadata.clone()),
            #[cfg(feature = "gvfs")]
            ItemMetadata::GvfsPath { .. } => self.path_opt().and_then(|p| fs::metadata(p).ok()),
            _ => {
                //TODO: other metadata types
                None
            }
        }
    }

    fn preview(&self) -> Element<'_, Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        // This loads the image only if thumbnailing worked
        let icon = widget::icon::icon(self.icon_handle_grid.clone())
            .content_fit(ContentFit::Contain)
            .size(IconSizes::default().grid())
            .into();
        match self
            .thumbnail_opt
            .as_ref()
            .unwrap_or(&ItemThumbnail::NotImage)
        {
            ItemThumbnail::NotImage => icon,
            ItemThumbnail::Image(handle, _original_dims) => {
                // Preview pane: ALWAYS show thumbnail for instant, responsive UI
                // Full resolution loading happens in gallery mode
                widget::image(handle.clone()).into()
            }
            ItemThumbnail::Svg(handle) => widget::svg(handle.clone()).into(),
            ItemThumbnail::Text(content) => widget::text_editor(content)
                .class(cosmic::theme::iced::TextEditor::Custom(Box::new(
                    text_editor_class,
                )))
                .width(THUMBNAIL_SIZE as f32)
                .height(Length::Fixed(THUMBNAIL_SIZE as f32))
                .padding(spacing.space_xxs)
                .into(),
        }
    }

    pub fn preview_actions(&self) -> Element<'_, Message> {
        let mut row = widget::row::with_capacity(3)
            .align_y(Alignment::Center)
            .spacing(theme::spacing().space_xxs)
            .push(
                widget::button::icon(widget::icon::from_name("go-previous-symbolic"))
                    .on_press(Message::ItemLeft),
            )
            .push(
                widget::button::icon(widget::icon::from_name("go-next-symbolic"))
                    .on_press(Message::ItemRight),
            );
        if self.can_gallery()
            && let Some(_path) = self.path_opt()
        {
            row = row.push(
                widget::button::icon(widget::icon::from_name("view-fullscreen-symbolic"))
                    .on_press(Message::Gallery(true)),
            );
        }
        row.into()
    }

    pub fn preview_view<'a>(
        &'a self,
        mime_app_cache_opt: Option<&'a mime_app::MimeAppCache>,
        military_time: bool,
    ) -> Element<'a, Message> {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_m,
            ..
        } = theme::active().cosmic().spacing;

        let mut column = widget::column().spacing(space_m);

        column = column.push(
            widget::container(self.preview())
                .center_x(Length::Fill)
                .max_height(THUMBNAIL_SIZE as f32),
        );

        let mut details = widget::column().spacing(space_xxxs);
        details = details.push(widget::text::heading(self.name.clone()));
        details = details.push(widget::text::body(fl!(
            "type",
            mime = self.mime.to_string()
        )));
        let mut settings = Vec::new();
        if let Some(mime_app_cache) = mime_app_cache_opt {
            let mime_apps = mime_app_cache.get(&self.mime);
            if !mime_apps.is_empty() {
                settings.push(
                    widget::settings::item::builder(fl!("open-with")).control(
                        Element::from(
                            widget::dropdown(
                                mime_apps,
                                mime_apps.iter().position(|x| x.is_default),
                                move |index| index,
                            )
                            .icons(Cow::Borrowed(mime_app_cache.icons(&self.mime))),
                        )
                        .map(|index| {
                            let mime_app = &mime_apps[index];
                            Message::SetOpenWith(self.mime.clone(), mime_app.id.clone())
                        }),
                    ),
                );
            }
        }

        if let Some(metadata) = self.file_metadata() {
            if metadata.is_dir() {
                if let Some(children) = self.metadata.children_count() {
                    details = details.push(widget::text::body(fl!("items", items = children)));
                }
                let size = match &self.dir_size {
                    DirSize::Calculating(_) => fl!("calculating"),
                    DirSize::Directory(size) => format_size(*size),
                    DirSize::NotDirectory => String::new(),
                    DirSize::Error(err) => err.clone(),
                };
                if !size.is_empty() {
                    details = details.push(widget::text::body(fl!("item-size", size = size)));
                }
            } else {
                details = details.push(widget::text::body(fl!(
                    "item-size",
                    size = format_size(metadata.len())
                )));
            }

            let date_time_formatter = date_time_formatter(military_time);
            let time_formatter = time_formatter(military_time);

            if let Ok(time) = metadata.created() {
                details = details.push(widget::text::body(fl!(
                    "item-created",
                    created = format_time(time, &date_time_formatter, &time_formatter).to_string()
                )));
            }

            if let Ok(time) = metadata.modified() {
                details = details.push(widget::text::body(fl!(
                    "item-modified",
                    modified = format_time(time, &date_time_formatter, &time_formatter).to_string()
                )));
            }

            if let Ok(time) = metadata.accessed() {
                details = details.push(widget::text::body(fl!(
                    "item-accessed",
                    accessed = format_time(time, &date_time_formatter, &time_formatter).to_string()
                )));
            }

            #[cfg(unix)]
            if let Some(path) = self.path_opt() {
                use std::os::unix::fs::MetadataExt;

                let mode = metadata.mode();

                let user_name = get_user_by_uid(metadata.uid())
                    .and_then(|user| user.name().to_str().map(ToOwned::to_owned))
                    .unwrap_or_default();
                let user_path = path.clone();
                settings.push(
                    widget::settings::item::builder(user_name)
                        .description(fl!("owner"))
                        .control(widget::dropdown(
                            Cow::Borrowed(MODE_NAMES.as_slice()),
                            Some(get_mode_part(mode, MODE_SHIFT_USER).try_into().unwrap()),
                            move |selected| {
                                Message::SetPermissions(
                                    user_path.clone(),
                                    set_mode_part(
                                        mode,
                                        MODE_SHIFT_USER,
                                        selected.try_into().unwrap(),
                                    ),
                                )
                            },
                        )),
                );

                let group_name = get_group_by_gid(metadata.gid())
                    .and_then(|group| group.name().to_str().map(ToOwned::to_owned))
                    .unwrap_or_default();
                let group_path = path.clone();
                settings.push(
                    widget::settings::item::builder(group_name)
                        .description(fl!("group"))
                        .control(widget::dropdown(
                            Cow::Borrowed(MODE_NAMES.as_slice()),
                            Some(get_mode_part(mode, MODE_SHIFT_GROUP).try_into().unwrap()),
                            move |selected| {
                                Message::SetPermissions(
                                    group_path.clone(),
                                    set_mode_part(
                                        mode,
                                        MODE_SHIFT_GROUP,
                                        selected.try_into().unwrap(),
                                    ),
                                )
                            },
                        )),
                );

                let other_path = path.clone();
                settings.push(widget::settings::item::builder(fl!("other")).control(
                    widget::dropdown(
                        Cow::Borrowed(MODE_NAMES.as_slice()),
                        Some(get_mode_part(mode, MODE_SHIFT_OTHER).try_into().unwrap()),
                        move |selected| {
                            Message::SetPermissions(
                                other_path.clone(),
                                set_mode_part(mode, MODE_SHIFT_OTHER, selected.try_into().unwrap()),
                            )
                        },
                    ),
                ));
            }
        }

        if let Some(path) = self.path_opt()
            && let Ok(img) = image::image_dimensions(path)
        {
            let (width, height) = img;
            details = details.push(widget::text::body(format!("{width}x{height}")));
        }
        column = column.push(details);

        if let Some(path) = self.path_opt() {
            if self.selected {
                column = column.push(
                    widget::button::standard(fl!("open")).on_press(Message::Open(Some(path.clone()))),
                );
            }
        }

        if !settings.is_empty() {
            let mut section = widget::settings::section();
            section = section.extend(settings);
            column = column.push(section);
        }

        column.into()
    }

    pub fn replace_view(&self, heading: String, military_time: bool) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::active().cosmic().spacing;

        let mut row = widget::row().spacing(space_xxxs);
        row = row.push(self.preview());

        let mut column = widget::column().spacing(space_xxxs);
        column = column.push(widget::text::heading(heading));

        //TODO: translate!
        //TODO: correct display of folder size?
        if let ItemMetadata::Path {
            metadata,
            children_opt,
        } = &self.metadata
        {
            if metadata.is_dir() {
                if let Some(children) = children_opt {
                    column = column.push(widget::text::body(format!("Items: {children}")));
                }
            } else {
                column = column.push(widget::text::body(format!(
                    "Size: {}",
                    format_size(metadata.len())
                )));
            }
            if let Ok(time) = metadata.modified() {
                let date_time_formatter = date_time_formatter(military_time);
                let time_formatter = time_formatter(military_time);

                column = column.push(widget::text::body(format!(
                    "Last modified: {}",
                    format_time(time, &date_time_formatter, &time_formatter)
                )));
            }
        } else {
            //TODO: other metadata
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
            Self::Name => write!(f, "{}", fl!("name")),
            Self::Modified => write!(f, "{}", fl!("modified")),
            Self::Size => write!(f, "{}", fl!("size")),
            Self::TrashedOn => write!(f, "{}", fl!("trashed-on")),
        }
    }
}

impl HeadingOptions {
    pub fn names() -> Vec<String> {
        vec![
            Self::Name.to_string(),
            Self::Modified.to_string(),
            Self::Size.to_string(),
            Self::TrashedOn.to_string(),
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
            Self::App | Self::Desktop => true,
            Self::Dialog(dialog) => dialog.multiple(),
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
    pub location_ancestors: Vec<(Location, String)>,
    pub location_title: String,
    pub location_context_menu_point: Option<Point>,
    pub location_context_menu_index: Option<usize>,
    pub context_menu: Option<Point>,
    pub mode: Mode,
    pub scroll_opt: Option<AbsoluteOffset>,
    pub size_opt: Cell<Option<Size>>,
    pub content_height_opt: Cell<Option<f32>>,
    pub viewport_opt: Option<Rectangle>,
    pub item_view_size_opt: Cell<Option<Size>>,
    pub edit_location: Option<EditLocation>,
    pub edit_location_id: widget::Id,
    pub history_i: usize,
    pub history: Vec<Location>,
    pub config: TabConfig,
    pub thumb_config: ThumbCfg,
    pub sort_name: HeadingOptions,
    pub sort_direction: bool,
    pub gallery: bool,
    pub(crate) parent_item_opt: Option<Item>,
    pub(crate) items_opt: Option<Vec<Item>>,
    pub dnd_hovered: Option<(Location, Instant)>,
    pub(crate) scrollable_id: widget::Id,
    select_focus: Option<usize>,
    select_range: Option<(usize, usize)>,
    clicked: Option<usize>,
    selected_clicked: bool,
    last_right_click: Option<usize>,
    search_context: Option<SearchContext>,
    date_time_formatter: DateTimeFormatter<fieldsets::YMDT>,
    time_formatter: DateTimeFormatter<fieldsets::T>,
    watch_drag: bool,
    window_id: Option<window::Id>,
    large_image_manager: LargeImageManager,
}

async fn calculate_dir_size(path: &Path, controller: Controller) -> Result<u64, OperationError> {
    let mut total = 0;
    for entry_res in WalkDir::new(path) {
        controller
            .check()
            .await
            .map_err(|s| OperationError::from_state(s, &controller))?;

        //TODO: report more errors?
        if let Ok(entry) = entry_res
            && let Ok(metadata) = entry.metadata()
            && metadata.is_file()
        {
            total += metadata.len();
        }

        // Yield in case this process takes a while.
        tokio::task::yield_now().await;
    }
    Ok(total)
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
                match (get_filename_from_path(path), fs::metadata(path)) {
                    (Ok(name), Ok(metadata)) => {
                        let is_gvfs = fs_kind(&metadata) == FsKind::Gvfs;
                        display_name_for_file(path, &name, is_gvfs, false)
                    }
                    _ => name.to_string_lossy().into_owned(),
                }
            }
        }
        None => {
            fl!("filesystem")
        }
    };
    (name, found_home)
}

// parse .hidden file and return files path
pub fn parse_hidden_file(path: &PathBuf) -> Box<[String]> {
    let Ok(file) = File::open(path) else {
        return Default::default();
    };

    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| {
            let line = line.trim();
            (!line.is_empty()).then_some(line.to_owned())
        })
        .collect()
}

impl Tab {
    pub fn new(
        location: Location,
        config: TabConfig,
        thumb_config: ThumbCfg,
        sorting_options: Option<&FxOrderMap<String, (HeadingOptions, bool)>>,
        scrollable_id: widget::Id,
        window_id: Option<window::Id>,
    ) -> Self {
        let location_str = location.to_string();
        let (sort_name, sort_direction) = sorting_options
            .and_then(|opts| opts.get(&location_str))
            .or_else(|| SORT_OPTION_FALLBACK.get(&location_str))
            .copied()
            .unwrap_or((HeadingOptions::Name, true));
        let location = location.normalize();
        let location_ancestors = location.ancestors();
        let location_title = location.title();
        let history = vec![location.clone()];
        Self {
            location,
            location_ancestors,
            location_title,
            context_menu: None,
            location_context_menu_point: None,
            location_context_menu_index: None,
            mode: Mode::App,
            scroll_opt: None,
            size_opt: Cell::new(None),
            content_height_opt: Cell::new(None),
            viewport_opt: None,
            item_view_size_opt: Cell::new(None),
            edit_location: None,
            edit_location_id: widget::Id::unique(),
            history_i: 0,
            history,
            config,
            thumb_config,
            sort_name,
            sort_direction,
            gallery: false,
            parent_item_opt: None,
            items_opt: None,
            scrollable_id,
            select_focus: None,
            select_range: None,
            clicked: None,
            dnd_hovered: None,
            selected_clicked: false,
            last_right_click: None,
            search_context: None,
            date_time_formatter: date_time_formatter(config.military_time),
            time_formatter: time_formatter(config.military_time),
            watch_drag: true,
            window_id,
            large_image_manager: LargeImageManager::new(),
        }
    }

    pub fn title(&self) -> String {
        //TODO: is it possible to return a &str?
        self.location_title.clone()
    }

    pub const fn items_opt(&self) -> Option<&Vec<Item>> {
        self.items_opt.as_ref()
    }

    pub const fn items_opt_mut(&mut self) -> Option<&mut Vec<Item>> {
        self.items_opt.as_mut()
    }

    pub fn set_items(&mut self, mut items: Vec<Item>) {
        let selected = self.selected_locations();
        for item in &mut items {
            item.selected = false;
            if let Some(location) = &item.location_opt
                && selected.contains(location)
            {
                item.selected = true;
            }
        }
        self.items_opt = Some(items);
    }

    pub fn cut_selected(&mut self) {
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                item.cut = item.selected;
            }
        }
    }

    pub fn refresh_cut(&mut self, locations: &[PathBuf]) {
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                item.cut = false;
                if let Some(location_path) = item.location_opt.as_ref().and_then(Location::path_opt)
                    && locations.contains(location_path)
                {
                    item.cut = true;
                }
            }
        }
    }

    pub fn selected_locations(&self) -> Vec<Location> {
        if let Some(ref items) = self.items_opt {
            items
                .iter()
                .filter_map(|item| {
                    if item.selected {
                        item.location_opt.clone()
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn select_all(&mut self) {
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
        self.select_focus = None;
        if let Some(ref mut items) = self.items_opt {
            for (i, item) in items.iter_mut().enumerate() {
                item.selected = item.name == name;
                if item.selected {
                    self.select_focus = Some(i);
                }
            }
        }
    }

    /// Selects the first item whose name starts with the given prefix (case-insensitive).
    /// Returns true if an item was selected.
    pub fn select_by_prefix(&mut self, prefix: &str) -> bool {
        let prefix_lower = prefix.to_lowercase();
        self.select_focus = None;

        if let Some(ref mut items) = self.items_opt {
            // First, deselect all items
            for item in items.iter_mut() {
                item.selected = false;
            }

            // Find first matching item
            for (i, item) in items.iter_mut().enumerate() {
                if item.name.to_lowercase().starts_with(&prefix_lower) {
                    item.selected = true;
                    self.select_focus = Some(i);
                    return true;
                }
            }
        }
        false
    }

    pub fn select_paths(&mut self, paths: Vec<PathBuf>) {
        self.select_focus = None;
        if let Some(ref mut items) = self.items_opt {
            for (i, item) in items.iter_mut().enumerate() {
                item.selected = false;
                if let Some(path) = item.path_opt()
                    && paths.contains(path)
                {
                    item.selected = true;
                    self.select_focus = Some(i);
                }
            }
        }
    }

    fn select_position(&mut self, row: usize, col: usize, mod_shift: bool) -> bool {
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
        if let Some(ref mut items) = self.items_opt {
            for item in items.iter_mut() {
                let was_overlapped = item.overlaps_drag_rect;
                item.overlaps_drag_rect = item.rect_opt.get().is_some_and(|r| r.intersects(&rect));

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

    pub(crate) fn select_focus_scroll(&mut self) -> Option<AbsoluteOffset> {
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
        let items = self.items_opt.as_ref()?;
        let mut first = None;
        for item in items {
            if !item.selected {
                continue;
            }

            let (row, col) = match item.pos_opt.get() {
                Some(some) => some,
                None => continue,
            };

            first = Some(match first {
                Some((first_row, first_col)) => match row.cmp(&first_row) {
                    Ordering::Less => (row, col),
                    Ordering::Equal => (row, col.min(first_row)),
                    Ordering::Greater => (first_row, first_col),
                },
                None => (row, col),
            });
        }
        first
    }

    fn select_last_pos_opt(&self) -> Option<(usize, usize)> {
        let items = self.items_opt.as_ref()?;
        let mut last = None;
        for item in items {
            if !item.selected {
                continue;
            }

            let (row, col) = match item.pos_opt.get() {
                Some(some) => some,
                None => continue,
            };

            last = Some(match last {
                Some((last_row, last_col)) => match row.cmp(&last_row) {
                    Ordering::Greater => (row, col),
                    Ordering::Equal => (row, col.max(last_row)),
                    Ordering::Less => (last_row, last_col),
                },
                None => (row, col),
            });
        }
        last
    }

    fn trigger_async_decode(&mut self) -> Vec<Command> {
        // Only trigger decode in gallery mode for the currently selected image
        if !self.gallery {
            return Vec::new();
        }

        let Some(index) = self.select_focus else {
            return Vec::new();
        };

        let Some(items) = &self.items_opt else {
            return Vec::new();
        };

        let Some(item) = items.get(index) else {
            return Vec::new();
        };

        let Some(ItemThumbnail::Image(_, original_dims)) = &item.thumbnail_opt else {
            return Vec::new();
        };

        if let Some((w, h)) = original_dims
            && !should_use_tiling(*w, *h)
        {
            return Vec::new();
        }

        let Some(path) = item.path_opt() else {
            return Vec::new();
        };

        // Clone path to avoid borrow checker issues
        let path = path.to_path_buf();

        // Get display size for adaptive resolution
        let display_dimensions = self
            .size_opt
            .get()
            .map(|size| (size.width as u32, size.height as u32));

        // Try to decode the image using LargeImageManager with adaptive resolution
        let (should_decode, target_dimensions, generation) = self
            .large_image_manager
            .try_decode(&path, display_dimensions);
        if should_decode {
            vec![Command::Iced(
                cosmic::iced::Task::perform(
                    decode_large_image(path, target_dimensions),
                    move |result| {
                        result
                            .map(|(path, width, height, pixels)| {
                                Message::ImageDecoded(
                                    path,
                                    width,
                                    height,
                                    pixels,
                                    display_dimensions,
                                    generation,
                                )
                            })
                            .unwrap_or_else(|| Message::AutoScroll(None))
                    },
                )
                .into(),
            )]
        } else {
            Vec::new()
        }
    }

    pub fn change_location(&mut self, location: &Location, history_i_opt: Option<usize>) {
        self.location = location.normalize();
        self.location_ancestors = self.location.ancestors();
        self.location_title = self.location.title();
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

            // Compact consecutive matching paths
            {
                let mut remove = false;
                if let Some(last_location) = self.history.last() {
                    if let Location::Network(last_uri, ..) = last_location
                        && let Location::Network(uri, ..) = location
                    {
                        remove = last_uri == uri;
                    } else if let Some(last_path) = last_location.path_opt()
                        && let Some(path) = location.path_opt()
                    {
                        remove = last_path == path;
                    }
                }
                if remove {
                    self.history.pop();
                }
            }

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
        let last_context_menu = self.context_menu;
        match message {
            Message::AddNetworkDrive => {
                commands.push(Command::AddNetworkDrive);
            }
            Message::AutoScroll(auto_scroll) => {
                commands.push(Command::AutoScroll(auto_scroll));
            }
            Message::ClickRelease(click_i_opt) => {
                // Single click to open.
                if !mod_ctrl && self.config.single_click {
                    let mut paths_to_open = Vec::new();
                    if let Some(ref mut items) = self.items_opt {
                        for (i, item) in items.iter_mut().enumerate() {
                            if Some(i) == click_i_opt {
                                if let Some(location) = &item.location_opt {
                                    if item.metadata.is_dir() {
                                        cd = Some(location.clone());
                                    } else if let Some(path) = location.path_opt() {
                                        paths_to_open.push(path.clone());
                                    } else {
                                        log::warn!("no path for item {item:?}");
                                    }
                                } else {
                                    log::warn!("no location for item {item:?}");
                                }
                            }
                        }
                    }
                    if !paths_to_open.is_empty() {
                        commands.push(Command::OpenFile(paths_to_open));
                    }
                }

                if click_i_opt != self.clicked.take() {
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
            }
            Message::DragEnd => {
                self.clicked = None;
                self.watch_drag = true;
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
                        } else if let Some(path) = location.path_opt() {
                            commands.push(Command::OpenFile(vec![path.clone()]));
                        } else {
                            log::warn!("no path for item {clicked_item:?}");
                        }
                    } else {
                        log::warn!("no location for item {clicked_item:?}");
                    }
                } else {
                    log::warn!("no item for click index {click_i_opt:?}");
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
                            let range_min = range.0.min(range.1);
                            let range_max = range.0.max(range.1);
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
                                        .map(<[Item]>::len)
                                        .unwrap_or_default();
                                    (0..len).collect()
                                });

                            // Find the true indices for the min and max element w.r.t.
                            // a sorted tab.
                            let min = indices
                                .iter()
                                .copied()
                                .position(|offset| offset == range_min)
                                .unwrap_or_default();
                            // We can't skip `min_real` elements here because the index of
                            // `max` may actually be before `min` in a sorted tab
                            let max = indices
                                .iter()
                                .copied()
                                .position(|offset| offset == range_max)
                                .unwrap_or(indices.len());
                            let min_real = min.min(max);
                            let max_real = max.max(min);

                            if let Some(ref mut items) = self.items_opt {
                                for index in indices
                                    .into_iter()
                                    .skip(min_real)
                                    .take(max_real - min_real + 1)
                                {
                                    if let Some(item) = items.get_mut(index) {
                                        if item.hidden {
                                            if self.config.show_hidden {
                                                item.selected = true;
                                            }
                                        } else {
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
                                .any(|&(e_i, e)| Some(e_i) == click_i_opt && e.selected)
                        });
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
                                }
                                self.select_range = Some((i, i));
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
                let military_time_changed = self.config.military_time != config.military_time;
                let show_hidden_changed = self.config.show_hidden != config.show_hidden;
                self.config = config;
                self.config.view = view;
                if military_time_changed {
                    self.date_time_formatter = date_time_formatter(self.config.military_time);
                    self.time_formatter = time_formatter(self.config.military_time);
                }
                if show_hidden_changed && let Location::Search(path, term, ..) = &self.location {
                    cd = Some(Location::Search(
                        path.clone(),
                        term.clone(),
                        self.config.show_hidden,
                        Instant::now(),
                    ));
                }
                // Unhighlight all items when config changes
                if let Some(ref mut items) = self.items_opt {
                    for item in items.iter_mut() {
                        item.highlighted = false;
                    }
                }
            }
            Message::ContextAction(action) => {
                // Close context menu
                self.context_menu = None;

                commands.push(Command::Action(action));
            }
            Message::ContextMenu(point_opt, _) => {
                self.edit_location = None;
                self.context_menu = point_opt;
                self.location_context_menu_index = None;

                //TODO: hack for clearing selecting when right clicking empty space
                if self.context_menu.is_some()
                    && self.last_right_click.take().is_none()
                    && let Some(ref mut items) = self.items_opt
                {
                    for item in items.iter_mut() {
                        item.selected = false;
                    }
                }
            }
            Message::LocationContextMenuPoint(point_opt) => {
                self.context_menu = None;
                self.location_context_menu_point = point_opt;
            }
            Message::LocationContextMenuIndex(p, index_opt) => {
                self.context_menu = None;
                self.location_context_menu_point = p;
                self.location_context_menu_index = index_opt;
            }
            Message::LocationMenuAction(action) => {
                self.location_context_menu_index = None;
                let path_for_index = |ancestor_index| {
                    self.location
                        .path_opt()
                        .and_then(|path| path.ancestors().nth(ancestor_index))
                        .map(Path::to_path_buf)
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
                                    log::warn!(
                                        "failed to get item from path {}: {}",
                                        path.display(),
                                        err
                                    );
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
            Message::Drag(rect_opt) => {
                self.watch_drag = false;
                if let Some(rect) = rect_opt {
                    self.context_menu = None;
                    self.location_context_menu_index = None;
                    if self.mode.multiple() {
                        self.select_rect(rect, mod_ctrl, mod_shift);
                    }
                    if self.select_focus.take().is_some() {
                        // Unfocus currently focused button
                        commands.push(Command::Iced(
                            widget::button::focus(widget::Id::unique()).into(),
                        ));
                    }
                }
            }
            Message::EditLocation(edit_location) => {
                self.edit_location = edit_location;
                if self.edit_location.is_some() {
                    commands.push(Command::Iced(
                        widget::text_input::focus(self.edit_location_id.clone()).into(),
                    ));
                }
            }
            Message::EditLocationComplete(selected) => {
                if let Some(mut edit_location) = self.edit_location.take()
                    && !matches!(edit_location.location, Location::Network(..))
                {
                    edit_location.selected = Some(selected);
                    cd = edit_location.resolve();
                }
            }
            Message::EditLocationEnable => {
                commands.push(Command::Iced(
                    widget::text_input::focus(self.edit_location_id.clone()).into(),
                ));
                self.edit_location = Some(self.location.clone().into());
            }
            Message::EditLocationSubmit => {
                if let Some(mut edit_location) = self.edit_location.take() {
                    // Select first completion if current location does not exist
                    if edit_location.selected.is_none()
                        && edit_location
                            .completions
                            .as_ref()
                            .is_some_and(|completions| !completions.is_empty())
                        && edit_location
                            .location
                            .path_opt()
                            .is_some_and(|path| !path.exists())
                    {
                        edit_location.selected = Some(0);
                    }

                    cd = edit_location.resolve();
                }
            }
            Message::EditLocationTab => {
                if let Some(edit_location) = &mut self.edit_location {
                    edit_location.select(!modifiers.contains(Modifiers::SHIFT));
                }
            }
            Message::OpenInNewTab(path) => {
                commands.push(Command::OpenInNewTab(path));
            }
            Message::EmptyTrash => {
                commands.push(Command::EmptyTrash);
            }
            #[cfg(feature = "desktop")]
            Message::ExecEntryAction(path, action) => {
                let lang_id = crate::localize::LANGUAGE_LOADER.current_language();
                let language = lang_id.language.as_str();
                match path.map_or_else(
                    || {
                        let items = self.items_opt.as_deref()?;
                        items.iter().find(|&item| item.selected).and_then(|item| {
                            let location = item.location_opt.as_ref()?;
                            let path = location.path_opt()?;
                            cosmic::desktop::load_desktop_file(&[language.into()], path.into())
                        })
                    },
                    |path| cosmic::desktop::load_desktop_file(&[language.into()], path),
                ) {
                    Some(entry) => commands.push(Command::ExecEntryAction(entry, action)),
                    None => log::warn!("Invalid desktop entry path passed to ExecEntryAction"),
                }
            }
            Message::Gallery(gallery) => {
                self.gallery = gallery;

                if gallery {
                    commands.extend(self.trigger_async_decode());
                }
            }
            Message::GalleryPrevious | Message::GalleryNext => {
                let mut pos_opt = None;
                if let Some(mut indices) = self.column_sort() {
                    if matches!(message, Message::GalleryPrevious) {
                        indices.reverse();
                    }
                    let mut found = false;
                    for (index, item) in indices {
                        if self.select_focus.is_none() {
                            found = true;
                        }
                        if self.select_focus == Some(index) {
                            found = true;
                            continue;
                        }
                        if found && item.can_gallery() {
                            pos_opt = item.pos_opt.get();
                            if pos_opt.is_some() {
                                break;
                            }
                        }
                    }
                }
                if let Some((row, col)) = pos_opt {
                    // Should mod_shift be available?
                    self.select_position(row, col, mod_shift);

                    commands.extend(self.trigger_async_decode());
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
                    for (_, item) in &indices {
                        if item.selected && item.can_gallery() {
                            self.gallery = !self.gallery;

                            if self.gallery {
                                commands.extend(self.trigger_async_decode());
                            }
                            break;
                        }
                    }
                }
            }
            Message::GoNext => {
                if let Some(history_i) = self.history_i.checked_add(1)
                    && let Some(location) = self.history.get(history_i)
                {
                    cd = Some(location.clone());
                    history_i_opt = Some(history_i);
                }
            }
            Message::GoPrevious => {
                if let Some(history_i) = self.history_i.checked_sub(1)
                    && let Some(location) = self.history.get(history_i)
                {
                    cd = Some(location.clone());
                    history_i_opt = Some(history_i);
                }
            }
            Message::ItemDown => {
                if let Some(edit_location) = &mut self.edit_location {
                    edit_location.select(true);
                } else if self.gallery {
                    commands.append(&mut self.update(Message::GalleryNext, modifiers));
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
                    commands.append(&mut self.update(Message::GalleryPrevious, modifiers));
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
                            .is_some_and(|col| self.select_position(row, col, mod_shift))
                        {
                            // Try to select last item in previous row
                            if !row.checked_sub(1).is_some_and(|row| {
                                let mut col = 0;
                                if let Some(ref items) = self.items_opt {
                                    for item in items {
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
                    commands.append(&mut self.update(Message::GalleryNext, modifiers));
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
                if let Some(edit_location) = &mut self.edit_location {
                    edit_location.select(false);
                } else if self.gallery {
                    commands.append(&mut self.update(Message::GalleryPrevious, modifiers));
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
                            .is_some_and(|row| self.select_position(row, col, mod_shift))
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
                            commands.push(Command::OpenFile(vec![path.clone()]));
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
                if let Location::Path(ref path) = self.location
                    && let Some(parent) = path.parent()
                {
                    cd = Some(Location::Path(parent.to_owned()));
                }
            }
            Message::Open(path_opt) => {
                match path_opt {
                    Some(path) => {
                        if path.is_dir() {
                            cd = Some(Location::Path(path));
                        } else {
                            commands.push(Command::OpenFile(vec![path]));
                        }
                    }
                    // Open selected items
                    None => {
                        enum ResolveResult {
                            Open(Option<PathBuf>),
                            OpenInTab(Option<PathBuf>),
                            OpenTrash,
                            OpenProperties,
                            Cd(Location),
                            Skip,
                        }
                        fn resolve_item(
                            item: &Item,
                            mode: &Mode,
                            is_only_one_selected: bool,
                        ) -> ResolveResult {
                            if !item.selected {
                                return ResolveResult::Skip;
                            }

                            let location = match &item.location_opt {
                                Some(l) => l,
                                None => return ResolveResult::OpenProperties,
                            };

                            let path_opt = location.path_opt();

                            if item.metadata.is_dir() {
                                match mode {
                                    Mode::App => {
                                        if is_only_one_selected {
                                            ResolveResult::Cd(location.clone())
                                        } else {
                                            ResolveResult::OpenInTab(path_opt.cloned())
                                        }
                                    }
                                    Mode::Desktop => match location {
                                        Location::Trash => ResolveResult::OpenTrash,
                                        _ => ResolveResult::Open(path_opt.cloned()),
                                    },
                                    Mode::Dialog(_) => {
                                        if is_only_one_selected {
                                            ResolveResult::Cd(location.clone())
                                        } else {
                                            ResolveResult::Skip
                                        }
                                    }
                                }
                            } else {
                                ResolveResult::Open(path_opt.cloned())
                            }
                        }
                        let mut open_files = Vec::new();
                        if let Some(items) = self.items_opt.as_ref() {
                            let selected_count = items.iter().filter(|i| i.selected).count();

                            for item in items.iter() {
                                match resolve_item(item, &self.mode, selected_count == 1) {
                                    ResolveResult::Open(Some(p)) => open_files.push(p),
                                    ResolveResult::OpenInTab(Some(p)) => {
                                        commands.push(Command::OpenInNewTab(p))
                                    }
                                    ResolveResult::Cd(loc) => cd = Some(loc),
                                    ResolveResult::OpenTrash => commands.push(Command::OpenTrash),
                                    ResolveResult::OpenProperties => {} //TODO: open properties?
                                    _ => {}
                                }
                            }
                        }
                        if !open_files.is_empty() {
                            commands.push(Command::OpenFile(open_files));
                        }
                    }
                }
            }
            Message::Reload => {
                //TODO: support keeping selected locations without paths
                let selected_paths = self
                    .selected_locations()
                    .into_iter()
                    .filter_map(Location::into_path_opt)
                    .collect();
                let location = self.location.clone();
                self.change_location(&location, None);
                commands.push(Command::ChangeLocation(
                    self.title(),
                    location,
                    Some(selected_paths),
                ));
            }
            Message::RightClick(_point_opt, click_i_opt) => {
                if mod_ctrl || mod_shift {
                    self.update(Message::Click(click_i_opt), modifiers);
                }
                if let Some(ref mut items) = self.items_opt
                    && !click_i_opt
                        .is_some_and(|click_i| items.get(click_i).is_some_and(|x| x.selected))
                {
                    // If item not selected, clear selection on other items
                    for (i, item) in items.iter_mut().enumerate() {
                        item.selected = Some(i) == click_i_opt;
                    }
                }
                //TODO: hack for clearing selecting when right clicking empty space
                self.last_right_click = click_i_opt;
            }
            Message::MiddleClick(click_i) => {
                if mod_ctrl || mod_shift {
                    self.update(Message::Click(Some(click_i)), modifiers);
                } else {
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
                                commands.push(Command::OpenInNewTab(path.clone()));
                            } else {
                                commands.push(Command::OpenFile(vec![path.clone()]));
                            }
                        } else {
                            log::warn!("no path for item {clicked_item:?}");
                        }
                    } else {
                        log::warn!("no item for click index {click_i:?}");
                    }
                }
            }
            Message::HighlightDeactivate(i) => {
                self.watch_drag = true;
                if let Some(item) = self.items_opt.as_mut().and_then(|f| f.get_mut(i)) {
                    item.highlighted = false;
                }
            }
            Message::HighlightActivate(i) => {
                self.watch_drag = true;
                if let Some(item) = self.items_opt.as_mut().and_then(|f| f.get_mut(i)) {
                    item.highlighted = true;
                }
            }
            Message::Resize(viewport) => {
                // Scroll to ensure focused item still in view
                if self.viewport_opt.map(|v| v.size()) != Some(viewport.size())
                    && let Some(offset) = self.select_focus_scroll()
                {
                    commands.push(Command::Iced(
                        scrollable::scroll_to(self.scrollable_id.clone(), offset).into(),
                    ));
                }

                self.viewport_opt = Some(viewport);
            }
            Message::Scroll(viewport) => {
                self.scroll_opt = Some(viewport.absolute_offset());
                self.watch_drag = true;
            }
            Message::ScrollTab(scroll_speed) => {
                commands.push(Command::Iced(
                    scrollable::scroll_by(
                        self.scrollable_id.clone(),
                        AbsoluteOffset {
                            x: 0.0,
                            y: scroll_speed,
                        },
                    )
                    .into(),
                ));
            }
            Message::ScrollToFocused => {
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
                            while let Ok((path, name, metadata)) = context.results_rx.try_recv() {
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
            Message::SelectFirst => {
                if self.select_position(0, 0, mod_shift) {
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
            Message::SelectLast => {
                if let Some(ref items) = self.items_opt
                    && let Some(last_pos) = items.iter().filter_map(|item| item.pos_opt.get()).max()
                    && self.select_position(last_pos.0, last_pos.1, mod_shift)
                {
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
            Message::SetOpenWith(mime, id) => {
                commands.push(Command::SetOpenWith(mime, id));
            }
            Message::SetPermissions(path, mode) => {
                commands.push(Command::SetPermissions(path, mode));
            }
            Message::SetSort(heading_option, dir) => {
                if !matches!(self.location, Location::Search(..)) {
                    self.sort_name = heading_option;
                    self.sort_direction = dir;
                    if !matches!(self.location, Location::Desktop(..)) {
                        commands.push(Command::SetSort(
                            self.location.normalize().to_string(),
                            heading_option,
                            self.sort_direction,
                        ));
                    }
                }
            }
            Message::TabComplete(path, completions) => {
                if let Some(edit_location) = &mut self.edit_location
                    && edit_location.location.path_opt() == Some(&path)
                {
                    edit_location.completions = Some(completions);
                    commands.push(Command::Iced(
                        widget::text_input::focus(self.edit_location_id.clone()).into(),
                    ));
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
                                item.icon_handle_grid.clone_from(&handle);
                                item.icon_handle_list.clone_from(&handle);
                                item.icon_handle_list_condensed = handle;
                            }
                            item.thumbnail_opt = Some(thumbnail);
                            break;
                        }
                    }
                }
            }
            Message::ImageDecoded(path, width, height, pixels, display_size, generation) => {
                // Create handle from pre-decoded RGBA data (fast!)
                let handle = widget::image::Handle::from_rgba(width, height, pixels);

                // Store decoded image handle if generation still matches (not superseded)
                self.large_image_manager.store_decoded_with_generation(
                    path,
                    handle,
                    display_size,
                    generation,
                );
            }
            Message::ToggleSort(heading_option) => {
                if !matches!(self.location, Location::Search(..)) {
                    let heading_sort = if self.sort_name == heading_option {
                        !self.sort_direction
                    } else {
                        // Default modified to descending, and others to ascending.
                        heading_option != HeadingOptions::Modified
                    };

                    if !matches!(self.location, Location::Desktop(..)) {
                        commands.push(Command::SetSort(
                            self.location.normalize().to_string(),
                            heading_option,
                            heading_sort,
                        ));
                    }

                    self.sort_direction = heading_sort;
                    self.sort_name = heading_option;
                }
            }
            Message::Drop(Some((to, mut from))) => {
                self.dnd_hovered = None;
                match to {
                    Location::Desktop(to, ..)
                    | Location::Path(to)
                    | Location::Network(_, _, Some(to)) => {
                        if let Ok(entries) = fs::read_dir(&to) {
                            for i in entries.into_iter().filter_map(Result::ok) {
                                let i = i.path();
                                from.paths.retain(|p| &i != p);
                                if from.paths.is_empty() {
                                    log::info!("All dropped files already in target directory.");
                                    return commands;
                                }
                            }
                        }
                        commands.push(Command::DropFiles(to, from));
                    }
                    Location::Trash if matches!(from.kind, ClipboardKind::Cut { .. }) => {
                        commands.push(Command::Delete(from.paths));
                    }
                    _ => {
                        log::warn!("{:?} to {:?} is not supported.", from.kind, to);
                    }
                }
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
                        cosmic::Task::future(async move {
                            tokio::time::sleep(HOVER_DURATION).await;
                            Message::DndHover(loc)
                        })
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
            Message::DirectorySize(path, dir_size) => {
                let location = Location::Path(path);
                if let Some(ref mut item) = self.parent_item_opt
                    && item.location_opt.as_ref() == Some(&location)
                {
                    item.dir_size.clone_from(&dir_size);
                }
                if let Some(ref mut items) = self.items_opt {
                    for item in items.iter_mut() {
                        if item.location_opt.as_ref() == Some(&location) {
                            item.dir_size = dir_size;
                            break;
                        }
                    }
                }
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
        if let Some(mut location) = cd {
            location = location.normalize();
            if matches!(self.mode, Mode::Desktop) {
                match location {
                    Location::Path(path) => {
                        commands.push(Command::OpenFile(vec![path]));
                    }
                    Location::Trash => {
                        commands.push(Command::OpenTrash);
                    }
                    _ => {}
                }
            } else {
                // Select parent if location is not directory
                let mut selected_paths = None;
                if let Some(path) = location.path_opt()
                    && !path.is_dir()
                    && let Some(parent) = path.parent()
                {
                    selected_paths = Some(vec![path.clone()]);
                    location = location.with_path(parent.to_path_buf());
                }
                if location != self.location || selected_paths.is_some() {
                    if location.path_opt().is_none_or(|path| path.is_dir()) {
                        if selected_paths.is_none() {
                            selected_paths =
                                self.location.path_opt().map(|path| vec![path.clone()]);
                        }
                        self.change_location(&location, history_i_opt);
                        commands.push(Command::ChangeLocation(
                            self.title(),
                            location,
                            selected_paths,
                        ));
                    } else {
                        log::warn!("tried to cd to {location:?} which is not a directory");
                    }
                }
            }
        }

        // Update context menu popup
        if self.context_menu != last_context_menu {
            if last_context_menu.is_some() {
                commands.push(Command::ContextMenu(None, self.window_id));
            }
            if let Some(point) = self.context_menu {
                commands.push(Command::ContextMenu(Some(point), self.window_id));
            }
        }

        commands
    }

    pub(crate) const fn sort_options(&self) -> (HeadingOptions, bool, bool) {
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
            if sort { ord } else { ord.reverse() }
        };
        let mut items: Vec<_> = self.items_opt.as_ref()?.iter().enumerate().collect();
        let (sort_name, sort_direction, folders_first) = self.sort_options();
        match sort_name {
            HeadingOptions::Size => {
                items.sort_by(|a, b| {
                    // entries take precedence over size
                    let get_size = |x: &Item| match &x.metadata {
                        ItemMetadata::Path {
                            metadata,
                            children_opt,
                        } => {
                            if metadata.is_dir() {
                                (true, children_opt.unwrap_or_default() as u64)
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
                        #[cfg(feature = "gvfs")]
                        ItemMetadata::GvfsPath {
                            size_opt,
                            children_opt,
                            ..
                        } => match children_opt {
                            Some(child_count) => (true, *child_count as u64),
                            None => (false, size_opt.unwrap_or_default()),
                        },
                    };
                    let (a_is_entry, a_size) = get_size(a.1);
                    let (b_is_entry, b_size) = get_size(b.1);

                    //TODO: use folders_first?
                    match (a_is_entry, b_is_entry) {
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                        _ => check_reverse(a_size.cmp(&b_size), sort_direction),
                    }
                });
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
        let is_dnd_hovered = self.dnd_hovered.as_ref().map(|(l, _)| l) == Some(location);
        let mut container = widget::container(
            DndDestination::for_data::<ClipboardPaste>(element, move |data, action| {
                if let Some(mut data) = data {
                    if action == DndAction::Copy {
                        Message::Drop(Some((location1.clone(), data)))
                    } else if action == DndAction::Move {
                        data.kind = ClipboardKind::Cut { is_dnd: true };
                        Message::Drop(Some((location1.clone(), data)))
                    } else {
                        log::warn!("unsupported action: {action:?}");
                        Message::Drop(None)
                    }
                } else {
                    Message::Drop(None)
                }
            })
            .on_enter(move |_, _, _| Message::DndEnter(location2.clone()))
            .on_leave(move || Message::DndLeave(location3.clone())),
        );
        // Desktop will not show DnD indicator
        if is_dnd_hovered && !matches!(self.mode, Mode::Desktop) {
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

    pub fn gallery_view(&self) -> Element<'_, Message> {
        let cosmic_theme::Spacing {
            space_xxs,
            space_xs,
            space_m,
            ..
        } = theme::active().cosmic().spacing;

        //TODO: display error messages when image not found?
        let mut name_opt = None;
        let mut element_opt: Option<Element<Message>> = None;
        if let Some(index) = self.select_focus
            && let Some(items) = &self.items_opt
            && let Some(item) = items.get(index)
        {
            name_opt = Some(widget::text::heading(&item.display_name));
            match item
                .thumbnail_opt
                .as_ref()
                .unwrap_or(&ItemThumbnail::NotImage)
            {
                ItemThumbnail::NotImage => {}
                ItemThumbnail::Image(handle, original_dims) => {
                    // Determine which image to show based on async decode state
                    let mut is_loading = false;
                    let mut error_msg_opt = None;
                    let image_handle = if let Some(path) = item.path_opt() {
                        if let Some(error_msg) = self.large_image_manager.get_error(path) {
                            error_msg_opt = Some(error_msg.clone());
                            handle.clone()
                        } else if self.large_image_manager.is_decoding(path) {
                            // Currently decoding (initial or re-decode) --> show cached/thumbnail with loading indicator
                            is_loading = true;
                            // Use decoded handle if available (re-decode), otherwise thumbnail (initial decode)
                            self.large_image_manager
                                .get_decoded(path)
                                .cloned()
                                .unwrap_or_else(|| handle.clone())
                        } else if let Some(decoded_handle) =
                            self.large_image_manager.get_decoded(path)
                        {
                            // Decoded and not currently decoding --> use it
                            decoded_handle.clone()
                        } else if let Some((w, h)) = original_dims {
                            // Check if image needs tiling
                            if should_use_tiling(*w, *h) {
                                // Large image --> show thumbnail only
                                handle.clone()
                            } else {
                                // Normal-sized image --> load full resolution directly
                                widget::image::Handle::from_path(path)
                            }
                        } else {
                            // No dimensions available --> show thumbnail
                            handle.clone()
                        }
                    } else {
                        handle.clone()
                    };

                    let content: cosmic::Element<'_, Message> =
                        if let Some(error_msg) = error_msg_opt {
                            widget::column()
                                .push(widget::image(image_handle))
                                .push(widget::text(format!("⚠ {}", error_msg)).size(13))
                                .padding(space_xs)
                                .align_x(cosmic::iced::Alignment::Center)
                                .into()
                        } else if is_loading {
                            widget::column()
                                .push(widget::image(image_handle))
                                .push(widget::text("Loading higher resolution...").size(14))
                                .padding(space_xs)
                                .align_x(cosmic::iced::Alignment::Center)
                                .into()
                        } else {
                            //TODO: use widget::image::viewer, when its zoom can be reset
                            widget::image(image_handle).into()
                        };

                    element_opt = Some(widget::container(content).center(Length::Fill).into());
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
                    element_opt = Some(
                        widget::container(widget::text_editor(text).padding(space_xxs).class(
                            cosmic::theme::iced::TextEditor::Custom(Box::new(text_editor_class)),
                        ))
                        .center(Length::Fill)
                        .into(),
                    );
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
                .on_press(|_| Message::WindowDrag)
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

    pub fn location_view(&self) -> Element<'_, Message> {
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
        fn text_width_body(content: &str) -> f32 {
            //TODO: should libcosmic set the font when using widget::text::body?
            text_width(content, font::default(), 14.0, 20.0)
        }
        fn text_width_heading(content: &str) -> f32 {
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
        w += f32::from(space_xxs).mul_add(2.0, 16.0);

        let mut next_button =
            widget::button::custom(widget::icon::from_name("go-next-symbolic").size(16))
                .padding(space_xxs)
                .class(theme::Button::Icon);
        if self.history_i + 1 < self.history.len() {
            next_button = next_button.on_press(Message::GoNext);
        }
        row = row.push(next_button);
        w += f32::from(space_xxs).mul_add(2.0, 16.0);

        row = row.push(widget::Space::with_width(Length::Fixed(space_s.into())));
        w += f32::from(space_s);

        //TODO: allow resizing?
        let name_width = 300.0;
        let modified_width = 200.0;
        let size_width = 100.0;
        let condensed = size.width < (name_width + modified_width + size_width);

        let (sort_name, sort_direction, _) = self.sort_options();
        let heading_item = |name, width, msg| {
            let mut row = widget::row::with_capacity(2)
                .align_y(Alignment::Center)
                .spacing(space_xxxs)
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

        let heading_row = widget::row::with_children([
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
        .padding([0, space_xxs]);

        let accent_rule =
            horizontal_rule(1).class(theme::Rule::Custom(Box::new(|theme| rule::Style {
                color: theme.cosmic().accent_color().into(),
                width: 1,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
            })));
        let heading_rule = widget::container(horizontal_rule(1))
            .padding([0, theme::active().cosmic().corner_radii.radius_xs[0] as u16]);

        if let Some(edit_location) = &self.edit_location {
            let mut text_input = None;

            //TODO: allow editing other locations
            if let Location::Network(ref uri, ..) = edit_location.location {
                let location = edit_location.location.clone();
                text_input = Some(
                    widget::text_input("", uri.clone())
                        .id(self.edit_location_id.clone())
                        .on_input(move |input| {
                            Message::EditLocation(Some(location.with_uri(input).into()))
                        })
                        .on_submit(|_| Message::EditLocationSubmit)
                        .line_height(1.0),
                );
            } else if let Some(resolved_location) = edit_location.resolve()
                && let Some(path) = resolved_location.path_opt().cloned()
            {
                text_input = Some(
                    widget::text_input("", path.to_string_lossy().into_owned())
                        .id(self.edit_location_id.clone())
                        .on_input(move |input| {
                            Message::EditLocation(Some(
                                resolved_location.with_path(PathBuf::from(input)).into(),
                            ))
                        })
                        .on_submit(|_| Message::EditLocationSubmit)
                        .on_tab(Message::EditLocationTab)
                        .on_unfocus(Message::EditLocation(None))
                        .line_height(1.0),
                );
            }
            if let Some(text_input) = text_input {
                row = row.push(
                    widget::button::custom(
                        widget::icon::from_name("window-close-symbolic").size(16),
                    )
                    .on_press(Message::EditLocation(None))
                    .padding(space_xxs)
                    .class(theme::Button::Icon),
                );
                let mut popover =
                    widget::popover(text_input).position(widget::popover::Position::Bottom);
                if let Some(completions) = &edit_location.completions
                    && !completions.is_empty()
                {
                    let mut column =
                        widget::column::with_capacity(completions.len()).padding(space_xxs);
                    for (i, (name, _path)) in completions.iter().enumerate() {
                        let selected = edit_location.selected == Some(i);
                        column = column.push(
                            widget::button::custom(widget::text::body(name))
                                //TODO: match to design
                                .class(if selected {
                                    theme::Button::Standard
                                } else {
                                    theme::Button::HeaderBar
                                })
                                .on_press(Message::EditLocationComplete(i))
                                .padding(space_xxs)
                                .width(Length::Fill),
                        );
                    }
                    popover = popover.popup(
                        widget::container(column)
                            .class(theme::Container::Dropdown)
                            //TODO: This is a hack to get the popover to be the right width
                            .max_width(size.width - 140.0),
                    );
                }
                row = row.push(popover);
                let mut column = widget::column::with_capacity(4).padding([0, space_s]);
                column = column.push(row);
                column = column.push(accent_rule);
                if self.config.view == View::List && !condensed {
                    column = column.push(heading_row);
                    column = column.push(heading_rule);
                }
                return column.into();
            }
        } else if let Some(path) = self.location.path_opt() {
            row = row.push(
                crate::mouse_area::MouseArea::new(
                    widget::button::custom(widget::icon::from_name("edit-symbolic").size(16))
                        .padding(space_xxs)
                        .class(theme::Button::Icon)
                        .on_press(Message::EditLocation(Some(self.location.clone().into()))),
                )
                .on_middle_press(move |_| Message::OpenInNewTab(path.clone())),
            );
            w += f32::from(space_xxs).mul_add(2.0, 16.0);
        }

        let mut children: Vec<Element<_>> = Vec::new();
        match &self.location {
            Location::Desktop(path, ..) | Location::Path(path) | Location::Search(path, ..) => {
                let excess_str = "...";
                let excess_width = text_width_body(excess_str);
                for (index, ancestor) in path.ancestors().enumerate() {
                    let (name, found_home) = folder_name(ancestor);
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
                    w += 2.0 * f32::from(space_xxxs);

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
                            .on_press(if ancestor == path {
                                Message::EditLocation(Some(self.location.clone().into()))
                            } else {
                                Message::Location(location.clone())
                            }),
                    );

                    if self.location_context_menu_index.is_some() {
                        mouse_area = mouse_area
                            .on_right_press(move |point_opt| {
                                Message::LocationContextMenuIndex(point_opt, None)
                            })
                            .wayland_on_right_press_window_position();
                    } else {
                        mouse_area = mouse_area
                            .on_right_press_no_capture()
                            .on_right_press(move |point_opt| {
                                Message::LocationContextMenuIndex(point_opt, Some(index))
                            })
                            .wayland_on_right_press_window_position();
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
            Location::Network(uri, display_name, path) => {
                children.push(
                    widget::button::custom(widget::text::heading(display_name))
                        .padding(space_xxxs)
                        .on_press(Message::Location(Location::Network(
                            uri.clone(),
                            display_name.clone(),
                            path.clone(),
                        )))
                        .class(theme::Button::Text)
                        .into(),
                );
            }
        }

        row = row.extend(children);
        let mut column = widget::column::with_capacity(4).padding([0, space_s]);
        column = column.push(row);
        column = column.push(accent_rule);

        if self.config.view == View::List && !condensed {
            column = column.push(heading_row);
            column = column.push(heading_rule);
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
                .position(widget::popover::Position::Point(point));
        }

        popover.into()
    }

    pub fn empty_view(&self, has_hidden: bool) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        mouse_area::MouseArea::new(widget::column::with_children([widget::container(
            match self.mode {
                Mode::App | Mode::Dialog(_) => widget::column::with_children([
                    widget::icon::from_name("folder-symbolic")
                        .size(64)
                        .icon()
                        .into(),
                    widget::text::body(if has_hidden {
                        fl!("empty-folder-hidden")
                    } else if matches!(self.location, Location::Search(..)) {
                        fl!("no-results")
                    } else {
                        fl!("empty-folder")
                    })
                    .into(),
                ]),
                Mode::Desktop => widget::column(),
            }
            .align_x(Alignment::Center)
            .spacing(space_xxs),
        )
        .center(Length::Fill)
        .into()]))
        .on_press(|_| Message::Click(None))
        .into()
    }

    pub fn grid_view(
        &self,
    ) -> (
        Option<Element<'static, Message>>,
        Element<'_, Message>,
        bool,
    ) {
        let cosmic_theme::Spacing {
            space_xxs,
            space_xxxs,
            ..
        } = theme::active().cosmic().spacing;

        let TabConfig {
            show_hidden,
            mut icon_sizes,
            ..
        } = self.config;

        let mut grid_spacing = space_xxs;
        if let Location::Desktop(_path, _output, desktop_config) = &self.location {
            icon_sizes.grid = desktop_config.icon_size;
            grid_spacing = desktop_config.grid_spacing_for(space_xxs);
        }

        let text_height = 3 * 20; // 3 lines of text
        let item_width = (3 * space_xxs + icon_sizes.grid() + 3 * space_xxs) as usize;
        let item_height =
            (space_xxxs + icon_sizes.grid() + space_xxxs + text_height + space_xxxs) as usize;

        let (width, height) = match self.size_opt.get() {
            Some(size) => (
                (size.width.floor() as usize)
                    .saturating_sub(2 * (space_xxs as usize))
                    .max(item_width),
                (size.height.floor() as usize).max(item_height),
            ),
            None => (item_width, item_height),
        };

        let (cols, column_spacing) = {
            let width_m1 = width.saturating_sub(item_width);
            let cols_m1 = width_m1 / (item_width + grid_spacing as usize);
            let cols = cols_m1 + 1;
            let spacing = width_m1
                .checked_div(cols_m1)
                .unwrap_or(0)
                .saturating_sub(item_width);
            (cols, spacing as u16)
        };

        let rows = {
            let height_m1 = height.saturating_sub(item_height);
            let rows_m1 = height_m1 / (item_height + grid_spacing as usize);
            rows_m1 + 1
        };

        //TODO: move to function
        let visible_rect = {
            // Use cached content height to clamp scroll offset after resize
            let max_scroll_y = self
                .content_height_opt
                .get()
                .map(|ch| (ch - height as f32).max(0.0))
                .unwrap_or(f32::MAX);
            let scroll_y = self
                .scroll_opt
                .map(|o| o.y.min(max_scroll_y).max(0.0))
                .unwrap_or(0.0);
            let point = Point::new(0.0, scroll_y);
            let size = self.size_opt.get().unwrap_or_else(|| Size::new(0.0, 0.0));
            Rectangle::new(point, size)
        };

        let mut grid = widget::grid()
            .column_spacing(column_spacing)
            .row_spacing(grid_spacing)
            .padding(space_xxs.into());
        let mut dnd_items: Vec<(usize, (usize, usize), &Item)> = Vec::new();
        let mut drag_w_i = usize::MAX;
        let mut drag_n_i = usize::MAX;
        let mut drag_e_i = 0;
        let mut drag_s_i = 0;

        let mut column = widget::column::with_capacity(2);
        if let Some(items) = self.column_sort() {
            let mut count = 0;
            let mut col = 0;
            let mut row = 0;
            let mut page_row = 0;
            let mut hidden = 0;
            let mut grid_elements = Vec::new();
            for &(i, item) in &items {
                if !show_hidden && item.hidden {
                    item.pos_opt.set(None);
                    item.rect_opt.set(None);
                    hidden += 1;
                    continue;
                }
                item.pos_opt.set(Some((row, col)));
                let item_rect = Rectangle::new(
                    Point::new(
                        (col * (item_width + column_spacing as usize) + space_xxs as usize) as f32,
                        (row * (item_height + grid_spacing as usize) + space_xxs as usize) as f32,
                    ),
                    Size::new(item_width as f32, item_height as f32),
                );
                item.rect_opt.set(Some(item_rect));

                //TODO: error if the row or col is already set?
                while grid_elements.len() <= row {
                    grid_elements.push(Vec::new());
                }

                // Only build elements if visible (for performance)
                if item_rect.intersects(&visible_rect) {
                    //TODO: one focus group per grid item (needs custom widget)
                    let buttons: Vec<Element<Message>> = vec![
                        widget::button::custom(
                            widget::icon::icon(item.icon_handle_grid.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_sizes.grid())
                                .width(Length::Shrink),
                        )
                        .padding(space_xxxs)
                        .class(button_style(
                            item.selected,
                            item.highlighted,
                            item.cut,
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
                                    item.cut,
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
                            column = column.push(button);
                        } else {
                            column = column.push(
                                mouse_area::MouseArea::new(button)
                                    .on_right_press_no_capture()
                                    .wayland_on_right_press_window_position()
                                    .on_right_press(move |point_opt| {
                                        Message::RightClick(point_opt, Some(i))
                                    }),
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
                    grid_elements[row].push(Element::from(mouse_area));
                } else {
                    // Add a spacer if the row is empty, so scroll works
                    if grid_elements[row].is_empty() {
                        grid_elements[row].push(Element::from(
                            widget::column()
                                .width(Length::Fill)
                                .height(Length::Fixed(item_height as f32)),
                        ));
                    }
                }

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

            column = column.push(grid);

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

                // Cache content height for scroll clamping on next frame
                self.content_height_opt.set(Some(max_bottom as f32));

                let top_deduct = 7 * (space_xxs as usize);

                self.item_view_size_opt
                    .set(self.size_opt.get().map(|s| Size {
                        width: s.width,
                        height: s.height - top_deduct as f32,
                    }));

                let spacer_height = height.saturating_sub(max_bottom + top_deduct);
                if spacer_height > 0 {
                    column = column.push(widget::container(Space::with_height(Length::Fixed(
                        spacer_height as f32,
                    ))));
                }
            }
        }

        let drag_list = (!dnd_items.is_empty()).then(|| {
            let mut dnd_grid = widget::grid()
                .column_spacing(column_spacing)
                .row_spacing(grid_spacing)
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
                                item.cut,
                                false,
                                false,
                                false,
                            )),
                            widget::button::custom(widget::text::body(item.display_name.clone()))
                                .id(item.button_id.clone())
                                .on_press(Message::Click(Some(*i)))
                                .padding([0, space_xxxs])
                                .class(button_style(
                                    item.selected,
                                    item.highlighted,
                                    item.cut,
                                    true,
                                    true,
                                    false,
                                )),
                        ];

                        let column =
                            widget::column::with_children(buttons.into_iter().map(Element::from))
                                .align_x(Alignment::Center)
                                .height(Length::Fixed(item_height as f32))
                                .width(Length::Fixed(item_width as f32));

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
        });

        let mut mouse_area = mouse_area::MouseArea::new(column.width(Length::Fill))
            .on_press(|_| Message::Click(None))
            .on_auto_scroll(Message::AutoScroll)
            .on_drag_end(|_| Message::DragEnd)
            .show_drag_rect(self.mode.multiple())
            .on_release(|_| Message::ClickRelease(None));
        if self.watch_drag {
            mouse_area = mouse_area.on_drag(Message::Drag);
        }

        (drag_list, mouse_area.into(), true)
    }

    pub fn list_view(
        &self,
    ) -> (
        Option<Element<'static, Message>>,
        Element<'_, Message>,
        bool,
    ) {
        let cosmic_theme::Spacing {
            space_s, space_xxs, ..
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

        let mut column = widget::column::with_capacity(3);
        let mut y: f32 = 0.0;

        let rule_padding = theme::active().cosmic().corner_radii.radius_xs[0] as u16;

        //TODO: move to function
        let visible_rect = {
            // Use cached content height to clamp scroll offset after resize
            let max_scroll_y = self
                .content_height_opt
                .get()
                .map(|ch| (ch - size.height).max(0.0))
                .unwrap_or(f32::MAX);
            let scroll_y = self
                .scroll_opt
                .map(|o| o.y.min(max_scroll_y).max(0.0))
                .unwrap_or(0.0);
            let point = Point::new(0.0, scroll_y);
            let size = self.size_opt.get().unwrap_or_else(|| Size::new(0.0, 0.0));
            Rectangle::new(point, size)
        };

        let mut drag_items = Vec::new();
        if let Some(items) = self.column_sort() {
            let mut count = 0;
            let mut hidden = 0;
            for (i, item) in items {
                if item.hidden && !show_hidden {
                    item.pos_opt.set(None);
                    item.rect_opt.set(None);
                    hidden += 1;
                    continue;
                }

                if count > 0 {
                    column = column
                        .push(widget::container(horizontal_rule(1)).padding([0, rule_padding]));
                    y += 1.0;
                }

                item.pos_opt.set(Some((count, 0)));
                let item_rect = Rectangle::new(
                    Point::new(f32::from(space_s), y),
                    Size::new(size.width - f32::from(2 * space_s), f32::from(row_height)),
                );
                item.rect_opt.set(Some(item_rect));

                // Only build elements if visible (for performance)
                let button_row = if item_rect.intersects(&visible_rect) {
                    let modified_text = match &item.metadata {
                        ItemMetadata::Path { metadata, .. } => match metadata.modified() {
                            Ok(time) => self.format_time(time).to_string(),
                            Err(_) => String::new(),
                        },
                        ItemMetadata::Trash { entry, .. } => FormatTime::from_secs(
                            entry.time_deleted,
                            &self.date_time_formatter,
                            &self.time_formatter,
                        )
                        .map(|t| t.to_string())
                        .unwrap_or_default(),
                        #[cfg(feature = "gvfs")]
                        ItemMetadata::GvfsPath { .. } => match item.metadata.modified() {
                            Some(mtime) => self.format_time(mtime).to_string(),
                            None => String::new(),
                        },
                        _ => String::new(),
                    };

                    let size_text = match &item.metadata {
                        ItemMetadata::Path {
                            metadata,
                            children_opt,
                        } => {
                            if metadata.is_dir() {
                                //TODO: translate
                                if let Some(children) = children_opt {
                                    if *children == 1 {
                                        format!("{children} item")
                                    } else {
                                        format!("{children} items")
                                    }
                                } else {
                                    String::new()
                                }
                            } else {
                                format_size(metadata.len())
                            }
                        }
                        ItemMetadata::Trash { metadata, .. } => match metadata.size {
                            trash::TrashItemSize::Entries(entries) => {
                                //TODO: translate
                                if entries == 1 {
                                    format!("{entries} item")
                                } else {
                                    format!("{entries} items")
                                }
                            }
                            trash::TrashItemSize::Bytes(bytes) => format_size(bytes),
                        },
                        ItemMetadata::SimpleDir { entries } => {
                            //TODO: translate
                            if *entries == 1 {
                                format!("{entries} item")
                            } else {
                                format!("{entries} items")
                            }
                        }
                        ItemMetadata::SimpleFile { size } => format_size(*size),
                        #[cfg(feature = "gvfs")]
                        ItemMetadata::GvfsPath {
                            size_opt,
                            children_opt,
                            ..
                        } => match children_opt {
                            Some(child_count) => {
                                if *child_count == 1 {
                                    format!("{child_count} item")
                                } else {
                                    format!("{child_count} items")
                                }
                            }
                            None => format_size(size_opt.unwrap_or_default()),
                        },
                    };

                    let row = if condensed {
                        widget::row::with_children([
                            widget::icon::icon(item.icon_handle_list_condensed.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::column::with_children([
                                widget::text::body(item.display_name.clone()).into(),
                                //TODO: translate?
                                widget::text::caption(format!("{modified_text} - {size_text}"))
                                    .into(),
                            ])
                            .into(),
                        ])
                        .height(Length::Fixed(f32::from(row_height)))
                        .align_y(Alignment::Center)
                        .spacing(space_xxs)
                    } else if is_search {
                        widget::row::with_children([
                            widget::icon::icon(item.icon_handle_list_condensed.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::column::with_children([
                                widget::text::body(item.display_name.clone()).into(),
                                widget::text::caption(match item.path_opt() {
                                    Some(path) => path.display().to_string(),
                                    None => String::new(),
                                })
                                .into(),
                            ])
                            .width(Length::Fill)
                            .into(),
                            widget::text::body(modified_text.clone())
                                .width(Length::Fixed(modified_width))
                                .into(),
                            widget::text::body(size_text.clone())
                                .width(Length::Fixed(size_width))
                                .into(),
                        ])
                        .height(Length::Fixed(f32::from(row_height)))
                        .align_y(Alignment::Center)
                        .spacing(space_xxs)
                    } else {
                        widget::row::with_children([
                            widget::icon::icon(item.icon_handle_list.clone())
                                .content_fit(ContentFit::Contain)
                                .size(icon_size)
                                .into(),
                            widget::text::body(item.display_name.clone())
                                .width(Length::Fill)
                                .into(),
                            widget::text::body(modified_text.clone())
                                .width(Length::Fixed(modified_width))
                                .into(),
                            widget::text::body(size_text.clone())
                                .width(Length::Fixed(size_width))
                                .into(),
                        ])
                        .height(Length::Fixed(f32::from(row_height)))
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
                                    item.cut,
                                    true,
                                    true,
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
                            mouse_area
                                .on_right_press_no_capture()
                                .wayland_on_right_press_window_position()
                                .on_right_press(move |point_opt| {
                                    Message::RightClick(point_opt, Some(i))
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
                            Element::from(Space::with_height(Length::Fixed(f32::from(row_height))))
                        } else if condensed {
                            widget::row::with_children([
                                widget::icon::icon(item.icon_handle_list_condensed.clone())
                                    .content_fit(ContentFit::Contain)
                                    .size(icon_size)
                                    .into(),
                                widget::column::with_children([
                                    widget::text::body(item.display_name.clone()).into(),
                                    //TODO: translate?
                                    widget::text::body(format!("{modified_text} - {size_text}"))
                                        .into(),
                                ])
                                .into(),
                            ])
                            .align_y(Alignment::Center)
                            .spacing(space_xxs)
                            .into()
                        } else if is_search {
                            widget::row::with_children([
                                widget::icon::icon(item.icon_handle_list_condensed.clone())
                                    .content_fit(ContentFit::Contain)
                                    .size(icon_size)
                                    .into(),
                                widget::column::with_children([
                                    widget::text::body(item.display_name.clone()).into(),
                                    widget::text::caption(match item.path_opt() {
                                        Some(path) => path.display().to_string(),
                                        None => String::new(),
                                    })
                                    .into(),
                                ])
                                .width(Length::Fill)
                                .into(),
                                widget::text::body(modified_text.clone())
                                    .width(Length::Fixed(modified_width))
                                    .into(),
                                widget::text::body(size_text.clone())
                                    .width(Length::Fixed(size_width))
                                    .into(),
                            ])
                            .align_y(Alignment::Center)
                            .spacing(space_xxs)
                            .into()
                        } else {
                            widget::row::with_children([
                                widget::icon::icon(item.icon_handle_list.clone())
                                    .content_fit(ContentFit::Contain)
                                    .size(icon_size)
                                    .into(),
                                widget::text::body(item.display_name.clone())
                                    .width(Length::Fill)
                                    .into(),
                                widget::text(modified_text)
                                    .width(Length::Fixed(modified_width))
                                    .into(),
                                widget::text::body(size_text)
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

                    button_row
                } else {
                    widget::column()
                        .width(Length::Fill)
                        .height(Length::Fixed(f32::from(row_height)))
                        .into()
                };

                count += 1;
                y += f32::from(row_height);
                column = column.push(button_row);
            }

            if count == 0 {
                return (None, self.empty_view(hidden > 0), false);
            }

            // Cache content height for scroll clamping on next frame
            self.content_height_opt.set(Some(y));
        }
        //TODO: HACK If we don't reach the bottom of the view, go ahead and add a spacer to do that
        {
            let top_deduct = (if condensed || is_search { 6 } else { 9 }) * space_xxs;

            self.item_view_size_opt
                .set(self.size_opt.get().map(|s| Size {
                    width: s.width,
                    height: s.height - f32::from(top_deduct),
                }));

            let spacer_height = size.height - y - f32::from(top_deduct);
            if spacer_height > 0. {
                column = column.push(widget::container(Space::with_height(spacer_height)));
            }
        }
        let drag_col = (!drag_items.is_empty())
            .then(|| Element::from(widget::column::with_children(drag_items)));

        let mut mouse_area = mouse_area::MouseArea::new(column.padding([0, space_s]))
            .with_id(Id::new("list-view"))
            .on_press(|_| Message::Click(None))
            .on_auto_scroll(Message::AutoScroll)
            .on_drag_end(|_| Message::DragEnd)
            .show_drag_rect(self.mode.multiple())
            .on_release(|_| Message::ClickRelease(None));
        if self.watch_drag {
            mouse_area = mouse_area.on_drag(Message::Drag);
        }

        (drag_col, mouse_area.into(), true)
    }

    pub fn view_responsive<'a>(
        &'a self,
        key_binds: &'a HashMap<KeyBind, Action>,
        modifiers: &'a Modifiers,
        size: Size,
    ) -> Element<'a, Message> {
        // Update cached size
        self.size_opt.set(Some(size));

        let cosmic_theme::Spacing {
            space_xxxs,
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
                    .filter_map(|item| {
                        if item.selected {
                            item.path_opt().cloned()
                        } else {
                            None
                        }
                    })
                    .collect::<Box<[PathBuf]>>()
            })
            .unwrap_or_default();
        let item_view =
            DndSource::<Message, ClipboardCopy>::with_id(item_view, Id::new("tab-view"));

        let view = self.config.view;
        let item_view = match drag_list {
            Some(drag_list) if self.selected_clicked => {
                let drag_list = RcElementWrapper::<Message>(Rc::new(RefCell::new(drag_list)));
                item_view
                    .drag_content(move || {
                        ClipboardCopy::new(crate::clipboard::ClipboardKind::Copy, &files)
                    })
                    .drag_icon(move |_| {
                        let state: tree::State = Widget::<Message, _, _>::state(&drag_list);
                        (
                            Element::from(drag_list.clone()).map(|_m| ()),
                            state,
                            match view {
                                // offset by grid padding so that we grab the top left corner of the item in the drag grid.
                                View::Grid => Vector::new(
                                    f32::from(space_xxs).mul_add(-3.0, -f32::from(space_xxxs)),
                                    -4. * f32::from(space_xxxs),
                                ),
                                View::List => Vector::ZERO,
                            },
                        )
                    })
            }
            _ => item_view,
        };

        let tab_location = self.location.clone();
        let mouse_area = mouse_area::MouseArea::new(item_view)
            .on_press(move |_point_opt| Message::Click(None))
            .on_release(|_| Message::ClickRelease(None))
            .on_resize(Message::Resize)
            .on_back_press(move |_point_opt| Message::GoPrevious)
            .on_forward_press(move |_point_opt| Message::GoNext)
            .on_scroll(|delta| respond_to_scroll_direction(delta, modifiers))
            .on_right_press(move |p| {
                Message::ContextMenu(
                    if self.context_menu.is_some() { None } else { p },
                    self.window_id,
                )
            })
            .wayland_on_right_press_window_position();

        let mut popover = widget::popover(mouse_area);
        if let Some(point) = self.context_menu
            && (!cfg!(feature = "wayland") || !crate::is_wayland())
        {
            let context_menu = menu::context_menu(self, key_binds, modifiers);
            popover = popover
                .popup(context_menu)
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
                if let Some(items) = self.items_opt()
                    && !items.is_empty()
                {
                    tab_column = tab_column.push(
                        widget::layer_container(widget::row::with_children([
                            widget::horizontal_space().into(),
                            widget::button::standard(fl!("empty-trash"))
                                .on_press(Message::EmptyTrash)
                                .into(),
                        ]))
                        .padding([space_xxs, space_xs])
                        .layer(cosmic_theme::Layer::Primary)
                        .apply(widget::container)
                        .padding([0, 0, 7, 0]),
                    );
                }
            }
            Location::Network(uri, _display_name, _path) if uri == "network:///" => {
                tab_column = tab_column.push(
                    widget::layer_container(widget::row::with_children([
                        widget::horizontal_space().into(),
                        widget::button::standard(fl!("add-network-drive"))
                            .on_press(Message::AddNetworkDrive)
                            .into(),
                    ]))
                    .padding([space_xxs, space_xs])
                    .layer(cosmic_theme::Layer::Primary)
                    .apply(widget::container)
                    .padding([0, 0, 7, 0]),
                );
            }
            _ => {}
        }
        let mut tab_view = widget::container(tab_column)
            .height(Length::Fill)
            .width(Length::Fill);

        // Desktop will not show DnD indicator
        if self.dnd_hovered.as_ref().map(|(l, _)| l) == Some(&tab_location)
            && !matches!(self.mode, Mode::Desktop)
        {
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
                    data.kind = ClipboardKind::Cut { is_dnd: true };
                    Message::Drop(Some((tab_location.clone(), data)))
                } else {
                    log::warn!("unsupported action: {action:?}");
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
    pub fn multi_preview_view<'a>(&'a self) -> Element<'a, Message> {
        let cosmic_theme::Spacing {
            space_xxxs,
            space_m,
            ..
        } = theme::active().cosmic().spacing;

        let mut column = widget::column().spacing(space_m);

        let handle = widget::icon::from_name("text-x-generic")
            .size(IconSizes::default().grid())
            .handle();

        let icon = widget::icon::icon(handle.clone())
            .content_fit(ContentFit::Contain)
            .size(IconSizes::default().grid());

        let icon_container1 = widget::container(icon.clone()).padding(padding::bottom(10).left(10));
        let icon_container2 =
            widget::container(icon.clone()).padding(padding::top(5).bottom(5).left(5).right(5));
        let icon_container3 = widget::container(icon).padding(padding::top(10).right(10));
        let stack = stack![icon_container1, icon_container2, icon_container3];

        column = column.push(
            widget::container(stack)
                .center_x(Length::Fill)
                .max_height(THUMBNAIL_SIZE as f32),
        );

        let selected_items: Vec<&Item> = self.items_opt().map_or(Vec::new(), |items| {
            items.iter().filter(|item| item.selected).collect()
        });

        let mut details = widget::column().spacing(space_xxxs);
        details = details.push(widget::text::body(fl!(
            "items",
            items = selected_items.len()
        )));

        let mut total_size: u64 = 0;
        let mut mime_type_counts: BTreeMap<String, u64> = BTreeMap::new();
        let mut calculating_dir_size = false;
        let mut dir_size_error: Option<String> = None;

        for item in selected_items.iter() {
            *mime_type_counts.entry(item.mime.to_string()).or_insert(0) += 1;

            if let Some(metadata) = item.file_metadata() {
                if metadata.is_dir() {
                    match &item.dir_size {
                        DirSize::Calculating(_) => {
                            calculating_dir_size = true;
                        }
                        DirSize::Directory(size) => {
                            total_size = total_size.saturating_add(*size);
                        }
                        DirSize::NotDirectory => (),
                        DirSize::Error(err) => {
                            dir_size_error = Some(err.clone());
                        }
                    };
                } else {
                    total_size = total_size.saturating_add(metadata.len());
                }
            }
        }
        let mut mime_types: Vec<(String, u64)> = mime_type_counts.into_iter().collect();
        mime_types.sort_by(|(_, v1), (_, v2)| v2.cmp(v1));

        // Limit the number of displayed mime types
        let limit = usize::min(10, mime_types.len());

        let mut mime_type_strings: Vec<String> = mime_types[..limit]
            .iter()
            .map(|(mime, count)| format!("{} ({})", mime, count))
            .collect();

        if mime_types.len() > limit {
            mime_type_strings.push("...".to_string());
        }

        details = details.push(widget::text::body(fl!(
            "type",
            mime = mime_type_strings.join(", ")
        )));

        let size = {
            if calculating_dir_size {
                fl!("calculating")
            } else if let Some(error) = dir_size_error {
                error
            } else {
                format_size(total_size)
            }
        };

        details = details.push(widget::text::body(fl!("item-size", size = size)));

        column = column.push(details);

        column = column.push(widget::button::standard(fl!("open")).on_press(Message::Open(None)));

        column.into()
    }
    pub fn view<'a>(
        &'a self,
        key_binds: &'a HashMap<KeyBind, Action>,
        modifiers: &'a Modifiers,
    ) -> Element<'a, Message> {
        widget::responsive(|size| self.view_responsive(key_binds, modifiers, size)).into()
    }

    pub fn subscription(&self, preview: bool) -> Subscription<Message> {
        //TODO: how many thumbnail loads should be in flight at once?
        let jobs = self.thumb_config.jobs.get() as usize;
        let mut subscriptions = Vec::with_capacity(jobs + 3);

        if let Some(items) = &self.items_opt {
            //TODO: move to function
            let visible_rect = {
                let point = match self.scroll_opt {
                    Some(offset) => Point::new(0.0, offset.y),
                    None => Point::new(0.0, 0.0),
                };
                let size = self.size_opt.get().unwrap_or_else(|| Size::new(0.0, 0.0));
                Rectangle::new(point, size)
            };

            for item in items {
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

                let Some(path) = item.path_opt().cloned() else {
                    continue;
                };

                let metadata = item.metadata.clone();
                let can_thumbnail = match metadata {
                    ItemMetadata::Path { .. } => true,
                    #[cfg(feature = "gvfs")]
                    ItemMetadata::GvfsPath { .. } => true,
                    _ => false,
                };
                if can_thumbnail {
                    let mime = item.mime.clone();
                    let max_jobs = jobs;
                    let max_mb = u64::from(self.thumb_config.max_mem_mb.get());
                    let max_size = u64::from(self.thumb_config.max_size_mb.get());

                    // Determine effective memory budget based on image size
                    let (effective_max_mb, effective_jobs) = if mime.type_() == mime::IMAGE {
                        match image::image_dimensions(&path) {
                            Ok((width, height)) => {
                                let (_use_dedicated, eff_mb, eff_jobs) =
                                    should_use_dedicated_worker(width, height, max_mb, max_jobs);
                                (eff_mb, eff_jobs)
                            }
                            Err(_) => (max_mb, max_jobs),
                        }
                    } else {
                        (max_mb, max_jobs)
                    };

                    subscriptions.push(Subscription::run_with_id(
                        ("thumbnail", path.clone()),
                        stream::channel(1, move |mut output| async move {
                            let message = {
                                let path = path.clone();

                                // Acquire semaphore permit
                                _ = THUMB_SEMAPHORE.acquire().await;

                                tokio::task::spawn_blocking(move || {
                                    let start = Instant::now();
                                    let thumbnail = ItemThumbnail::new(
                                        &path,
                                        metadata,
                                        mime,
                                        THUMBNAIL_SIZE,
                                        effective_max_mb,
                                        effective_jobs,
                                        max_size,
                                    );
                                    log::debug!(
                                        "thumbnailed {} in {:?}",
                                        path.display(),
                                        start.elapsed()
                                    );
                                    Message::Thumbnail(path, thumbnail)
                                })
                                .await
                                .unwrap()
                            };

                            match output.send(message).await {
                                Ok(()) => {}
                                Err(err) => {
                                    log::warn!(
                                        "failed to send thumbnail for {}: {}",
                                        path.display(),
                                        err
                                    );
                                }
                            }

                            std::future::pending().await
                        }),
                    ));
                }

                if subscriptions.len() >= jobs {
                    break;
                }
            }

            if preview {
                // Load directory size for selected items

                let mut selected_items: Vec<&Item> =
                    items.iter().filter(|item| item.selected).collect();

                if selected_items.is_empty()
                    && let Some(p) = self.parent_item_opt.as_ref()
                {
                    selected_items.push(p)
                }
                for item in selected_items {
                    // Item must have a path
                    if let Some(path) = item.path_opt().cloned() {
                        // Item must be calculating directory size
                        if let DirSize::Calculating(controller) = &item.dir_size {
                            let controller = controller.clone();
                            subscriptions.push(Subscription::run_with_id(
                                ("dir_size", path.clone()),
                                stream::channel(1, |mut output| async move {
                                    let message = {
                                        let start = Instant::now();
                                        match calculate_dir_size(&path, controller).await {
                                            Ok(size) => {
                                                log::debug!(
                                                    "calculated directory size of {} in {:?}",
                                                    path.display(),
                                                    start.elapsed()
                                                );
                                                Message::DirectorySize(
                                                    path.clone(),
                                                    DirSize::Directory(size),
                                                )
                                            }
                                            Err(err) => {
                                                log::warn!(
                                                    "failed to calculate directory size of {}: {}",
                                                    path.display(),
                                                    err
                                                );
                                                Message::DirectorySize(
                                                    path.clone(),
                                                    DirSize::Error(err.to_string()),
                                                )
                                            }
                                        }
                                    };

                                    match output.send(message).await {
                                        Ok(()) => {}
                                        Err(err) => {
                                            log::warn!(
                                                "failed to send directory size for {}: {}",
                                                path.display(),
                                                err
                                            );
                                        }
                                    }

                                    std::future::pending().await
                                }),
                            ));
                        }
                    }
                }
            }
        }

        // Load search items incrementally
        if let Location::Search(path, term, show_hidden, start) = &self.location {
            let location = self.location.clone();
            let path = path.clone();
            let term = term.clone();
            let show_hidden = *show_hidden;
            let start = *start;
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
                                            if ready.swap(true, atomic::Ordering::SeqCst) {
                                                true
                                            } else {
                                                // Wake up update method
                                                futures::executor::block_on(async {
                                                    output
                                                        .lock()
                                                        .await
                                                        .send(Message::SearchReady(false))
                                                        .await
                                                })
                                                .is_ok()
                                            }
                                        }
                                        Err(_) => false,
                                    }
                                },
                            );
                            log::info!(
                                "searched for {:?} in {} in {:?}",
                                term,
                                path.display(),
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

        if let Some(path) = self
            .edit_location
            .as_ref()
            .and_then(|x| x.location.path_opt())
            .cloned()
        {
            subscriptions.push(Subscription::run_with_id(
                ("tab_complete", path.to_string_lossy().into_owned()),
                stream::channel(1, |mut output| async move {
                    let message = {
                        let path = path.clone();
                        tokio::task::spawn_blocking(move || {
                            let start = Instant::now();
                            match tab_complete(&path) {
                                Ok(completions) => {
                                    log::info!(
                                        "tab completed {} in {:?}",
                                        path.display(),
                                        start.elapsed()
                                    );
                                    Message::TabComplete(path.clone(), completions)
                                }
                                Err(err) => {
                                    log::warn!(
                                        "failed to tab complete {}: {}",
                                        path.display(),
                                        err
                                    );
                                    Message::TabComplete(path.clone(), Vec::new())
                                }
                            }
                        })
                        .await
                        .unwrap()
                    };

                    match output.send(message).await {
                        Ok(()) => {}
                        Err(err) => {
                            log::warn!(
                                "failed to send tab completion for {}: {}",
                                path.display(),
                                err
                            );
                        }
                    }

                    std::future::pending().await
                }),
            ));
        }

        Subscription::batch(subscriptions)
    }

    const fn format_time(&self, time: SystemTime) -> FormatTime<'_> {
        format_time(time, &self.date_time_formatter, &self.time_formatter)
    }
}

pub fn respond_to_scroll_direction(delta: ScrollDelta, modifiers: &Modifiers) -> Option<Message> {
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

#[derive(Clone)]
pub struct RcElementWrapper<M>(pub Rc<RefCell<Element<'static, M>>>);

impl<M> Widget<M, cosmic::Theme, cosmic::Renderer> for RcElementWrapper<M> {
    fn size(&self) -> Size<Length> {
        self.0.borrow().as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.0.borrow().as_widget().size_hint()
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        renderer: &cosmic::Renderer,
        limits: &cosmic::iced_core::layout::Limits,
    ) -> cosmic::iced_core::layout::Node {
        self.0.borrow().as_widget().layout(tree, renderer, limits)
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
            .borrow()
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn tag(&self) -> tree::Tag {
        self.0.borrow().as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.0.borrow().as_widget().state()
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.0.borrow().as_widget().children()
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        self.0.borrow_mut().as_widget_mut().diff(tree);
    }

    fn operate(
        &self,
        state: &mut tree::Tree,
        layout: cosmic::iced_core::Layout<'_>,
        renderer: &cosmic::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.0
            .borrow()
            .as_widget()
            .operate(state, layout, renderer, operation);
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
        self.0.borrow_mut().as_widget_mut().on_event(
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
            .borrow()
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
        self.0.borrow().as_widget().id()
    }

    fn set_id(&mut self, _id: Id) {
        self.0.borrow_mut().as_widget_mut().set_id(_id);
    }

    fn drag_destinations(
        &self,
        _state: &tree::Tree,
        _layout: cosmic::iced_core::Layout<'_>,
        renderer: &cosmic::Renderer,
        _dnd_rectangles: &mut cosmic::iced_core::clipboard::DndDestinationRectangles,
    ) {
        self.0
            .borrow()
            .as_widget()
            .drag_destinations(_state, _layout, renderer, _dnd_rectangles);
    }
}

impl<Message: 'static> From<RcElementWrapper<Message>> for Element<'static, Message> {
    fn from(wrapper: RcElementWrapper<Message>) -> Self {
        Element::new(wrapper)
    }
}

fn text_editor_class(
    theme: &cosmic::Theme,
    status: cosmic::widget::text_editor::Status,
) -> cosmic::iced_widget::text_editor::Style {
    let cosmic = theme.cosmic();
    let container = theme.current_container();

    let mut background: cosmic::iced::Color = container.component.base.into();
    background.a = 0.25;
    let selection = cosmic.accent.base.into();
    let value = cosmic.palette.neutral_9.into();
    let mut placeholder = cosmic.palette.neutral_9;
    placeholder.alpha = 0.7;
    let placeholder = placeholder.into();
    let icon = cosmic.background.on.into();

    match status {
        cosmic::iced_widget::text_editor::Status::Active
        | cosmic::iced_widget::text_editor::Status::Disabled => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 2.0,
                    color: container.component.divider.into(),
                },
                icon,
                placeholder,
                value,
                selection,
            }
        }
        cosmic::iced_widget::text_editor::Status::Hovered
        | cosmic::iced_widget::text_editor::Status::Focused => {
            cosmic::iced_widget::text_editor::Style {
                background: background.into(),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_m.into(),
                    width: 2.0,
                    color: cosmic::iced::Color::from(cosmic.accent.base),
                },
                icon,
                placeholder,
                value,
                selection,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io, path::PathBuf};

    use cosmic::{iced::mouse::ScrollDelta, iced_runtime::keyboard::Modifiers, widget};
    use log::{debug, trace};
    use tempfile::TempDir;
    use test_log::test;

    use super::{Location, Message, Tab, respond_to_scroll_direction, scan_path};
    use crate::{
        app::test_utils::{
            NAME_LEN, NUM_DIRS, NUM_FILES, NUM_HIDDEN, NUM_NESTED, assert_eq_tab_path, empty_fs,
            eq_path_item, filter_dirs, read_dir_sorted, simple_fs, tab_click_new,
        },
        config::{IconSizes, TabConfig, ThumbCfg},
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

        for (i, (&expected, actual)) in expected_selected.iter().zip(items).enumerate() {
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
        let mut tab = Tab::new(
            Location::Path(path.into()),
            TabConfig::default(),
            ThumbCfg::default(),
            None,
            widget::Id::unique(),
            None,
        );

        // All directories (simple_fs only produces one nested layer)
        let dirs: Vec<PathBuf> = {
            let top_level = filter_dirs(path)?;
            let mut result = Vec::new();
            for dir in top_level {
                let nested_dirs = filter_dirs(&dir)?;
                result.push(dir);
                result.extend(nested_dirs);
            }
            result
        };
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
        assert!(
            entries
                .into_iter()
                .zip(actual.into_iter())
                .all(|(path, item)| eq_path_item(&path, &item))
        );

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

        let mut tab = Tab::new(
            Location::Path(path.to_owned()),
            TabConfig::default(),
            ThumbCfg::default(),
            None,
            widget::Id::unique(),
            None,
        );
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
            respond_to_scroll_direction(ScrollDelta::Pixels { x: 0.0, y: 1.0 }, &Modifiers::CTRL);
        assert!(message_maybe.is_some());
        assert!(matches!(message_maybe.unwrap(), Message::ZoomIn));
        Ok(())
    }

    #[test]
    fn tab_scroll_up_without_ctrl_modifier_does_not_zoom() -> io::Result<()> {
        let message_maybe = respond_to_scroll_direction(
            ScrollDelta::Pixels { x: 0.0, y: 1.0 },
            &Modifiers::empty(),
        );
        assert!(message_maybe.is_none());
        Ok(())
    }

    #[test]
    fn tab_scroll_down_with_ctrl_modifier_zooms() -> io::Result<()> {
        let message_maybe =
            respond_to_scroll_direction(ScrollDelta::Pixels { x: 0.0, y: -1.0 }, &Modifiers::CTRL);
        assert!(message_maybe.is_some());
        assert!(matches!(message_maybe.unwrap(), Message::ZoomOut));
        Ok(())
    }

    #[test]
    fn tab_scroll_down_without_ctrl_modifier_does_not_zoom() -> io::Result<()> {
        let message_maybe = respond_to_scroll_direction(
            ScrollDelta::Pixels { x: 0.0, y: -1.0 },
            &Modifiers::empty(),
        );
        assert!(message_maybe.is_none());
        Ok(())
    }
    #[test]
    fn tab_empty_history_does_nothing_on_prev_next() -> io::Result<()> {
        let fs = simple_fs(0, NUM_NESTED, NUM_DIRS, 0, NAME_LEN)?;
        let path = fs.path();
        let mut tab = Tab::new(
            Location::Path(path.into()),
            TabConfig::default(),
            ThumbCfg::default(),
            None,
            widget::Id::unique(),
            None,
        );

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

        let mut tab = Tab::new(
            Location::Path(next_dir.clone()),
            TabConfig::default(),
            ThumbCfg::default(),
            None,
            widget::Id::unique(),
            None,
        );
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
        let mut base_nums: Vec<_> = ('0'..='9').collect();
        fastrand::shuffle(&mut base_nums);
        debug!("Shuffled numbers for paths: {base_nums:?}");
        let paths: Vec<_> = base_nums
            .iter()
            .copied()
            .map(|base| path.join(std::iter::repeat_n(base, 255).collect::<String>()))
            .collect();

        for (file, base) in paths.iter().zip(base_nums.into_iter()) {
            trace!("Creating long file name for {base}");
            fs::File::create(file)?;
        }

        debug!("Creating tab for directory of long file names");
        Tab::new(
            Location::Path(path.into()),
            TabConfig::default(),
            ThumbCfg::default(),
            None,
            widget::Id::unique(),
            None,
        );

        Ok(())
    }

    #[test]
    fn mode_calculations() {
        use super::{
            MODE_SHIFT_GROUP, MODE_SHIFT_OTHER, MODE_SHIFT_USER, get_mode_part, set_mode_part,
        };
        for user in 0..=7 {
            for group in 0..=7 {
                for other in 0..=7 {
                    let mode = (user << MODE_SHIFT_USER)
                        | (group << MODE_SHIFT_GROUP)
                        | (other << MODE_SHIFT_OTHER);
                    assert_eq!(format!("{mode:03o}"), format!("{user:o}{group:o}{other:o}"),);
                    assert_eq!(get_mode_part(mode, MODE_SHIFT_USER), user);
                    assert_eq!(get_mode_part(mode, MODE_SHIFT_GROUP), group);
                    assert_eq!(get_mode_part(mode, MODE_SHIFT_OTHER), other);

                    let mode_no_user = (group << MODE_SHIFT_GROUP) | (other << MODE_SHIFT_OTHER);
                    assert_eq!(
                        format!("{mode_no_user:03o}"),
                        format!("0{group:o}{other:o}")
                    );
                    assert_eq!(set_mode_part(mode_no_user, MODE_SHIFT_USER, user), mode);

                    let mode_no_group = (user << MODE_SHIFT_USER) | (other << MODE_SHIFT_OTHER);
                    assert_eq!(
                        format!("{mode_no_group:03o}"),
                        format!("{user:o}0{other:o}")
                    );
                    assert_eq!(set_mode_part(mode_no_group, MODE_SHIFT_GROUP, group), mode);

                    let mode_no_other = (user << MODE_SHIFT_USER) | (group << MODE_SHIFT_GROUP);
                    assert_eq!(
                        format!("{mode_no_other:03o}"),
                        format!("{user:o}{group:o}0")
                    );
                    assert_eq!(set_mode_part(mode_no_other, MODE_SHIFT_OTHER, other), mode);
                }
            }
        }
    }
}
