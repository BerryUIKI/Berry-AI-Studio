//! Recoverable file operations. Destinations are never overwritten.
use berry_storage::Database;
use std::{fs, io::Write, path::Path};

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Copy,
    Move,
    Trash,
}

pub fn execute(
    db_path: &Path,
    paths: &[String],
    destination: Option<i64>,
    operation: Operation,
) -> Result<usize, String> {
    let db = Database::connect(db_path).map_err(|e| e.to_string())?;
    let target = destination
        .map(|id| {
            db.list_folders()
                .map_err(|e| e.to_string())?
                .into_iter()
                .find(|folder| folder.id == id)
                .ok_or_else(|| "Destination folder not found".to_string())
        })
        .transpose()?;
    let journal_path = db_path.with_extension("file-operations.jsonl");
    let mut journal = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)
        .map_err(|e| e.to_string())?;
    let mut completed = 0;
    let mut errors = Vec::new();
    for path in paths {
        let result = (|| -> Result<(), String> {
            let original = db
                .get_file_by_path(path)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Source is not indexed".to_string())?;
            let source = Path::new(path);
            let mut sources = vec![source.to_path_buf()];
            for extension in ["txt", "json"] {
                let sidecar = source.with_extension(extension);
                if sidecar != source && sidecar.is_file() {
                    sources.push(sidecar);
                }
            }
            let destinations = if let Some(folder) = &target {
                sources
                    .iter()
                    .map(|source| {
                        Path::new(&folder.path).join(source.file_name().unwrap_or_default())
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            if destinations.iter().any(|path| path.exists()) {
                return Err("Destination already exists; original preserved".into());
            }
            // Persist the operation plan before any filesystem mutation for recovery.
            writeln!(journal, "{}", serde_json::json!({"state":"started", "operation":format!("{operation:?}"), "sources":sources,"destinations":destinations}))
                .and_then(|_| journal.sync_all()).map_err(|e| e.to_string())?;
            if matches!(operation, Operation::Trash) {
                // No permanent-delete fallback, including sidecars.
                trash::delete_all(&sources).map_err(|e| e.to_string())?;
                db.delete_file_by_path(path).map_err(|e| e.to_string())?;
            } else {
                let mut published = Vec::new();
                for (source, destination) in sources.iter().zip(&destinations) {
                    if let Err(error) = copy_new(source, destination) {
                        for path in published {
                            let _ = fs::remove_file(path);
                        }
                        return Err(error.to_string());
                    }
                    published.push(destination.clone());
                }
                let mut copied = original.clone();
                copied.id = None;
                copied.path = destinations[0].to_string_lossy().into_owned();
                copied.folder_id = destination.ok_or("Missing destination")?;
                copied.stack_id = None;
                copied.stack_order = 0;
                // Persist destination before removing any source. Failed writes leave originals safe.
                let persisted = if matches!(operation, Operation::Copy) {
                    db.upsert_file(&copied).map(|_| ())
                } else {
                    db.move_file_record(path, &copied.path, copied.folder_id)
                        .map(|_| ())
                };
                if let Err(error) = persisted {
                    for path in published {
                        let _ = fs::remove_file(path);
                    }
                    return Err(error.to_string());
                }
                if matches!(operation, Operation::Move) {
                    for source in &sources {
                        fs::remove_file(source).map_err(|e| {
                            format!(
                                "Destination saved at {}; source cleanup failed: {e}",
                                copied.path
                            )
                        })?;
                    }
                }
            }
            writeln!(
                journal,
                "{}",
                serde_json::json!({"state":"completed", "source":path})
            )
            .and_then(|_| journal.sync_all())
            .map_err(|e| e.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => completed += 1,
            Err(error) => errors.push(format!("{path}: {error}")),
        }
    }
    if errors.is_empty() {
        Ok(completed)
    } else {
        Err(format!(
            "{completed}/{} completed. Recovery journal: {}\n{}",
            paths.len(),
            journal_path.display(),
            errors.join("\n")
        ))
    }
}

fn copy_new(source: &Path, destination: &Path) -> std::io::Result<()> {
    let mut temporary = tempfile::NamedTempFile::new_in(
        destination
            .parent()
            .ok_or_else(|| std::io::Error::other("Missing destination directory"))?,
    )?;
    std::io::copy(&mut fs::File::open(source)?, temporary.as_file_mut())?;
    temporary.as_file().sync_all()?;
    temporary
        .persist_noclobber(destination)
        .map_err(|e| e.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copy_never_overwrites_an_existing_destination() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let destination = dir.path().join("destination");
        fs::write(&source, b"new").unwrap();
        fs::write(&destination, b"original").unwrap();
        assert!(copy_new(&source, &destination).is_err());
        assert_eq!(fs::read(destination).unwrap(), b"original");
        assert_eq!(fs::read(source).unwrap(), b"new");
    }
}
