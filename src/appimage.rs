// SPDX-License-Identifier: GPL-3.0-only

//! Extracts the embedded icon from AppImage files for display as file icons
//! (issue #2164).
//!
//! AppImages store their icon canonically in the AppDir root, `.DirIcon`
//! (https://docs.appimage.org/reference/appdir.html). Type 2 AppImages are
//! SquashFS filesystems with the ELF runtime prepended, so the SquashFS magic
//! is searched for in the file bytes, and `unsquashfs` extracts `.DirIcon`.

use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use mime_guess::mime;
use tempfile::TempDir;

/// AppImage MIME types that may carry an embedded icon.
const APPIMAGE_MIMES: [&str; 3] = [
    "application/vnd.appimage",
    "application/x-iso9660-appimage",
    "application/x-appimage",
];

/// SquashFS superblock magic as stored in the file.
const SQUASHFS_MAGIC: [u8; 4] = *b"hsqs";

/// How far into the file to scan for the SquashFS magic. AppImage payloads start
/// shortly after the ELF runtime, so the cap just bounds the scan.
const MAX_SCAN_OFFSET: u64 = 8 * 1024 * 1024;

/// Maximum `.DirIcon` size read from the AppImage.
const MAX_ICON_BYTES: u64 = 1024 * 1024;

pub fn is_appimage(mime: &mime::Mime) -> bool {
    let mime: &str = mime.as_ref();
    APPIMAGE_MIMES.contains(&mime)
}

/// Returns true for icon bytes that `image` cannot decode but `svg` handles.
pub fn looks_like_svg(data: &[u8]) -> bool {
    let mut start = 0;
    while start < data.len() && data[start].is_ascii_whitespace() {
        start += 1;
    }
    data[start..].starts_with(b"<?xml") || data[start..].starts_with(b"<svg")
}

/// Extracts the embedded `.DirIcon` from an AppImage file, as raw bytes.
///
/// Returns `None` if the file has no readable `.DirIcon` or the container is
/// not a SquashFS AppImage (e.g. ISO9600 type 1 or DwarfFS type 3).
pub fn extract_icon(path: &Path) -> Option<Vec<u8>> {
    for offset in squashfs_offsets(path) {
        if let Some(icon) = extract_dir_icon(path, offset) {
            return Some(icon);
        }
    }
    None
}

/// Finds offsets of the SquashFS magic in the file, scanning the first
/// `MAX_SCAN_OFFSET` bytes. Returns offsets in ascending order.
fn squashfs_offsets(path: &Path) -> Vec<u64> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            log::debug!("failed to open {}: {err}", path.display());
            return Vec::new();
        }
    };
    let mut offsets: Vec<u64> = Vec::new();
    let mut tail: Vec<u8> = Vec::new();
    let mut end = 0u64;
    loop {
        if end >= MAX_SCAN_OFFSET {
            break;
        }
        let mut chunk = [0u8; 65536];
        let n = match file.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(err) => {
                log::debug!("failed to scan {}: {err}", path.display());
                break;
            }
        };
        let chunk_end = end + n as u64;
        end = chunk_end;
        let mut window = std::mem::take(&mut tail);
        window.extend_from_slice(&chunk[..n]);
        let window_start = chunk_end - window.len() as u64;
        for i in find_magic(&window) {
            let offset = window_start + i as u64;
            if !offsets.contains(&offset) && offset <= MAX_SCAN_OFFSET {
                offsets.push(offset);
            }
        }
        // Keep the tail so the magic can straddle chunk boundaries.
        tail = window[window.len().saturating_sub(3)..].to_vec();
    }
    offsets
}

fn find_magic(window: &[u8]) -> Vec<usize> {
    let mut found = Vec::new();
    if window.len() >= 4 {
        for i in 0..=window.len() - 4 {
            if window.get(i..i + 4) == Some(&SQUASHFS_MAGIC[..]) {
                found.push(i);
            }
        }
    }
    found
}

/// Runs `unsquashfs -offset <offset>` on the file and reads the extracted
/// `.DirIcon` from the output directory.
fn extract_dir_icon(path: &Path, offset: u64) -> Option<Vec<u8>> {
    let dest = TempDir::new().ok()?;
    let status = Command::new("unsquashfs")
        .arg("-offset")
        .arg(offset.to_string())
        .arg("-n") // do not display the progress bar
        .arg("-f") // overwrite if the destination exists
        .arg("-d")
        .arg(dest.path())
        .arg(path)
        .arg(".DirIcon")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()?;
    if !status.success() {
        log::debug!("unsquashfs failed on {} at offset {offset}", path.display());
        return None;
    }
    read_bounded(&dest.path().join(".DirIcon"))
}

/// Reads a file bounded by `MAX_ICON_BYTES`.
fn read_bounded(path: &Path) -> Option<Vec<u8>> {
    let len = std::fs::metadata(path).ok()?.len();
    if len > MAX_ICON_BYTES {
        return None;
    }
    let mut data = Vec::new();
    File::open(path)
        .ok()?
        .take(MAX_ICON_BYTES)
        .read_to_end(&mut data)
        .ok()?;
    Some(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(data: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(data).unwrap();
        file
    }

    #[test]
    fn find_magic_finds_all_occurrences() {
        assert_eq!(find_magic(b"abc\x68\x73\x71\x73def"), vec![3]);
        assert_eq!(
            find_magic(b"\x68\x73\x71\x73xx\x68\x73\x71\x73"),
            vec![0, 6]
        );
        assert!(find_magic(b"abcd").is_empty());
        assert!(find_magic(b"abc").is_empty());
        assert!(find_magic(b"").is_empty());
    }

    #[test]
    fn squashfs_offsets_scans_beyond_chunk_boundary() {
        let mut data = b"fake appimage payload".to_vec();
        data.extend([0u8; 70_000]);
        data.extend_from_slice(&SQUASHFS_MAGIC);
        data.extend([0x11u8; 10]);
        let file = write_temp(&data);
        assert_eq!(
            squashfs_offsets(file.path()),
            vec![(data.len() - 10 - 4) as u64]
        );
    }

    #[test]
    fn squashfs_offsets_empty_for_missing_file() {
        assert!(squashfs_offsets(Path::new("/nonexistent/cosmic-files-test")).is_empty());
    }

    #[test]
    fn detects_svg_icon() {
        assert!(looks_like_svg(b"<?xml version=\"1.0\"?>\n<svg/>"));
        assert!(looks_like_svg(b"<svg xmlns=\"x\"/>"));
        assert!(!looks_like_svg(b"\x89PNG fake"));
        assert!(!looks_like_svg(b""));
    }

    #[test]
    fn extract_icon_none_for_empty_file() {
        let file = tempfile::NamedTempFile::new().unwrap();
        assert!(extract_icon(file.path()).is_none());
    }

    #[test]
    fn extract_icon_from_fake_appimage() {
        if Command::new("mksquashfs").arg("-h").output().is_err()
            || Command::new("unsquashfs").arg("-h").output().is_err()
        {
            // Skip without squashfs tooling; scan and extraction are covered by other tests.
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let appdir = dir.path().join("AppDir");
        fs::create_dir_all(&appdir).unwrap();
        fs::write(appdir.join(".DirIcon"), b"fake icon").unwrap();
        let image = dir.path().join("payload.squashfs");
        let status = Command::new("mksquashfs")
            .arg(&appdir)
            .arg(&image)
            .args(["-comp", "gzip", "-no-progress", "-all-root", "-no-xattrs"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        if !status.success() {
            return;
        }
        // Prepend junk to simulate the ELF runtime prepended to AppImages.
        let mut fake = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
        fake.write_all(b"fake appimage runtime prefix\n").unwrap();
        fake.write_all(&fs::read(&image).unwrap()).unwrap();
        assert_eq!(extract_icon(fake.path()), Some(b"fake icon".to_vec()));
    }
}
