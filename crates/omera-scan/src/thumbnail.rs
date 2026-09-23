//! High-performance thumbnail generation with limited concurrency and disk cache management.

use image::ImageReader;
use omera_storage::{Database, ThumbnailCacheEntry};
use rayon::prelude::*;
use rayon::ThreadPool;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

/// Runtime per-job queue and cache diagnostics.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ThumbnailQueueDiagnostics {
    pub queued: u64,
    pub running: u64,
    pub completed: u64,
    pub canceled: u64,
    pub failed: u64,
    pub reused_tier_hits: u64,
    pub manifest_hits: u64,
    pub active_generation: u64,
}

struct AtomicQueueCounters {
    queued: AtomicU64,
    running: AtomicU64,
    completed: AtomicU64,
    canceled: AtomicU64,
    failed: AtomicU64,
    reused_tier_hits: AtomicU64,
    manifest_hits: AtomicU64,
}

static QUEUE_COUNTERS: AtomicQueueCounters = AtomicQueueCounters {
    queued: AtomicU64::new(0),
    running: AtomicU64::new(0),
    completed: AtomicU64::new(0),
    canceled: AtomicU64::new(0),
    failed: AtomicU64::new(0),
    reused_tier_hits: AtomicU64::new(0),
    manifest_hits: AtomicU64::new(0),
};

/// Retrieve snapshot of thumbnail queue diagnostics.
pub fn get_thumbnail_queue_diagnostics() -> ThumbnailQueueDiagnostics {
    ThumbnailQueueDiagnostics {
        queued: QUEUE_COUNTERS.queued.load(Ordering::Relaxed),
        running: QUEUE_COUNTERS.running.load(Ordering::Relaxed),
        completed: QUEUE_COUNTERS.completed.load(Ordering::Relaxed),
        canceled: QUEUE_COUNTERS.canceled.load(Ordering::Relaxed),
        failed: QUEUE_COUNTERS.failed.load(Ordering::Relaxed),
        reused_tier_hits: QUEUE_COUNTERS.reused_tier_hits.load(Ordering::Relaxed),
        manifest_hits: QUEUE_COUNTERS.manifest_hits.load(Ordering::Relaxed),
        active_generation: 0,
    }
}

/// Reset thumbnail queue diagnostics counters.
pub fn reset_thumbnail_queue_diagnostics() {
    QUEUE_COUNTERS.queued.store(0, Ordering::Relaxed);
    QUEUE_COUNTERS.running.store(0, Ordering::Relaxed);
    QUEUE_COUNTERS.completed.store(0, Ordering::Relaxed);
    QUEUE_COUNTERS.canceled.store(0, Ordering::Relaxed);
    QUEUE_COUNTERS.failed.store(0, Ordering::Relaxed);
    QUEUE_COUNTERS.reused_tier_hits.store(0, Ordering::Relaxed);
    QUEUE_COUNTERS.manifest_hits.store(0, Ordering::Relaxed);
}

/// Dedicated background thread pool with strictly limited concurrency
/// to ensure the main UI and WebView are never starved of CPU or disk I/O.
static THUMB_POOL: OnceLock<ThreadPool> = OnceLock::new();
static RECENT_TOUCHES: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();
const ACCESS_PERSIST_INTERVAL_SECONDS: i64 = 3600;
const MAX_RECENT_TOUCHES: usize = 100_000;

fn get_thumb_pool() -> &'static ThreadPool {
    THUMB_POOL.get_or_init(|| {
        let threads = std::thread::available_parallelism()
            .map(|n| (n.get() / 2).clamp(2, 6))
            .unwrap_or(4);
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .thread_name(|idx| format!("berry-thumb-{idx}"))
            .build()
            .expect("Failed to initialize thumbnail worker thread pool")
    })
}

/// Stats for the thumbnail cache on disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThumbnailCacheStats {
    pub total_bytes: u64,
    pub file_count: usize,
    pub cache_dir: String,
    pub budget_bytes: u64,
}

/// Thumbnail generation progress event payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThumbnailProgress {
    pub current: usize,
    pub total: usize,
    pub done: bool,
}

/// Outcome of one bounded thumbnail batch.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThumbnailBatchResult {
    pub generated: usize,
    pub canceled: usize,
}

/// Source identity and requested size tier for one visible thumbnail.
pub struct ThumbnailRequest<'a> {
    pub file_id: i64,
    pub file_path: &'a str,
    pub modified_at: i64,
    pub max_edge: u32,
}

/// Compute canonical destination path for a thumbnail.
pub fn get_thumbnail_path(
    cache_dir: &Path,
    file_id: i64,
    modified_at: i64,
    max_edge: u32,
) -> PathBuf {
    let thumb_dir = cache_dir.join("thumbnails");
    thumb_dir.join(format!("{}_{}_{}.webp", file_id, modified_at, max_edge))
}

/// Generate a downscaled thumbnail and save to `dst_path` as WebP.
pub fn generate_thumbnail(src_path: &Path, dst_path: &Path, max_edge: u32) -> Result<(), String> {
    if !src_path.exists() {
        return Err(format!(
            "Source image does not exist: {}",
            src_path.display()
        ));
    }

    // Ensure parent directory exists
    if let Some(parent) = dst_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create thumbnail dir: {e}"))?;
    }

    // Open and decode source image
    let reader = ImageReader::open(src_path)
        .map_err(|e| format!("Failed to open image {}: {e}", src_path.display()))?
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess format for {}: {e}", src_path.display()))?;

    let img = reader
        .decode()
        .map_err(|e| format!("Failed to decode image {}: {e}", src_path.display()))?;

    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return Err("Image dimensions are zero".to_string());
    }

    // Calculate downscaled dimensions (respecting aspect ratio)
    let (target_w, target_h) = if w >= h {
        if w > max_edge {
            let scale = max_edge as f32 / w as f32;
            (max_edge, ((h as f32 * scale).round() as u32).max(1))
        } else {
            (w, h)
        }
    } else {
        if h > max_edge {
            let scale = max_edge as f32 / h as f32;
            (((w as f32 * scale).round() as u32).max(1), max_edge)
        } else {
            (w, h)
        }
    };

    // Fast Lanczos3 downscaling
    let thumb = img.thumbnail(target_w, target_h);

    // Save as WebP so the manifest codec always matches the encoded file.
    thumb
        .save_with_format(dst_path, image::ImageFormat::WebP)
        .map_err(|e| format!("Failed to encode thumbnail {}: {e}", dst_path.display()))?;

    Ok(())
}

/// Ensure a thumbnail exists on disk for a given file. If not present, generate it.
pub fn ensure_thumbnail<C>(
    cache_dir: &Path,
    db_path: &Path,
    request: ThumbnailRequest<'_>,
    budget_bytes: u64,
    should_continue: C,
) -> Result<String, String>
where
    C: Fn() -> bool + Send + Sync,
{
    QUEUE_COUNTERS.queued.fetch_add(1, Ordering::Relaxed);
    if !should_continue() {
        QUEUE_COUNTERS.queued.fetch_sub(1, Ordering::Relaxed);
        QUEUE_COUNTERS.canceled.fetch_add(1, Ordering::Relaxed);
        return Err("thumbnail request canceled".to_string());
    }
    let dst = get_thumbnail_path(
        cache_dir,
        request.file_id,
        request.modified_at,
        request.max_edge,
    );
    if dst.exists() {
        QUEUE_COUNTERS.queued.fetch_sub(1, Ordering::Relaxed);
        QUEUE_COUNTERS.manifest_hits.fetch_add(1, Ordering::Relaxed);
        QUEUE_COUNTERS.completed.fetch_add(1, Ordering::Relaxed);
        if should_persist_access(&dst) {
            if let Err(error) = record_thumbnail(
                db_path,
                request.file_id,
                request.modified_at,
                request.max_edge,
                &dst,
            ) {
                forget_recent_access(&dst);
                return Err(error);
            }
        }
        return Ok(dst.to_string_lossy().to_string());
    }

    let db = Database::connect(db_path).map_err(|error| error.to_string())?;
    while let Some(entry) = db
        .find_sufficient_thumbnail_cache_entry(
            request.file_id,
            request.modified_at,
            request.max_edge,
        )
        .map_err(|error| error.to_string())?
    {
        let reusable_path = PathBuf::from(&entry.path);
        if reusable_path.is_file() {
            QUEUE_COUNTERS.queued.fetch_sub(1, Ordering::Relaxed);
            QUEUE_COUNTERS
                .reused_tier_hits
                .fetch_add(1, Ordering::Relaxed);
            QUEUE_COUNTERS.completed.fetch_add(1, Ordering::Relaxed);
            if should_persist_access(&reusable_path) {
                let accessed_at = current_timestamp();
                if let Err(error) = db.touch_thumbnail_cache_entry(
                    entry.file_id,
                    entry.modified_at,
                    entry.max_edge,
                    &entry.codec,
                    accessed_at,
                    accessed_at - ACCESS_PERSIST_INTERVAL_SECONDS,
                ) {
                    forget_recent_access(&reusable_path);
                    return Err(error.to_string());
                }
            }
            return Ok(entry.path);
        }
        db.delete_thumbnail_cache_entries(&[entry.path])
            .map_err(|error| error.to_string())?;
    }

    let src = Path::new(request.file_path);
    QUEUE_COUNTERS.queued.fetch_sub(1, Ordering::Relaxed);
    QUEUE_COUNTERS.running.fetch_add(1, Ordering::Relaxed);
    let decode_result = get_thumb_pool().install(|| {
        if !should_continue() {
            QUEUE_COUNTERS.running.fetch_sub(1, Ordering::Relaxed);
            QUEUE_COUNTERS.canceled.fetch_add(1, Ordering::Relaxed);
            return Err("thumbnail request canceled".to_string());
        }
        let res = generate_thumbnail(src, &dst, request.max_edge);
        QUEUE_COUNTERS.running.fetch_sub(1, Ordering::Relaxed);
        if res.is_ok() {
            QUEUE_COUNTERS.completed.fetch_add(1, Ordering::Relaxed);
        } else {
            QUEUE_COUNTERS.failed.fetch_add(1, Ordering::Relaxed);
        }
        res
    });
    decode_result?;
    record_thumbnail(
        db_path,
        request.file_id,
        request.modified_at,
        request.max_edge,
        &dst,
    )?;
    mark_recent_access(&dst);
    enforce_thumbnail_cache_budget(db_path, budget_bytes)?;

    Ok(dst.to_string_lossy().to_string())
}

/// Batch generate thumbnails in parallel using a bounded Rayon worker pool.
pub fn batch_generate_thumbnails<F, C>(
    cache_dir: &Path,
    db_path: &Path,
    items: Vec<(i64, String, i64)>, // (file_id, file_path, modified_at)
    max_edge: u32,
    budget_bytes: u64,
    progress_callback: Option<F>,
    should_continue: C,
) -> Result<ThumbnailBatchResult, String>
where
    F: Fn(usize, usize) + Send + Sync,
    C: Fn() -> bool + Send + Sync,
{
    let total = items.len();
    if total == 0 {
        return Ok(ThumbnailBatchResult {
            generated: 0,
            canceled: 0,
        });
    }

    QUEUE_COUNTERS
        .queued
        .fetch_add(total as u64, Ordering::Relaxed);
    let completed_counter = AtomicUsize::new(0);
    let canceled_counter = AtomicUsize::new(0);
    let pool = get_thumb_pool();

    let generated = pool.install(|| {
        items
            .into_par_iter()
            .filter_map(|(file_id, file_path, modified_at)| {
                QUEUE_COUNTERS.queued.fetch_sub(1, Ordering::Relaxed);
                if !should_continue() {
                    QUEUE_COUNTERS.canceled.fetch_add(1, Ordering::Relaxed);
                    canceled_counter.fetch_add(1, Ordering::Relaxed);
                    let current = completed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if let Some(ref cb) = progress_callback {
                        cb(current, total);
                    }
                    return None;
                }
                QUEUE_COUNTERS.running.fetch_add(1, Ordering::Relaxed);
                let dst = get_thumbnail_path(cache_dir, file_id, modified_at, max_edge);
                let generated = if dst.exists() {
                    QUEUE_COUNTERS.manifest_hits.fetch_add(1, Ordering::Relaxed);
                    QUEUE_COUNTERS.running.fetch_sub(1, Ordering::Relaxed);
                    QUEUE_COUNTERS.completed.fetch_add(1, Ordering::Relaxed);
                    false
                } else {
                    let src = Path::new(&file_path);
                    let res = generate_thumbnail(src, &dst, max_edge);
                    QUEUE_COUNTERS.running.fetch_sub(1, Ordering::Relaxed);
                    if res.is_ok() {
                        QUEUE_COUNTERS.completed.fetch_add(1, Ordering::Relaxed);
                        true
                    } else {
                        QUEUE_COUNTERS.failed.fetch_add(1, Ordering::Relaxed);
                        false
                    }
                };

                let current = completed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                if let Some(ref cb) = progress_callback {
                    cb(current, total);
                }

                if dst.is_file() {
                    Some((file_id, modified_at, dst, generated))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    });

    let entries = generated
        .iter()
        .filter_map(|(file_id, modified_at, path, _)| {
            thumbnail_entry(*file_id, *modified_at, max_edge, path).ok()
        })
        .collect::<Vec<_>>();
    let db = Database::connect(db_path).map_err(|error| error.to_string())?;
    db.upsert_thumbnail_cache_entries(&entries)
        .map_err(|error| error.to_string())?;
    for (_, _, path, _) in &generated {
        mark_recent_access(path);
    }
    enforce_thumbnail_cache_budget_with_db(&db, budget_bytes)?;
    Ok(ThumbnailBatchResult {
        generated: generated
            .iter()
            .filter(|(_, _, _, was_generated)| *was_generated)
            .count(),
        canceled: canceled_counter.load(Ordering::Relaxed),
    })
}

/// Get disk statistics for the thumbnail cache.
pub fn get_thumbnail_cache_stats(
    cache_dir: &Path,
    db_path: &Path,
    budget_bytes: u64,
) -> Result<ThumbnailCacheStats, String> {
    synchronize_thumbnail_manifest(cache_dir, db_path, budget_bytes)?;
    let thumb_dir = cache_dir.join("thumbnails");
    if !thumb_dir.exists() {
        return Ok(ThumbnailCacheStats {
            total_bytes: 0,
            file_count: 0,
            cache_dir: thumb_dir.to_string_lossy().to_string(),
            budget_bytes,
        });
    }

    let mut total_bytes = 0u64;
    let mut file_count = 0usize;

    if let Ok(entries) = fs::read_dir(&thumb_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_bytes += meta.len();
                    file_count += 1;
                }
            }
        }
    }

    Ok(ThumbnailCacheStats {
        total_bytes,
        file_count,
        cache_dir: thumb_dir.to_string_lossy().to_string(),
        budget_bytes,
    })
}

/// Clear all cached thumbnail files from disk.
pub fn clear_thumbnail_cache(cache_dir: &Path, db_path: &Path) -> Result<usize, String> {
    if let Some(recent_touches) = RECENT_TOUCHES.get() {
        if let Ok(mut recent_touches) = recent_touches.lock() {
            recent_touches.clear();
        }
    }
    let thumb_dir = cache_dir.join("thumbnails");
    if !thumb_dir.exists() {
        Database::connect(db_path)
            .map_err(|error| error.to_string())?
            .clear_thumbnail_cache_entries()
            .map_err(|error| error.to_string())?;
        return Ok(0);
    }

    let mut removed = 0;
    let mut removed_paths = Vec::new();
    if let Ok(entries) = fs::read_dir(&thumb_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() && fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                    removed_paths.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }

    let db = Database::connect(db_path).map_err(|error| error.to_string())?;
    let has_remaining_files = fs::read_dir(&thumb_dir)
        .map(|mut entries| entries.any(|entry| entry.is_ok_and(|entry| entry.path().is_file())))
        .unwrap_or(false);
    if has_remaining_files {
        db.delete_thumbnail_cache_entries(&removed_paths)
            .map_err(|error| error.to_string())?;
    } else {
        db.clear_thumbnail_cache_entries()
            .map_err(|error| error.to_string())?;
    }

    Ok(removed)
}

/// Import existing cache files into the manifest and enforce the configured budget.
pub fn synchronize_thumbnail_manifest(
    cache_dir: &Path,
    db_path: &Path,
    budget_bytes: u64,
) -> Result<(), String> {
    let thumb_dir = cache_dir.join("thumbnails");
    if !thumb_dir.exists() {
        return Ok(());
    }

    let db = Database::connect(db_path).map_err(|error| error.to_string())?;
    let mut pending = Vec::with_capacity(256);
    for entry in fs::read_dir(&thumb_dir).map_err(|error| error.to_string())? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let Some((file_id, modified_at, max_edge)) = parse_thumbnail_filename(&entry.path()) else {
            continue;
        };
        let accessed_at = entry
            .metadata()
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_else(current_timestamp);
        let Ok(manifest_entry) =
            thumbnail_entry_at(file_id, modified_at, max_edge, &entry.path(), accessed_at)
        else {
            continue;
        };
        pending.push(manifest_entry);
        if pending.len() == 256 {
            db.upsert_thumbnail_cache_entries(&pending)
                .map_err(|error| error.to_string())?;
            pending.clear();
        }
    }
    db.upsert_thumbnail_cache_entries(&pending)
        .map_err(|error| error.to_string())?;
    enforce_thumbnail_cache_budget_with_db(&db, budget_bytes)
}

fn record_thumbnail(
    db_path: &Path,
    file_id: i64,
    modified_at: i64,
    max_edge: u32,
    path: &Path,
) -> Result<(), String> {
    let entry = thumbnail_entry(file_id, modified_at, max_edge, path)?;
    Database::connect(db_path)
        .map_err(|error| error.to_string())?
        .upsert_thumbnail_cache_entries(&[entry])
        .map_err(|error| error.to_string())
}

fn should_persist_access(path: &Path) -> bool {
    let now = current_timestamp();
    let key = path.to_string_lossy().to_string();
    let recent_touches = RECENT_TOUCHES.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut recent_touches) = recent_touches.lock() else {
        return true;
    };
    if recent_touches
        .get(&key)
        .is_some_and(|last_access| now - last_access < ACCESS_PERSIST_INTERVAL_SECONDS)
    {
        return false;
    }
    if recent_touches.len() >= MAX_RECENT_TOUCHES {
        recent_touches
            .retain(|_, last_access| now - *last_access < ACCESS_PERSIST_INTERVAL_SECONDS);
    }
    recent_touches.insert(key, now);
    true
}

fn mark_recent_access(path: &Path) {
    let recent_touches = RECENT_TOUCHES.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut recent_touches) = recent_touches.lock() {
        recent_touches.insert(path.to_string_lossy().to_string(), current_timestamp());
    }
}

fn forget_recent_access(path: &Path) {
    let Some(recent_touches) = RECENT_TOUCHES.get() else {
        return;
    };
    if let Ok(mut recent_touches) = recent_touches.lock() {
        recent_touches.remove(path.to_string_lossy().as_ref());
    }
}

fn thumbnail_entry(
    file_id: i64,
    modified_at: i64,
    max_edge: u32,
    path: &Path,
) -> Result<ThumbnailCacheEntry, String> {
    thumbnail_entry_at(file_id, modified_at, max_edge, path, current_timestamp())
}

fn thumbnail_entry_at(
    file_id: i64,
    modified_at: i64,
    max_edge: u32,
    path: &Path,
    last_accessed_at: i64,
) -> Result<ThumbnailCacheEntry, String> {
    let metadata = path.metadata().map_err(|error| error.to_string())?;
    Ok(ThumbnailCacheEntry {
        file_id,
        modified_at,
        max_edge,
        codec: "webp".to_string(),
        path: path.to_string_lossy().to_string(),
        size_bytes: metadata.len(),
        last_accessed_at,
    })
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn parse_thumbnail_filename(path: &Path) -> Option<(i64, i64, u32)> {
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("webp"))
    {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    let mut parts = stem.rsplitn(3, '_');
    let max_edge = parts.next()?.parse().ok()?;
    let modified_at = parts.next()?.parse().ok()?;
    let file_id = parts.next()?.parse().ok()?;
    Some((file_id, modified_at, max_edge))
}

fn enforce_thumbnail_cache_budget(db_path: &Path, budget_bytes: u64) -> Result<(), String> {
    let db = Database::connect(db_path).map_err(|error| error.to_string())?;
    enforce_thumbnail_cache_budget_with_db(&db, budget_bytes)
}

fn enforce_thumbnail_cache_budget_with_db(db: &Database, budget_bytes: u64) -> Result<(), String> {
    let (mut total_bytes, _) = db
        .thumbnail_cache_usage()
        .map_err(|error| error.to_string())?;
    while total_bytes > budget_bytes {
        let candidates = db
            .list_thumbnail_cache_entries_lru(256)
            .map_err(|error| error.to_string())?;
        if candidates.is_empty() {
            break;
        }

        let mut deleted_paths = Vec::new();
        for candidate in candidates {
            if total_bytes <= budget_bytes {
                break;
            }
            let path = Path::new(&candidate.path);
            if !path.exists() || fs::remove_file(path).is_ok() {
                total_bytes = total_bytes.saturating_sub(candidate.size_bytes);
                deleted_paths.push(candidate.path);
            }
        }
        if deleted_paths.is_empty() {
            break;
        }
        db.delete_thumbnail_cache_entries(&deleted_paths)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("berry-thumbnail-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(path.join("thumbnails")).unwrap();
        path
    }

    #[test]
    fn parses_canonical_thumbnail_filenames() {
        assert_eq!(
            parse_thumbnail_filename(Path::new("12_1700000000_384.webp")),
            Some((12, 1_700_000_000, 384))
        );
        assert_eq!(parse_thumbnail_filename(Path::new("invalid.webp")), None);
        assert_eq!(parse_thumbnail_filename(Path::new("12_10_384.jpg")), None);
    }

    #[test]
    fn synchronization_imports_entries_and_evicts_oldest_files() {
        let dir = test_dir("manifest");
        let db_path = dir.join("berry.db");
        Database::connect(&db_path).unwrap();
        let first = get_thumbnail_path(&dir, 1, 10, 256);
        let second = get_thumbnail_path(&dir, 2, 20, 256);
        fs::write(&first, vec![1; 80]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&second, vec![2; 80]).unwrap();

        synchronize_thumbnail_manifest(&dir, &db_path, 100).unwrap();

        assert!(!first.exists());
        assert!(second.exists());
        assert_eq!(
            Database::connect(&db_path)
                .unwrap()
                .thumbnail_cache_usage()
                .unwrap(),
            (80, 1)
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn canceled_batch_skips_files_before_decode() {
        let dir = test_dir("canceled-batch");
        let db_path = dir.join("berry.db");
        Database::connect(&db_path).unwrap();
        let result = batch_generate_thumbnails(
            &dir,
            &db_path,
            vec![(1, dir.join("missing.png").to_string_lossy().to_string(), 10)],
            256,
            1024,
            None::<fn(usize, usize)>,
            || false,
        )
        .unwrap();

        assert_eq!(result.generated, 0);
        assert_eq!(result.canceled, 1);
        assert!(!get_thumbnail_path(&dir, 1, 10, 256).exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn visible_request_reuses_a_sufficient_cached_tier() {
        let dir = test_dir("tier-reuse");
        let db_path = dir.join("berry.db");
        let db = Database::connect(&db_path).unwrap();
        let cached_path = get_thumbnail_path(&dir, 1, 10, 384);
        fs::write(&cached_path, vec![1; 80]).unwrap();
        let missing_path = get_thumbnail_path(&dir, 1, 10, 320);
        db.upsert_thumbnail_cache_entries(&[
            ThumbnailCacheEntry {
                file_id: 1,
                modified_at: 10,
                max_edge: 320,
                codec: "webp".to_string(),
                path: missing_path.to_string_lossy().to_string(),
                size_bytes: 80,
                last_accessed_at: 1,
            },
            thumbnail_entry(1, 10, 384, &cached_path).unwrap(),
        ])
        .unwrap();

        let resolved = ensure_thumbnail(
            &dir,
            &db_path,
            ThumbnailRequest {
                file_id: 1,
                file_path: dir.join("missing.png").to_string_lossy().as_ref(),
                modified_at: 10,
                max_edge: 256,
            },
            1024,
            || true,
        )
        .unwrap();

        assert_eq!(Path::new(&resolved), cached_path);
        assert!(!get_thumbnail_path(&dir, 1, 10, 256).exists());
        assert_eq!(db.thumbnail_cache_usage().unwrap(), (80, 1));
        drop(db);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn tracks_thumbnail_queue_diagnostics() {
        let dir = test_dir("diagnostics");
        let db_path = dir.join("berry.db");
        let db = Database::connect(&db_path).unwrap();

        // 1. Canceled request increments canceled counter
        let before_canceled = get_thumbnail_queue_diagnostics().canceled;
        let err = ensure_thumbnail(
            &dir,
            &db_path,
            ThumbnailRequest {
                file_id: 1,
                file_path: dir.join("img.png").to_string_lossy().as_ref(),
                modified_at: 10,
                max_edge: 256,
            },
            1024,
            || false,
        );
        assert!(err.is_err());
        let after_canceled = get_thumbnail_queue_diagnostics().canceled;
        assert!(after_canceled > before_canceled);

        // 2. Manifest hit increments manifest_hits and completed counters
        let cached_path = get_thumbnail_path(&dir, 2, 20, 256);
        fs::write(&cached_path, vec![1; 50]).unwrap();
        let before_hits = get_thumbnail_queue_diagnostics().manifest_hits;
        let before_completed = get_thumbnail_queue_diagnostics().completed;
        let ok = ensure_thumbnail(
            &dir,
            &db_path,
            ThumbnailRequest {
                file_id: 2,
                file_path: dir.join("img2.png").to_string_lossy().as_ref(),
                modified_at: 20,
                max_edge: 256,
            },
            1024,
            || true,
        );
        assert!(ok.is_ok());
        let after_hits = get_thumbnail_queue_diagnostics().manifest_hits;
        let after_completed = get_thumbnail_queue_diagnostics().completed;
        assert!(after_hits > before_hits);
        assert!(after_completed > before_completed);

        drop(db);
        fs::remove_dir_all(dir).unwrap();
    }
}
