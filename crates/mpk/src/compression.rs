use std::{
    borrow::Cow,
    io::{Cursor, Read},
};

use anyhow::{ensure, Context};
use tracing::warn;

#[derive(Debug)]
pub enum CompressionType {
    None,
    Zlib,
    Zstd,
    G108Zstd,
    Lzma,
    Lz4,
    G108Lz4,
}

impl CompressionType {
    pub fn is_g108(&self) -> bool {
        matches!(self, CompressionType::G108Lz4 | CompressionType::G108Zstd)
    }

    pub fn detect_from_slice(buf: &[u8]) -> Option<CompressionType> {
        if buf.len() < 4 {
            return None;
        }

        match &buf[0..4] {
            b"NNNN" => Some(CompressionType::None),
            &[0x42, 0xA6, ..] => Some(CompressionType::Zlib),
            b"LZMA" => Some(CompressionType::Lzma),
            b"ZZZ4" => Some(CompressionType::Lz4),
            b"ZSTD" => Some(CompressionType::Zstd),
            b"1084" => Some(CompressionType::G108Lz4),
            b"108D" => Some(CompressionType::G108Zstd),
            _ => None,
        }
    }
}

/// Decompresses the given buffer.
///
/// This function may modify the given buffer due to the use of in-place XOR encryption.
pub fn decompress(buf: &mut [u8]) -> anyhow::Result<Cow<'_, [u8]>> {
    match CompressionType::detect_from_slice(buf) {
        Some(CompressionType::Zlib) => {
            let input = unxor_zlib(buf);
            let mut decompressor = flate2::read::ZlibDecoder::new(Cursor::new(input));
            let mut result_buf = vec![];
            decompressor.read_to_end(&mut result_buf).context("zlib")?;

            Ok(result_buf.into())
        }
        Some(c @ (CompressionType::Lz4 | CompressionType::G108Lz4)) => {
            let mut v = [0u8; 4];
            v.copy_from_slice(&buf[4..8]);
            let mut uncompressed_size = u32::from_le_bytes(v) as usize;
            let input = if c.is_g108() { unxor(buf) } else { &buf[8..] };

            let decompressed_bytes = loop {
                match lz4_flex::decompress(input, uncompressed_size) {
                    Ok(o) => break o,
                    Err(lz4_flex::block::DecompressError::OutputTooSmall { actual, expected }) => {
                        uncompressed_size = expected;
                        warn!("Adjusting LZ4 output buffer from {actual} to {expected} bytes");
                        continue;
                    }
                    Err(e) => {
                        anyhow::bail!("lz4 error: {e}");
                    }
                };
            };

            Ok(decompressed_bytes.into())
        }
        Some(CompressionType::Lzma) => {
            let mut v = [0u8; 4];
            v.copy_from_slice(&buf[4..8]);
            let uncompressed_size = u32::from_le_bytes(v) as usize;

            let mut reader = std::io::Cursor::new(&buf[8..]);
            let option = lzma_rs::decompress::Options {
                unpacked_size: lzma_rs::decompress::UnpackedSize::UseProvided(Some(
                    uncompressed_size as u64,
                )),
                memlimit: None,
                allow_incomplete: false,
            };
            let mut decompressed = Vec::new();
            lzma_rs::lzma_decompress_with_options(&mut reader, &mut decompressed, &option)?;
            Ok(decompressed.into())
        }
        Some(c @ (CompressionType::Zstd | CompressionType::G108Zstd)) => {
            let mut v = [0u8; 4];
            v.copy_from_slice(&buf[4..8]);
            let uncompressed_size = u32::from_le_bytes(v);
            let input = if c.is_g108() { unxor(buf) } else { &buf[8..] };

            let mut out_buf = vec![];
            let mut decompressor = zstd::stream::Decoder::new(Cursor::new(input))?;
            let decompressed_bytes = match decompressor.read_to_end(&mut out_buf) {
                Ok(o) => o,
                Err(e) => {
                    anyhow::bail!("zstd read error: {e}");
                }
            };
            ensure!(
                decompressed_bytes == uncompressed_size as usize,
                "zstd: Decompressed size does not match expected size"
            );

            Ok(out_buf.into())
        }
        Some(CompressionType::None) => Ok(Cow::Borrowed(&buf[4..])),
        None => Ok(Cow::Borrowed(buf)),
    }
}

const XOR_KEY: &[u8] = &[
    0xA1, 0xBB, 0x22, 0x24, 0x40, 0x59, 0x4B, 0xE9, 0x7B, 0x38, 0x34, 0x7C, 0xB8, 0x5C, 0x13, 0xC2,
    0xA0, 0x31, 0x34, 0x79, 0xF8, 0x52, 0xF2, 0xD1, 0xED, 0xC8, 0x62, 0x86, 0x12, 0xF0, 0x4B, 0x97,
];

/// Applies the ZSTD/LZ4 flavor of the XOR encryption to the given buffer.
///
/// Pass in the data __with__ the identifier+size header. This function will return a slice that can be passed to the decompressor.
fn unxor(buf: &mut [u8]) -> &[u8] {
    let xor_size = (buf.len() - 8).clamp(0, 256);
    for (i, x) in buf[8..8 + xor_size].iter_mut().enumerate() {
        // New encrypted used since the beta
        *x = !(*x ^ XOR_KEY[i % XOR_KEY.len()]);

        // Old encryption used in the alpha
        // *x ^= 0x5E;
    }

    &buf[8..]
}

/// Applies the ZLIB flavor of the XOR encryption to the given buffer.
///
/// Pass in the data __with__ the identifier+size header. This function will return a slice that can be passed to the decompressor.
fn unxor_zlib(buf: &mut [u8]) -> &[u8] {
    let offset = (buf.len() - 8) % 37;
    let end = 128 - offset;
    let end = end.min(buf.len());
    let head = &mut buf[..end];
    for x in head.iter_mut() {
        *x ^= 0x3A;
    }
    let end = if end == buf.len() { end } else { buf.len() - 8 };

    &buf[..end]
}
