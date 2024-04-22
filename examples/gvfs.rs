use gio::prelude::*;

fn main() {
    let monitor = gio::VolumeMonitor::get();
    for drive in monitor.connected_drives() {
        println!("Drive: {}", drive.name());
        for volume in drive.volumes() {
            println!("  Volume: {}", volume.name());
            if let Some(mount) = volume.get_mount() {
                println!("    Mount: {}", mount.name());
            }
        }
    }

    for mount in monitor.mounts() {
        println!("Mount: {}", mount.name());
    }

    for volume in monitor.volumes() {
        println!("Volume: {}", volume.name());
        if let Some(mount) = volume.get_mount() {
            println!("  Mount: {}", mount.name());
        }
    }
}
