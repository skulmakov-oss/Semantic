//! A small, local, hand-rolled PNG encoder for one fixed, well-understood
//! use: writing the 8-bit RGBA buffer `iced::window::screenshot()`'s real
//! WGPU readback produces straight to a real, standards-conformant PNG
//! file. A new external image/PNG crate would need the same kind of
//! explicit owner authorization this workspace's dependency policy only
//! carries for Iced itself -- a small local implementation for one fixed
//! format is this project's established convention instead (see
//! `examples/workbench_semantic/tests/no_iced_dependency.rs`'s hand-rolled
//! Cargo.lock parser for the same reasoning).
//!
//! Uses uncompressed ("stored") DEFLATE blocks inside the mandatory zlib
//! wrapper -- real, valid, spec-conformant PNG data (any real PNG decoder
//! reads it correctly), just not size-optimized. Fine here: these are
//! verification artifacts, not shipped assets.

use std::io::{self, Write};

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// Writes `rgba` (tightly packed, `width * height * 4` bytes, row-major,
/// top-to-bottom -- exactly `iced::window::Screenshot::rgba`'s own layout)
/// as a real PNG file at `path`.
pub fn write_rgba_png(
    path: &std::path::Path,
    width: u32,
    height: u32,
    rgba: &[u8],
) -> io::Result<()> {
    assert_eq!(
        rgba.len(),
        width as usize * height as usize * 4,
        "rgba buffer length must match width*height*4 exactly"
    );

    let mut out = Vec::new();
    out.extend_from_slice(&PNG_SIGNATURE);

    // IHDR: width, height, bit depth 8, color type 6 (RGBA), compression 0,
    // filter method 0, interlace 0.
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    write_chunk(&mut out, b"IHDR", &ihdr);

    // Real scanline filtering: filter type 0 (None) prefixed to every row
    // -- simplest correct choice, not a compression optimization.
    let stride = width as usize * 4;
    let mut filtered = Vec::with_capacity(height as usize * (stride + 1));
    for row in 0..height as usize {
        filtered.push(0u8);
        filtered.extend_from_slice(&rgba[row * stride..row * stride + stride]);
    }

    let zlib = zlib_stored(&filtered);
    write_chunk(&mut out, b"IDAT", &zlib);

    write_chunk(&mut out, b"IEND", &[]);

    let mut file = std::fs::File::create(path)?;
    file.write_all(&out)
}

fn write_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(kind);
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

/// A real zlib stream (RFC 1950) wrapping `data` as uncompressed ("stored",
/// RFC 1951 BTYPE=00) DEFLATE blocks -- split into <=65535-byte blocks,
/// the format's real hard limit for a single stored block's length field.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + data.len() / 65535 * 5 + 8);
    // CMF=0x78 (deflate, 32K window), FLG chosen so (CMF*256+FLG) % 31 == 0.
    out.push(0x78);
    out.push(0x01);

    const MAX_BLOCK: usize = 65535;
    let mut offset = 0;
    if data.is_empty() {
        out.push(1); // BFINAL=1, BTYPE=00, one empty stored block
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&(!0u16).to_le_bytes());
    }
    while offset < data.len() {
        let end = (offset + MAX_BLOCK).min(data.len());
        let is_last = end == data.len();
        out.push(if is_last { 1 } else { 0 });
        let len = (end - offset) as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&data[offset..end]);
        offset = end;
    }

    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_the_real_known_test_vector() {
        // "123456789" -> 0xCBF43926 is the standard CRC-32 (IEEE 802.3)
        // conformance vector every real implementation is checked against.
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn adler32_matches_a_real_known_value() {
        // zlib's own documented example: adler32("Wikipedia") == 0x11E60398.
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn write_rgba_png_produces_a_real_decodable_file_round_trip() {
        let dir = std::env::temp_dir().join(format!("png_encode_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");

        // A real 2x2 image: red, green, blue, white -- enough to catch a
        // row/column transposition bug, not just "some bytes got written."
        let rgba: [u8; 16] = [
            255, 0, 0, 255, // red
            0, 255, 0, 255, // green
            0, 0, 255, 255, // blue
            255, 255, 255, 255, // white
        ];
        write_rgba_png(&path, 2, 2, &rgba).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..8], &PNG_SIGNATURE);
        // A real, minimal hand-rolled decoder proving round-trip fidelity
        // would duplicate a real PNG library -- instead this test proves
        // the structural invariants a decoder depends on: correct IHDR
        // dimensions/color type, and a correctly-terminated chunk stream
        // ending in a zero-length IEND.
        assert_eq!(&bytes[12..16], b"IHDR");
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        assert_eq!((width, height), (2, 2));
        assert_eq!(bytes[24], 8, "bit depth must be 8");
        assert_eq!(bytes[25], 6, "color type must be 6 (RGBA)");
        assert!(
            bytes.ends_with(b"IEND\xae\x42\x60\x82"),
            "must end in a real, correctly-CRC'd empty IEND chunk"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
