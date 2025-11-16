use cosmic::widget;
use image::ImageReader;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

/// Bytes per pixel in RGBA format (Red, Green, Blue, Alpha = 4 bytes)
pub const RGBA_BYTES_PER_PIXEL: u64 = 4;

/// Overhead factor for image decoding operations (30% additional memory for decode buffers,
/// fragment allocations, and intermediate representations during image decoding)
const DECODE_OVERHEAD_FACTOR: f64 = 1.3;

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

/// Conservative GPU buffer size limit in MB. Each atlas fragment can be up to this size.
/// Based on wgpu device limits - most GPUs support at least 256MB buffers.
/// Reference: https://docs.rs/wgpu/latest/wgpu/struct.Limits.html#structfield.max_buffer_size
const MAX_GPU_BUFFER_MB: u64 = 256;

/// Conversion factor: 1 MB = 1024 * 1024 bytes (binary megabyte, used for RAM calculations)
pub const MB_TO_BYTES: u64 = 1024 * 1024;

/// Conversion factor: 1 MB = 1000 * 1000 bytes (decimal megabyte, used by image crate)
/// The image crate's memory limits use decimal MB, not binary MB.
pub const DECIMAL_MB_TO_BYTES: u64 = 1000 * 1000;

/// Maximum dimension for image decoding
pub const MAX_DIMENSION_FOR_DECODE: u32 = 65536;

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

/// Check if there's sufficient memory to decode an image.
///
/// This function performs two types of checks:
/// 1. System RAM availability (Linux only via procfs)
/// 2. GPU buffer limits (all platforms)
///
/// Platform-specific behavior:
/// - Linux: Full RAM checking via /proc/meminfo + GPU checks
/// - Windows/macOS: GPU buffer checks only (RAM checking not yet implemented)
///
/// Returns: (has_memory, error_message)
pub fn check_memory_available(width: u32, height: u32) -> (bool, Option<String>) {
    if width == 0 || height == 0 {
        let error_msg = format!(
            "Invalid image dimensions: {}x{} (zero dimension)",
            width, height
        );
        log::error!("{}", error_msg);
        return (false, Some(error_msg));
    }

    let pixels = match (width as u64).checked_mul(height as u64) {
        Some(p) => p,
        None => {
            let error_msg = format!(
                "Image dimensions too large: {}x{} causes overflow in pixel calculation",
                width, height
            );
            log::error!("{}", error_msg);
            return (false, Some(error_msg));
        }
    };

    let bytes_needed = match pixels.checked_mul(RGBA_BYTES_PER_PIXEL) {
        Some(b) => b,
        None => {
            let error_msg = format!(
                "Image memory requirements overflow: {}x{} pixels requires more than {} bytes",
                width,
                height,
                u64::MAX
            );
            log::error!("{}", error_msg);
            return (false, Some(error_msg));
        }
    };

    // Add overhead for decode buffers, fragment allocations, and intermediate representations
    let bytes_with_overhead = (bytes_needed as f64 * DECODE_OVERHEAD_FACTOR) as u64;
    let mb_needed = bytes_with_overhead / MB_TO_BYTES;

    // Check system RAM availability (Linux only)
    #[cfg(target_os = "linux")]
    {
        use procfs::Current;
        match procfs::Meminfo::current() {
            Ok(meminfo) => {
                // MemAvailable includes reclaimable cache and is the best estimate of
                // actually available memory for new allocations
                let available_kb = meminfo.mem_available.unwrap_or(0);
                let available_bytes = available_kb * 1024;

                // Maintain system reserve to prevent thrashing and OOM killer
                let min_reserve_bytes = SYSTEM_MEMORY_RESERVE_MB * MB_TO_BYTES;
                let usable_bytes = available_bytes.saturating_sub(min_reserve_bytes);

                if bytes_with_overhead > usable_bytes {
                    let available_mb = available_bytes / MB_TO_BYTES;
                    let error_msg = format!(
                        "Insufficient memory: need {}MB, available {}MB. Try closing other applications.",
                        mb_needed, available_mb
                    );
                    log::warn!("{}", error_msg);
                    return (false, Some(error_msg));
                }
            }
            Err(e) => {
                log::warn!("Failed to read /proc/meminfo: {}. Skipping RAM check.", e);
                // Graceful fallback: continue to GPU checks
            }
        }
    }

    // Note: RAM checking not implemented for Windows/macOS
    // These platforms will only validate against GPU buffer limits below
    #[cfg(not(target_os = "linux"))]
    {
        log::debug!(
            "RAM checking not available on this platform. Only GPU limits will be enforced."
        );
    }

    // Check GPU fragment/atlas tile limits
    // Large images are split into atlas fragments for GPU upload.
    // Each fragment must fit within GPU buffer size limits.
    let fragment_bytes =
        (ATLAS_FRAGMENT_SIZE as u64) * (ATLAS_FRAGMENT_SIZE as u64) * RGBA_BYTES_PER_PIXEL;
    let max_gpu_buffer_bytes = MAX_GPU_BUFFER_MB * MB_TO_BYTES;

    let fragments_x = (width + ATLAS_FRAGMENT_SIZE - 1) / ATLAS_FRAGMENT_SIZE;
    let fragments_y = (height + ATLAS_FRAGMENT_SIZE - 1) / ATLAS_FRAGMENT_SIZE;
    let fragment_count = fragments_x as u64 * fragments_y as u64;

    // Fragments are uploaded sequentially, so we only need one fragment buffer at a time.
    // However, each individual fragment must fit within GPU buffer size limits.
    if fragment_bytes > max_gpu_buffer_bytes {
        let max_dimension = (MAX_GPU_BUFFER_MB * MB_TO_BYTES / RGBA_BYTES_PER_PIXEL) as f64;
        let max_dimension = (max_dimension.sqrt() as u32).saturating_sub(100); // Add safety margin

        let error_msg = format!(
            "Image too large for GPU: {}x{} pixels exceeds GPU buffer limits. \
             Maximum supported dimension is approximately {}x{} pixels.",
            width, height, max_dimension, max_dimension
        );
        log::error!("{}", error_msg);
        return (false, Some(error_msg));
    }

    log::debug!(
        "Memory check passed: {}x{} image needs {}MB RAM, will use {} GPU fragment(s) of {}MB each",
        width,
        height,
        mb_needed,
        fragment_count,
        fragment_bytes / MB_TO_BYTES
    );

    (true, None)
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

    pub fn mark_decoding(&mut self, path: PathBuf) {
        self.decoding_images.insert(path);
    }

    pub fn store_decoded(&mut self, path: PathBuf, handle: widget::image::Handle) {
        self.decoded_images.insert(path.clone(), handle);
        self.decoding_images.remove(&path);
    }

    pub fn store_error(&mut self, path: PathBuf, error: String) {
        self.decode_errors.insert(path, error);
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
}
