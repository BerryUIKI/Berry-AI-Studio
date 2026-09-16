//! Recursive folder scanning and indexing.
//!
//! A [`Scanner`] walks a folder recursively, detects each supported media
//! file's container from its magic bytes, upserts rows into the database in
//! batches, and removes rows for files that no longer exist on disk. Metadata
//! extraction is plugged in from `berry-metadata` via [`Scanner::with_extractor`].

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use berry_domain::{Container, ExtractedMetadata, ImageFile};
use berry_storage::{Database, DatabaseError};
use serde::Serialize;
use walkdir::WalkDir;

/// Supported media file extensions, lowercased and without the leading dot.
const MEDIA_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "mp4"];

/// How many file upserts happen per transaction.
const BATCH_SIZE: usize = 256;

/// Errors produced by a scan.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("storage error: {0}")]
    Storage(#[from] DatabaseError),
    #[error("scan root is not an existing directory: {0}")]
    NotADirectory(PathBuf),
    #[error("could not read media file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Progress reported during a scan, after every media file.
#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    /// Id of the folder being scanned.
    pub folder_id: i64,
    /// Media files processed so far (including skipped unchanged ones).
    pub scanned: u64,
    /// Total media files found by the walk (the progress bar's denominator).
    pub found: u64,
    /// The file currently being processed, for display.
    pub current: Option<String>,
}

/// Aggregate outcome of a completed scan.
#[derive(Debug, Clone, Serialize)]
pub struct ScanStats {
    /// Id of the folder that was scanned.
    pub folder_id: i64,
    /// Media files found by the walk.
    pub found: u64,
    /// Rows newly inserted.
    pub added: u64,
    /// Rows updated (content or metadata changed).
    pub updated: u64,
    /// Files skipped because (size, mtime) matched the stored cache.
    pub unchanged: u64,
    /// Rows deleted because the file disappeared from disk.
    pub removed: u64,
    /// Media files that could not be read or recognized.
    pub failed: u64,
    /// Wall-clock duration of the scan, in milliseconds.
    pub duration_ms: u64,
}

/// A media file discovered by the walk, before container detection.
struct MediaFile {
    path: PathBuf,
    size_bytes: u64,
    modified_at: i64,
}

/// A function that extracts metadata from a media file, returning `None` when
/// the file carries no recognizable metadata.
pub type MetadataExtractor =
    Box<dyn Fn(Container, &Path) -> Option<ExtractedMetadata> + Send + Sync>;

/// Scans folders and persists the results through `berry-storage`.
///
/// Each [`scan_folder`](Self::scan_folder) call opens its own database
/// connection to the same file (SQLite WAL allows concurrent readers), so a
/// long scan does not block the app shell's connection used by read commands.
pub struct Scanner {
    db_path: PathBuf,
    /// Whether `extractor` can produce metadata. When false, unchanged files
    /// are skipped purely on (size, mtime); when true, unchanged files that
    /// still lack metadata are re-processed so extraction can fill them in.
    extracts: bool,
    /// When true, every file is re-processed regardless of the (size, mtime)
    /// cache — the "rebuild metadata" mode.
    force_extract: bool,
    extractor: MetadataExtractor,
}

impl Scanner {
    /// A scanner that indexes files without extracting metadata.
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            extracts: false,
            force_extract: false,
            extractor: Box::new(|_, _| None),
        }
    }

    /// A scanner using [`berry_metadata::extract_metadata`] as the extractor.
    ///
    /// This is the production configuration: PNGInfo is extracted now, and the
    /// extractor grows EXIF / sidecar support as `berry-metadata` does.
    pub fn with_default_extractor(db_path: PathBuf) -> Self {
        Self::with_extractor(db_path, berry_metadata::extract_metadata)
    }

    /// A scanner that runs `extractor` on each file to fill in `metadata`.
    ///
    /// Metadata extraction lives in `berry-metadata`; the app shell composes
    /// it here. A `None` return means "no metadata found for this file".
    pub fn with_extractor(
        db_path: PathBuf,
        extractor: impl Fn(Container, &Path) -> Option<ExtractedMetadata> + Send + Sync + 'static,
    ) -> Self {
        Self {
            db_path,
            extracts: true,
            force_extract: false,
            extractor: Box::new(extractor),
        }
    }

    /// A scanner that re-extracts metadata from every file, ignoring the
    /// incremental cache. Used by the "rebuild metadata" action so files
    /// indexed under an older extractor get the current one's output.
    pub fn with_forced_extractor(db_path: PathBuf) -> Self {
        Self {
            db_path,
            extracts: true,
            force_extract: true,
            extractor: Box::new(berry_metadata::extract_metadata),
        }
    }

    /// Scan `root`, indexing every supported media file under `folder_id`.
    ///
    /// `on_progress` is called after each media file is processed (and once
    /// more with `current: None` when the scan finishes).
    pub fn scan_folder(
        &self,
        folder_id: i64,
        root: &Path,
        mut on_progress: impl FnMut(ScanProgress),
    ) -> Result<ScanStats, ScanError> {
        if !root.is_dir() {
            return Err(ScanError::NotADirectory(root.to_path_buf()));
        }

        let started = Instant::now();
        let files = collect_media_files(root);
        let found = files.len() as u64;

        let mut stats = ScanStats {
            folder_id,
            found,
            added: 0,
            updated: 0,
            unchanged: 0,
            removed: 0,
            failed: 0,
            duration_ms: 0,
        };

        // Cache what the database already knows so unchanged files are skipped
        // without reopening them.
        let db = Database::connect(&self.db_path)?;
        let existing: HashMap<String, (u64, i64, bool)> = db
            .list_file_fingerprints(folder_id)?
            .into_iter()
            .map(|(path, size, mtime, has_meta)| (path, (size, mtime, has_meta)))
            .collect();

        // Paths seen this run; rows indexed before but not seen are removed at
        // the end (orphan cleanup). Failed files are kept in the list so a
        // temporarily unreadable file does not lose its row.
        let mut seen: Vec<String> = Vec::with_capacity(files.len());
        // Files awaiting upsert, flushed in batches of BATCH_SIZE.
        let mut pending: Vec<ImageFile> = Vec::with_capacity(BATCH_SIZE);

        let mut scanned = 0u64;
        for file in files {
            scanned += 1;
            let path_str = file.path.to_string_lossy().to_string();
            let current = Some(path_str.clone());

            // Skip unchanged files that already have everything this scan
            // would produce (incremental scan). A forced rebuild bypasses the
            // cache so every file is re-extracted.
            let cache = existing.get(&path_str);
            let unchanged = !self.force_extract
                && cache.is_some_and(|(size, mtime, has_metadata)| {
                    *size == file.size_bytes
                        && *mtime == file.modified_at
                        && (*has_metadata || !self.extracts)
                });
            if unchanged {
                stats.unchanged += 1;
                seen.push(path_str);
                on_progress(ScanProgress {
                    folder_id,
                    scanned,
                    found,
                    current,
                });
                continue;
            }

            // Determine the container; fall back to the extension when the
            // magic bytes are unrecognizable.
            let container = match detect_container(&file.path) {
                Ok(Some(container)) => container,
                Ok(None) | Err(_) => {
                    stats.failed += 1;
                    seen.push(path_str);
                    on_progress(ScanProgress {
                        folder_id,
                        scanned,
                        found,
                        current,
                    });
                    continue;
                }
            };

            let metadata = (self.extractor)(container, &file.path);
            pending.push(ImageFile {
                id: None,
                folder_id,
                path: path_str.clone(),
                size_bytes: file.size_bytes,
                modified_at: file.modified_at,
                container,
                metadata,
                rating: None,
                aesthetic_score: None,
                is_favorite: false,
                is_nsfw: false,
                stack_id: None,
                stack_order: 0,
            });

            if cache.is_some() {
                stats.updated += 1;
            } else {
                stats.added += 1;
            }

            if pending.len() >= BATCH_SIZE {
                db.upsert_files(&pending)?;
                pending.clear();
            }

            seen.push(path_str);
            on_progress(ScanProgress {
                folder_id,
                scanned,
                found,
                current,
            });
        }

        if !pending.is_empty() {
            db.upsert_files(&pending)?;
        }

        // Drop rows for files that were indexed before but are gone now.
        stats.removed = db.delete_files_not_in(folder_id, &seen)?;

        stats.duration_ms = started.elapsed().as_millis() as u64;
        on_progress(ScanProgress {
            folder_id,
            scanned,
            found,
            current: None,
        });
        Ok(stats)
    }
}

/// Recursively collect supported media files under `root`, skipping hidden
/// directories. Unreadable entries are skipped without failing the scan.
fn collect_media_files(root: &Path) -> Vec<MediaFile> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !is_hidden(entry))
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() || !is_media(entry.path()) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        files.push(MediaFile {
            path: entry.path().to_path_buf(),
            size_bytes: meta.len(),
            modified_at,
        });
    }
    files
}

/// Whether the walker should descend into `entry` (hidden directories are
/// pruned, e.g. `.git`).
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir()
        && entry
            .file_name()
            .to_str()
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
}

/// Whether `path` has a supported media extension.
fn is_media(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Detect a file's container from its magic bytes, falling back to the
/// extension when the bytes are unrecognizable.
///
/// `Err` means the file could not be opened or read; `Ok(None)` means neither
/// magic bytes nor extension identified a supported container.
fn detect_container(path: &Path) -> Result<Option<Container>, ScanError> {
    let mut buf = [0u8; 16];
    let n = {
        let mut file = File::open(path).map_err(|source| ScanError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        file.read(&mut buf).map_err(|source| ScanError::Read {
            path: path.to_path_buf(),
            source,
        })?
    };

    if let Some(container) = berry_metadata::detect_container(&buf[..n]) {
        return Ok(Some(container));
    }

    Ok(container_from_extension(path))
}

/// Container implied by a file's extension (`.jpeg` maps to Jpeg).
fn container_from_extension(path: &Path) -> Option<Container> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => Some(Container::Png),
        Some("jpg") | Some("jpeg") => Some(Container::Jpeg),
        Some("webp") => Some(Container::WebP),
        Some("mp4") => Some(Container::Mp4),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use berry_domain::MetadataFormat;

    struct TestEnv {
        dir: PathBuf,
        db: PathBuf,
        images: PathBuf,
    }

    fn setup(name: &str) -> TestEnv {
        let dir = std::env::temp_dir().join(format!("berry-scan-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let images = dir.join("images");
        std::fs::create_dir_all(&images).unwrap();
        TestEnv {
            dir: dir.clone(),
            db: dir.join("test.db"),
            images: dir.join("images"),
        }
    }

    fn write(path: &Path, content: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn png(content: &[u8]) -> Vec<u8> {
        [
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A][..],
            content,
        ]
        .concat()
    }

    /// Build a PNG chunk (the walker does not validate CRC, so zeros are fine).
    fn chunk(chunk_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(chunk_type);
        out.extend_from_slice(data);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    /// A minimal PNG carrying an A1111 `parameters` tEXt chunk.
    fn a1111_png(parameters: &str) -> Vec<u8> {
        let ihdr = chunk(b"IHDR", &[0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0]);
        let mut tex_data = b"parameters\x00".to_vec();
        tex_data.extend_from_slice(parameters.as_bytes());
        let tex = chunk(b"tEXt", &tex_data);
        let iend = chunk(b"IEND", &[]);
        png(&[ihdr, tex, iend].concat())
    }

    fn jpg(content: &[u8]) -> Vec<u8> {
        [&[0xFF, 0xD8, 0xFF, 0xE0, 0x00][..], content].concat()
    }

    fn webp() -> Vec<u8> {
        b"RIFF\x00\x00\x00\x00WEBP\x10\x00\x00\x00".to_vec()
    }

    fn mp4() -> Vec<u8> {
        b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00".to_vec()
    }

    fn scan(env: &TestEnv, folder_id: i64) -> ScanStats {
        Scanner::new(env.db.clone())
            .scan_folder(folder_id, &env.images, |_| {})
            .unwrap()
    }

    fn paths(env: &TestEnv, rel: &str) -> String {
        let mut path = env.images.clone();
        for part in rel.split(['/', '\\']) {
            path.push(part);
        }
        path.to_string_lossy().to_string()
    }

    #[test]
    fn scan_indexes_media_files_recursively() {
        let env = setup("index");
        write(&env.images.join("a.png"), &png(b"hello"));
        write(&env.images.join("b.jpg"), &jpg(b"photo"));
        write(&env.images.join("sub/c.webp"), &webp());
        write(&env.images.join("sub/d.mp4"), &mp4());
        write(&env.images.join("notes.txt"), b"sidecar");
        write(&env.images.join("data.bin"), b"\x00\x01\x02");

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        let stats = scan(&env, folder.id);
        assert_eq!(stats.found, 4);
        assert_eq!(stats.added, 4);
        assert_eq!(stats.unchanged, 0);
        assert_eq!(stats.removed, 0);
        assert_eq!(stats.failed, 0);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 4, "txt and bin files are not indexed");
        let by_path: HashMap<&str, &ImageFile> =
            files.iter().map(|f| (f.path.as_str(), f)).collect();

        assert_eq!(
            by_path[paths(&env, "a.png").as_str()].container,
            Container::Png
        );
        assert_eq!(
            by_path[paths(&env, "b.jpg").as_str()].container,
            Container::Jpeg
        );
        assert_eq!(
            by_path[paths(&env, "sub/c.webp").as_str()].container,
            Container::WebP
        );
        assert_eq!(
            by_path[paths(&env, "sub/d.mp4").as_str()].container,
            Container::Mp4
        );
        assert!(!by_path.contains_key(paths(&env, "notes.txt").as_str()));

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn incremental_scan_skips_unchanged_files() {
        let env = setup("incremental");
        write(&env.images.join("a.png"), &png(b"one"));
        write(&env.images.join("b.jpg"), &jpg(b"two"));

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        let first = scan(&env, folder.id);
        assert_eq!(first.added, 2);

        let second = scan(&env, folder.id);
        assert_eq!(second.unchanged, 2);
        assert_eq!(second.added, 0);
        assert_eq!(second.updated, 0);
        assert_eq!(second.removed, 0);

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn changed_file_is_updated() {
        let env = setup("changed");
        write(&env.images.join("a.png"), &png(b"short"));
        write(&env.images.join("b.jpg"), &jpg(b"stable"));

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        scan(&env, folder.id);

        // Rewrite a.png with different content (and thus size).
        write(
            &env.images.join("a.png"),
            &png(b"a much longer body changes the size"),
        );
        let stats = scan(&env, folder.id);
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.unchanged, 1);

        let files = db.list_files(folder.id).unwrap();
        let a = files
            .iter()
            .find(|f| f.path == paths(&env, "a.png"))
            .unwrap();
        assert_eq!(
            a.size_bytes,
            png(b"a much longer body changes the size").len() as u64
        );

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn missing_file_row_is_removed() {
        let env = setup("orphan");
        write(&env.images.join("a.png"), &png(b"one"));
        write(&env.images.join("b.jpg"), &jpg(b"two"));

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();
        scan(&env, folder.id);

        std::fs::remove_file(env.images.join("b.jpg")).unwrap();
        let stats = scan(&env, folder.id);
        assert_eq!(stats.removed, 1);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, paths(&env, "a.png"));

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn scanning_missing_root_errors() {
        let env = setup("missing");
        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder("/does/not/exist").unwrap();

        let err = Scanner::new(env.db.clone())
            .scan_folder(folder.id, Path::new("/does/not/exist"), |_| {})
            .unwrap_err();
        assert!(matches!(err, ScanError::NotADirectory(_)));

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn progress_reaches_found_count() {
        let env = setup("progress");
        write(&env.images.join("a.png"), &png(b"one"));
        write(&env.images.join("b.jpg"), &jpg(b"two"));
        write(&env.images.join("sub/c.webp"), &webp());

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        let mut last_seen = 0u64;
        let mut max_scanned = 0u64;
        Scanner::new(env.db.clone())
            .scan_folder(folder.id, &env.images, |p| {
                last_seen = p.scanned;
                max_scanned = max_scanned.max(p.scanned);
            })
            .unwrap();
        assert_eq!(max_scanned, 3);
        assert_eq!(last_seen, 3);

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn scan_extracts_png_parameters() {
        let env = setup("extract");
        write(
            &env.images.join("a.png"),
            &a1111_png(
                "a cat on a couch\nNegative prompt: blurry\nSteps: 20, Sampler: Euler a, \
                 CFG scale: 6, Seed: 42, Size: 512x768, Model hash: abc123, \
                 Model: realisticVision.safetensors",
            ),
        );

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        Scanner::with_default_extractor(env.db.clone())
            .scan_folder(folder.id, &env.images, |_| {})
            .unwrap();

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 1);
        let meta = files[0].metadata.as_ref().expect("metadata extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a cat on a couch"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(meta.steps, Some(20));
        assert_eq!(meta.seed.as_deref(), Some("42"));
        assert_eq!(meta.width, Some(512));
        assert_eq!(meta.height, Some(768));
        assert_eq!(meta.sampler.as_deref(), Some("Euler a"));
        assert_eq!(meta.cfg_scale, Some(6.0));
        assert_eq!(
            meta.model_name.as_deref(),
            Some("realisticVision.safetensors")
        );
        assert_eq!(meta.model_hash.as_deref(), Some("abc123"));

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn scan_with_extractor_backfills_missing_metadata() {
        let env = setup("backfill");
        write(
            &env.images.join("a.png"),
            &a1111_png("a robot\nSteps: 5, Sampler: Euler, Size: 512x512"),
        );

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        // First scan without extraction leaves metadata empty.
        Scanner::new(env.db.clone())
            .scan_folder(folder.id, &env.images, |_| {})
            .unwrap();
        let before = db.list_files(folder.id).unwrap();
        assert!(before[0].metadata.is_none());

        // A second scan with extraction fills metadata in even though the file
        // (size, mtime) is unchanged.
        let stats = Scanner::with_default_extractor(env.db.clone())
            .scan_folder(folder.id, &env.images, |_| {})
            .unwrap();
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.unchanged, 0);

        let after = db.list_files(folder.id).unwrap();
        let meta = after[0].metadata.as_ref().expect("metadata backfilled");
        assert_eq!(meta.prompt.as_deref(), Some("a robot"));
        assert_eq!(meta.steps, Some(5));

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn scan_with_extractor_skips_files_that_have_metadata() {
        let env = setup("reskip");
        write(
            &env.images.join("a.png"),
            &a1111_png("a robot\nSteps: 5, Sampler: Euler, Size: 512x512"),
        );

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        Scanner::with_default_extractor(env.db.clone())
            .scan_folder(folder.id, &env.images, |_| {})
            .unwrap();

        // Unchanged files that already carry metadata are skipped entirely.
        let stats = Scanner::with_default_extractor(env.db.clone())
            .scan_folder(folder.id, &env.images, |_| {})
            .unwrap();
        assert_eq!(stats.unchanged, 1);
        assert_eq!(stats.updated, 0);

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn forced_scan_reprocesses_unchanged_files() {
        let env = setup("forced");
        write(
            &env.images.join("a.png"),
            &a1111_png("a robot\nSteps: 5, Sampler: Euler, Size: 512x512"),
        );

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        Scanner::with_default_extractor(env.db.clone())
            .scan_folder(folder.id, &env.images, |_| {})
            .unwrap();
        let before = db.list_files(folder.id).unwrap();
        assert!(before[0].metadata.is_some());

        // A forced rebuild re-processes the unchanged file instead of skipping.
        let stats = Scanner::with_forced_extractor(env.db.clone())
            .scan_folder(folder.id, &env.images, |_| {})
            .unwrap();
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.unchanged, 0);

        let after = db.list_files(folder.id).unwrap();
        let meta = after[0].metadata.as_ref().expect("metadata kept");
        assert_eq!(meta.prompt.as_deref(), Some("a robot"));
        assert_eq!(meta.steps, Some(5));

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }
}
