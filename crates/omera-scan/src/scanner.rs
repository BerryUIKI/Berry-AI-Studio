//! Recursive folder scanning and indexing.
//!
//! A [`Scanner`] walks a folder recursively, detects each supported media
//! file's container from its magic bytes, upserts rows into the database in
//! batches, and removes rows for files that no longer exist on disk. Metadata
//! extraction is plugged in from `berry-metadata` via [`Scanner::with_extractor`].

use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use omera_domain::{Container, ExtractedMetadata, ImageFile};
use omera_storage::{Database, DatabaseError};
use serde::Serialize;
use walkdir::WalkDir;

/// Supported media file extensions, lowercased and without the leading dot.
const MEDIA_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "mp4", "webm"];

/// How many file upserts happen per transaction.
const BATCH_SIZE: usize = 256;

/// Maximum number of processed files between progress updates.
const PROGRESS_FILE_INTERVAL: u64 = 64;

/// Maximum time between progress updates while processing slow files.
const PROGRESS_TIME_INTERVAL: Duration = Duration::from_millis(100);

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
    /// Media files found so far, or the exact total once discovery finishes.
    pub found: u64,
    /// Whether the full directory walk may still discover more files.
    pub discovering: bool,
    /// The file currently being processed, for display.
    pub current: Option<String>,
}

struct ProgressCadence {
    last_scanned: u64,
    last_emitted_at: Instant,
}

impl ProgressCadence {
    fn new(now: Instant) -> Self {
        Self {
            last_scanned: 0,
            last_emitted_at: now,
        }
    }

    fn should_emit(&mut self, scanned: u64, now: Instant, force: bool) -> bool {
        if !force
            && scanned.saturating_sub(self.last_scanned) < PROGRESS_FILE_INTERVAL
            && now.duration_since(self.last_emitted_at) < PROGRESS_TIME_INTERVAL
        {
            return false;
        }
        self.last_scanned = scanned;
        self.last_emitted_at = now;
        true
    }
}

struct ProgressReporter<F> {
    folder_id: i64,
    cadence: ProgressCadence,
    callback: F,
}

impl<F> ProgressReporter<F>
where
    F: FnMut(ScanProgress),
{
    fn new(folder_id: i64, callback: F) -> Self {
        Self {
            folder_id,
            cadence: ProgressCadence::new(Instant::now()),
            callback,
        }
    }

    fn report(
        &mut self,
        scanned: u64,
        found: u64,
        current: Option<String>,
        discovering: bool,
        force: bool,
    ) {
        if self.cadence.should_emit(
            scanned,
            Instant::now(),
            force || (!discovering && scanned == found),
        ) {
            (self.callback)(ScanProgress {
                folder_id: self.folder_id,
                scanned,
                found,
                discovering,
                current,
            });
        }
    }
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

    /// A scanner using [`omera_metadata::extract_metadata`] as the extractor.
    ///
    /// This is the production configuration: PNGInfo is extracted now, and the
    /// extractor grows EXIF / sidecar support as `omera-metadata` does.
    pub fn with_default_extractor(db_path: PathBuf) -> Self {
        Self::with_extractor(db_path, omera_metadata::extract_metadata)
    }

    /// A scanner that runs `extractor` on each file to fill in `metadata`.
    ///
    /// Metadata extraction lives in `omera-metadata`; the app shell composes
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
            extractor: Box::new(omera_metadata::extract_metadata),
        }
    }

    /// Scan `root`, indexing every supported media file under `folder_id`.
    ///
    /// `on_progress` is rate-limited by file count and elapsed time, then
    /// called once more with `current: None` when the scan finishes.
    pub fn scan_folder(
        &self,
        folder_id: i64,
        root: &Path,
        on_progress: impl FnMut(ScanProgress),
    ) -> Result<ScanStats, ScanError> {
        if !root.is_dir() {
            return Err(ScanError::NotADirectory(root.to_path_buf()));
        }

        let started = Instant::now();
        let mut stats = ScanStats {
            folder_id,
            found: 0,
            added: 0,
            updated: 0,
            unchanged: 0,
            removed: 0,
            failed: 0,
            duration_ms: 0,
        };
        let mut progress = ProgressReporter::new(folder_id, on_progress);
        progress.report(0, 0, None, true, true);

        // Cache what the database already knows so unchanged files are skipped
        // without reopening them.
        let db = Database::connect(&self.db_path)?;
        let mut existing: HashMap<String, (u64, i64, bool)> = db
            .list_file_fingerprints(folder_id)?
            .into_iter()
            .map(|(path, size, mtime, has_meta)| (path, (size, mtime, has_meta)))
            .collect();

        // Files awaiting upsert, flushed in batches of BATCH_SIZE.
        let mut pending: Vec<ImageFile> = Vec::with_capacity(BATCH_SIZE);

        let mut scanned = 0u64;
        for file in walk_media_files(root) {
            stats.found += 1;
            scanned += 1;
            let path_str = file.path.to_string_lossy().to_string();
            let current = Some(path_str.clone());

            // Skip unchanged files that already have everything this scan
            // would produce (incremental scan). A forced rebuild bypasses the
            // cache so every file is re-extracted.
            let cache = existing.remove(&path_str);
            let unchanged = !self.force_extract
                && cache.as_ref().is_some_and(|(size, mtime, has_metadata)| {
                    *size == file.size_bytes
                        && *mtime == file.modified_at
                        && (*has_metadata || !self.extracts)
                });
            if unchanged {
                stats.unchanged += 1;
                progress.report(scanned, stats.found, current, true, false);
                continue;
            }

            // Determine the container; fall back to the extension when the
            // magic bytes are unrecognizable.
            let container = match detect_container(&file.path) {
                Ok(Some(container)) => container,
                Ok(None) | Err(_) => {
                    stats.failed += 1;
                    progress.report(scanned, stats.found, current, true, false);
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

            progress.report(scanned, stats.found, current, true, false);
        }

        if !pending.is_empty() {
            db.upsert_files(&pending)?;
        }

        // Entries remaining in the fingerprint map were not observed during
        // the streaming walk and can be removed without retaining a second
        // full list of paths seen on disk.
        let missing_paths = existing.into_keys().collect::<Vec<_>>();
        stats.removed = db.delete_files_by_paths(folder_id, &missing_paths)?;

        stats.duration_ms = started.elapsed().as_millis() as u64;
        progress.report(scanned, stats.found, None, false, true);
        Ok(stats)
    }

    /// Reconcile only paths reported by the filesystem watcher.
    ///
    /// Existing directories are expanded recursively, deleted paths remove the
    /// matching row or subtree, and sidecar text changes re-index a sibling
    /// image when one exists. Unlike [`scan_folder`](Self::scan_folder), this
    /// method never walks the complete registered root for an ordinary file
    /// event and never performs whole-folder orphan cleanup.
    pub fn reconcile_paths(
        &self,
        folder_id: i64,
        root: &Path,
        changed_paths: &[PathBuf],
        on_progress: impl FnMut(ScanProgress),
    ) -> Result<ScanStats, ScanError> {
        if !root.is_dir() {
            return Err(ScanError::NotADirectory(root.to_path_buf()));
        }

        let started = Instant::now();
        let mut candidates = BTreeSet::new();
        for path in changed_paths.iter().filter(|path| path.starts_with(root)) {
            if path.is_dir() {
                candidates.extend(collect_media_files(path).into_iter().map(|file| file.path));
            } else if is_sidecar(path) {
                candidates.extend(sidecar_image_candidates(path));
            } else {
                candidates.insert(path.clone());
            }
        }

        let found = candidates.len() as u64;
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
        let mut progress = ProgressReporter::new(folder_id, on_progress);
        let db = Database::connect(&self.db_path)?;

        for (index, path) in candidates.into_iter().enumerate() {
            let path_str = path.to_string_lossy().to_string();
            if path.is_file() && is_media(&path) {
                match media_file_from_path(&path).and_then(|file| {
                    detect_container(&file.path).map(|container| (file, container))
                }) {
                    Ok((file, Some(container))) => {
                        let existing = db.get_file_by_path(&path_str)?;
                        let metadata = (self.extractor)(container, &file.path);
                        db.upsert_file(&ImageFile {
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
                        })?;
                        if existing.is_some() {
                            stats.updated += 1;
                        } else {
                            stats.added += 1;
                        }
                    }
                    Ok((_, None)) | Err(_) => stats.failed += 1,
                }
            } else if !path.exists() {
                if db.get_file_by_path(&path_str)?.is_some() {
                    db.delete_file_by_path(&path_str)?;
                    stats.removed += 1;
                }
                stats.removed +=
                    db.delete_files_under_path(folder_id, &path_str, std::path::MAIN_SEPARATOR)?;
            } else {
                stats.unchanged += 1;
            }

            progress.report(index as u64 + 1, found, Some(path_str), false, false);
        }

        stats.duration_ms = started.elapsed().as_millis() as u64;
        progress.report(found, found, None, false, true);
        Ok(stats)
    }
}

fn media_file_from_path(path: &Path) -> Result<MediaFile, ScanError> {
    let meta = path.metadata().map_err(|source| ScanError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let modified_at = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    Ok(MediaFile {
        path: path.to_path_buf(),
        size_bytes: meta.len(),
        modified_at,
    })
}

fn is_sidecar(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
}

fn sidecar_image_candidates(path: &Path) -> Vec<PathBuf> {
    ["png", "jpg", "jpeg", "webp"]
        .into_iter()
        .map(|extension| path.with_extension(extension))
        .filter(|candidate| candidate.is_file())
        .collect()
}

/// Recursively collect supported media files under `root`, skipping hidden
/// directories. Unreadable entries are skipped without failing the scan.
fn collect_media_files(root: &Path) -> Vec<MediaFile> {
    walk_media_files(root).collect()
}

/// Stream supported media files under `root` without retaining the tree.
fn walk_media_files(root: &Path) -> impl Iterator<Item = MediaFile> + '_ {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !is_hidden(entry))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.file_type().is_file() || !is_media(entry.path()) {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let modified_at = meta
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs() as i64)
                .unwrap_or(0);
            Some(MediaFile {
                path: entry.path().to_path_buf(),
                size_bytes: meta.len(),
                modified_at,
            })
        })
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

    if let Some(container) = omera_metadata::detect_container(&buf[..n]) {
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
        Some("webm") => Some(Container::Webm),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use omera_domain::MetadataFormat;

    struct TestEnv {
        _temp: tempfile::TempDir,
        dir: PathBuf,
        db: PathBuf,
        images: PathBuf,
    }

    fn setup(label: &str) -> TestEnv {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().to_path_buf();
        let db = dir.join(format!("{label}.db"));
        let images = dir.join("images");
        std::fs::create_dir_all(&images).unwrap();
        TestEnv {
            _temp: temp,
            dir,
            db,
            images,
        }
    }

    fn write(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, bytes).unwrap();
    }

    fn png(data: &[u8]) -> Vec<u8> {
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend_from_slice(data);
        bytes
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

    fn jpg(data: &[u8]) -> Vec<u8> {
        let mut bytes = b"\xFF\xD8\xFF".to_vec();
        bytes.extend_from_slice(data);
        bytes
    }

    fn webp() -> Vec<u8> {
        b"RIFF\x00\x00\x00\x00WEBP\x10\x00\x00\x00".to_vec()
    }

    fn mp4() -> Vec<u8> {
        b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00".to_vec()
    }

    fn webm() -> Vec<u8> {
        vec![
            0x1A, 0x45, 0xDF, 0xA3, 0x9F, 0x42, 0x86, 0x81, 0x01, 0x42, 0xF7, 0x81, 0x01, 0x42,
            0xF2, 0x81, 0x04, 0x42, 0xF3, 0x81, 0x08, 0x42, 0x82, 0x84, b'w', b'e', b'b', b'm',
        ]
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
        write(&env.images.join("sub/e.webm"), &webm());
        write(&env.images.join("notes.txt"), b"sidecar");
        write(&env.images.join("data.bin"), b"\x00\x01\x02");

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        let stats = scan(&env, folder.id);
        assert_eq!(stats.found, 5);
        assert_eq!(stats.added, 5);
        assert_eq!(stats.unchanged, 0);
        assert_eq!(stats.removed, 0);
        assert_eq!(stats.failed, 0);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 5, "txt and bin files are not indexed");
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
        assert_eq!(
            by_path[paths(&env, "sub/e.webm").as_str()].container,
            Container::Webm
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
    fn reconcile_paths_updates_only_reported_files_and_directories() {
        let env = setup("reconcile");
        write(&env.images.join("a.png"), &png(b"one"));
        write(&env.images.join("b.jpg"), &jpg(b"two"));
        write(&env.images.join("untouched.webp"), &webp());

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();
        scan(&env, folder.id);

        write(&env.images.join("a.png"), &png(b"one changed and longer"));
        std::fs::remove_file(env.images.join("b.jpg")).unwrap();
        write(&env.images.join("new/c.png"), &png(b"three"));
        let stats = Scanner::new(env.db.clone())
            .reconcile_paths(
                folder.id,
                &env.images,
                &[
                    env.images.join("a.png"),
                    env.images.join("b.jpg"),
                    env.images.join("new"),
                ],
                |_| {},
            )
            .unwrap();

        assert_eq!(stats.added, 1);
        assert_eq!(stats.updated, 1);
        assert_eq!(stats.removed, 1);
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files
            .iter()
            .any(|file| file.path == paths(&env, "untouched.webp")));
        assert!(files
            .iter()
            .any(|file| file.path == paths(&env, "new/c.png")));

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn reconcile_paths_removes_deleted_directory_rows() {
        let env = setup("reconcile-dir-delete");
        write(&env.images.join("set/a.png"), &png(b"one"));
        write(&env.images.join("set/nested/b.png"), &png(b"two"));

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();
        scan(&env, folder.id);
        let removed_dir = env.images.join("set");
        std::fs::remove_dir_all(&removed_dir).unwrap();

        let stats = Scanner::new(env.db.clone())
            .reconcile_paths(folder.id, &env.images, &[removed_dir], |_| {})
            .unwrap();
        assert_eq!(stats.removed, 2);
        assert!(db.list_files(folder.id).unwrap().is_empty());

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
        for index in 0..70 {
            write(
                &env.images.join(format!("set/{index}.png")),
                &png(b"content"),
            );
        }

        let db = Database::connect(&env.db).unwrap();
        let folder = db.add_folder(env.images.to_str().unwrap()).unwrap();

        let mut last_seen = 0u64;
        let mut max_scanned = 0u64;
        let mut saw_discovery = false;
        let mut completed_discovery = false;
        Scanner::new(env.db.clone())
            .scan_folder(folder.id, &env.images, |p| {
                last_seen = p.scanned;
                max_scanned = max_scanned.max(p.scanned);
                saw_discovery |= p.discovering;
                completed_discovery = !p.discovering && p.current.is_none();
            })
            .unwrap();
        assert_eq!(max_scanned, 70);
        assert_eq!(last_seen, 70);
        assert!(saw_discovery);
        assert!(completed_discovery);

        drop(db);
        std::fs::remove_dir_all(&env.dir).unwrap();
    }

    #[test]
    fn progress_cadence_limits_fast_updates_and_preserves_slow_updates() {
        let started = Instant::now();
        let mut cadence = ProgressCadence::new(started);

        assert!(!cadence.should_emit(1, started, false));
        assert!(!cadence.should_emit(63, started, false));
        assert!(cadence.should_emit(64, started, false));
        assert!(!cadence.should_emit(65, started, false));
        assert!(cadence.should_emit(65, started + PROGRESS_TIME_INTERVAL, false));
        assert!(cadence.should_emit(66, started + PROGRESS_TIME_INTERVAL, true));
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
