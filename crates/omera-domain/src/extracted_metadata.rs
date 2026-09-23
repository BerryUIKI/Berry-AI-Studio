//! Structured metadata extracted from a media file's embedded info.

use serde::{Deserialize, Serialize};

use crate::MetadataFormat;

/// Metadata extracted from an image/video, normalized from whatever the
/// generator embedded (PNGInfo text chunks, EXIF, `.txt` sidecars).
///
/// Serialized to a JSON `metadata` column on the `files` row. `parameters`
/// keeps the raw generator string (e.g. the A1111 "parameters" chunk) so the
/// original text survives even as structured fields are added.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExtractedMetadata {
    /// The generator platform this metadata came from.
    pub format: MetadataFormat,
    /// Raw A1111/SD.Next-style parameter string, if present.
    pub parameters: Option<String>,
    /// Any other raw text (e.g. a ComfyUI `prompt`/`workflow` chunk or the
    /// full contents of a `.txt` sidecar).
    pub raw: Option<String>,
    /// The generation prompt.
    pub prompt: Option<String>,
    /// The negative prompt, if any.
    pub negative_prompt: Option<String>,
    /// Image width in pixels.
    pub width: Option<u32>,
    /// Image height in pixels.
    pub height: Option<u32>,
    /// Seed (kept as a string: seeds can be 64-bit values or hashes).
    pub seed: Option<String>,
    /// Number of sampling steps.
    pub steps: Option<u32>,
    /// CFG scale.
    pub cfg_scale: Option<f64>,
    /// Sampler name.
    pub sampler: Option<String>,
    /// Checkpoint model name.
    pub model_name: Option<String>,
    /// Checkpoint model hash.
    pub model_hash: Option<String>,
    /// Video duration in seconds (for video media).
    #[serde(default)]
    pub duration_seconds: Option<f64>,
    /// Video frame rate in frames per second (for video media).
    #[serde(default)]
    pub fps: Option<f64>,
    /// Video codec identifier (e.g. h264, hevc, vp9, av1).
    #[serde(default)]
    pub video_codec: Option<String>,
}
