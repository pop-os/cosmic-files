use cosmic::widget;
use image::ImageReader;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

/// Bytes per pixel in RGBA format (Red, Green, Blue, Alpha = 4 bytes)
pub const RGBA_BYTES_PER_PIXEL: u64 = 4;

/// System memory reserve in MB to maintain for system stability (prevents thrashing)
/// Note: RAM checking is currently only available on Linux via procfs.
/// On Windows and macOS, only GPU buffer limits are enforced.
const SYSTEM_MEMORY_RESERVE_MB: u64 = 500;

/// Maximum memory allocation for gallery image decoding in MB.
/// Gallery mode uses the full memory budget since only one image decodes at a time.
/// This matches the ThumbCfg max_mem_mb budget for consistency.
const GALLERY_MEMORY_LIMIT_MB: u64 = 2000;

/// Threshold for considering an image "large" requiring GPU tiling
/// Atlas fragment/tile size in pixels. Large images are split into fragments of this size.
/// Must match the atlas SIZE constant in libcosmic/iced/wgpu/src/image/atlas.rs
pub const ATLAS_FRAGMENT_SIZE: u32 = 4096;

/// Conversion factor: 1 MB = 1024 * 1024 bytes (binary megabyte, used for RAM calculations)
pub const MB_TO_BYTES: u64 = 1024 * 1024;

/// Conversion factor: 1 MB = 1000 * 1000 bytes (decimal megabyte, used by image crate)
/// The image crate's memory limits use decimal MB, not binary MB.
pub const DECIMAL_MB_TO_BYTES: u64 = 1000 * 1000;

/// Check if an image's dimensions would exceed the available memory budget.
/// Returns true if the image is too large to decode.
pub fn exceeds_memory_limit(width: u32, height: u32, memory_limit_mb: u64) -> bool {
    let Some(bytes_needed) = calculate_image_memory(width, height) else {
        // Overflow in calculation means it definitely exceeds any reasonable limit
        return true;
    };

    let max_bytes = memory_limit_mb * MB_TO_BYTES;
    bytes_needed > max_bytes
}

/// Check if an image should use GPU tiling for display.
/// Images larger than the atlas fragment size need to be split into tiles for GPU upload.
pub fn should_use_tiling(width: u32, height: u32) -> bool {
    width > ATLAS_FRAGMENT_SIZE || height > ATLAS_FRAGMENT_SIZE
}

/// Determine if an image should use the dedicated worker for thumbnail generation.
/// Returns (use_dedicated_worker, effective_max_mb, effective_jobs).
///
/// Large images that exceed per-worker memory budget get routed to a dedicated worker
/// with full memory budget. Smaller images use the normal parallel worker pool.
pub fn should_use_dedicated_worker(
    width: u32,
    height: u32,
    total_budget_mb: u64,
    parallel_workers: usize,
) -> (bool, u64, usize) {
    if width == 0 || height == 0 {
        log::warn!(
            "Invalid image dimensions {}x{}, using normal queue",
            width,
            height
        );
        return (false, total_budget_mb, parallel_workers);
    }

    let Some(bytes_needed) = calculate_image_memory(width, height) else {
        log::warn!(
            "Image dimensions {}x{} overflow memory calculation, using normal queue",
            width,
            height
        );
        return (false, total_budget_mb, parallel_workers);
    };

    let mb_needed = bytes_needed / MB_TO_BYTES;
    let per_worker_budget_mb = total_budget_mb / parallel_workers as u64;

    if mb_needed > per_worker_budget_mb {
        log::info!(
            "Large image {}x{} needs {}MB (exceeds per-worker {}MB), using dedicated worker",
            width,
            height,
            mb_needed,
            per_worker_budget_mb
        );
        // Use dedicated worker with full budget
        (true, total_budget_mb, 1)
    } else {
        log::debug!(
            "Normal image {}x{} needs {}MB (within per-worker {}MB), using parallel workers",
            width,
            height,
            mb_needed,
            per_worker_budget_mb
        );
        // Use parallel worker pool with shared budget
        (false, total_budget_mb, parallel_workers)
    }
}

/// Get the dimensions of an image without fully decoding it
pub fn get_image_dimensions(path: &Path) -> Option<(u32, u32)> {
    match ImageReader::open(path) {
        Ok(reader) => match reader.into_dimensions() {
            Ok((width, height)) => {
                log::debug!(
                    "Image dimensions: {}x{} for {}",
                    width,
                    height,
                    path.display()
                );
                Some((width, height))
            }
            Err(e) => {
                log::warn!("Failed to get dimensions for {}: {}", path.display(), e);
                None
            }
        },
        Err(e) => {
            log::warn!("Failed to open image reader for {}: {}", path.display(), e);
            None
        }
    }
}

/// Calculate the memory required to decode an image in bytes.
/// Returns None if the calculation overflows.
fn calculate_image_memory(width: u32, height: u32) -> Option<u64> {
    let pixels = (width as u64).checked_mul(height as u64)?;
    pixels.checked_mul(RGBA_BYTES_PER_PIXEL)
}

/// Check if there's sufficient system RAM to decode an image (Linux only).
/// Returns: (has_memory, error_message)
#[cfg(target_os = "linux")]
fn check_ram_available(width: u32, height: u32) -> (bool, Option<String>) {
    use procfs::Current;

    let Some(bytes_needed) = calculate_image_memory(width, height) else {
        let error_msg = format!(
            "Image dimensions too large: {}x{} causes overflow in memory calculation",
            width, height
        );
        log::error!("{}", error_msg);
        return (false, Some(error_msg));
    };

    let mb_needed = bytes_needed / MB_TO_BYTES;

    match procfs::Meminfo::current() {
        Ok(meminfo) => {
            // MemAvailable includes reclaimable cache and is the best estimate of
            // actually available memory for new allocations
            let available_kb = meminfo.mem_available.unwrap_or(0);
            let available_bytes = available_kb * 1024;

            // Maintain system reserve to prevent thrashing and OOM killer
            let min_reserve_bytes = SYSTEM_MEMORY_RESERVE_MB * MB_TO_BYTES;
            let usable_bytes = available_bytes.saturating_sub(min_reserve_bytes);

            if bytes_needed > usable_bytes {
                let available_mb = available_bytes / MB_TO_BYTES;
                let error_msg = format!(
                    "Insufficient memory: need {}MB, available {}MB. Try closing other applications.",
                    mb_needed, available_mb
                );
                log::warn!("{}", error_msg);
                return (false, Some(error_msg));
            }

            (true, None)
        }
        Err(e) => {
            log::warn!("Failed to read /proc/meminfo: {}. Skipping RAM check.", e);
            // Graceful fallback: assume RAM is available
            (true, None)
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn check_ram_available(_width: u32, _height: u32) -> (bool, Option<String>) {
    // RAM checking not implemented for this platform
    (true, None)
}

pub fn check_memory_available(width: u32, height: u32) -> (bool, Option<String>) {
    if width == 0 || height == 0 {
        let error_msg = format!(
            "Invalid image dimensions: {}x{} (zero dimension)",
            width, height
        );
        log::error!("{}", error_msg);
        return (false, Some(error_msg));
    }

    // Check system RAM availability
    check_ram_available(width, height)
}

/// Decode a large image asynchronously in a blocking thread pool.
///
/// This function is used for gallery mode where full-resolution images need to be loaded.
/// It uses the full memory budget (GALLERY_MEMORY_LIMIT_MB) since only one image
/// decodes at a time in gallery mode.
pub async fn decode_large_image(path: PathBuf) -> Option<(PathBuf, u32, u32, Vec<u8>)> {
    // Decode image in blocking thread pool (CPU-intensive work should not block)
    tokio::task::spawn_blocking(move || {
        log::info!("Starting async decode of {}", path.display());

        // Use ImageReader with explicit memory limits to avoid "Memory limit exceeded" errors
        // Gallery mode uses the full memory budget since only one image decodes at a time
        match image::ImageReader::open(&path) {
            Ok(reader) => {
                match reader.with_guessed_format() {
                    Ok(mut reader) => {
                        // Note: image crate uses decimal MB (1000^2), not binary MB (1024^2)
                        let mut limits = image::Limits::default();
                        limits.max_alloc = Some(GALLERY_MEMORY_LIMIT_MB * DECIMAL_MB_TO_BYTES);
                        reader.limits(limits);

                        match reader.decode() {
                            Ok(img) => {
                                let rgba = img.into_rgba8();
                                let width = rgba.width();
                                let height = rgba.height();
                                let pixels = rgba.into_raw();

                                log::info!(
                                    "Decoded {}x{} image: {}",
                                    width,
                                    height,
                                    path.display()
                                );
                                Some((path, width, height, pixels))
                            }
                            Err(e) => {
                                log::warn!("Failed to decode {}: {}", path.display(), e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to guess format for {}: {}", path.display(), e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to open {}: {}", path.display(), e);
                None
            }
        }
    })
    .await
    .ok()
    .flatten()
}

/// Manages state and operations for large image decoding in gallery mode
#[derive(Debug, Default)]
pub struct LargeImageManager {
    /// Paths of images currently being decoded
    decoding_images: HashSet<PathBuf>,
    /// Cache of decoded image handles
    decoded_images: HashMap<PathBuf, widget::image::Handle>,
    /// Errors encountered during decoding
    decode_errors: HashMap<PathBuf, String>,
}

impl LargeImageManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_decoding(&self, path: &Path) -> bool {
        self.decoding_images.contains(path)
    }

    pub fn get_decoded(&self, path: &Path) -> Option<&widget::image::Handle> {
        self.decoded_images.get(path)
    }

    pub fn get_error(&self, path: &Path) -> Option<&String> {
        self.decode_errors.get(path)
    }

    pub fn store_decoded(&mut self, path: PathBuf, handle: widget::image::Handle) {
        self.decoded_images.insert(path.clone(), handle);
        self.decoding_images.remove(&path);
    }

    pub fn store_error(&mut self, path: PathBuf, error: String) {
        self.decode_errors.insert(path.clone(), error);
        self.decoding_images.remove(&path);
    }

    pub fn clear_error(&mut self, path: &Path) {
        self.decode_errors.remove(path);
    }

    pub fn clear_cache(&mut self) {
        log::info!(
            "Clearing {} cached images from large image manager",
            self.decoded_images.len()
        );
        self.decoded_images.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.decoded_images.len()
    }

    pub fn cache_is_empty(&self) -> bool {
        self.decoded_images.is_empty()
    }

    /// Attempt to decode a large image, checking memory availability first.
    /// Returns true if decode was initiated, false if skipped due to insufficient memory.
    pub fn try_decode(&mut self, path: &PathBuf) -> bool {
        self.clear_error(path);

        // Check if already decoded or decoding
        if self.get_decoded(path).is_some() || self.is_decoding(path) {
            return false;
        }

        let Some((width, height)) = get_image_dimensions(path) else {
            self.store_error(path.clone(), "Failed to read image dimensions".to_string());
            return false;
        };

        if !self.ensure_memory_available(path, width, height) {
            return false;
        }

        // Mark as decoding
        self.decoding_images.insert(path.clone());
        true
    }

    /// Check if sufficient memory is available, clearing cache if needed.
    /// Returns true if memory is available, false otherwise.
    fn ensure_memory_available(&mut self, path: &PathBuf, width: u32, height: u32) -> bool {
        let (has_memory, error_opt) = check_memory_available(width, height);

        if has_memory {
            return true;
        }

        if self.cache_is_empty() {
            if let Some(error_msg) = error_opt {
                self.store_error(path.clone(), error_msg);
                log::warn!(
                    "Cannot load {}: insufficient memory and cache is empty",
                    path.display()
                );
            }
            return false;
        }

        log::info!(
            "Insufficient memory, clearing {} cached images",
            self.cache_size()
        );
        self.clear_cache();

        let (has_memory_after_clear, error_opt_after) = check_memory_available(width, height);

        if has_memory_after_clear {
            log::info!("Memory available after cache clear, proceeding with decode");
            return true;
        }

        if let Some(error_msg) = error_opt_after {
            self.store_error(path.clone(), error_msg);
            log::warn!(
                "Cannot load {}: insufficient memory even after cache clear",
                path.display()
            );
        }
        false
    }
}
