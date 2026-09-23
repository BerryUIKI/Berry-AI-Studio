//! Restore at startup, before opening application connections or starting workers.
use rusqlite::{backup::Backup, Connection, OpenFlags};
use std::{path::Path, time::Duration};

fn validated_source(path: &Path) -> Result<Connection, String> {
    let source = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| e.to_string())?;
    let integrity: String = source
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if integrity != "ok" {
        return Err(format!("Invalid backup: {integrity}"));
    }
    let version: i64 = source
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if !(1..=crate::migrations::LATEST_VERSION).contains(&version) {
        return Err("Unsupported backup schema".into());
    }
    source
        .prepare("SELECT id, path FROM files LIMIT 0")
        .map_err(|e| e.to_string())?;
    let violations = source
        .prepare("PRAGMA foreign_key_check")
        .map_err(|e| e.to_string())?
        .exists([])
        .map_err(|e| e.to_string())?;
    if violations {
        return Err("Backup contains broken foreign-key relationships".into());
    }
    Ok(source)
}

/// Copy and upgrade a legacy library without changing the source or replacing
/// an existing destination. The SQLite backup API includes committed WAL data.
/// Caller owns configuration/credential migration and the activation receipt.
pub fn migrate_copy(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        return Err("Destination library already exists; choose an explicit import".into());
    }
    let source = validated_source(source)?;
    let parent = destination
        .parent()
        .ok_or("Missing destination directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let staging = tempfile::tempdir_in(parent).map_err(|e| e.to_string())?;
    let staged_path = staging.path().join("library.db");
    copy_database(&source, &staged_path)?;
    let migrated = crate::Database::connect(&staged_path).map_err(|e| e.to_string())?;
    // Consolidate the staged copy before publishing a single database file.
    migrated
        .connection()
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")
        .map_err(|e| e.to_string())?;
    drop(migrated);
    drop(validated_source(&staged_path)?);
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    let mut input = std::fs::File::open(&staged_path).map_err(|e| e.to_string())?;
    std::io::copy(&mut input, temporary.as_file_mut()).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary
        .persist_noclobber(destination)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_database(source: &Connection, path: &Path) -> Result<(), String> {
    let mut destination = Connection::open(path).map_err(|e| e.to_string())?;
    Backup::new(source, &mut destination)
        .map_err(|e| e.to_string())?
        .run_to_completion(128, Duration::from_millis(5), None)
        .map_err(|e| e.to_string())?;
    destination.close().map_err(|(_, e)| e.to_string())?;
    Ok(())
}

pub fn stage_restore(source: &Path, active: &Path) -> Result<(), String> {
    let source = validated_source(source)?;
    let temporary =
        tempfile::NamedTempFile::new_in(active.parent().ok_or("Missing database directory")?)
            .map_err(|e| e.to_string())?;
    copy_database(&source, temporary.path())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    // Never replace an already pending restore silently.
    temporary
        .persist_noclobber(active.with_extension("pending-restore.db"))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_pending_restore(active: &Path) -> Result<(), String> {
    let pending = active.with_extension("pending-restore.db");
    if !pending.exists() {
        return Ok(());
    }
    let source = validated_source(&pending)?;
    // Retain each recovery point. An interrupted activation may be retried;
    // never replace the only pre-restore copy with an already restored database.
    let rollback = active.with_extension(format!("pre-restore-{}.db", uuid::Uuid::new_v4()));
    if active.exists() {
        let original = validated_source(active)?;
        copy_database(&original, &rollback)?;
        validated_source(&rollback)?;
    }
    // SQLite's backup API updates the destination transactionally, including WAL.
    // All application workers and connections are still stopped at this point.
    copy_database(&source, active)?;
    validated_source(active)?;
    drop(source);
    std::fs::remove_file(pending).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stages_without_touching_live_database_and_retains_rollback() {
        let dir = tempfile::tempdir().unwrap();
        let active = dir.path().join("active.db");
        let source = dir.path().join("backup.db");
        let live = crate::Database::connect(&active).unwrap();
        let backup = crate::Database::connect(&source).unwrap();
        backup
            .connection()
            .execute("INSERT INTO tags(name) VALUES ('restored')", [])
            .unwrap();
        stage_restore(&source, &active).unwrap();
        assert!(live.list_tags().unwrap().is_empty());
        drop(live);
        drop(backup);
        apply_pending_restore(&active).unwrap();
        assert_eq!(
            crate::Database::connect(&active)
                .unwrap()
                .list_tags()
                .unwrap()
                .len(),
            1
        );
        assert!(std::fs::read_dir(dir.path()).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("pre-restore-")));
    }
    #[test]
    fn rejects_invalid_backup_without_modifying_active() {
        let dir = tempfile::tempdir().unwrap();
        let active = dir.path().join("active.db");
        let bad = dir.path().join("bad.db");
        std::fs::write(&bad, b"invalid").unwrap();
        assert!(stage_restore(&bad, &active).is_err());
        assert!(!active.exists());
    }

    #[test]
    fn migration_copies_committed_wal_and_preserves_source() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("berry.db");
        let destination = dir.path().join("omera.db");
        let source = crate::Database::connect(&source_path).unwrap();
        source
            .connection()
            .execute("INSERT INTO tags(name) VALUES ('preserved')", [])
            .unwrap();
        assert!(source_path.with_extension("db-wal").exists());
        migrate_copy(&source_path, &destination).unwrap();
        assert_eq!(source.list_tags().unwrap()[0].name, "preserved");
        assert_eq!(
            crate::Database::connect(&destination)
                .unwrap()
                .list_tags()
                .unwrap()[0]
                .name,
            "preserved"
        );
        assert!(migrate_copy(&source_path, &destination).is_err());
    }

    #[test]
    fn migration_upgrades_only_the_copy() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("berry.db");
        let destination = dir.path().join("omera.db");
        let source = Connection::open(&source_path).unwrap();
        for sql in crate::migrations::MIGRATIONS.iter().take(2) {
            source.execute_batch(sql).unwrap();
        }
        source.pragma_update(None, "user_version", 2).unwrap();
        migrate_copy(&source_path, &destination).unwrap();
        assert_eq!(
            source
                .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            crate::Database::connect(&destination)
                .unwrap()
                .user_version()
                .unwrap(),
            crate::migrations::LATEST_VERSION
        );
    }

    #[test]
    fn future_schema_is_rejected_by_migration_and_normal_open() {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("future.db");
        let destination = dir.path().join("omera.db");
        let source = crate::Database::connect(&source_path).unwrap();
        source
            .connection()
            .pragma_update(None, "user_version", crate::migrations::LATEST_VERSION + 1)
            .unwrap();
        drop(source);
        let before = std::fs::read(&source_path).unwrap();
        assert!(migrate_copy(&source_path, &destination).is_err());
        assert!(crate::Database::connect(&source_path).is_err());
        assert!(!destination.exists());
        assert_eq!(std::fs::read(&source_path).unwrap(), before);
    }
}
