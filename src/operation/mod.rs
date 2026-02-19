use crate::{
    app::{ArchiveType, DialogPage, Message, REPLACE_BUTTON_ID},
    config::IconSizes,
    fl,
    spawn_detached::spawn_detached,
    tab,
};
use cosmic::iced::futures::{self, SinkExt, StreamExt, channel::mpsc::Sender, stream};
use std::{
    borrow::Cow,
    fmt::Formatter,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{Mutex as TokioMutex, mpsc};
use walkdir::WalkDir;
use zip::AesMode::Aes256;

pub use self::controller::{Controller, ControllerState};
pub mod controller;

pub use self::reader::OpReader;
pub mod reader;

use self::recursive::{Context, Method};
pub mod recursive;

async fn handle_replace(
    msg_tx: Arc<TokioMutex<Sender<Message>>>,
    file_from: PathBuf,
    file_to: PathBuf,
    multiple: bool,
) -> ReplaceResult {
    let item_from = match tab::item_from_path(file_from, IconSizes::default()) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("{err}");
            return ReplaceResult::Cancel;
        }
    };

    let item_to = match tab::item_from_path(file_to, IconSizes::default()) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("{err}");
            return ReplaceResult::Cancel;
        }
    };

    let (tx, mut rx) = mpsc::channel(1);
    let _ = msg_tx
        .lock()
        .await
        .send(Message::DialogPush(
            DialogPage::Replace {
                from: item_from,
                to: item_to,
                multiple,
                apply_to_all: false,
                tx,
            },
            Some(REPLACE_BUTTON_ID.clone()),
        ))
        .await;
    rx.recv().await.unwrap_or(ReplaceResult::Cancel)
}

fn get_directory_name(file_name: &str) -> &str {
    // TODO: Chain with COMPOUND_EXTENSIONS once more formats are supported
    for ext in crate::archive::SUPPORTED_EXTENSIONS {
        if let Some(stripped) = file_name.strip_suffix(ext) {
            return stripped;
        }
    }
    file_name
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReplaceResult {
    Replace(bool),
    KeepBoth,
    Skip(bool),
    Cancel,
}

async fn copy_or_move(
    paths: Vec<PathBuf>,
    to: PathBuf,
    method: Method,
    msg_tx: &Arc<TokioMutex<Sender<Message>>>,
    controller: Controller,
) -> Result<OperationSelection, OperationError> {
    let msg_tx = msg_tx.clone();
    let controller_c = controller.clone();

    compio::runtime::spawn(async move {
        let controller = controller_c;
        log::info!(
            "{} {:?} to {}",
            match method {
                Method::Copy => "Copy",
                Method::Move { .. } => "Move",
            },
            paths,
            to.display()
        );

        // Handle duplicate file names by renaming paths
        let mut from_to_pairs: Vec<(PathBuf, PathBuf)> = paths
            .into_iter()
            .zip(std::iter::repeat(to.as_path()))
            .filter_map(|(from, to)| {
                if matches!(from.parent(), Some(parent) if parent == to)
                    && matches!(method, Method::Copy)
                {
                    // `from`'s parent is equal to `to` which means we're copying to the same
                    // directory (duplicating files)
                    let to = copy_unique_path(&from, to);
                    Some((from, to))
                } else if let Some(name) = from.file_name() {
                    let to = to.join(name);
                    Some((from, to))
                } else {
                    //TODO: how to handle from missing file name?
                    None
                }
            })
            .collect();

        // Attempt quick and simple renames
        //TODO: allow rename to be used for directories in recursive context?
        if matches!(method, Method::Move { .. }) {
            from_to_pairs.retain(|(from, to)| {
                //TODO: show replace dialog here?
                if to.exists() {
                    return true;
                }

                //TODO: use compio::fs::rename?
                match fs::rename(from, to) {
                    Ok(()) => {
                        log::info!("renamed {} to {}", from.display(), to.display());
                        false
                    }
                    Err(err) => {
                        log::info!(
                            "failed to rename {} to {}, fallback to recursive move: {}",
                            from.display(),
                            to.display(),
                            err
                        );
                        true
                    }
                }
            });
        }

        let mut context = Context::new(controller.clone());

        {
            let controller = controller.clone();
            context = context.on_progress(move |_op, progress| {
                let item_progress = match progress.total_bytes {
                    Some(total_bytes) => {
                        if total_bytes == 0 {
                            1.0
                        } else {
                            progress.current_bytes as f32 / total_bytes as f32
                        }
                    }
                    None => 0.0,
                };
                let total_progress =
                    (item_progress + progress.current_ops as f32) / progress.total_ops as f32;
                controller.set_progress(total_progress);
            });
        }

        {
            let msg_tx = msg_tx.clone();
            context = context.on_replace(move |op| {
                let msg_tx = msg_tx.clone();
                Box::pin(handle_replace(msg_tx, op.from.clone(), op.to.clone(), true))
            });
        }

        context
            .recursive_copy_or_move(from_to_pairs, method)
            .await?;

        Result::<OperationSelection, OperationError>::Ok(context.op_sel)
    })
    .await
    .map_err(wrap_compio_spawn_error)?
}

pub async fn sync_to_disk(
    written_files: Vec<PathBuf>,
    target_dirs: std::collections::HashSet<PathBuf>,
) {
    // Sync files to disk
    stream::iter(written_files.into_iter().map(|path| async move {
        if let Ok(file) = compio::fs::OpenOptions::new().write(true).open(&path).await {
            let _ = file.sync_all().await;
        }
    }))
    .buffer_unordered(32)
    .collect::<Vec<_>>()
    .await;

    // Sync directories to disk
    stream::iter(target_dirs.into_iter().map(|path| async move {
        if let Ok(dir) = compio::fs::OpenOptions::new().read(true).open(&path).await {
            let _ = dir.sync_all().await;
        }
    }))
    .buffer_unordered(16)
    .collect::<Vec<_>>()
    .await;
}

pub fn copy_unique_path(from: &Path, to: &Path) -> PathBuf {
    // List of compound extensions to check
    const COMPOUND_EXTENSIONS: &[&str] = &[
        ".tar.gz",
        ".tar.bz2",
        ".tar.xz",
        ".tar.zst",
        ".tar.lz",
        ".tar.lzma",
        ".tar.sz",
        ".tar.lzo",
        ".tar.br",
        ".tar.Z",
        ".tar.pz",
    ];

    let mut to = to.to_owned();
    if let Some(file_name) = from.file_name().and_then(|name| name.to_str()) {
        let (stem, ext) = if from.is_dir() {
            (file_name.to_string(), None)
        } else {
            let file_name = file_name.to_string();
            COMPOUND_EXTENSIONS
                .iter()
                .copied()
                .find(|&ext| file_name.ends_with(ext))
                .map(|ext| {
                    (
                        file_name.strip_suffix(ext).unwrap().to_string(),
                        Some(ext[1..].to_string()),
                    )
                })
                .unwrap_or_else(|| {
                    from.file_stem()
                        .and_then(|s| s.to_str())
                        .map_or((file_name, None), |stem| {
                            (
                                stem.to_string(),
                                from.extension()
                                    .and_then(|e| e.to_str())
                                    .map(str::to_string),
                            )
                        })
                })
        };

        for n in 0.. {
            let new_name = if n == 0 {
                file_name.to_string()
            } else {
                match ext {
                    Some(ref ext) => format!("{} ({} {}).{}", stem, fl!("copy_noun"), n, ext),
                    None => format!("{} ({} {})", stem, fl!("copy_noun"), n),
                }
            };

            to.push(&new_name);

            if !matches!(to.try_exists(), Ok(true)) {
                break;
            }
            // Continue if a copy with index exists
            to.pop();
        }
    }
    to
}

fn file_name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .map_or_else(|| fl!("unknown-folder").into(), |x| x.to_string_lossy())
}

fn parent_name(path: &Path) -> Cow<'_, str> {
    let Some(parent) = path.parent() else {
        return fl!("unknown-folder").into();
    };

    file_name(parent)
}

fn paths_parent_name(paths: &[PathBuf]) -> Cow<'_, str> {
    let Some(first_path) = paths.first() else {
        return fl!("unknown-folder").into();
    };

    let Some(parent) = first_path.parent() else {
        return fl!("unknown-folder").into();
    };

    for path in paths {
        //TODO: is it possible to have different parents, and what should be returned?
        if path.parent() != Some(parent) {
            return fl!("unknown-folder").into();
        }
    }

    file_name(parent)
}

#[derive(Clone, Debug, Default)]
pub struct OperationSelection {
    // Paths to ignore if they are already selected
    pub ignored: Vec<PathBuf>,
    // Paths to select
    pub selected: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Operation {
    /// Compress files
    Compress {
        paths: Vec<PathBuf>,
        to: PathBuf,
        archive_type: ArchiveType,
        password: Option<String>,
    },
    /// Copy items
    Copy {
        paths: Vec<PathBuf>,
        to: PathBuf,
    },
    /// Move items to the trash
    Delete {
        paths: Vec<PathBuf>,
    },
    /// Delete a path from the trash
    DeleteTrash {
        items: Vec<trash::TrashItem>,
    },
    /// Empty the trash
    EmptyTrash,
    /// Uncompress files
    Extract {
        paths: Box<[PathBuf]>,
        to: PathBuf,
        password: Option<String>,
    },
    /// Move items
    Move {
        paths: Vec<PathBuf>,
        to: PathBuf,
        cross_device_copy: bool,
    },
    NewFile {
        path: PathBuf,
    },
    NewFolder {
        path: PathBuf,
    },
    /// Permanently delete items, skipping the trash
    PermanentlyDelete {
        paths: Box<[PathBuf]>,
    },
    RemoveFromRecents {
        paths: Box<[PathBuf]>,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
    },
    /// Restore a path from the trash
    Restore {
        items: Vec<trash::TrashItem>,
    },
    /// Set executable and launch
    SetExecutableAndLaunch {
        path: PathBuf,
    },
    /// Set permissions
    SetPermissions {
        path: PathBuf,
        mode: u32,
    },
}

#[derive(Clone, Debug)]
pub enum OperationErrorType {
    Generic(String),
    PasswordRequired,
}
#[derive(Clone, Debug)]
pub struct OperationError {
    pub kind: OperationErrorType,
}

impl OperationError {
    pub fn from_state(state: ControllerState, controller: &Controller) -> Self {
        let message = if state == ControllerState::Failed {
            controller.set_state(ControllerState::Failed);
            fl!("failed")
        } else {
            controller.cancel();
            fl!("cancelled")
        };

        Self {
            kind: OperationErrorType::Generic(message),
        }
    }

    pub fn from_err<T: ToString>(err: T, controller: &Controller) -> Self {
        controller.set_state(ControllerState::Failed);

        Self {
            kind: OperationErrorType::Generic(err.to_string()),
        }
    }

    pub fn from_kind(kind: OperationErrorType, controller: &Controller) -> Self {
        controller.set_state(ControllerState::Failed);
        Self { kind }
    }

    pub fn from_msg(m: impl Into<String>) -> Self {
        Self {
            kind: OperationErrorType::Generic(m.into()),
        }
    }
}

impl std::error::Error for OperationError {}

impl std::fmt::Display for OperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            OperationErrorType::Generic(s) => s.fmt(f),
            OperationErrorType::PasswordRequired => f.write_str("Password required"),
        }
    }
}

impl Operation {
    pub fn pending_text(&self, ratio: f32, state: ControllerState) -> String {
        let percent = (ratio * 100.0) as i32;
        let progress = || match state {
            ControllerState::Running => fl!("progress", percent = percent),
            ControllerState::Paused => fl!("progress-paused", percent = percent),
            ControllerState::Cancelled => fl!("progress-cancelled", percent = percent),
            ControllerState::Failed => fl!("progress-failed", percent = percent),
        };
        match self {
            Self::Compress { paths, to, .. } => fl!(
                "compressing",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to),
                progress = progress()
            ),
            Self::Copy { paths, to } => fl!(
                "copying",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to),
                progress = progress()
            ),
            Self::Delete { paths } => fl!(
                "moving",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = fl!("trash"),
                progress = progress()
            ),
            Self::DeleteTrash { items } => {
                fl!("deleting", items = items.len(), progress = progress())
            }
            Self::EmptyTrash => fl!("emptying-trash", progress = progress()),
            Self::Extract {
                paths,
                to,
                password: _,
            } => fl!(
                "extracting",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to),
                progress = progress()
            ),
            Self::Move { paths, to, .. } => fl!(
                "moving",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to),
                progress = progress()
            ),
            Self::NewFile { path } => fl!(
                "creating",
                name = file_name(path),
                parent = parent_name(path)
            ),
            Self::NewFolder { path } => fl!(
                "creating",
                name = file_name(path),
                parent = parent_name(path)
            ),
            Self::PermanentlyDelete { paths } => fl!("permanently-deleting", items = paths.len()),
            Self::Rename { from, to } => {
                fl!("renaming", from = file_name(from), to = file_name(to))
            }
            Self::RemoveFromRecents { paths } => fl!("removing-from-recents", items = paths.len()),
            Self::Restore { items } => fl!("restoring", items = items.len(), progress = progress()),
            Self::SetExecutableAndLaunch { path } => {
                fl!("setting-executable-and-launching", name = file_name(path))
            }
            Self::SetPermissions { path, mode } => {
                fl!(
                    "setting-permissions",
                    name = file_name(path),
                    mode = format!("{:#03o}", mode)
                )
            }
        }
    }

    pub fn completed_text(&self) -> String {
        match self {
            Self::Compress { paths, to, .. } => fl!(
                "compressed",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
            ),
            Self::Copy { paths, to } => fl!(
                "copied",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
            ),
            Self::Delete { paths } => fl!(
                "moved",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = fl!("trash")
            ),
            Self::DeleteTrash { items } => fl!("deleted", items = items.len()),
            Self::EmptyTrash => fl!("emptied-trash"),
            Self::Extract {
                paths,
                to,
                password: _,
            } => fl!(
                "extracted",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
            ),
            Self::Move { paths, to, .. } => fl!(
                "moved",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
            ),
            Self::NewFile { path } => fl!(
                "created",
                name = file_name(path),
                parent = parent_name(path)
            ),
            Self::NewFolder { path } => fl!(
                "created",
                name = file_name(path),
                parent = parent_name(path)
            ),
            Self::PermanentlyDelete { paths } => fl!("permanently-deleted", items = paths.len()),
            Self::RemoveFromRecents { paths } => fl!("removed-from-recents", items = paths.len()),
            Self::Rename { from, to } => fl!("renamed", from = file_name(from), to = file_name(to)),
            Self::Restore { items } => fl!("restored", items = items.len()),
            Self::SetExecutableAndLaunch { path } => {
                fl!("set-executable-and-launched", name = file_name(path))
            }
            Self::SetPermissions { path, mode } => {
                fl!(
                    "set-permissions",
                    name = file_name(path),
                    mode = format!("{:#03o}", mode)
                )
            }
        }
    }

    pub const fn show_progress_notification(&self) -> bool {
        // Long running operations show a progress notification
        match self {
            Self::Compress { .. }
            | Self::Copy { .. }
            | Self::Delete { .. }
            | Self::DeleteTrash { .. }
            | Self::EmptyTrash
            | Self::Extract { .. }
            | Self::Move { .. }
            | Self::PermanentlyDelete { .. }
            | Self::Restore { .. } => true,
            Self::NewFile { .. }
            | Self::NewFolder { .. }
            | Self::RemoveFromRecents { .. }
            | Self::Rename { .. }
            | Self::SetExecutableAndLaunch { .. }
            | Self::SetPermissions { .. } => false,
        }
    }

    pub fn toast(&self) -> Option<String> {
        match self {
            Self::Compress { .. } => Some(self.completed_text()),
            Self::Delete { .. } => Some(self.completed_text()),
            Self::Extract { .. } => Some(self.completed_text()),
            //TODO: more toasts
            _ => None,
        }
    }

    /// Perform the operation
    pub async fn perform(
        self,
        msg_tx: &Arc<TokioMutex<Sender<Message>>>,
        controller: Controller,
    ) -> Result<OperationSelection, OperationError> {
        let controller_clone = controller.clone();

        //TODO: IF ERROR, RETURN AN Operation THAT CAN UNDO THE CURRENT STATE
        let paths: Result<OperationSelection, OperationError> = match self {
            Self::Compress {
                paths,
                to,
                archive_type,
                password,
            } => {
                let controller_c = controller.clone();
                compio::runtime::spawn_blocking(
                    move || -> Result<OperationSelection, OperationError> {
                        let controller = controller_c;
                        let Some(relative_root) = to.parent() else {
                            return Err(OperationError::from_err(
                                format!("path {} has no parent directory", to.display()),
                                &controller,
                            ));
                        };

                        let op_sel = OperationSelection {
                            ignored: paths.clone(),
                            selected: vec![to.clone()],
                        };

                        let mut paths = paths;
                        for path in &paths.clone() {
                            if path.is_dir() {
                                let new_paths_it = WalkDir::new(path).into_iter();
                                for entry in new_paths_it.skip(1) {
                                    let entry = entry
                                        .map_err(|e| OperationError::from_err(e, &controller))?;
                                    paths.push(entry.into_path());
                                }
                            }
                        }

                        match archive_type {
                            ArchiveType::Tgz => {
                                let mut archive = fs::File::create(&to)
                                    .map(io::BufWriter::new)
                                    .map(|w| {
                                        flate2::write::GzEncoder::new(
                                            w,
                                            flate2::Compression::default(),
                                        )
                                    })
                                    .map(tar::Builder::new)
                                    .map_err(|e| OperationError::from_err(e, &controller))?;

                                let total_paths = paths.len();
                                for (i, path) in paths.iter().enumerate() {
                                    futures::executor::block_on(async {
                                        controller
                                            .check()
                                            .await
                                            .map_err(|e| OperationError::from_state(e, &controller))
                                    })?;

                                    controller.set_progress((i as f32) / total_paths as f32);

                                    if let Some(relative_path) = path
                                        .strip_prefix(relative_root)
                                        .map_err(|e| OperationError::from_err(e, &controller))?
                                        .to_str()
                                    {
                                        archive
                                            .append_path_with_name(path, relative_path)
                                            .map_err(|e| {
                                                OperationError::from_err(e, &controller)
                                            })?;
                                    }
                                }

                                archive
                                    .finish()
                                    .map_err(|e| OperationError::from_err(e, &controller))?;
                            }
                            ArchiveType::Zip => {
                                let mut archive = fs::File::create(&to)
                                    .map(io::BufWriter::new)
                                    .map(zip::ZipWriter::new)
                                    .map_err(|e| OperationError::from_err(e, &controller))?;

                                let total_paths = paths.len();
                                let mut buffer = vec![0; 4 * 1024 * 1024];
                                for (i, path) in paths.iter().enumerate() {
                                    futures::executor::block_on(async {
                                        controller
                                            .check()
                                            .await
                                            .map_err(|s| OperationError::from_state(s, &controller))
                                    })?;

                                    controller.set_progress((i as f32) / total_paths as f32);

                                    let mut zip_options = zip::write::SimpleFileOptions::default();
                                    if password.is_some() {
                                        zip_options = zip_options.with_aes_encryption(
                                            Aes256,
                                            password.as_deref().unwrap(),
                                        );
                                    }
                                    if let Some(relative_path) = path
                                        .strip_prefix(relative_root)
                                        .map_err(|e| OperationError::from_err(e, &controller))?
                                        .to_str()
                                    {
                                        if path.is_file() {
                                            let mut file = fs::File::open(path).map_err(|e| {
                                                OperationError::from_err(e, &controller)
                                            })?;
                                            let metadata = file.metadata().map_err(|e| {
                                                OperationError::from_err(e, &controller)
                                            })?;
                                            let total = metadata.len();
                                            if total >= 4 * 1024 * 1024 * 1024 {
                                                // The large file option must be enabled for files above 4 GiB
                                                zip_options = zip_options.large_file(true);
                                            }
                                            #[cfg(unix)]
                                            {
                                                use std::os::unix::fs::MetadataExt;
                                                let mode = metadata.mode();
                                                zip_options = zip_options.unix_permissions(mode);
                                            }
                                            archive
                                                .start_file(relative_path, zip_options)
                                                .map_err(|e| {
                                                    OperationError::from_err(e, &controller)
                                                })?;
                                            let mut current = 0;
                                            loop {
                                                futures::executor::block_on(async {
                                                    controller.check().await.map_err(|s| {
                                                        OperationError::from_state(s, &controller)
                                                    })
                                                })?;

                                                let count =
                                                    file.read(&mut buffer).map_err(|e| {
                                                        OperationError::from_err(e, &controller)
                                                    })?;
                                                if count == 0 {
                                                    break;
                                                }
                                                archive.write_all(&buffer[..count]).map_err(
                                                    |e| OperationError::from_err(e, &controller),
                                                )?;
                                                current += count;

                                                let file_progress = current as f32 / total as f32;
                                                let total_progress =
                                                    (i as f32 + file_progress) / total_paths as f32;
                                                controller.set_progress(total_progress);
                                            }
                                        } else {
                                            archive
                                                .add_directory(relative_path, zip_options)
                                                .map_err(|e| {
                                                    OperationError::from_err(e, &controller)
                                                })?;
                                        }
                                    }
                                }

                                archive
                                    .finish()
                                    .map_err(|e| OperationError::from_err(e, &controller))?;
                            }
                        }

                        Ok(op_sel)
                    },
                )
                .await
                .map_err(wrap_compio_spawn_error)?
            }
            Self::Copy { paths, to } => {
                copy_or_move(paths, to, Method::Copy, msg_tx, controller).await
            }
            Self::Delete { paths } => {
                let total = paths.len();
                for (i, path) in paths.into_iter().enumerate() {
                    futures::executor::block_on(async {
                        controller
                            .check()
                            .await
                            .map_err(|s| OperationError::from_state(s, &controller))
                    })?;

                    controller.set_progress((i as f32) / (total as f32));

                    let _items_opt = compio::runtime::spawn_blocking(|| trash::delete(path))
                        .await
                        .map_err(wrap_compio_spawn_error)?;
                    //TODO: items_opt allows for easy restore
                }
                Ok(OperationSelection::default())
            }
            Self::DeleteTrash { items } => {
                #[cfg(any(
                    target_os = "windows",
                    all(
                        unix,
                        not(target_os = "macos"),
                        not(target_os = "ios"),
                        not(target_os = "android")
                    )
                ))]
                {
                    let controller_clone = controller.clone();
                    compio::runtime::spawn_blocking(move || -> Result<(), OperationError> {
                        let controller = controller_clone;
                        let count = items.len();
                        for (i, item) in items.into_iter().enumerate() {
                            futures::executor::block_on(async {
                                controller
                                    .check()
                                    .await
                                    .map_err(|s| OperationError::from_state(s, &controller))
                            })?;

                            controller.set_progress(i as f32 / count as f32);

                            trash::os_limited::purge_all([item])
                                .map_err(|e| OperationError::from_err(e, &controller))?;
                        }
                        Ok(())
                    })
                    .await
                    .map_err(wrap_compio_spawn_error)?
                    .map_err(|e| OperationError::from_err(e, &controller))?;
                }
                Ok(OperationSelection::default())
            }
            Self::EmptyTrash => {
                #[cfg(any(
                    target_os = "windows",
                    all(
                        unix,
                        not(target_os = "macos"),
                        not(target_os = "ios"),
                        not(target_os = "android")
                    )
                ))]
                {
                    let controller_clone = controller.clone();
                    compio::runtime::spawn_blocking(move || -> Result<(), OperationError> {
                        let controller = controller_clone;
                        let items = trash::os_limited::list()
                            .map_err(|e| OperationError::from_err(e, &controller))?;
                        let count = items.len();
                        let mut errors: Vec<trash::Error> = Vec::new();

                        for (i, item) in items.into_iter().enumerate() {
                            futures::executor::block_on(async {
                                controller
                                    .check()
                                    .await
                                    .map_err(|s| OperationError::from_state(s, &controller))
                            })?;

                            if let Err(e) = trash::os_limited::purge_all([item]) {
                                errors.push(e);
                            }

                            controller.set_progress(i as f32 / count as f32);
                        }

                        // Report errors at the end
                        if !errors.is_empty() {
                            log::warn!("Failed to purge {} items:", errors.len());
                            for e in &errors {
                                log::warn!("  - {e}");
                            }

                            // Return an error to signal partial failure
                            return Err(OperationError::from_err(
                                format!(
                                    "Failed to delete {} of {} items. Check log for details.",
                                    errors.len(),
                                    count
                                ),
                                &controller,
                            ));
                        }

                        Ok(())
                    })
                    .await
                    .map_err(wrap_compio_spawn_error)?
                    .map_err(|e| OperationError::from_err(e, &controller))?;
                }
                Ok(OperationSelection::default())
            }
            Self::Extract {
                paths,
                to,
                password,
            } => {
                let controller_clone = controller.clone();
                compio::runtime::spawn_blocking(
                    move || -> Result<OperationSelection, OperationError> {
                        let controller = controller_clone;
                        let total_paths = paths.len();
                        let mut op_sel = OperationSelection::default();
                        for (i, path) in paths.iter().enumerate() {
                            futures::executor::block_on(async {
                                controller
                                    .check()
                                    .await
                                    .map_err(|s| OperationError::from_state(s, &controller))
                            })?;

                            controller.set_progress((i as f32) / total_paths as f32);

                            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                                let dir_name = get_directory_name(file_name);
                                let mut new_dir = to.join(dir_name);

                                if new_dir.exists()
                                    && let Some(new_dir_parent) = new_dir.parent()
                                {
                                    new_dir = copy_unique_path(&new_dir, new_dir_parent);
                                }

                                op_sel.ignored.push(path.clone());
                                op_sel.selected.push(new_dir.clone());

                                crate::archive::extract(path, &new_dir, &password, &controller)?;
                            }
                        }

                        Ok(op_sel)
                    },
                )
            }
            .await
            .map_err(wrap_compio_spawn_error)?,
            Self::Move {
                paths,
                to,
                cross_device_copy,
            } => {
                copy_or_move(
                    paths,
                    to,
                    Method::Move { cross_device_copy },
                    msg_tx,
                    controller,
                )
                .await
            }
            Self::NewFolder { path } => {
                let controller_clone = controller.clone();
                compio::runtime::spawn(async move {
                    let controller = controller_clone;
                    controller
                        .check()
                        .await
                        .map_err(|s| OperationError::from_state(s, &controller))?;
                    compio::fs::create_dir(&path)
                        .await
                        .map_err(|e| OperationError::from_err(e, &controller))?;
                    Result::<_, OperationError>::Ok(OperationSelection {
                        ignored: Vec::new(),
                        selected: vec![path],
                    })
                })
            }
            .await
            .map_err(wrap_compio_spawn_error)?,
            Self::NewFile { path } => {
                let controller_clone = controller.clone();
                compio::runtime::spawn(async move {
                    let controller = controller_clone;
                    controller
                        .check()
                        .await
                        .map_err(|s| OperationError::from_state(s, &controller))?;
                    compio::fs::File::create(&path)
                        .await
                        .map_err(|e| OperationError::from_err(e, &controller))?;
                    Result::<_, OperationError>::Ok(OperationSelection {
                        ignored: Vec::new(),
                        selected: vec![path],
                    })
                })
            }
            .await
            .map_err(wrap_compio_spawn_error)?,
            Self::PermanentlyDelete { paths } => {
                let total = paths.len();
                for (idx, path) in paths.into_iter().enumerate() {
                    controller
                        .check()
                        .await
                        .map_err(|s| OperationError::from_state(s, &controller))?;

                    controller.set_progress((idx as f32) / (total as f32));

                    tokio::task::spawn_blocking(|| {
                        if path.is_symlink() || path.is_file() {
                            fs::remove_file(path)
                        } else if path.is_dir() {
                            fs::remove_dir_all(path)
                        } else {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "File to delete is not symlink, file or directory",
                            ))
                        }
                    })
                    .await
                    .map_err(|e| OperationError::from_err(e, &controller))?
                    .map_err(|e| OperationError::from_err(e, &controller))?;
                }

                Ok(OperationSelection::default())
            }
            Self::RemoveFromRecents { paths } => {
                tokio::task::spawn_blocking(move || {
                    let path_refs = paths.iter().map(PathBuf::as_path).collect::<Box<[_]>>();
                    recently_used_xbel::remove_recently_used(&path_refs)
                })
                .await
                .map_err(|e| OperationError::from_err(e, &controller))?
                .map_err(|e| OperationError::from_err(e, &controller))?;

                Ok(OperationSelection::default())
            }
            Self::Rename { from, to } => {
                let controller_clone = controller.clone();

                compio::runtime::spawn(async move {
                    let controller = controller_clone;
                    controller
                        .check()
                        .await
                        .map_err(|s| OperationError::from_state(s, &controller))?;
                    compio::fs::rename(&from, &to)
                        .await
                        .map_err(|e| OperationError::from_err(e, &controller))?;
                    Result::<_, OperationError>::Ok(OperationSelection {
                        ignored: vec![from],
                        selected: vec![to],
                    })
                })
            }
            .await
            .map_err(wrap_compio_spawn_error)?,
            #[cfg(target_os = "macos")]
            Self::Restore { .. } => {
                // TODO: add support for macos
                return OperationError::from_msg("Restoring from trash is not supported on macos");
            }
            #[cfg(not(target_os = "macos"))]
            Self::Restore { items } => {
                let total = items.len();
                let mut paths = Vec::with_capacity(total);
                for (i, item) in items.into_iter().enumerate() {
                    controller
                        .check()
                        .await
                        .map_err(|s| OperationError::from_state(s, &controller))?;

                    controller.set_progress((i as f32) / (total as f32));

                    paths.push(item.original_path());

                    compio::runtime::spawn_blocking(|| trash::os_limited::restore_all([item]))
                        .await
                        .map_err(wrap_compio_spawn_error)?
                        .map_err(|e| OperationError::from_err(e, &controller))?;
                }
                Ok(OperationSelection {
                    ignored: Vec::new(),
                    selected: paths,
                })
            }
            Self::SetExecutableAndLaunch { path } => {
                controller
                    .check()
                    .await
                    .map_err(|s| OperationError::from_state(s, &controller))?;

                let controller_clone = controller.clone();
                compio::runtime::spawn_blocking(move || -> Result<(), OperationError> {
                    let controller = controller_clone;
                    //TODO: what to do on non-Unix systems?
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;

                        let mut perms = fs::metadata(&path)
                            .map_err(|e| OperationError::from_err(e, &controller))?
                            .permissions();
                        let current_mode = perms.mode();
                        let new_mode = current_mode | 0o111;
                        perms.set_mode(new_mode);
                        fs::set_permissions(&path, perms)
                            .map_err(|e| OperationError::from_err(e, &controller))?;
                    }

                    let mut command = std::process::Command::new(path);
                    spawn_detached(&mut command)
                        .map_err(|e| OperationError::from_err(e, &controller))?;

                    Ok(())
                })
                .await
                .map_err(wrap_compio_spawn_error)?
                .map_err(|e| OperationError::from_err(e, &controller))?;
                Ok(OperationSelection::default())
            }
            Self::SetPermissions { path, mode } => {
                controller
                    .check()
                    .await
                    .map_err(|s| OperationError::from_state(s, &controller))?;

                let controller_clone = controller.clone();
                let path_clone = path.clone();
                compio::runtime::spawn_blocking(move || -> Result<(), OperationError> {
                    let controller = controller_clone;
                    let path = path_clone;
                    //TODO: what to do on non-Unix systems?
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let perms = fs::Permissions::from_mode(mode);
                        fs::set_permissions(&path, perms)
                            .map_err(|e| OperationError::from_err(e, &controller))?;
                    }

                    Ok(())
                })
                .await
                .map_err(wrap_compio_spawn_error)?
                .map_err(|e| OperationError::from_err(e, &controller))?;
                Ok(OperationSelection {
                    ignored: Vec::new(),
                    selected: vec![path],
                })
            }
        };

        controller_clone.set_progress(1.0);

        paths
    }
}

#[track_caller]
fn wrap_compio_spawn_error(err: Box<dyn std::any::Any + Send>) -> OperationError {
    log::error!(
        "compio runtime spawn failed: {}",
        std::backtrace::Backtrace::capture()
    );

    // Preserve error if it's already an OperationError
    if let Ok(err) = err.downcast() {
        *err
    } else {
        OperationError::from_msg("compio runtime spawn failed")
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io,
        path::PathBuf,
    };

    use cosmic::iced::futures::{StreamExt, channel::mpsc, future};
    use log::debug;
    use test_log::test;
    use tokio::sync;

    use super::{Controller, Operation, OperationError, OperationSelection, ReplaceResult};
    use crate::{
        app::{
            DialogPage, Message,
            test_utils::{
                NAME_LEN, NUM_DIRS, NUM_FILES, NUM_HIDDEN, NUM_NESTED, empty_fs, filter_dirs,
                filter_files, simple_fs,
            },
        },
        fl,
    };

    /// Simple wrapper around `[Operation::Copy]`
    pub async fn operation_copy(
        paths: Vec<PathBuf>,
        to: PathBuf,
    ) -> Result<OperationSelection, OperationError> {
        let id = fastrand::u64(0..u64::MAX);
        let (tx, mut rx) = mpsc::channel(1);
        let paths_clone = paths.clone();
        let to_clone = to.clone();

        // Wrap this into its own future so that it may be polled concurerntly with the message handler.
        let handle_copy = async move {
            Operation::Copy {
                paths: paths_clone,
                to: to_clone,
            }
            .perform(&sync::Mutex::new(tx).into(), Controller::default())
            .await
        };

        // Concurrently handling messages will prevent the mpsc channel from blocking when full.
        let handle_messages = async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    Message::DialogPush(DialogPage::Replace { tx, .. }, _id_to_focus) => {
                        debug!("[{id}] Replace request");
                        tx.send(ReplaceResult::Cancel)
                            .await
                            .expect("Sending a response to a replace request should succeed");
                    }
                    _ => unreachable!(
                        "Only [ `Message::PendingProgress`, `Message::DialogPush(DialogPage::Replace)` ] are sent from operation"
                    ),
                }
            }
        };

        future::join(handle_messages, handle_copy).await.1
    }

    #[test(compio::test)]
    async fn copy_file_to_same_location() -> io::Result<()> {
        let fs = simple_fs(NUM_FILES, 0, 1, 0, NAME_LEN)?;
        let path = fs.path();

        // Get the first file from the first directory
        let first_dir = filter_dirs(path)?
            .next()
            .expect("Should have at least one directory");
        let first_file = filter_files(&first_dir)?
            .next()
            .expect("Should have at least one file");

        // Duplicate that file
        let base_name = first_file
            .file_name()
            .and_then(|name| name.to_str())
            .expect("File name exists and is valid");
        debug!(
            "Duplicating {} in {}",
            first_file.display(),
            first_dir.display()
        );
        operation_copy(vec![first_file.clone()], first_dir.clone())
            .await
            .expect("Copy operation should have succeeded");

        assert!(first_file.exists(), "Original file should still exist");
        let expected = first_dir.join(format!("{base_name} ({} 1)", fl!("copy_noun")));
        assert!(expected.exists(), "File should have been duplicated");

        Ok(())
    }

    #[test(compio::test)]
    async fn copy_file_with_extension_to_same_loc() -> io::Result<()> {
        let fs = empty_fs()?;
        let path = fs.path();

        let base_name = "foo.txt";
        let base_path = path.join(base_name);
        File::create(&base_path)?;
        debug!("Duplicating {}", base_path.display());
        operation_copy(vec![base_path.clone()], path.to_owned())
            .await
            .expect("Copy operation should have succeeded");

        assert!(base_path.exists(), "Original file should still exist");
        let expected = path.join(format!("foo ({} 1).txt", fl!("copy_noun")));
        assert!(expected.exists(), "File should have been duplicated");

        Ok(())
    }

    #[test(compio::test)]
    async fn copy_dir_to_same_location() -> io::Result<()> {
        let fs = simple_fs(NUM_FILES, 0, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        // First directory path
        let first_dir = filter_dirs(path)?
            .next()
            .expect("Should have at least one directory");
        let base_name = first_dir
            .file_name()
            .and_then(|name| name.to_str())
            .expect("First directory exists and has a valid name");
        debug!("Duplicating directory {}", first_dir.display());
        operation_copy(vec![first_dir.clone()], path.to_owned())
            .await
            .expect("Copy operation should have succeeded");

        assert!(first_dir.exists(), "Original directory should still exist");
        let expected = path.join(format!("{base_name} ({} 1)", fl!("copy_noun")));
        assert!(expected.exists(), "Directory should have been duplicated");

        Ok(())
    }

    #[test(compio::test)]
    async fn copying_file_multiple_times_to_same_location() -> io::Result<()> {
        let fs = empty_fs()?;
        let path = fs.path();

        let base_name = "cosmic";
        let base_path = path.join(base_name);
        File::create(&base_path)?;

        for i in 1..5 {
            debug!("Duplicating {}", base_path.display());
            operation_copy(vec![base_path.clone()], path.to_owned())
                .await
                .expect("Copy operation should have succeeded");
            assert!(base_path.exists(), "Original file should still exist");
            assert!(
                path.join(format!("{base_name} ({} {i})", fl!("copy_noun")))
                    .exists(),
                "File should have been duplicated (copy #{i})"
            );
        }

        Ok(())
    }

    #[test(compio::test)]
    async fn copy_to_diff_dir_doesnt_dupe_files() -> io::Result<()> {
        let fs = simple_fs(NUM_FILES, NUM_HIDDEN, NUM_DIRS, NUM_NESTED, NAME_LEN)?;
        let path = fs.path();

        let (first_dir, second_dir) = {
            let mut dirs = filter_dirs(path)?;
            (
                dirs.next().expect("Should have at least two dirs"),
                dirs.next().expect("Should have at least two dirs"),
            )
        };
        let first_file = filter_files(&first_dir)?
            .next()
            .expect("Should have at least one file");
        // Both directories have a file with the same name.
        let base_name = first_file
            .file_name()
            .and_then(|name| name.to_str())
            .expect("File name exists and is valid");

        debug!(
            "Copying {} to {}",
            first_file.display(),
            second_dir.display()
        );
        operation_copy(vec![first_file.clone()], second_dir.clone())
            .await
            .expect(concat!(
                "Copy operation should have been cancelled ",
                "because we're copying to different directories ",
                "without replacement"
            ));
        assert!(
            first_dir.join(base_name).exists(),
            "First file should still exist"
        );
        assert!(
            second_dir.join(base_name).exists(),
            "Second file should still exist"
        );

        Ok(())
    }

    #[test(compio::test)]
    async fn copy_file_with_diff_name_to_diff_dir() -> io::Result<()> {
        let fs = empty_fs()?;
        let path = fs.path();

        let dir_path = path.join("cosmic");
        fs::create_dir(&dir_path)?;
        let file_path = path.join("ferris");
        File::create(&file_path)?;
        let expected = dir_path.join("ferris");

        debug!("Copying {} to {}", file_path.display(), expected.display());
        operation_copy(vec![file_path.clone()], dir_path.clone())
            .await
            .expect("Copy operation should have succeeded");

        assert!(file_path.exists(), "Original file should still exist");
        assert!(expected.exists(), "File should have been copied");

        Ok(())
    }
}
