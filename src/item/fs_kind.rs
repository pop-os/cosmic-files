use rustc_hash::FxHashMap;
use std::fs::Metadata;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsKind {
    Local,
    Remote,
    Gvfs,
}

impl FsKind {
    #[cfg(target_os = "linux")]
    pub fn new(metadata: &Metadata) -> FsKind {
        use std::os::unix::fs::MetadataExt;

        //TODO: method to reload remote filesystems dynamically
        //TODO: fix for https://github.com/eminence/procfs/issues/262
        static DEVICES: LazyLock<FxHashMap<u64, FsKind>> = LazyLock::new(|| {
            let mut devices = FxHashMap::default();
            match procfs::process::Process::myself() {
                Ok(process) => match process.mountinfo() {
                    Ok(mount_infos) => {
                        devices =
                            FxHashMap::from_iter(mount_infos.iter().filter_map(|mount_info| {
                                let mut parts = mount_info.majmin.split(':');
                                let major_str = parts.next()?;
                                let minor_str = parts.next()?;
                                let major = major_str.parse::<libc::c_uint>().ok()?;
                                let minor = minor_str.parse::<libc::c_uint>().ok()?;
                                let dev = libc::makedev(major, minor);
                                // Network and distributed filesystem types
                                // Based on common remote filesystem types found in /proc/mounts
                                let kind = match mount_info.fs_type.as_str() {
                                    // SMB/CIFS variants
                                    "cifs" | "smb" | "smb2" | "smbfs" => FsKind::Remote,

                                    // NFS variants
                                    "nfs" | "nfs4" => FsKind::Remote,

                                    // FUSE-based remote filesystems
                                    "fuse.rclone" | "fuse.sshfs" | "fuse.davfs2" | "fuse.ceph"
                                    | "fuse.glusterfs" | "fuse.s3fs" | "fuse.goofys"
                                    | "fuse.gcsfuse" | "fuse.afp" | "fuse.afpfs" => FsKind::Remote,

                                    // Other network protocols
                                    "afs" | "coda" | "ncpfs" | "davfs" | "davfs2" | "shfs" => {
                                        FsKind::Remote
                                    }

                                    // Cluster/distributed filesystems
                                    "ceph" | "glusterfs" | "lustre" | "gfs" | "gfs2" | "ocfs2" => {
                                        FsKind::Remote
                                    }

                                    // GVFS (GNOME Virtual File System)
                                    "fuse.gvfsd-fuse" => FsKind::Gvfs,

                                    // Everything else is local
                                    _ => FsKind::Local,
                                };
                                Some((dev, kind))
                            }));
                    }
                    Err(err) => {
                        log::warn!("failed to get mount info: {err}");
                    }
                },
                Err(err) => {
                    log::warn!("failed to get process info: {err}");
                }
            }
            devices
        });
        DEVICES.get(&metadata.dev()).map_or(FsKind::Local, |x| *x)
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new(_metadata: &Metadata) -> FsKind {
        //TODO: support BSD, macOS, Windows?
        FsKind::Local
    }
}
