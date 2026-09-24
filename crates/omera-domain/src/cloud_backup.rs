//! Domain models for cloud backup, snapshots, and disaster recovery.

use serde::{Deserialize, Serialize};

/// Supported storage backend providers for cloud backup snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CloudStorageProvider {
    /// Local directory or network mounted SMB/NFS storage path.
    #[default]
    LocalPath,
    /// WebDAV server (Nextcloud, Synology DSM, QNAP, ownCloud, etc.).
    WebDav,
    /// S3-compatible object storage (AWS S3, Cloudflare R2, MinIO, Backblaze B2).
    S3,
}

/// User configuration for cloud backup and disaster recovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CloudBackupConfig {
    /// Selected storage provider.
    #[serde(default)]
    pub provider: CloudStorageProvider,

    /// Local or network directory path (for `LocalPath` provider).
    pub local_path: Option<String>,

    /// WebDAV server base URL (e.g. `https://nas.local:5006/dav/berry_backups`).
    pub webdav_endpoint: Option<String>,
    /// WebDAV HTTP authentication username.
    pub webdav_username: Option<String>,
    /// WebDAV HTTP authentication password or app token.
    pub webdav_password: Option<String>,

    /// S3 endpoint URL (e.g. `https://<account_id>.r2.cloudflarestorage.com` or `https://minio.local:9000`).
    pub s3_endpoint: Option<String>,
    /// S3 bucket name.
    pub s3_bucket: Option<String>,
    /// S3 region (e.g. `us-east-1` or `auto`).
    pub s3_region: Option<String>,
    /// S3 Access Key ID.
    pub s3_access_key: Option<String>,
    /// S3 Secret Access Key.
    pub s3_secret_key: Option<String>,
    /// S3 object prefix path (e.g. `backups/`).
    pub s3_prefix: Option<String>,

    /// Whether automatic background snapshot creation is enabled.
    #[serde(default)]
    pub auto_backup_enabled: bool,
    /// Scheduled interval in days (e.g. 1 = daily, 7 = weekly).
    #[serde(default = "default_auto_backup_days")]
    pub auto_backup_interval_days: u32,
}

fn default_auto_backup_days() -> u32 {
    7
}

/// Metadata describing a single backup snapshot archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudSnapshotMeta {
    /// Unique snapshot identifier (e.g. `berry_snapshot_2026-09-21_143000`).
    pub snapshot_id: String,
    /// Archive filename (e.g. `berry_snapshot_2026-09-21_143000.zip`).
    pub filename: String,
    /// Archive file size in bytes.
    pub size_bytes: u64,
    /// Snapshot creation timestamp (Unix epoch seconds).
    pub created_at: i64,
    /// Number of media files recorded in the snapshot database.
    pub file_count: i64,
    /// Number of library folders recorded.
    pub folder_count: i64,
    /// Number of tags recorded.
    pub tag_count: i64,
    /// Number of albums recorded.
    pub album_count: i64,
    /// Optional user-provided description for the snapshot.
    pub description: Option<String>,
    /// Legacy Berry version that produced the snapshot (if applicable).
    #[serde(default)]
    pub berry_version: String,
    /// Omera version that produced the snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omera_version: Option<String>,
}

/// Connectivity and latency test result for a cloud storage provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudPingResult {
    pub success: bool,
    pub latency_ms: u64,
    pub message: String,
}

/// Result of a completed snapshot creation job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudBackupResult {
    pub success: bool,
    pub snapshot: Option<CloudSnapshotMeta>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Result of restoring a snapshot database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudRestoreResult {
    pub success: bool,
    pub restored_files_count: i64,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_backup_config_serde_roundtrip() {
        let config = CloudBackupConfig {
            provider: CloudStorageProvider::S3,
            local_path: None,
            webdav_endpoint: None,
            webdav_username: None,
            webdav_password: None,
            s3_endpoint: Some("https://r2.cloudflarestorage.com".to_string()),
            s3_bucket: Some("berry-backups".to_string()),
            s3_region: Some("auto".to_string()),
            s3_access_key: Some("test_key".to_string()),
            s3_secret_key: Some("test_secret".to_string()),
            s3_prefix: Some("studio/".to_string()),
            auto_backup_enabled: true,
            auto_backup_interval_days: 3,
        };

        let json = serde_json::to_string(&config).unwrap();
        let decoded: CloudBackupConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, config);
    }
}
