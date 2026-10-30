#[cfg(feature = "desktop")]
use cosmic::desktop::fde::{DesktopEntry, get_languages_from_env};
use cosmic::iced::advanced::text;
use cosmic::iced::{Alignment, ContentFit, Length};
use cosmic::{Element, cosmic_theme, theme, widget};
use mime_guess::{Mime, mime};
use rustc_hash::FxHashMap;
use std::borrow::Cow;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};

use crate::config::IconSizes;
use crate::format_time::{date_time_formatter, format_time, time_formatter};
use crate::location::Location;
use crate::mime_icon::{mime_for_path, mime_icon};
use crate::operation::Controller;
use crate::tab::{
    MODE_NAMES, MODE_SHIFT_GROUP, MODE_SHIFT_OTHER, MODE_SHIFT_USER, Message, THUMBNAIL_SIZE,
    get_mode_part, set_mode_part, text_editor_class,
};
use crate::trash::{TrashItem, TrashItemSize, item_from_trash_entry};
use crate::{fl, format_size, mime_app};

mod fs_kind;
mod metadata;
mod state;
mod thumbnail;

pub use self::fs_kind::FsKind;
pub use self::metadata::ItemMetadata;
pub use self::state::ItemState;
pub use self::thumbnail::ItemThumbnail;

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

/// Creates an icon handle from a desktop file's Icon field value.
/// Supports both icon names (looked up in theme) and absolute paths (used directly).
fn desktop_icon_handle(icon: &str, size: u16) -> widget::icon::Handle {
    let icon_path = Path::new(icon);
    if icon_path.is_absolute() && icon_path.exists() {
        widget::icon::from_path(icon_path.to_path_buf())
    } else {
        widget::icon::from_name(icon)
            .prefer_svg(true)
            .size(size)
            .handle()
    }
}

pub fn display_name_for_file(
    path: &Path,
    name: &str,
    get_from_gvfs: bool,
    is_desktop: bool,
) -> String {
    if is_desktop {
        return get_desktop_file_display_name(path).map_or_else(
            || Item::display_name(name),
            |desktop_name| Item::display_name(desktop_name.as_str()),
        );
    } else if get_from_gvfs {
        #[cfg(feature = "gvfs")]
        {
            let file = gio::File::for_path(path);
            if let Ok(info) = gio::prelude::FileExt::query_info(
                &file,
                "standard::display-name",
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            ) {
                return Item::display_name(info.display_name().as_str());
            }
        }
    }
    Item::display_name(name)
}

pub fn folder_icon(path: &PathBuf, icon_size: u16) -> widget::icon::Handle {
    widget::icon::from_name(SPECIAL_DIRS.get(path).map_or("folder", |x| *x))
        .prefer_svg(true)
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

pub fn folder_name<P: AsRef<Path>>(path: P) -> (String, bool) {
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
                        let is_gvfs = FsKind::new(&metadata) == FsKind::Gvfs;
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

#[cfg(not(feature = "desktop"))]
fn get_desktop_file_display_name(path: &Path) -> Option<String> {
    None
}

#[cfg(not(feature = "desktop"))]
fn get_desktop_file_icon(path: &Path) -> Option<String> {
    None
}

#[cfg(feature = "desktop")]
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

#[cfg(feature = "desktop")]
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

pub fn get_filename_from_path(path: &Path) -> Result<String, String> {
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

// Whether a dir lives on a remote filesystem, according to GIO.
#[cfg(feature = "gvfs")]
fn gvfs_dir_is_remote(dir: &Path) -> bool {
    static REMOTE_CACHE: LazyLock<RwLock<FxHashMap<PathBuf, bool>>> =
        LazyLock::new(|| RwLock::new(FxHashMap::default()));

    if let Some(remote) = REMOTE_CACHE.read().unwrap().get(dir) {
        return *remote;
    }

    let remote = match gio::prelude::FileExt::query_filesystem_info(
        &gio::File::for_path(dir),
        gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE,
        gio::Cancellable::NONE,
    ) {
        Ok(info) => info.boolean(gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE),
        Err(err) => {
            log::warn!(
                "failed to get GIO filesystem info for {}: {}",
                dir.display(),
                err
            );

            //Assume remote so that the "expensive tasks" are rather skipped then actually executed.
            true
        }
    };

    REMOTE_CACHE
        .write()
        .unwrap()
        .insert(dir.to_path_buf(), remote);
    remote
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

#[cfg(feature = "gvfs")]
pub fn item_from_gvfs_info(path: PathBuf, file_info: gio::FileInfo, sizes: IconSizes) -> Item {
    let file_name = file_info
        .attribute_as_string(gio::FILE_ATTRIBUTE_STANDARD_NAME)
        .unwrap_or_default();
    let mtime = file_info.attribute_uint64(gio::FILE_ATTRIBUTE_TIME_MODIFIED);
    let mut is_desktop = false;
    let remote = path.parent().is_none_or(gvfs_dir_is_remote);
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
            is_dir,
        },
        hidden,
        image_dimensions: (!remote && mime.type_() == mime::IMAGE)
            .then(|| image::image_dimensions(&path).ok())
            .flatten(),
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
        dir_size,
        state: ItemState::default(),
    }
}

pub fn item_from_search_item(search_item: SearchItem, sizes: IconSizes) -> Item {
    match search_item {
        SearchItem::Path(path, name, metadata) => item_from_entry(path, name, metadata, sizes),
        SearchItem::Trash(entry, size) => item_from_trash_entry(entry, size, sizes),
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

    let remote = match FsKind::new(&metadata) {
        FsKind::Local => false,
        FsKind::Remote => true,
        #[cfg(feature = "gvfs")]
        FsKind::Gvfs => {
            is_gvfs = true;
            path.parent().is_none_or(gvfs_dir_is_remote)
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
        image_dimensions: None,
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        thumbnail_opt: remote.then_some(ItemThumbnail::NotImage),
        dir_size,
        state: ItemState::default(),
    }
}

pub fn item_from_path<P: Into<PathBuf>>(path: P, sizes: IconSizes) -> Result<Item, String> {
    let path = path.into();
    let name = get_filename_from_path(&path)?;
    let metadata = fs::metadata(&path)
        .map_err(|err| format!("failed to read metadata for {}: {}", path.display(), err))?;
    Ok(item_from_entry(path, name, metadata, sizes))
}

/// State of checksum computation for a file.
#[derive(Clone, Debug, Default)]
pub enum ChecksumState {
    #[default]
    NotCalculated,
    Calculating,
    Calculated(FileChecksums),
    Error(String),
}

#[derive(Clone, Debug)]
pub enum DirSize {
    Calculating(Controller),
    Directory(u64),
    NotDirectory,
    Error(String),
}

/// Checksums computed for a file. Only SHA256 is currently exposed; see
/// [`calculate_checksums`] for how to add more.
#[derive(Clone, Debug, Default)]
pub struct FileChecksums {
    pub sha256: String,
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
    pub image_dimensions: Option<(u32, u32)>,
    pub icon_handle_grid: widget::icon::Handle,
    pub icon_handle_list: widget::icon::Handle,
    pub icon_handle_list_condensed: widget::icon::Handle,
    pub thumbnail_opt: Option<ItemThumbnail>,
    pub dir_size: DirSize,
    pub state: ItemState,
}

impl Item {
    pub(crate) fn display_name(name: &str) -> String {
        // In order to wrap at periods and underscores, add a zero width space after each one
        name.replace('.', ".\u{200B}").replace('_', "_\u{200B}")
    }

    /// Text widget for a filename in grid/icon view: word-or-glyph wrapping, middle-ellipsized to 3 lines.
    pub fn grid_display_name<'a>(
        name: impl Into<Cow<'a, str>> + 'a,
    ) -> widget::Text<'a, cosmic::Theme, cosmic::Renderer> {
        widget::text::body(name)
            .wrapping(text::Wrapping::WordOrGlyph)
            .align_x(text::Alignment::Center)
            .ellipsize(text::Ellipsize::Middle(text::EllipsizeHeightLimit::Lines(
                3,
            )))
    }

    /// Text widget for a filename in list view: word-or-glyph wrapping, middle-ellipsized to 1 line.
    pub fn list_display_name<'a>(
        name: impl Into<Cow<'a, str>> + 'a,
    ) -> widget::Text<'a, cosmic::Theme, cosmic::Renderer> {
        widget::text::body(name)
            .wrapping(text::Wrapping::WordOrGlyph)
            .ellipsize(text::Ellipsize::Middle(text::EllipsizeHeightLimit::Lines(
                1,
            )))
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
        let spacing = cosmic::theme::spacing();
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
            ItemThumbnail::Text(content) => widget::text_editor::text_editor(content)
                .style(text_editor_class)
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
        } = theme::spacing();

        let mut column = widget::column::with_capacity(4).spacing(space_m);

        column = column.push(
            widget::container(self.preview())
                .center_x(Length::Fill)
                .max_height(THUMBNAIL_SIZE as f32),
        );

        let mut details = widget::column::with_capacity(8).spacing(space_xxxs);
        details = details.push(widget::selectable_text::heading(self.name.clone()));
        details = details.push(widget::text::body(fl!(
            "type",
            mime = self.mime.to_string()
        )));
        let mut settings = Vec::new();
        if let Some(mime_app_cache) = mime_app_cache_opt {
            let mime_apps = mime_app_cache.get_apps_for_mime(&self.mime, false);
            if !mime_apps.is_empty() {
                let (names, icons) = mime_apps
                    .iter()
                    .map(|(app, _)| (Cow::Owned(app.name.clone()), app.icon()))
                    .collect::<(Vec<_>, Vec<_>)>();
                settings.push(
                    widget::settings::item::builder(fl!("open-with")).control(
                        Element::from(
                            widget::dropdown(
                                names,
                                mime_apps.iter().position(|(x, _)| x.is_default(&self.mime)),
                                move |index| index,
                            )
                            .icons(Cow::Owned(icons)),
                        )
                        .map(move |index| {
                            let mime_app = &mime_apps[index].0;
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
                details = details.push(widget::selectable_text::body(fl!(
                    "item-created",
                    created = format_time(time, &date_time_formatter, &time_formatter).to_string()
                )));
            }

            if let Ok(time) = metadata.modified() {
                details = details.push(widget::selectable_text::body(fl!(
                    "item-modified",
                    modified = format_time(time, &date_time_formatter, &time_formatter).to_string()
                )));
            }

            if let Ok(time) = metadata.accessed() {
                details = details.push(widget::selectable_text::body(fl!(
                    "item-accessed",
                    accessed = format_time(time, &date_time_formatter, &time_formatter).to_string()
                )));
            }

            #[cfg(unix)]
            if let Some(path) = self.path_opt() {
                use std::os::unix::fs::MetadataExt;

                let mode = metadata.mode();

                let user_name = uzers::get_user_by_uid(metadata.uid())
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

                let group_name = uzers::get_group_by_gid(metadata.gid())
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

        if let Some(metadata) = self.file_metadata()
            && !metadata.is_dir()
            && let Some(path) = self.path_opt()
        {
            let control: Element<'_, Message> = match &self.state.checksums {
                ChecksumState::NotCalculated => widget::button::standard(fl!("calculate"))
                    .on_press(Message::CalculateChecksums(path.clone()))
                    .into(),
                ChecksumState::Calculating => widget::row::with_capacity(2)
                    .align_y(Alignment::Center)
                    .spacing(space_xxxs)
                    .push(widget::indeterminate_circular().size(16.0))
                    .push(widget::text::body(fl!("calculating")))
                    .into(),
                ChecksumState::Calculated(checksums) => {
                    let value = checksums.sha256.clone();
                    // Middle-ellipsize the digest to fit, full value on hover.
                    let value_text = widget::tooltip(
                        widget::text::body(value.clone())
                            .font(cosmic::font::mono())
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::None)
                            .ellipsize(text::Ellipsize::Middle(text::EllipsizeHeightLimit::Lines(
                                1,
                            ))),
                        widget::text::body(value.clone()),
                        widget::tooltip::Position::Bottom,
                    );
                    let copy_button = widget::button::icon(
                        widget::icon::from_name("edit-copy-symbolic").size(16),
                    )
                    .on_press(Message::CopyChecksum(value.clone()))
                    .tooltip(fl!("copy"));
                    widget::row::with_capacity(2)
                        .align_y(Alignment::Center)
                        .spacing(space_xxxs)
                        .push(value_text)
                        .push(copy_button)
                        .into()
                }
                ChecksumState::Error(err) => {
                    widget::text::body(format!("{}: {}", fl!("error"), err)).into()
                }
            };
            settings.push(
                widget::settings::item::builder(fl!("checksum", kind = "SHA256")).control(control),
            );
        }

        if let Some(path) = self.path_opt()
            && self.state.selected
        {
            column = column.push(
                widget::button::standard(fl!("open")).on_press(Message::Open(Some(path.clone()))),
            );
        }

        if !settings.is_empty() {
            let mut section = widget::settings::section();
            section = section.extend(settings);
            column = column.push(section);
        }

        column.into()
    }

    pub fn replace_view(&self, heading: String, military_time: bool) -> Element<'_, Message> {
        let cosmic_theme::Spacing { space_xxxs, .. } = theme::spacing();

        let mut row = widget::row::with_capacity(2).spacing(space_xxxs);
        row = row.push(self.preview());

        let mut column = widget::column::with_capacity(3).spacing(space_xxxs);
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

#[derive(Clone, Debug)]
pub enum SearchItem {
    Path(PathBuf, String, fs::Metadata),
    Trash(TrashItem, TrashItemSize),
}
