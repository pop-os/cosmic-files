use cosmic::iced::futures::SinkExt;
use cosmic::iced::{Subscription, stream};
use cosmic::{Task, widget};
use gio::glib;
use gio::prelude::*;
use std::any::TypeId;
use std::cell::Cell;
use std::future::pending;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use super::{Mounter, MounterAuth, MounterItem, MounterItems, MounterMessage};
use crate::config::IconSizes;
use crate::err_str;
use crate::tab::{self, ChecksumState, DirSize, ItemMetadata, ItemThumbnail, Location};

const TARGET_URI_ATTRIBUTE: &str = "standard::target-uri";

// Attributes requested when listing a network directory.
const SCAN_ATTRIBUTES: &str = "standard::name,\
standard::display-name,\
standard::type,\
standard::size,\
standard::icon,\
standard::is-hidden,\
time::modified";

fn resolve_uri(uri: &str) -> (String, gio::File) {
    let file = gio::File::for_uri(uri);
    // Resolve the target-uri if it exists
    if let Ok(file_info) = file.query_info(
        TARGET_URI_ATTRIBUTE,
        gio::FileQueryInfoFlags::NONE,
        gio::Cancellable::NONE,
    ) && let Some(resolved_uri) = file_info.attribute_as_string(TARGET_URI_ATTRIBUTE)
    {
        let resolved_uri = String::from(resolved_uri);
        let file = gio::File::for_uri(&resolved_uri);
        return (resolved_uri, file);
    }

    (uri.to_string(), file)
}

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
        // Hide shadowed mounts
        .filter(|(_, mount)| !mount.is_shadowed())
        .map(|(i, mount)| {
            let root = MountExt::root(&mount);
            let is_remote = root
                .query_filesystem_info(
                    gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE,
                    gio::Cancellable::NONE,
                )
                .ok()
                .map(|info| info.boolean(gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE))
                .unwrap_or(true); // Default to remote if query fails

            MounterItem::Gvfs(Item {
                uri: mount.root().uri().into(),
                kind: ItemKind::Mount,
                index: i,
                name: mount.name().into(),
                is_mounted: true,
                is_remote,
                icon_opt: gio_icon_to_path(&MountExt::icon(&mount), sizes.grid()),
                icon_symbolic_opt: gio_icon_to_path(&MountExt::symbolic_icon(&mount), 16),
                path_opt: root.path(),
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
                    is_remote: false,
                    icon_opt: gio_icon_to_path(&VolumeExt::icon(&volume), sizes.grid()),
                    icon_symbolic_opt: gio_icon_to_path(&VolumeExt::symbolic_icon(&volume), 16),
                    path_opt: None,
                })
            }),
    );
    items
}

fn network_scan(uri: &str, sizes: IconSizes) -> Result<Vec<tab::Item>, String> {
    let force_dir = uri.starts_with("network:///");
    let (_, file) = resolve_uri(uri);

    // Read .hidden file if present
    let hidden_files: Box<[String]> = if let Some(path) = file.path() {
        let hidden_file_path = path.join(".hidden");
        if hidden_file_path.is_file() {
            tab::parse_hidden_file(&hidden_file_path)
        } else {
            Box::from([])
        }
    } else {
        Box::from([])
    };

    // `filesystem::remote` belongs to the filesystem namespace, which `enumerate_children`
    // never fills in, so it has to be queried once for the directory being listed.
    let remote = file
        .query_filesystem_info(
            gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE,
            gio::Cancellable::NONE,
        )
        .map(|info| info.boolean(gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE))
        .unwrap_or_else(|err| {
            log::warn!("failed to get GIO filesystem info for {uri}: {err}");
            // Assume remote, so per-entry work is skipped rather than retried
            true
        });

    let mut items = Vec::new();
    for info_res in file
        .enumerate_children(
            SCAN_ATTRIBUTES,
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        )
        .map_err(err_str)?
    {
        let info = info_res.map_err(err_str)?;
        items.push(item_from_info(
            &file,
            &info,
            sizes,
            force_dir,
            !remote,
            &hidden_files,
        ));
    }
    Ok(items)
}

fn item_from_info(
    dir: &gio::File,
    info: &gio::FileInfo,
    sizes: IconSizes,
    force_dir: bool,
    count_children: bool,
    hidden_files: &[String],
) -> tab::Item {
    let name = info.name().to_string_lossy().into_owned();
    let display_name = String::from(info.display_name());

    let uri = String::from(dir.child(info.name()).uri());

    //TODO: what is the best way to resolve shortcuts?
    let location = Location::Network(uri, display_name.clone(), dir.child(&name).path());

    let metadata = if force_dir {
        ItemMetadata::SimpleDir { entries: 0 }
    } else {
        let mtime = info.attribute_uint64(gio::FILE_ATTRIBUTE_TIME_MODIFIED);
        let is_dir = matches!(info.file_type(), gio::FileType::Directory);
        let size_opt = (!is_dir).then_some(info.size() as u64);
        let mut children_opt = None;

        // Counting children costs a directory listing per entry, which is far too
        // expensive on a remote filesystem
        if is_dir && count_children {
            if let Some(path) = dir.child(&name).path() {
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
            is_dir,
        }
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

    // Check if item is hidden
    let hidden = name.starts_with('.')
        || info.boolean(gio::FILE_ATTRIBUTE_STANDARD_IS_HIDDEN)
        || hidden_files.contains(&name);

    tab::Item {
        name,
        is_mount_point: false,
        display_name,
        metadata,
        hidden,
        location_opt: Some(location),
        image_dimensions: None,
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
        checksums: ChecksumState::default(),
    }
}

fn network_search(
    uri: &str,
    regex: &regex::Regex,
    show_hidden: bool,
    sizes: IconSizes,
    callback: &dyn Fn(tab::Item) -> bool,
) -> Result<(), String> {
    let (_, root) = resolve_uri(uri);

    // Shares in a server listing are mountable entries whose canonical
    // location is only available in the target-uri attribute
    let attributes = format!("{SCAN_ATTRIBUTES},{TARGET_URI_ATTRIBUTE}");

    // Breadth-first traversal of the network location
    let mut first = true;
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::from([root]);
    while let Some(dir) = queue.pop_front() {
        // Guard against traversal cycles introduced by target-uri redirects
        if !visited.insert(String::from(dir.uri())) {
            continue;
        }

        let enumerator = match dir.enumerate_children(
            &attributes,
            // Do not follow symlinks to avoid traversal cycles
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
            gio::Cancellable::NONE,
        ) {
            Ok(ok) => ok,
            // Fail the search if the location itself cannot be enumerated
            Err(err) if first => return Err(err_str(err)),
            Err(err) => {
                log::warn!("failed to enumerate {}: {}", dir.uri(), err);
                continue;
            }
        };
        first = false;

        for info_res in enumerator {
            let info = match info_res {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!("failed to read entry in {}: {}", dir.uri(), err);
                    continue;
                }
            };

            let name = info.name().to_string_lossy().into_owned();
            let hidden =
                name.starts_with('.') || info.boolean(gio::FILE_ATTRIBUTE_STANDARD_IS_HIDDEN);
            if hidden && !show_hidden {
                continue;
            }

            if regex.is_match(&info.display_name()) || regex.is_match(&name) {
                let item = item_from_info(&dir, &info, sizes, false, false, &[]);
                if !callback(item) {
                    return Ok(());
                }
            }

            match info.file_type() {
                gio::FileType::Directory => queue.push_back(dir.child(info.name())),
                // Shares below a server (e.g. smb://server/) are mountable
                // entries; descend into them via their resolved location.
                // Enumerating an unmounted share fails with NotMounted and
                // is skipped above, so this never triggers mount prompts
                gio::FileType::Mountable => {
                    if let Some(target_uri) = info.attribute_as_string(TARGET_URI_ATTRIBUTE) {
                        queue.push_back(gio::File::for_uri(&target_uri));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn dir_info(uri: &str) -> Result<(String, String, Option<PathBuf>), glib::Error> {
    let (resolved_uri, file) = resolve_uri(uri);
    let info = file.query_info(
        gio::FILE_ATTRIBUTE_STANDARD_DISPLAY_NAME,
        gio::FileQueryInfoFlags::NONE,
        gio::Cancellable::NONE,
    )?;

    Ok((resolved_uri, info.display_name().into(), file.path()))
}

fn mount_op(
    uri: String,
    event_tx: std::sync::Weak<crate::channel::Sender<Event>>,
) -> gio::MountOperation {
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
            if let Some(event_tx) = event_tx.upgrade() {
                event_tx.send(Event::NetworkAuth(uri.clone(), auth, auth_tx));
            }
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
    DirInfo(
        String,
        mpsc::Sender<Result<(String, String, Option<PathBuf>), glib::Error>>,
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
    is_remote: bool,
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

    pub const fn is_remote(&self) -> bool {
        self.is_remote
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
    event_rx: Arc<crate::channel::Receiver<Event>>,
}

impl Gvfs {
    pub fn new() -> Self {
        //TODO: switch to using gvfs-zbus which will better integrate with async rust
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = crate::channel::channel();
        let event_tx = Arc::new(event_tx);
        std::thread::spawn(move || {
            let main_loop = glib::MainLoop::new(None, false);
            main_loop.context().spawn_local(async move {
                let event_tx = Arc::downgrade(&event_tx);
                let monitor = gio::VolumeMonitor::get();
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_changed(move |_monitor, mount| {
                        log::info!("mount changed {}", MountExt::name(mount));
                        if let Some(event_tx) = event_tx.upgrade() {
                            event_tx.send(Event::Changed);
                        }
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_added(move |_monitor, mount| {
                        log::info!("mount added {}", MountExt::name(mount));
                        if let Some(event_tx) = event_tx.upgrade() {
                            event_tx.send(Event::Changed);
                        }
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_mount_removed(move |_monitor, mount| {
                        log::info!("mount removed {}", MountExt::name(mount));
                        if let Some(event_tx) = event_tx.upgrade() {
                            event_tx.send(Event::Changed);
                        }
                    });
                }

                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_changed(move |_monitor, volume| {
                        log::info!("volume changed {}", VolumeExt::name(volume));
                        if let Some(event_tx) = event_tx.upgrade() {
                            event_tx.send(Event::Changed);
                        }
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_added(move |_monitor, volume| {
                        log::info!("volume added {}", VolumeExt::name(volume));
                        if let Some(event_tx) = event_tx.upgrade() {
                            event_tx.send(Event::Changed);
                        }
                    });
                }
                {
                    let event_tx = event_tx.clone();
                    monitor.connect_volume_removed(move |_monitor, volume| {
                        log::info!("volume removed {}", VolumeExt::name(volume));
                        if let Some(event_tx) = event_tx.upgrade() {
                            event_tx.send(Event::Changed);
                        }
                    });
                }

                while let Some(command) = command_rx.recv().await {
                    match command {
                        Cmd::Items(sizes, items_tx) => {
                            items_tx.send(items(&monitor, sizes)).await.unwrap();
                        }
                        Cmd::Rescan => {
                            let Some(event_tx) = event_tx.upgrade() else {
                                return;
                            };

                            event_tx.send(Event::Items(items(&monitor, IconSizes::default())));
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
                                let volume_for_callback = volume.clone();
                                VolumeExt::mount(
                                    &volume,
                                    gio::MountMountFlags::NONE,
                                    Some(&mount_op),
                                    gio::Cancellable::NONE,
                                    move |res| {
                                        log::info!("mount {name}: result {res:?}");
                                        // Update the mounter_item with mount information after successful mount
                                        let mut updated_item = mounter_item.clone();
                                        if res.is_ok()
                                            && let MounterItem::Gvfs(ref mut item) = updated_item
                                                && let Some(mount) = volume_for_callback.get_mount() {
                                                    let root = MountExt::root(&mount);
                                                    item.path_opt = root.path();
                                                    item.is_mounted = true;
                                                    // Query if remote
                                                    item.is_remote = root
                                                        .query_filesystem_info(
                                                            gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE,
                                                            gio::Cancellable::NONE,
                                                        )
                                                        .ok().map(|info| info.boolean(gio::FILE_ATTRIBUTE_FILESYSTEM_REMOTE))
                                                        .unwrap_or(true);
                                                }
                                        let Some(event_tx) = event_tx.upgrade() else {
                                            return;
                                        };
                                        event_tx.send(Event::MountResult(updated_item, match res {
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
                                        }));
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
                                    let Some(event_tx) = event_tx.upgrade() else {
                                        return;
                                    };
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
                                    }));
                                }
                            );
                        }
                        Cmd::NetworkScan(uri, sizes, items_tx) => {
                            let (resolved_uri, file) = resolve_uri(&uri);

                            let needs_mount = resolved_uri != "network:///" && match file.find_enclosing_mount(gio::Cancellable::NONE) {
                                Ok(_) => false,
                                Err(err) => matches!(err.kind::<gio::IOErrorEnum>(), Some(gio::IOErrorEnum::NotMounted))
                            };

                            if needs_mount {
                                let mount_op = mount_op(resolved_uri.clone(), event_tx.clone());
                                let event_tx = event_tx.clone();
                                file.mount_enclosing_volume(
                                    gio::MountMountFlags::empty(),
                                    Some(&mount_op),
                                    gio::Cancellable::NONE,
                                    move |res| {
                                        log::info!("network scan mounted {resolved_uri}: result {res:?}");
                                        // FIXME sometimes a uri can be mounted and then not recognized as mounted...
                                        // seems to be related to uri with a path
                                        items_tx.blocking_send(network_scan(&uri, sizes)).unwrap();
                                        let Some(event_tx) = event_tx.upgrade() else {
                                            return;
                                        };
                                        event_tx.send(Event::NetworkResult(resolved_uri, match res {
                                            Ok(()) => {
                                                Ok(true)
                                            },
                                            Err(err) => match err.kind::<gio::IOErrorEnum>() {
                                                Some(gio::IOErrorEnum::FailedHandled) => Ok(false),
                                                _ => Err(format!("{err}"))
                                            }
                                        }));
                                    }
                                );
                            } else {
                                items_tx.send(network_scan(&uri, sizes)).await.unwrap();
                            }
                        }
                        Cmd::DirInfo(uri, result_tx) => {
                            result_tx.send(dir_info(&uri)).await.unwrap();
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
            event_rx: Arc::new(event_rx),
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

    fn network_drive(&self, uri: String) -> Task<bool> {
        let command_tx = self.command_tx.clone();
        Task::perform(
            async move {
                let (res_tx, res_rx) = tokio::sync::oneshot::channel();

                command_tx.send(Cmd::NetworkDrive(uri, res_tx)).unwrap();
                res_rx.await
            },
            |result| match result {
                Ok(Ok(())) => true,
                Ok(Err(err)) => {
                    log::error!("{err:?}");
                    false
                }
                Err(err) => {
                    log::error!("{err:?}");
                    false
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

    fn network_search(
        &self,
        uri: &str,
        regex: &regex::Regex,
        show_hidden: bool,
        sizes: IconSizes,
        callback: &dyn Fn(tab::Item) -> bool,
    ) -> Option<Result<(), String>> {
        // Runs on the caller's thread so long searches do not block the
        // mounter command loop; synchronous GIO file operations are thread safe
        Some(network_search(uri, regex, show_hidden, sizes, callback))
    }

    fn dir_info(&self, uri: &str) -> Option<(String, String, Option<PathBuf>)> {
        let (result_tx, mut result_rx) = mpsc::channel(1);
        self.command_tx
            .send(Cmd::DirInfo(uri.to_string(), result_tx))
            .unwrap();
        result_rx.blocking_recv().and_then(|res| res.ok())
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
        struct Wrapper {
            command_tx: mpsc::UnboundedSender<Cmd>,
            event_rx: Arc<crate::channel::Receiver<Event>>,
        }
        impl Hash for Wrapper {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                TypeId::of::<Self>().hash(state);
            }
        }
        Subscription::run_with(
            Wrapper {
                command_tx,
                event_rx,
            },
            |Wrapper {
                 command_tx,
                 event_rx,
             }| {
                let command_tx = command_tx.clone();
                let event_rx = event_rx.clone();
                stream::channel(
                    1,
                    move |mut output: cosmic::iced::futures::channel::mpsc::Sender<
                        MounterMessage,
                    >| async move {
                        command_tx.send(Cmd::Rescan).unwrap();
                        while let Some(event) = event_rx.recv().await {
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
                    },
                )
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;

    use super::network_search;
    use crate::config::IconSizes;
    use crate::tab::Location;

    fn search_names(uri: &str, regex: &regex::Regex, show_hidden: bool) -> Vec<String> {
        let found = RefCell::new(Vec::new());
        network_search(uri, regex, show_hidden, IconSizes::default(), &|item| {
            found.borrow_mut().push(item.name.clone());
            true
        })
        .unwrap();
        let mut names = found.into_inner();
        names.sort();
        names
    }

    #[test]
    fn network_search_finds_nested_matches() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("subdir/nested")).unwrap();
        fs::write(root.join("match-top.txt"), b"").unwrap();
        fs::write(root.join("subdir/match-mid.txt"), b"").unwrap();
        fs::write(root.join("subdir/nested/match-deep.txt"), b"").unwrap();
        fs::write(root.join("subdir/other.txt"), b"").unwrap();
        fs::create_dir(root.join(".hidden-dir")).unwrap();
        fs::write(root.join(".hidden-dir/match-hidden.txt"), b"").unwrap();

        let uri = format!("file://{}", root.display());
        let regex = regex::RegexBuilder::new(&regex::escape("Match"))
            .case_insensitive(true)
            .build()
            .unwrap();

        assert_eq!(
            search_names(&uri, &regex, false),
            vec!["match-deep.txt", "match-mid.txt", "match-top.txt"]
        );

        // With show_hidden, hidden directories are traversed as well
        assert_eq!(
            search_names(&uri, &regex, true),
            vec![
                "match-deep.txt",
                "match-hidden.txt",
                "match-mid.txt",
                "match-top.txt"
            ]
        );

        // Callback returning false stops the search early
        let count = RefCell::new(0usize);
        network_search(&uri, &regex, true, IconSizes::default(), &|_| {
            *count.borrow_mut() += 1;
            false
        })
        .unwrap();
        assert_eq!(*count.borrow(), 1);
    }

    #[test]
    fn network_search_fails_on_missing_root() {
        let regex = regex::Regex::new("x").unwrap();
        assert!(
            network_search(
                "file:///nonexistent-cosmic-files-test-dir",
                &regex,
                false,
                IconSizes::default(),
                &|_| true,
            )
            .is_err()
        );
    }

    #[test]
    fn network_search_items_are_network_locations() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("match-dir")).unwrap();
        fs::write(root.join("match-dir/match-file.txt"), b"data").unwrap();

        let uri = format!("file://{}", root.display());
        let regex = regex::Regex::new("match").unwrap();

        let found = RefCell::new(Vec::new());
        network_search(&uri, &regex, false, IconSizes::default(), &|item| {
            found.borrow_mut().push(item);
            true
        })
        .unwrap();
        let mut items = found.into_inner();
        items.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(items.len(), 2);

        // Results must carry network locations, like items listed by
        // network_scan, so that opening them stays in the network view
        let dir_item = &items[0];
        assert_eq!(dir_item.name, "match-dir");
        assert!(dir_item.metadata.is_dir());
        match &dir_item.location_opt {
            Some(Location::Network(item_uri, display_name, path_opt)) => {
                assert_eq!(item_uri, &format!("{uri}/match-dir"));
                assert_eq!(display_name, "match-dir");
                assert_eq!(path_opt.as_deref(), Some(root.join("match-dir").as_path()));
            }
            other => panic!("expected network location, got {other:?}"),
        }

        let file_item = &items[1];
        assert_eq!(file_item.name, "match-file.txt");
        assert!(!file_item.metadata.is_dir());
        assert_eq!(file_item.metadata.file_size(), Some(4));
        // Modified time drives search result ordering in the tab
        assert!(file_item.metadata.modified().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn network_search_does_not_follow_symlink_cycles() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir(root.join("subdir")).unwrap();
        fs::write(root.join("subdir/match.txt"), b"").unwrap();
        // Symlink cycle back to the root of the search
        std::os::unix::fs::symlink(root, root.join("subdir/match-loop")).unwrap();

        let uri = format!("file://{}", root.display());
        let regex = regex::Regex::new("match").unwrap();

        // The search must terminate and report each entry exactly once
        assert_eq!(
            search_names(&uri, &regex, false),
            vec!["match-loop", "match.txt"]
        );
    }
}
