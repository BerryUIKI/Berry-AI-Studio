//! Domain types for AIGC Ingestion Pipelines and delayed cleanup queue.

use serde::{Deserialize, Serialize};

/// An auto-detected local AI image generator output location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineDetectedPath {
    /// Name of the generator tool, e.g. "Stable Diffusion WebUI", "ComfyUI", "Fooocus".
    pub tool_name: String,
    /// Absolute path on the local filesystem.
    pub path: String,
    /// Output category or subfolder name (e.g. "txt2img-images", "output").
    pub category: String,
}

/// An item in the delayed cleanup queue waiting to be safely moved to trash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupQueueItem {
    /// Unique database record ID.
    pub id: i64,
    /// Absolute path of the original source file.
    pub source_file_path: String,
    /// Target image ID in Berry AI Studio's library.
    pub target_image_id: i64,
    /// Scheduled deletion time in Unix epoch seconds.
    pub scheduled_delete_at: i64,
    /// Record creation timestamp in Unix epoch seconds.
    pub created_at: i64,
    /// Status: "pending", "deleted", "cancelled", or "failed".
    pub status: String,
}
