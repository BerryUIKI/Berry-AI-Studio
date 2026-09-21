//! Collaboration, optimistic concurrency control (OCC), and change log models.
//!
//! Provides the data structures for multi-user real-time change synchronization
//! and conflict resolution across shared database instances.

use serde::{Deserialize, Serialize};

/// Represents an atomic change recorded in the shared database journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeLogEntry {
    /// Monotonically increasing unique sequence ID.
    pub id: i64,
    /// Event category: e.g. "file.rated", "file.tagged", "file.trashed", "stack.created".
    pub event_type: String,
    /// Primary entity identifier (e.g. file_id or album_id).
    pub entity_id: i64,
    /// Optional secondary identifier (e.g. stack_id or tag_name).
    pub secondary_id: Option<String>,
    /// Identifier of the client/workstation that initiated this change.
    pub client_id: String,
    /// Optional serialized JSON payload with event details.
    pub payload: Option<String>,
    /// Unix timestamp in seconds when the change was committed.
    pub created_at: i64,
}

/// Result of an optimistic concurrency control (OCC) mutation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationResult {
    /// Whether the mutation succeeded without collision.
    pub success: bool,
    /// The new version number after update, or current database version if collision occurred.
    pub current_version: i64,
    /// Number of rows modified (0 if collision detected).
    pub rows_affected: u64,
    /// Flag indicating if an optimistic concurrency conflict occurred.
    pub conflict_detected: bool,
}

/// Query parameters for fetching real-time change log journals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeLogSyncQuery {
    /// Fetch changes strictly greater than this log sequence ID.
    pub after_id: i64,
    /// Optional client identifier to exclude from results (avoiding self-echo).
    pub exclude_client_id: Option<String>,
    /// Maximum number of change records to retrieve in a single batch (default 100).
    pub limit: u32,
}

impl Default for ChangeLogSyncQuery {
    fn default() -> Self {
        Self {
            after_id: 0,
            exclude_client_id: None,
            limit: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changelog_entry_serde_roundtrip() {
        let entry = ChangeLogEntry {
            id: 42,
            event_type: "file.rated".to_string(),
            entity_id: 1042,
            secondary_id: Some("hero".to_string()),
            client_id: "workstation_01".to_string(),
            payload: Some(r#"{"rating": 9}"#.to_string()),
            created_at: 1726000000,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert_eq!(
            serde_json::from_str::<ChangeLogEntry>(&json).unwrap(),
            entry
        );
    }

    #[test]
    fn mutation_result_serde_roundtrip() {
        let result = MutationResult {
            success: true,
            current_version: 5,
            rows_affected: 1,
            conflict_detected: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(
            serde_json::from_str::<MutationResult>(&json).unwrap(),
            result
        );
    }

    #[test]
    fn changelog_sync_query_defaults() {
        let query = ChangeLogSyncQuery::default();
        assert_eq!(query.after_id, 0);
        assert_eq!(query.exclude_client_id, None);
        assert_eq!(query.limit, 100);
    }
}
