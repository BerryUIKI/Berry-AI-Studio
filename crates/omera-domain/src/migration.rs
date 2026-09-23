//! Domain types and DTOs for the Berry -> Omera legacy migration service.
use serde::{Deserialize, Serialize};

/// A discovered legacy data source (e.g. Berry AI Studio or Berry AIGC Toolbox).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredSource {
    pub source_id: String,
    pub identifier: String,
    pub root_path: String,
    pub database_path: String,
    pub config_path: Option<String>,
    pub file_count: u64,
    pub database_size_bytes: u64,
    pub total_size_bytes: u64,
    pub schema_version: i64,
    pub is_locked: bool,
}

/// An individual artifact tracked during migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigratedArtifact {
    pub category: String,
    pub source_path: String,
    pub destination_path: String,
    pub status: String,
    pub size_bytes: u64,
}

/// A durable, verifiable receipt stored at the destination after migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationReceipt {
    pub receipt_id: String,
    pub source_id: String,
    pub source_identifier: String,
    pub source_root: String,
    pub source_schema_version: i64,
    pub destination_root: String,
    pub destination_db: String,
    pub created_at: u64,
    pub artifacts: Vec<MigratedArtifact>,
    pub integrity_hash: String,
    pub cleanup_status: String,
}

/// Structured migration error as defined in API contracts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationError {
    pub code: String,
    pub message_key: String,
    pub retryable: bool,
    pub context: Option<String>,
}

impl MigrationError {
    pub fn new(
        code: impl Into<String>,
        message_key: impl Into<String>,
        retryable: bool,
        context: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message_key: message_key.into(),
            retryable,
            context,
        }
    }
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.code,
            self.message_key,
            self.context.as_deref().unwrap_or("no details")
        )
    }
}

impl std::error::Error for MigrationError {}

/// Overall migration lifecycle status for the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyMigrationStatus {
    pub stage: String,
    pub discovered_sources: Vec<DiscoveredSource>,
    pub destination_exists: bool,
    pub active_receipt: Option<MigrationReceipt>,
    pub available_actions: Vec<String>,
}

/// Preview of a migration plan before execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyMigrationPreview {
    pub plan_id: String,
    pub source: DiscoveredSource,
    pub destination_root: String,
    pub destination_db: String,
    pub required_space_bytes: u64,
    pub available_space_bytes: u64,
    pub conflicts: Vec<String>,
    pub exclusions: Vec<String>,
}

/// Status and progress of an active or completed migration job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyMigrationJob {
    pub job_id: String,
    pub plan_id: String,
    pub status: String,
    pub progress: f32,
    pub current_step: String,
    pub error: Option<MigrationError>,
    pub receipt: Option<MigrationReceipt>,
}

/// An individual file or directory item considered for cleanup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CleanupItem {
    pub path: String,
    pub category: String,
    pub size_bytes: u64,
    pub eligible: bool,
    pub reason: Option<String>,
}

/// Preview of eligible legacy artifacts for optional user cleanup.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyCleanupPreview {
    pub preview_id: String,
    pub receipt_id: String,
    pub expires_at: u64,
    pub items: Vec<CleanupItem>,
    pub total_size_bytes: u64,
    pub destination_healthy: bool,
}

/// Result of an approved cleanup operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LegacyCleanupResult {
    pub receipt_id: String,
    pub cleaned_count: usize,
    pub failed_count: usize,
    pub cleaned_bytes: u64,
    pub errors: Vec<String>,
    pub status: String,
}
