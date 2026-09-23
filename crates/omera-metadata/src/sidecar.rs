//! `.txt` sidecar metadata reading.
//!
//! Some generators (or users organizing their output folders) keep the prompt /
//! parameters in a sibling `<image>.txt` file alongside the image. We read that
//! file and reuse the A1111 parameter parser — sidecars are always treated as
//! A1111-style even when the image itself carries no embedded metadata.

use std::path::Path;

/// Read the sibling `<path>.txt` file's contents, if it exists.
pub fn read_sidecar(path: &Path) -> Option<String> {
    let sidecar = path.with_extension("txt");
    if !sidecar.is_file() {
        return None;
    }
    std::fs::read_to_string(&sidecar).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sibling_txt() {
        let dir = std::env::temp_dir().join(format!("berry-sidecar-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let image = dir.join("image.png");
        std::fs::write(&image, b"fake png").unwrap();
        std::fs::write(dir.join("image.txt"), "a cat\nSteps: 20, Seed: 1").unwrap();

        assert_eq!(
            read_sidecar(&image).as_deref(),
            Some("a cat\nSteps: 20, Seed: 1")
        );

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_sidecar_returns_none() {
        let dir = std::env::temp_dir().join(format!("berry-sidecar-none-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let image = dir.join("image.png");
        std::fs::write(&image, b"fake png").unwrap();

        assert_eq!(read_sidecar(&image), None);

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unreadable_sidecar_returns_none() {
        let dir = std::env::temp_dir().join(format!("berry-sidecar-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let image = dir.join("image.png");
        std::fs::write(&image, b"fake png").unwrap();
        // A directory named image.txt is "not a file", so read_sidecar skips it.
        std::fs::create_dir_all(dir.join("image.txt")).unwrap();

        assert_eq!(read_sidecar(&image), None);

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
