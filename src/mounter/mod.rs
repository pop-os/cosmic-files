use cosmic::iced::Subscription;
use cosmic::{Task, widget};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use tokio::sync::mpsc;

use crate::config::IconSizes;
use crate::tab;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskUsage {
    pub free: u64,
    pub total: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskUsageLevel {
    Normal,
    Warning,
    Critical,
}

impl DiskUsage {
    #[allow(clippy::cast_precision_loss)]
    pub fn fraction(self) -> f32 {
        if self.total == 0 || self.free > self.total {
            return 0.0;
        }

        (self.total - self.free) as f32 / self.total as f32
    }

    pub fn level(self) -> DiskUsageLevel {
        if self.total == 0 || self.free > self.total {
            return DiskUsageLevel::Normal;
        }

        let used = u128::from(self.total - self.free);
        let total = u128::from(self.total);
        if used * 100 >= total * 90 {
            DiskUsageLevel::Critical
        } else if used * 100 >= total * 80 {
            DiskUsageLevel::Warning
        } else {
            DiskUsageLevel::Normal
        }
    }
}

#[cfg(feature = "gvfs")]
mod gvfs;

#[derive(Clone)]
pub struct MounterAuth {
    pub message: String,
    pub username_opt: Option<String>,
    pub domain_opt: Option<String>,
    pub password_opt: Option<String>,
    pub remember_opt: Option<bool>,
    pub anonymous_opt: Option<bool>,
}

// Custom debug for MounterAuth to hide password
impl fmt::Debug for MounterAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MounterAuth")
            .field("username_opt", &self.username_opt)
            .field("domain_opt", &self.domain_opt)
            .field(
                "password_opt",
                if self.password_opt.is_some() {
                    &"Some(*)"
                } else {
                    &"None"
                },
            )
            .field("remember_opt", &self.remember_opt)
            .field("anonymous_opt", &self.anonymous_opt)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum MounterItem {
    #[cfg(feature = "gvfs")]
    Gvfs(gvfs::Item),
    #[allow(dead_code)]
    None,
}

impl MounterItem {
    pub fn name(&self) -> String {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.name(),
            Self::None => unreachable!(),
        }
    }

    pub fn uri(&self) -> String {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.uri(),
            Self::None => unreachable!(),
        }
    }

    pub fn is_mounted(&self) -> bool {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.is_mounted(),
            Self::None => unreachable!(),
        }
    }

    pub fn icon(&self, symbolic: bool) -> Option<widget::icon::Handle> {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.icon(symbolic),
            Self::None => unreachable!(),
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.path(),
            Self::None => unreachable!(),
        }
    }

    pub fn is_remote(&self) -> bool {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.is_remote(),
            Self::None => unreachable!(),
        }
    }

    pub fn disk_usage(&self) -> Option<DiskUsage> {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.disk_usage(),
            Self::None => unreachable!(),
        }
    }

    pub fn can_unmount(&self) -> bool {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.can_unmount(),
            Self::None => unreachable!(),
        }
    }
}

pub type MounterItems = Vec<MounterItem>;

#[derive(Clone, Debug)]
pub enum MounterMessage {
    Items(MounterItems),
    MountResult(MounterItem, Result<bool, String>),
    NetworkAuth(String, MounterAuth, mpsc::Sender<MounterAuth>),
    NetworkResult(String, Result<bool, String>),
}

pub trait Mounter: Send + Sync {
    fn items(&self, sizes: IconSizes) -> Option<MounterItems>;
    //TODO: send result
    fn mount(&self, item: MounterItem) -> Task<()>;
    fn network_drive(&self, uri: String) -> Task<bool>;
    fn network_scan(&self, uri: &str, sizes: IconSizes) -> Option<Result<Vec<tab::Item>, String>>;
    fn dir_info(&self, uri: &str) -> Option<(String, String, Option<PathBuf>)>;
    fn unmount(&self, item: MounterItem) -> Task<()>;
    fn subscription(&self) -> Subscription<MounterMessage>;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MounterKey(pub &'static str);
pub type MounterMap = BTreeMap<MounterKey, Box<dyn Mounter>>;
pub type Mounters = Arc<MounterMap>;

pub fn mounters() -> Mounters {
    #[allow(unused_mut)]
    let mut mounters = MounterMap::new();

    #[cfg(feature = "gvfs")]
    {
        mounters.insert(MounterKey("gvfs"), Box::new(gvfs::Gvfs::new()));
    }

    Mounters::new(mounters)
}

pub static MOUNTERS: LazyLock<Mounters> = LazyLock::new(mounters);
