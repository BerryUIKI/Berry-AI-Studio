use serde::{Deserialize, Serialize};

/// Statistics describing SQLite database status and table metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub file_count: i64,
    pub folder_count: i64,
    pub album_count: i64,
    pub tag_count: i64,
    pub model_cache_count: i64,
    pub db_size_bytes: u64,
    pub page_size: i64,
    pub page_count: i64,
    pub freelist_count: i64,
}
