use cosmic::widget;
use image::{DynamicImage, ImageReader};
use mime_guess::mime;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use tempfile::NamedTempFile;

use crate::format_size;
use crate::item::ItemMetadata;
use crate::large_image::{exceeds_memory_limit, should_use_tiling};
use crate::thumbnail_cacher::{CachedThumbnail, ThumbnailCacher, ThumbnailSize};
use crate::thumbnailer::thumbnailer;

/// Maximum bytes of text to pass to the editor for preview; caps shaping work to avoid blocking.
/// Files larger than this get a truncated preview (first N bytes only).
const TEXT_PREVIEW_MAX_BYTES: usize = 256 * 1024; // 256 KiB
/// Maximum file size (bytes) to attempt text preview; files larger than this are skipped entirely.
const TEXT_PREVIEW_MAX_FILE_BYTES: u64 = 8 * 1000 * 1000; // 8 MiB

#[derive(Debug)]
pub enum ItemThumbnail {
    NotImage,
    Image(widget::image::Handle, Option<(u32, u32)>),
    Svg(widget::svg::Handle),
    Text(widget::text_editor::Content),
}

impl Clone for ItemThumbnail {
    fn clone(&self) -> Self {
        match self {
            Self::NotImage => Self::NotImage,
            Self::Image(handle, size_opt) => Self::Image(handle.clone(), *size_opt),
            Self::Svg(handle) => Self::Svg(handle.clone()),
            // Content cannot be cloned simply
            Self::Text(content) => {
                Self::Text(widget::text_editor::Content::with_text(&content.text()))
            }
        }
    }
}

impl ItemThumbnail {
    pub fn new(
        path: &Path,
        metadata: ItemMetadata,
        mime: mime::Mime,
        mut thumbnail_size: u32,
        max_mem: u64,
        jobs: usize,
        max_size_mb: u64,
    ) -> Self {
        let thumbnail_cacher =
            ThumbnailCacher::new(path, ThumbnailSize::from_pixel_size(thumbnail_size));
        match thumbnail_cacher.as_ref() {
            Ok(cache) => match cache.get_cached_thumbnail() {
                CachedThumbnail::Valid((thumbnail_path, size)) => {
                    // Check original image dimensions even when loading cached thumbnail
                    // This prevents trying to load huge images in preview mode
                    let original_dims = match image::image_dimensions(path) {
                        Ok((width, height)) => Some((width, height)),
                        Err(_) => size.map(|s| (s.pixel_size(), s.pixel_size())),
                    };

                    return Self::Image(
                        widget::image::Handle::from_path(thumbnail_path),
                        original_dims,
                    );
                }
                CachedThumbnail::Failed => {
                    if mime.type_() != mime::IMAGE {
                        return Self::NotImage;
                    }
                }
                CachedThumbnail::RequiresUpdate(size) => {
                    thumbnail_size = size.pixel_size();
                }
            },
            Err(err) => {
                log::warn!(
                    "failed to create ThumbnailCache for {}: {}",
                    path.display(),
                    err
                );
            }
        }

        let size = metadata.file_size().unwrap_or_default();
        let check_size = |thumbnailer: &str, max_size| {
            if size <= max_size {
                true
            } else {
                log::warn!(
                    "skipping internal {} thumbnailer for {}: file size {} is larger than {}",
                    thumbnailer,
                    path.display(),
                    format_size(size),
                    format_size(max_size)
                );
                false
            }
        };

        let mut tried_supported_file = false;
        // First try built-in image thumbnailer
        if mime.type_() == mime::IMAGE && check_size("image", max_size_mb * 1000 * 1000) {
            // Check if image dimensions would exceed available memory budget
            // The GPU tiling system can handle large images, but we still need to decode them first
            let dimensions_ok = match image::image_dimensions(path) {
                Ok((width, height)) => {
                    if exceeds_memory_limit(width, height, max_mem) {
                        log::warn!(
                            "skipping thumbnail generation for {}: {}x{} image would exceed {}MB memory budget",
                            path.display(),
                            width,
                            height,
                            max_mem
                        );
                        false
                    } else {
                        if should_use_tiling(width, height) {
                            log::info!(
                                "Large image {}x{} detected, will use GPU tiling for display",
                                width,
                                height
                            );
                        }
                        true
                    }
                }
                Err(err) => {
                    log::debug!(
                        "failed to read dimensions for {}: {}, will try decoding",
                        path.display(),
                        err
                    );
                    true // If we can't read dimensions, try anyway
                }
            };

            if !dimensions_ok {
                // Skip this image entirely since it is too large to safely decode
                return Self::NotImage;
            }

            tried_supported_file = true;
            let dyn_img = match image::ImageReader::open(path)
                .and_then(image::ImageReader::with_guessed_format)
            {
                Ok(mut reader) => {
                    let mut limits = image::Limits::default();
                    let max_ram = max_mem * 1000 * 1000 / jobs as u64;
                    limits.max_alloc = Some(max_ram);
                    reader.limits(limits);
                    match reader.decode() {
                        Ok(reader) => Some(reader),
                        Err(err) => {
                            log::warn!("failed to decode {}: {}", path.display(), err);
                            None
                        }
                    }
                }
                Err(err) => {
                    log::warn!("failed to read {}: {}", path.display(), err);
                    None
                }
            };

            if let Some(dyn_img) = dyn_img {
                let (img_width, img_height) = (dyn_img.width(), dyn_img.height());

                if let Ok(cacher) = thumbnail_cacher.as_ref() {
                    match cacher.update_with_image(dyn_img) {
                        Ok(thumb_path) => {
                            return Self::Image(
                                widget::image::Handle::from_path(thumb_path),
                                Some((img_width, img_height)),
                            );
                        }
                        Err(err) => {
                            log::warn!("cacher failed to decode {}: {}", path.display(), err);
                        }
                    }
                } else {
                    // Fallback for when thumbnail cacher isn't available.
                    let thumbnail = dyn_img
                        .thumbnail(thumbnail_size, thumbnail_size)
                        .into_rgba8();
                    return Self::Image(
                        widget::image::Handle::from_rgba(
                            thumbnail.width(),
                            thumbnail.height(),
                            thumbnail.into_raw(),
                        ),
                        Some((img_width, img_height)),
                    );
                }
            }
        }

        // Try external thumbnailers.
        let thumbnail_dir = thumbnail_cacher
            .as_ref()
            .ok()
            .map(ThumbnailCacher::thumbnail_dir);
        if let Some((item_thumbnail, temp_file)) =
            Self::generate_thumbnail_external(path, &mime, thumbnail_size, thumbnail_dir)
        {
            if let Ok(cache) = thumbnail_cacher
                && let Err(err) = cache.update_with_temp_file(temp_file)
            {
                log::warn!("failed to update cache for {}: {}", path.display(), err);
            }
            return item_thumbnail;
        }

        tried_supported_file = tried_supported_file || !thumbnailer(&mime).is_empty();

        // Try internal thumbnailers that don't get cached.
        //TODO: adjust limits for internal thumbnailers as desired
        if mime.type_() == mime::IMAGE
            && mime.subtype() == mime::SVG
            && check_size("svg", 8 * 1000 * 1000)
        {
            tried_supported_file = true;
            // Try built-in svg thumbnailer
            match fs::read(path) {
                Ok(data) => {
                    //TODO: validate SVG data
                    return Self::Svg(widget::svg::Handle::from_memory(data));
                }
                Err(err) => {
                    log::warn!("failed to read {}: {}", path.display(), err);
                }
            }
        } else if mime.type_() == mime::TEXT && check_size("text", TEXT_PREVIEW_MAX_FILE_BYTES) {
            tried_supported_file = true;
            if size > 0 {
                // Reuse size from metadata above; cap allocation and read
                let read_cap = (size.min(TEXT_PREVIEW_MAX_BYTES as u64)) as usize;
                let mut buf = vec![0u8; read_cap];
                match File::open(path).and_then(|f| {
                    let n = Read::read(&mut f.take(read_cap as u64), &mut buf)?;
                    buf.truncate(n);
                    Ok(())
                }) {
                    Ok(()) => {
                        let text = match std::str::from_utf8(&buf) {
                            Ok(s) => s.to_string(),
                            Err(e) => {
                                // Use only the valid UTF-8 prefix (slice is guaranteed valid by valid_up_to())
                                std::str::from_utf8(&buf[..e.valid_up_to()])
                                    .unwrap_or("")
                                    .to_string()
                            }
                        };
                        if !text.is_empty() {
                            return Self::Text(widget::text_editor::Content::with_text(&text));
                        }
                    }
                    Err(err) => {
                        log::warn!("failed to read {}: {}", path.display(), err);
                    }
                }
            }
            // size == 0: empty file or unknown size; skip read and allocation
        }

        // If we weren't able to create a thumbnail, but we should have
        // been able to, create a fail marker so that it isn't tried the
        // next time.
        if let Ok(cacher) = thumbnail_cacher
            && tried_supported_file
            && let Err(err) = cacher.create_fail_marker()
        {
            log::warn!(
                "failed to create thumbnail fail marker for {}: {}",
                path.display(),
                err
            );
        }

        Self::NotImage
    }

    fn generate_thumbnail_external(
        path: &Path,
        mime: &mime::Mime,
        thumbnail_size: u32,
        thumbnail_dir: Option<&Path>,
    ) -> Option<(Self, NamedTempFile)> {
        // Try external thumbnailers
        for thumbnailer in thumbnailer(mime) {
            let is_evince = thumbnailer.exec.starts_with("evince-thumbnailer ");
            let prefix = if is_evince {
                //TODO: apparmor config for evince-thumbnailer does not allow /tmp/cosmic-files*
                "gnome-desktop-"
            } else {
                "cosmic-files-"
            };

            // It's preferable to create the tempfile in the same directory as the final cached
            // thumbnail to ensure that no copies across filesytems need to be made. However,
            // the apparmor config for evince-thumbnailer does not allow this, so we need to
            // fallback to the system tempdir.
            let dir = if is_evince { None } else { thumbnail_dir };
            let file = match dir {
                Some(d) => tempfile::Builder::new().prefix(prefix).tempfile_in(d),
                None => tempfile::Builder::new().prefix(prefix).tempfile(),
            };
            let file = match file {
                Ok(ok) => ok,
                Err(err) => {
                    log::warn!(
                        "failed to create temporary file for thumbnail of {}: {}",
                        path.display(),
                        err
                    );
                    continue;
                }
            };

            let Some(mut command) = thumbnailer.command(path, file.path(), thumbnail_size) else {
                continue;
            };
            match command.status() {
                Ok(status) => {
                    if status.success() {
                        match image::ImageReader::open(file.path())
                            .and_then(ImageReader::with_guessed_format)
                        {
                            Ok(reader) => match reader.decode().map(DynamicImage::into_rgba8) {
                                Ok(image) => {
                                    return Some((
                                        Self::Image(
                                            widget::image::Handle::from_rgba(
                                                image.width(),
                                                image.height(),
                                                image.into_raw(),
                                            ),
                                            None,
                                        ),
                                        file,
                                    ));
                                }
                                Err(err) => {
                                    log::warn!("failed to decode {}: {}", path.display(), err);
                                }
                            },
                            Err(err) => {
                                log::warn!("failed to read {}: {}", path.display(), err);
                            }
                        }
                    } else {
                        log::warn!(
                            "failed to run {:?} for {}: {}",
                            thumbnailer,
                            path.display(),
                            status
                        );
                    }
                }
                Err(err) => {
                    log::warn!(
                        "failed to run {thumbnailer:?} for {}: {}",
                        path.display(),
                        err
                    );
                }
            }
        }

        None
    }
}
