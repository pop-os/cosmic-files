use cosmic::iced::futures::{channel::mpsc, executor, SinkExt};
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{self, AtomicU64},
        Arc,
    },
};

use crate::app::Message;

fn err_str<T: ToString>(err: T) -> String {
    err.to_string()
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
        msg_tx: &Arc<tokio::sync::Mutex<mpsc::Sender<Message>>>,
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
                            log::info!("{:?}", from.parent());
                            if matches!(from.parent(), Some(parent) if parent == to) {
                                // `from`'s parent is equal to `to` which means we're copying to the same
                                // directory (duplicating files)
                                let mut to = to.to_owned();
                                let to = if let Some(full_name) =
                                    from.file_name().and_then(|name| name.to_str())
                                {
                                    // Separate the full file name into its file name plus extension.
                                    let (base_name, ext, needs_dot) = if full_name.starts_with('.')
                                    {
                                        // `[Path::file_name]` returns the full name for dotfiles (e.g.
                                        // .someconf is the file_name)
                                        (full_name, "", false)
                                    } else {
                                        // Consider everything beyond the first '.' to be a file
                                        // extension.
                                        full_name
                                            .split_once('.')
                                            .map(|(full_name, extension)| {
                                                (full_name, extension, !extension.is_empty())
                                            })
                                            // File without an extension
                                            .unwrap_or((full_name, "", false))
                                    };
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
                                        let dot = if needs_dot { "." } else { "" };
                                        let new_name = format!("{base_name} (Copy {n}){dot}{ext}");
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
                    let dir_options = fs_extra::dir::CopyOptions::default().copy_inside(true);
                    let file_options = fs_extra::file::CopyOptions::default();
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
                    let file_handler = |progress: fs_extra::file::TransitProcess| {
                        copied_bytes.fetch_add(progress.copied_bytes, atomic::Ordering::Relaxed);
                        handler();
                    };
                    let dir_handler = |progress: fs_extra::TransitProcess| {
                        copied_bytes.fetch_add(progress.copied_bytes, atomic::Ordering::Relaxed);
                        handler();
                        //TODO: handle exceptions
                        fs_extra::dir::TransitProcessResult::ContinueOrAbort
                    };
                    //TODO: set options as desired
                    for (from, to) in paths.into_iter().zip(to.into_iter()) {
                        // This is essentially what `[fs_extra::copy_items_with_progress]` does
                        // except without handling options (e.g. overwrite). We're currently using
                        // the defaults anyway.
                        if from.is_dir() {
                            fs_extra::copy_items_with_progress(
                                &[from],
                                to,
                                &dir_options,
                                dir_handler,
                            )?;
                        } else {
                            fs_extra::file::copy_with_progress(
                                from,
                                to,
                                &file_options,
                                file_handler,
                            )?;
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
            Self::Move { paths, to } => {
                let msg_tx = msg_tx.clone();
                tokio::task::spawn_blocking(move || {
                    log::info!("Move {:?} to {:?}", paths, to);
                    let options = fs_extra::dir::CopyOptions::default();
                    //TODO: set options as desired
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
                        //TODO: handle exceptions
                        fs_extra::dir::TransitProcessResult::ContinueOrAbort
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
