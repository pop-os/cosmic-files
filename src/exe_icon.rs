// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::io::Read;
use std::path::Path;
use std::process::Command;

/// Extract the embedded icon from a Windows PE executable using wrestool.
/// Only attempts extraction for .exe files.
pub fn exe_icon(path: &Path) -> Option<icon::Handle> {
    // Fast extension check — skip non-executables without touching the file
    let ext = path.extension()?.to_str()?;
    if ext != "exe" {
        return None;
    }

    // PE check
    let mut header = [0u8; 2];
    std::fs::File::open(path)
        .ok()?
        .read_exact(&mut header)
        .ok()?;
    if &header != b"MZ" {
        return None;
    }

    // Single wrestool call for the ICO group
    let output = Command::new("wrestool")
        .args(["-x", "-t", "14"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.len() < 6 {
        return None;
    }

    // Parse ICO container: 6-byte header + count × 16-byte directory entries
    let ico = &output.stdout;
    let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;

    let mut best: Option<(u32, u32, u32, Vec<u8>)> = None;

    for i in 0..count.min(32) {
        let e = 6 + i * 16;
        if ico.len() < e + 16 {
            break;
        }
        let size = u32::from_le_bytes([ico[e + 8], ico[e + 9], ico[e + 10], ico[e + 11]]) as usize;
        let off = u32::from_le_bytes([ico[e + 12], ico[e + 13], ico[e + 14], ico[e + 15]]) as usize;
        if off + size > ico.len() {
            continue;
        }
        let data = &ico[off..off + size];

        // Try PNG (modern executables)
        let decoded = if data.starts_with(b"\x89PNG") {
            decode(data)
        } else {
            // Try DIB (older executables)
            bmp_from_dib(data).and_then(|bmp| decode(&bmp))
        };

        if let Some((w, h, rgba)) = decoded {
            let pixels = w as u32 * h as u32;
            match &best {
                Some((best_px, _, _, _)) if *best_px >= pixels => {}
                _ => best = Some((pixels, w, h, rgba)),
            }
        }
    }

    best.map(|(_, w, h, rgba)| icon::from_raster_pixels(w, h, rgba))
}

fn decode(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let (w, h) = img.dimensions();
    Some((w, h, img.into_raw()))
}

/// Prepend a BITMAPFILEHEADER to raw DIB data.
/// Icon DIBs store height as 2x (XOR + AND mask); halve it and strip the mask.
fn bmp_from_dib(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 40 {
        return None;
    }
    let hs = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if hs < 12 || hs > 124 || hs > data.len() {
        return None;
    }

    let width = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let icon_h = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let bpp = u16::from_le_bytes([data[14], data[15]]);
    let compression = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);

    let image_h = icon_h / 2;
    let row = ((width * bpp as u32 + 31) / 32 * 4) as usize;
    let xor = image_h as usize * row;

    let colors = if bpp <= 8 {
        let used = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        (if used == 0 { 1u32 << bpp } else { used }) as usize
    } else if compression == 3 {
        3
    } else {
        0
    };
    let ct_bytes = colors * 4;
    let off = (14 + hs + ct_bytes) as u32;

    let data_end = hs + ct_bytes + xor;
    if data_end > data.len() {
        return None;
    }
    let file_size = 14 + data_end;

    let mut bmp = Vec::with_capacity(file_size);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&off.to_le_bytes());
    bmp.extend_from_slice(&data[..data_end]);
    bmp[22..26].copy_from_slice(&image_h.to_le_bytes());

    Some(bmp)
}
