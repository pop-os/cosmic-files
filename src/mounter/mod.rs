use cosmic::widget;
use std::error::Error;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

#[cfg(feature = "gvfs")]
mod gvfs;

pub trait MounterItem {
    fn name(&self) -> String;
    fn icon(&self, size: u16) -> widget::icon::Handle;
    fn path(&self) -> Option<PathBuf>;
}

pub type MounterItems = Vec<Box<dyn MounterItem>>;

pub trait Mounter {
    fn items(&self) -> Result<MounterItems, Box<dyn Error>>;
}

pub type MounterKey = &'static str;
pub type MounterMap = BTreeMap<MounterKey, Box<dyn Mounter>>;
pub type Mounters = Arc<MounterMap>;

pub fn mounters() -> Mounters {
    let mut mounters = MounterMap::new();

    #[cfg(feature = "gvfs")]
    {
        mounters.insert("gvfs", Box::new(gvfs::Gvfs::new()));
    }

    Mounters::new(mounters)
}
