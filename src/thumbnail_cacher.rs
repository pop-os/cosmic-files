use anyhow::Context;
use image::DynamicImage;
use md5::{Digest, Md5};
use rustc_hash::FxHashMap;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::Path,
    sync::LazyLock,
    time::UNIX_EPOCH,
};
use tempfile::NamedTempFile;
use url::Url;

/// Implements thumbnail caching based on the freedesktop.org Thumbnail Managing Standard.
/// <https://specifications.freedesktop.org/thumbnail-spec/latest>/
pub struct ThumbnailCacher {
    file_path: Box<Path>,
    file_uri: Box<str>,
    thumbnail_dir: Box<Path>,
    thumbnail_path: Box<Path>,
    thumbnail_size: ThumbnailSize,
    thumbnail_fail_marker_path: Box<Path>,
}

impl ThumbnailCacher {
    pub fn new(file_path: &Path, thumbnail_size: ThumbnailSize) -> anyhow::Result<Self> {
        let file_uri = thumbnail_uri(file_path).context("failed to create Uri")?;
        let Some(cache_base_dir) = THUMBNAIL_CACHE_BASE_DIR.as_ref() else {
            anyhow::bail!("failed to get thumbnail cache directory");
        };
        let thumbnail_filename = thumbnail_cache_filename(&file_uri);
        let thumbnail_dir = cache_base_dir.join(thumbnail_size.subdirectory_name());
        if !thumbnail_dir.is_dir() {
            log::warn!(
                "{} is not a directory, creating one now",
                thumbnail_dir.display()
            );
            fs::create_dir_all(&thumbnail_dir)?;
        }
        let thumbnail_path = thumbnail_dir.join(&thumbnail_filename).into_boxed_path();

        let mut thumbnail_fail_marker_path = cache_base_dir.join("fail");
        thumbnail_fail_marker_path.push(format!("cosmic-files-{}", env!("CARGO_PKG_VERSION")));
        thumbnail_fail_marker_path.push(&thumbnail_filename);

        Ok(Self {
            file_path: file_path.into(),
            file_uri,
            thumbnail_dir: thumbnail_dir.into_boxed_path(),
            thumbnail_path,
            thumbnail_size,
            thumbnail_fail_marker_path: thumbnail_fail_marker_path.into_boxed_path(),
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

    pub fn update_with_temp_file(&self, temp_file: &NamedTempFile) -> anyhow::Result<&Path> {
        #[cfg(unix)]
        fs::set_permissions(temp_file.path(), fs::Permissions::from_mode(0o600))?;
        self.update_thumbnail_text_metadata(temp_file.path())?;
        fs::rename(temp_file.path(), &self.thumbnail_path)?;

        Ok(&self.thumbnail_path)
    }

    pub fn update_with_image(&self, image: &DynamicImage) -> anyhow::Result<&Path> {
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
            encoder.write_header()?.write_image_data(image.as_raw())?;
        }

        self.update_with_temp_file(&temp_file)
    }

    pub fn create_fail_marker(&self) -> anyhow::Result<()> {
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

    fn update_thumbnail_text_metadata(&self, path: &Path) -> anyhow::Result<()> {
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

        let Some(image_data_size) = reader.output_buffer_size() else {
            anyhow::bail!("The required image buffer size is too large.")
        };

        let mut image_data = vec![0; image_data_size];
        reader.next_frame(&mut image_data)?;

        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(color_type);
        encoder.set_depth(bit_depth);

        text_chunks.insert("Software".to_string(), "COSMIC Files".to_string());
        text_chunks.insert("Thumb::URI".to_string(), self.file_uri.to_string());
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
        let Ok(thumbnail_file) = File::open(thumbnail_path) else {
            return false;
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

        let mut thumb_uri = None;
        let mut thumb_mtime = None;
        let mut thumb_size = None;

        for text in &reader.info().uncompressed_latin1_text {
            match text.keyword.as_str() {
                "Thumb::URI" => thumb_uri = Some(&text.text),
                "Thumb::MTime" => thumb_mtime = Some(&text.text),
                "Thumb::Size" => thumb_size = Some(&text.text),
                _ => (),
            }
        }

        // Thumb::URI is required and must match.
        if thumb_uri.is_none_or(|thumb_uri| *thumb_uri != *self.file_uri) {
            return false;
        }

        // Thumb::MTime is required and must match.
        let Some(thumb_mtime) = thumb_mtime else {
            return false;
        };

        let metadata = match std::fs::metadata(&self.file_path) {
            Ok(m) => m,
            Err(err) => {
                log::warn!(
                    "failed to get metadata of {}: {}",
                    self.file_path.display(),
                    err
                );
                return false;
            }
        };

        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(err) => {
                log::warn!(
                    "failed to get modified from metadata of {}, {}",
                    self.file_path.display(),
                    err
                );
                return false;
            }
        };
        let mtime = modified
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if thumb_mtime.parse() != Ok(mtime) {
            return false;
        }

        // Thumb::Size isn't required, but it should be verified if present.
        if thumb_size.is_some_and(|thumb_size| thumb_size.parse() != Ok(metadata.len())) {
            return false;
        }

        true
    }
}

fn thumbnail_uri(path: &Path) -> io::Result<Box<str>> {
    let absolute_path = fs::canonicalize(path)?;
    let url = Url::from_file_path(&absolute_path)
        .expect("Url::from_file_path should not error on an absolute path");
    // Technically braces don't need to be percent encoded,
    // and they aren't by the url crate, but the thumbnailer used by
    // Gnome Files does. In order to share thumbnails and not get duplicates
    // we should do the same.
    static BRACES_AC: LazyLock<aho_corasick::AhoCorasick> = LazyLock::new(|| {
        aho_corasick::AhoCorasick::new(["[", "]"])
            .expect("Expected AhoCorasick searcher to be built successfully")
    });

    let url = BRACES_AC.replace_all(url.as_str(), &["%5B", "%5D"]);
    Ok(url.into_boxed_str())
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

impl From<u32> for ThumbnailSize {
    fn from(value: u32) -> Self {
        if value <= Self::Normal.pixel_size() {
            Self::Normal
        } else if value <= Self::Large.pixel_size() {
            Self::Large
        } else if value <= Self::XLarge.pixel_size() {
            Self::XLarge
        } else {
            Self::XXLarge
        }
    }
}

pub enum CachedThumbnail {
    /// The cached thumbnail is valid and should be used with size if known.
    Valid((Box<Path>, Option<ThumbnailSize>)),
    /// The cached thumbnail doesn't exist or it's invalid and
    /// needs to be recreated with the pixel size.
    RequiresUpdate(ThumbnailSize),
    // The cached thumbnail is in a failed state.
    // This means it failed to create by cosmic-files in the past
    // and shouldn't be tried again.
    Failed,
}

static THUMBNAIL_CACHE_BASE_DIR: LazyLock<Option<Box<Path>>> = LazyLock::new(|| {
    if let Some(mut cache_dir) = dirs::cache_dir() {
        cache_dir.push("thumbnails");
        return Some(cache_dir.into_boxed_path());
    }

    log::warn!("failed to get thumbnail cache directory, thumbnails will not be cached");

    None
});
