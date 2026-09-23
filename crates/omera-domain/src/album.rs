use serde::{Deserialize, Serialize};

/// An album used to organize collections of media files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Album {
    /// Database row ID.
    pub id: i64,
    /// User-visible unique name of the album.
    pub name: String,
    /// Optional description or notes for the album.
    pub description: Option<String>,
    /// Creation timestamp (ISO-8601 string).
    pub created_at: String,
}
