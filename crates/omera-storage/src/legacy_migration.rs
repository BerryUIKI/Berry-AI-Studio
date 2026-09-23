//! Safe, WAL-preserving SQLite migration engine and receipt persistence for Berry -> Omera.
use omera_domain::{DiscoveredSource, MigrationError, MigrationReceipt};
use rusqlite::{backup::Backup, Connection, OpenFlags};
use std::{path::Path, time::Duration};

pub const RECEIPT_FILENAME: &str = "migration_receipt.json";

/// Validate a source database for integrity, foreign keys, and supported schema version.
pub fn validate_source_database(path: &Path) -> Result<Connection, MigrationError> {
    if !path.exists() {
        return Err(MigrationError::new(
            "SOURCE_CHANGED",
            "error.migration.source_not_found",
            false,
            Some(format!("Database file not found at {}", path.display())),
        ));
    }

    let conn =
        Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
            MigrationError::new(
                "SOURCE_BUSY",
                "error.migration.cannot_open_source",
                true,
                Some(e.to_string()),
            )
        })?;

    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| {
            MigrationError::new(
                "INTEGRITY_FAILED",
                "error.migration.integrity_failed",
                false,
                Some(e.to_string()),
            )
        })?;

    if integrity != "ok" {
        return Err(MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.integrity_failed",
            false,
            Some(format!("SQLite integrity check failed: {integrity}")),
        ));
    }

    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| {
            MigrationError::new(
                "UNSUPPORTED_SCHEMA",
                "error.migration.unsupported_schema",
                false,
                Some(e.to_string()),
            )
        })?;

    if !(1..=crate::migrations::LATEST_VERSION).contains(&version) {
        return Err(MigrationError::new(
            "UNSUPPORTED_SCHEMA",
            "error.migration.unsupported_schema",
            false,
            Some(format!(
                "Source database schema version {version} is outside supported range 1..={}",
                crate::migrations::LATEST_VERSION
            )),
        ));
    }

    let fk_violations: bool = conn
        .prepare("PRAGMA foreign_key_check")
        .and_then(|mut stmt| stmt.exists([]))
        .map_err(|e| {
            MigrationError::new(
                "INTEGRITY_FAILED",
                "error.migration.foreign_key_failed",
                false,
                Some(e.to_string()),
            )
        })?;

    if fk_violations {
        return Err(MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.foreign_key_failed",
            false,
            Some("Source database contains foreign-key relationship violations".into()),
        ));
    }

    Ok(conn)
}

/// Discover a potential legacy data root and return its inventory metrics.
pub fn discover_database_source(
    root_path: &Path,
    identifier: &str,
) -> Result<Option<DiscoveredSource>, String> {
    let db_path = root_path.join("berry.db");
    if !db_path.exists() {
        return Ok(None);
    }

    let mut db_size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or_default();

    let wal_path = root_path.join("berry.db-wal");
    if wal_path.exists() {
        db_size += std::fs::metadata(&wal_path)
            .map(|m| m.len())
            .unwrap_or_default();
    }
    let shm_path = root_path.join("berry.db-shm");
    if shm_path.exists() {
        db_size += std::fs::metadata(&shm_path)
            .map(|m| m.len())
            .unwrap_or_default();
    }

    let config_path = root_path.join("config.json");
    let config_path_str = if config_path.exists() {
        Some(config_path.to_string_lossy().to_string())
    } else {
        None
    };

    // Calculate total app directory size
    let total_size = calculate_dir_size(root_path);

    // Inspect database details safely
    let mut file_count = 0u64;
    let mut schema_version = 0i64;
    let mut is_locked = false;

    match Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(conn) => {
            if let Ok(v) = conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0)) {
                schema_version = v;
            }
            if let Ok(c) = conn.query_row("SELECT COUNT(*) FROM files", [], |r| r.get::<_, i64>(0))
            {
                file_count = c.max(0) as u64;
            }
        }
        Err(_) => {
            is_locked = true;
        }
    }

    let source_id = format!("{}_{}", identifier, sanitize_path_id(root_path));

    Ok(Some(DiscoveredSource {
        source_id,
        identifier: identifier.to_string(),
        root_path: root_path.to_string_lossy().to_string(),
        database_path: db_path.to_string_lossy().to_string(),
        config_path: config_path_str,
        file_count,
        database_size_bytes: db_size,
        total_size_bytes: total_size,
        schema_version,
        is_locked,
    }))
}

/// Copy database snapshot through the SQLite Backup API (including WAL content),
/// validate, upgrade staged copy to LATEST_VERSION, and atomically publish to destination.
pub fn migrate_database(
    source_db: &Path,
    destination_db: &Path,
    staging_dir: &Path,
) -> Result<u64, MigrationError> {
    if destination_db.exists() {
        return Err(MigrationError::new(
            "DESTINATION_EXISTS",
            "error.migration.destination_exists",
            false,
            Some(format!(
                "Target database {} already exists",
                destination_db.display()
            )),
        ));
    }

    let source_conn = validate_source_database(source_db)?;

    let dest_parent = destination_db.parent().ok_or_else(|| {
        MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.invalid_destination",
            false,
            Some("Destination path has no parent directory".into()),
        )
    })?;
    std::fs::create_dir_all(dest_parent).map_err(|e| {
        MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.cannot_create_dir",
            false,
            Some(e.to_string()),
        )
    })?;

    // Create staged database file in staging_dir
    let staged_db_path = staging_dir.join("omera.staged.db");
    {
        let mut staged_conn = Connection::open(&staged_db_path).map_err(|e| {
            MigrationError::new(
                "INTEGRITY_FAILED",
                "error.migration.staging_error",
                false,
                Some(e.to_string()),
            )
        })?;

        // SQLite online backup API safely captures all committed WAL pages without blocking readers
        Backup::new(&source_conn, &mut staged_conn)
            .map_err(|e| {
                MigrationError::new(
                    "INTEGRITY_FAILED",
                    "error.migration.backup_failed",
                    false,
                    Some(e.to_string()),
                )
            })?
            .run_to_completion(128, Duration::from_millis(5), None)
            .map_err(|e| {
                MigrationError::new(
                    "INTEGRITY_FAILED",
                    "error.migration.backup_failed",
                    false,
                    Some(e.to_string()),
                )
            })?;

        staged_conn.close().map_err(|(_, e)| {
            MigrationError::new(
                "INTEGRITY_FAILED",
                "error.migration.staging_close_error",
                false,
                Some(e.to_string()),
            )
        })?;
    }

    // Connect to staged copy with Database::connect to apply migrations up to LATEST_VERSION
    let migrated = crate::Database::connect(&staged_db_path).map_err(|e| {
        MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.migration_apply_failed",
            false,
            Some(e.to_string()),
        )
    })?;

    // Consolidate staged database to clean rollback journal
    migrated
        .connection()
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")
        .map_err(|e| {
            MigrationError::new(
                "INTEGRITY_FAILED",
                "error.migration.checkpoint_failed",
                false,
                Some(e.to_string()),
            )
        })?;

    drop(migrated);

    // Validate the migrated staged database
    validate_source_database(&staged_db_path)?;

    // Atomically persist to destination_db
    let mut temp = tempfile::NamedTempFile::new_in(dest_parent).map_err(|e| {
        MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.temp_file_error",
            false,
            Some(e.to_string()),
        )
    })?;

    let mut staged_file = std::fs::File::open(&staged_db_path).map_err(|e| {
        MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.open_staged_error",
            false,
            Some(e.to_string()),
        )
    })?;

    std::io::copy(&mut staged_file, temp.as_file_mut()).map_err(|e| {
        MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.copy_error",
            false,
            Some(e.to_string()),
        )
    })?;

    temp.as_file().sync_all().map_err(|e| {
        MigrationError::new(
            "INTEGRITY_FAILED",
            "error.migration.sync_error",
            false,
            Some(e.to_string()),
        )
    })?;

    temp.persist_noclobber(destination_db).map_err(|e| {
        MigrationError::new(
            "DESTINATION_EXISTS",
            "error.migration.persist_collision",
            false,
            Some(e.to_string()),
        )
    })?;

    let final_size = std::fs::metadata(destination_db)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(final_size)
}

/// Atomically write a durable migration receipt to the destination directory.
pub fn save_receipt(destination_root: &Path, receipt: &MigrationReceipt) -> Result<(), String> {
    std::fs::create_dir_all(destination_root).map_err(|e| e.to_string())?;
    let receipt_path = destination_root.join(RECEIPT_FILENAME);
    let bytes = serde_json::to_vec_pretty(receipt).map_err(|e| e.to_string())?;

    let mut temp = tempfile::NamedTempFile::new_in(destination_root).map_err(|e| e.to_string())?;
    use std::io::Write;
    temp.write_all(&bytes).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    temp.persist(&receipt_path).map_err(|e| e.to_string())?;

    Ok(())
}

/// Load the durable migration receipt from the destination directory if present.
pub fn load_receipt(destination_root: &Path) -> Result<Option<MigrationReceipt>, String> {
    let receipt_path = destination_root.join(RECEIPT_FILENAME);
    if !receipt_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read(&receipt_path).map_err(|e| e.to_string())?;
    let receipt: MigrationReceipt = serde_json::from_slice(&content).map_err(|e| e.to_string())?;
    Ok(Some(receipt))
}

/// Verify that destination omera.db exists, is non-empty, and passes integrity check.
pub fn verify_destination_health(destination_db: &Path) -> Result<bool, String> {
    if !destination_db.exists() {
        return Ok(false);
    }
    match validate_source_database(destination_db) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Destination database unhealthy: {e}")),
    }
}

fn calculate_dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    let path = entry.path();
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    // Exclude external mounts or backups from app-data size calculation
                    if !name_str.starts_with('.') && !meta.is_symlink() {
                        total += calculate_dir_size(&path);
                    }
                }
            }
        }
    }
    total
}

fn sanitize_path_id(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_database_source_none_when_no_db() {
        let dir = tempfile::tempdir().unwrap();
        let res = discover_database_source(dir.path(), "com.berryuiki.berryaistudio").unwrap();
        assert!(res.is_none());
    }

    #[test]
    fn test_discover_and_migrate_database_wal_preservation() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let staging_dir = tempfile::tempdir().unwrap();

        let source_db = source_dir.path().join("berry.db");
        let dest_db = dest_dir.path().join("omera.db");

        // Create a database with pending WAL content
        let conn = crate::Database::connect(&source_db).unwrap();
        conn.connection()
            .execute("INSERT INTO tags(name) VALUES ('wal_tag')", [])
            .unwrap();

        // Discover source
        let discovered = discover_database_source(source_dir.path(), "com.berryuiki.berryaistudio")
            .unwrap()
            .expect("should find source");
        assert_eq!(discovered.identifier, "com.berryuiki.berryaistudio");
        assert!(discovered.database_size_bytes > 0);

        // Migrate
        let final_size = migrate_database(&source_db, &dest_db, staging_dir.path()).unwrap();
        assert!(final_size > 0);
        assert!(dest_db.exists());

        // Validate dest DB contains the WAL data and is healthy
        assert!(verify_destination_health(&dest_db).unwrap());
        let dest_conn = crate::Database::connect(&dest_db).unwrap();
        let tags = dest_conn.list_tags().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "wal_tag");

        // Re-migrating to existing destination must error with DESTINATION_EXISTS
        let err = migrate_database(&source_db, &dest_db, staging_dir.path()).unwrap_err();
        assert_eq!(err.code, "DESTINATION_EXISTS");
    }

    #[test]
    fn test_receipt_save_and_load() {
        let dest_dir = tempfile::tempdir().unwrap();
        let receipt = MigrationReceipt {
            receipt_id: "test-receipt-1".into(),
            source_id: "src-1".into(),
            source_identifier: "com.berryuiki.berryaistudio".into(),
            source_root: "/path/to/src".into(),
            source_schema_version: 12,
            destination_root: dest_dir.path().to_string_lossy().to_string(),
            destination_db: dest_dir
                .path()
                .join("omera.db")
                .to_string_lossy()
                .to_string(),
            created_at: 123456789,
            artifacts: vec![],
            integrity_hash: "hash123".into(),
            cleanup_status: "pending".into(),
        };

        save_receipt(dest_dir.path(), &receipt).unwrap();
        let loaded = load_receipt(dest_dir.path())
            .unwrap()
            .expect("should load receipt");
        assert_eq!(loaded, receipt);
    }

    #[test]
    fn test_corrupt_database_rejected() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let staging_dir = tempfile::tempdir().unwrap();

        let source_db = source_dir.path().join("berry.db");
        let dest_db = dest_dir.path().join("omera.db");

        std::fs::write(&source_db, b"sqlite format 3\0THIS_IS_CORRUPTED_DATA").unwrap();

        let err = migrate_database(&source_db, &dest_db, staging_dir.path()).unwrap_err();
        assert_eq!(err.code, "INTEGRITY_FAILED");
        assert!(!dest_db.exists());
    }

    #[test]
    fn test_future_schema_rejected() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let staging_dir = tempfile::tempdir().unwrap();

        let source_db = source_dir.path().join("berry.db");
        let dest_db = dest_dir.path().join("omera.db");

        let conn = crate::Database::connect(&source_db).unwrap();
        conn.connection()
            .pragma_update(None, "user_version", crate::migrations::LATEST_VERSION + 1)
            .unwrap();
        drop(conn);

        let err = migrate_database(&source_db, &dest_db, staging_dir.path()).unwrap_err();
        assert_eq!(err.code, "UNSUPPORTED_SCHEMA");
        assert!(!dest_db.exists());
    }

    #[test]
    fn test_schema_upgrade_applied_to_copy_only() {
        let source_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let staging_dir = tempfile::tempdir().unwrap();

        let source_db = source_dir.path().join("berry.db");
        let dest_db = dest_dir.path().join("omera.db");

        let conn = Connection::open(&source_db).unwrap();
        for sql in crate::migrations::MIGRATIONS.iter().take(2) {
            conn.execute_batch(sql).unwrap();
        }
        conn.pragma_update(None, "user_version", 2).unwrap();
        drop(conn);

        migrate_database(&source_db, &dest_db, staging_dir.path()).unwrap();

        let source_conn = Connection::open(&source_db).unwrap();
        let source_version: i64 = source_conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(source_version, 2);

        let dest_conn = crate::Database::connect(&dest_db).unwrap();
        assert_eq!(
            dest_conn.user_version().unwrap(),
            crate::migrations::LATEST_VERSION
        );
    }
}
