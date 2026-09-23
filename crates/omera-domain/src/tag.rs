use serde::{Deserialize, Serialize};

/// A custom user tag for categorizing images.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// Database row ID.
    pub id: i64,
    /// Tag label (unique).
    pub name: String,
    /// Optional color code (e.g. hex `#3b82f6`) for UI rendering.
    pub color: Option<String>,
    /// Creation timestamp (ISO-8601 string).
    pub created_at: String,
}
