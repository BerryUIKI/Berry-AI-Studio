//! A user-added folder that the app indexes.

use serde::{Deserialize, Serialize};

/// A folder the user has asked the app to index.
///
/// Stored in the `folders` table. Its scanned files live in
/// `files` and are cascade-deleted when the folder is removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    /// Database row id.
    pub id: i64,
    /// Absolute path of the folder.
    pub path: String,
    /// When the folder was added, as an ISO-8601 UTC timestamp.
    pub added_at: String,
    /// Operating mode: 'link' (default), 'managed', or 'pipeline'.
    #[serde(default = "default_folder_type")]
    pub folder_type: String,
    /// For pipeline: source output path being watched.
    pub source_path: Option<String>,
    /// For pipeline: ingest action: 'copy' or 'move'.
    pub ingest_action: Option<String>,
    /// For pipeline: grace period in hours before source file is moved to trash.
    pub grace_period_hours: Option<i32>,
    /// For pipeline: whether real-time / auto-harvest is active.
    #[serde(default = "default_auto_harvest")]
    pub auto_harvest: bool,
}

fn default_folder_type() -> String {
    "link".to_string()
}

fn default_auto_harvest() -> bool {
    true
}
