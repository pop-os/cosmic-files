use cosmic::{
    iced::{futures::SinkExt, stream, Subscription},
    widget, Task,
};
use gio::{glib, prelude::*};
use std::{any::TypeId, cell::Cell, future::pending, path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, Mutex};

use super::{Mounter, MounterAuth, MounterItem, MounterItems, MounterMessage};
use crate::{
    config::IconSizes,
    err_str,
    tab::{self, ItemMetadata, ItemThumbnail, Location},
};

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

fn items(monitor: &gio::VolumeMonitor, sizes: IconSizes) -> MounterItems {
    let mut items = MounterItems::new();
    for (i, mount) in monitor.mounts().into_iter().enumerate() {
        items.push(MounterItem::Gvfs(Item {
            kind: ItemKind::Mount,
            index: i,
            name: MountExt::name(&mount).to_string(),
            is_mounted: true,
            icon_opt: gio_icon_to_path(&MountExt::icon(&mount), sizes.grid()),
            icon_symbolic_opt: gio_icon_to_path(&MountExt::symbolic_icon(&mount), 16),
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
            icon_opt: gio_icon_to_path(&VolumeExt::icon(&volume), sizes.grid()),
            icon_symbolic_opt: gio_icon_to_path(&VolumeExt::symbolic_icon(&volume), 16),
            path_opt: None,
        }));
    }
    items
}

fn network_scan(uri: &str, sizes: IconSizes) -> Result<Vec<tab::Item>, String> {
    let file = gio::File::for_uri(uri);
    let mut items = Vec::new();
    for info_res in file
        .enumerate_children("*", gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE)
        .map_err(err_str)?
    {
        let info = info_res.map_err(err_str)?;
        println!("{:?}", info.display_name());
        for attribute in info.list_attributes(None) {
            println!(
                "  {:?}: {:?}: {:?}",
                attribute,
                info.attribute_type(&attribute),
                info.attribute_as_string(&attribute)
            );
        }

        let name = info.name().to_string_lossy().to_string();
        let display_name = info.display_name().to_string();

        //TODO: what is the best way to resolve shortcuts?
        let location = Location::Network(
            if let Some(target_uri) = info.attribute_string(gio::FILE_ATTRIBUTE_STANDARD_TARGET_URI)
            {
                target_uri.to_string()
            } else {
                file.child(info.name()).uri().to_string()
            },
            display_name.clone(),
        );

        //TODO: support dir or file
        let metadata = ItemMetadata::SimpleDir { entries: 0 };

        let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = {
            let file_icon = |size| {
                info.icon()
                    .as_ref()
                    .and_then(|icon| gio_icon_to_path(icon, size))
                    .map(|path| widget::icon::from_path(path))
                    .unwrap_or(
                        widget::icon::from_name(if metadata.is_dir() {
                            "folder"
                        } else {
                            "text-x-generic"
                        })
                        .size(size)
                        .handle(),
                    )
            };
            (
                //TODO: get mime from content_type?
                "inode/directory".parse().unwrap(),
                file_icon(sizes.grid()),
                file_icon(sizes.list()),
                file_icon(sizes.list_condensed()),
            )
        };

        items.push(tab::Item {
            name,
            display_name,
            metadata,
            hidden: false,
            location_opt: Some(location),
            mime,
            icon_handle_grid,
            icon_handle_list,
            icon_handle_list_condensed,
            open_with: Vec::new(),
            thumbnail_opt: Some(ItemThumbnail::NotImage),
            button_id: widget::Id::unique(),
            pos_opt: Cell::new(None),
            rect_opt: Cell::new(None),
            selected: false,
            highlighted: false,
            overlaps_drag_rect: false,
        });
    }
    Ok(items)
}

fn mount_op(uri: String, event_tx: mpsc::UnboundedSender<Event>) -> gio::MountOperation {
    let mount_op = gio::MountOperation::new();
    mount_op.connect_ask_password(
        move |mount_op, message, default_user, default_domain, flags| {
            let auth = MounterAuth {
                message: message.to_string(),
                username_opt: if flags.contains(gio::AskPasswordFlags::NEED_USERNAME) {
                    Some(default_user.to_string())
                } else {
                    None
                },
                domain_opt: if flags.contains(gio::AskPasswordFlags::NEED_DOMAIN) {
                    Some(default_domain.to_string())
                } else {
                    None
                },
                password_opt: if flags.contains(gio::AskPasswordFlags::NEED_PASSWORD) {
                    Some(String::new())
                } else {
                    None
                },
                remember_opt: if flags.contains(gio::AskPasswordFlags::SAVING_SUPPORTED) {
                    Some(false)
                } else {
                    None
                },
                anonymous_opt: if flags.contains(gio::AskPasswordFlags::ANONYMOUS_SUPPORTED) {
                    Some(false)
                } else {
                    None
                },
            };
            let (auth_tx, mut auth_rx) = mpsc::channel(1);
            event_tx
                .send(Event::NetworkAuth(uri.clone(), auth, auth_tx))
                .unwrap();
            //TODO: async recv?
            if let Some(auth) = auth_rx.blocking_recv() {
                if auth.anonymous_opt == Some(true) {
                    mount_op.set_anonymous(true);
                } else {
                    mount_op.set_username(auth.username_opt.as_deref());
                    mount_op.set_domain(auth.domain_opt.as_deref());
                    mount_op.set_password(auth.password_opt.as_deref());
                    if auth.remember_opt == Some(true) {
                        mount_op.set_password_save(gio::PasswordSave::Permanently);
                    }
                }
                mount_op.reply(gio::MountOperationResult::Handled);
            } else {
                mount_op.reply(gio::MountOperationResult::Aborted);
            }
        },
    );
    mount_op
}

enum Cmd {
    Items(IconSizes, mpsc::Sender<MounterItems>),
    Rescan,
    Mount(MounterItem),
    NetworkDrive(String),
    NetworkScan(
        String,
        IconSizes,
        mpsc::Sender<Result<Vec<tab::Item>, String>>,
    ),
    Unmount(MounterItem),
}

enum Event {
    Changed,
    Items(MounterItems),
    MountResult(MounterItem, Result<bool, String>),
    NetworkAuth(String, MounterAuth, mpsc::Sender<MounterAuth>),
    NetworkResult(String, Result<bool, String>),
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
    icon_symbolic_opt: Option<PathBuf>,
    path_opt: Option<PathBuf>,
}

impl Item {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn is_mounted(&self) -> bool {
        self.is_mounted
    }

    pub fn icon(&self, symbolic: bool) -> Option<widget::icon::Handle> {
        if symbolic {
            self.icon_symbolic_opt.as_ref()
        } else {
            self.icon_opt.as_ref()
        }
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
                        log::info!("mount changed {}", MountExt::name(mount));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_added(move |_monitor, mount| {
                        log::info!("mount added {}", MountExt::name(mount));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_removed(move |_monitor, mount| {
                        log::info!("mount removed {}", MountExt::name(mount));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }

                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_changed(move |_monitor, volume| {
                        log::info!("volume changed {}", VolumeExt::name(volume));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_added(move |_monitor, volume| {
                        log::info!("volume added {}", VolumeExt::name(volume));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_removed(move |_monitor, volume| {
                        log::info!("volume removed {}", VolumeExt::name(volume));
                        event_tx.send(Event::Changed).unwrap();
                    });
                }

                while let Some(command) = command_rx.recv().await {
                    match command {
                        Cmd::Items(sizes, items_tx) => {
                            items_tx.send(items(&monitor, sizes)).await.unwrap();
                        }
                        Cmd::Rescan => {
                            event_tx.send(Event::Items(items(&monitor, IconSizes::default()))).unwrap();
                        }
                        Cmd::Mount(mounter_item) => {
                            let MounterItem::Gvfs(ref item) = mounter_item else { continue };
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
                                //TODO: do not use name as a URI for mount_op
                                let mount_op = mount_op(name.to_string(), event_tx.clone());
                                let event_tx = event_tx.clone();
                                let mounter_item = mounter_item.clone();
                                VolumeExt::mount(
                                    &volume,
                                    gio::MountMountFlags::NONE,
                                    Some(&mount_op),
                                    gio::Cancellable::NONE,
                                    move |res| {
                                        log::info!("mount {}: result {:?}", name, res);
                                        event_tx.send(Event::MountResult(mounter_item, match res {
                                            Ok(()) => Ok(true),
                                            Err(err) => match err.kind::<gio::IOErrorEnum>() {
                                                Some(gio::IOErrorEnum::FailedHandled) => Ok(false),
                                                _ => Err(format!("{}", err))
                                            }
                                        })).unwrap();
                                    },
                                );
                            }
                        }
                        Cmd::NetworkDrive(uri) => {
                            let file = gio::File::for_uri(&uri);
                            let mount_op = mount_op(uri.clone(), event_tx.clone());
                            let event_tx = event_tx.clone();
                            file.mount_enclosing_volume(
                                gio::MountMountFlags::NONE,
                                Some(&mount_op),
                                gio::Cancellable::NONE,
                                move |res| {
                                    log::info!("network drive {}: result {:?}", uri, res);
                                    event_tx.send(Event::NetworkResult(uri, match res {
                                        Ok(()) => Ok(true),
                                        Err(err) => match err.kind::<gio::IOErrorEnum>() {
                                            Some(gio::IOErrorEnum::FailedHandled) => Ok(false),
                                            _ => Err(format!("{}", err))
                                        }
                                    })).unwrap();
                                }
                            );
                        }
                        Cmd::NetworkScan(uri, sizes, items_tx) => {
                            let file = gio::File::for_uri(&uri);
                            let needs_mount = match file.find_enclosing_mount(gio::Cancellable::NONE) {
                                Ok(_) => false,
                                Err(err) => match err.kind::<gio::IOErrorEnum>() {
                                    Some(gio::IOErrorEnum::NotMounted) => true,
                                    _ => false
                                }
                            };
                            if needs_mount {
                                let mount_op = mount_op(uri.clone(), event_tx.clone());
                                let event_tx = event_tx.clone();
                                file.mount_enclosing_volume(
                                    gio::MountMountFlags::empty(),
                                    Some(&mount_op),
                                    gio::Cancellable::NONE,
                                    move |res| {
                                        log::info!("network scan mounted {}: result {:?}", uri, res);
                                        items_tx.blocking_send(network_scan(&uri, sizes)).unwrap();
                                        event_tx.send(Event::NetworkResult(uri, match res {
                                            Ok(()) => {
                                                Ok(true)
                                            },
                                            Err(err) => match err.kind::<gio::IOErrorEnum>() {
                                                Some(gio::IOErrorEnum::FailedHandled) => Ok(false),
                                                _ => Err(format!("{}", err))
                                            }
                                        })).unwrap();
                                    }
                                );
                            } else {
                                items_tx.send(network_scan(&uri, sizes)).await.unwrap();
                            }
                        }
                        Cmd::Unmount(mounter_item) => {
                            let MounterItem::Gvfs(item) = mounter_item else { continue };
                            let ItemKind::Mount = item.kind else { continue };
                            for (i, mount) in monitor.mounts().into_iter().enumerate() {
                                if i != item.index {
                                    continue;
                                }

                                let name = MountExt::name(&mount);
                                if item.name != name {
                                    log::warn!("trying to unmount mount {} failed: name is {:?} when {:?} was expected", i, name, item.name);
                                    continue;
                                }

                                log::info!("unmount {}", name);
                                MountExt::eject_with_operation(
                                    &mount,
                                    gio::MountUnmountFlags::NONE,
                                    gio::MountOperation::NONE,
                                    gio::Cancellable::NONE,
                                    move |result| {
                                        log::info!("unmount {}: result {:?}", name, result);
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
    fn items(&self, sizes: IconSizes) -> Option<MounterItems> {
        let (items_tx, mut items_rx) = mpsc::channel(1);
        self.command_tx.send(Cmd::Items(sizes, items_tx)).unwrap();
        items_rx.blocking_recv()
    }

    fn mount(&self, item: MounterItem) -> Task<()> {
        let command_tx = self.command_tx.clone();
        Task::perform(
            async move {
                command_tx.send(Cmd::Mount(item)).unwrap();
                ()
            },
            |x| x,
        )
    }

    fn network_drive(&self, uri: String) -> Task<()> {
        let command_tx = self.command_tx.clone();
        Task::perform(
            async move {
                command_tx.send(Cmd::NetworkDrive(uri)).unwrap();
                ()
            },
            |x| x,
        )
    }

    fn network_scan(&self, uri: &str, sizes: IconSizes) -> Option<Result<Vec<tab::Item>, String>> {
        let (items_tx, mut items_rx) = mpsc::channel(1);
        self.command_tx
            .send(Cmd::NetworkScan(uri.to_string(), sizes, items_tx))
            .unwrap();
        items_rx.blocking_recv()
    }

    fn unmount(&self, item: MounterItem) -> Task<()> {
        let command_tx = self.command_tx.clone();
        Task::perform(
            async move {
                command_tx.send(Cmd::Unmount(item)).unwrap();
                ()
            },
            |x| x,
        )
    }

    fn subscription(&self) -> Subscription<MounterMessage> {
        let command_tx = self.command_tx.clone();
        let event_rx = self.event_rx.clone();
        Subscription::run_with_id(
            TypeId::of::<Self>(),
            stream::channel(1, |mut output| async move {
                command_tx.send(Cmd::Rescan).unwrap();
                while let Some(event) = event_rx.lock().await.recv().await {
                    match event {
                        Event::Changed => command_tx.send(Cmd::Rescan).unwrap(),
                        Event::Items(items) => {
                            output.send(MounterMessage::Items(items)).await.unwrap()
                        }
                        Event::MountResult(item, res) => output
                            .send(MounterMessage::MountResult(item, res))
                            .await
                            .unwrap(),
                        Event::NetworkAuth(uri, auth, auth_tx) => output
                            .send(MounterMessage::NetworkAuth(uri, auth, auth_tx))
                            .await
                            .unwrap(),
                        Event::NetworkResult(uri, res) => output
                            .send(MounterMessage::NetworkResult(uri, res))
                            .await
                            .unwrap(),
                    }
                }
                pending().await
            }),
        )
    }
}
