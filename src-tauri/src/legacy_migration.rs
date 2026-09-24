//! Migration coordinator, state management, and platform cleanup for Berry -> Omera.
use crate::config_store;
use omera_domain::{
    CleanupItem, LegacyCleanupPreview, LegacyCleanupResult, LegacyMigrationJob,
    LegacyMigrationPreview, LegacyMigrationStatus, MigratedArtifact, MigrationError,
    MigrationReceipt,
};
use omera_storage::legacy_migration as storage_migration;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

/// State holder for active and cached migration plans and operations.
#[derive(Default)]
pub struct MigrationCoordinator {
    pub active_job: Option<LegacyMigrationJob>,
    pub active_plans: HashMap<String, LegacyMigrationPreview>,
    pub active_cleanup_previews: HashMap<String, LegacyCleanupPreview>,
    pub migration_lock: Arc<Mutex<()>>,
}

impl MigrationCoordinator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve the overall migration lifecycle status for the application.
    pub fn get_status(&mut self, app: &AppHandle) -> Result<LegacyMigrationStatus, String> {
        let (omera_root, candidates) = resolve_migration_roots(app)?;
        let destination_exists = omera_root.join("omera.db").exists();
        let active_receipt = storage_migration::load_receipt(&omera_root).ok().flatten();

        let mut discovered_sources = Vec::new();
        for (identifier, path) in candidates {
            if path.exists() {
                if let Ok(Some(src)) =
                    storage_migration::discover_database_source(&path, &identifier)
                {
                    discovered_sources.push(src);
                }
            }
        }

        let stage = if let Some(ref receipt) = active_receipt {
            if receipt.cleanup_status == "pending" {
                "cleanup_available".to_string()
            } else {
                "migrated".to_string()
            }
        } else if let Some(ref job) = self.active_job {
            job.status.clone()
        } else if destination_exists {
            "migrated".to_string()
        } else if discovered_sources.is_empty() {
            "none".to_string()
        } else if discovered_sources.len() > 1 {
            "awaiting_source_choice".to_string()
        } else {
            "discovered".to_string()
        };

        let mut available_actions = Vec::new();
        if stage == "cleanup_available" {
            available_actions.push("preview_cleanup".to_string());
            available_actions.push("defer_cleanup".to_string());
        } else if stage == "discovered" || stage == "awaiting_source_choice" {
            available_actions.push("preview_migration".to_string());
        }

        Ok(LegacyMigrationStatus {
            stage,
            discovered_sources,
            destination_exists,
            active_receipt,
            available_actions,
        })
    }

    /// Attempt automatic migration on startup if omera.db does not exist yet
    /// and there is exactly one unambiguous, unlocked legacy source database.
    pub fn auto_migrate_if_unambiguous(&mut self, app: &AppHandle) -> Result<bool, String> {
        let (omera_root, _) = resolve_migration_roots(app)?;
        if omera_root.join("omera.db").exists() {
            return Ok(false);
        }

        let status = self.get_status(app)?;
        if status.discovered_sources.len() == 1 {
            let source = &status.discovered_sources[0];
            if !source.is_locked {
                let preview = self.preview_migration(app, &source.source_id)?;
                if preview.conflicts.is_empty() {
                    let job = self.start_migration(app, &preview.plan_id)?;
                    if job.status == "completed" {
                        eprintln!(
                            "Auto-migrated legacy Berry library from {} to {}",
                            source.database_path,
                            omera_root.join("omera.db").display()
                        );
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Preview a migration plan for a specific discovered source.
    pub fn preview_migration(
        &mut self,
        app: &AppHandle,
        source_id: &str,
    ) -> Result<LegacyMigrationPreview, String> {
        let (omera_root, candidates) = resolve_migration_roots(app)?;
        let mut target_source = None;
        for (identifier, path) in &candidates {
            if let Ok(Some(src)) = storage_migration::discover_database_source(path, identifier) {
                if src.source_id == source_id {
                    target_source = Some(src);
                    break;
                }
            }
        }

        let source = target_source.ok_or_else(|| {
            format!("Discovered source '{source_id}' not found among candidate roots")
        })?;

        let mut conflicts = Vec::new();
        let destination_db = omera_root.join("omera.db");
        if destination_db.exists() {
            conflicts.push("Target database omera.db already exists. An explicit import or manual recovery is required.".into());
        }
        if source.is_locked {
            conflicts.push("Source database is active or locked by another process. Please close Berry AI Studio before migrating.".into());
        }

        let exclusions = vec![
            "External linked directories and custom storage vaults are preserved in place and excluded from migration staging.".into(),
            "Snapshot backup archives (*.zip) are preserved in their original location.".into(),
        ];

        let required_space_bytes = source.database_size_bytes * 2 + 1024 * 1024; // DB copy + staging buffer
        let available_space_bytes = get_available_disk_space(&omera_root).unwrap_or(u64::MAX);

        if available_space_bytes < required_space_bytes {
            conflicts.push(format!(
                "Insufficient disk space on destination volume: required {} bytes, available {} bytes",
                required_space_bytes, available_space_bytes
            ));
        }

        let plan_id = uuid::Uuid::new_v4().to_string();
        let preview = LegacyMigrationPreview {
            plan_id: plan_id.clone(),
            source,
            destination_root: omera_root.to_string_lossy().to_string(),
            destination_db: destination_db.to_string_lossy().to_string(),
            required_space_bytes,
            available_space_bytes,
            conflicts,
            exclusions,
        };

        self.active_plans.insert(plan_id, preview.clone());
        Ok(preview)
    }

    /// Execute a verified migration plan and record a durable receipt.
    pub fn start_migration(
        &mut self,
        _app: &AppHandle,
        plan_id: &str,
    ) -> Result<LegacyMigrationJob, String> {
        let plan =
            self.active_plans.get(plan_id).cloned().ok_or_else(|| {
                format!("Migration plan '{plan_id}' has expired or was not found")
            })?;

        if !plan.conflicts.is_empty() {
            let conflict_msg = plan.conflicts.join("; ");
            return Err(format!(
                "Cannot proceed with migration due to conflicts: {conflict_msg}"
            ));
        }

        let _lock = self
            .migration_lock
            .try_lock()
            .map_err(|_| "Another migration is currently in progress")?;

        let job_id = uuid::Uuid::new_v4().to_string();
        let mut job = LegacyMigrationJob {
            job_id: job_id.clone(),
            plan_id: plan_id.to_string(),
            status: "copying".to_string(),
            progress: 0.1,
            current_step: "Staging and validating SQLite database".to_string(),
            error: None,
            receipt: None,
        };
        self.active_job = Some(job.clone());

        let source_root = PathBuf::from(&plan.source.root_path);
        let source_db = PathBuf::from(&plan.source.database_path);
        let destination_root = PathBuf::from(&plan.destination_root);
        let destination_db = PathBuf::from(&plan.destination_db);

        std::fs::create_dir_all(&destination_root).map_err(|e| e.to_string())?;
        let staging_dir = tempfile::tempdir_in(&destination_root).map_err(|e| e.to_string())?;

        // 1. Stage and migrate SQLite database
        let db_size = match storage_migration::migrate_database(
            &source_db,
            &destination_db,
            staging_dir.path(),
        ) {
            Ok(size) => size,
            Err(e) => {
                let err = MigrationError::new(
                    e.code.clone(),
                    e.message_key.clone(),
                    e.retryable,
                    e.context.clone(),
                );
                job.status = "failed".to_string();
                job.error = Some(err);
                self.active_job = Some(job.clone());
                return Err(format!("Database migration failed: {e}"));
            }
        };

        job.progress = 0.5;
        job.status = "validating".to_string();
        job.current_step = "Validating migrated database and configuration".to_string();
        self.active_job = Some(job.clone());

        // Compute SHA-256 integrity hash of published database
        let integrity_hash = compute_file_sha256(&destination_db).unwrap_or_default();

        let mut artifacts = vec![MigratedArtifact {
            category: "database".to_string(),
            source_path: source_db.to_string_lossy().to_string(),
            destination_path: destination_db.to_string_lossy().to_string(),
            status: "success".to_string(),
            size_bytes: db_size,
        }];

        // 2. Migrate configuration if present
        let source_config_path = source_root.join("config.json");
        let dest_config_path = destination_root.join("config.json");
        if source_config_path.exists() {
            match config_store::load(&source_config_path) {
                Ok(legacy_cfg) => {
                    // Migrate credentials into new Omera keyring
                    let _ = config_store::migrate_credentials_to_omera(&legacy_cfg);

                    // If destination config exists, keep destination authoritative;
                    // otherwise save migrated config.
                    if !dest_config_path.exists() {
                        let _ = config_store::save(&dest_config_path, legacy_cfg);
                    }

                    let config_size = std::fs::metadata(&source_config_path)
                        .map(|m| m.len())
                        .unwrap_or(0);

                    artifacts.push(MigratedArtifact {
                        category: "config".to_string(),
                        source_path: source_config_path.to_string_lossy().to_string(),
                        destination_path: dest_config_path.to_string_lossy().to_string(),
                        status: "success".to_string(),
                        size_bytes: config_size,
                    });
                }
                Err(e) => {
                    eprintln!(
                        "Warning: could not read legacy config {}: {e}",
                        source_config_path.display()
                    );
                }
            }
        }

        job.progress = 0.8;
        job.status = "activating".to_string();
        job.current_step = "Writing durable migration receipt".to_string();
        self.active_job = Some(job.clone());

        // 3. Write durable migration receipt
        let receipt_id = uuid::Uuid::new_v4().to_string();
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let receipt = MigrationReceipt {
            receipt_id,
            source_id: plan.source.source_id.clone(),
            source_identifier: plan.source.identifier.clone(),
            source_root: source_root.to_string_lossy().to_string(),
            source_schema_version: plan.source.schema_version,
            destination_root: destination_root.to_string_lossy().to_string(),
            destination_db: destination_db.to_string_lossy().to_string(),
            created_at: now_sec,
            artifacts,
            integrity_hash,
            cleanup_status: "pending".to_string(),
        };

        if let Err(e) = storage_migration::save_receipt(&destination_root, &receipt) {
            job.status = "failed".to_string();
            job.error = Some(MigrationError::new(
                "INTEGRITY_FAILED",
                "error.migration.receipt_failed",
                false,
                Some(e.clone()),
            ));
            self.active_job = Some(job.clone());
            return Err(format!("Failed to record durable receipt: {e}"));
        }

        job.progress = 1.0;
        job.status = "completed".to_string();
        job.current_step = "Migration completed successfully".to_string();
        job.receipt = Some(receipt);
        self.active_job = Some(job.clone());

        Ok(job)
    }

    /// Retrieve status of an active or recent migration job.
    pub fn get_job(&self, job_id: &str) -> Result<LegacyMigrationJob, String> {
        self.active_job
            .as_ref()
            .filter(|j| j.job_id == job_id)
            .cloned()
            .ok_or_else(|| format!("Job '{job_id}' not found"))
    }

    /// Preview eligible legacy application-owned files for user-approved cleanup.
    pub fn preview_cleanup(
        &mut self,
        app: &AppHandle,
        receipt_id: &str,
    ) -> Result<LegacyCleanupPreview, String> {
        let (omera_root, _) = resolve_migration_roots(app)?;
        let receipt = storage_migration::load_receipt(&omera_root)?
            .filter(|r| r.receipt_id == receipt_id)
            .ok_or_else(|| format!("Receipt '{receipt_id}' not found"))?;

        // 1. Verify destination health
        let dest_db = PathBuf::from(&receipt.destination_db);
        let destination_healthy = storage_migration::verify_destination_health(&dest_db)
            .map_err(|e| format!("Destination check failed: {e}"))?;

        if !destination_healthy {
            return Err("Destination database is unhealthy or missing; cleanup blocked to protect source data.".into());
        }

        // 2. Scan source root and collect eligible app-owned files
        let source_root = PathBuf::from(&receipt.source_root);
        if !source_root.exists() {
            return Err(format!(
                "Source root '{}' no longer exists",
                source_root.display()
            ));
        }

        let mut items = Vec::new();
        let mut total_size_bytes = 0u64;

        if let Ok(entries) = std::fs::read_dir(&source_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Whitelisted application-owned categories
                let (category, eligible, reason) = match name_str.as_ref() {
                    "berry.db" | "berry.db-wal" | "berry.db-shm" => {
                        ("database".to_string(), true, None)
                    }
                    "config.json" | "config.json.bak" => ("config".to_string(), true, None),
                    "thumbnails" => ("thumbnails".to_string(), true, None),
                    "models" => ("models".to_string(), true, None),
                    "updates" => ("updates".to_string(), true, None),
                    other => {
                        // User assets, external vaults, backups, or unexpected files
                        if other.ends_with(".zip") {
                            (
                                "backup".to_string(),
                                false,
                                Some("Preserved backup archive".into()),
                            )
                        } else {
                            (
                                "user_data".to_string(),
                                false,
                                Some("Protected user directory or asset".into()),
                            )
                        }
                    }
                };

                let size = if path.is_file() {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                } else if path.is_dir() {
                    calculate_folder_size(&path)
                } else {
                    0
                };

                if eligible {
                    total_size_bytes += size;
                }

                items.push(CleanupItem {
                    path: path.to_string_lossy().to_string(),
                    category,
                    size_bytes: size,
                    eligible,
                    reason,
                });
            }
        }

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let preview_id = uuid::Uuid::new_v4().to_string();
        let preview = LegacyCleanupPreview {
            preview_id: preview_id.clone(),
            receipt_id: receipt_id.to_string(),
            expires_at: now_sec + 300, // Valid for 5 minutes
            items,
            total_size_bytes,
            destination_healthy,
        };

        self.active_cleanup_previews
            .insert(preview_id, preview.clone());
        Ok(preview)
    }

    /// Execute user-confirmed cleanup of eligible legacy artifacts using system Trash.
    pub fn confirm_cleanup(
        &mut self,
        app: &AppHandle,
        preview_id: &str,
        confirmed: bool,
    ) -> Result<LegacyCleanupResult, String> {
        if !confirmed {
            return Err("Cleanup was not confirmed by user".into());
        }

        let preview = self
            .active_cleanup_previews
            .remove(preview_id)
            .ok_or_else(|| format!("Cleanup preview '{preview_id}' not found or expired"))?;

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now_sec > preview.expires_at {
            return Err("Cleanup preview has expired. Please review and preview again.".into());
        }

        let (omera_root, _) = resolve_migration_roots(app)?;
        let mut receipt = storage_migration::load_receipt(&omera_root)?
            .filter(|r| r.receipt_id == preview.receipt_id)
            .ok_or_else(|| format!("Receipt '{}' not found", preview.receipt_id))?;

        // Re-verify destination health immediately prior to disposal
        let dest_db = PathBuf::from(&receipt.destination_db);
        if !storage_migration::verify_destination_health(&dest_db).unwrap_or(false) {
            return Err("Destination health verification failed immediately prior to cleanup. Operation aborted.".into());
        }

        let mut cleaned_count = 0;
        let mut failed_count = 0;
        let mut cleaned_bytes = 0u64;
        let mut errors = Vec::new();

        let source_root_canon = PathBuf::from(&receipt.source_root)
            .canonicalize()
            .map_err(|e| e.to_string())?;

        for item in preview.items {
            if !item.eligible {
                continue;
            }

            let path = PathBuf::from(&item.path);
            if !path.exists() {
                continue;
            }

            // Ensure path does not escape the legacy root
            if let Ok(canon) = path.canonicalize() {
                if !canon.starts_with(&source_root_canon) {
                    errors.push(format!(
                        "Rejected path outside source root: {}",
                        path.display()
                    ));
                    failed_count += 1;
                    continue;
                }
            } else {
                continue;
            }

            // Move to system Trash. NEVER use permanent deletion!
            match trash::delete(&path) {
                Ok(_) => {
                    cleaned_count += 1;
                    cleaned_bytes += item.size_bytes;
                }
                Err(e) => {
                    failed_count += 1;
                    errors.push(format!("Could not trash {}: {e}", path.display()));
                }
            }
        }

        // Clean up legacy credentials safely
        let dest_config_path = omera_root.join("config.json");
        if dest_config_path.exists() {
            if let Ok(config) = config_store::load(&dest_config_path) {
                let _ = config_store::cleanup_legacy_credentials(&config);
            }
        }

        let status = if failed_count == 0 {
            "cleaned".to_string()
        } else {
            "partially_cleaned".to_string()
        };

        receipt.cleanup_status = status.clone();
        let _ = storage_migration::save_receipt(&omera_root, &receipt);

        Ok(LegacyCleanupResult {
            receipt_id: preview.receipt_id,
            cleaned_count,
            failed_count,
            cleaned_bytes,
            errors,
            status,
        })
    }

    /// Persist user decision to defer cleanup and avoid repetitive launch prompts.
    pub fn defer_cleanup(&mut self, app: &AppHandle, receipt_id: &str) -> Result<(), String> {
        let (omera_root, _) = resolve_migration_roots(app)?;
        let mut receipt = storage_migration::load_receipt(&omera_root)?
            .filter(|r| r.receipt_id == receipt_id)
            .ok_or_else(|| format!("Receipt '{receipt_id}' not found"))?;

        receipt.cleanup_status = "deferred".to_string();
        storage_migration::save_receipt(&omera_root, &receipt)
    }
}

/// Resolve candidate legacy roots and target Omera root from the runtime application environment.
pub fn resolve_migration_roots(
    app: &AppHandle,
) -> Result<(PathBuf, Vec<(String, PathBuf)>), String> {
    let current_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let parent = current_data_dir
        .parent()
        .ok_or("Cannot resolve parent of app_data_dir")?;
    let omera_root = parent.join("com.berryuiki.omera");

    let mut candidates = Vec::new();
    let legacy_1 = parent.join("com.berryuiki.berryaistudio");
    if legacy_1 != omera_root {
        candidates.push(("com.berryuiki.berryaistudio".to_string(), legacy_1));
    }
    let legacy_2 = parent.join("com.berryuiki.berryaigctoolbox");
    if legacy_2 != omera_root {
        candidates.push(("com.berryuiki.berryaigctoolbox".to_string(), legacy_2));
    }
    if current_data_dir.join("berry.db").exists()
        && !candidates.iter().any(|(_, p)| p == &current_data_dir)
    {
        candidates.push(("legacy_in_root".to_string(), current_data_dir));
    }

    Ok((omera_root, candidates))
}

fn compute_file_sha256(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| e.to_string())?;
    Ok(hex::encode(hasher.finalize()))
}

fn calculate_folder_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() && !meta.is_symlink() {
                    total += calculate_folder_size(&entry.path());
                }
            }
        }
    }
    total
}

#[cfg(unix)]
fn get_available_disk_space(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    let c_path = CString::new(path.to_string_lossy().as_bytes()).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } == 0 {
        Some(stat.f_bavail as u64 * stat.f_frsize as u64)
    } else {
        None
    }
}

#[cfg(windows)]
fn get_available_disk_space(_path: &Path) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_defaults() {
        let coordinator = MigrationCoordinator::new();
        assert!(coordinator.active_job.is_none());
        assert!(coordinator.active_plans.is_empty());
        assert!(coordinator.active_cleanup_previews.is_empty());
        assert!(coordinator.get_job("non-existent").is_err());
    }

    #[test]
    fn test_compute_file_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, b"hello world").unwrap();
        let hash = compute_file_sha256(&file_path).unwrap();
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_calculate_folder_size() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("a.txt"), b"12345").unwrap();
        std::fs::write(dir.path().join("b.txt"), b"67890").unwrap();
        assert_eq!(calculate_folder_size(dir.path()), 10);
    }
}
