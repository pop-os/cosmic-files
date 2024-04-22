use cosmic::widget;
use gio::prelude::*;
use gio::{Mount, ThemedIcon, Volume, VolumeMonitor};
use std::{error::Error, path::PathBuf};

use super::{Mounter, MounterItem, MounterItems};

pub struct Gvfs {
    monitor: VolumeMonitor,
}

impl Gvfs {
    pub fn new() -> Self {
        let monitor = VolumeMonitor::get();
        Self { monitor }
    }
}

impl Mounter for Gvfs {
    fn items(&self) -> Result<MounterItems, Box<dyn Error>> {
        let mut items = MounterItems::new();
        for mount in self.monitor.mounts() {
            items.push(Box::new(mount));
        }
        Ok(items)
    }
}

impl MounterItem for Mount {
    fn name(&self) -> String {
        MountExt::name(self).to_string()
    }

    fn icon(&self, size: u16) -> widget::icon::Handle {
        let icon = MountExt::symbolic_icon(self);
        if let Some(themed_icon) = icon.downcast_ref::<ThemedIcon>() {
            for name in themed_icon.names() {
                let named = widget::icon::from_name(name.as_str()).size(size);
                if let Some(path) = named.path() {
                    return widget::icon::from_path(path);
                }
            }
        }

        //TODO: handle more gio icon types
        widget::icon::from_name("folder-symbolic")
            .size(size)
            .handle()
    }

    fn path(&self) -> Option<PathBuf> {
        MountExt::root(self).path()
    }
}
