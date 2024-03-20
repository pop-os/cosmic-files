use cosmic::iced::futures::{channel::mpsc, executor, SinkExt};
use std::{fs, path::PathBuf, sync::Arc};

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
                let msg_tx = msg_tx.clone();
                tokio::task::spawn_blocking(move || {
                    log::info!("Copy {:?} to {:?}", paths, to);
                    let options = fs_extra::dir::CopyOptions::default();
                    //TODO: set options as desired
                    fs_extra::copy_items_with_progress(&paths, &to, &options, |progress| {
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
