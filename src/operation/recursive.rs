use compio::buf::{IntoInner, IoBuf};
use compio::io::{AsyncReadAt, AsyncWriteAt};
use compio::BufResult;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;
use std::{cell::Cell, error::Error, fs, ops::ControlFlow, path::PathBuf, rc::Rc};
use walkdir::WalkDir;

use super::{copy_unique_path, Controller, OperationSelection, ReplaceResult};

pub struct Context {
    buf: Vec<u8>,
    controller: Controller,
    on_progress: Box<dyn OnProgress>,
    on_replace: Pin<Box<dyn OnReplace>>,
    pub(crate) op_sel: OperationSelection,
    replace_result_opt: Option<ReplaceResult>,
}

pub trait OnProgress: Fn(&Op, &Progress) + 'static {}
impl<F> OnProgress for F where F: Fn(&Op, &Progress) + 'static {}

pub trait OnReplace:
    for<'a> Fn(&'a Op) -> Pin<Box<dyn Future<Output = ReplaceResult> + 'a>> + 'static
{
}
impl<F> OnReplace for F where
    F: for<'a> Fn(&'a Op) -> Pin<Box<dyn Future<Output = ReplaceResult> + 'a>> + 'static
{
}

impl Context {
    pub fn new(controller: Controller) -> Self {
        Self {
            // 128K is the optimal upper size of a buffer.
            buf: vec![0u8; 128 * 1024],
            controller,
            on_progress: Box::new(|_op, _progress| {}),
            on_replace: Box::pin(|_op| Box::pin(async { ReplaceResult::Cancel })),
            op_sel: OperationSelection::default(),
            replace_result_opt: None,
        }
    }

    pub async fn recursive_copy_or_move(
        &mut self,
        from_to_pairs: Vec<(PathBuf, PathBuf)>,
        moving: bool,
    ) -> Result<bool, String> {
        let mut ops = Vec::new();
        let mut cleanup_ops = Vec::new();
        for (from_parent, to_parent) in from_to_pairs {
            self.controller.check().await?;

            if from_parent == to_parent {
                // Skip matching source and destination
                continue;
            }

            for entry in WalkDir::new(&from_parent).into_iter() {
                self.controller.check().await?;

                let entry = entry.map_err(|err| {
                    format!("failed to walk directory {:?}: {}", from_parent, err)
                })?;
                let file_type = entry.file_type();
                let from = entry.into_path();
                let kind = if file_type.is_dir() {
                    OpKind::Mkdir
                } else if file_type.is_file() {
                    if moving {
                        OpKind::Move
                    } else {
                        OpKind::Copy
                    }
                } else if file_type.is_symlink() {
                    let target = fs::read_link(&from)
                        .map_err(|err| format!("failed to read link {:?}: {}", from, err))?;
                    OpKind::Symlink { target }
                } else {
                    //TODO: present dialog and allow continue
                    return Err(format!("{} is not a known file type", from.display()));
                };
                let to = if from == from_parent {
                    // When copying a file, from matches from_parent, and to_parent must be used
                    to_parent.clone()
                } else {
                    let relative = from.strip_prefix(&from_parent).map_err(|err| {
                        format!(
                            "failed to remove prefix {:?} from {:?}: {}",
                            from_parent, from, err
                        )
                    })?;
                    //TODO: ensure to is inside of to_parent?
                    to_parent.join(relative)
                };
                let op = Op {
                    kind,
                    from,
                    to,
                    skipped: Rc::new(Cell::new(false)),
                };
                if moving {
                    if let Some(cleanup_op) = op.move_cleanup_op() {
                        cleanup_ops.push(cleanup_op);
                    }
                }
                ops.push(op);
            }

            self.op_sel.ignored.push(from_parent);
        }

        // Add cleanup ops after standard ops, in reverse
        for cleanup_op in cleanup_ops.into_iter().rev() {
            ops.push(cleanup_op);
        }

        let total_ops = ops.len();
        for (current_ops, mut op) in ops.into_iter().enumerate() {
            self.controller.check().await?;

            let progress = Progress {
                current_ops,
                total_ops,
                current_bytes: 0,
                total_bytes: None,
            };
            (self.on_progress)(&op, &progress);
            if op.run(self, progress).await.map_err(|err| {
                format!(
                    "failed to {:?} {:?} to {:?}: {}",
                    op.kind, op.from, op.to, err
                )
            })? {
                // The from path is ignored in the operation selection if it is a top level item
                if self.op_sel.ignored.contains(&op.from) {
                    // So add the to path to the selection
                    self.op_sel.selected.push(op.to.clone());
                }
            } else {
                // Cancelled
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn on_progress<F: OnProgress>(mut self, f: F) -> Self {
        self.on_progress = Box::new(f);
        self
    }

    pub fn on_replace(mut self, f: impl OnReplace + 'static) -> Self {
        self.on_replace = Box::pin(f);
        self
    }

    async fn replace(&mut self, op: &Op) -> Result<ControlFlow<bool, PathBuf>, Box<dyn Error>> {
        let replace_result = match self.replace_result_opt {
            Some(result) => result,
            None => (self.on_replace)(op).await,
        };

        match replace_result {
            ReplaceResult::Replace(apply_to_all) => {
                if apply_to_all {
                    self.replace_result_opt = Some(replace_result);
                }
                compio::fs::remove_file(&op.to).await?;
                Ok(ControlFlow::Continue(op.to.clone()))
            }
            ReplaceResult::KeepBoth => match op.to.parent() {
                Some(to_parent) => Ok(ControlFlow::Continue(copy_unique_path(&op.from, to_parent))),
                None => Err(format!("failed to get parent of {:?}", op.to).into()),
            },
            ReplaceResult::Skip(apply_to_all) => {
                if apply_to_all {
                    self.replace_result_opt = Some(replace_result);
                }
                op.skipped.set(true);
                Ok(ControlFlow::Break(true))
            }
            ReplaceResult::Cancel => Ok(ControlFlow::Break(false)),
        }
    }
}

#[derive(Debug)]
pub struct Progress {
    pub current_ops: usize,
    pub total_ops: usize,
    pub current_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[derive(Debug)]
pub enum OpKind {
    Copy,
    Move,
    Mkdir,
    Remove,
    Rmdir,
    Symlink { target: PathBuf },
}

#[derive(Debug)]
pub struct Op {
    pub kind: OpKind,
    pub from: PathBuf,
    pub to: PathBuf,
    pub skipped: Rc<Cell<bool>>,
}

impl Op {
    fn move_cleanup_op(&self) -> Option<Self> {
        let kind = match self.kind {
            OpKind::Copy | OpKind::Move | OpKind::Symlink { .. } => OpKind::Remove,
            OpKind::Mkdir => OpKind::Rmdir,
            OpKind::Remove | OpKind::Rmdir => return None,
        };
        Some(Self {
            kind,
            from: self.from.clone(),
            //TODO: it is strange to have `to` here
            to: self.to.clone(),
            skipped: self.skipped.clone(),
        })
    }

    async fn run(
        &mut self,
        ctx: &mut Context,
        mut progress: Progress,
    ) -> Result<bool, Box<dyn Error>> {
        if self.skipped.get() {
            return Ok(true);
        }
        match self.kind {
            OpKind::Copy => {
                // Remove `to` if overwriting and it is an existing file
                if self.to.is_file() {
                    match ctx.replace(self).await? {
                        ControlFlow::Continue(to) => {
                            self.to = to;
                        }
                        ControlFlow::Break(ret) => {
                            return Ok(ret);
                        }
                    }
                }

                let (from_file, metadata, mut to_file) = futures::try_join!(
                    async {
                        compio::fs::OpenOptions::new()
                            .read(true)
                            .open(&self.from)
                            .await
                    },
                    compio::fs::metadata(&self.from),
                    // This is atomic and ensures `to` is not created by any other process
                    async {
                        compio::fs::OpenOptions::new()
                            .create_new(true)
                            .write(true)
                            .open(&self.to)
                            .await
                    }
                )?;

                progress.total_bytes = Some(metadata.len());
                (ctx.on_progress)(self, &progress);
                to_file.set_permissions(metadata.permissions()).await?;

                // Prevent spamming the progress callbacks.
                let mut last_progress_update = Instant::now();
                // io_uring/IOCP requires transferring ownership of the buffer to the kernel.
                let mut buf_in = std::mem::take(&mut ctx.buf);
                // Track where the current read/write position is at.
                let mut pos = 0;

                loop {
                    let BufResult(result, buf_out) = from_file.read_at(buf_in, pos).await;

                    let count = match result {
                        Ok(0) => {
                            ctx.buf = buf_out;
                            break;
                        }
                        Ok(count) => count,
                        Err(why) => {
                            ctx.buf = buf_out;
                            return Err(why.into());
                        }
                    };

                    let BufResult(result, buf_out_slice) =
                        to_file.write_at(buf_out.slice(..count), pos).await;
                    let buf_out = buf_out_slice.into_inner();

                    if let Err(why) = result {
                        ctx.buf = buf_out;
                        return Err(why.into());
                    }

                    progress.current_bytes += count as u64;
                    pos += count as u64;

                    // Avoid spamming progress messages too early.
                    let current = Instant::now();
                    if current.duration_since(last_progress_update).as_millis() > 49 {
                        last_progress_update = current;
                        (ctx.on_progress)(self, &progress);

                        // Also check if the progress was cancelled.
                        if let Err(why) = ctx.controller.check().await {
                            ctx.buf = buf_out;
                            return Err(why.into());
                        }
                    }

                    buf_in = buf_out;
                }

                to_file.sync_all().await?;
            }
            OpKind::Move => {
                // Remove `to` if overwriting and it is an existing file
                if self.to.is_file() {
                    match ctx.replace(self).await? {
                        ControlFlow::Continue(to) => {
                            self.to = to;
                        }
                        ControlFlow::Break(ret) => {
                            return Ok(ret);
                        }
                    }
                }
                // This is atomic and ensures `to` is not created by any other process
                match compio::fs::hard_link(&self.from, &self.to).await {
                    Ok(()) => {}
                    Err(err) => {
                        // https://docs.rs/windows-sys/latest/windows_sys/Win32/Foundation/constant.ERROR_NOT_SAME_DEVICE.html
                        #[cfg(windows)]
                        const EXDEV: i32 = 17;
                        #[cfg(unix)]
                        const EXDEV: i32 = libc::EXDEV as _;

                        if err.raw_os_error() == Some(EXDEV) {
                            // Try standard copy if hard link fails with cross device error
                            let mut copy_op = Op {
                                kind: OpKind::Copy,
                                from: self.from.clone(),
                                to: self.to.clone(),
                                skipped: self.skipped.clone(),
                            };
                            return Box::pin(copy_op.run(ctx, progress)).await;
                        } else {
                            return Err(err.into());
                        }
                    }
                }
            }
            OpKind::Mkdir => {
                compio::fs::create_dir_all(&self.to).await?;
            }
            OpKind::Remove => {
                compio::fs::remove_file(&self.from).await?;
            }
            OpKind::Rmdir => {
                compio::fs::remove_dir(&self.from).await?;
            }
            OpKind::Symlink { ref target } => {
                // Remove `to` if overwriting and it is an existing file
                if self.to.is_file() {
                    match ctx.replace(self).await? {
                        ControlFlow::Continue(to) => {
                            self.to = to;
                        }
                        ControlFlow::Break(ret) => {
                            return Ok(ret);
                        }
                    }
                }
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(target, &self.to)?;
                }
                #[cfg(windows)]
                {
                    if target.is_dir() {
                        std::os::windows::fs::symlink_dir(target, &self.to)?;
                    } else {
                        std::os::windows::fs::symlink_file(target, &self.to)?;
                    }
                }
            }
        }
        Ok(true)
    }
}
