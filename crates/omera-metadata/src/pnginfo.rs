//! PNG text-chunk metadata (PNGInfo) parsing.
//!
//! AUTOMATIC1111 / SD.Next embed generation parameters in a `tEXt` chunk with
//! keyword `parameters`; ComfyUI embeds its JSON as uncompressed `iTXt` chunks
//! (`prompt`, `workflow`). This module walks a PNG's chunk stream with no
//! external dependencies and returns the decodable text chunks.
//!
//! Compression (zTXt / iTXt with the compression flag set) is skipped — the
//! generators we target store their metadata uncompressed.

use std::fmt;

/// PNG signature (8 bytes): `\x89PNG\r\n\x1a\n`.
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Maximum allowed length (in bytes) for a PNG text chunk (64 MB).
/// Protects against memory exhaustion / DoS when encountering malformed or crafted files.
pub const MAX_TEXT_CHUNK_LEN: usize = 64 * 1024 * 1024;

/// Errors produced while walking a PNG's chunk stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PngError {
    /// The input does not start with the PNG signature.
    BadSignature,
    /// The input ends before a chunk's declared length could be read.
    Truncated,
    /// A `tEXt`/`iTXt` chunk is missing the keyword/length separator.
    InvalidText,
}

impl fmt::Display for PngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadSignature => f.write_str("input is not a PNG file"),
            Self::Truncated => f.write_str("PNG chunk stream is truncated"),
            Self::InvalidText => f.write_str("PNG text chunk is malformed"),
        }
    }
}

impl std::error::Error for PngError {}

/// One decodable text chunk (`tEXt` or uncompressed `iTXt`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PngTextChunk {
    /// The chunk keyword, e.g. `parameters`, `Comment`, `prompt`.
    pub keyword: String,
    /// The chunk text payload.
    pub text: String,
}

use std::io::{Read, Seek, SeekFrom};

/// Walk a PNG's chunk stream from a reader and return every decodable text chunk,
/// in file order. Seeks past non-text chunks (such as IDAT pixel data) to avoid
/// reading large images into RAM. Stops at `IEND`.
pub fn read_text_chunks_from_reader<R: Read + Seek>(
    mut reader: R,
) -> Result<Vec<PngTextChunk>, PngError> {
    let mut sig = [0u8; 8];
    reader
        .read_exact(&mut sig)
        .map_err(|_| PngError::BadSignature)?;
    if sig != PNG_SIGNATURE {
        return Err(PngError::BadSignature);
    }

    let mut chunks = Vec::new();
    let mut header = [0u8; 8];

    while reader.read_exact(&mut header).is_ok() {
        let length = u32::from_be_bytes(header[0..4].try_into().unwrap()) as usize;
        let chunk_type = &header[4..8];

        match chunk_type {
            b"IEND" => break,
            b"tEXt" => {
                if length > MAX_TEXT_CHUNK_LEN {
                    let skip = (length as u64).saturating_add(4);
                    reader
                        .seek(SeekFrom::Current(skip as i64))
                        .map_err(|_| PngError::Truncated)?;
                    continue;
                }
                let mut data = vec![0u8; length];
                reader
                    .read_exact(&mut data)
                    .map_err(|_| PngError::Truncated)?;
                // Read and discard 4-byte CRC
                let mut crc = [0u8; 4];
                reader
                    .read_exact(&mut crc)
                    .map_err(|_| PngError::Truncated)?;
                chunks.push(parse_text(&data)?);
            }
            b"iTXt" => {
                if length > MAX_TEXT_CHUNK_LEN {
                    let skip = (length as u64).saturating_add(4);
                    reader
                        .seek(SeekFrom::Current(skip as i64))
                        .map_err(|_| PngError::Truncated)?;
                    continue;
                }
                let mut data = vec![0u8; length];
                reader
                    .read_exact(&mut data)
                    .map_err(|_| PngError::Truncated)?;
                // Read and discard 4-byte CRC
                let mut crc = [0u8; 4];
                reader
                    .read_exact(&mut crc)
                    .map_err(|_| PngError::Truncated)?;
                if let Some(chunk) = parse_itxt(&data)? {
                    chunks.push(chunk);
                }
            }
            _ => {
                // Seek past chunk data and 4-byte CRC
                let skip = (length as u64).saturating_add(4);
                reader
                    .seek(SeekFrom::Current(skip as i64))
                    .map_err(|_| PngError::Truncated)?;
            }
        }
    }

    Ok(chunks)
}

/// Walk a PNG's chunk stream and return every decodable text chunk, in file
/// order. Stops at `IEND`; compressed chunks and non-text chunks are ignored.
pub fn text_chunks(bytes: &[u8]) -> Result<Vec<PngTextChunk>, PngError> {
    if bytes.len() < PNG_SIGNATURE.len() || bytes[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err(PngError::BadSignature);
    }

    let mut chunks = Vec::new();
    let mut offset = PNG_SIGNATURE.len();
    let total = bytes.len();

    while offset < total {
        // Each chunk: 4-byte big-endian length, 4-byte type, `length` data
        // bytes, 4-byte CRC.
        if total.saturating_sub(offset) < 8 {
            return Err(PngError::Truncated);
        }
        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        let chunk_type = &bytes[offset + 4..offset + 8];
        let data_start = offset + 8;
        if total.saturating_sub(data_start) < length.saturating_add(4) {
            return Err(PngError::Truncated);
        }
        let data = &bytes[data_start..data_start + length];

        match chunk_type {
            b"IEND" => break,
            b"tEXt" if length <= MAX_TEXT_CHUNK_LEN => chunks.push(parse_text(data)?),
            b"iTXt" if length <= MAX_TEXT_CHUNK_LEN => {
                if let Some(chunk) = parse_itxt(data)? {
                    chunks.push(chunk);
                }
            }
            // zTXt, compressed iTXt, oversized, and non-text chunks are skipped.
            _ => {}
        }

        offset = data_start + length + 4;
    }

    Ok(chunks)
}

/// Return the text of the `parameters` chunk (A1111 / SD.Next PNGInfo), if any.
pub fn extract_parameters(bytes: &[u8]) -> Option<String> {
    text_chunks(bytes)
        .ok()?
        .into_iter()
        .find(|chunk| chunk.keyword == "parameters")
        .map(|chunk| chunk.text)
}

/// Parse a `tEXt` chunk payload: `keyword \0 text` (both latin-1).
fn parse_text(data: &[u8]) -> Result<PngTextChunk, PngError> {
    let nul = data
        .iter()
        .position(|&b| b == 0)
        .ok_or(PngError::InvalidText)?;
    Ok(PngTextChunk {
        keyword: latin1(&data[..nul]),
        text: latin1(&data[nul + 1..]),
    })
}

/// Parse an `iTXt` chunk payload: `keyword \0 flag \0 method \0 lang \0
/// translated \0 text`. Returns `Ok(None)` for compressed text (skipped).
fn parse_itxt(data: &[u8]) -> Result<Option<PngTextChunk>, PngError> {
    // keyword
    let nul = data
        .iter()
        .position(|&b| b == 0)
        .ok_or(PngError::InvalidText)?;
    let keyword = latin1(&data[..nul]);
    let rest = &data[nul + 1..];
    // compression flag (1 byte) and method (1 byte)
    if rest.len() < 2 {
        return Err(PngError::InvalidText);
    }
    let compression_flag = rest[0];
    let _compression_method = rest[1];
    let rest = &rest[2..];
    if compression_flag != 0 {
        return Ok(None); // zlib-compressed iTXt, skip
    }
    // language tag and translated keyword, each NUL-terminated
    let (_lang, rest) = split_nul(rest).ok_or(PngError::InvalidText)?;
    let (_translated, text) = split_nul(rest).ok_or(PngError::InvalidText)?;
    Ok(Some(PngTextChunk {
        keyword,
        text: String::from_utf8_lossy(text).into_owned(),
    }))
}

/// Split `data` at the first NUL byte.
fn split_nul(data: &[u8]) -> Option<(&[u8], &[u8])> {
    let nul = data.iter().position(|&b| b == 0)?;
    Some((&data[..nul], &data[nul + 1..]))
}

/// Decode latin-1 (ISO 8859-1) bytes to Unicode by mapping each byte 1:1.
fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIGNATURE: [u8; 8] = PNG_SIGNATURE;

    /// Build a PNG chunk (CRC is not validated by the walker, so zeros are fine).
    fn chunk(chunk_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(chunk_type);
        out.extend_from_slice(data);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn png(chunks: &[&[u8]]) -> Vec<u8> {
        let mut out = SIGNATURE.to_vec();
        for c in chunks {
            out.extend_from_slice(c);
        }
        out
    }

    fn ihdr() -> Vec<u8> {
        chunk(b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0])
    }

    fn iend() -> Vec<u8> {
        chunk(b"IEND", &[])
    }

    fn tex(keyword: &str, text: &str) -> Vec<u8> {
        let mut data = keyword.as_bytes().to_vec();
        data.push(0);
        data.extend_from_slice(text.as_bytes());
        chunk(b"tEXt", &data)
    }

    fn itxt(keyword: &str, text: &str, compressed: bool) -> Vec<u8> {
        let mut data = keyword.as_bytes().to_vec();
        data.push(0);
        data.push(if compressed { 1 } else { 0 });
        data.push(0); // compression method
        data.push(0); // empty language tag
        data.push(0); // empty translated keyword
        data.extend_from_slice(text.as_bytes());
        chunk(b"iTXt", &data)
    }

    #[test]
    fn walks_tex_text_chunks() {
        let file = png(&[
            &ihdr(),
            &tex("parameters", "Steps: 20, Sampler: Euler"),
            &tex("Comment", "hi there"),
            &iend(),
        ]);
        let chunks = text_chunks(&file).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].keyword, "parameters");
        assert_eq!(chunks[0].text, "Steps: 20, Sampler: Euler");
        assert_eq!(chunks[1].keyword, "Comment");
        assert_eq!(chunks[1].text, "hi there");
    }

    #[test]
    fn extracts_a1111_parameters() {
        let file = png(&[
            &ihdr(),
            &tex("parameters", "a cat\nSteps: 20, Seed: 42, Size: 512x768"),
            &iend(),
        ]);
        assert_eq!(
            extract_parameters(&file).as_deref(),
            Some("a cat\nSteps: 20, Seed: 42, Size: 512x768")
        );
    }

    #[test]
    fn reads_uncompressed_itxt() {
        // ComfyUI-style `prompt` iTXt chunk.
        let file = png(&[&ihdr(), &itxt("prompt", r#"{"nodes":[]}"#, false), &iend()]);
        let chunks = text_chunks(&file).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].keyword, "prompt");
        assert_eq!(chunks[0].text, r#"{"nodes":[]}"#);
    }

    #[test]
    fn skips_compressed_itxt() {
        let file = png(&[&ihdr(), &itxt("parameters", "secret", true), &iend()]);
        assert!(text_chunks(&file).unwrap().is_empty());
    }

    #[test]
    fn decodes_latin1_text() {
        let data = b"Comment\x00caf\xE9".to_vec();
        let c = chunk(b"tEXt", &data);
        let file = png(&[&c, &iend()]);
        let chunks = text_chunks(&file).unwrap();
        assert_eq!(chunks[0].text, "café");
    }

    #[test]
    fn stops_at_iend() {
        let file = png(&[
            &ihdr(),
            &tex("parameters", "first"),
            &iend(),
            &tex("parameters", "after"),
        ]);
        let chunks = text_chunks(&file).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "first");
    }

    #[test]
    fn rejects_bad_signature() {
        assert_eq!(
            text_chunks(b"not a png").unwrap_err(),
            PngError::BadSignature
        );
        assert_eq!(text_chunks(b"").unwrap_err(), PngError::BadSignature);
    }

    #[test]
    fn rejects_truncated_chunk() {
        // Declares a 100-byte chunk but provides none.
        let mut file = SIGNATURE.to_vec();
        file.extend_from_slice(&100u32.to_be_bytes());
        file.extend_from_slice(b"tEXt");
        assert_eq!(text_chunks(&file).unwrap_err(), PngError::Truncated);
    }

    #[test]
    fn rejects_text_without_keyword_separator() {
        // tEXt payload with no NUL separator.
        let c = chunk(b"tEXt", b"no separator here");
        let file = png(&[&c, &iend()]);
        assert_eq!(text_chunks(&file).unwrap_err(), PngError::InvalidText);
    }

    #[test]
    fn empty_text_is_kept() {
        let file = png(&[&ihdr(), &tex("Comment", ""), &iend()]);
        let chunks = text_chunks(&file).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].keyword, "Comment");
        assert_eq!(chunks[0].text, "");
    }

    #[test]
    fn missing_parameters_chunk_returns_none() {
        let file = png(&[&ihdr(), &tex("Comment", "x"), &iend()]);
        assert_eq!(extract_parameters(&file), None);
    }

    #[test]
    fn reads_text_chunks_from_reader_skipping_idat() {
        let idat_data = vec![0x78, 0x9c, 0x63, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01];
        let file = png(&[
            &ihdr(),
            &tex("parameters", "prompt before idat"),
            &chunk(b"IDAT", &idat_data),
            &tex("Comment", "comment after idat"),
            &iend(),
        ]);
        let cursor = std::io::Cursor::new(file);
        let chunks = read_text_chunks_from_reader(cursor).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].keyword, "parameters");
        assert_eq!(chunks[0].text, "prompt before idat");
        assert_eq!(chunks[1].keyword, "Comment");
        assert_eq!(chunks[1].text, "comment after idat");
    }

    #[test]
    fn rejects_oversized_text_chunk_without_oom() {
        // Declares a 2GB chunk without providing the payload
        let mut file = SIGNATURE.to_vec();
        file.extend_from_slice(&2_000_000_000u32.to_be_bytes());
        file.extend_from_slice(b"tEXt");
        let cursor = std::io::Cursor::new(file);
        // It must NOT allocate 2GB and OOM; it must safely seek past and return Ok([])
        let chunks = read_text_chunks_from_reader(cursor).unwrap();
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn text_chunks_skips_oversized_chunks() {
        // A chunk with declared length > MAX_TEXT_CHUNK_LEN
        let mut file = SIGNATURE.to_vec();
        file.extend_from_slice(&ihdr());
        // Add oversized chunk header
        let oversized_len = (MAX_TEXT_CHUNK_LEN as u32) + 100;
        file.extend_from_slice(&oversized_len.to_be_bytes());
        file.extend_from_slice(b"tEXt");
        // Because the buffer doesn't have `oversized_len` bytes, text_chunks returns Truncated
        assert_eq!(text_chunks(&file).unwrap_err(), PngError::Truncated);
    }
}
