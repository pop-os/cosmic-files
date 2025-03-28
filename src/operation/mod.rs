use crate::{
    app::{ArchiveType, DialogPage, Message},
    config::IconSizes,
    fl,
    mime_icon::mime_for_path,
    spawn_detached::spawn_detached,
    tab,
};
use cosmic::iced::futures::{channel::mpsc::Sender, executor, SinkExt};
use std::collections::VecDeque;
use std::fmt::Formatter;
use std::{
    borrow::Cow,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex as TokioMutex};
use walkdir::WalkDir;
use zip::result::ZipError;
use zip::AesMode::Aes256;

pub use self::controller::{Controller, ControllerState};
pub mod controller;

use self::reader::OpReader;
pub mod reader;

use self::recursive::Context;
pub mod recursive;

fn handle_replace(
    msg_tx: &Arc<TokioMutex<Sender<Message>>>,
    file_from: PathBuf,
    file_to: PathBuf,
    multiple: bool,
) -> ReplaceResult {
    let item_from = match tab::item_from_path(file_from, IconSizes::default()) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("{}", err);
            return ReplaceResult::Cancel;
        }
    };

    let item_to = match tab::item_from_path(file_to, IconSizes::default()) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("{}", err);
            return ReplaceResult::Cancel;
        }
    };

    executor::block_on(async {
        let (tx, mut rx) = mpsc::channel(1);
        let _ = msg_tx
            .lock()
            .await
            .send(Message::DialogPush(DialogPage::Replace {
                from: item_from,
                to: item_to,
                multiple,
                apply_to_all: false,
                tx,
            }))
            .await;
        rx.recv().await.unwrap_or(ReplaceResult::Cancel)
    })
}

fn get_directory_name(file_name: &str) -> &str {
    // TODO: Chain with COMPOUND_EXTENSIONS once more formats are supported
    const SUPPORTED_EXTENSIONS: &[&str] = &[
        ".tar.bz2",
        ".tar.gz",
        ".tar.lzma",
        ".tar.xz",
        ".tgz",
        ".tar",
        ".zip",
    ];

    for ext in SUPPORTED_EXTENSIONS {
        if let Some(stripped) = file_name.strip_suffix(ext) {
            return stripped;
        }
    }
    file_name
}

// From https://docs.rs/zip/latest/zip/read/struct.ZipArchive.html#method.extract, with cancellation and progress added
fn zip_extract<R: io::Read + io::Seek, P: AsRef<Path>>(
    archive: &mut zip::ZipArchive<R>,
    directory: P,
    controller: Controller,
    password: Option<String>,
) -> zip::result::ZipResult<()> {
    use std::{ffi::OsString, fs};
    use zip::result::ZipError;

    fn make_writable_dir_all<T: AsRef<Path>>(outpath: T) -> Result<(), ZipError> {
        fs::create_dir_all(outpath.as_ref())?;
        #[cfg(unix)]
        {
            // Dirs must be writable until all normal files are extracted
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                outpath.as_ref(),
                std::fs::Permissions::from_mode(
                    0o700 | std::fs::metadata(outpath.as_ref())?.permissions().mode(),
                ),
            )?;
        }
        Ok(())
    }

    #[cfg(unix)]
    let mut files_by_unix_mode = Vec::new();
    let mut buffer = vec![0; 4 * 1024 * 1024];
    let total_files = archive.len();
    let mut pending_directory_creates = VecDeque::new();

    for i in 0..total_files {
        controller
            .check()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        controller.set_progress((i as f32) / total_files as f32);

        let mut file = match &password {
            None => archive.by_index(i),
            Some(pwd) => archive.by_index_decrypt(i, pwd.as_bytes()),
        }
        .map_err(|e| e)?;
        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("Invalid file path"))?;

        let outpath = directory.as_ref().join(filepath);

        if file.is_dir() {
            pending_directory_creates.push_back(outpath.clone());
            continue;
        }
        let symlink_target = if file.is_symlink() && (cfg!(unix) || cfg!(windows)) {
            let mut target = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut target)?;
            Some(target)
        } else {
            None
        };
        drop(file);
        if let Some(target) = symlink_target {
            // create all pending dirs
            while let Some(pending_dir) = pending_directory_creates.pop_front() {
                make_writable_dir_all(pending_dir)?;
            }

            if let Some(p) = outpath.parent() {
                make_writable_dir_all(p)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStringExt;
                let target = OsString::from_vec(target);
                std::os::unix::fs::symlink(&target, outpath.as_path())?;
            }
            #[cfg(windows)]
            {
                let Ok(target) = String::from_utf8(target) else {
                    return Err(ZipError::InvalidArchive("Invalid UTF-8 as symlink target"));
                };
                let target = target.into_boxed_str();
                let target_is_dir_from_archive =
                    archive.shared.files.contains_key(&target) && is_dir(&target);
                let target_path = directory.as_ref().join(OsString::from(target.to_string()));
                let target_is_dir = if target_is_dir_from_archive {
                    true
                } else if let Ok(meta) = std::fs::metadata(&target_path) {
                    meta.is_dir()
                } else {
                    false
                };
                if target_is_dir {
                    std::os::windows::fs::symlink_dir(target_path, outpath.as_path())?;
                } else {
                    std::os::windows::fs::symlink_file(target_path, outpath.as_path())?;
                }
            }
            continue;
        }
        let mut file = match &password {
            None => archive.by_index(i),
            Some(pwd) => archive.by_index_decrypt(i, pwd.as_bytes()),
        }
        .map_err(|e| e)?;

        // create all pending dirs
        while let Some(pending_dir) = pending_directory_creates.pop_front() {
            make_writable_dir_all(pending_dir)?;
        }

        if let Some(p) = outpath.parent() {
            make_writable_dir_all(p)?;
        }

        let total = file.size();
        let mut outfile = fs::File::create(&outpath)?;
        let mut current = 0;
        loop {
            controller
                .check()
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            outfile.write_all(&buffer[..count])?;
            current += count as u64;

            if current < total {
                let file_progress = current as f32 / total as f32;
                let total_progress = (i as f32 + file_progress) / total_files as f32;
                controller.set_progress(total_progress);
            }
        }
        outfile.sync_all()?;
        #[cfg(unix)]
        {
            // Check for real permissions, which we'll set in a second pass
            if let Some(mode) = file.unix_mode() {
                files_by_unix_mode.push((outpath.clone(), mode));
            }
        }
    }
    #[cfg(unix)]
    {
        use std::cmp::Reverse;
        use std::os::unix::fs::PermissionsExt;

        if files_by_unix_mode.len() > 1 {
            // Ensure we update children's permissions before making a parent unwritable
            files_by_unix_mode.sort_by_key(|(path, _)| Reverse(path.clone()));
        }
        for (path, mode) in files_by_unix_mode.into_iter() {
            fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;
        }
    }
    Ok(())
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
    moving: bool,
    msg_tx: &Arc<TokioMutex<Sender<Message>>>,
    controller: Controller,
) -> Result<OperationSelection, OperationError> {
    let msg_tx = msg_tx.clone();
    tokio::task::spawn_blocking(move || -> Result<OperationSelection, OperationError> {
        log::info!(
            "{} {:?} to {:?}",
            if moving { "Move" } else { "Copy" },
            paths,
            to
        );

        // Handle duplicate file names by renaming paths
        let from_to_pairs: Vec<(PathBuf, PathBuf)> = paths
            .into_iter()
            .zip(std::iter::repeat(to.as_path()))
            .filter_map(|(from, to)| {
                if matches!(from.parent(), Some(parent) if parent == to) && !moving {
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

        let mut context = Context::new(controller.clone());

        {
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
                handle_replace(&msg_tx, op.from.clone(), op.to.clone(), true)
            });
        }

        context
            .recursive_copy_or_move(from_to_pairs, moving)
            .map_err(OperationError::from_str)?;

        Ok(context.op_sel)
    })
    .await
    .map_err(OperationError::from_str)?
    //.map_err(OperationError::from_str)
}

fn copy_unique_path(from: &Path, to: &Path) -> PathBuf {
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
                .find(|&&ext| file_name.ends_with(ext))
                .map(|&ext| {
                    (
                        file_name.strip_suffix(ext).unwrap().to_string(),
                        Some(ext[1..].to_string()),
                    )
                })
                .unwrap_or_else(|| {
                    from.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|stem| {
                            (
                                stem.to_string(),
                                from.extension()
                                    .and_then(|e| e.to_str())
                                    .map(|e| e.to_string()),
                            )
                        })
                        .unwrap_or((file_name, None))
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

            to = to.join(&new_name);

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

    for path in paths.iter() {
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
        paths: Vec<PathBuf>,
        to: PathBuf,
        password: Option<String>,
    },
    /// Move items
    Move {
        paths: Vec<PathBuf>,
        to: PathBuf,
    },
    NewFile {
        path: PathBuf,
    },
    NewFolder {
        path: PathBuf,
    },
    /// Permanently delete items, skipping the trash
    PermanentlyDelete {
        paths: Vec<PathBuf>,
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
    pub fn from_str<T: ToString>(err: T) -> Self {
        OperationError {
            kind: OperationErrorType::Generic(err.to_string()),
        }
    }
}

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
            Self::Move { paths, to } => fl!(
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
            Self::Restore { items } => fl!("restoring", items = items.len(), progress = progress()),
            Self::SetExecutableAndLaunch { path } => {
                fl!("setting-executable-and-launching", name = file_name(path))
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
            Self::Move { paths, to } => fl!(
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
            Self::Rename { from, to } => fl!("renamed", from = file_name(from), to = file_name(to)),
            Self::Restore { items } => fl!("restored", items = items.len()),
            Self::SetExecutableAndLaunch { path } => {
                fl!("set-executable-and-launched", name = file_name(path))
            }
        }
    }

    pub fn show_progress_notification(&self) -> bool {
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
            | Self::Rename { .. }
            | Self::SetExecutableAndLaunch { .. } => false,
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
        let paths = match self {
            Self::Compress {
                paths,
                to,
                archive_type,
                password,
            } => {
                tokio::task::spawn_blocking(
                    move || -> Result<OperationSelection, OperationError> {
                        let Some(relative_root) = to.parent() else {
                            return Err(OperationError::from_str(format!(
                                "path {:?} has no parent directory",
                                to
                            )));
                        };

                        let op_sel = OperationSelection {
                            ignored: paths.clone(),
                            selected: vec![to.clone()],
                        };

                        let mut paths = paths;
                        for path in paths.clone().iter() {
                            if path.is_dir() {
                                let new_paths_it = WalkDir::new(path).into_iter();
                                for entry in new_paths_it.skip(1) {
                                    let entry = entry.map_err(OperationError::from_str)?;
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
                                    .map_err(OperationError::from_str)?;

                                let total_paths = paths.len();
                                for (i, path) in paths.iter().enumerate() {
                                    controller.check().map_err(OperationError::from_str)?;

                                    controller.set_progress((i as f32) / total_paths as f32);

                                    if let Some(relative_path) = path
                                        .strip_prefix(relative_root)
                                        .map_err(OperationError::from_str)?
                                        .to_str()
                                    {
                                        archive
                                            .append_path_with_name(path, relative_path)
                                            .map_err(OperationError::from_str)?;
                                    }
                                }

                                archive.finish().map_err(OperationError::from_str)?;
                            }
                            ArchiveType::Zip => {
                                let mut archive = fs::File::create(&to)
                                    .map(io::BufWriter::new)
                                    .map(zip::ZipWriter::new)
                                    .map_err(OperationError::from_str)?;

                                let total_paths = paths.len();
                                let mut buffer = vec![0; 4 * 1024 * 1024];
                                for (i, path) in paths.iter().enumerate() {
                                    controller.check().map_err(OperationError::from_str)?;

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
                                        .map_err(OperationError::from_str)?
                                        .to_str()
                                    {
                                        if path.is_file() {
                                            let mut file = fs::File::open(path)
                                                .map_err(OperationError::from_str)?;
                                            let metadata = file
                                                .metadata()
                                                .map_err(OperationError::from_str)?;
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
                                                .map_err(OperationError::from_str)?;
                                            let mut current = 0;
                                            loop {
                                                controller
                                                    .check()
                                                    .map_err(OperationError::from_str)?;

                                                let count = file
                                                    .read(&mut buffer)
                                                    .map_err(OperationError::from_str)?;
                                                if count == 0 {
                                                    break;
                                                }
                                                archive
                                                    .write_all(&buffer[..count])
                                                    .map_err(OperationError::from_str)?;
                                                current += count;

                                                let file_progress = current as f32 / total as f32;
                                                let total_progress =
                                                    (i as f32 + file_progress) / total_paths as f32;
                                                controller.set_progress(total_progress);
                                            }
                                        } else {
                                            archive
                                                .add_directory(relative_path, zip_options)
                                                .map_err(OperationError::from_str)?;
                                        }
                                    }
                                }

                                archive.finish().map_err(OperationError::from_str)?;
                            }
                        }

                        Ok(op_sel)
                    },
                )
                .await
                .map_err(OperationError::from_str)?
                //.map_err(|e| e)?
            }
            Self::Copy { paths, to } => copy_or_move(paths, to, false, msg_tx, controller).await,
            Self::Delete { paths } => {
                let total = paths.len();
                for (i, path) in paths.into_iter().enumerate() {
                    controller.check().map_err(OperationError::from_str)?;

                    controller.set_progress((i as f32) / (total as f32));

                    let _items_opt = tokio::task::spawn_blocking(|| trash::delete(path))
                        .await
                        .map_err(OperationError::from_str)?
                        .map_err(OperationError::from_str)?;
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
                    tokio::task::spawn_blocking(move || -> Result<(), OperationError> {
                        let count = items.len();
                        for (i, item) in items.into_iter().enumerate() {
                            controller.check().map_err(OperationError::from_str)?;

                            controller.set_progress(i as f32 / count as f32);

                            trash::os_limited::purge_all([item])
                                .map_err(OperationError::from_str)?;
                        }
                        Ok(())
                    })
                    .await
                    .map_err(OperationError::from_str)??;
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
                    tokio::task::spawn_blocking(move || -> Result<(), OperationError> {
                        let items = trash::os_limited::list().map_err(OperationError::from_str)?;
                        let count = items.len();
                        for (i, item) in items.into_iter().enumerate() {
                            controller.check().map_err(OperationError::from_str)?;

                            controller.set_progress(i as f32 / count as f32);

                            trash::os_limited::purge_all([item])
                                .map_err(OperationError::from_str)?;
                        }
                        Ok(())
                    })
                    .await
                    .map_err(OperationError::from_str)??;
                }
                Ok(OperationSelection::default())
            }
            Self::Extract {
                paths,
                to,
                password,
            } => {
                tokio::task::spawn_blocking(
                    move || -> Result<OperationSelection, OperationError> {
                        let total_paths = paths.len();
                        let mut op_sel = OperationSelection::default();
                        for (i, path) in paths.iter().enumerate() {
                            controller.check().map_err(OperationError::from_str)?;

                            controller.set_progress((i as f32) / total_paths as f32);

                            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                                let dir_name = get_directory_name(file_name);
                                let mut new_dir = to.join(dir_name);

                                if new_dir.exists() {
                                    if let Some(new_dir_parent) = new_dir.parent() {
                                        new_dir = copy_unique_path(&new_dir, new_dir_parent);
                                    }
                                }

                                op_sel.ignored.push(path.clone());
                                op_sel.selected.push(new_dir.clone());

                                let controller = controller.clone();
                                let mime = mime_for_path(path);
                                let password = password.clone();
                                match mime.essence_str() {
                                    "application/gzip" | "application/x-compressed-tar" => {
                                        OpReader::new(path, controller)
                                            .map(io::BufReader::new)
                                            .map(flate2::read::GzDecoder::new)
                                            .map(tar::Archive::new)
                                            .and_then(|mut archive| archive.unpack(&new_dir))
                                            .map_err(OperationError::from_str)?
                                    }
                                    "application/x-tar" => OpReader::new(path, controller)
                                        .map(io::BufReader::new)
                                        .map(tar::Archive::new)
                                        .and_then(|mut archive| archive.unpack(&new_dir))
                                        .map_err(OperationError::from_str)?,
                                    "application/zip" => fs::File::open(path)
                                        .map(io::BufReader::new)
                                        .map(zip::ZipArchive::new)
                                        .map_err(OperationError::from_str)?
                                        .and_then(move |mut archive| {
                                            zip_extract(
                                                &mut archive,
                                                &new_dir,
                                                controller,
                                                password,
                                            )
                                        })
                                        .map_err(|e| match e {
                                            ZipError::UnsupportedArchive(
                                                ZipError::PASSWORD_REQUIRED,
                                            )
                                            | ZipError::InvalidPassword => OperationError {
                                                kind: OperationErrorType::PasswordRequired,
                                            },
                                            _ => OperationError::from_str(e),
                                        })?,
                                    #[cfg(feature = "bzip2")]
                                    "application/x-bzip" | "application/x-bzip-compressed-tar" => {
                                        OpReader::new(path, controller)
                                            .map(io::BufReader::new)
                                            .map(bzip2::read::BzDecoder::new)
                                            .map(tar::Archive::new)
                                            .and_then(|mut archive| archive.unpack(&new_dir))
                                            .map_err(OperationError::from_str)?
                                    }
                                    #[cfg(feature = "liblzma")]
                                    "application/x-xz" | "application/x-xz-compressed-tar" => {
                                        OpReader::new(path, controller)
                                            .map(io::BufReader::new)
                                            .map(liblzma::read::XzDecoder::new)
                                            .map(tar::Archive::new)
                                            .and_then(|mut archive| archive.unpack(&new_dir))
                                            .map_err(OperationError::from_str)?
                                    }
                                    _ => Err(OperationError::from_str(format!(
                                        "unsupported mime type {:?}",
                                        mime
                                    )))?,
                                }
                            }
                        }

                        Ok(op_sel)
                    },
                )
                .await
                .map_err(OperationError::from_str)?
                //.map_err(OperationError::from_str)?
            }
            Self::Move { paths, to } => copy_or_move(paths, to, true, msg_tx, controller).await,
            Self::NewFolder { path } => tokio::task::spawn_blocking(
                move || -> Result<OperationSelection, OperationError> {
                    controller.check().map_err(OperationError::from_str)?;
                    fs::create_dir(&path).map_err(OperationError::from_str)?;
                    Ok(OperationSelection {
                        ignored: Vec::new(),
                        selected: vec![path],
                    })
                },
            )
            .await
            .map_err(OperationError::from_str)?,
            Self::NewFile { path } => tokio::task::spawn_blocking(
                move || -> Result<OperationSelection, OperationError> {
                    controller.check().map_err(OperationError::from_str)?;
                    fs::File::create(&path).map_err(OperationError::from_str)?;
                    Ok(OperationSelection {
                        ignored: Vec::new(),
                        selected: vec![path],
                    })
                },
            )
            .await
            .map_err(OperationError::from_str)?,
            Self::PermanentlyDelete { paths } => {
                let total = paths.len();
                for (idx, path) in paths.into_iter().enumerate() {
                    controller.check().map_err(OperationError::from_str)?;

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
                    .map_err(OperationError::from_str)?
                    .map_err(OperationError::from_str)?;
                }

                Ok(OperationSelection::default())
            }
            Self::Rename { from, to } => tokio::task::spawn_blocking(
                move || -> Result<OperationSelection, OperationError> {
                    controller.check().map_err(OperationError::from_str)?;
                    fs::rename(&from, &to).map_err(OperationError::from_str)?;
                    Ok(OperationSelection {
                        ignored: vec![from],
                        selected: vec![to],
                    })
                },
            )
            .await
            .map_err(OperationError::from_str)?,
            #[cfg(target_os = "macos")]
            Self::Restore { .. } => {
                // TODO: add support for macos
                return Err("Restoring from trash is not supported on macos".to_string());
            }
            #[cfg(not(target_os = "macos"))]
            Self::Restore { items } => {
                let total = items.len();
                let mut paths = Vec::with_capacity(total);
                for (i, item) in items.into_iter().enumerate() {
                    controller.check().map_err(OperationError::from_str)?;

                    controller.set_progress((i as f32) / (total as f32));

                    paths.push(item.original_path());

                    tokio::task::spawn_blocking(|| trash::os_limited::restore_all([item]))
                        .await
                        .map_err(OperationError::from_str)?
                        .map_err(OperationError::from_str)?;
                }
                Ok(OperationSelection {
                    ignored: Vec::new(),
                    selected: paths,
                })
            }
            Self::SetExecutableAndLaunch { path } => {
                tokio::task::spawn_blocking(move || -> Result<(), OperationError> {
                    //TODO: what to do on non-Unix systems?
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;

                        controller.check().map_err(OperationError::from_str)?;

                        let mut perms = fs::metadata(&path)
                            .map_err(OperationError::from_str)?
                            .permissions();
                        let current_mode = perms.mode();
                        let new_mode = current_mode | 0o111;
                        perms.set_mode(new_mode);
                        fs::set_permissions(&path, perms).map_err(OperationError::from_str)?;
                    }

                    controller.check().map_err(OperationError::from_str)?;

                    let mut command = std::process::Command::new(path);
                    spawn_detached(&mut command).map_err(OperationError::from_str)?;

                    Ok(())
                })
                .await
                .map_err(OperationError::from_str)?
                .map_err(|e| e)?;
                Ok(OperationSelection::default())
            }
        };

        controller_clone.set_progress(100.0);

        paths
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io,
        path::PathBuf,
    };

    use cosmic::iced::futures::{channel::mpsc, StreamExt};
    use log::debug;
    use test_log::test;
    use tokio::sync;

    use super::{Controller, Operation, OperationError, OperationSelection, ReplaceResult};
    use crate::{
        app::{
            test_utils::{
                empty_fs, filter_dirs, filter_files, simple_fs, NAME_LEN, NUM_DIRS, NUM_FILES,
                NUM_HIDDEN, NUM_NESTED,
            },
            DialogPage, Message,
        },
        fl,
    };

    // Tests hang with lower values
    const BUF_SIZE: usize = 8;

    /// Simple wrapper around `[Operation::Copy]`
    pub async fn operation_copy(
        paths: Vec<PathBuf>,
        to: PathBuf,
    ) -> Result<OperationSelection, OperationError> {
        let id = fastrand::u64(0..u64::MAX);
        let (tx, mut rx) = mpsc::channel(BUF_SIZE);
        let paths_clone = paths.clone();
        let to_clone = to.clone();
        let handle_copy = tokio::spawn(async move {
            Operation::Copy {
                paths: paths_clone,
                to: to_clone,
            }
            .perform(&sync::Mutex::new(tx).into(), Controller::default())
            .await
        });

        while let Some(msg) = rx.next().await {
            match msg {
                Message::DialogPush(DialogPage::Replace { tx, .. }) => {
                    debug!("[{id}] Replace request");
                    tx.send(ReplaceResult::Cancel).await.expect("Sending a response to a replace request should succeed")

                }
                _ => unreachable!("Only [ `Message::PendingProgress`, `Message::DialogPush(DialogPage::Replace)` ] are sent from operation"),
            }
        }

        handle_copy.await.unwrap()
    }

    #[test(tokio::test)]
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

    #[test(tokio::test)]
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

    #[test(tokio::test)]
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

    #[test(tokio::test)]
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

    #[test(tokio::test)]
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
            .expect(
                "Copy operation should have been cancelled because we're copying to different directories without replacement",
            );
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

    #[test(tokio::test)]
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
