use cosmic::{
    Task,
    iced::{Subscription, futures::SinkExt, stream},
    widget,
};
use gio::{glib, prelude::*};
use std::{any::TypeId, cell::Cell, future::pending, path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, mpsc};

use super::{Mounter, MounterAuth, MounterItem, MounterItems, MounterMessage};
use crate::{
    config::IconSizes,
    err_str,
    tab::{self, DirSize, ItemMetadata, ItemThumbnail, Location},
};

const TARGET_URI_ATTRIBUTE: &str = "standard::target-uri";

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
    let mut items: MounterItems = (monitor.mounts().into_iter())
        .enumerate()
        .map(|(i, mount)| {
            MounterItem::Gvfs(Item {
                uri: mount.root().uri().into(),
                kind: ItemKind::Mount,
                index: i,
                name: mount.name().into(),
                is_mounted: true,
                icon_opt: gio_icon_to_path(&MountExt::icon(&mount), sizes.grid()),
                icon_symbolic_opt: gio_icon_to_path(&MountExt::symbolic_icon(&mount), 16),
                path_opt: MountExt::root(&mount).path(),
            })
        })
        .collect();
    items.extend(
        (monitor.volumes().into_iter())
            .enumerate()
            // Volumes with mounts are already listed by mount
            .filter(|(_, volume)| volume.get_mount().is_none())
            .map(|(i, volume)| {
                let uri = VolumeExt::activation_root(&volume)
                    .map(|f| f.uri().into())
                    .unwrap_or_default();
                MounterItem::Gvfs(Item {
                    // TODO can we get URI for volumes with no mount?
                    uri,
                    kind: ItemKind::Volume,
                    index: i,
                    name: volume.name().into(),
                    is_mounted: false,
                    icon_opt: gio_icon_to_path(&VolumeExt::icon(&volume), sizes.grid()),
                    icon_symbolic_opt: gio_icon_to_path(&VolumeExt::symbolic_icon(&volume), 16),
                    path_opt: None,
                })
            }),
    );
    items
}

fn network_scan(uri: &str, sizes: IconSizes) -> Result<Vec<tab::Item>, String> {
    let mut file = gio::File::for_uri(uri);
    let force_dir = uri.starts_with("network:///");

    // Resolve the target-uri if it exists
    if let Ok(file_info) = file.query_info(
        TARGET_URI_ATTRIBUTE,
        gio::FileQueryInfoFlags::NONE,
        gio::Cancellable::NONE,
    ) {
        if let Some(resolved_uri) = file_info.attribute_as_string(TARGET_URI_ATTRIBUTE) {
            file = gio::File::for_uri(resolved_uri.as_str());
        }
    }

    let mut items = Vec::new();
    for info_res in file
        .enumerate_children("*", gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE)
        .map_err(err_str)?
    {
        let info = info_res.map_err(err_str)?;
        let name = info.name().to_string_lossy().into_owned();
        let display_name = String::from(info.display_name());

        let uri = String::from(file.child(info.name()).uri());

        //TODO: what is the best way to resolve shortcuts?
        let location = Location::Network(uri, display_name.clone(), file.child(&name).path());

        let metadata = if !force_dir && !info.boolean(gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE) {
            let mtime = info.attribute_uint64(gio::FILE_ATTRIBUTE_TIME_MODIFIED);
            let is_dir = matches!(info.file_type(), gio::FileType::Directory);
            let size_opt = (!is_dir).then_some(info.size() as u64);
            let mut children_opt = None;

            if is_dir {
                if let Some(path) = file.child(&name).path() {
                    //TODO: calculate children in the background (and make it cancellable?)
                    match std::fs::read_dir(&path) {
                        Ok(entries) => {
                            children_opt = Some(entries.count());
                        }
                        Err(err) => {
                            log::warn!("failed to read directory {}: {}", path.display(), err);
                            children_opt = Some(0);
                        }
                    }
                } else {
                    children_opt = Some(0);
                }
            }
            ItemMetadata::GvfsPath {
                mtime,
                size_opt,
                children_opt,
            }
        } else {
            ItemMetadata::SimpleDir { entries: 0 }
        };

        let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = {
            let file_icon = |size| {
                info.icon()
                    .as_ref()
                    .and_then(|icon| gio_icon_to_path(icon, size))
                    .map(widget::icon::from_path)
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
            is_mount_point: false,
            display_name,
            metadata,
            hidden: false,
            location_opt: Some(location),
            mime,
            icon_handle_grid,
            icon_handle_list,
            icon_handle_list_condensed,
            thumbnail_opt: Some(ItemThumbnail::NotImage),
            button_id: widget::Id::unique(),
            pos_opt: Cell::new(None),
            rect_opt: Cell::new(None),
            selected: false,
            highlighted: false,
            overlaps_drag_rect: false,
            //TODO: scan directory size on gvfs mounts?
            dir_size: DirSize::NotDirectory,
            cut: false,
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
                username_opt: flags
                    .contains(gio::AskPasswordFlags::NEED_USERNAME)
                    .then(|| default_user.to_string()),
                domain_opt: flags
                    .contains(gio::AskPasswordFlags::NEED_DOMAIN)
                    .then(|| default_domain.to_string()),
                password_opt: flags
                    .contains(gio::AskPasswordFlags::NEED_PASSWORD)
                    .then(String::new),
                remember_opt: flags
                    .contains(gio::AskPasswordFlags::SAVING_SUPPORTED)
                    .then_some(false),
                anonymous_opt: flags
                    .contains(gio::AskPasswordFlags::ANONYMOUS_SUPPORTED)
                    .then_some(false),
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
    Mount(
        MounterItem,
        tokio::sync::oneshot::Sender<anyhow::Result<()>>,
    ),
    NetworkDrive(String, tokio::sync::oneshot::Sender<anyhow::Result<()>>),
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
    uri: String,
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

    pub const fn is_mounted(&self) -> bool {
        self.is_mounted
    }

    pub fn uri(&self) -> String {
        self.uri.clone()
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
                        Cmd::Mount(mounter_item, complete_tx) => {
                            let MounterItem::Gvfs(ref item) = mounter_item else {
                                _ = complete_tx.send(Err(anyhow::anyhow!("No mounter item")));
                                continue
                            };
                            let ItemKind::Volume = item.kind else {
                                _ = complete_tx.send(Err(anyhow::anyhow!("No mounter volume")));
                                continue
                            };
                            for (i, volume) in monitor.volumes().into_iter().enumerate() {
                                if i != item.index {
                                    continue;
                                }

                                let name = VolumeExt::name(&volume);
                                if item.name != name {
                                    log::warn!("trying to mount volume {} failed: name is {:?} when {:?} was expected", i, name, item.name);
                                    continue;
                                }

                                log::info!("mount {name}");
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
                                        log::info!("mount {name}: result {res:?}");
                                        event_tx.send(Event::MountResult(mounter_item, match res {
                                            Ok(()) => {
                                                _ = complete_tx.send(Ok(()));
                                                Ok(true)
                                            },
                                            Err(err) => {
                                                _ = complete_tx.send(Err(anyhow::anyhow!("{err:?}")));
                                                match err.kind::<gio::IOErrorEnum>() {
                                                Some(gio::IOErrorEnum::FailedHandled) => Ok(false),
                                                _ => Err(format!("{err}"))
                                            }}
                                        })).unwrap();
                                    },
                                );
                                break;
                            }
                        }
                        Cmd::NetworkDrive(uri, result_tx) => {
                            let file = gio::File::for_uri(&uri);
                            let mount_op = mount_op(uri.clone(), event_tx.clone());
                            let event_tx = event_tx.clone();
                            file.mount_enclosing_volume(
                                gio::MountMountFlags::NONE,
                                Some(&mount_op),
                                gio::Cancellable::NONE,
                                move |res| {
                                    log::info!("network drive {uri}: result {res:?}");
                                    event_tx.send(Event::NetworkResult(uri, match res {
                                        Ok(()) => {
                                            _ = result_tx.send(Ok(()));
                                            Ok(true)},
                                        Err(err) => {
                                            _ = result_tx.send(Err(anyhow::anyhow!("{err:?}")));
                                            match err.kind::<gio::IOErrorEnum>() {
                                            Some(gio::IOErrorEnum::FailedHandled) => Ok(false),
                                            _ => Err(format!("{err}"))
                                        }}
                                    })).unwrap();
                                }
                            );
                        }
                        Cmd::NetworkScan(mut uri, sizes, items_tx) => {
                            let original_uri = uri.clone();
                            let mut file = gio::File::for_uri(&uri);
                            if let Ok(file_info) = file.query_info(
                                TARGET_URI_ATTRIBUTE,
                                gio::FileQueryInfoFlags::NONE,
                                gio::Cancellable::NONE,
                            ) {
                                if let Some(resolved_uri) = file_info.attribute_as_string(TARGET_URI_ATTRIBUTE) {
                                    uri = resolved_uri.into();
                                    file = gio::File::for_uri(&uri);
                                }
                            }

                            let needs_mount = uri != "network:///" && match file.find_enclosing_mount(gio::Cancellable::NONE) {
                                Ok(_) => false,
                                Err(err) => matches!(err.kind::<gio::IOErrorEnum>(), Some(gio::IOErrorEnum::NotMounted))
                            };

                            if needs_mount {
                                let mount_op = mount_op(uri.clone(), event_tx.clone());
                                let event_tx = event_tx.clone();
                                file.mount_enclosing_volume(
                                    gio::MountMountFlags::empty(),
                                    Some(&mount_op),
                                    gio::Cancellable::NONE,
                                    move |res| {
                                        log::info!("network scan mounted {uri}: result {res:?}");
                                        // FIXME sometimes a uri can be mounted and then not recognized as mounted...
                                        // seems to be related to uri with a path
                                        items_tx.blocking_send(network_scan(&original_uri, sizes)).unwrap();
                                        event_tx.send(Event::NetworkResult(uri, match res {
                                            Ok(()) => {
                                                Ok(true)
                                            },
                                            Err(err) => match err.kind::<gio::IOErrorEnum>() {
                                                Some(gio::IOErrorEnum::FailedHandled) => Ok(false),
                                                _ => Err(format!("{err}"))
                                            }
                                        })).unwrap();
                                    }
                                );
                            } else {
                                items_tx.send(network_scan(&original_uri, sizes)).await.unwrap();
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

                                if MountExt::can_eject(&mount) {
                                    log::info!("eject {name}");
                                    MountExt::eject_with_operation(
                                        &mount,
                                        gio::MountUnmountFlags::NONE,
                                        gio::MountOperation::NONE,
                                        gio::Cancellable::NONE,
                                        move |result| {
                                            log::info!("eject {name}: result {result:?}");
                                        },
                                    );
                                } else {
                                    log::info!("unmount {name}");
                                    MountExt::unmount_with_operation(
                                        &mount,
                                        gio::MountUnmountFlags::NONE,
                                        gio::MountOperation::NONE,
                                        gio::Cancellable::NONE,
                                        move |result| {
                                            log::info!("unmount {name}: result {result:?}");
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            });
            main_loop.run();
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
                let (res_tx, res_rx) = tokio::sync::oneshot::channel();

                command_tx.send(Cmd::Mount(item, res_tx)).unwrap();
                res_rx.await
            },
            |x| {
                if let Err(err) = x {
                    log::error!("{err:?}");
                }
            },
        )
    }

    fn network_drive(&self, uri: String) -> Task<()> {
        let command_tx = self.command_tx.clone();
        Task::perform(
            async move {
                let (res_tx, res_rx) = tokio::sync::oneshot::channel();

                command_tx.send(Cmd::NetworkDrive(uri, res_tx)).unwrap();
                res_rx.await
            },
            |x| {
                if let Err(err) = x {
                    log::error!("{err:?}");
                }
            },
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
        Task::future(async move {
            command_tx.send(Cmd::Unmount(item)).unwrap();
        })
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
                            output.send(MounterMessage::Items(items)).await.unwrap();
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
