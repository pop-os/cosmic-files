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

/// Scale factor for HiDPI displays - decode at higher resolution than display size
/// for better quality on high-DPI screens. 1.5x provides good balance between
/// quality and memory usage and also prevets re-decoding on small windows size adjustments.
const DISPLAY_SCALE_FACTOR: f32 = 1.5;

/// Calculate optimal target dimensions for decoding based on display size.
/// Returns None if no resizing is needed (image is smaller than display).
///
/// This helps reduce memory usage by decoding large images at a resolution
/// appropriate for the display, rather than always using full resolution.
pub fn calculate_target_dimensions(
    image_width: u32,
    image_height: u32,
    display_width: u32,
    display_height: u32,
) -> Option<(u32, u32)> {
    let target_width = (display_width as f32 * DISPLAY_SCALE_FACTOR) as u32;
    let target_height = (display_height as f32 * DISPLAY_SCALE_FACTOR) as u32;

    if image_width <= target_width && image_height <= target_height {
        return None;
    }

    let image_aspect = image_width as f32 / image_height as f32;
    let target_aspect = target_width as f32 / target_height as f32;

    let (new_width, new_height) = if image_aspect > target_aspect {
        let w = target_width;
        let h = (target_width as f32 / image_aspect) as u32;
        (w, h)
    } else {
        let h = target_height;
        let w = (target_height as f32 * image_aspect) as u32;
        (w, h)
    };

    let new_width = new_width.max(1);
    let new_height = new_height.max(1);

    log::info!(
        "Calculated target dimensions: {}x{} -> {}x{} (display: {}x{}, scale: {}x)",
        image_width,
        image_height,
        new_width,
        new_height,
        display_width,
        display_height,
        DISPLAY_SCALE_FACTOR
    );

    Some((new_width, new_height))
}

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
pub async fn decode_large_image(
    path: PathBuf,
    target_dimensions: Option<(u32, u32)>,
) -> Option<(PathBuf, u32, u32, Vec<u8>)> {
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
                                let orig_width = rgba.width();
                                let orig_height = rgba.height();

                                // Resize if target dimensions provided
                                let (final_img, width, height) = if let Some((target_w, target_h)) = target_dimensions {
                                    log::info!(
                                        "Resizing {}x{} -> {}x{} for memory optimization: {}",
                                        orig_width, orig_height, target_w, target_h,
                                        path.display()
                                    );

                                    // Use Lanczos3 for high-quality downsampling
                                    let resized = image::imageops::resize(
                                        &rgba,
                                        target_w,
                                        target_h,
                                        image::imageops::FilterType::Lanczos3,
                                    );

                                    let resized_w = resized.width();
                                    let resized_h = resized.height();

                                    log::info!(
                                        "Resize complete: {}x{} image now uses ~{} MB instead of ~{} MB",
                                        resized_w, resized_h,
                                        (resized_w as u64 * resized_h as u64 * 4) / MB_TO_BYTES,
                                        (orig_width as u64 * orig_height as u64 * 4) / MB_TO_BYTES
                                    );

                                    (resized, resized_w, resized_h)
                                } else {
                                    log::info!(
                                        "Decoded {}x{} image at full resolution: {}",
                                        orig_width, orig_height,
                                        path.display()
                                    );
                                    (rgba, orig_width, orig_height)
                                };

                                let pixels = final_img.into_raw();
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
    /// Display dimensions used for each decoded image (for resize detection)
    decoded_display_sizes: HashMap<PathBuf, (u32, u32)>,
    /// Errors encountered during decoding
    decode_errors: HashMap<PathBuf, String>,
    /// Generation counter for each decode to support cancellation.
    /// When a new decode is started for the same path, the generation is incremented.
    /// Only decodes matching the current generation are accepted when they complete.
    decode_generations: HashMap<PathBuf, u64>,
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

    /// Store a decoded image if the generation matches (not superseded by newer decode).
    /// Returns true if stored, false if rejected due to generation mismatch.
    pub fn store_decoded_with_generation(
        &mut self,
        path: PathBuf,
        handle: widget::image::Handle,
        display_size: Option<(u32, u32)>,
        generation: u64,
    ) -> bool {
        // Check if this decode is still current (not superseded by a newer one)
        if let Some(&current_gen) = self.decode_generations.get(&path) {
            if generation != current_gen {
                log::info!(
                    "Discarding outdated decode for {} (generation {} != current {})",
                    path.display(),
                    generation,
                    current_gen
                );
                return false;
            }
        }

        log::info!(
            "Storing decoded image for {} (generation {})",
            path.display(),
            generation
        );

        self.decoded_images.insert(path.clone(), handle);
        if let Some(size) = display_size {
            self.decoded_display_sizes.insert(path.clone(), size);
        }
        self.decoding_images.remove(&path);
        true
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

    /// Check if an image should be re-decoded due to display size increase.
    /// Returns true only if the display size has INCREASED by more than 20% in either dimension.
    /// Does NOT re-decode for smaller sizes (GPU can efficiently downscale).
    pub fn needs_redecode_for_size(
        &self,
        path: &Path,
        new_display_size: Option<(u32, u32)>,
    ) -> bool {
        let Some(new_size) = new_display_size else {
            return false;
        };

        let Some(&old_size) = self.decoded_display_sizes.get(path) else {
            return false;
        };

        const REDECODE_THRESHOLD: f32 = 0.2;

        let width_increase = (new_size.0 as f32 / old_size.0 as f32) - 1.0;
        let height_increase = (new_size.1 as f32 / old_size.1 as f32) - 1.0;

        let needs_redecode =
            width_increase > REDECODE_THRESHOLD || height_increase > REDECODE_THRESHOLD;

        if needs_redecode {
            log::info!(
                "Display size increased significantly for {}: {}x{} -> {}x{} (increase: {:.1}% width, {:.1}% height) - re-decoding at higher resolution",
                path.display(),
                old_size.0,
                old_size.1,
                new_size.0,
                new_size.1,
                width_increase * 100.0,
                height_increase * 100.0
            );
        } else if width_increase < -REDECODE_THRESHOLD || height_increase < -REDECODE_THRESHOLD {
            log::debug!(
                "Display size decreased for {}: {}x{} -> {}x{} (decrease: {:.1}% width, {:.1}% height) - keeping existing higher resolution",
                path.display(),
                old_size.0,
                old_size.1,
                new_size.0,
                new_size.1,
                width_increase * 100.0,
                height_increase * 100.0
            );
        }

        needs_redecode
    }

    /// Attempt to decode a large image, checking memory availability first.
    /// Returns (should_decode, target_dimensions, generation) tuple.
    pub fn try_decode(
        &mut self,
        path: &PathBuf,
        display_dimensions: Option<(u32, u32)>,
    ) -> (bool, Option<(u32, u32)>, u64) {
        self.clear_error(path);
        let is_currently_decoding = self.is_decoding(path);
        let needs_redecode = self.needs_redecode_for_size(path, display_dimensions);

        if is_currently_decoding && !needs_redecode {
            // Get current generation for the ongoing decode
            let generation = self.decode_generations.get(path).copied().unwrap_or(0);
            return (false, None, generation);
        }

        if self.get_decoded(path).is_some() && !needs_redecode && !is_currently_decoding {
            let generation = self.decode_generations.get(path).copied().unwrap_or(0);
            return (false, None, generation);
        }

        let Some((width, height)) = get_image_dimensions(path) else {
            self.store_error(path.clone(), "Failed to read image dimensions".to_string());
            return (false, None, 0);
        };

        let target_dimensions = if let Some((display_w, display_h)) = display_dimensions {
            calculate_target_dimensions(width, height, display_w, display_h)
        } else {
            None
        };

        // Check memory for target size (if resizing) or full size
        let (check_w, check_h) = target_dimensions.unwrap_or((width, height));
        if !self.ensure_memory_available(path, check_w, check_h) {
            return (false, None, 0);
        }

        // Increment generation counter (cancels any previous decode)
        let generation = self
            .decode_generations
            .entry(path.clone())
            .and_modify(|g| *g += 1)
            .or_insert(1);
        let generation = *generation;

        if is_currently_decoding {
            log::info!(
                "Cancelling previous decode for {} and starting new one (generation {})",
                path.display(),
                generation
            );
        }

        // Mark as decoding
        self.decoding_images.insert(path.clone());
        (true, target_dimensions, generation)
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
