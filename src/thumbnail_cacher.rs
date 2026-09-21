use image::DynamicImage;
use md5::{Digest, Md5};
use rustc_hash::FxHashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::UNIX_EPOCH;
use tempfile::NamedTempFile;
use url::Url;

/// Implements thumbnail caching based on the freedesktop.org Thumbnail Managing Standard.
/// <https://specifications.freedesktop.org/thumbnail-spec/latest>/
pub struct ThumbnailCacher {
    file_path: PathBuf,
    file_uri: String,
    thumbnail_dir: PathBuf,
    thumbnail_path: PathBuf,
    thumbnail_size: ThumbnailSize,
    thumbnail_fail_marker_path: PathBuf,
}

impl ThumbnailCacher {
    pub fn new(file_path: &Path, thumbnail_size: ThumbnailSize) -> Result<Self, String> {
        let file_uri = thumbnail_uri(file_path)
            .map_err(|err| format!("failed to create URI for {}: {}", file_path.display(), err))?;
        let cache_base_dir = THUMBNAIL_CACHE_BASE_DIR
            .as_ref()
            .ok_or("failed to get thumbnail cache directory".to_string())?;
        let thumbnail_relative_path = thumbnail_cache_relative_path(&file_uri, thumbnail_size);
        let thumbnail_filename = thumbnail_cache_filename(&file_uri);
        let thumbnail_dir = cache_base_dir.join(thumbnail_size.subdirectory_name());
        if !thumbnail_dir.is_dir() {
            log::warn!(
                "{} is not a directory, creating one now",
                thumbnail_dir.display()
            );
            let _: () = log::error!(
                "{} failed to create directory, this error can be expected on first run",
                thumbnail_dir.display()
            );
            fs::create_dir_all(&thumbnail_dir).unwrap_or(());
        }
        let thumbnail_path = cache_base_dir.join(&thumbnail_relative_path);
        let thumbnail_fail_marker_path = cache_base_dir
            .join("fail")
            .join(format!("cosmic-files-{}", env!("CARGO_PKG_VERSION")))
            .join(&thumbnail_filename);

        Ok(Self {
            file_path: file_path.to_path_buf(),
            file_uri,
            thumbnail_dir,
            thumbnail_path,
            thumbnail_size,
            thumbnail_fail_marker_path,
        })
    }

    pub fn get_cached_thumbnail(&self) -> CachedThumbnail {
        // If the file is already a thumbnail, just use it so we don't generate
        // cached thumbnails of thumbnails.
        if let (Some(cache_base_dir), Ok(metadata)) = (
            THUMBNAIL_CACHE_BASE_DIR.as_ref(),
            std::fs::metadata(&self.file_path),
        ) && metadata.is_file()
            && self.file_path.starts_with(cache_base_dir)
        {
            return CachedThumbnail::Valid((self.file_path.clone(), None));
        }

        // Use cached thumbnail if it is valid.
        if self.is_thumbnail_valid(&self.thumbnail_path) {
            return CachedThumbnail::Valid((
                self.thumbnail_path.clone(),
                Some(self.thumbnail_size),
            ));
        }

        // Check if there is a fail marker from an earlier failure.
        if self.is_thumbnail_valid(&self.thumbnail_fail_marker_path) {
            return CachedThumbnail::Failed;
        }

        CachedThumbnail::RequiresUpdate(self.thumbnail_size)
    }

    pub fn thumbnail_dir(&self) -> &Path {
        &self.thumbnail_dir
    }

    pub fn update_with_temp_file(&self, temp_file: NamedTempFile) -> Result<&Path, Box<dyn Error>> {
        #[cfg(unix)]
        fs::set_permissions(temp_file.path(), fs::Permissions::from_mode(0o600))?;
        self.update_thumbnail_text_metadata(temp_file.path())?;
        fs::rename(temp_file.path(), &self.thumbnail_path)?;

        Ok(&self.thumbnail_path)
    }

    pub fn update_with_image(&self, image: DynamicImage) -> Result<&Path, Box<dyn Error>> {
        let temp_file = tempfile::Builder::new()
            .prefix("cosmic-files-")
            .tempfile_in(&self.thumbnail_dir)?;
        {
            let file = File::create(temp_file.path())?;
            let image = image
                .thumbnail(
                    self.thumbnail_size.pixel_size(),
                    self.thumbnail_size.pixel_size(),
                )
                .into_rgba8();
            let writer = BufWriter::new(file);
            let mut encoder = png::Encoder::new(writer, image.width(), image.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()?
                .write_image_data(&image.into_raw())?;
        }

        self.update_with_temp_file(temp_file)
    }

    pub fn create_fail_marker(&self) -> Result<(), Box<dyn Error>> {
        if let Some(dir) = self.thumbnail_fail_marker_path.parent() {
            fs::create_dir_all(dir)?;
            #[cfg(unix)]
            fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
        }

        let file = File::create(&self.thumbnail_fail_marker_path)?;
        let writer = BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, 1, 1);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::One);
        encoder.write_header()?.write_image_data(&[0])?;
        self.update_thumbnail_text_metadata(&self.thumbnail_fail_marker_path)
    }

    fn update_thumbnail_text_metadata(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let decoder = png::Decoder::new(reader);
        let mut reader = decoder.read_info()?;
        let (width, height, color_type, bit_depth, mut text_chunks) = {
            let info = reader.info();
            let text_chunks: FxHashMap<String, String> = info
                .uncompressed_latin1_text
                .iter()
                .map(|chunk| (chunk.keyword.clone(), chunk.text.clone()))
                .collect();
            (
                info.width,
                info.height,
                info.color_type,
                info.bit_depth,
                text_chunks,
            )
        };

        let mut image_data = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or("The required image buffer size is too large.")?
        ];
        reader.next_frame(&mut image_data)?;

        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(color_type);
        encoder.set_depth(bit_depth);

        text_chunks.insert("Software".to_string(), "COSMIC Files".to_string());
        text_chunks.insert("Thumb::URI".to_string(), self.file_uri.clone());
        let metadata = std::fs::metadata(&self.file_path)?;
        let size = metadata.len();
        text_chunks.insert("Thumb::Size".to_string(), size.to_string());
        let mtime = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        text_chunks.insert("Thumb::MTime".to_string(), mtime.to_string());

        for (keyword, text) in text_chunks {
            encoder.add_text_chunk(keyword, text)?;
        }

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&image_data)?;

        Ok(())
    }

    fn is_thumbnail_valid(&self, thumbnail_path: &Path) -> bool {
        let thumbnail_file = match File::open(thumbnail_path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        let decoder = png::Decoder::new(BufReader::new(thumbnail_file));
        let reader = match decoder.read_info() {
            Ok(reader) => reader,
            Err(err) => {
                log::warn!(
                    "failed to decode {} as PNG: {}",
                    thumbnail_path.display(),
                    err
                );
                return false;
            }
        };

        let texts = &reader.info().uncompressed_latin1_text;

        // Thumb::URI is required and must match.
        let thumb_uri = texts
            .iter()
            .find(|&text| text.keyword == "Thumb::URI")
            .map(|t| &t.text);
        if let Some(thumb_uri) = thumb_uri {
            if *thumb_uri != self.file_uri {
                return false;
            }
        } else {
            return false;
        }

        let metadata = match std::fs::metadata(&self.file_path) {
            Ok(m) => m,
            Err(err) => {
                log::warn!(
                    "failed to get metatdata of {}: {}",
                    self.file_path.display(),
                    err
                );
                return false;
            }
        };

        // Thumb::MTime is required and must match.
        let thumb_mtime = texts
            .iter()
            .find(|&text| text.keyword == "Thumb::MTime")
            .map(|t| &t.text);
        if let Some(thumb_mtime) = thumb_mtime {
            let modified = match metadata.modified() {
                Ok(m) => m,
                Err(err) => {
                    log::warn!(
                        "failed to get modified from metatdata of {}, {}",
                        self.file_path.display(),
                        err
                    );
                    return false;
                }
            };
            let mtime = modified
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string();
            if *thumb_mtime != mtime {
                return false;
            }
        } else {
            return false;
        }

        // Thumb::Size isn't required, but it should be verified if present.
        let thumb_size = texts
            .iter()
            .find(|&text| text.keyword == "Thumb::Size")
            .map(|t| &t.text);
        if let Some(thumb_size) = thumb_size {
            let size = metadata.len();
            if *thumb_size != size.to_string() {
                return false;
            }
        }

        true
    }
}

fn thumbnail_uri(path: &Path) -> io::Result<String> {
    let absolute_path = fs::canonicalize(path)?;
    let url = Url::from_file_path(&absolute_path).map_err(|()| {
        io::Error::other(format!(
            "failed to create URI for thumbnail_file: {}",
            absolute_path.display()
        ))
    })?;
    // Technically square brackets don't need to be percent encoded,
    // and they aren't by the url crate, but the thumbnailer used by
    // Gnome Files does. In order to share thumbnails and not get duplicates
    // we should do the same.
    let url = url.as_str().replace('[', "%5B").replace(']', "%5D");
    Ok(url)
}

/// The number of pixels a thumbnail must be rendered at to look sharp when `logical_size`
/// logical pixels of it are drawn on a display with `scale_factor` pixels per logical pixel.
///
/// A scale factor that is not a positive, finite number is not usable: macOS reports zero
/// before the window is on a screen (docs/macos-porting-notes.md section 3.2). Fall back to
/// the logical size rather than ask for a thumbnail of no pixels at all.
pub fn thumbnail_pixel_size(logical_size: u32, scale_factor: f32) -> u32 {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return logical_size;
    }

    let scaled = (f64::from(logical_size) * f64::from(scale_factor)).round();
    if scaled >= f64::from(u32::MAX) {
        u32::MAX
    } else {
        scaled as u32
    }
}

/// Where a thumbnail of `file_uri` at `size` is cached, relative to the cache root.
///
/// The size is part of the key, not just the name: the freedesktop layout gives each size its
/// own directory, so a thumbnail rendered for one scale factor never overwrites or is read
/// back in place of one rendered for another.
fn thumbnail_cache_relative_path(file_uri: &str, size: ThumbnailSize) -> PathBuf {
    Path::new(size.subdirectory_name()).join(thumbnail_cache_filename(file_uri))
}

fn thumbnail_cache_filename(file_uri: &str) -> String {
    let hash = Md5::digest(file_uri);
    format!("{hash:x}.png")
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ThumbnailSize {
    Normal = 128,
    Large = 256,
    XLarge = 512,
    XXLarge = 1024,
}

impl ThumbnailSize {
    pub fn from_pixel_size(pixel_size: u32) -> Self {
        if pixel_size <= Self::Normal.pixel_size() {
            Self::Normal
        } else if pixel_size <= Self::Large.pixel_size() {
            Self::Large
        } else if pixel_size <= Self::XLarge.pixel_size() {
            Self::XLarge
        } else {
            Self::XXLarge
        }
    }

    pub const fn pixel_size(self) -> u32 {
        self as u32
    }

    pub const fn subdirectory_name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Large => "large",
            Self::XLarge => "x-large",
            Self::XXLarge => "xx-large",
        }
    }
}

pub enum CachedThumbnail {
    /// The cached thumbnail is valid and should be used with size if known.
    Valid((PathBuf, Option<ThumbnailSize>)),
    /// The cached thumbnail doesn't exist or it's invalid and
    /// needs to be recreated with the pixel size.
    RequiresUpdate(ThumbnailSize),
    // The cached thumbnail is in a failed state.
    // This means it failed to create by cosmic-files in the past
    // and shouldn't be tried again.
    Failed,
}

static THUMBNAIL_CACHE_BASE_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    if let Some(cache_dir) = dirs::cache_dir() {
        return Some(cache_dir.join("thumbnails"));
    }

    log::warn!("failed to get thumbnail cache directory, thumbnails will not be cached");

    None
});

#[cfg(test)]
mod tests {
    use super::{ThumbnailSize, thumbnail_cache_relative_path, thumbnail_pixel_size};

    /// Where a grid thumbnail of `file_uri` lands for a window at `scale_factor`.
    fn cache_path_at(file_uri: &str, scale_factor: f32) -> std::path::PathBuf {
        let pixel_size = thumbnail_pixel_size(320, scale_factor);
        thumbnail_cache_relative_path(file_uri, ThumbnailSize::from_pixel_size(pixel_size))
    }

    #[test]
    fn a_thumbnail_is_requested_at_the_displays_pixel_size() {
        // A 320 logical pixel grid icon needs 640 pixels to be sharp on a 2x panel.
        assert_eq!(thumbnail_pixel_size(320, 2.0), 640);
        assert_eq!(thumbnail_pixel_size(320, 1.0), 320);
        assert_eq!(thumbnail_pixel_size(320, 1.5), 480);
    }

    #[test]
    fn an_unusable_scale_factor_requests_the_logical_size() {
        // macOS reports a scale factor of zero before the window is on a screen.
        assert_eq!(thumbnail_pixel_size(320, 0.0), 320);
        assert_eq!(thumbnail_pixel_size(320, -2.0), 320);
        assert_eq!(thumbnail_pixel_size(320, f32::NAN), 320);
        assert_eq!(thumbnail_pixel_size(320, f32::INFINITY), 320);
    }

    #[test]
    fn a_thumbnail_cached_for_one_scale_does_not_overwrite_another() {
        let uri = "file:///home/shylo/holiday.jpg";
        assert_ne!(cache_path_at(uri, 1.0), cache_path_at(uri, 2.0));
    }

    #[test]
    fn the_same_file_at_the_same_scale_reuses_one_cache_entry() {
        let uri = "file:///home/shylo/holiday.jpg";
        assert_eq!(cache_path_at(uri, 2.0), cache_path_at(uri, 2.0));
        assert_ne!(
            cache_path_at(uri, 2.0),
            cache_path_at("file:///home/shylo/other.jpg", 2.0)
        );
    }

    #[test]
    fn a_cache_entry_names_the_size_it_holds() {
        // The freedesktop layout puts the size in the directory, so an entry rendered for one
        // scale is never read back as if it were the other scale's.
        let uri = "file:///home/shylo/holiday.jpg";
        assert!(cache_path_at(uri, 1.0).starts_with(ThumbnailSize::XLarge.subdirectory_name()));
        assert!(cache_path_at(uri, 2.0).starts_with(ThumbnailSize::XXLarge.subdirectory_name()));
    }
}
