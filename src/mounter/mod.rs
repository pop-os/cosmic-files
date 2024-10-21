use cosmic::{iced::Subscription, widget, Task};
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, fmt, path::PathBuf, sync::Arc};
use tokio::sync::mpsc;

use crate::{config::IconSizes, tab};

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
    fn network_drive(&self, uri: String) -> Task<()>;
    fn network_scan(&self, uri: &str, sizes: IconSizes) -> Option<Result<Vec<tab::Item>, String>>;
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

pub static MOUNTERS: Lazy<Mounters> = Lazy::new(|| mounters());
