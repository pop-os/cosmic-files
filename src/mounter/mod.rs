use cosmic::{iced::subscription, widget, Command};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

#[cfg(feature = "gvfs")]
mod gvfs;

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

    pub fn icon(&self) -> Option<widget::icon::Handle> {
        match self {
            #[cfg(feature = "gvfs")]
            Self::Gvfs(item) => item.icon(),
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

pub trait Mounter {
    //TODO: send result
    fn mount(&self, item: MounterItem) -> Command<()>;
    fn subscription(&self) -> subscription::Subscription<MounterItems>;
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
