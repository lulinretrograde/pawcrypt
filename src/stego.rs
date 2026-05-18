// LSB steganography: hide a .paw vault inside a PNG.
//
// Layout (1 bit per raw RGBA byte, LSB-first within each payload byte):
//   bytes 0..4   — magic "PAW\x01"
//   bytes 4..8   — vault length as u32 LE
//   bytes 8..    — vault data
//
// Capacity: floor(width * height * 4 / 8) bytes.
// Visual change per pixel: ±1 in one channel — imperceptible.

use anyhow::{bail, Result};
use image::RgbaImage;
use std::path::Path;

const MAGIC: &[u8; 4] = b"PAW\x01";
const HEADER_LEN: usize = MAGIC.len() + 4; // magic + u32

pub fn hide(vault_bytes: &[u8], img_path: &Path, out_path: &Path) -> Result<()> {
    let img = image::open(img_path)?;
    let mut buf: RgbaImage = img.to_rgba8();

    let vault_len: u32 = vault_bytes
        .len()
        .try_into()
        .map_err(|_| anyhow::anyhow!("vault too large to embed"))?;

    let capacity = buf.len() / 8;
    let total = HEADER_LEN + vault_bytes.len();
    if total > capacity {
        bail!(
            "image too small: need {} bytes capacity, have {} — try a larger PNG",
            total,
            capacity
        );
    }

    let mut payload = Vec::with_capacity(total);
    payload.extend_from_slice(MAGIC);
    payload.extend_from_slice(&vault_len.to_le_bytes());
    payload.extend_from_slice(vault_bytes);

    let mut bit_iter = payload
        .iter()
        .flat_map(|&b| (0..8u8).map(move |i| (b >> i) & 1));

    for raw in buf.iter_mut() {
        match bit_iter.next() {
            Some(bit) => *raw = (*raw & 0xFE) | bit,
            None => break,
        }
    }

    buf.save(out_path)?;
    Ok(())
}

pub fn extract(img_path: &Path) -> Result<Vec<u8>> {
    let img = image::open(img_path)?;
    let buf: RgbaImage = img.to_rgba8();

    let lsbs: Vec<u8> = buf.iter().map(|&b| b & 1).collect();

    if lsbs.len() < HEADER_LEN * 8 {
        bail!("image too small to contain a vault");
    }

    let read_byte = |byte_idx: usize| -> u8 {
        let start = byte_idx * 8;
        (0..8usize).fold(0u8, |acc, i| acc | (lsbs[start + i] << i))
    };

    for (i, &m) in MAGIC.iter().enumerate() {
        if read_byte(i) != m {
            bail!("no pawcrypt vault found in this image >:3");
        }
    }

    let len_bytes: [u8; 4] = std::array::from_fn(|i| read_byte(MAGIC.len() + i));
    let vault_len = u32::from_le_bytes(len_bytes) as usize;

    if lsbs.len() < (HEADER_LEN + vault_len) * 8 {
        bail!("image data truncated — vault may be corrupted");
    }

    Ok((0..vault_len)
        .map(|i| read_byte(HEADER_LEN + i))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_png(path: &Path, width: u32, height: u32) {
        let img = RgbaImage::new(width, height);
        img.save(path).unwrap();
    }

    #[test]
    fn hide_extract_roundtrip() {
        let img_path = std::path::PathBuf::from("/tmp/stego_test_in.png");
        let out_path = std::path::PathBuf::from("/tmp/stego_test_out.png");
        make_test_png(&img_path, 100, 100);

        let vault = b"nyaaa this is vault data owo~";
        hide(vault, &img_path, &out_path).unwrap();
        let recovered = extract(&out_path).unwrap();
        assert_eq!(recovered, vault);

        std::fs::remove_file(&img_path).ok();
        std::fs::remove_file(&out_path).ok();
    }

    #[test]
    fn wrong_image_fails() {
        let img_path = std::path::PathBuf::from("/tmp/stego_blank.png");
        make_test_png(&img_path, 50, 50);
        assert!(extract(&img_path).is_err());
        std::fs::remove_file(&img_path).ok();
    }
}
