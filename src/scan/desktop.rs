use std::path::PathBuf;

use crate::config::{DesktopConfig, IconSizes};
use crate::fl;
use crate::item::{DirSize, Item, ItemMetadata, ItemState, ItemThumbnail, item_from_path};
use crate::location::Location;
use crate::mounter::MOUNTERS;
use crate::scan::scan_path;
use crate::trash::{Trash, TrashExt};

//TODO: organize desktop items based on display
pub fn scan_desktop(
    tab_path: &PathBuf,
    _display: &str,
    desktop_config: DesktopConfig,
    mut sizes: IconSizes,
) -> Vec<Item> {
    sizes.grid = desktop_config.icon_size;

    let mut items = Vec::new();

    if desktop_config.show_content {
        items.extend(scan_path(tab_path, sizes));
    }

    if desktop_config.show_mounted_drives {
        for mounter in MOUNTERS.values() {
            let Some(mounter_items) = mounter.items(sizes) else {
                continue;
            };
            items.extend(mounter_items.into_iter().filter_map(|mounter_item| {
                let path = mounter_item.path()?;
                // Get most item data from path
                let mut item = match item_from_path(&path, sizes) {
                    Ok(item) => item,
                    Err(err) => {
                        log::warn!(
                            "failed to get item from mounter item {}: {}",
                            path.display(),
                            err
                        );
                        return None;
                    }
                };

                //Override some data with mounter information
                item.name = mounter_item.name();
                item.display_name = Item::display_name(&item.name);

                //TODO: use icon size for mounter item icon
                if let Some(icon) = mounter_item.icon(false) {
                    item.icon_handle_grid.clone_from(&icon);
                    item.icon_handle_list.clone_from(&icon);
                    item.icon_handle_list_condensed = icon;
                }

                Some(item)
            }));
        }
    }

    if desktop_config.show_trash {
        let name = fl!("trash");
        let display_name = Item::display_name(&name);

        let metadata = ItemMetadata::SimpleDir {
            entries: Trash::entries() as u64,
        };

        let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = {
            (
                "inode/directory".parse().unwrap(),
                Trash::icon(sizes.grid()),
                Trash::icon(sizes.list()),
                Trash::icon(sizes.list_condensed()),
            )
        };

        items.push(Item {
            name,
            display_name,
            is_mount_point: false,
            metadata,
            hidden: false,
            location_opt: Some(Location::Trash),
            image_dimensions: None,
            mime,
            icon_handle_grid,
            icon_handle_list,
            icon_handle_list_condensed,
            thumbnail_opt: Some(ItemThumbnail::NotImage),
            dir_size: DirSize::NotDirectory,
            state: ItemState::default(),
        });
    }

    items
}
