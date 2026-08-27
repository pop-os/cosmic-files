// SPDX-License-Identifier: GPL-3.0-only

use image::{DynamicImage, ImageFormat, RgbaImage};
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;

#[allow(dead_code)]
pub fn thumbnail_from_args(mut args: impl Iterator<Item = OsString>) -> Result<(), Box<dyn Error>> {
    let usage = || {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: cosmic-files-thumbnailer OUTPUT --size SIZE INPUT",
        )
    };
    let output = args.next().ok_or_else(usage)?;
    if args.next().as_deref() != Some(OsStr::new("--size")) {
        return Err(usage().into());
    }
    let size = args
        .next()
        .and_then(|arg| arg.into_string().ok())
        .and_then(|arg| arg.parse::<u32>().ok())
        .filter(|size| *size > 0)
        .ok_or_else(usage)?;
    let input = args.next().ok_or_else(usage)?;
    if args.next().is_some() {
        return Err(usage().into());
    }

    thumbnail(Path::new(&input), Path::new(&output), size)
}

#[allow(dead_code)]
pub fn thumbnail(input: &Path, output: &Path, size: u32) -> Result<(), Box<dyn Error>> {
    let icon = thumbnail_image(input, size)?;
    DynamicImage::ImageRgba8(icon).save_with_format(output, ImageFormat::Png)?;

    Ok(())
}

pub fn thumbnail_image(input: &Path, size: u32) -> Result<RgbaImage, Box<dyn Error>> {
    if size == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "thumbnail size must be positive",
        )
        .into());
    }

    // Map the PE so only the headers and resource pages touched by pelite are read.
    let data = pelite::FileMap::open(input)?;
    let bytes = data.as_ref();
    if bytes.len() < 64 || !bytes.starts_with(b"MZ") {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "input is not a PE file").into());
    }

    let file = pelite::PeFile::from_bytes(bytes).map_err(invalid_data)?;
    let resources = file.resources().map_err(invalid_data)?;
    let (_name, group) = resources
        .icons()
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "PE file has no icon group"))?
        .map_err(invalid_data)?;

    let mut ico = Vec::new();
    group.write(&mut ico).map_err(invalid_data)?;

    Ok(
        DynamicImage::ImageRgba8(largest_icon(&ico).map_err(invalid_data)?)
            .thumbnail(size, size)
            .into_rgba8(),
    )
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

fn largest_icon(ico: &[u8]) -> Result<RgbaImage, String> {
    if ico.len() < 6 {
        return Err("ICO header is truncated".to_string());
    }
    let count = usize::from(u16::from_le_bytes([ico[4], ico[5]]));
    let mut best: Option<(u64, RgbaImage)> = None;
    let mut errors = Vec::new();

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

        match decode_icon(&ico[entry..entry + 16], data) {
            Ok(image) => {
                let pixels = u64::from(image.width()) * u64::from(image.height());
                if best
                    .as_ref()
                    .is_none_or(|(best_pixels, _)| pixels > *best_pixels)
                {
                    best = Some((pixels, image));
                }
            }
            Err(err) => errors.push(format!("frame {i}: {err}")),
        }
    }

    best.map(|(_, image)| image).ok_or_else(|| {
        if errors.is_empty() {
            "PE icon contains no complete frames".to_string()
        } else {
            format!("PE icon could not be decoded ({})", errors.join("; "))
        }
    })
}

fn decode_icon(entry: &[u8], data: &[u8]) -> Result<RgbaImage, String> {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        return image::ImageReader::with_format(io::Cursor::new(data), ImageFormat::Png)
            .decode()
            .map(DynamicImage::into_rgba8)
            .map_err(|err| err.to_string());
    }

    let capacity = 22usize
        .checked_add(data.len())
        .ok_or_else(|| "frame is too large".to_string())?;
    let data_len = u32::try_from(data.len()).map_err(|_| "frame is too large".to_string())?;
    let mut ico = Vec::with_capacity(capacity);
    ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
    ico.extend_from_slice(entry);
    ico[14..18].copy_from_slice(&data_len.to_le_bytes());
    ico[18..22].copy_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(data);

    image::ImageReader::with_format(io::Cursor::new(ico), ImageFormat::Ico)
        .decode()
        .map(DynamicImage::into_rgba8)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{largest_icon, thumbnail_from_args};
    use image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
    use std::ffi::OsString;
    use std::io::Cursor;

    #[test]
    fn malformed_thumbnail_request_returns_an_error() {
        let args = [OsString::from("output.png")];
        assert!(thumbnail_from_args(args.into_iter()).is_err());
    }

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

    #[test]
    fn dib_and_mask_is_applied() {
        let mut dib = vec![0; 40];
        dib[0..4].copy_from_slice(&40u32.to_le_bytes());
        dib[4..8].copy_from_slice(&2i32.to_le_bytes());
        dib[8..12].copy_from_slice(&4i32.to_le_bytes());
        dib[12..14].copy_from_slice(&1u16.to_le_bytes());
        dib[14..16].copy_from_slice(&24u16.to_le_bytes());

        // Two bottom-up BGR rows, padded to four-byte boundaries.
        dib.extend_from_slice(&[0, 0, 255, 0, 0, 255, 0, 0]);
        dib.extend_from_slice(&[0, 0, 255, 0, 0, 255, 0, 0]);
        // The first mask row is the bottom image row; its first pixel is transparent.
        dib.extend_from_slice(&[0b1000_0000, 0, 0, 0]);
        dib.extend_from_slice(&[0, 0, 0, 0]);

        let mut ico = vec![0, 0, 1, 0, 1, 0];
        let mut entry = [0; 16];
        entry[..8].copy_from_slice(&[2, 2, 0, 0, 1, 0, 24, 0]);
        entry[8..12].copy_from_slice(&u32::try_from(dib.len()).unwrap().to_le_bytes());
        entry[12..16].copy_from_slice(&22u32.to_le_bytes());
        ico.extend_from_slice(&entry);
        ico.extend_from_slice(&dib);

        let decoded = largest_icon(&ico).expect("valid DIB icon");
        assert_eq!(decoded.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
        assert_eq!(decoded.get_pixel(0, 1), &Rgba([255, 0, 0, 0]));
    }

    #[test]
    fn rgb_png_icon_is_decoded() {
        let image = RgbImage::from_pixel(2, 2, Rgb([1, 2, 3]));
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(image)
            .write_to(&mut png, ImageFormat::Png)
            .unwrap();
        let png = png.into_inner();

        let mut ico = vec![0, 0, 1, 0, 1, 0];
        let mut entry = [0; 16];
        entry[..8].copy_from_slice(&[2, 2, 0, 0, 1, 0, 24, 0]);
        entry[8..12].copy_from_slice(&u32::try_from(png.len()).unwrap().to_le_bytes());
        entry[12..16].copy_from_slice(&22u32.to_le_bytes());
        ico.extend_from_slice(&entry);
        ico.extend_from_slice(&png);

        let decoded = largest_icon(&ico).expect("valid RGB PNG icon");
        assert_eq!(decoded.get_pixel(0, 0), &Rgba([1, 2, 3, 255]));
    }
}
