use cosmic::{
    iced::{futures::SinkExt, subscription},
    widget, Command,
};
use gio::{glib, prelude::*};
use std::{any::TypeId, future::pending, path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, Mutex};

use super::{Mounter, MounterItem, MounterItems};

fn gio_icon_to_path(icon: &gio::Icon, size: u16) -> Option<PathBuf> {
    if let Some(themed_icon) = icon.downcast_ref::<gio::ThemedIcon>() {
        for name in themed_icon.names() {
            let named = widget::icon::from_name(name.as_str()).size(size);
            if let Some(path) = named.path() {
                return Some(path);
            }
        }
    }
    //TODO: handle more gio icon types
    None
}

enum Cmd {
    Rescan,
    Mount(MounterItem),
}

enum Event {
    Changed,
    Items(MounterItems),
}

#[derive(Clone, Debug)]
enum ItemKind {
    Mount,
    Volume,
}

//TODO: better method of matching items
#[derive(Clone, Debug)]
pub struct Item {
    kind: ItemKind,
    index: usize,
    name: String,
    is_mounted: bool,
    icon_opt: Option<PathBuf>,
    path_opt: Option<PathBuf>,
}

impl Item {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn is_mounted(&self) -> bool {
        self.is_mounted
    }

    pub fn icon(&self) -> Option<widget::icon::Handle> {
        self.icon_opt
            .as_ref()
            .map(|icon| widget::icon::from_path(icon.clone()))
    }

    pub fn path(&self) -> Option<PathBuf> {
        self.path_opt.clone()
    }
}

pub struct Gvfs {
    command_tx: mpsc::UnboundedSender<Cmd>,
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<Event>>>,
}

impl Gvfs {
    pub fn new() -> Self {
        //TODO: switch to using gvfs-zbus which will better integrate with async rust
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        std::thread::spawn(move || {
            let main_loop = glib::MainLoop::new(None, false);
            main_loop.context().spawn_local(async move {
                let monitor = gio::VolumeMonitor::get();
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_changed(move |_monitor, mount| {
                        eprintln!("mount changed {}", MountExt::name(mount));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_added(move |_monitor, mount| {
                        eprintln!("mount added {}", MountExt::name(mount));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_removed(move |_monitor, mount| {
                        eprintln!("mount removed {}", MountExt::name(mount));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }

                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_changed(move |_monitor, volume| {
                        eprintln!("volume changed {}", VolumeExt::name(volume));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_added(move |_monitor, volume| {
                        eprintln!("volume added {}", VolumeExt::name(volume));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_removed(move |_monitor, volume| {
                        eprintln!("volume removed {}", VolumeExt::name(volume));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }

                while let Some(command) = command_rx.recv().await {
                    match command {
                        Cmd::Rescan => {
                            let mut items = MounterItems::new();
                            for (i, mount) in monitor.mounts().into_iter().enumerate() {
                                items.push(MounterItem::Gvfs(Item {
                                    kind: ItemKind::Mount,
                                    index: i,
                                    name: MountExt::name(&mount).to_string(),
                                    is_mounted: true,
                                    icon_opt: gio_icon_to_path(
                                        &MountExt::symbolic_icon(&mount),
                                        16,
                                    ),
                                    path_opt: MountExt::root(&mount).path(),
                                }));
                            }
                            for (i, volume) in monitor.volumes().into_iter().enumerate() {
                                if volume.get_mount().is_some() {
                                    // Volumes with mounts are already listed by mount
                                    continue;
                                }
                                items.push(MounterItem::Gvfs(Item {
                                    kind: ItemKind::Volume,
                                    index: i,
                                    name: VolumeExt::name(&volume).to_string(),
                                    is_mounted: false,
                                    icon_opt: gio_icon_to_path(
                                        &VolumeExt::symbolic_icon(&volume),
                                        16,
                                    ),
                                    path_opt: None,
                                }));
                            }
                            event_tx.send(Event::Items(items)).unwrap();
                        }
                        Cmd::Mount(mounter_item) => {
                            #[allow(irrefutable_let_patterns)]
                            let MounterItem::Gvfs(item) = mounter_item else { continue };
                            let ItemKind::Volume = item.kind else { continue };
                            for (i, volume) in monitor.volumes().into_iter().enumerate() {
                                if i != item.index {
                                    continue;
                                }

                                let name = VolumeExt::name(&volume);
                                if item.name != name {
                                    log::warn!("trying to mount volume {} failed: name is {:?} when {:?} was expected", i, name, item.name);
                                    continue;
                                }

                                log::info!("mount {}", name);
                                VolumeExt::mount(
                                    &volume,
                                    gio::MountMountFlags::NONE,
                                    //TODO: gio::MountOperation needed for network shares with auth
                                    gio::MountOperation::NONE,
                                    gio::Cancellable::NONE,
                                    move |result| {
                                        log::info!("mount {}: result {:?}", name, result);
                                    },
                                );
                            }
                        }
                    }
                }
            });
            main_loop.run()
        });
        Self {
            command_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
        }
    }
}

impl Mounter for Gvfs {
    fn mount(&self, item: MounterItem) -> Command<()> {
        let command_tx = self.command_tx.clone();
        Command::perform(
            async move {
                command_tx.send(Cmd::Mount(item)).unwrap();
                ()
            },
            |x| x,
        )
    }

    fn subscription(&self) -> subscription::Subscription<MounterItems> {
        let command_tx = self.command_tx.clone();
        let event_rx = self.event_rx.clone();
        subscription::channel(TypeId::of::<Self>(), 1, |mut output| async move {
            command_tx.send(Cmd::Rescan).unwrap();
            while let Some(event) = event_rx.lock().await.recv().await {
                match event {
                    Event::Changed => {
                        command_tx.send(Cmd::Rescan).unwrap();
                    }
                    Event::Items(items) => output.send(items).await.unwrap(),
                }
            }
            pending().await
        })
    }
}
