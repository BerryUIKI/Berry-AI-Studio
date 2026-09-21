//! Magic-byte detection of media container formats.
//!
//! Detection is based on the well-known file signatures:
//!
//! - PNG:   `\x89PNG\r\n\x1a\n`
//! - JPEG:  `\xFF\xD8\xFF`
//! - WebP:  `RIFF` + `WEBP` at offset 8
//! - MP4:   `ftyp` at offset 4 (ISO base media file format family)
//! - TXT:   UTF-8 text with a reasonable likelihood of being a sidecar

use berry_domain::Container;

/// PNG signature (8 bytes).
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
/// JPEG starts with the SOI marker `FF D8 FF`.
const JPEG_SIGNATURE: [u8; 3] = [0xFF, 0xD8, 0xFF];
/// Bytes 0..4 of a RIFF file.
const RIFF_SIGNATURE: [u8; 4] = *b"RIFF";
/// Bytes 8..12 of a WebP file.
const WEBP_SIGNATURE: [u8; 4] = *b"WEBP";
/// WebM / Matroska EBML signature `1A 45 DF A3`.
const WEBM_SIGNATURE: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];

/// Detect the container format of a file from its leading bytes.
///
/// Returns `None` when the signature is not recognized. The implementation
/// never reads past the first 12 bytes, so passing a partial read (e.g. a
/// truncated file) is safe — it just yields `None` for anything incomplete.
pub fn detect_container(bytes: &[u8]) -> Option<Container> {
    if bytes.len() >= PNG_SIGNATURE.len() && bytes[..PNG_SIGNATURE.len()] == PNG_SIGNATURE {
        return Some(Container::Png);
    }

    if bytes.len() >= JPEG_SIGNATURE.len() && bytes[..JPEG_SIGNATURE.len()] == JPEG_SIGNATURE {
        return Some(Container::Jpeg);
    }

    if bytes.len() >= 12 && bytes[..4] == RIFF_SIGNATURE && bytes[8..12] == WEBP_SIGNATURE {
        return Some(Container::WebP);
    }

    if bytes.len() >= 8 && bytes[4..8] == *b"ftyp" {
        return Some(Container::Mp4);
    }

    if bytes.len() >= WEBM_SIGNATURE.len() && bytes[..WEBM_SIGNATURE.len()] == WEBM_SIGNATURE {
        return Some(Container::Webm);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_png_signature() {
        let mut bytes = PNG_SIGNATURE.to_vec();
        bytes.extend_from_slice(b"\x00\x00\x00\rIHDR");
        assert_eq!(detect_container(&bytes), Some(Container::Png));
    }

    #[test]
    fn detects_jpeg_signature() {
        let bytes = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_container(&bytes), Some(Container::Jpeg));
    }

    #[test]
    fn detects_webp_signature() {
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 0]);
        bytes.extend_from_slice(b"WEBP");
        bytes.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]);
        assert_eq!(detect_container(&bytes), Some(Container::WebP));
    }

    #[test]
    fn detects_mp4_signature() {
        let mut bytes = b"\x00\x00\x00\x18ftypmp42".to_vec();
        bytes.extend_from_slice(b"\x00\x00\x00\x00");
        assert_eq!(detect_container(&bytes), Some(Container::Mp4));
    }

    #[test]
    fn detects_webm_signature() {
        let mut bytes = vec![0x1A, 0x45, 0xDF, 0xA3];
        bytes.extend_from_slice(
            b"\x9F\x42\x86\x81\x01\x42\xF7\x81\x01\x42\xF2\x81\x04\x42\xF3\x81\x08\x42\x82\x84webm",
        );
        assert_eq!(detect_container(&bytes), Some(Container::Webm));
    }

    #[test]
    fn rejects_unknown_and_truncated_input() {
        assert_eq!(detect_container(b""), None);
        assert_eq!(detect_container(b"\x89PN"), None);
        assert_eq!(detect_container(b"RIFF"), None); // truncated WebP
        assert_eq!(detect_container(b"not a known format at all"), None);
    }

    #[test]
    fn pdf_does_not_match() {
        let bytes = b"%PDF-1.7 ...";
        assert_eq!(detect_container(bytes), None);
    }
}
