//! Cross-platform storage root definitions and path resolution.
//!
//! Enables distributed workstations (Windows, macOS, Linux) to collaborate
//! over a shared NAS mount without hardcoding OS-specific absolute drive letters
//! or mount paths into the shared database.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a physical or network storage vault in the central library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageRoot {
    /// Unique identifier for this storage root (e.g. UUID v4 or slug).
    pub root_uuid: String,
    /// Friendly display name for the user (e.g. "Studio Vault 1").
    pub display_name: String,
    /// Storage mount type: "local_mount", "smb", "s3", etc.
    pub root_type: String,
    /// Creation timestamp in unix seconds.
    pub created_at: i64,
    /// Last update timestamp in unix seconds.
    pub updated_at: i64,
}

/// A normalized path tuple referencing a file inside a StorageRoot.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NormalizedPath {
    /// Root UUID referencing a `StorageRoot`.
    pub root_uuid: String,
    /// Forward-slash normalized relative path inside the root (e.g. "renders/daily/image.png").
    pub relative_path: String,
}

impl NormalizedPath {
    pub fn new(root_uuid: impl Into<String>, relative_path: impl AsRef<str>) -> Self {
        Self {
            root_uuid: root_uuid.into(),
            relative_path: PathResolver::normalize_relative_path(relative_path.as_ref()),
        }
    }
}

/// Resolves paths between local machine-specific absolute paths and canonical
/// `(root_uuid, relative_path)` tuples using client mount configurations.
#[derive(Debug, Clone, Default)]
pub struct PathResolver {
    /// Map from root_uuid to local absolute base directory.
    mappings: HashMap<String, PathBuf>,
}

impl PathResolver {
    /// Create an empty resolver.
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    /// Create a resolver from an existing mapping table.
    pub fn from_mappings(mappings: HashMap<String, String>) -> Self {
        let mut resolver = Self::new();
        for (root_uuid, local_path) in mappings {
            resolver.register_root(root_uuid, PathBuf::from(local_path));
        }
        resolver
    }

    /// Register or update a local mount mapping for a root UUID.
    pub fn register_root(
        &mut self,
        root_uuid: impl Into<String>,
        local_base_path: impl Into<PathBuf>,
    ) {
        self.mappings
            .insert(root_uuid.into(), local_base_path.into());
    }

    /// Remove a root mapping.
    pub fn unregister_root(&mut self, root_uuid: &str) -> Option<PathBuf> {
        self.mappings.remove(root_uuid)
    }

    /// Returns the local absolute base directory registered for a root UUID, if any.
    pub fn get_local_path(&self, root_uuid: &str) -> Option<&Path> {
        self.mappings.get(root_uuid).map(|p| p.as_path())
    }

    /// Returns all registered mappings.
    pub fn mappings(&self) -> &HashMap<String, PathBuf> {
        &self.mappings
    }

    /// Normalizes a relative path string by converting backslashes to forward slashes,
    /// removing duplicate slashes, and stripping leading/trailing slashes.
    pub fn normalize_relative_path(rel: &str) -> String {
        let normalized = rel.replace('\\', "/");
        let parts: Vec<&str> = normalized
            .split('/')
            .filter(|p| !p.is_empty() && *p != ".")
            .collect();
        parts.join("/")
    }

    /// Resolves a `(root_uuid, relative_path)` to an absolute filesystem PathBuf on the current machine.
    pub fn resolve_absolute(&self, root_uuid: &str, relative_path: &str) -> Option<PathBuf> {
        let base = self.get_local_path(root_uuid)?;
        let clean_rel = Self::normalize_relative_path(relative_path);
        if clean_rel.is_empty() {
            Some(base.to_path_buf())
        } else {
            Some(base.join(clean_rel))
        }
    }

    /// Identifies the best matching `root_uuid` and computes the relative path for a given absolute path.
    ///
    /// If multiple roots match (e.g. nested mounts), selects the longest matching prefix.
    pub fn relativize(&self, absolute_path: &Path) -> Option<NormalizedPath> {
        let abs_str = absolute_path.to_string_lossy().replace('\\', "/");
        let mut best_match: Option<(&str, usize)> = None;

        for (root_uuid, base_path) in &self.mappings {
            let base_str = base_path.to_string_lossy().replace('\\', "/");
            let base_clean = base_str.trim_end_matches('/');

            let matches = if abs_str.starts_with(base_clean) {
                if abs_str.len() == base_clean.len() {
                    true
                } else {
                    abs_str.as_bytes().get(base_clean.len()) == Some(&b'/')
                }
            } else {
                false
            };

            if matches {
                let match_len = base_clean.len();
                if best_match.map(|(_, len)| match_len > len).unwrap_or(true) {
                    best_match = Some((root_uuid.as_str(), match_len));
                }
            }
        }

        let (root_uuid, prefix_len) = best_match?;
        let remaining = &abs_str[prefix_len..];
        Some(NormalizedPath::new(root_uuid, remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_root_serde_roundtrip() {
        let root = StorageRoot {
            root_uuid: "vault-central-01".to_string(),
            display_name: "Team NAS Vault".to_string(),
            root_type: "local_mount".to_string(),
            created_at: 1726000000,
            updated_at: 1726000500,
        };
        let json = serde_json::to_string(&root).unwrap();
        assert_eq!(serde_json::from_str::<StorageRoot>(&json).unwrap(), root);
    }

    #[test]
    fn normalize_relative_path_handles_mixed_separators_and_redundant_slashes() {
        assert_eq!(
            PathResolver::normalize_relative_path(r"renders\daily//model_a\\test.png"),
            "renders/daily/model_a/test.png"
        );
        assert_eq!(
            PathResolver::normalize_relative_path("/already/clean/path/"),
            "already/clean/path"
        );
        assert_eq!(
            PathResolver::normalize_relative_path(r".\sub\.\file.png"),
            "sub/file.png"
        );
        assert_eq!(PathResolver::normalize_relative_path(""), "");
    }

    #[test]
    fn path_resolver_resolves_and_relativizes() {
        let mut resolver = PathResolver::new();
        resolver.register_root("vault-01", PathBuf::from("/Volumes/NAS/AIGC"));

        // Relativize sub-path
        let file_path = Path::new("/Volumes/NAS/AIGC/projects/art/gen_01.png");
        let normalized = resolver.relativize(file_path).expect("should relativize");
        assert_eq!(normalized.root_uuid, "vault-01");
        assert_eq!(normalized.relative_path, "projects/art/gen_01.png");

        // Resolve absolute
        let resolved = resolver
            .resolve_absolute("vault-01", "projects/art/gen_01.png")
            .expect("should resolve");
        assert_eq!(
            resolved.to_string_lossy().replace('\\', "/"),
            "/Volumes/NAS/AIGC/projects/art/gen_01.png"
        );

        // Outside path returns None
        assert!(resolver
            .relativize(Path::new("/tmp/unrelated.png"))
            .is_none());
    }

    #[test]
    fn path_resolver_prefers_longest_matching_prefix() {
        let mut resolver = PathResolver::new();
        resolver.register_root("general", PathBuf::from("/mnt/storage"));
        resolver.register_root("aigc_deep", PathBuf::from("/mnt/storage/ai_vault"));

        let file = Path::new("/mnt/storage/ai_vault/model.safetensors");
        let normalized = resolver.relativize(file).unwrap();
        assert_eq!(normalized.root_uuid, "aigc_deep");
        assert_eq!(normalized.relative_path, "model.safetensors");
    }
}
