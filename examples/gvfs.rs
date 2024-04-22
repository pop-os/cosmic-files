use gio::prelude::*;

fn main() {
    let monitor = gio::VolumeMonitor::get();
    for drive in monitor.connected_drives() {
        println!("Drive: {}", drive.name());
        for id in drive.enumerate_identifiers() {
            println!("  ID: {}={:?}", id, drive.identifier(&id));
        }
        for volume in drive.volumes() {
            println!("  Volume: {}", volume.name());
            println!("    UUID: {:?}", volume.uuid());
            for id in volume.enumerate_identifiers() {
                println!("    ID: {}={:?}", id, volume.identifier(&id));
            }
            if let Some(mount) = volume.get_mount() {
                println!("    Mount: {}", mount.name());
                println!("      UUID: {:?}", mount.uuid());
            }
        }
    }

    for mount in monitor.mounts() {
        println!("Mount: {}", mount.name());
        println!("  UUID: {:?}", mount.uuid());
    }

    for volume in monitor.volumes() {
        println!("Volume: {}", volume.name());
        println!("  UUID: {:?}", volume.uuid());
        for id in volume.enumerate_identifiers() {
            println!("  ID: {}={:?}", id, volume.identifier(&id));
        }
        if let Some(mount) = volume.get_mount() {
            println!("  Mount: {}", mount.name());
            println!("    UUID: {:?}", mount.uuid());
        }
    }
}
