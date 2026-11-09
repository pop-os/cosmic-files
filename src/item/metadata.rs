use std::fs::Metadata;
use std::time::{Duration, SystemTime};

use crate::trash::{TrashItem, TrashItemSize};

#[derive(Clone, Debug)]
pub enum ItemMetadata {
    Path {
        metadata: Metadata,
        children_opt: Option<usize>,
    },
    Trash {
        size: TrashItemSize,
        entry: TrashItem,
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
        is_dir: bool,
    },
}

impl ItemMetadata {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::Path { metadata, .. } => metadata.is_dir(),
            Self::Trash { size, .. } => match size {
                TrashItemSize::Entries(_) => true,
                TrashItemSize::Bytes(_) => false,
            },
            Self::SimpleDir { .. } => true,
            Self::SimpleFile { .. } => false,
            #[cfg(feature = "gvfs")]
            Self::GvfsPath { is_dir, .. } => *is_dir,
        }
    }

    pub fn modified(&self) -> Option<SystemTime> {
        match self {
            Self::Path { metadata, .. } => metadata.modified().ok(),
            #[cfg(feature = "gvfs")]
            Self::GvfsPath { mtime, .. } => {
                SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(*mtime))
            }
            _ => None,
        }
    }

    pub fn file_size(&self) -> Option<u64> {
        match self {
            Self::Path { metadata, .. } => (!metadata.is_dir()).then_some(metadata.len()),
            Self::Trash { size, .. } => match size {
                TrashItemSize::Bytes(size) => Some(*size),
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

    pub(super) fn same_state(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Path { metadata, .. },
                Self::Path {
                    metadata: other, ..
                },
            ) => {
                metadata.file_type() == other.file_type()
                    && metadata.len() == other.len()
                    && metadata.modified().ok() == other.modified().ok()
            }
            #[cfg(feature = "gvfs")]
            (
                Self::GvfsPath {
                    mtime,
                    size_opt,
                    is_dir,
                    ..
                },
                Self::GvfsPath {
                    mtime: other_mtime,
                    size_opt: other_size_opt,
                    is_dir: other_is_dir,
                    ..
                },
            ) => mtime == other_mtime && size_opt == other_size_opt && is_dir == other_is_dir,
            _ => false,
        }
    }

    pub(super) fn keep_children_count(&mut self, old: &Self) {
        match (self, old) {
            (
                Self::Path { children_opt, .. },
                Self::Path {
                    children_opt: old_children_opt,
                    ..
                },
            ) if children_opt.is_none() => *children_opt = *old_children_opt,
            #[cfg(feature = "gvfs")]
            (
                Self::GvfsPath { children_opt, .. },
                Self::GvfsPath {
                    children_opt: old_children_opt,
                    ..
                },
            ) if children_opt.is_none() => *children_opt = *old_children_opt,
            _ => {}
        }
    }
}
