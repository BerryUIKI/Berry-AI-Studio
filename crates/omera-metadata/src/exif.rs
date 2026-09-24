//! EXIF metadata extraction for JPEG / WebP images.
//!
//! AI image generators commonly write a `Software` EXIF tag naming the
//! generator (e.g. "ComfyUI", "NovelAI", "Stable Diffusion WebUI"). We read the
//! dimensions and that `Software` value, and only claim metadata when it names a
//! generator we recognize — plain camera photos (which carry `Make`/`Model`
//! instead) are left alone.

use std::fmt;
use std::io::Cursor;
use std::path::Path;

use exif::{In, Reader, Tag, Value};
use omera_domain::MetadataFormat;

/// Dimensions and generator name read from a file's EXIF block.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExifInfo {
    /// The `Software` tag value, if present.
    pub software: Option<String>,
    /// Image width in pixels.
    pub width: Option<u32>,
    /// Image height in pixels.
    pub height: Option<u32>,
}

/// Read EXIF info from a media file's container (JPEG APP1, WebP EXIF chunk).
///
/// Returns `Ok(None)` when the file has no EXIF block or none of the fields we
/// care about; `Err` when the container cannot be read or parsed as EXIF.
pub fn read_exif(path: &Path) -> Result<Option<ExifInfo>, ExifError> {
    let bytes = std::fs::read(path).map_err(ExifError::Io)?;
    let mut cursor = Cursor::new(bytes);
    let exif = match Reader::new().read_from_container(&mut cursor) {
        Ok(exif) => exif,
        // A well-formed JPEG/WebP/PNG with no EXIF segment yields NotFound.
        Err(exif::Error::NotFound(_)) => return Ok(None),
        Err(e) => return Err(ExifError::Parse(e)),
    };

    let software = field_string(&exif, Tag::Software);
    let width = get_dimension(&exif, Tag::PixelXDimension)
        .or_else(|| get_dimension(&exif, Tag::ImageWidth));
    let height = get_dimension(&exif, Tag::PixelYDimension)
        .or_else(|| get_dimension(&exif, Tag::ImageLength));

    if software.is_none() && width.is_none() && height.is_none() {
        return Ok(None);
    }
    Ok(Some(ExifInfo {
        software,
        width,
        height,
    }))
}

/// Map an EXIF `Software` string to a known generator format.
///
/// Returns `None` when the software does not identify a recognized generator
/// (e.g. "Adobe Photoshop", a camera name, or an empty value).
pub fn infer_format(software: &str) -> Option<MetadataFormat> {
    let s = software.to_ascii_lowercase();
    if s.contains("comfyui") {
        Some(MetadataFormat::ComfyUI)
    } else if s.contains("novelai") {
        Some(MetadataFormat::NovelAI)
    } else if s.contains("fooocus") {
        Some(MetadataFormat::Fooocus)
    } else if s.contains("invoke") {
        Some(MetadataFormat::InvokeAI)
    } else if s.contains("easydiffusion") {
        Some(MetadataFormat::EasyDiffusion)
    } else if s.contains("swarm") {
        Some(MetadataFormat::StableSwarm)
    } else if s.contains("a1111") || s.contains("automatic1111") {
        Some(MetadataFormat::A1111)
    } else if s.contains("sd.next")
        || s.contains("stable diffusion")
        || s.contains("webui")
        || s.contains("stablediffusion")
    {
        Some(MetadataFormat::StableDiffusion)
    } else {
        None
    }
}

/// Errors produced while reading a file's EXIF block.
#[derive(Debug)]
pub enum ExifError {
    /// The file could not be read.
    Io(std::io::Error),
    /// The file has no EXIF block, or it could not be parsed.
    Parse(exif::Error),
}

impl fmt::Display for ExifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "could not read file: {e}"),
            Self::Parse(e) => write!(f, "could not parse EXIF data: {e}"),
        }
    }
}

impl std::error::Error for ExifError {}

impl From<std::io::Error> for ExifError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Read a tag's ASCII value as a string (e.g. the `Software` name).
fn field_string(exif: &exif::Exif, tag: Tag) -> Option<String> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    let bytes = match field.value {
        Value::Ascii(ref v) => v.first()?,
        _ => return None,
    };
    // ASCII values are NUL-terminated; strip the trailing NUL.
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let text = String::from_utf8_lossy(&bytes[..end]).into_owned();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Read a tag's value as an unsigned dimension.
fn get_dimension(exif: &exif::Exif, tag: Tag) -> Option<u32> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    match field.value {
        Value::Short(ref v) => v.first().map(|&n| n as u32),
        Value::Long(ref v) => v.first().copied(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal JPEG containing an EXIF APP1 segment with a `Software`
    /// string in IFD0 and `PixelXDimension`/`PixelYDimension` tags in the Exif
    /// sub-IFD (the layout real generators use).
    fn exif_jpeg(software: &str, width: u16, height: u16) -> Vec<u8> {
        // IFD0: header(8) + count(2) + 2 entries(24) + next-offset(4) = 38,
        // then the NUL-terminated software string.
        let software_offset = 38u32;
        let exif_ifd_offset = software_offset + software.len() as u32 + 1;

        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II"); // little-endian
        tiff.extend_from_slice(&0x002A_u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes()); // IFD0 offset

        tiff.extend_from_slice(&2u16.to_le_bytes()); // entry count

        // Software: tag 0x0131, type ASCII (2), value at software_offset.
        tiff.extend_from_slice(&0x0131_u16.to_le_bytes());
        tiff.extend_from_slice(&2u16.to_le_bytes());
        tiff.extend_from_slice(&(software.len() as u32 + 1).to_le_bytes());
        tiff.extend_from_slice(&software_offset.to_le_bytes());

        // ExifIFDPointer: tag 0x8769, type LONG (4), inline offset.
        tiff.extend_from_slice(&0x8769_u16.to_le_bytes());
        tiff.extend_from_slice(&4u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&exif_ifd_offset.to_le_bytes());

        tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD0 offset
        tiff.extend_from_slice(software.as_bytes());
        tiff.push(0);

        // Exif sub-IFD: PixelXDimension + PixelYDimension, both SHORT inline.
        tiff.extend_from_slice(&2u16.to_le_bytes()); // entry count
        tiff.extend_from_slice(&0xA002_u16.to_le_bytes());
        tiff.extend_from_slice(&3u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&(width as u32).to_le_bytes());
        tiff.extend_from_slice(&0xA003_u16.to_le_bytes());
        tiff.extend_from_slice(&3u16.to_le_bytes());
        tiff.extend_from_slice(&1u32.to_le_bytes());
        tiff.extend_from_slice(&(height as u32).to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD offset

        // Wrap in JPEG SOI + APP1(Exif) + EOI.
        let marker = b"Exif\0\0";
        let app1_len = 2 + marker.len() + tiff.len();
        let mut out = vec![0xFF, 0xD8]; // SOI
        out.extend_from_slice(&[0xFF, 0xE1]); // APP1
        out.extend_from_slice(&(app1_len as u16).to_be_bytes());
        out.extend_from_slice(marker);
        out.extend_from_slice(&tiff);
        out.extend_from_slice(&[0xFF, 0xD9]); // EOI
        out
    }

    #[test]
    fn reads_dimensions_and_software() {
        let dir = std::env::temp_dir().join(format!("omera-exif-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("comfy.jpg");
        std::fs::write(&path, exif_jpeg("ComfyUI", 1024, 768)).unwrap();

        let info = read_exif(&path).unwrap().expect("exif present");
        assert_eq!(info.software.as_deref(), Some("ComfyUI"));
        assert_eq!(info.width, Some(1024));
        assert_eq!(info.height, Some(768));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn non_exif_jpeg_has_no_info() {
        let dir = std::env::temp_dir().join(format!("omera-exif-none-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("plain.jpg");
        // Minimal valid JPEG (SOI + EOI, no APP1 Exif segment).
        std::fs::write(&path, [0xFF, 0xD8, 0xFF, 0xD9]).unwrap();

        assert_eq!(read_exif(&path).unwrap(), None);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn infers_generator_formats() {
        assert_eq!(infer_format("ComfyUI"), Some(MetadataFormat::ComfyUI));
        assert_eq!(infer_format("NovelAI"), Some(MetadataFormat::NovelAI));
        assert_eq!(
            infer_format("Stable Diffusion WebUI"),
            Some(MetadataFormat::StableDiffusion)
        );
        assert_eq!(
            infer_format("SD.Next"),
            Some(MetadataFormat::StableDiffusion)
        );
        assert_eq!(infer_format("A1111"), Some(MetadataFormat::A1111));
        assert_eq!(infer_format("InvokeAI"), Some(MetadataFormat::InvokeAI));
        assert_eq!(infer_format("Fooocus"), Some(MetadataFormat::Fooocus));
        assert_eq!(infer_format("Adobe Photoshop"), None);
        assert_eq!(infer_format(""), None);
    }
}
