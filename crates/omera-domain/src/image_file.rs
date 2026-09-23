//! The media container of a scanned file.

use serde::{Deserialize, Serialize};

/// The file container format of a scanned media file.
///
/// Detection by magic bytes lives in `berry-metadata`; this type only
/// describes the possible outcomes so the rest of the app can branch on it
/// (e.g. "only PNG/JPEG/WebP carry embedded PNGInfo/EXIF metadata").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Container {
    /// Portable Network Graphics (`.png`).
    Png,
    /// JPEG / JFIF (`.jpg`, `.jpeg`).
    Jpeg,
    /// WebP (`.webp`).
    WebP,
    /// MP4 (`.mp4`).
    Mp4,
    /// WebM (`.webm`).
    Webm,
    /// Plain text (`.txt`) sidecar metadata file.
    Txt,
}

impl Container {
    /// Whether this container holds an image (rather than a video or text).
    pub const fn is_image(self) -> bool {
        matches!(self, Self::Png | Self::Jpeg | Self::WebP)
    }

    /// Whether this container holds a video.
    pub const fn is_video(self) -> bool {
        matches!(self, Self::Mp4 | Self::Webm)
    }

    /// A canonical file extension for the container, without the leading dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Mp4 => "mp4",
            Self::Webm => "webm",
            Self::Txt => "txt",
        }
    }

    /// A stable identifier used for database storage and display.
    pub const fn id(self) -> &'static str {
        self.extension()
    }

    /// Look up a container by its stable [`id`](Self::id).
    pub fn from_id(id: &str) -> Option<Container> {
        match id {
            "png" => Some(Self::Png),
            "jpg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            "mp4" => Some(Self::Mp4),
            "webm" => Some(Self::Webm),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }
}

/// A single media file discovered by a scan and persisted to the `files` row.
///
/// `id` is `None` for a file that has not been inserted yet; the scanner
/// builds these from disk metadata and `Database::upsert_file` fills in the id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageFile {
    /// Database row id, `None` until the file is persisted.
    pub id: Option<i64>,
    /// Id of the [`Folder`](crate::Folder) this file was scanned from.
    pub folder_id: i64,
    /// Absolute path of the file on disk.
    pub path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Filesystem modification time as unix seconds; the incremental-scan
    /// cache compares (size, mtime) to skip unchanged files.
    pub modified_at: i64,
    /// Detected container format.
    pub container: Container,
    /// Extracted generation metadata, `None` until extraction runs.
    pub metadata: Option<crate::ExtractedMetadata>,
    /// User rating from 1 to 10, if set.
    pub rating: Option<u8>,
    /// Calculated or extracted aesthetic score, if available.
    pub aesthetic_score: Option<f64>,
    /// Whether this file is marked as a favorite.
    #[serde(default)]
    pub is_favorite: bool,
    /// Whether this file is flagged as NSFW.
    #[serde(default)]
    pub is_nsfw: bool,
    /// Identifier of the image stack this image belongs to, if any.
    #[serde(default)]
    pub stack_id: Option<String>,
    /// Order index within the stack (0 = Hero Cover, 1..N = members).
    #[serde(default)]
    pub stack_order: i32,
}

/// Field by which to sort a list of [`ImageFile`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileSortField {
    /// Filesystem modification timestamp (`modified_at`).
    ModifiedAt,
    /// Absolute file path (`path`).
    Path,
    /// File size in bytes (`size_bytes`).
    SizeBytes,
    /// User rating (`rating`).
    Rating,
    /// Aesthetic score (`aesthetic_score`).
    AestheticScore,
}

/// Direction of sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    /// Ascending (low to high, oldest to newest, A-Z).
    Asc,
    /// Descending (high to low, newest to oldest, Z-A).
    Desc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_are_not_videos() {
        for container in [Container::Png, Container::Jpeg, Container::WebP] {
            assert!(container.is_image());
            assert!(!container.is_video());
        }
    }

    #[test]
    fn mp4_is_a_video() {
        assert!(Container::Mp4.is_video());
        assert!(!Container::Mp4.is_image());
        assert!(Container::Webm.is_video());
        assert!(!Container::Webm.is_image());
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&Container::WebP).unwrap();
        let back: Container = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Container::WebP);
    }

    #[test]
    fn container_id_roundtrips() {
        for container in [
            Container::Png,
            Container::Jpeg,
            Container::WebP,
            Container::Mp4,
            Container::Webm,
            Container::Txt,
        ] {
            assert_eq!(Container::from_id(container.id()), Some(container));
        }
        assert_eq!(Container::from_id("bmp"), None);
    }

    #[test]
    fn ids_are_also_extensions() {
        assert_eq!(Container::Jpeg.id(), "jpg");
        assert_eq!(Container::Jpeg.extension(), "jpg");
    }

    #[test]
    fn sort_enums_serde_roundtrip() {
        let field_json = serde_json::to_string(&FileSortField::AestheticScore).unwrap();
        assert_eq!(field_json, "\"aesthetic_score\"");
        let back_field: FileSortField = serde_json::from_str(&field_json).unwrap();
        assert_eq!(back_field, FileSortField::AestheticScore);

        let dir_json = serde_json::to_string(&SortDirection::Desc).unwrap();
        assert_eq!(dir_json, "\"desc\"");
        let back_dir: SortDirection = serde_json::from_str(&dir_json).unwrap();
        assert_eq!(back_dir, SortDirection::Desc);
    }
}
