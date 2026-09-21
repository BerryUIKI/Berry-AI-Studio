use serde::{Deserialize, Serialize};

/// Target image container format for export transcoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// Keep the source image container format (no re-encoding if no resizing/stripping).
    Original,
    /// Encode as modern WebP.
    #[default]
    Webp,
    /// Encode as JPEG.
    Jpeg,
    /// Encode as PNG.
    Png,
}

/// Level of metadata sanitization applied during export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MetadataPrivacyMode {
    /// Retain all embedded generation metadata, parameters, and EXIF.
    #[default]
    KeepAll,
    /// Remove prompt and negative prompt text, but keep technical parameters (sampler, seed, model).
    StripPromptOnly,
    /// Remove all AI generation metadata chunks (A1111, ComfyUI, InvokeAI, NovelAI).
    StripAllAiMetadata,
    /// Complete sanitization: write clean image pixels only without EXIF, ICC, or metadata chunks.
    StripAll,
}

/// Optional sidecar file generated alongside each exported image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExportSidecar {
    /// Do not generate sidecar files.
    #[default]
    None,
    /// Generate a `.txt` file containing the positive prompt.
    TextPrompt,
    /// Generate a `.json` file containing full generation metadata and tags.
    JsonMetadata,
}

/// Configuration parameters for a batch export job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportOptions {
    /// IDs of files to export from the database.
    pub file_ids: Vec<i64>,
    /// Target image format.
    pub format: ExportFormat,
    /// Compression quality (1..=100) for lossy formats (JPEG/WebP).
    pub quality: u8,
    /// Privacy and metadata scrubbing mode.
    pub privacy: MetadataPrivacyMode,
    /// Optional sidecar generation.
    pub sidecar: ExportSidecar,
    /// Filename template string (e.g. "{name}", "{date}_{name}", "{model}_{seed}_{name}").
    pub filename_template: String,
    /// Destination folder path or ZIP file path on the local filesystem.
    pub destination_path: String,
    /// If true, package all exported items into a single .zip archive.
    pub as_zip: bool,
    /// Optional maximum bounding edge (in pixels) for downscaling.
    pub max_edge: Option<u32>,
}

/// Real-time progress update emitted during batch export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportProgressEvent {
    pub current: usize,
    pub total: usize,
    pub current_filename: String,
}

/// Outcome summary of a completed batch export job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSummary {
    pub success: bool,
    pub total_exported: usize,
    pub total_failed: usize,
    pub total_bytes_written: u64,
    pub duration_ms: u64,
    pub output_path: String,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_options_serde_roundtrip() {
        let options = ExportOptions {
            file_ids: vec![101, 102, 103],
            format: ExportFormat::Webp,
            quality: 85,
            privacy: MetadataPrivacyMode::StripAllAiMetadata,
            sidecar: ExportSidecar::TextPrompt,
            filename_template: "{date}_{name}".to_string(),
            destination_path: "C:\\Exports\\batch.zip".to_string(),
            as_zip: true,
            max_edge: Some(2048),
        };

        let json = serde_json::to_string(&options).unwrap();
        let decoded: ExportOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, options);
    }

    #[test]
    fn export_summary_serde_roundtrip() {
        let summary = ExportSummary {
            success: true,
            total_exported: 42,
            total_failed: 0,
            total_bytes_written: 10485760,
            duration_ms: 320,
            output_path: "/output/bundle.zip".to_string(),
            errors: Vec::new(),
        };

        let json = serde_json::to_string(&summary).unwrap();
        let decoded: ExportSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, summary);
    }
}
