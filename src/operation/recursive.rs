use std::{
    error::Error,
    fs,
    io::{Read, Write},
    ops::ControlFlow,
    path::PathBuf,
};
use walkdir::WalkDir;

use super::{copy_unique_path, Controller, OperationSelection, ReplaceResult};

pub struct Context {
    buf: Vec<u8>,
    controller: Controller,
    on_progress: Box<dyn Fn(&Op, &Progress) + 'static>,
    on_replace: Box<dyn Fn(&Op) -> ReplaceResult + 'static>,
    pub(crate) op_sel: OperationSelection,
    replace_result_opt: Option<ReplaceResult>,
}

impl Context {
    pub fn new(controller: Controller) -> Self {
        Self {
            buf: vec![0; 4 * 1024 * 1024],
            controller,
            on_progress: Box::new(|_op, _progress| {}),
            on_replace: Box::new(|_op| ReplaceResult::Cancel),
            op_sel: OperationSelection::default(),
            replace_result_opt: None,
        }
    }

    pub fn recursive_copy_or_move(
        &mut self,
        from_to_pairs: Vec<(PathBuf, PathBuf)>,
        moving: bool,
    ) -> Result<bool, String> {
        let mut ops = Vec::new();
        let mut cleanup_ops = Vec::new();
        for (from_parent, to_parent) in from_to_pairs {
            self.controller.check()?;

            if from_parent == to_parent {
                // Skip matching source and destination
                continue;
            }

            for entry in WalkDir::new(&from_parent).into_iter() {
                self.controller.check()?;

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
                    return Err(format!("{} is not a known file type", from.display()).into());
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
                let op = Op { kind, from, to };
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
            self.controller.check()?;

            let progress = Progress {
                current_ops,
                total_ops,
                current_bytes: 0,
                total_bytes: None,
            };
            (self.on_progress)(&op, &progress);
            if op.run(self, progress).map_err(|err| {
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

    pub fn on_progress<F: Fn(&Op, &Progress) + 'static>(mut self, f: F) -> Self {
        self.on_progress = Box::new(f);
        self
    }

    pub fn on_replace<F: Fn(&Op) -> ReplaceResult + 'static>(mut self, f: F) -> Self {
        self.on_replace = Box::new(f);
        self
    }

    fn replace(&mut self, op: &Op) -> Result<ControlFlow<bool, PathBuf>, Box<dyn Error>> {
        let replace_result = self
            .replace_result_opt
            .unwrap_or_else(|| (self.on_replace)(op));
        match replace_result {
            ReplaceResult::Replace(apply_to_all) => {
                if apply_to_all {
                    self.replace_result_opt = Some(replace_result);
                }
                fs::remove_file(&op.to)?;
                Ok(ControlFlow::Continue(op.to.clone()))
            }
            ReplaceResult::KeepBoth => match op.to.parent() {
                Some(to_parent) => Ok(ControlFlow::Continue(copy_unique_path(
                    &op.from, &to_parent,
                ))),
                None => Err(format!("failed to get parent of {:?}", op.to).into()),
            },
            ReplaceResult::Skip(apply_to_all) => {
                if apply_to_all {
                    self.replace_result_opt = Some(replace_result);
                }
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
        })
    }

    fn run(&mut self, ctx: &mut Context, mut progress: Progress) -> Result<bool, Box<dyn Error>> {
        match self.kind {
            OpKind::Copy => {
                let mut from_file = fs::OpenOptions::new().read(true).open(&self.from)?;
                let metadata = from_file.metadata()?;
                // Remove `to` if overwriting and it is an existing file
                if self.to.is_file() {
                    match ctx.replace(&self)? {
                        ControlFlow::Continue(to) => {
                            self.to = to;
                        }
                        ControlFlow::Break(ret) => {
                            return Ok(ret);
                        }
                    }
                }
                progress.total_bytes = Some(metadata.len());
                (ctx.on_progress)(&self, &progress);
                // This is atomic and ensures `to` is not created by any other process
                let mut to_file = fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&self.to)?;
                to_file.set_permissions(metadata.permissions())?;
                loop {
                    ctx.controller.check()?;

                    let count = from_file.read(&mut ctx.buf)?;
                    if count == 0 {
                        break;
                    }
                    to_file.write_all(&ctx.buf[..count])?;
                    progress.current_bytes += count as u64;
                    (ctx.on_progress)(&self, &progress);
                }
                to_file.sync_all()?;
            }
            OpKind::Move => {
                // Remove `to` if overwriting and it is an existing file
                if self.to.is_file() {
                    match ctx.replace(&self)? {
                        ControlFlow::Continue(to) => {
                            self.to = to;
                        }
                        ControlFlow::Break(ret) => {
                            return Ok(ret);
                        }
                    }
                }
                // This is atomic and ensures `to` is not created by any other process
                match fs::hard_link(&self.from, &self.to) {
                    Ok(()) => {}
                    Err(err) => {
                        //TODO: what is the error code on Windows?
                        if err.raw_os_error() == Some(libc::EXDEV) {
                            // Try standard copy if hard link fails with cross device error
                            let mut copy_op = Op {
                                kind: OpKind::Copy,
                                from: self.from.clone(),
                                to: self.to.clone(),
                            };
                            copy_op.run(ctx, progress)?;
                        } else {
                            return Err(err.into());
                        }
                    }
                }
            }
            OpKind::Mkdir => {
                fs::create_dir_all(&self.to)?;
            }
            OpKind::Remove => {
                fs::remove_file(&self.from)?;
            }
            OpKind::Rmdir => {
                fs::remove_dir(&self.from)?;
            }
            OpKind::Symlink { ref target } => {
                // Remove `to` if overwriting and it is an existing file
                if self.to.is_file() {
                    match ctx.replace(&self)? {
                        ControlFlow::Continue(to) => {
                            self.to = to;
                        }
                        ControlFlow::Break(ret) => {
                            return Ok(ret);
                        }
                    }
                }
                //TODO: use OS-specific function
                fs::soft_link(&target, &self.to)?;
            }
        }
        Ok(true)
    }
}
