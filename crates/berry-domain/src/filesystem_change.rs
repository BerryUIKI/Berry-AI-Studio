//! Persisted filesystem changes awaiting targeted reconciliation.

use serde::{Deserialize, Serialize};

/// One coalesced watcher event for a registered folder path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemChange {
    pub folder_id: i64,
    pub path: String,
    pub event_kind: String,
    pub observed_at: i64,
}
