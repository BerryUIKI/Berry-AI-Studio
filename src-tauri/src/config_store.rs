//! Serialized, atomic configuration persistence; secrets live in the OS credential store.
use crate::commands::AppConfig;
use std::{io::Write, path::Path, sync::Mutex};

static CONFIG_LOCK: Mutex<()> = Mutex::new(());
const PREFIX: &str = "keyring:";

fn resolve(value: &mut Option<String>) -> Result<(), String> {
    if let Some(account) = value.as_deref().and_then(|s| s.strip_prefix(PREFIX)) {
        let password = keyring::Entry::new("Omera", account)
            .ok()
            .and_then(|e| e.get_password().ok())
            .or_else(|| {
                keyring::Entry::new("Berry-AI-Studio", account)
                    .ok()
                    .and_then(|e| e.get_password().ok())
            })
            .or_else(|| {
                keyring::Entry::new("Berry-AIGC-Toolbox", account)
                    .ok()
                    .and_then(|e| e.get_password().ok())
            })
            .ok_or_else(|| "Credential store unavailable or entry not found".to_string())?;
        *value = Some(password);
    }
    Ok(())
}

fn protect(value: &mut Option<String>) -> Result<(), String> {
    if let Some(secret) = value
        .as_deref()
        .filter(|s| !s.is_empty() && !s.starts_with(PREFIX))
    {
        let account = uuid::Uuid::new_v4().to_string();
        keyring::Entry::new("Omera", &account)
            .map_err(|e| e.to_string())?
            .set_password(secret)
            .map_err(|e| format!("Could not secure credentials: {e}"))?;
        *value = Some(format!("{PREFIX}{account}"));
    }
    Ok(())
}

fn secrets(
    config: &mut AppConfig,
    operation: fn(&mut Option<String>) -> Result<(), String>,
) -> Result<(), String> {
    operation(&mut config.cloud_backup.webdav_password)?;
    operation(&mut config.cloud_backup.s3_secret_key)?;
    operation(&mut config.cloud_backup.s3_access_key)?;
    let mut remote = Some(std::mem::take(&mut config.remote_connection_url));
    operation(&mut remote)?;
    config.remote_connection_url = remote.unwrap_or_default();
    Ok(())
}

fn read(path: &Path) -> Result<AppConfig, String> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = std::fs::read(path)
        .map_err(|e| format!("Cannot read {}; original preserved: {e}", path.display()))?;
    serde_json::from_slice(&content).map_err(|e| {
        format!(
            "Invalid {}; original preserved. Restore config.json.bak or repair the file: {e}",
            path.display()
        )
    })
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing config directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(contents)
        .and_then(|_| temp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    temp.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load(path: &Path) -> Result<AppConfig, String> {
    let _guard = CONFIG_LOCK.lock().map_err(|e| e.to_string())?;
    let mut config = read(path)?;
    let old = serde_json::to_vec(&config).map_err(|e| e.to_string())?;
    // Migrate plaintext credentials only after the credential store accepts them.
    secrets(&mut config, protect)?;
    let secured = serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?;
    if serde_json::to_vec(&config).map_err(|e| e.to_string())? != old {
        atomic_write(path, &secured)?;
    }
    secrets(&mut config, resolve)?;
    config.storage_backend = "sqlite".into();
    Ok(config)
}

pub fn save(path: &Path, mut config: AppConfig) -> Result<AppConfig, String> {
    let _guard = CONFIG_LOCK.lock().map_err(|e| e.to_string())?;
    let current = read(path)?;
    if current.config_revision != config.config_revision {
        return Err("Settings changed elsewhere. Reload settings and try again.".into());
    }
    config.config_revision += 1;
    config.storage_backend = "sqlite".into();
    let result = config.clone();
    secrets(&mut config, protect)?;
    // Backup only a parsed, secured configuration, never a corrupt or plaintext file.
    let mut backup = current;
    secrets(&mut backup, protect)?;
    atomic_write(
        &path.with_extension("json.bak"),
        &serde_json::to_vec_pretty(&backup).map_err(|e| e.to_string())?,
    )?;
    atomic_write(
        path,
        &serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?,
    )?;
    Ok(result)
}

/// Migrate secret entries from legacy keyring services to the Omera service,
/// verifying readback before marking complete.
pub fn migrate_credentials_to_omera(config: &AppConfig) -> Result<usize, String> {
    let mut accounts = Vec::new();
    let mut collect = |val: &Option<String>| {
        if let Some(account) = val.as_deref().and_then(|s| s.strip_prefix(PREFIX)) {
            accounts.push(account.to_string());
        }
    };
    collect(&config.cloud_backup.webdav_password);
    collect(&config.cloud_backup.s3_secret_key);
    collect(&config.cloud_backup.s3_access_key);
    if let Some(account) = config.remote_connection_url.strip_prefix(PREFIX) {
        accounts.push(account.to_string());
    }

    let mut migrated_count = 0;
    for account in accounts {
        let secret =
            match keyring::Entry::new("Berry-AI-Studio", &account).and_then(|e| e.get_password()) {
                Ok(s) => s,
                Err(_) => match keyring::Entry::new("Berry-AIGC-Toolbox", &account)
                    .and_then(|e| e.get_password())
                {
                    Ok(s) => s,
                    Err(_) => continue,
                },
            };

        let omera_entry = keyring::Entry::new("Omera", &account).map_err(|e| e.to_string())?;
        omera_entry
            .set_password(&secret)
            .map_err(|e| format!("Failed to set Omera keyring: {e}"))?;

        let readback = omera_entry
            .get_password()
            .map_err(|e| format!("Failed to verify Omera keyring: {e}"))?;
        if readback != secret {
            return Err("Credential verification mismatch for Omera service".into());
        }
        migrated_count += 1;
    }
    Ok(migrated_count)
}

/// Clean up legacy keyring entries only after verifying the Omera service has them safely stored.
pub fn cleanup_legacy_credentials(config: &AppConfig) -> Result<usize, String> {
    let mut accounts = Vec::new();
    let mut collect = |val: &Option<String>| {
        if let Some(account) = val.as_deref().and_then(|s| s.strip_prefix(PREFIX)) {
            accounts.push(account.to_string());
        }
    };
    collect(&config.cloud_backup.webdav_password);
    collect(&config.cloud_backup.s3_secret_key);
    collect(&config.cloud_backup.s3_access_key);
    if let Some(account) = config.remote_connection_url.strip_prefix(PREFIX) {
        accounts.push(account.to_string());
    }

    let mut removed = 0;
    for account in accounts {
        if let Ok(omera_entry) = keyring::Entry::new("Omera", &account) {
            if omera_entry.get_password().is_ok() {
                if let Ok(entry) = keyring::Entry::new("Berry-AI-Studio", &account) {
                    let _ = entry.delete_credential();
                    removed += 1;
                }
                if let Ok(entry) = keyring::Entry::new("Berry-AIGC-Toolbox", &account) {
                    let _ = entry.delete_credential();
                }
            }
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrupt_config_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, b"broken").unwrap();
        assert!(load(&path).is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"broken");
    }
    #[test]
    fn rejects_stale_settings_updates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let original = AppConfig::default();
        let saved = save(&path, original.clone()).unwrap();
        assert_eq!(saved.config_revision, 1);
        assert!(save(&path, original).is_err());
        assert!(path.with_extension("json.bak").exists());
    }
}
