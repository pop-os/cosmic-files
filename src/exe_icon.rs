// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::io::Read;
use std::path::Path;

pub fn exe_icon(path: &Path) -> Option<icon::Handle> {
    if path.extension()?.to_str()? != "exe" {
        return None;
    }
    let mut data = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .read_to_end(&mut data)
        .ok()?;
    if data.len() < 64 || &data[..2] != b"MZ" {
        return None;
    }

    let file = pelite::PeFile::from_bytes(&data).ok()?;
    let resources = file.resources().ok()?;

    // Write the first ICO group to a vec, then parse it
    let mut ico = Vec::new();
    let (_name, group) = resources.icons().next()?.ok()?;
    group.write(&mut ico).ok()?;

    if ico.len() < 6 {
        return None;
    }
    let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;

    let mut best: Option<(u32, u32, u32, Vec<u8>)> = None;

    for i in 0..count.min(32) {
        let e = 6 + i * 16;
        if ico.len() < e + 16 {
            break;
        }
        let sz = u32::from_le_bytes([ico[e + 8], ico[e + 9], ico[e + 10], ico[e + 11]]) as usize;
        let off = u32::from_le_bytes([ico[e + 12], ico[e + 13], ico[e + 14], ico[e + 15]]) as usize;
        if off + sz > ico.len() {
            continue;
        }
        let d = &ico[off..off + sz];

        let dec = if d.starts_with(b"\x89PNG") {
            decode(d)
        } else {
            dib_to_bmp(d).and_then(|b| decode(&b))
        };
        if let Some((w, h, rgba)) = dec {
            let px = w as u32 * h as u32;
            if best.as_ref().map_or(true, |b| b.0 < px) {
                best = Some((px, w, h, rgba));
            }
        }
    }
    best.map(|(_, w, h, rgba)| icon::from_raster_pixels(w, h, rgba))
}

fn decode(d: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::ImageReader::new(std::io::Cursor::new(d))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    Some((img.width(), img.height(), img.into_raw()))
}

fn dib_to_bmp(d: &[u8]) -> Option<Vec<u8>> {
    if d.len() < 40 {
        return None;
    }
    let hs = u32::from_le_bytes([d[0], d[1], d[2], d[3]]) as usize;
    if hs < 12 || hs > 124 || hs > d.len() {
        return None;
    }
    let w = u32::from_le_bytes([d[4], d[5], d[6], d[7]]);
    // Icon DIBs store height as 2x (XOR + AND mask)
    let h = u32::from_le_bytes([d[8], d[9], d[10], d[11]]) / 2;
    let bpp = u16::from_le_bytes([d[14], d[15]]);
    let comp = u32::from_le_bytes([d[16], d[17], d[18], d[19]]);
    let xor = h as usize * ((w * bpp as u32 + 31) / 32 * 4) as usize;

    let colors = if bpp <= 8 {
        let u = u32::from_le_bytes([d[32], d[33], d[34], d[35]]);
        if u == 0 { 1u32 << bpp } else { u }
    } else if comp == 3 {
        3
    } else {
        0
    };
    let ct = colors as usize * 4;
    if hs + ct + xor > d.len() {
        return None;
    }
    let fsz = 14 + hs + ct + xor;

    let mut b = Vec::with_capacity(fsz);
    b.extend_from_slice(b"BM");
    b.extend_from_slice(&(fsz as u32).to_le_bytes());
    b.extend_from_slice(&0u32.to_le_bytes());
    b.extend_from_slice(&((14 + hs + ct) as u32).to_le_bytes());
    b.extend_from_slice(&d[..hs + ct + xor]);
    // Fix the halved height in the DIB header
    b[22..26].copy_from_slice(&h.to_le_bytes());
    Some(b)
}
