//! Restore at startup, before opening application connections or starting workers.
use std::{path::Path, time::Duration};
use rusqlite::{backup::Backup, Connection, OpenFlags};

fn validated_source(path: &Path) -> Result<Connection, String> {
    let source = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| e.to_string())?;
    let integrity: String = source.query_row("PRAGMA integrity_check", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if integrity != "ok" { return Err(format!("Invalid backup: {integrity}")); }
    let version: i64 = source.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if !(1..=crate::migrations::LATEST_VERSION).contains(&version) { return Err("Unsupported backup schema".into()); }
    source.prepare("SELECT id, path FROM files LIMIT 0").map_err(|e| e.to_string())?;
    Ok(source)
}

fn copy_database(source: &Connection, path: &Path) -> Result<(), String> {
    let mut destination = Connection::open(path).map_err(|e| e.to_string())?;
    Backup::new(source, &mut destination).map_err(|e| e.to_string())?
        .run_to_completion(128, Duration::from_millis(5), None).map_err(|e| e.to_string())?;
    destination.close().map_err(|(_, e)| e.to_string())?;
    Ok(())
}

pub fn stage_restore(source: &Path, active: &Path) -> Result<(), String> {
    let source = validated_source(source)?;
    let temporary = tempfile::NamedTempFile::new_in(active.parent().ok_or("Missing database directory")?).map_err(|e| e.to_string())?;
    copy_database(&source, temporary.path())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    // Never replace an already pending restore silently.
    temporary.persist_noclobber(active.with_extension("pending-restore.db")).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn apply_pending_restore(active: &Path) -> Result<(), String> {
    let pending = active.with_extension("pending-restore.db");
    if !pending.exists() { return Ok(()); }
    let source = validated_source(&pending)?;
    let rollback = active.with_extension("pre-restore.db");
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
        let active = dir.path().join("active.db"); let source = dir.path().join("backup.db");
        let live = crate::Database::connect(&active).unwrap();
        let backup = crate::Database::connect(&source).unwrap();
        backup.connection().execute("INSERT INTO tags(name) VALUES ('restored')", []).unwrap();
        stage_restore(&source, &active).unwrap();
        assert!(live.list_tags().unwrap().is_empty());
        drop(live); drop(backup);
        apply_pending_restore(&active).unwrap();
        assert_eq!(crate::Database::connect(&active).unwrap().list_tags().unwrap().len(), 1);
        assert!(active.with_extension("pre-restore.db").exists());
    }
    #[test]
    fn rejects_invalid_backup_without_modifying_active() {
        let dir = tempfile::tempdir().unwrap();
        let active = dir.path().join("active.db"); let bad = dir.path().join("bad.db");
        std::fs::write(&bad, b"invalid").unwrap();
        assert!(stage_restore(&bad, &active).is_err());
        assert!(!active.exists());
    }
}
