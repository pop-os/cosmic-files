// SPDX-License-Identifier: GPL-3.0-only

use image::{DynamicImage, ImageFormat, RgbaImage};
use std::error::Error;
use std::io;
use std::path::Path;

pub fn thumbnail(input: &Path, output: &Path, size: u32) -> Result<(), Box<dyn Error>> {
    if size == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "thumbnail size must be positive",
        )
        .into());
    }

    // pelite requires the complete PE image. This runs only in the external thumbnailer process.
    let data = std::fs::read(input)?;
    if data.len() < 64 || !data.starts_with(b"MZ") {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "input is not a PE file").into());
    }

    let file = pelite::PeFile::from_bytes(&data).map_err(invalid_data)?;
    let resources = file.resources().map_err(invalid_data)?;
    let (_name, group) = resources
        .icons()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "PE file has no icon group"))?
        .map_err(invalid_data)?;

    let mut ico = Vec::new();
    group.write(&mut ico).map_err(invalid_data)?;

    let icon = largest_icon(&ico).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "PE icon could not be decoded")
    })?;
    DynamicImage::ImageRgba8(icon)
        .thumbnail(size, size)
        .save_with_format(output, ImageFormat::Png)?;

    Ok(())
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn largest_icon(ico: &[u8]) -> Option<RgbaImage> {
    if ico.len() < 6 {
        return None;
    }
    let count = usize::from(u16::from_le_bytes([ico[4], ico[5]]));
    let mut best: Option<(u64, RgbaImage)> = None;

    for i in 0..count.min(32) {
        let entry = 6 + i * 16;
        if ico.len() < entry + 16 {
            break;
        }
        let size = u32::from_le_bytes([
            ico[entry + 8],
            ico[entry + 9],
            ico[entry + 10],
            ico[entry + 11],
        ]) as usize;
        let offset = u32::from_le_bytes([
            ico[entry + 12],
            ico[entry + 13],
            ico[entry + 14],
            ico[entry + 15],
        ]) as usize;
        let Some(end) = offset.checked_add(size) else {
            continue;
        };
        let Some(data) = ico.get(offset..end) else {
            continue;
        };

        let decoded = if data.starts_with(b"\x89PNG") {
            decode(data)
        } else {
            dib_to_bmp(data).and_then(|bmp| decode(&bmp))
        };
        if let Some(image) = decoded {
            let pixels = u64::from(image.width()) * u64::from(image.height());
            if best
                .as_ref()
                .is_none_or(|(best_pixels, _)| pixels > *best_pixels)
            {
                best = Some((pixels, image));
            }
        }
    }

    best.map(|(_, image)| image)
}

fn decode(data: &[u8]) -> Option<RgbaImage> {
    image::ImageReader::new(io::Cursor::new(data))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
        .map(DynamicImage::into_rgba8)
}

fn dib_to_bmp(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 40 {
        return None;
    }
    let header_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if !(40..=124).contains(&header_size) || header_size > data.len() {
        return None;
    }

    let width = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    // Icon DIBs store height as 2x (XOR image plus AND mask).
    let height = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) / 2;
    let bits_per_pixel = u16::from_le_bytes([data[14], data[15]]);
    let compression = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let row_bits = width.checked_mul(u32::from(bits_per_pixel))?;
    let row_bytes = row_bits.checked_add(31)?.checked_div(32)?.checked_mul(4)?;
    let xor_size = usize::try_from(height)
        .ok()?
        .checked_mul(usize::try_from(row_bytes).ok()?)?;

    let colors = if bits_per_pixel <= 8 {
        let used = u32::from_le_bytes([data[32], data[33], data[34], data[35]]);
        if used == 0 {
            1u32 << bits_per_pixel
        } else {
            used
        }
    } else if compression == 3 {
        3
    } else {
        0
    };
    let color_table_size = usize::try_from(colors).ok()?.checked_mul(4)?;
    let dib_size = header_size
        .checked_add(color_table_size)?
        .checked_add(xor_size)?;
    if dib_size > data.len() {
        return None;
    }

    let file_size = 14usize.checked_add(dib_size)?;
    let pixel_offset = 14usize
        .checked_add(header_size)?
        .checked_add(color_table_size)?;
    let mut bmp = Vec::with_capacity(file_size);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&u32::try_from(file_size).ok()?.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&u32::try_from(pixel_offset).ok()?.to_le_bytes());
    bmp.extend_from_slice(&data[..dib_size]);
    bmp[22..26].copy_from_slice(&height.to_le_bytes());
    Some(bmp)
}

#[cfg(test)]
mod tests {
    use super::largest_icon;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    #[test]
    fn largest_icon_skips_invalid_entries() {
        let image = RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]));
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut png, ImageFormat::Png)
            .unwrap();
        let png = png.into_inner();

        let mut ico = vec![0, 0, 1, 0, 2, 0];
        let mut invalid_entry = [0; 16];
        invalid_entry[8..12].copy_from_slice(&4u32.to_le_bytes());
        invalid_entry[12..16].copy_from_slice(&u32::MAX.to_le_bytes());
        ico.extend_from_slice(&invalid_entry);

        let mut valid_entry = [0; 16];
        valid_entry[..8].copy_from_slice(&[2, 2, 0, 0, 1, 0, 32, 0]);
        valid_entry[8..12].copy_from_slice(&u32::try_from(png.len()).unwrap().to_le_bytes());
        valid_entry[12..16].copy_from_slice(&38u32.to_le_bytes());
        ico.extend_from_slice(&valid_entry);
        ico.extend_from_slice(&png);

        let decoded = largest_icon(&ico).expect("valid icon after malformed entry");
        assert_eq!(decoded.dimensions(), (2, 2));
        assert_eq!(decoded.get_pixel(0, 0), &Rgba([1, 2, 3, 255]));
    }
}
