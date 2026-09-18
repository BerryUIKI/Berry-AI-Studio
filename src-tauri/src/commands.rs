//! Tauri IPC commands exposed to the frontend.
//!
//! Each command is a thin wrapper: parse the request, call into the core
//! crates, and serialize the result. Business logic lives in the `berry-*`
//! crates, not here.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::MutexGuard;

use berry_clip::{ClipEngine, ClipModelInfo};
use berry_domain::{
    plan_prompt_stacks, Album, CheckpointModelStat, CleanupQueueItem, DatabaseStats, DetectedLora,
    FilePage, FileSortField, Folder, ImageFile, LoraModel, ModelCacheEntry, PipelineDetectedPath,
    PromptStackCandidate, PromptStat, SearchCriteria, SimilarityMatch, SortDirection, StackSummary,
    Tag,
};
use berry_scan::{ScanStats, Scanner};
use berry_storage::Database;
use berry_tagger::{ModelInfo, TagPrediction, TaggerConfig, Wd14Tagger};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::AppState;

/// Lock and return the shared database connection.
fn db<'a>(state: &'a State<'a, AppState>) -> Result<MutexGuard<'a, Database>, String> {
    state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())
}

/// Lock and return the optional tagger instance.
fn tagger_guard<'a>(
    state: &'a State<'a, AppState>,
) -> Result<MutexGuard<'a, Option<Wd14Tagger>>, String> {
    state
        .tagger
        .lock()
        .map_err(|_| "tagger lock poisoned".to_string())
}

/// Lock and return the optional clip engine instance.
fn clip_guard<'a>(
    state: &'a State<'a, AppState>,
) -> Result<MutexGuard<'a, Option<ClipEngine>>, String> {
    state
        .clip
        .lock()
        .map_err(|_| "clip lock poisoned".to_string())
}

/// Resolve a user-supplied folder path to a canonical absolute path.
///
/// The Windows `\\?\` verbatim prefix produced by `canonicalize` is stripped so
/// stored paths (and the walk keys derived from them) stay readable.
fn canonicalize_folder(path: &str) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("cannot access path: {e}"))?;
    if !meta.is_dir() {
        return Err("path is not a directory".to_string());
    }
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
    let text = canonical.to_string_lossy();
    let stripped = text.strip_prefix(r"\\?\").unwrap_or(&text);
    Ok(stripped.to_string())
}

/// Diagnostics shown on the M1 shell to prove the full chain works
/// (frontend → IPC → Rust → SQLite).
#[derive(Serialize)]
pub struct AppInfo {
    /// Crate version from `Cargo.toml`.
    pub app_version: String,
    /// The SQLite `PRAGMA user_version` after migration.
    pub schema_version: i64,
    /// Absolute path of the opened database (empty for in-memory).
    pub database_path: String,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfo, String> {
    let db = db(&state)?;
    let schema_version = db.user_version().map_err(|e| e.to_string())?;
    let database_path = db
        .path()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    Ok(AppInfo {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        database_path,
    })
}

/// Register a folder for scanning. Rejects paths that do not exist, are not
/// directories, or are already registered.
#[tauri::command]
pub fn add_folder(path: String, state: State<'_, AppState>) -> Result<Folder, String> {
    add_folder_with_options(path, None, None, None, None, None, state)
}

/// Register a folder with explicit mode ('link', 'managed', 'pipeline') and pipeline options.
#[tauri::command]
pub fn add_folder_with_options(
    path: String,
    folder_type: Option<String>,
    source_path: Option<String>,
    ingest_action: Option<String>,
    grace_period_hours: Option<i32>,
    auto_harvest: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Folder, String> {
    let ftype = folder_type.unwrap_or_else(|| "link".to_string());
    if ftype == "managed" {
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create managed folder directory: {e}"))?;
    }

    let canonical = canonicalize_folder(&path)?;
    let db = db(&state)?;
    if db
        .find_folder_by_path(&canonical)
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Err("folder is already added".to_string());
    }

    let src = match source_path {
        Some(s) if !s.trim().is_empty() => Some(canonicalize_folder(&s)?),
        _ => None,
    };

    let folder = db
        .add_folder_with_mode(
            &canonical,
            &ftype,
            src.as_deref(),
            ingest_action.as_deref(),
            grace_period_hours,
            auto_harvest.unwrap_or(true),
        )
        .map_err(|e| e.to_string())?;
    drop(db);
    if let Ok(mut watcher) = state.watcher.lock() {
        if let Some(watcher) = watcher.as_mut() {
            if let Err(error) = watcher.watch_folder(&folder) {
                eprintln!(
                    "could not watch newly added folder {}: {error}",
                    folder.path
                );
            }
        }
    }
    Ok(folder)
}

/// All registered folders, ordered by id.
#[tauri::command]
pub fn list_folders(state: State<'_, AppState>) -> Result<Vec<Folder>, String> {
    db(&state)?.list_folders().map_err(|e| e.to_string())
}

/// Remove a folder and its indexed files.
#[tauri::command]
pub fn remove_folder(folder_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let database = db(&state)?;
    let folder = database
        .list_folders()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|folder| folder.id == folder_id)
        .ok_or_else(|| format!("no folder with id {folder_id}"))?;
    database
        .remove_folder(folder_id)
        .map_err(|e| e.to_string())?;
    drop(database);
    if let Ok(mut watcher) = state.watcher.lock() {
        if let Some(watcher) = watcher.as_mut() {
            if let Err(error) = watcher.unwatch_folder(&folder) {
                eprintln!(
                    "could not stop watching removed folder {}: {error}",
                    folder.path
                );
            }
        }
    }
    Ok(())
}

/// Files of a folder, ordered by path.
#[tauri::command]
pub async fn list_files(
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?.list_files(folder_id).map_err(|e| e.to_string())
}

/// Query indexed files with optional folder filtering and multi-criteria sorting.
#[tauri::command]
pub async fn query_files(
    folder_id: Option<i64>,
    sort: Option<FileSortField>,
    direction: Option<SortDirection>,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    let sort = sort.unwrap_or(FileSortField::ModifiedAt);
    let direction = direction.unwrap_or(SortDirection::Desc);
    db(&state)?
        .query_files(folder_id, sort, direction)
        .map_err(|e| e.to_string())
}

/// Search indexed files using structured criteria.
#[tauri::command]
pub async fn search_files(
    criteria: SearchCriteria,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?
        .search_files(&criteria)
        .map_err(|e| e.to_string())
}

/// Search a bounded result page and return its exact filtered total.
#[tauri::command]
pub async fn search_files_page(
    criteria: SearchCriteria,
    state: State<'_, AppState>,
) -> Result<FilePage, String> {
    db(&state)?
        .search_gallery_files_page(&criteria)
        .map_err(|e| e.to_string())
}

/// Fetch one complete file record after a gallery summary is selected.
#[tauri::command]
pub fn get_file_details(file_id: i64, state: State<'_, AppState>) -> Result<ImageFile, String> {
    db(&state)?
        .get_file_by_id(file_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("no file with id {file_id}"))
}

/// Search indexed files using a parsed query string.
#[tauri::command]
pub async fn search_files_by_query(
    query: String,
    folder_id: Option<i64>,
    sort: Option<FileSortField>,
    direction: Option<SortDirection>,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    let mut criteria = SearchCriteria::from_query(&query);
    if folder_id.is_some() {
        criteria.folder_id = folder_id;
    }
    if sort.is_some() {
        criteria.sort = sort;
    }
    if direction.is_some() {
        criteria.direction = direction;
    }
    db(&state)?
        .search_files(&criteria)
        .map_err(|e| e.to_string())
}

/// Parse a free-form query and return one bounded page of matching files.
#[tauri::command]
pub async fn search_files_by_query_page(
    query: String,
    folder_id: Option<i64>,
    sort: Option<FileSortField>,
    direction: Option<SortDirection>,
    limit: usize,
    offset: usize,
    state: State<'_, AppState>,
) -> Result<FilePage, String> {
    let mut criteria = SearchCriteria::from_query(&query);
    criteria.folder_id = folder_id;
    criteria.sort = sort;
    criteria.direction = direction;
    criteria.limit = Some(limit);
    criteria.offset = Some(offset);
    db(&state)?
        .search_gallery_files_page(&criteria)
        .map_err(|e| e.to_string())
}

/// List distinct model names present in indexed metadata.
#[tauri::command]
pub fn list_distinct_models(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    db(&state)?
        .list_distinct_models()
        .map_err(|e| e.to_string())
}

/// List distinct sampler names present in indexed metadata.
#[tauri::command]
pub fn list_distinct_samplers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    db(&state)?
        .list_distinct_samplers()
        .map_err(|e| e.to_string())
}

/// Update user rating (1–10, or null to clear) for an image file.
#[tauri::command]
pub fn set_file_rating(
    file_id: i64,
    rating: Option<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .set_file_rating(file_id, rating)
        .map_err(|e| e.to_string())
}

/// Update user rating (1–10, or null to clear) for multiple image files.
#[tauri::command]
pub fn set_files_rating(
    file_ids: Vec<i64>,
    rating: Option<u8>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .set_files_rating(&file_ids, rating)
        .map_err(|e| e.to_string())
}

// --- Albums ---

/// Create a new album.
#[tauri::command]
pub fn create_album(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Album, String> {
    db(&state)?
        .create_album(&name, description.as_deref())
        .map_err(|e| e.to_string())
}

/// Retrieve an album by ID.
#[tauri::command]
pub fn get_album(id: i64, state: State<'_, AppState>) -> Result<Option<Album>, String> {
    db(&state)?.get_album(id).map_err(|e| e.to_string())
}

/// List all albums.
#[tauri::command]
pub fn list_albums(state: State<'_, AppState>) -> Result<Vec<Album>, String> {
    db(&state)?.list_albums().map_err(|e| e.to_string())
}

/// Rename an album.
#[tauri::command]
pub fn rename_album(id: i64, new_name: String, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .rename_album(id, &new_name)
        .map_err(|e| e.to_string())
}

/// Delete an album.
#[tauri::command]
pub fn delete_album(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?.delete_album(id).map_err(|e| e.to_string())
}

/// Add a file to an album.
#[tauri::command]
pub fn add_file_to_album(
    album_id: i64,
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .add_file_to_album(album_id, file_id)
        .map_err(|e| e.to_string())
}

/// Add multiple files to an album.
#[tauri::command]
pub fn add_files_to_album(
    album_id: i64,
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .add_files_to_album(album_id, &file_ids)
        .map_err(|e| e.to_string())
}

/// Remove a file from an album.
#[tauri::command]
pub fn remove_file_from_album(
    album_id: i64,
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .remove_file_from_album(album_id, file_id)
        .map_err(|e| e.to_string())
}

/// Remove multiple files from an album.
#[tauri::command]
pub fn remove_files_from_album(
    album_id: i64,
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .remove_files_from_album(album_id, &file_ids)
        .map_err(|e| e.to_string())
}

/// Count files in an album.
#[tauri::command]
pub fn count_album_files(album_id: i64, state: State<'_, AppState>) -> Result<i64, String> {
    db(&state)?
        .count_album_files(album_id)
        .map_err(|e| e.to_string())
}

/// List files in an album.
#[tauri::command]
pub fn list_album_files(
    album_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?
        .list_album_files(album_id)
        .map_err(|e| e.to_string())
}

// --- Tags ---

/// Create a tag.
#[tauri::command]
pub fn create_tag(
    name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    db(&state)?
        .create_tag(&name, color.as_deref())
        .map_err(|e| e.to_string())
}

/// List all tags.
#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    db(&state)?.list_tags().map_err(|e| e.to_string())
}

/// Delete a tag.
#[tauri::command]
pub fn delete_tag(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?.delete_tag(id).map_err(|e| e.to_string())
}

/// Tag a single file.
#[tauri::command]
pub fn tag_file(file_id: i64, tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .tag_file(file_id, tag_id)
        .map_err(|e| e.to_string())
}

/// Tag multiple files.
#[tauri::command]
pub fn tag_files(
    file_ids: Vec<i64>,
    tag_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .tag_files(&file_ids, tag_id)
        .map_err(|e| e.to_string())
}

/// Untag a file.
#[tauri::command]
pub fn untag_file(file_id: i64, tag_id: i64, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .untag_file(file_id, tag_id)
        .map_err(|e| e.to_string())
}

/// Untag multiple files.
#[tauri::command]
pub fn untag_files(
    file_ids: Vec<i64>,
    tag_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .untag_files(&file_ids, tag_id)
        .map_err(|e| e.to_string())
}

/// Get tags attached to a file.
#[tauri::command]
pub fn get_file_tags(file_id: i64, state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    db(&state)?
        .get_file_tags(file_id)
        .map_err(|e| e.to_string())
}

/// List files with a specific tag.
#[tauri::command]
pub fn list_files_by_tag(
    tag_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    db(&state)?
        .list_files_by_tag(tag_id)
        .map_err(|e| e.to_string())
}

// --- Favorites & NSFW ---

/// Set favorite status for a single file.
#[tauri::command]
pub fn set_file_favorite(
    file_id: i64,
    is_favorite: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .set_file_favorite(file_id, is_favorite)
        .map_err(|e| e.to_string())
}

/// Set favorite status for multiple files.
#[tauri::command]
pub fn set_files_favorite(
    file_ids: Vec<i64>,
    is_favorite: bool,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .set_files_favorite(&file_ids, is_favorite)
        .map_err(|e| e.to_string())
}

/// Set NSFW status for a single file.
#[tauri::command]
pub fn set_file_nsfw(
    file_id: i64,
    is_nsfw: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .set_file_nsfw(file_id, is_nsfw)
        .map_err(|e| e.to_string())
}

/// Set NSFW status for multiple files.
#[tauri::command]
pub fn set_files_nsfw(
    file_ids: Vec<i64>,
    is_nsfw: bool,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    db(&state)?
        .set_files_nsfw(&file_ids, is_nsfw)
        .map_err(|e| e.to_string())
}

// --- Prompt Stats ---

/// Get frequency statistics for prompt tags.
#[tauri::command]
pub fn get_prompt_stats(
    is_negative: bool,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<PromptStat>, String> {
    db(&state)?
        .get_prompt_stats(is_negative, limit)
        .map_err(|e| e.to_string())
}

/// Aggregated file counts across the library and per folder.
#[derive(Serialize)]
pub struct LibraryCounts {
    pub total: i64,
    pub folders: HashMap<i64, i64>,
}

/// Get file counts per folder plus total indexed files across all folders.
#[tauri::command]
pub fn get_library_counts(state: State<'_, AppState>) -> Result<LibraryCounts, String> {
    let db = db(&state)?;
    let total = db.count_all_files().map_err(|e| e.to_string())?;
    let folders = db.get_folder_file_counts().map_err(|e| e.to_string())?;
    Ok(LibraryCounts { total, folders })
}

/// Scan a folder on a blocking thread, emitting `scan-progress` events as it
/// goes. Returns the final [`ScanStats`].
///
/// The scanner opens its own database connection, so the shell's connection
/// (used by read commands) is never blocked by a long scan.
#[tauri::command]
pub async fn scan_folder(
    app: AppHandle,
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<ScanStats, String> {
    run_scan(app, folder_id, state, false).await
}

/// Re-extract metadata from every file in a folder, ignoring the incremental
/// cache. Use after a metadata-parser update so already-indexed files get the
/// current extractor's output.
#[tauri::command]
pub async fn rebuild_metadata(
    app: AppHandle,
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<ScanStats, String> {
    run_scan(app, folder_id, state, true).await
}

/// Shared scan runner: snapshots the folder path and database path while the
/// lock is held, then spawns a [`Scanner`] on a blocking thread. `forced` makes
/// the scanner bypass the incremental cache (used by `rebuild_metadata`).
async fn run_scan(
    app: AppHandle,
    folder_id: i64,
    state: State<'_, AppState>,
    forced: bool,
) -> Result<ScanStats, String> {
    // Snapshot the folder path and database path while the lock is held, then
    // release it before spawning the scan thread.
    let (root, db_path) = {
        let db = db(&state)?;
        let folder = db
            .list_folders()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.id == folder_id)
            .ok_or_else(|| format!("no folder with id {folder_id}"))?;
        let db_path = db
            .path()
            .ok_or_else(|| "in-memory database cannot be scanned".to_string())?
            .to_path_buf();
        (folder.path, db_path)
    };

    let scanner = if forced {
        Scanner::with_forced_extractor(db_path)
    } else {
        Scanner::with_default_extractor(db_path)
    };
    tauri::async_runtime::spawn_blocking(move || {
        scanner
            .scan_folder(folder_id, Path::new(&root), |progress| {
                let _ = app.emit("scan-progress", progress);
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- Checkpoints and Model Cache ---

/// Get list of indexed checkpoint models and their occurrence counts.
#[tauri::command]
pub fn get_checkpoint_models(
    state: State<'_, AppState>,
) -> Result<Vec<CheckpointModelStat>, String> {
    db(&state)?
        .get_checkpoint_models()
        .map_err(|e| e.to_string())
}

/// Import A1111 cache.json or custom model hash mapping JSON file.
#[tauri::command]
pub fn import_model_cache_file(path: String, state: State<'_, AppState>) -> Result<usize, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read cache file at {path}: {e}"))?;

    let root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("failed to parse JSON in cache file: {e}"))?;

    let mut entries = Vec::new();

    if let Some(obj) = root.as_object() {
        for (key, val) in obj {
            if let Some(item_obj) = val.as_object() {
                // A1111 format: key = "checkpoint/name [hash]", val = { "model_name": ..., "hash": ..., "sha256": ... }
                let hash = item_obj
                    .get("hash")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        item_obj
                            .get("hashes")
                            .and_then(|h| h.get("SHA256"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or(key.as_str());

                let name = item_obj
                    .get("model_name")
                    .or_else(|| item_obj.get("filename"))
                    .or_else(|| item_obj.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(key.as_str());

                let title = item_obj
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let sha256 = item_obj
                    .get("sha256")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        item_obj
                            .get("hashes")
                            .and_then(|h| h.get("SHA256"))
                            .and_then(|v| v.as_str())
                    })
                    .map(|s| s.to_string());

                entries.push(ModelCacheEntry {
                    hash: hash.to_string(),
                    name: name.to_string(),
                    title,
                    sha256,
                });
            } else if let Some(str_val) = val.as_str() {
                // Simple { "hash": "model_name" } map
                entries.push(ModelCacheEntry {
                    hash: key.clone(),
                    name: str_val.to_string(),
                    title: None,
                    sha256: if key.len() == 64 {
                        Some(key.clone())
                    } else {
                        None
                    },
                });
            }
        }
    } else if let Some(arr) = root.as_array() {
        for item in arr {
            if let Ok(entry) = serde_json::from_value::<ModelCacheEntry>(item.clone()) {
                entries.push(entry);
            }
        }
    }

    db(&state)?
        .import_model_cache(&entries)
        .map_err(|e| e.to_string())
}

/// Resolve a model name from its short hash or SHA256.
#[tauri::command]
pub fn resolve_model_hash(
    hash: String,
    state: State<'_, AppState>,
) -> Result<Option<ModelCacheEntry>, String> {
    db(&state)?
        .resolve_model_hash(&hash)
        .map_err(|e| e.to_string())
}

/// List all entries in model cache.
#[tauri::command]
pub fn list_model_cache(state: State<'_, AppState>) -> Result<Vec<ModelCacheEntry>, String> {
    db(&state)?.list_model_cache().map_err(|e| e.to_string())
}

// --- File Operations & Drag-and-Drop ---

/// Move files and their sidecars to a target indexed folder, updating database paths.
#[tauri::command]
pub fn move_files(
    file_paths: Vec<String>,
    target_folder_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let target_folder = {
        let database = db(&state)?;
        database
            .list_folders()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.id == target_folder_id)
            .ok_or_else(|| format!("Target folder {target_folder_id} not found"))?
    };

    let target_dir = Path::new(&target_folder.path);
    let mut moved_count = 0;

    let database = db(&state)?;
    for src_str in file_paths {
        let src_path = Path::new(&src_str);
        if !src_path.exists() {
            continue;
        }
        let file_name = match src_path.file_name() {
            Some(name) => name,
            None => continue,
        };
        let dest_path = target_dir.join(file_name);
        if dest_path == src_path {
            continue;
        }

        // Rename main file
        if std::fs::rename(src_path, &dest_path).is_err() {
            // Cross-device fallback: copy then remove
            std::fs::copy(src_path, &dest_path)
                .map_err(|e| format!("Failed to move file to {}: {e}", dest_path.display()))?;
            std::fs::remove_file(src_path).map_err(|e| {
                format!(
                    "Copied to {} but failed to remove source file {}: {e}",
                    dest_path.display(),
                    src_path.display()
                )
            })?;
        }

        // Check and move sibling sidecars (.txt, .json)
        let sidecar_txt = src_path.with_extension("txt");
        if sidecar_txt.exists() {
            let dest_txt = dest_path.with_extension("txt");
            let _ = std::fs::rename(&sidecar_txt, &dest_txt).or_else(|_| {
                std::fs::copy(&sidecar_txt, &dest_txt)
                    .and_then(|_| std::fs::remove_file(&sidecar_txt))
            });
        }

        // Update database record
        let new_path_str = dest_path.to_string_lossy().to_string();
        let _ = database.move_file_record(&src_str, &new_path_str, target_folder_id);
        moved_count += 1;
    }

    Ok(moved_count)
}

/// Copy files and their sidecars to a target indexed folder, inserting new database rows.
#[tauri::command]
pub fn copy_files(
    file_paths: Vec<String>,
    target_folder_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let target_folder = {
        let database = db(&state)?;
        database
            .list_folders()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|f| f.id == target_folder_id)
            .ok_or_else(|| format!("Target folder {target_folder_id} not found"))?
    };

    let target_dir = Path::new(&target_folder.path);
    let mut copied_count = 0;

    let database = db(&state)?;
    for src_str in file_paths {
        let src_path = Path::new(&src_str);
        if !src_path.exists() {
            continue;
        }
        let file_name = match src_path.file_name() {
            Some(name) => name,
            None => continue,
        };
        let dest_path = target_dir.join(file_name);
        if dest_path == src_path {
            continue;
        }

        // Copy main file
        std::fs::copy(src_path, &dest_path)
            .map_err(|e| format!("Failed to copy file to {}: {e}", dest_path.display()))?;

        // Copy sidecars if present
        let sidecar_txt = src_path.with_extension("txt");
        if sidecar_txt.exists() {
            let dest_txt = dest_path.with_extension("txt");
            let _ = std::fs::copy(&sidecar_txt, &dest_txt);
        }

        // Copy database record with new path
        if let Ok(Some(mut orig)) = database.get_file_by_path(&src_str) {
            orig.id = None;
            orig.folder_id = target_folder_id;
            orig.path = dest_path.to_string_lossy().to_string();
            let _ = database.upsert_file(&orig);
        }
        copied_count += 1;
    }

    Ok(copied_count)
}

/// Safely move files and sidecars to the system Trash / Recycle Bin and remove from DB.
#[tauri::command]
pub fn trash_files(file_paths: Vec<String>, state: State<'_, AppState>) -> Result<usize, String> {
    let mut trashed_count = 0;
    let database = db(&state)?;

    for path_str in file_paths {
        let path = Path::new(&path_str);
        if path.exists() {
            if let Err(e) = trash::delete(path) {
                // If trash fails (e.g. headless/external), fallback to permanent remove
                std::fs::remove_file(path).map_err(|rem_e| {
                    format!(
                        "Failed to trash or remove {}: trash error: {e}, remove error: {rem_e}",
                        path.display()
                    )
                })?;
            }
            let sidecar_txt = path.with_extension("txt");
            if sidecar_txt.exists() {
                let _ = trash::delete(&sidecar_txt).or_else(|_| std::fs::remove_file(&sidecar_txt));
            }
        }
        let _ = database.delete_file_by_path(&path_str);
        trashed_count += 1;
    }

    Ok(trashed_count)
}

/// Reveal the selected file in the system file manager (Finder / Explorer / Files).
#[tauri::command]
pub fn reveal_in_file_manager(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("File does not exist: {path}"));
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {e}"))?;
    }

    #[cfg(target_os = "windows")]
    {
        if path.contains('"') {
            return Err("Path contains invalid characters for Explorer".to_string());
        }
        std::process::Command::new("explorer")
            .arg(format!("/select,\"{path}\""))
            .spawn()
            .map_err(|e| format!("Failed to open Explorer: {e}"))?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent = p.parent().unwrap_or(p);
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {e}"))?;
    }

    Ok(())
}

// --- Database Maintenance ---

/// Run SQLite VACUUM and optimize to compact database and reclaim unused disk pages.
#[tauri::command]
pub fn vacuum_database(state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?.vacuum_database().map_err(|e| e.to_string())
}

/// Backup the current database to a designated file destination via VACUUM INTO.
#[tauri::command]
pub fn backup_database(destination_path: String, state: State<'_, AppState>) -> Result<(), String> {
    db(&state)?
        .backup_database(&destination_path)
        .map_err(|e| e.to_string())
}

/// Retrieve database storage and table statistics.
#[tauri::command]
pub fn get_database_stats(state: State<'_, AppState>) -> Result<DatabaseStats, String> {
    db(&state)?.get_database_stats().map_err(|e| e.to_string())
}

/// Restore database from a backup file, verifying integrity and reloading connection.
#[tauri::command]
pub fn restore_database(
    source_path: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri::Manager;
    let src = Path::new(&source_path);
    if !src.exists() {
        return Err(format!("Backup source file does not exist: {source_path}"));
    }

    // Verify backup database is valid and readable
    let _test_db =
        Database::connect(src).map_err(|e| format!("Invalid backup SQLite file: {e}"))?;

    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let active_db_path = data_dir.join("berry.db");

    // Release connection on active database file to avoid Windows lock collision
    {
        let mut guard = state
            .db
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        *guard = Database::connect_in_memory()
            .map_err(|e| format!("Failed to create temporary DB: {e}"))?;
    }

    // Copy backup to active database location
    std::fs::copy(src, &active_db_path)
        .map_err(|e| format!("Failed to copy backup database to active location: {e}"))?;

    // Reopen database connection in AppState
    let new_db = Database::connect(&active_db_path)
        .map_err(|e| format!("Failed to reconnect restored database: {e}"))?;
    let mut guard = state
        .db
        .lock()
        .map_err(|_| "Database lock poisoned".to_string())?;
    *guard = new_db;

    Ok(())
}

/// Open an external URL in the system's default browser.
#[tauri::command]
pub fn open_external_url(url: String, app_handle: AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(format!("Refusing to open non-HTTP(S) URL: {url}"));
    }
    app_handle
        .opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {e}"))?;
    Ok(())
}

/// Request a single thumbnail (lazy on-demand generation).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailRequestArgs {
    pub file_id: i64,
    pub file_path: String,
    pub modified_at: i64,
    pub max_edge: Option<u32>,
    pub cache_budget_mb: Option<u64>,
    pub generation: Option<u64>,
}

#[tauri::command]
pub async fn get_or_create_thumbnail(
    app_handle: AppHandle,
    request: ThumbnailRequestArgs,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let max_edge = request.max_edge.unwrap_or(384);
    let db_path = data_dir.join("berry.db");
    let budget_bytes = thumbnail_budget_bytes(request.cache_budget_mb);
    let generation = request.generation;
    let file_id = request.file_id;
    let file_path = request.file_path;
    let modified_at = request.modified_at;
    let generation_tracker = state.thumbnail_generation.clone();
    if let Some(generation) = generation {
        generation_tracker.fetch_max(generation, Ordering::AcqRel);
    }
    tauri::async_runtime::spawn_blocking(move || {
        let worker_generation_tracker = generation_tracker.clone();
        berry_scan::ensure_thumbnail(
            &data_dir,
            &db_path,
            berry_scan::ThumbnailRequest {
                file_id,
                file_path: &file_path,
                modified_at,
                max_edge,
            },
            budget_bytes,
            move || {
                generation.is_none_or(|generation| {
                    worker_generation_tracker.load(Ordering::Acquire) == generation
                })
            },
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Deserialize)]
pub struct BatchThumbnailItem {
    pub file_id: i64,
    pub file_path: String,
    pub modified_at: i64,
}

/// Request background batch thumbnail generation with progress event emission.
#[tauri::command]
pub async fn batch_generate_thumbnails(
    app_handle: AppHandle,
    items: Vec<BatchThumbnailItem>,
    max_edge: Option<u32>,
    cache_budget_mb: Option<u64>,
    generation: Option<u64>,
    state: State<'_, AppState>,
) -> Result<berry_scan::ThumbnailBatchResult, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let max_edge = max_edge.unwrap_or(384);
    let db_path = data_dir.join("berry.db");
    let budget_bytes = thumbnail_budget_bytes(cache_budget_mb);
    let generation_tracker = state.thumbnail_generation.clone();
    let generation = generation.unwrap_or_else(|| generation_tracker.load(Ordering::Acquire));
    generation_tracker.fetch_max(generation, Ordering::AcqRel);
    let total = items.len();
    let tuples: Vec<(i64, String, i64)> = items
        .into_iter()
        .map(|i| (i.file_id, i.file_path, i.modified_at))
        .collect();

    let app_clone = app_handle.clone();
    let worker_generation_tracker = generation_tracker.clone();
    let count = tauri::async_runtime::spawn_blocking(move || {
        berry_scan::batch_generate_thumbnails(
            &data_dir,
            &db_path,
            tuples,
            max_edge,
            budget_bytes,
            Some(move |current: usize, total: usize| {
                let _ = app_clone.emit(
                    "thumbnail-progress",
                    berry_scan::ThumbnailProgress {
                        current,
                        total,
                        done: current >= total,
                    },
                );
            }),
            move || worker_generation_tracker.load(Ordering::Acquire) == generation,
        )
    })
    .await
    .map_err(|e| format!("Thumbnail generation task failed: {e}"))??;

    let _ = app_handle.emit(
        "thumbnail-progress",
        berry_scan::ThumbnailProgress {
            current: total,
            total,
            done: true,
        },
    );

    Ok(count)
}

/// Cancel queued thumbnail work from older viewport generations.
#[tauri::command]
pub fn cancel_thumbnail_requests(generation: u64, state: State<'_, AppState>) {
    state
        .thumbnail_generation
        .fetch_max(generation, Ordering::AcqRel);
}

/// Get stats for thumbnail cache on disk.
#[tauri::command]
pub async fn get_thumbnail_cache_stats(
    app_handle: AppHandle,
    cache_budget_mb: Option<u64>,
) -> Result<berry_scan::ThumbnailCacheStats, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let db_path = data_dir.join("berry.db");
    let budget_bytes = thumbnail_budget_bytes(cache_budget_mb);
    tauri::async_runtime::spawn_blocking(move || {
        berry_scan::get_thumbnail_cache_stats(&data_dir, &db_path, budget_bytes)
    })
    .await
    .map_err(|error| format!("Thumbnail cache statistics task failed: {error}"))?
}

/// Clear thumbnail cache files from disk.
#[tauri::command]
pub async fn clear_thumbnail_cache(app_handle: AppHandle) -> Result<usize, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let db_path = data_dir.join("berry.db");
    tauri::async_runtime::spawn_blocking(move || {
        berry_scan::clear_thumbnail_cache(&data_dir, &db_path)
    })
    .await
    .map_err(|error| format!("Thumbnail cache clearing task failed: {error}"))?
}

pub(crate) fn thumbnail_budget_bytes(cache_budget_mb: Option<u64>) -> u64 {
    cache_budget_mb
        .unwrap_or_else(default_thumbnail_cache_budget_mb)
        .clamp(256, 65_536)
        .saturating_mul(1024 * 1024)
}

// --- Visual Similarity and Embeddings ---

/// Result item for visual similarity search combining the image file metadata with its match score.
#[derive(Serialize, Deserialize)]
pub struct SimilarFileItem {
    pub file: ImageFile,
    pub score: f32,
}

/// Upsert an embedding vector for an image file.
#[tauri::command]
pub fn upsert_file_embedding(
    file_id: i64,
    model_id: String,
    embedding: Vec<f32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    db(&state)?
        .upsert_file_embedding(file_id, &model_id, &embedding)
        .map_err(|e| e.to_string())
}

/// Remove an embedding record for an image file and model.
#[tauri::command]
pub fn remove_file_embedding(
    file_id: i64,
    model_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    db(&state)?
        .remove_file_embedding(file_id, &model_id)
        .map_err(|e| e.to_string())
}

/// Retrieve the stored embedding vector for an image file and model.
#[tauri::command]
pub fn get_file_embedding(
    file_id: i64,
    model_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Vec<f32>>, String> {
    db(&state)?
        .get_file_embedding(file_id, &model_id)
        .map_err(|e| e.to_string())
}

/// List all model IDs for which embeddings exist for an image file.
#[tauri::command]
pub fn get_file_embedding_models(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    db(&state)?
        .get_file_embedding_models(file_id)
        .map_err(|e| e.to_string())
}

/// Search for files similar to `query` vector using cosine similarity.
#[tauri::command]
pub fn search_similar_files(
    model_id: String,
    query: Vec<f32>,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<SimilarityMatch>, String> {
    db(&state)?
        .search_similar_files(&model_id, &query, limit)
        .map_err(|e| e.to_string())
}

/// Find files visually similar to an existing image file.
///
/// Returns matching files enriched with metadata and similarity score,
/// ordered by descending similarity score.
#[tauri::command]
pub fn find_similar_to_file(
    file_id: i64,
    model_id: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<SimilarFileItem>, String> {
    let database = db(&state)?;
    let lim = limit.unwrap_or(100);
    let matches = database
        .find_similar_to_file(file_id, model_id.as_deref(), lim)
        .map_err(|e| e.to_string())?;

    let mut results = Vec::with_capacity(matches.len());
    for m in matches {
        if let Some(file) = database
            .get_file_by_id(m.file_id)
            .map_err(|e| e.to_string())?
        {
            results.push(SimilarFileItem {
                file,
                score: m.score,
            });
        }
    }
    Ok(results)
}

// --- WD14 Tagger & AI Tagging ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggerModelSummary {
    pub name: String,
    pub dir_path: String,
    pub model_path: String,
    pub tags_path: String,
    pub is_loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTagResult {
    pub processed_files: usize,
    pub tags_added: usize,
}

/// Scan for available WD14 Tagger models.
/// Checks `<AppData>/models/wd14/` and any model currently loaded in state.
#[tauri::command]
pub fn list_tagger_models(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<TaggerModelSummary>, String> {
    let mut models = Vec::new();

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let wd14_dir = data_dir.join("models").join("wd14");
    let _ = std::fs::create_dir_all(&wd14_dir);

    let loaded_model_path = {
        let guard = tagger_guard(&state)?;
        guard.as_ref().map(|t| t.model_info.model_path.clone())
    };

    if let Ok(entries) = std::fs::read_dir(&wd14_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let model_path = path.join("model.onnx");
                let tags_path = path.join("selected_tags.csv");

                if model_path.exists() && tags_path.exists() {
                    let dir_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("wd14")
                        .to_string();
                    let is_loaded = loaded_model_path
                        .as_ref()
                        .map(|p| p == &model_path)
                        .unwrap_or(false);

                    models.push(TaggerModelSummary {
                        name: dir_name,
                        dir_path: path.to_string_lossy().to_string(),
                        model_path: model_path.to_string_lossy().to_string(),
                        tags_path: tags_path.to_string_lossy().to_string(),
                        is_loaded,
                    });
                }
            }
        }
    }

    // If currently loaded model is outside app data models dir, make sure it is also included
    if let Some(guard) = tagger_guard(&state)?.as_ref() {
        let loaded_path_str = guard.model_info.model_path.to_string_lossy().to_string();
        if !models.iter().any(|m| m.model_path == loaded_path_str) {
            models.push(TaggerModelSummary {
                name: guard.model_info.name.clone(),
                dir_path: guard
                    .model_info
                    .model_path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                model_path: loaded_path_str,
                tags_path: guard.model_info.tags_path.to_string_lossy().to_string(),
                is_loaded: true,
            });
        }
    }

    Ok(models)
}

/// Load a WD14 Tagger ONNX model and its selected_tags.csv.
#[tauri::command]
pub fn load_tagger_model(
    model_path: String,
    tags_path: String,
    state: State<'_, AppState>,
) -> Result<ModelInfo, String> {
    let tagger = Wd14Tagger::load(Path::new(&model_path), Path::new(&tags_path))
        .map_err(|e| e.to_string())?;
    let info = tagger.model_info.clone();

    let mut guard = tagger_guard(&state)?;
    *guard = Some(tagger);

    Ok(info)
}

/// Retrieve the currently active tagger model info, if loaded.
#[tauri::command]
pub fn get_loaded_tagger_model(state: State<'_, AppState>) -> Result<Option<ModelInfo>, String> {
    let guard = tagger_guard(&state)?;
    Ok(guard.as_ref().map(|t| t.model_info.clone()))
}

/// Run tag prediction on a single image file.
/// If `apply_tags` is true, newly recognized tags will be created in the database and linked to the image.
#[tauri::command]
pub fn auto_tag_file(
    file_id: i64,
    config: TaggerConfig,
    apply_tags: bool,
    state: State<'_, AppState>,
) -> Result<Vec<TagPrediction>, String> {
    let file = {
        let database = db(&state)?;
        database
            .get_file_by_id(file_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("File with id {file_id} not found"))?
    };

    let predictions = {
        let guard = tagger_guard(&state)?;
        let tagger = guard
            .as_ref()
            .ok_or_else(|| "No WD14 tagger model loaded. Please load a model first.".to_string())?;
        tagger
            .predict_file(Path::new(&file.path), &config)
            .map_err(|e| e.to_string())?
    };

    if apply_tags {
        let database = db(&state)?;
        for pred in &predictions {
            let tag = database
                .get_or_create_tag(&pred.name, None)
                .map_err(|e| e.to_string())?;
            let _ = database.tag_file(file_id, tag.id);
        }
    }

    Ok(predictions)
}

/// Batch run tag prediction across multiple image files and attach recognized tags.
#[tauri::command]
pub fn batch_auto_tag_files(
    file_ids: Vec<i64>,
    config: TaggerConfig,
    state: State<'_, AppState>,
) -> Result<BatchTagResult, String> {
    let mut processed_files = 0;
    let mut tags_added = 0;

    for fid in file_ids {
        let file = {
            let database = db(&state)?;
            database.get_file_by_id(fid).map_err(|e| e.to_string())?
        };

        if let Some(file) = file {
            let predictions = {
                let guard = tagger_guard(&state)?;
                if let Some(tagger) = guard.as_ref() {
                    tagger.predict_file(Path::new(&file.path), &config).ok()
                } else {
                    return Err(
                        "No WD14 tagger model loaded. Please load a model first.".to_string()
                    );
                }
            };

            if let Some(predictions) = predictions {
                let database = db(&state)?;
                for pred in predictions {
                    if let Ok(tag) = database.get_or_create_tag(&pred.name, None) {
                        if database.tag_file(fid, tag.id).is_ok() {
                            tags_added += 1;
                        }
                    }
                }
                processed_files += 1;
            }
        }
    }

    Ok(BatchTagResult {
        processed_files,
        tags_added,
    })
}

// --- CLIP / SigLIP Multi-Modal Semantic Search ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipModelSummary {
    pub name: String,
    pub dir_path: String,
    pub visual_path: String,
    pub textual_path: String,
    pub tokenizer_path: String,
    pub is_loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipIndexStatus {
    pub model_id: String,
    pub indexed_images: usize,
    pub total_images: usize,
    pub is_loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipBatchIndexResult {
    pub indexed_count: usize,
    pub remaining_count: usize,
    pub total_count: usize,
}

/// Scan for available CLIP / SigLIP models.
/// Checks `<AppData>/models/clip/` directory and any model currently loaded in state.
#[tauri::command]
pub fn list_clip_models(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<ClipModelSummary>, String> {
    let mut models = Vec::new();

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let clip_dir = data_dir.join("models").join("clip");
    let _ = std::fs::create_dir_all(&clip_dir);

    let loaded_folder_path = {
        let guard = clip_guard(&state)?;
        guard.as_ref().map(|c| c.info.folder_path.clone())
    };

    if let Ok(entries) = std::fs::read_dir(&clip_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let has_visual = path.join("visual.onnx").is_file()
                    || path.join("vision_model.onnx").is_file()
                    || path.join("model.onnx").is_file();
                let has_textual =
                    path.join("textual.onnx").is_file() || path.join("text_model.onnx").is_file();
                let has_tokenizer = path.join("tokenizer.json").is_file();

                if has_visual && has_textual && has_tokenizer {
                    let dir_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("clip-model")
                        .to_string();

                    let is_loaded = loaded_folder_path
                        .as_ref()
                        .map(|p| p == &path)
                        .unwrap_or(false);

                    let visual_path = if path.join("visual.onnx").is_file() {
                        path.join("visual.onnx")
                    } else if path.join("vision_model.onnx").is_file() {
                        path.join("vision_model.onnx")
                    } else {
                        path.join("model.onnx")
                    };

                    let textual_path = if path.join("textual.onnx").is_file() {
                        path.join("textual.onnx")
                    } else {
                        path.join("text_model.onnx")
                    };

                    models.push(ClipModelSummary {
                        name: dir_name,
                        dir_path: path.to_string_lossy().to_string(),
                        visual_path: visual_path.to_string_lossy().to_string(),
                        textual_path: textual_path.to_string_lossy().to_string(),
                        tokenizer_path: path.join("tokenizer.json").to_string_lossy().to_string(),
                        is_loaded,
                    });
                }
            }
        }
    }

    // If currently loaded model is outside app data models dir, make sure it is also included
    if let Some(guard) = clip_guard(&state)?.as_ref() {
        let loaded_path_str = guard.info.folder_path.to_string_lossy().to_string();
        if !models.iter().any(|m| m.dir_path == loaded_path_str) {
            models.push(ClipModelSummary {
                name: guard.info.name.clone(),
                dir_path: loaded_path_str,
                visual_path: guard.info.visual_model_path.to_string_lossy().to_string(),
                textual_path: guard.info.textual_model_path.to_string_lossy().to_string(),
                tokenizer_path: guard.info.tokenizer_path.to_string_lossy().to_string(),
                is_loaded: true,
            });
        }
    }

    Ok(models)
}

/// Load a CLIP / SigLIP model from its folder directory.
#[tauri::command]
pub fn load_clip_model(
    dir_path: String,
    state: State<'_, AppState>,
) -> Result<ClipModelInfo, String> {
    let engine = ClipEngine::load_from_dir(Path::new(&dir_path)).map_err(|e| e.to_string())?;
    let info = engine.info.clone();

    let mut guard = clip_guard(&state)?;
    *guard = Some(engine);

    Ok(info)
}

/// Retrieve the currently active CLIP model info, if loaded.
#[tauri::command]
pub fn get_loaded_clip_model(state: State<'_, AppState>) -> Result<Option<ClipModelInfo>, String> {
    let guard = clip_guard(&state)?;
    Ok(guard.as_ref().map(|c| c.info.clone()))
}

/// Get embedding index progress statistics for the currently loaded model or a specified model_id.
#[tauri::command]
pub fn get_clip_index_status(
    model_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<ClipIndexStatus, String> {
    let active_model_id = {
        let guard = clip_guard(&state)?;
        guard.as_ref().map(|c| c.info.model_id.clone())
    };

    let target_model_id = match model_id {
        Some(m) if !m.trim().is_empty() => m,
        _ => active_model_id
            .clone()
            .unwrap_or_else(|| "clip-vit-base-patch32".to_string()),
    };

    let database = db(&state)?;
    let (indexed_images, total_images) = database
        .get_embedding_index_stats(&target_model_id)
        .map_err(|e| e.to_string())?;

    let is_loaded = active_model_id
        .map(|m| m == target_model_id)
        .unwrap_or(false);

    Ok(ClipIndexStatus {
        model_id: target_model_id,
        indexed_images,
        total_images,
        is_loaded,
    })
}

/// Process a batch of unindexed images using the active CLIP vision model and save embeddings to database.
#[tauri::command]
pub fn index_clip_images_batch(
    batch_size: usize,
    state: State<'_, AppState>,
) -> Result<ClipBatchIndexResult, String> {
    let guard = clip_guard(&state)?;
    let engine = guard
        .as_ref()
        .ok_or_else(|| "No CLIP model loaded. Please load a model first.".to_string())?;

    let model_id = engine.info.model_id.clone();
    let limit = if batch_size == 0 { 20 } else { batch_size };

    let unindexed = {
        let database = db(&state)?;
        database
            .get_unindexed_files(&model_id, limit)
            .map_err(|e| e.to_string())?
    };

    let mut indexed_count = 0;
    for file in &unindexed {
        let file_id = match file.id {
            Some(id) => id,
            None => continue,
        };

        if let Ok(img) = image::open(&file.path) {
            if let Ok(embedding) = engine.encode_image(&img) {
                let database = db(&state)?;
                if database
                    .upsert_file_embedding(file_id, &model_id, &embedding)
                    .is_ok()
                {
                    indexed_count += 1;
                }
            }
        }
    }

    let database = db(&state)?;
    let (indexed_total, total_images) = database
        .get_embedding_index_stats(&model_id)
        .map_err(|e| e.to_string())?;

    Ok(ClipBatchIndexResult {
        indexed_count,
        remaining_count: total_images.saturating_sub(indexed_total),
        total_count: total_images,
    })
}

/// Text-to-image semantic search: encode text prompt via active CLIP textual model and search database.
#[tauri::command]
pub fn search_by_text_prompt(
    prompt: String,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<SimilarityMatch>, String> {
    if prompt.trim().is_empty() {
        return Ok(vec![]);
    }

    let guard = clip_guard(&state)?;
    let engine = guard
        .as_ref()
        .ok_or_else(|| "No CLIP model loaded. Please load a CLIP model first.".to_string())?;

    let model_id = engine.info.model_id.clone();
    let query_vector = engine.encode_text(&prompt).map_err(|e| e.to_string())?;

    let database = db(&state)?;
    let results = database
        .search_similar_files(&model_id, &query_vector, limit)
        .map_err(|e| e.to_string())?;

    Ok(results)
}

/// Helper to strip basic HTML tags from descriptions (e.g. Civitai info).
fn strip_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut inside = false;
    for c in input.chars() {
        match c {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => result.push(c),
            _ => {}
        }
    }
    result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .trim()
        .to_string()
}

/// List all saved LoRAs from the database catalog.
#[tauri::command]
pub fn list_loras(state: State<'_, AppState>) -> Result<Vec<LoraModel>, String> {
    let database = db(&state)?;
    database.list_loras().map_err(|e| e.to_string())
}

/// Retrieve a single LoRA by its ID.
#[tauri::command]
pub fn get_lora(id: i64, state: State<'_, AppState>) -> Result<Option<LoraModel>, String> {
    let database = db(&state)?;
    database.get_lora(id).map_err(|e| e.to_string())
}

/// Save (create or update) a LoRA entry in the catalog.
#[tauri::command]
pub fn save_lora(lora: LoraModel, state: State<'_, AppState>) -> Result<LoraModel, String> {
    let database = db(&state)?;
    database.save_lora(&lora).map_err(|e| e.to_string())
}

/// Delete a LoRA entry by ID.
#[tauri::command]
pub fn delete_lora(id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    let database = db(&state)?;
    database.delete_lora(id).map_err(|e| e.to_string())
}

/// Extract all LoRAs detected in an image's prompt or ComfyUI workflow graph,
/// resolving any known trigger words and details from the local LoRA database.
#[tauri::command]
pub fn get_image_detected_loras(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<DetectedLora>, String> {
    let database = db(&state)?;
    let file = database
        .get_file_by_id(file_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("File not found with id {file_id}"))?;

    let mut detected = Vec::new();
    if let Some(ref meta) = file.metadata {
        detected = berry_metadata::lora::extract_loras(
            meta.prompt.as_deref(),
            meta.raw.as_deref().or(meta.parameters.as_deref()),
        );
        for l in &mut detected {
            if let Ok(Some(m)) = database.find_lora_by_name_or_hash(&l.name) {
                l.model = Some(m);
            }
        }
    }
    Ok(detected)
}

/// Parse a `.civitai.info` or JSON metadata sidecar file and save it to the LoRA database.
#[tauri::command]
pub fn import_lora_civitai_info(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<LoraModel, String> {
    let path = Path::new(&file_path);
    if !path.is_file() {
        return Err(format!("File does not exist: {file_path}"));
    }

    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read LoRA metadata file: {e}"))?;

    let root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in LoRA metadata file: {e}"))?;

    // Model name resolution
    let name = root
        .get("model")
        .and_then(|m| m.get("name"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            root.get("files")
                .and_then(|f| f.as_array())
                .and_then(|arr| arr.first())
                .and_then(|f0| f0.get("name"))
                .and_then(|v| v.as_str())
        })
        .or_else(|| root.get("name").and_then(|v| v.as_str()))
        .map(berry_metadata::lora::clean_lora_name)
        .unwrap_or_else(|| {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown_lora");
            berry_metadata::lora::clean_lora_name(stem)
        });

    if name.is_empty() {
        return Err("Could not resolve a valid LoRA name from metadata".to_string());
    }

    // Trigger words / activation tags
    let mut trigger_words: Vec<String> = Vec::new();
    if let Some(words) = root.get("trainedWords").and_then(|v| v.as_array()) {
        for w in words {
            if let Some(s) = w.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty()
                    && !trigger_words
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case(trimmed))
                {
                    trigger_words.push(trimmed.to_string());
                }
            }
        }
    } else if let Some(act_text) = root
        .get("activation text")
        .or_else(|| root.get("trigger_words"))
        .and_then(|v| v.as_str())
    {
        for part in act_text.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty()
                && !trigger_words
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(trimmed))
            {
                trigger_words.push(trimmed.to_string());
            }
        }
    }

    // Hash resolution
    let hash = root
        .get("files")
        .and_then(|f| f.as_array())
        .and_then(|arr| arr.first())
        .and_then(|f0| f0.get("hashes"))
        .and_then(|h| {
            h.get("AutoV2")
                .or_else(|| h.get("SHA256"))
                .and_then(|v| v.as_str())
        })
        .or_else(|| {
            root.get("hashes")
                .and_then(|h| h.get("AutoV2").and_then(|v| v.as_str()))
        })
        .or_else(|| root.get("sha256").and_then(|v| v.as_str()))
        .map(|s| s.trim().to_string());

    // Description
    let description = root
        .get("description")
        .and_then(|v| v.as_str())
        .map(strip_html_tags);

    // Recommended / preferred weight
    let weight_default = root
        .get("preferred weight")
        .or_else(|| root.get("weight"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    // Preview URL
    let preview_url = root
        .get("images")
        .and_then(|arr| arr.as_array())
        .and_then(|a| a.first())
        .and_then(|img| img.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    let lora = LoraModel {
        id: 0,
        name,
        hash,
        trigger_words,
        preview_url,
        description,
        weight_default,
        created_at: String::new(),
        updated_at: String::new(),
    };

    let database = db(&state)?;
    database.save_lora(&lora).map_err(|e| e.to_string())
}

/// Recursively scan a directory for LoRA `.civitai.info` or metadata `.json` files
/// and import all found entries into the local LoRA database.
#[tauri::command]
pub fn scan_loras_directory(dir_path: String, state: State<'_, AppState>) -> Result<usize, String> {
    let root = Path::new(&dir_path);
    if !root.is_dir() {
        return Err(format!("Not a directory: {dir_path}"));
    }

    let mut count = 0;
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if (fname.ends_with(".civitai.info") || fname.ends_with(".info"))
                    && import_lora_civitai_info(path.to_string_lossy().to_string(), state.clone())
                        .is_ok()
                {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

// --- Application Configuration & Storage Directory Management ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub locale: String,
    pub auto_scan: bool,
    #[serde(default = "default_startup_scan_interval")]
    pub startup_scan_interval_minutes: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
    pub blur_nsfw: bool,
    pub show_card_badges: bool,
    pub default_view: String,
    pub thumbnail_max_edge: u32,
    #[serde(default = "default_thumbnail_cache_budget_mb")]
    pub thumbnail_cache_budget_mb: u64,
    pub similarity_limit: u32,
    pub auto_check_update: bool,
    pub silent_install: bool,
    #[serde(default)]
    pub has_completed_onboarding: bool,
    #[serde(default = "default_auto_stack")]
    pub auto_stack: bool,
    #[serde(default = "default_stack_similarity")]
    pub stack_similarity_threshold: f64,
    #[serde(default = "default_stack_time_window")]
    pub stack_time_window_minutes: i64,
    #[serde(default)]
    pub allow_multiple_open_stacks: bool,
    #[serde(default)]
    pub suppressed_warnings: Vec<String>,
}

fn default_auto_stack() -> bool {
    false
}

fn default_startup_scan_interval() -> u32 {
    360
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_thumbnail_cache_budget_mb() -> u64 {
    2048
}

fn default_stack_similarity() -> f64 {
    0.85
}

fn default_stack_time_window() -> i64 {
    180
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: "auto".to_string(),
            auto_scan: false,
            startup_scan_interval_minutes: default_startup_scan_interval(),
            theme: default_theme(),
            blur_nsfw: true,
            show_card_badges: true,
            default_view: "grid".to_string(),
            thumbnail_max_edge: 384,
            thumbnail_cache_budget_mb: default_thumbnail_cache_budget_mb(),
            similarity_limit: 50,
            auto_check_update: true,
            silent_install: false,
            has_completed_onboarding: false,
            auto_stack: false,
            stack_similarity_threshold: 0.85,
            stack_time_window_minutes: 180,
            allow_multiple_open_stacks: false,
            suppressed_warnings: Vec::new(),
        }
    }
}

#[cfg(test)]
mod app_config_tests {
    use super::AppConfig;

    #[test]
    fn legacy_config_receives_defaults_for_new_preferences() {
        let mut value = serde_json::to_value(AppConfig::default()).unwrap();
        value.as_object_mut().unwrap().remove("suppressed_warnings");
        value
            .as_object_mut()
            .unwrap()
            .remove("startup_scan_interval_minutes");
        value.as_object_mut().unwrap().remove("theme");
        value
            .as_object_mut()
            .unwrap()
            .remove("thumbnail_cache_budget_mb");

        let config: AppConfig = serde_json::from_value(value).unwrap();
        assert!(config.suppressed_warnings.is_empty());
        assert_eq!(config.startup_scan_interval_minutes, 360);
        assert_eq!(config.theme, "system");
        assert_eq!(config.thumbnail_cache_budget_mb, 2048);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePaths {
    pub data_dir: String,
    pub config_file: String,
    pub database_file: String,
    pub thumbnails_dir: String,
    pub models_dir: String,
    pub updates_dir: String,
}

/// Retrieve the persisted application configuration from config.json.
#[tauri::command]
pub fn get_app_config(app: AppHandle) -> Result<AppConfig, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_path = data_dir.join("config.json");

    if config_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return Ok(cfg);
            }
        }
    }

    // Default configuration if not found or corrupted
    let default_cfg = AppConfig::default();
    if let Ok(json) = serde_json::to_string_pretty(&default_cfg) {
        let _ = std::fs::create_dir_all(&data_dir);
        let _ = std::fs::write(&config_path, json);
    }
    Ok(default_cfg)
}

/// Save the application configuration to config.json in the app data directory.
#[tauri::command]
pub fn save_app_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let _ = std::fs::create_dir_all(&data_dir);
    let config_path = data_dir.join("config.json");
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize configuration: {e}"))?;
    std::fs::write(&config_path, json)
        .map_err(|e| format!("Failed to write configuration file: {e}"))?;
    Ok(())
}

/// Retrieve all standard storage and cache paths.
#[tauri::command]
pub fn get_storage_paths(app: AppHandle) -> Result<StoragePaths, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let _ = std::fs::create_dir_all(&data_dir);
    let _ = std::fs::create_dir_all(data_dir.join("thumbnails"));
    let _ = std::fs::create_dir_all(data_dir.join("models"));
    let _ = std::fs::create_dir_all(data_dir.join("updates"));

    Ok(StoragePaths {
        data_dir: data_dir.to_string_lossy().to_string(),
        config_file: data_dir.join("config.json").to_string_lossy().to_string(),
        database_file: data_dir.join("berry.db").to_string_lossy().to_string(),
        thumbnails_dir: data_dir.join("thumbnails").to_string_lossy().to_string(),
        models_dir: data_dir.join("models").to_string_lossy().to_string(),
        updates_dir: data_dir.join("updates").to_string_lossy().to_string(),
    })
}

/// Open a designated storage directory or highlight a file in the system file manager.
#[tauri::command]
pub fn open_storage_dir(app: AppHandle, target: String) -> Result<(), String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    let path_to_open = match target.as_str() {
        "config" => data_dir.join("config.json"),
        "database" => data_dir.join("berry.db"),
        "thumbnails" => data_dir.join("thumbnails"),
        "models" => data_dir.join("models"),
        "updates" => data_dir.join("updates"),
        _ => data_dir.clone(),
    };

    if !path_to_open.exists() {
        if path_to_open.is_file() || target == "config" || target == "database" {
            // Parent dir must exist
            let _ = std::fs::create_dir_all(&data_dir);
        } else {
            let _ = std::fs::create_dir_all(&path_to_open);
        }
    }

    let p_str = path_to_open.to_string_lossy().to_string();
    if path_to_open.is_file() {
        reveal_in_file_manager(p_str)
    } else {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(&p_str)
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {e}"))?;
            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&p_str)
                .spawn()
                .map_err(|e| format!("Failed to open Finder: {e}"))?;
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&p_str)
                .spawn()
                .map_err(|e| format!("Failed to open file manager: {e}"))?;
            Ok(())
        }
    }
}

// --- In-App Auto Update Download & In-Place Installation ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub speed_bytes_per_sec: u64,
    pub done: bool,
    pub target_file: Option<String>,
}

/// Download an update asset directly into updates/ directory, reporting progress via events.
#[tauri::command]
pub async fn download_update(
    app: AppHandle,
    url: String,
    filename: String,
) -> Result<String, String> {
    use std::io::{Read, Write};

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let updates_dir = data_dir.join("updates");
    std::fs::create_dir_all(&updates_dir).map_err(|e| e.to_string())?;

    let clean_filename = Path::new(&filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("update_installer.exe")
        .to_string();
    let dest_path = updates_dir.join(&clean_filename);
    let tmp_path = updates_dir.join(format!("{clean_filename}.tmp"));

    let app_clone = app.clone();
    let url_clone = url.clone();
    let dest_path_clone = dest_path.clone();

    tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let agent = ureq::builder()
            .redirects(10)
            .timeout(std::time::Duration::from_secs(600))
            .build();

        let resp = agent
            .get(&url_clone)
            .set("User-Agent", "Berry-AI-Studio-Updater")
            .call()
            .map_err(|e| format!("Download request failed: {e}"))?;

        let total_bytes = resp
            .header("content-length")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let mut reader = resp.into_reader();
        let mut file = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create temporary update file: {e}"))?;

        let mut buffer = [0u8; 64 * 1024];
        let mut downloaded_bytes = 0u64;
        let mut last_progress_time = std::time::Instant::now();
        let mut bytes_since_last_progress = 0u64;
        let mut speed_bytes_per_sec = 0u64;

        loop {
            let read_len = reader
                .read(&mut buffer)
                .map_err(|e| format!("Failed reading download stream: {e}"))?;

            if read_len == 0 {
                break;
            }

            file.write_all(&buffer[..read_len])
                .map_err(|e| format!("Failed writing to update file: {e}"))?;

            downloaded_bytes += read_len as u64;
            bytes_since_last_progress += read_len as u64;

            let elapsed = last_progress_time.elapsed();
            if elapsed >= std::time::Duration::from_millis(200) {
                let secs = elapsed.as_secs_f64();
                if secs > 0.0 {
                    speed_bytes_per_sec = (bytes_since_last_progress as f64 / secs) as u64;
                }
                last_progress_time = std::time::Instant::now();
                bytes_since_last_progress = 0;

                let percent = if total_bytes > 0 {
                    (downloaded_bytes as f64 / total_bytes as f64 * 100.0).min(100.0)
                } else {
                    0.0
                };

                let _ = app_clone.emit(
                    "update-download-progress",
                    UpdateDownloadProgress {
                        downloaded_bytes,
                        total_bytes,
                        percent,
                        speed_bytes_per_sec,
                        done: false,
                        target_file: None,
                    },
                );
            }
        }

        file.flush()
            .map_err(|e| format!("Failed to flush update file: {e}"))?;
        drop(file);

        if dest_path_clone.exists() {
            let _ = std::fs::remove_file(&dest_path_clone);
        }
        std::fs::rename(&tmp_path, &dest_path_clone)
            .map_err(|e| format!("Failed to rename update file: {e}"))?;

        let final_path_str = dest_path_clone.to_string_lossy().to_string();

        let _ = app_clone.emit(
            "update-download-progress",
            UpdateDownloadProgress {
                downloaded_bytes,
                total_bytes: if total_bytes == 0 {
                    downloaded_bytes
                } else {
                    total_bytes
                },
                percent: 100.0,
                speed_bytes_per_sec: 0,
                done: true,
                target_file: Some(final_path_str.clone()),
            },
        );

        Ok(final_path_str)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Launch the downloaded installer to execute in-place upgrade and cleanly exit current process.
#[tauri::command]
pub fn install_update(
    app: AppHandle,
    installer_path: String,
    silent: Option<bool>,
) -> Result<(), String> {
    let p = Path::new(&installer_path);
    if !p.exists() {
        return Err(format!("Installer file not found: {installer_path}"));
    }

    let _is_silent = silent.unwrap_or(false);

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let mut cmd = Command::new(&installer_path);
        if _is_silent {
            cmd.arg("/S");
        }
        cmd.spawn()
            .map_err(|e| format!("Failed to launch installer: {e}"))?;
        app.exit(0);
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if installer_path.ends_with(".dmg") {
            Command::new("open")
                .arg(&installer_path)
                .spawn()
                .map_err(|e| format!("Failed to open DMG: {e}"))?;
        } else {
            Command::new("open")
                .arg("-R")
                .arg(&installer_path)
                .spawn()
                .map_err(|e| format!("Failed to reveal update: {e}"))?;
        }
        app.exit(0);
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        if installer_path.ends_with(".AppImage") {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&installer_path) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&installer_path, perms);
                }
            }
            Command::new(&installer_path)
                .spawn()
                .map_err(|e| format!("Failed to launch AppImage: {e}"))?;
            app.exit(0);
        } else {
            let parent = p.parent().unwrap_or(p);
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open file manager: {e}"))?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// AIGC Ingestion Pipeline & Local AI Tool Autodetection
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn autodetect_local_ai_paths() -> Result<Vec<PipelineDetectedPath>, String> {
    let mut detected = Vec::new();

    #[cfg(target_os = "windows")]
    let drive_roots: Vec<String> = (b'C'..=b'Z')
        .map(|d| format!("{}:\\", d as char))
        .filter(|root| Path::new(root).exists())
        .collect();

    #[cfg(not(target_os = "windows"))]
    let drive_roots: Vec<String> = vec![
        std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()),
        "/opt".to_string(),
        "/media".to_string(),
    ];

    let patterns = [
        (
            "Stable Diffusion WebUI",
            "stable-diffusion-webui/outputs/txt2img-images",
            "txt2img",
        ),
        (
            "Stable Diffusion WebUI",
            "stable-diffusion-webui/outputs/img2img-images",
            "img2img",
        ),
        (
            "Stable Diffusion WebUI",
            "sd.webui/outputs/txt2img-images",
            "txt2img",
        ),
        (
            "Stable Diffusion WebUI (Aki)",
            "sd-webui-aki/outputs/txt2img-images",
            "txt2img",
        ),
        (
            "Stable Diffusion WebUI (Aki)",
            "sd-webui-aki/outputs/img2img-images",
            "img2img",
        ),
        ("ComfyUI", "ComfyUI/output", "output"),
        (
            "ComfyUI (Portable)",
            "ComfyUI_windows_portable/ComfyUI/output",
            "output",
        ),
        ("ComfyUI (Aki)", "ComfyUI-aki/ComfyUI/output", "output"),
        ("Fooocus", "Fooocus/outputs", "outputs"),
        ("Fooocus (MRE)", "Fooocus-MRE/outputs", "outputs"),
        ("InvokeAI", "invokeai/outputs", "outputs"),
    ];

    for root in drive_roots {
        let root_path = Path::new(&root);
        for (tool, rel_path, cat) in &patterns {
            let candidate = root_path.join(rel_path);
            if candidate.is_dir() {
                let p = candidate.display().to_string();
                if !detected.iter().any(|d: &PipelineDetectedPath| d.path == p) {
                    detected.push(PipelineDetectedPath {
                        tool_name: tool.to_string(),
                        path: p,
                        category: cat.to_string(),
                    });
                }
            }
        }
    }

    Ok(detected)
}

#[tauri::command]
pub fn harvest_pipeline_folder(
    folder_id: i64,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let folder = {
        let db = db(&state)?;
        db.find_folder_by_id(folder_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "folder not found".to_string())?
    };

    if folder.folder_type != "pipeline" {
        return Err("folder is not an ingestion pipeline".to_string());
    }

    let source_path_str = match &folder.source_path {
        Some(s) if !s.is_empty() => s.clone(),
        _ => return Err("pipeline folder has no source path configured".to_string()),
    };

    let source_dir = Path::new(&source_path_str);
    if !source_dir.is_dir() {
        return Err(format!(
            "pipeline source path does not exist: {source_path_str}"
        ));
    }

    let dest_dir = Path::new(&folder.path);
    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("failed to create destination directory: {e}"))?;

    let supported_exts = ["png", "jpg", "jpeg", "webp", "mp4"];
    let mut harvested_count = 0;

    let entries = std::fs::read_dir(source_dir)
        .map_err(|e| format!("failed to read source directory: {e}"))?;

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    for entry in entries.flatten() {
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !supported_exts.contains(&ext.as_str()) {
            continue;
        }

        let meta = match file_path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if meta.len() == 0 {
            continue;
        }

        let file_mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if now_ts.saturating_sub(file_mtime) < 1 {
            // Still being written to, debounce
            continue;
        }

        let file_name = match file_path.file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => continue,
        };

        let target_path = dest_dir.join(&file_name);
        let src_str = file_path.display().to_string();
        let tgt_str = target_path.display().to_string();

        if target_path.exists() {
            if let Ok(t_meta) = target_path.metadata() {
                if t_meta.len() == meta.len() {
                    continue;
                }
            }
        }

        if let Err(e) = std::fs::copy(&file_path, &target_path) {
            eprintln!("Failed to copy {src_str} to {tgt_str}: {e}");
            continue;
        }

        let src_txt = file_path.with_extension("txt");
        if src_txt.exists() {
            let tgt_txt = target_path.with_extension("txt");
            let _ = std::fs::copy(&src_txt, &tgt_txt);
        }

        let container = match ext.as_str() {
            "png" => berry_domain::Container::Png,
            "jpg" | "jpeg" => berry_domain::Container::Jpeg,
            "webp" => berry_domain::Container::WebP,
            "mp4" => berry_domain::Container::Mp4,
            _ => continue,
        };

        let metadata = berry_metadata::extract_metadata(container, &target_path);

        let image_file = ImageFile {
            id: None,
            folder_id,
            path: tgt_str.clone(),
            size_bytes: meta.len(),
            modified_at: file_mtime,
            container,
            metadata,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };

        let db = db(&state)?;
        if let Ok(inserted_id) = db.upsert_file(&image_file) {
            harvested_count += 1;

            if folder.ingest_action.as_deref() == Some("move") {
                let grace = folder.grace_period_hours.unwrap_or(24);
                if grace <= 0 {
                    let _ = trash::delete(&file_path);
                    if src_txt.exists() {
                        let _ = trash::delete(&src_txt);
                    }
                } else {
                    let _ = db.enqueue_cleanup(&src_str, inserted_id, grace);
                }
            }
        }
    }

    Ok(harvested_count)
}

#[tauri::command]
pub fn process_pipeline_cleanups(state: State<'_, AppState>) -> Result<u64, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let db = db(&state)?;
    let due_items = db.list_due_cleanups(now).map_err(|e| e.to_string())?;
    let mut deleted_count = 0;

    for item in due_items {
        let p = Path::new(&item.source_file_path);
        if p.exists() {
            if let Err(e) = trash::delete(p) {
                eprintln!(
                    "Failed to trash expired pipeline file {}: {e}",
                    item.source_file_path
                );
                let _ = db.update_cleanup_status(item.id, "failed");
                continue;
            }
        }
        let txt = p.with_extension("txt");
        if txt.exists() {
            let _ = trash::delete(&txt);
        }

        let _ = db.update_cleanup_status(item.id, "deleted");
        deleted_count += 1;
    }

    Ok(deleted_count)
}

#[tauri::command]
pub fn get_pipeline_cleanup_queue(
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<CleanupQueueItem>, String> {
    let db = db(&state)?;
    db.get_cleanup_queue(limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Image Stacking & Burst Grouping
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn stack_images(
    file_ids: Vec<i64>,
    custom_stack_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = db(&state)?;
    db.stack_images(&file_ids, custom_stack_id.as_deref())
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct StackMergeResult {
    pub stack_id: String,
    pub members: Vec<ImageFile>,
}

#[tauri::command]
pub fn merge_stacks(
    target_stack_id: String,
    source_stack_ids: Vec<String>,
    standalone_file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<StackMergeResult, String> {
    let db = db(&state)?;
    db.merge_stacks(&target_stack_id, &source_stack_ids, &standalone_file_ids)
        .map_err(|e| e.to_string())?;
    let members = db
        .get_stack_members(&target_stack_id)
        .map_err(|e| e.to_string())?;
    Ok(StackMergeResult {
        stack_id: target_stack_id,
        members,
    })
}

#[tauri::command]
pub fn unstack_images(stack_id: String, state: State<'_, AppState>) -> Result<u64, String> {
    let db = db(&state)?;
    db.unstack_images(&stack_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_stack_hero(
    stack_id: String,
    hero_file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = db(&state)?;
    db.set_stack_hero(&stack_id, hero_file_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_stack_members(
    stack_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ImageFile>, String> {
    let db = db(&state)?;
    db.get_stack_members(&stack_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_stacks(
    folder_id: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<StackSummary>, String> {
    let db = db(&state)?;
    db.list_stacks(folder_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn cull_stack_drafts(
    stack_id: String,
    min_rating: u8,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let db = db(&state)?;
    let cull_ids = db
        .get_stack_cull_candidate_ids(&stack_id, min_rating)
        .map_err(|e| e.to_string())?;

    let mut trashed = 0;
    for id in cull_ids {
        if let Ok(Some(file)) = db.get_file_by_id(id) {
            let p = Path::new(&file.path);
            if p.exists() {
                let _ = trash::delete(p);
            }
            let txt = p.with_extension("txt");
            if txt.exists() {
                let _ = trash::delete(&txt);
            }
            let _ = db.delete_file_by_id(id);
            trashed += 1;
        }
    }

    Ok(trashed)
}

#[derive(Debug, Clone, Serialize)]
pub struct AutoStackResult {
    pub created_stacks: usize,
    pub stacked_images: usize,
    pub eligible_images: usize,
    pub skipped_without_prompt: usize,
}

#[tauri::command]
pub fn auto_stack_images(
    folder_id: Option<i64>,
    similarity_threshold: Option<f32>,
    time_window_minutes: Option<i64>,
    state: State<'_, AppState>,
) -> Result<AutoStackResult, String> {
    let db = db(&state)?;
    let threshold = similarity_threshold.unwrap_or(0.85);
    let time_window_secs = time_window_minutes.unwrap_or(180).max(0).saturating_mul(60);

    let criteria = SearchCriteria {
        folder_id,
        sort: Some(FileSortField::ModifiedAt),
        direction: Some(SortDirection::Asc),
        ..Default::default()
    };
    let files = db.search_files(&criteria).map_err(|e| e.to_string())?;

    let candidates = files
        .into_iter()
        .filter_map(|file| {
            Some(PromptStackCandidate {
                file_id: file.id?,
                folder_id: file.folder_id,
                modified_at: file.modified_at,
                prompt: file
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.prompt.clone())
                    .unwrap_or_default(),
                model_name: file
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.model_name.clone()),
                is_stacked: file.stack_id.is_some(),
            })
        })
        .collect::<Vec<_>>();
    let plan = plan_prompt_stacks(&candidates, threshold, time_window_secs);
    let stacked_images = plan.groups.iter().map(Vec::len).sum();
    for group in &plan.groups {
        db.stack_images(group, None)
            .map_err(|error| error.to_string())?;
    }

    Ok(AutoStackResult {
        created_stacks: plan.groups.len(),
        stacked_images,
        eligible_images: plan.eligible_images,
        skipped_without_prompt: plan.skipped_without_prompt,
    })
}
