//! Domain models for Milestone 12.4: Incremental Remote Asset Mirroring & Delta Sync.

use serde::{Deserialize, Serialize};

/// Direction of incremental mirroring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudSyncDirection {
    /// Mirror local media files up to cloud/remote storage.
    UploadToRemote,
    /// Mirror remote media files down to local library storage.
    DownloadFromRemote,
}

impl Default for CloudSyncDirection {
    fn default() -> Self {
        Self::UploadToRemote
    }
}

/// Strategy for detecting whether a file has changed remotely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudSyncStrategy {
    /// Compares file size and modified timestamp / remote ETag without reading the entire file.
    FastFingerprint,
    /// Calculates complete SHA-256 hash for strict delta verification.
    Sha256Checksum,
}

impl Default for CloudSyncStrategy {
    fn default() -> Self {
        Self::FastFingerprint
    }
}

/// Phase of the sync process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudSyncPhase {
    Idle,
    Scanning,
    Syncing,
    Completed,
    Cancelled,
    Failed,
}

impl Default for CloudSyncPhase {
    fn default() -> Self {
        Self::Idle
    }
}

/// Options configured by the user for media delta mirroring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncOptions {
    /// Direction: upload to remote or download from remote.
    #[serde(default)]
    pub direction: CloudSyncDirection,
    /// Strategy: FastFingerprint (size/etag) or Sha256Checksum.
    #[serde(default)]
    pub strategy: CloudSyncStrategy,
    /// Concurrency level (number of parallel transfer threads, 1..16).
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    /// Optional upload/download bandwidth limit in KB/s (None or 0 = unlimited).
    #[serde(default)]
    pub bandwidth_limit_kbs: Option<u64>,
    /// If true, performs discovery and comparison without transferring files.
    #[serde(default)]
    pub dry_run: bool,
    /// Remote object prefix/subfolder for media assets (default "media/").
    #[serde(default = "default_media_prefix")]
    pub remote_prefix: String,
    /// Optional library folder IDs to restrict sync to; None or empty = all folders.
    #[serde(default)]
    pub folder_ids: Option<Vec<i64>>,
}

fn default_concurrency() -> usize {
    4
}

fn default_media_prefix() -> String {
    "media/".to_string()
}

impl Default for CloudSyncOptions {
    fn default() -> Self {
        Self {
            direction: CloudSyncDirection::UploadToRemote,
            strategy: CloudSyncStrategy::FastFingerprint,
            concurrency: default_concurrency(),
            bandwidth_limit_kbs: None,
            dry_run: false,
            remote_prefix: default_media_prefix(),
            folder_ids: None,
        }
    }
}

/// Live progress status emitted during media delta sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncProgress {
    pub phase: CloudSyncPhase,
    pub total_files: usize,
    pub completed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub current_file: Option<String>,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: Option<u64>,
    pub error: Option<String>,
}

impl Default for CloudSyncProgress {
    fn default() -> Self {
        Self {
            phase: CloudSyncPhase::Idle,
            total_files: 0,
            completed_files: 0,
            skipped_files: 0,
            failed_files: 0,
            total_bytes: 0,
            transferred_bytes: 0,
            current_file: None,
            speed_bytes_per_sec: 0,
            eta_seconds: None,
            error: None,
        }
    }
}

/// Final summary result of a media delta sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncResult {
    pub success: bool,
    pub total_files: usize,
    pub synced_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub transferred_bytes: u64,
    pub duration_ms: u64,
    pub dry_run: bool,
    pub errors: Vec<String>,
}

impl Default for CloudSyncResult {
    fn default() -> Self {
        Self {
            success: true,
            total_files: 0,
            synced_files: 0,
            skipped_files: 0,
            failed_files: 0,
            transferred_bytes: 0,
            duration_ms: 0,
            dry_run: false,
            errors: Vec::new(),
        }
    }
}
