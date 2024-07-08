use cosmic::iced::futures::{channel::mpsc::Sender, executor, SinkExt};
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{self, AtomicU64},
        Arc,
    },
};
use tokio::sync::{mpsc, Mutex};

use crate::{
    app::{DialogPage, Message},
    config::IconSizes,
    fl, tab,
};

fn err_str<T: ToString>(err: T) -> String {
    err.to_string()
}

fn handle_replace(
    msg_tx: &Arc<Mutex<Sender<Message>>>,
    file_from: PathBuf,
    file_to: PathBuf,
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

            handle_replace(msg_tx, file_from, file_to).into()
        }
        fs_extra::dir::TransitState::NoAccess => {
            //TODO: permission error dialog
            fs_extra::dir::TransitProcessResult::ContinueOrAbort
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReplaceResult {
    Replace,
    Skip,
    Cancel,
}

impl From<ReplaceResult> for fs_extra::dir::TransitProcessResult {
    fn from(f: ReplaceResult) -> fs_extra::dir::TransitProcessResult {
        match f {
            ReplaceResult::Replace => fs_extra::dir::TransitProcessResult::Overwrite,
            ReplaceResult::Skip => fs_extra::dir::TransitProcessResult::Skip,
            ReplaceResult::Cancel => fs_extra::dir::TransitProcessResult::Abort,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Operation {
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
}

impl Operation {
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
            Self::Copy { paths, to } => {
                // Handle duplicate file names by renaming paths
                let (paths, to): (Vec<_>, Vec<_>) = tokio::task::spawn_blocking(move || {
                    paths
                        .into_iter()
                        .zip(std::iter::repeat(to.as_path()))
                        .map(|(from, to)| {
                            if matches!(from.parent(), Some(parent) if parent == to) {
                                // `from`'s parent is equal to `to` which means we're copying to the same
                                // directory (duplicating files)
                                let mut to = to.to_owned();
                                // Separate the full file name into its file name plus extension.
                                // `[Path::file_stem]` returns the full name for dotfiles (e.g.
                                // .someconf is the file name)
                                let to = if let (Some(stem), ext) = (
                                    // FIXME: Replace `[Path::file_stem]` with `[Path::file_prefix]` when stablized to handle .tar.gz et al. better
                                    from.file_stem().and_then(|name| name.to_str()),
                                    from.extension()
                                        .and_then(|ext| ext.to_str())
                                        .unwrap_or_default(),
                                ) {
                                    // '.' needs to be re-added for paths with extensions.
                                    let dot = if ext.is_empty() { "" } else { "." };
                                    let mut n = 0u32;
                                    // Loop until a valid `copy n` variant is found
                                    loop {
                                        n = if let Some(n) = n.checked_add(1) {
                                            n
                                        } else {
                                            // TODO: Return error? fs_extra will handle it anyway
                                            break to;
                                        };

                                        // Rebuild file name
                                        let new_name =
                                            format!("{stem} ({} {n}){dot}{ext}", fl!("copy_noun"));
                                        to = to.join(new_name);

                                        if !matches!(to.try_exists(), Ok(true)) {
                                            break to;
                                        }
                                        // Continue if a copy with index exists
                                        to.pop();
                                    }
                                } else {
                                    to
                                };

                                (from, to)
                            } else if let Some(name) =
                                from.is_file().then(|| from.file_name()).flatten()
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
                    log::info!("Copy {:?} to {:?}", paths, to);
                    let copied_bytes = AtomicU64::default();
                    let total_bytes = paths
                        .iter()
                        .map(fs_extra::dir::get_size)
                        .sum::<Result<u64, _>>()?;
                    let handler = || {
                        executor::block_on(async {
                            let _ = msg_tx
                                .lock()
                                .await
                                .send(Message::PendingProgress(
                                    id,
                                    100.0 * copied_bytes.load(atomic::Ordering::Relaxed) as f32
                                        / total_bytes as f32,
                                ))
                                .await;
                        })
                    };
                    // Files and directory progress are handled separately
                    let file_handler = |progress: fs_extra::file::TransitProcess| {
                        copied_bytes.fetch_add(progress.copied_bytes, atomic::Ordering::Relaxed);
                        handler();
                    };
                    let dir_handler = |progress: fs_extra::TransitProcess| {
                        copied_bytes.fetch_add(progress.copied_bytes, atomic::Ordering::Relaxed);
                        handler();
                        handle_progress_state(&msg_tx, &progress)
                    };
                    for (from, to) in paths.into_iter().zip(to.into_iter()) {
                        if from.is_dir() {
                            let options = fs_extra::dir::CopyOptions::default().copy_inside(true);
                            fs_extra::copy_items_with_progress(&[from], to, &options, dir_handler)?;
                        } else {
                            let mut options = fs_extra::file::CopyOptions::default();
                            if to.exists() {
                                match handle_replace(&msg_tx, from.clone(), to.clone()) {
                                    ReplaceResult::Replace => {
                                        options.overwrite = true;
                                    }
                                    ReplaceResult::Skip => {
                                        options.skip_exist = true;
                                    }
                                    ReplaceResult::Cancel => {
                                        //TODO: be silent, but collect actual changes made for undo
                                        return Err(fs_extra::error::Error::new(
                                            fs_extra::error::ErrorKind::Interrupted,
                                            "operation cancelled",
                                        ));
                                    }
                                }
                            }
                            fs_extra::file::copy_with_progress(from, to, &options, file_handler)?;
                        }
                    }
                    Ok(())
                })
                .await
                .map_err(err_str)?
                .map_err(err_str)?;
            }
            Self::Delete { paths } => {
                let total = paths.len();
                let mut count = 0;
                for path in paths {
                    tokio::task::spawn_blocking(|| trash::delete(path))
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
            Self::Move { paths, to } => {
                let msg_tx = msg_tx.clone();
                tokio::task::spawn_blocking(move || {
                    log::info!("Move {:?} to {:?}", paths, to);
                    let options = fs_extra::dir::CopyOptions::default();
                    fs_extra::move_items_with_progress(&paths, &to, &options, |progress| {
                        executor::block_on(async {
                            let _ = msg_tx
                                .lock()
                                .await
                                .send(Message::PendingProgress(
                                    id,
                                    100.0 * (progress.copied_bytes as f32)
                                        / (progress.total_bytes as f32),
                                ))
                                .await;
                        });
                        handle_progress_state(&msg_tx, &progress)
                    })
                })
                .await
                .map_err(err_str)?
                .map_err(err_str)?;
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

    use cosmic::iced::futures::channel::mpsc;
    use log::{debug, trace};
    use test_log::test;
    use tokio::sync;

    use super::Operation;
    use crate::{
        app::{
            test_utils::{
                empty_fs, filter_dirs, filter_files, read_dir_sorted, simple_fs, NAME_LEN,
                NUM_DIRS, NUM_FILES, NUM_HIDDEN, NUM_NESTED,
            },
            Message,
        },
        fl,
    };

    // Tests hang with lower values
    const BUF_SIZE: usize = 8;

    /// Simple wrapper around `[Operation::Copy]`
    pub async fn operation_copy(paths: Vec<PathBuf>, to: PathBuf) -> Result<(), String> {
        let id = fastrand::u64(0..u64::MAX);
        let (tx, mut rx) = mpsc::channel(BUF_SIZE);
        Operation::Copy {
            paths: paths.clone(),
            to: to.clone(),
        }
        .perform(id, &sync::Mutex::new(tx).into())
        .await?;

        loop {
            match rx.try_next() {
                Ok(Some(Message::PendingProgress(id, progress))) => {
                    trace!("({id}) [ {paths:?} => {to:?} ] {progress}% complete)")
                }
                Ok(None) => break,
                Err(e) => panic!("Receiving message from operation should succeed: {e:?}"),
                _ => unreachable!("Only `Message::PendingProgress` is sent from operation"),
            }
        }

        Ok(())
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
            .expect_err(
                "Copy operation should have failed because we're copying to different directories",
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
