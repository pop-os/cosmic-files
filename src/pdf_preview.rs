// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

//! Large PDF preview for detail pane and gallery using evince-thumbnailer at higher resolution.

use crate::large_image::LargeImageManager;
use mime_guess::Mime;
use std::path::{Path, PathBuf};

/// Size in pixels for the large PDF preview (first page) in preview pane and gallery.
pub const PDF_PREVIEW_SIZE: u32 = 1024;

/// Returns true if the MIME type is application/pdf.
pub fn is_pdf_mime(mime: &Mime) -> bool {
    mime.type_() == "application" && mime.subtype() == "pdf"
}

/// Starts PDF preview decode via the shared large-image manager. Caller should spawn
/// decode_pdf_preview and on success use store_decoded_with_generation(path, handle, None, 0).
/// Returns true if decode was started, false if already decoding or already decoded.
pub fn try_decode_pdf(manager: &mut LargeImageManager, path: &Path) -> bool {
    manager.try_start_decode(path, 0)
}

/// Runs the PDF thumbnailer (e.g. evince-thumbnailer) at larger size and returns RGBA pixels.
pub async fn decode_pdf_preview(path: PathBuf) -> Option<(PathBuf, u32, u32, Vec<u8>)> {
    tokio::task::spawn_blocking(move || {
        let mime: Mime = "application/pdf".parse().ok()?;
        let thumbnailers = crate::thumbnailer::thumbnailer(&mime);
        for thumb in thumbnailers {
            let is_evince = thumb.exec.starts_with("evince-thumbnailer ");
            // evince-thumbnailer often runs under apparmor and cannot write to cache dirs
            let prefix = if is_evince { "gnome-desktop-" } else { "cosmic-files-" };
            let file = tempfile::Builder::new().prefix(prefix).tempfile();
            let file = match file {
                Ok(f) => f,
                Err(e) => {
                    log::warn!("pdf preview temp file for {}: {}", path.display(), e);
                    continue;
                }
            };
            let mut cmd = thumb.command(&path, file.path(), PDF_PREVIEW_SIZE)?;
            if cmd.status().ok()?.success() {
                let reader = image::ImageReader::open(file.path())
                    .and_then(image::ImageReader::with_guessed_format)
                    .ok()?;
                let img = reader.decode().ok()?;
                let rgba = img.into_rgba8();
                let (w, h) = (rgba.width(), rgba.height());
                let pixels = rgba.into_raw();
                log::debug!("Decoded PDF preview {}x{} for {}", w, h, path.display());
                return Some((path, w, h, pixels));
            }
        }
        log::warn!("No thumbnailer produced preview for {}", path.display());
        None
    })
    .await
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::large_image::LargeImageManager;
    use mime_guess::Mime;

    use super::{is_pdf_mime, try_decode_pdf};

    #[test]
    fn is_pdf_mime_accepts_application_pdf() {
        let mime: Mime = "application/pdf".parse().unwrap();
        assert!(is_pdf_mime(&mime));
    }

    #[test]
    fn is_pdf_mime_rejects_other_types() {
        for s in [
            "application/zip",
            "application/x-pdf",
            "text/plain",
            "image/png",
            "application/octet-stream",
        ] {
            let mime: Mime = s.parse().unwrap();
            assert!(!is_pdf_mime(&mime), "expected {} to not be pdf mime", s);
        }
    }

    #[test]
    fn try_decode_pdf_returns_true_when_empty() {
        let mut m = LargeImageManager::new();
        let path = PathBuf::from("/tmp/test.pdf");
        assert!(try_decode_pdf(&mut m, &path));
        assert!(m.is_decoding(&path));
    }

    #[test]
    fn try_decode_pdf_returns_false_when_already_decoding() {
        let mut m = LargeImageManager::new();
        let path = PathBuf::from("/tmp/test.pdf");
        assert!(try_decode_pdf(&mut m, &path));
        assert!(!try_decode_pdf(&mut m, &path));
    }

    #[test]
    fn try_decode_pdf_returns_false_when_already_decoded() {
        let mut m = LargeImageManager::new();
        let path = PathBuf::from("/tmp/test.pdf");
        assert!(try_decode_pdf(&mut m, &path));
        let handle = cosmic::widget::image::Handle::from_rgba(1, 1, vec![0, 0, 0, 255]);
        m.store_decoded_with_generation(path.clone(), handle, None, 0);
        assert!(!try_decode_pdf(&mut m, &path));
        assert!(m.get_decoded(&path).is_some());
    }
}
