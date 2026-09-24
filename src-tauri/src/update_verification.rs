use minisign_verify::{PublicKey, Signature};
use std::{fs::File, io::Read, path::Path};

pub fn trusted_key() -> Result<PublicKey, String> {
    let key_str = option_env!("OMERA_UPDATE_PUBLIC_KEY")
        .or(option_env!("BERRY_UPDATE_PUBLIC_KEY"))
        .filter(|key| !key.is_empty())
        .ok_or("Signed automatic updates are not configured. Use the release page for manual installation.")?;
    PublicKey::from_base64(key_str).map_err(|e| format!("Invalid embedded update key: {e}"))
}

pub fn validate_url(url: &str) -> Result<(), String> {
    let parsed = tauri::Url::parse(url).map_err(|e| e.to_string())?;
    let path = parsed.path();
    let is_valid_repo = path.starts_with("/BerryUIKI/Omera/releases/download/")
        || path.starts_with("/BerryUIKI/Berry-AI-Studio/releases/download/");
    if parsed.scheme() != "https"
        || parsed.host_str() != Some("github.com")
        || !is_valid_repo
        || parsed.query().is_some()
    {
        return Err("Update URL must reference an official HTTPS release asset".into());
    }
    Ok(())
}

pub fn verify(path: &Path, signature: &str) -> Result<(), String> {
    verify_with_key(path, signature, &trusted_key()?)
}

fn verify_with_key(path: &Path, signature: &str, key: &PublicKey) -> Result<(), String> {
    let signature =
        Signature::decode(signature).map_err(|e| format!("Invalid update signature: {e}"))?;
    let mut verifier = key.verify_stream(&signature).map_err(|e| e.to_string())?;
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        verifier.update(&buffer[..count]);
    }
    verifier
        .finalize()
        .map_err(|e| format!("Update signature verification failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_untrusted_update_origins() {
        assert!(validate_url(
            "https://github.com/BerryUIKI/Omera/releases/download/v0.3.0/Omera_Windows_x64.exe"
        )
        .is_ok());
        assert!(validate_url(
            "https://github.com/BerryUIKI/Berry-AI-Studio/releases/download/v1/setup.exe"
        )
        .is_ok());
        assert!(validate_url(
            "http://github.com/BerryUIKI/Omera/releases/download/v0.3.0/Omera_Windows_x64.exe"
        )
        .is_err());
        assert!(
            validate_url("https://github.com/other/repo/releases/download/v1/setup.exe").is_err()
        );
        assert!(validate_url(
            "https://github.com.evil.test/BerryUIKI/Omera/releases/download/v0.3.0/Omera_Windows_x64.exe"
        )
        .is_err());
    }
}
