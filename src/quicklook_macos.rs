// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Thumbnails and previews from the macOS Quick Look thumbnailing service.
//!
//! COSMIC Files normally renders previews either with the `image` crate or with an external
//! freedesktop.org thumbnailer. Neither is available on macOS: `image` cannot decode HEIC or
//! camera RAW, and there are no `.thumbnailer` files to run. `QLThumbnailGenerator` covers all
//! of those formats plus PDFs, office documents and video, using the same generators that Finder
//! and Spotlight use.
//!
//! The generator works from a completely unbundled binary, needs no entitlements, and can be
//! driven from a worker thread with no run loop and no `NSApplication`. Its completion handler
//! fires on a background thread, so [`save_thumbnail_png`] bridges it back to the caller over a
//! rendezvous channel and blocks; it is meant to be called from a `spawn_blocking` worker.

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use mime_guess::Mime;
use objc2::AnyThread;
use objc2_foundation::{NSError, NSSize, NSString, NSURL};
use objc2_quick_look_thumbnailing::{
    QLThumbnailGenerationRequest, QLThumbnailGenerationRequestRepresentationTypes,
    QLThumbnailGenerator,
};
use objc2_uniform_type_identifiers::UTTypePNG;

/// Point size requested for the full-screen gallery preview. With [`GALLERY_PREVIEW_SCALE`] this
/// bounds the generated image at 2048x2048 pixels, which stays sharp on a HiDPI display.
pub const GALLERY_PREVIEW_SIZE: f64 = 1024.0;

/// Scale factor requested for the full-screen gallery preview.
pub const GALLERY_PREVIEW_SCALE: f64 = 2.0;

/// How long to wait for the generator before giving up and cancelling the request.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Which representations the generator is allowed to return.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Representation {
    /// Only a real rendering of the file's contents. Unsupported types fail quickly (~5ms)
    /// instead of producing a generic document icon that is hard to tell from a preview.
    Thumbnail,
    /// Any representation, including an icon badged with the file type. Worth falling back to
    /// for image formats, where a badged icon still carries the embedded preview.
    IconOrThumbnail,
}

impl Representation {
    fn to_raw(self) -> QLThumbnailGenerationRequestRepresentationTypes {
        match self {
            Self::Thumbnail => QLThumbnailGenerationRequestRepresentationTypes::Thumbnail,
            Self::IconOrThumbnail => QLThumbnailGenerationRequestRepresentationTypes::All,
        }
    }
}

/// Image subtypes the built-in `image` crate decoder handles, plus SVG, which has its own
/// renderer. Quick Look stays out of the way of these so that the gallery keeps its existing
/// full-resolution and tiling behaviour for them.
const BUILTIN_IMAGE_SUBTYPES: &[&str] = &[
    "apng",
    "avif",
    "bmp",
    "farbfeld",
    "gif",
    "jpeg",
    "jxl",
    "png",
    "qoi",
    // The MIME parser splits a `+xml` suffix off, so `image/svg+xml` arrives as `svg`.
    "svg",
    "tiff",
    "vnd.microsoft.icon",
    "vnd.radiance",
    "webp",
    "x-bmp",
    "x-exr",
    "x-icon",
    "x-portable-anymap",
    "x-portable-bitmap",
    "x-portable-graymap",
    "x-portable-pixmap",
    "x-qoi",
    "x-tga",
    "x-tiff",
];

/// Whether Quick Look, rather than a built-in renderer, owns the preview for this MIME type.
///
/// This does not promise that Quick Look can preview the file — only the generator knows that.
/// It answers the narrower question of which renderer to route a file to when a preview exists.
pub fn owns_preview(mime: &Mime) -> bool {
    match mime.type_().as_str() {
        // Text is shown in an editor widget, not as an image.
        "text" => false,
        // Images go to the `image` crate unless it cannot decode them (HEIC, camera RAW).
        "image" => !BUILTIN_IMAGE_SUBTYPES.contains(&mime.subtype().as_str()),
        _ => true,
    }
}

/// Whether a failed [`Representation::Thumbnail`] request is worth retrying as
/// [`Representation::IconOrThumbnail`].
///
/// Some image formats decline to render a thumbnail but still return their embedded preview
/// through the icon representation. For non-image types a badged generic icon is not a preview,
/// so a failure stays a failure.
pub fn allows_icon_fallback(mime: &Mime) -> bool {
    mime.type_().as_str() == "image"
}

/// Render `src` into a PNG at `dst`, blocking until the generator answers.
///
/// `size` is in points and `scale` is the backing scale factor, so the generated image is bounded
/// at `size * scale` pixels on its longest edge; the aspect ratio of the source is preserved.
/// `dst` is overwritten if it exists, and is left untouched if generation fails.
///
/// Safe to call from any thread. Returns the localized description of the underlying `NSError`
/// on failure.
pub fn save_thumbnail_png(
    src: &Path,
    dst: &Path,
    size: f64,
    scale: f64,
    representation: Representation,
) -> Result<(), String> {
    let src_str = src
        .to_str()
        .ok_or_else(|| format!("path is not valid UTF-8: {}", src.display()))?;
    let dst_str = dst
        .to_str()
        .ok_or_else(|| format!("path is not valid UTF-8: {}", dst.display()))?;

    // SAFETY: every object below is created here and used only while this call holds a strong
    // reference to it. The completion block is reference counted, so the generator keeps it alive
    // past a timeout; the block only sends on a channel, which is harmless once the receiver is
    // gone. No AppKit or main-thread-only API is touched.
    unsafe {
        let src_url = NSURL::fileURLWithPath(&NSString::from_str(src_str));
        let dst_url = NSURL::fileURLWithPath(&NSString::from_str(dst_str));

        let request =
            QLThumbnailGenerationRequest::initWithFileAtURL_size_scale_representationTypes(
                QLThumbnailGenerationRequest::alloc(),
                &src_url,
                NSSize::new(size, size),
                scale,
                representation.to_raw(),
            );

        // A rendezvous channel: the completion handler runs on one of the generator's own
        // threads, and this is the only place it hands its result back.
        let (sender, receiver) = mpsc::sync_channel::<Option<String>>(1);
        let completion = RcBlock::new(move |error: *mut NSError| {
            let failure = if error.is_null() {
                None
            } else {
                Some((*error).localizedDescription().to_string())
            };
            // A full or disconnected channel means the caller already timed out.
            let _ = sender.try_send(failure);
        });

        let generator = QLThumbnailGenerator::sharedGenerator();
        generator.saveBestRepresentationForRequest_toFileAtURL_asContentType_completionHandler(
            &request,
            &dst_url,
            UTTypePNG,
            &completion,
        );

        match receiver.recv_timeout(REQUEST_TIMEOUT) {
            Ok(None) => Ok(()),
            Ok(Some(error)) => Err(error),
            Err(_) => {
                generator.cancelRequest(&request);
                Err(format!(
                    "Quick Look timed out after {}s",
                    REQUEST_TIMEOUT.as_secs()
                ))
            }
        }
    }
}

/// Render `src` into a PNG at `dst`, retrying as an icon request for the MIME types where that
/// is still a real preview. See [`save_thumbnail_png`] for the parameters.
pub fn save_preview_png(
    src: &Path,
    dst: &Path,
    mime: &Mime,
    size: f64,
    scale: f64,
) -> Result<(), String> {
    match save_thumbnail_png(src, dst, size, scale, Representation::Thumbnail) {
        Ok(()) => Ok(()),
        Err(err) => {
            if allows_icon_fallback(mime) {
                save_thumbnail_png(src, dst, size, scale, Representation::IconOrThumbnail)
            } else {
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn mime(s: &str) -> Mime {
        s.parse().expect("test MIME type should parse")
    }

    /// Writes a small solid-colour PNG and returns its path. Quick Look caches a destination
    /// path it has already written, so every test uses a distinct file name.
    fn write_png_fixture(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        let file = std::fs::File::create(&path).expect("fixture should be creatable");
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), 32, 32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .expect("PNG header should be writable")
            .write_image_data(&[0x40u8; 32 * 32 * 4])
            .expect("PNG data should be writable");
        path
    }

    #[test]
    fn owns_preview_leaves_text_and_decodable_images_alone() {
        for decodable in [
            "text/plain",
            "text/markdown",
            "image/png",
            "image/jpeg",
            "image/gif",
            "image/webp",
            "image/svg+xml",
            "image/avif",
        ] {
            assert!(
                !owns_preview(&mime(decodable)),
                "{decodable} should be rendered by the built-in path"
            );
        }
    }

    #[test]
    fn owns_preview_claims_documents_media_and_undecodable_images() {
        for claimed in [
            "application/pdf",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "application/vnd.apple.keynote",
            "video/mp4",
            "video/quicktime",
            "image/heic",
            "image/heif",
            "image/x-canon-cr2",
            "image/x-adobe-dng",
        ] {
            assert!(
                owns_preview(&mime(claimed)),
                "{claimed} should be rendered by Quick Look"
            );
        }
    }

    #[test]
    fn icon_fallback_is_only_offered_to_images() {
        assert!(allows_icon_fallback(&mime("image/heic")));
        assert!(allows_icon_fallback(&mime("image/png")));
        assert!(!allows_icon_fallback(&mime("application/pdf")));
        assert!(!allows_icon_fallback(&mime("video/mp4")));
        assert!(!allows_icon_fallback(&mime("application/x-sharedlib")));
    }

    #[test]
    fn generates_a_png_for_a_supported_file() {
        let dir = tempfile::tempdir().expect("temp dir should be creatable");
        let src = write_png_fixture(dir.path(), "source.png");
        let dst = dir.path().join("supported-out.png");

        save_thumbnail_png(&src, &dst, 128.0, 1.0, Representation::Thumbnail)
            .expect("Quick Look should render a PNG");

        let header = std::fs::read(&dst).expect("output should be readable");
        assert_eq!(
            &header[..8],
            b"\x89PNG\r\n\x1a\n",
            "output should be a PNG file"
        );
    }

    #[test]
    fn respects_the_requested_size() {
        let dir = tempfile::tempdir().expect("temp dir should be creatable");
        let src = write_png_fixture(dir.path(), "sized-source.png");
        let dst = dir.path().join("sized-out.png");

        save_thumbnail_png(&src, &dst, 64.0, 1.0, Representation::Thumbnail)
            .expect("Quick Look should render a PNG");

        let (width, height) =
            image::image_dimensions(&dst).expect("output dimensions should be readable");
        assert!(
            width <= 64 && height <= 64,
            "expected at most 64x64, got {width}x{height}"
        );
    }

    #[test]
    fn reports_failure_without_writing_the_destination() {
        let dir = tempfile::tempdir().expect("temp dir should be creatable");
        let src = dir.path().join("unpreviewable.bin");
        std::fs::write(&src, [0x00u8, 0x01, 0x02, 0x03]).expect("fixture should be writable");
        let dst = dir.path().join("failure-out.png");

        let result = save_thumbnail_png(&src, &dst, 128.0, 1.0, Representation::Thumbnail);

        assert!(result.is_err(), "unpreviewable file should not succeed");
        assert!(!dst.exists(), "failed request should not leave a file");
    }

    #[test]
    fn reports_failure_for_a_missing_source() {
        let dir = tempfile::tempdir().expect("temp dir should be creatable");
        let dst = dir.path().join("missing-out.png");

        let result = save_thumbnail_png(
            &dir.path().join("does-not-exist.pdf"),
            &dst,
            128.0,
            1.0,
            Representation::Thumbnail,
        );

        assert!(result.is_err(), "missing source should not succeed");
        assert!(!dst.exists(), "failed request should not leave a file");
    }

    #[test]
    fn rejects_paths_that_are_not_utf8() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let bad = PathBuf::from(OsStr::from_bytes(b"/tmp/\xff\xfe.pdf"));
        let err = save_thumbnail_png(
            &bad,
            Path::new("/tmp/unused.png"),
            128.0,
            1.0,
            Representation::Thumbnail,
        )
        .expect_err("non-UTF-8 path should be rejected");
        assert!(err.contains("not valid UTF-8"), "unexpected error: {err}");
    }
}
