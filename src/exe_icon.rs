// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::io::Read;
use std::path::Path;
use std::process::Command;

/// Extract the embedded icon from a Windows PE executable using wrestool.
pub fn exe_icon(path: &Path) -> Option<icon::Handle> {
    let mut header = [0u8; 2];
    if std::fs::File::open(path)
        .ok()?
        .read_exact(&mut header)
        .is_err()
    {
        return None;
    }
    if &header != b"MZ" {
        return None;
    }

    log::info!("exe_icon called for {}", path.display());

    // Extract individual RT_ICON entries, pick the largest decodable one
    let mut best: Option<(u32, u32, u32, Vec<u8>)> = None;
    for id in (1..=20).rev() {
        let output = Command::new("wrestool")
            .args(["-x", "--raw", "-t", "3", "-n", &id.to_string()])
            .arg(path)
            .output()
            .ok()?;
        if output.stdout.is_empty() || !output.status.success() {
            continue;
        }
        log::info!("exe_icon: id={} {} bytes", id, output.stdout.len());
        if let Some((w, h, rgba)) = decode_icon(&output.stdout) {
            let pixels = w as u32 * h as u32;
            match &best {
                Some((best_px, _, _, _)) if *best_px >= pixels => {}
                _ => best = Some((pixels, w, h, rgba)),
            }
        }
    }
    best.map(|(_, w, h, rgba)| icon::from_raster_pixels(w, h, rgba))
}

fn decode_icon(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if let Some(result) = try_decode(data) {
        log::info!("exe_icon: decoded direct ({} bytes)", data.len());
        return Some(result);
    }
    if let Some(bmp) = bmp_from_dib(data) {
        if let Some(result) = try_decode(&bmp) {
            log::info!("exe_icon: decoded DIB->BMP ({} bytes raw)", data.len());
            return Some(result);
        }
    }
    log::info!("exe_icon: decode failed for {} bytes", data.len());
    None
}

fn try_decode(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    use image::ImageReader;
    let img = ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let (w, h) = img.dimensions();
    Some((w, h, img.into_raw()))
}

/// Prepend a BITMAPFILEHEADER to raw DIB data so the image crate can decode it.
/// Icon DIBs store height as 2x (XOR + AND mask); we halve it and strip the AND mask.
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
    bmp.extend_from_slice(&0u32.to_le_bytes()); // reserved
    bmp.extend_from_slice(&off.to_le_bytes());
    bmp.extend_from_slice(&data[..data_end]);
    // Fix height: offset 8 in DIB = offset 22 in BMP
    bmp[22..26].copy_from_slice(&image_h.to_le_bytes());

    Some(bmp)
}
