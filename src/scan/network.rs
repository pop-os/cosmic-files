use crate::config::IconSizes;
use crate::item::Item;
use crate::mounter::MOUNTERS;

pub fn scan_network(uri: &str, sizes: IconSizes) -> Vec<Item> {
    for mounter in MOUNTERS.values() {
        match mounter.network_scan(uri, sizes) {
            Some(Ok(items)) => return items,
            Some(Err(err)) => {
                log::warn!("failed to scan {uri:?}: {err}");
            }
            None => {}
        }
    }
    Vec::new()
}
