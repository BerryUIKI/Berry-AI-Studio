use serde::{Deserialize, Serialize};

/// LoRA model entry stored in the local catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoraModel {
    pub id: i64,
    /// Canonical name of the LoRA (e.g. 'detail_tweaker' or 'anime_outline_xl')
    pub name: String,
    /// Model hash (Civitai short hash or full SHA256)
    pub hash: Option<String>,
    /// List of trigger words / activation tags associated with this LoRA
    pub trigger_words: Vec<String>,
    /// Optional preview image URL or local path
    pub preview_url: Option<String>,
    /// Description or author notes
    pub description: Option<String>,
    /// Default or recommended weight (usually 0.5 - 1.0)
    pub weight_default: f64,
    /// ISO timestamp of creation
    pub created_at: String,
    /// ISO timestamp of last update
    pub updated_at: String,
}

/// A LoRA instance detected in an image prompt or workflow graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectedLora {
    /// Name extracted from the prompt or node (without path or extension)
    pub name: String,
    /// Weight/multiplier applied (e.g. 0.8)
    pub weight: f64,
    /// Optional match in the local LoRA database (if known)
    pub model: Option<LoraModel>,
}
