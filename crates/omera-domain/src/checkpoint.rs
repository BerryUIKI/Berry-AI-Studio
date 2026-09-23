use serde::{Deserialize, Serialize};

/// Checkpoint model statistics and hash mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointModelStat {
    /// Model name or filename.
    pub model_name: String,
    /// Model hash (short or sha256), if available.
    pub model_hash: Option<String>,
    /// Number of indexed images using this model.
    pub count: usize,
}

/// Checkpoint cache mapping entry (e.g. from A1111 cache.json).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCacheEntry {
    pub hash: String,
    pub name: String,
    pub title: Option<String>,
    pub sha256: Option<String>,
}
