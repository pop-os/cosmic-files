use cosmic::iced::futures::{channel::mpsc::Sender, executor, SinkExt};
use std::{
    borrow::Cow,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex};
use walkdir::WalkDir;

use crate::{
    app::{ArchiveType, DialogPage, Message},
    config::IconSizes,
    err_str, fl,
    mime_icon::mime_for_path,
    spawn_detached::spawn_detached,
    tab,
};

fn handle_replace(
    msg_tx: &Arc<Mutex<Sender<Message>>>,
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

fn handle_progress_state(
    msg_tx: &Arc<Mutex<Sender<Message>>>,
    progress: &fs_extra::TransitProcess,
) -> fs_extra::dir::TransitProcessResult {
    log::warn!("{:?}", progress);
    match progress.state {
        fs_extra::dir::TransitState::Normal => fs_extra::dir::TransitProcessResult::ContinueOrAbort,
        fs_extra::dir::TransitState::Exists => {
            let Some(file_from) = progress.file_from.clone() else {
                log::warn!("missing file_from in progress");
                return fs_extra::dir::TransitProcessResult::Abort;
            };

            let Some(file_to) = progress.file_to.clone() else {
                log::warn!("missing file_to in progress");
                return fs_extra::dir::TransitProcessResult::Abort;
            };

            if file_from == file_to {
                log::warn!("trying to copy {:?} to itself", file_from);
                return fs_extra::dir::TransitProcessResult::Abort;
            }

            handle_replace(msg_tx, file_from, file_to, true).into()
        }
        fs_extra::dir::TransitState::NoAccess => {
            //TODO: permission error dialog
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        }
    }
}

fn get_directory_name(file_name: &str) -> &str {
    const SUPPORTED_EXTENSIONS: [&str; 4] = [".tar.gz", ".tgz", ".tar", ".zip"];

    for ext in &SUPPORTED_EXTENSIONS {
        if file_name.ends_with(ext) {
            return &file_name[..file_name.len() - ext.len()];
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

impl From<ReplaceResult> for fs_extra::dir::TransitProcessResult {
    fn from(f: ReplaceResult) -> fs_extra::dir::TransitProcessResult {
        match f {
            ReplaceResult::Replace(apply_to_all) => {
                if apply_to_all {
                    fs_extra::dir::TransitProcessResult::OverwriteAll
                } else {
                    fs_extra::dir::TransitProcessResult::Overwrite
                }
            }
            ReplaceResult::KeepBoth => {
                log::warn!("tried to keep both when replacing multiple files");
                fs_extra::dir::TransitProcessResult::Abort
            }
            ReplaceResult::Skip(apply_to_all) => {
                if apply_to_all {
                    fs_extra::dir::TransitProcessResult::SkipAll
                } else {
                    fs_extra::dir::TransitProcessResult::Skip
                }
            }
            ReplaceResult::Cancel => fs_extra::dir::TransitProcessResult::Abort,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Operation {
    /// Compress files
    Compress {
        paths: Vec<PathBuf>,
        to: PathBuf,
        archive_type: ArchiveType,
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
    /// Empty the trash
    EmptyTrash,
    /// Uncompress files
    Extract {
        paths: Vec<PathBuf>,
        to: PathBuf,
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
    Rename {
        from: PathBuf,
        to: PathBuf,
    },
    /// Restore a path from the trash
    Restore {
        paths: Vec<trash::TrashItem>,
    },
    /// Set executable and launch
    SetExecutableAndLaunch {
        path: PathBuf,
    },
}

async fn copy_or_move(
    paths: Vec<PathBuf>,
    to: PathBuf,
    moving: bool,
    id: u64,
    msg_tx: &Arc<Mutex<Sender<Message>>>,
) -> Result<(), String> {
    // Handle duplicate file names by renaming paths
    let (paths, to): (Vec<_>, Vec<_>) = tokio::task::spawn_blocking(move || {
        paths
            .into_iter()
            .zip(std::iter::repeat(to.as_path()))
            .map(|(from, to)| {
                if matches!(from.parent(), Some(parent) if parent == to) && !moving {
                    // `from`'s parent is equal to `to` which means we're copying to the same
                    // directory (duplicating files)
                    let to = copy_unique_path(&from, &to);
                    (from, to)
                } else if let Some(name) = (from.is_file() || moving)
                    .then(|| from.file_name())
                    .flatten()
                {
                    let to = to.join(name);
                    (from, to)
                } else {
                    (from, to.to_owned())
                }
            })
            .unzip()
    })
    .await
    .unwrap();

    let msg_tx = msg_tx.clone();
    tokio::task::spawn_blocking(move || -> fs_extra::error::Result<()> {
        log::info!(
            "{} {:?} to {:?}",
            if moving { "Move" } else { "Copy" },
            paths,
            to
        );
        let total_paths = paths.len();
        for (path_i, (from, mut to)) in paths.into_iter().zip(to.into_iter()).enumerate() {
            let handler = |copied_bytes, total_bytes| {
                let item_progress = if total_bytes == 0 {
                    1.0
                } else {
                    copied_bytes as f32 / total_bytes as f32
                };
                let total_progress = (item_progress + path_i as f32) / total_paths as f32;
                executor::block_on(async {
                    let _ = msg_tx
                        .lock()
                        .await
                        .send(Message::PendingProgress(id, 100.0 * total_progress))
                        .await;
                })
            };

            if from == to {
                log::info!(
                    "Skipping {} of {:?} to itself",
                    if moving { "move" } else { "copy" },
                    from
                );
                handler(0, 0);
                continue;
            }

            if from.is_dir() {
                let options = fs_extra::dir::CopyOptions::default().copy_inside(true);
                if moving {
                    fs_extra::move_items_with_progress(
                        &[from],
                        to,
                        &options,
                        |progress: fs_extra::TransitProcess| {
                            handler(progress.copied_bytes, progress.total_bytes);
                            handle_progress_state(&msg_tx, &progress)
                        },
                    )?;
                } else {
                    fs_extra::copy_items_with_progress(
                        &[from],
                        to,
                        &options,
                        |progress: fs_extra::TransitProcess| {
                            handler(progress.copied_bytes, progress.total_bytes);
                            handle_progress_state(&msg_tx, &progress)
                        },
                    )?;
                }
            } else {
                let mut options = fs_extra::file::CopyOptions::default();
                if to.exists() {
                    match handle_replace(&msg_tx, from.clone(), to.clone(), false) {
                        ReplaceResult::Replace(_) => {
                            options.overwrite = true;
                        }
                        ReplaceResult::KeepBoth => {
                            match to.parent() {
                                Some(to_parent) => {
                                    to = copy_unique_path(&from, &to_parent);
                                }
                                None => {
                                    log::warn!("failed to get parent of {:?}", to);
                                    //TODO: error?
                                }
                            }
                        }
                        ReplaceResult::Skip(_) => {
                            options.skip_exist = true;
                        }
                        ReplaceResult::Cancel => {
                            //TODO: be silent, but collect actual changes made for undo
                            continue;
                        }
                    }
                }
                if moving {
                    //TODO: optimize to fs::rename when possible
                    fs_extra::file::move_file_with_progress(
                        from,
                        to,
                        &options,
                        |progress: fs_extra::file::TransitProcess| {
                            handler(progress.copied_bytes, progress.total_bytes);
                        },
                    )?;
                } else {
                    fs_extra::file::copy_with_progress(
                        from,
                        to,
                        &options,
                        |progress: fs_extra::file::TransitProcess| {
                            handler(progress.copied_bytes, progress.total_bytes);
                        },
                    )?;
                }
            }
        }
        Ok(())
    })
    .await
    .map_err(err_str)?
    .map_err(err_str)
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

fn file_name<'a>(path: &'a Path) -> Cow<'a, str> {
    path.file_name()
        .map_or_else(|| fl!("unknown-folder").into(), |x| x.to_string_lossy())
}

fn parent_name<'a>(path: &'a Path) -> Cow<'a, str> {
    let Some(parent) = path.parent() else {
        return fl!("unknown-folder").into();
    };

    file_name(parent)
}

fn paths_parent_name<'a>(paths: &'a Vec<PathBuf>) -> Cow<'a, str> {
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

impl Operation {
    pub fn pending_text(&self) -> String {
        match self {
            Self::Compress { paths, to, .. } => fl!(
                "compressing",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
            ),
            Self::Copy { paths, to } => fl!(
                "copying",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
            ),
            Self::Delete { paths } => fl!(
                "moving",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = fl!("trash")
            ),
            Self::EmptyTrash => fl!("emptying-trash"),
            Self::Extract { paths, to } => fl!(
                "extracting",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
            ),
            Self::Move { paths, to } => fl!(
                "moving",
                items = paths.len(),
                from = paths_parent_name(paths),
                to = file_name(to)
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
            Self::Rename { from, to } => {
                fl!("renaming", from = file_name(from), to = file_name(to))
            }
            Self::Restore { paths } => fl!("restoring", items = paths.len()),
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
            Self::EmptyTrash => fl!("emptied-trash"),
            Self::Extract { paths, to } => fl!(
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
            Self::Rename { from, to } => fl!("renamed", from = file_name(from), to = file_name(to)),
            Self::Restore { paths } => fl!("restored", items = paths.len()),
            Self::SetExecutableAndLaunch { path } => {
                fl!("set-executable-and-launched", name = file_name(path))
            }
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
        id: u64,
        msg_tx: &Arc<Mutex<Sender<Message>>>,
    ) -> Result<(), String> {
        let _ = msg_tx
            .lock()
            .await
            .send(Message::PendingProgress(id, 0.0))
            .await;

        //TODO: IF ERROR, RETURN AN Operation THAT CAN UNDO THE CURRENT STATE
        //TODO: SAFELY HANDLE CANCEL
        match self {
            Self::Compress {
                paths,
                to,
                archive_type,
            } => {
                let msg_tx = msg_tx.clone();
                tokio::task::spawn_blocking(move || -> Result<(), String> {
                    let Some(relative_root) = to.parent() else {
                        return Err(format!("path {:?} has no parent directory", to));
                    };

                    let mut paths = paths;
                    for path in paths.clone().iter() {
                        if path.is_dir() {
                            let new_paths_it = WalkDir::new(path).into_iter();
                            for entry in new_paths_it.skip(1) {
                                let entry = entry.map_err(err_str)?;
                                paths.push(entry.path().to_path_buf());
                            }
                        }
                    }

                    match archive_type {
                        ArchiveType::Tgz => {
                            let mut archive = fs::File::create(&to)
                                .map(io::BufWriter::new)
                                .map(|w| {
                                    flate2::write::GzEncoder::new(w, flate2::Compression::default())
                                })
                                .map(tar::Builder::new)
                                .map_err(err_str)?;

                            let total_paths = paths.len();
                            for (i, path) in paths.iter().enumerate() {
                                executor::block_on(async {
                                    let total_progress = (i as f32) / total_paths as f32;
                                    let _ = msg_tx
                                        .lock()
                                        .await
                                        .send(Message::PendingProgress(id, 100.0 * total_progress))
                                        .await;
                                });

                                if let Some(relative_path) =
                                    path.strip_prefix(relative_root).map_err(err_str)?.to_str()
                                {
                                    archive
                                        .append_path_with_name(path, relative_path)
                                        .map_err(err_str)?;
                                }
                            }

                            archive.finish().map_err(err_str)?;
                        }
                        ArchiveType::Zip => {
                            let mut archive = fs::File::create(&to)
                                .map(io::BufWriter::new)
                                .map(zip::ZipWriter::new)
                                .map_err(err_str)?;

                            //TODO: set unix_permissions per file?
                            let zip_options = zip::write::SimpleFileOptions::default();

                            let total_paths = paths.len();
                            for (i, path) in paths.iter().enumerate() {
                                executor::block_on(async {
                                    let total_progress = (i as f32) / total_paths as f32;
                                    let _ = msg_tx
                                        .lock()
                                        .await
                                        .send(Message::PendingProgress(id, 100.0 * total_progress))
                                        .await;
                                });

                                if let Some(relative_path) =
                                    path.strip_prefix(relative_root).map_err(err_str)?.to_str()
                                {
                                    if path.is_file() {
                                        archive
                                            .start_file(relative_path, zip_options)
                                            .map_err(err_str)?;

                                        let mut buffer = Vec::new();
                                        let mut file = fs::File::open(&path)
                                            .map(io::BufReader::new)
                                            .map_err(err_str)?;

                                        file.read_to_end(&mut buffer).map_err(err_str)?;
                                        archive.write_all(&buffer).map_err(err_str)?;
                                    } else {
                                        archive
                                            .add_directory(relative_path, zip_options)
                                            .map_err(err_str)?;
                                    }
                                }
                            }

                            archive.finish().map_err(err_str)?;
                        }
                    }

                    Ok(())
                })
                .await
                .map_err(err_str)?
                .map_err(err_str)?;
            }
            Self::Copy { paths, to } => {
                copy_or_move(paths, to, false, id, msg_tx).await?;
            }
            Self::Delete { paths } => {
                let total = paths.len();
                let mut count = 0;
                for path in paths {
                    let items_opt = tokio::task::spawn_blocking(|| trash::delete(path))
                        .await
                        .map_err(err_str)?
                        .map_err(err_str)?;
                    //TODO: items_opt allows for easy restore
                    count += 1;
                    let _ = msg_tx
                        .lock()
                        .await
                        .send(Message::PendingProgress(
                            id,
                            100.0 * (count as f32) / (total as f32),
                        ))
                        .await;
                }
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
                    tokio::task::spawn_blocking(|| {
                        let items = trash::os_limited::list()?;
                        trash::os_limited::purge_all(items)
                    })
                    .await
                    .map_err(err_str)?
                    .map_err(err_str)?;
                }
                let _ = msg_tx
                    .lock()
                    .await
                    .send(Message::PendingProgress(id, 100.0))
                    .await;
            }
            Self::Extract { paths, to } => {
                let msg_tx = msg_tx.clone();
                tokio::task::spawn_blocking(move || -> Result<(), String> {
                    let total_paths = paths.len();
                    for (i, path) in paths.iter().enumerate() {
                        executor::block_on(async {
                            let total_progress = (i as f32) / total_paths as f32;
                            let _ = msg_tx
                                .lock()
                                .await
                                .send(Message::PendingProgress(id, 100.0 * total_progress))
                                .await;
                        });

                        let to = to.to_owned();

                        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                            let dir_name = get_directory_name(file_name);
                            let mut new_dir = to.join(dir_name);

                            if new_dir.exists() {
                                if let Some(new_dir_parent) = new_dir.parent() {
                                    new_dir = copy_unique_path(&new_dir, new_dir_parent);
                                }
                            }

                            let mime = mime_for_path(&path);
                            match mime.essence_str() {
                                "application/gzip" | "application/x-compressed-tar" => {
                                    fs::File::open(path)
                                        .map(io::BufReader::new)
                                        .map(flate2::read::GzDecoder::new)
                                        .map(tar::Archive::new)
                                        .and_then(|mut archive| archive.unpack(&new_dir))
                                        .map_err(err_str)?
                                }
                                "application/x-tar" => fs::File::open(path)
                                    .map(io::BufReader::new)
                                    .map(tar::Archive::new)
                                    .and_then(|mut archive| archive.unpack(&new_dir))
                                    .map_err(err_str)?,
                                "application/zip" => fs::File::open(path)
                                    .map(io::BufReader::new)
                                    .map(zip::ZipArchive::new)
                                    .map_err(err_str)?
                                    .and_then(|mut archive| archive.extract(&new_dir))
                                    .map_err(err_str)?,
                                #[cfg(feature = "bzip2")]
                                "application/x-bzip" | "application/x-bzip-compressed-tar" => {
                                    fs::File::open(path)
                                        .map(io::BufReader::new)
                                        .map(bzip2::read::BzDecoder::new)
                                        .map(tar::Archive::new)
                                        .and_then(|mut archive| archive.unpack(new_dir))
                                        .map_err(err_str)?
                                }
                                #[cfg(feature = "liblzma")]
                                "application/x-xz" | "application/x-xz-compressed-tar" => {
                                    fs::File::open(path)
                                        .map(io::BufReader::new)
                                        .map(liblzma::read::XzDecoder::new)
                                        .map(tar::Archive::new)
                                        .and_then(|mut archive| archive.unpack(new_dir))
                                        .map_err(err_str)?
                                }
                                _ => Err(format!("unsupported mime type {:?}", mime))?,
                            }
                        }
                    }

                    Ok(())
                })
                .await
                .map_err(err_str)?
                .map_err(err_str)?;
            }
            Self::Move { paths, to } => {
                copy_or_move(paths, to, true, id, msg_tx).await?;
            }
            Self::NewFolder { path } => {
                tokio::task::spawn_blocking(|| fs::create_dir(path))
                    .await
                    .map_err(err_str)?
                    .map_err(err_str)?;
                let _ = msg_tx
                    .lock()
                    .await
                    .send(Message::PendingProgress(id, 100.0))
                    .await;
            }
            Self::NewFile { path } => {
                tokio::task::spawn_blocking(|| fs::File::create(path))
                    .await
                    .map_err(err_str)?
                    .map_err(err_str)?;
                let _ = msg_tx
                    .lock()
                    .await
                    .send(Message::PendingProgress(id, 100.0))
                    .await;
            }
            Self::Rename { from, to } => {
                tokio::task::spawn_blocking(|| fs::rename(from, to))
                    .await
                    .map_err(err_str)?
                    .map_err(err_str)?;
                let _ = msg_tx
                    .lock()
                    .await
                    .send(Message::PendingProgress(id, 100.0))
                    .await;
            }
            #[cfg(target_os = "macos")]
            Self::Restore { .. } => {
                // TODO: add support for macos
                return Err("Restoring from trash is not supported on macos".to_string());
            }
            #[cfg(not(target_os = "macos"))]
            Self::Restore { paths } => {
                let total = paths.len();
                let mut count = 0;
                for path in paths {
                    tokio::task::spawn_blocking(|| trash::os_limited::restore_all([path]))
                        .await
                        .map_err(err_str)?
                        .map_err(err_str)?;
                    count += 1;
                    let _ = msg_tx
                        .lock()
                        .await
                        .send(Message::PendingProgress(
                            id,
                            100.0 * (count as f32) / (total as f32),
                        ))
                        .await;
                }
            }
            Self::SetExecutableAndLaunch { path } => {
                tokio::task::spawn_blocking(move || -> io::Result<()> {
                    //TODO: what to do on non-Unix systems?
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let mut perms = fs::metadata(&path)?.permissions();
                        let current_mode = perms.mode();
                        let new_mode = current_mode | 0o111;
                        perms.set_mode(new_mode);
                        fs::set_permissions(&path, perms)?;
                    }

                    let mut command = std::process::Command::new(path);
                    spawn_detached(&mut command)?;

                    Ok(())
                })
                .await
                .map_err(err_str)?
                .map_err(err_str)?;
                let _ = msg_tx
                    .lock()
                    .await
                    .send(Message::PendingProgress(id, 100.0))
                    .await;
            }
        }

        let _ = msg_tx
            .lock()
            .await
            .send(Message::PendingProgress(id, 100.0))
            .await;

        Ok(())
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
    use log::{debug, trace};
    use test_log::test;
    use tokio::sync;

    use super::{Operation, ReplaceResult};
    use crate::{
        app::{
            test_utils::{
                empty_fs, filter_dirs, filter_files, read_dir_sorted, simple_fs, NAME_LEN,
                NUM_DIRS, NUM_FILES, NUM_HIDDEN, NUM_NESTED,
            },
            DialogPage, Message,
        },
        fl,
    };

    // Tests hang with lower values
    const BUF_SIZE: usize = 8;

    /// Simple wrapper around `[Operation::Copy]`
    pub async fn operation_copy(paths: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
        let id = fastrand::u64(0..u64::MAX);
        let (tx, mut rx) = mpsc::channel(BUF_SIZE);
        let paths_clone = paths.clone();
        let to_clone = to.clone();
        let handle_copy = tokio::spawn(async move {
            Operation::Copy {
                paths: paths_clone,
                to: to_clone,
            }
            .perform(id, &sync::Mutex::new(tx).into())
            .await
        });

        while let Some(msg) = rx.next().await {
            match msg {
                Message::PendingProgress(id, progress) => {
                    trace!("({id}) [ {paths:?} => {to:?} ] {progress}% complete)")
                }
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
