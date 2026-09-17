//! SQLite database connection and migration runner.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use berry_domain::{
    Album, CheckpointModelStat, CleanupQueueItem, Container, DatabaseStats, ExtractedMetadata,
    FilePage, FileSortField, Folder, ImageFile, LoraModel, ModelCacheEntry, PromptStat,
    SearchCriteria, SimilarityMatch, SortDirection, StackSummary, Tag,
};
use rusqlite::{params, Connection, OpenFlags};
use uuid::Uuid;

use crate::migrations::{LATEST_VERSION, MIGRATIONS};

/// Errors produced by the storage layer.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown container id stored in database: {0}")]
    UnknownContainer(String),
    #[error("no folder with id {0}")]
    FolderNotFound(i64),
    #[error("no file with id {0}")]
    FileNotFound(i64),
    #[error("no album with id {0}")]
    AlbumNotFound(i64),
    #[error("no tag with id {0}")]
    TagNotFound(i64),
    #[error("no lora with id {0}")]
    LoraNotFound(i64),
    #[error("no image stack with id {0}")]
    StackNotFound(String),
    #[error("file {file_id} already belongs to stack {stack_id}")]
    FileAlreadyStacked { file_id: i64, stack_id: String },
    #[error("rating must be between 1 and 10, got {0}")]
    InvalidRating(u8),
    #[error("failed to open database at {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("model ID cannot be blank or empty")]
    BlankModelId,
    #[error("embedding vector cannot be empty")]
    EmptyVector,
    #[error("embedding vector contains non-finite value")]
    NonFiniteVectorValue,
    #[error("embedding vector norm is zero or negligible")]
    ZeroVectorNorm,
    #[error("similarity search limit must be greater than zero")]
    ZeroResultLimit,
    #[error("embedding dimension mismatch for file {file_id}: query dimension {query} does not match stored dimension {stored}")]
    DimensionMismatch {
        file_id: i64,
        query: usize,
        stored: usize,
    },
    #[error("stored embedding BLOB for file {file_id} has invalid length {actual_bytes}, expected {expected_bytes} for {dimensions} dimensions")]
    MalformedStoredBlob {
        file_id: i64,
        dimensions: usize,
        expected_bytes: usize,
        actual_bytes: usize,
    },
    #[error("stored embedding for file {file_id} has invalid dimensions: {dimensions}")]
    InvalidStoredDimensions { file_id: i64, dimensions: i64 },
    #[error("stored embedding for file {file_id} contains corrupt non-finite value")]
    CorruptStoredVectorValue { file_id: i64 },
    #[error("stored embedding for file {file_id} has zero norm")]
    CorruptStoredZeroNorm { file_id: i64 },
}

/// SQL that inserts or updates a file row keyed by its unique path.
const UPSERT_FILE_SQL: &str =
    "INSERT INTO files (folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
     ON CONFLICT(path) DO UPDATE SET
         folder_id       = excluded.folder_id,
         container       = excluded.container,
         size_bytes      = excluded.size_bytes,
         modified_at     = excluded.modified_at,
         metadata        = excluded.metadata,
         rating          = coalesce(excluded.rating, files.rating),
         aesthetic_score = coalesce(excluded.aesthetic_score, files.aesthetic_score),
         is_favorite     = files.is_favorite,
         is_nsfw         = files.is_nsfw,
         stack_id        = coalesce(excluded.stack_id, files.stack_id),
         stack_order     = coalesce(excluded.stack_order, files.stack_order)";

/// A SQLite database with a fully migrated schema.
pub struct Database {
    conn: Connection,
    path: Option<PathBuf>,
}

impl Database {
    /// Open (creating if necessary) the database file at `path` and migrate
    /// it to the latest schema version.
    pub fn connect(path: &Path) -> Result<Self, DatabaseError> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|source| DatabaseError::Open {
            path: path.to_path_buf(),
            source,
        })?;

        Self::init(conn, Some(path.to_path_buf()))
    }

    /// Open an in-memory database, primarily for tests.
    pub fn connect_in_memory() -> Result<Self, DatabaseError> {
        Self::init(Connection::open_in_memory()?, None)
    }

    fn init(conn: Connection, path: Option<PathBuf>) -> Result<Self, DatabaseError> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", -16000)?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        conn.pragma_update(None, "mmap_size", 268435456i64)?;
        conn.pragma_update(None, "busy_timeout", 5000)?;

        let mut db = Self { conn, path };
        db.migrate()?;
        Ok(db)
    }

    /// The path the database was opened from, if any (in-memory has none).
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Apply any pending migrations, advancing `PRAGMA user_version`.
    fn migrate(&mut self) -> Result<(), DatabaseError> {
        let current = self.user_version()?;
        debug_assert!(
            current <= LATEST_VERSION,
            "database schema (v{current}) is newer than this build (v{LATEST_VERSION})"
        );

        for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
            let target = i as i64 + 1;
            self.apply_migration(sql, target)?;
        }
        Ok(())
    }

    /// Apply a single migration script and bump `user_version`, atomically.
    fn apply_migration(&mut self, sql: &str, target_version: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", target_version)?;
        tx.commit()?;
        Ok(())
    }

    /// The current `PRAGMA user_version` of the database.
    pub fn user_version(&self) -> Result<i64, DatabaseError> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        Ok(version)
    }

    /// Read a value from the `meta` table, if present.
    pub fn meta_get(&self, key: &str) -> Result<Option<String>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT value FROM meta WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Run SQLite VACUUM and PRAGMA optimize.
    pub fn vacuum_database(&self) -> Result<(), DatabaseError> {
        self.conn.execute("VACUUM", [])?;
        self.conn.execute("PRAGMA optimize", [])?;
        Ok(())
    }

    /// Point-in-time database backup using SQLite VACUUM INTO.
    pub fn backup_database(&self, destination_path: &str) -> Result<(), DatabaseError> {
        let dest = Path::new(destination_path);
        if dest.exists() {
            std::fs::remove_file(dest)?;
        }
        self.conn.execute("VACUUM INTO ?1", [destination_path])?;
        Ok(())
    }

    /// Retrieve database storage and table statistics.
    pub fn get_database_stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let file_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
        let folder_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM folders", [], |row| row.get(0))?;
        let album_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM albums", [], |row| row.get(0))?;
        let tag_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;
        let model_cache_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM model_cache", [], |row| row.get(0))
            .unwrap_or(0);

        let page_size: i64 = self
            .conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let page_count: i64 = self
            .conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let freelist_count: i64 = self
            .conn
            .query_row("PRAGMA freelist_count", [], |row| row.get(0))?;

        let db_size_bytes = (page_count * page_size) as u64;

        Ok(DatabaseStats {
            file_count,
            folder_count,
            album_count,
            tag_count,
            model_cache_count,
            db_size_bytes,
            page_size,
            page_count,
            freelist_count,
        })
    }

    // --- Folders ---

    /// Insert a new folder with default 'link' mode.
    pub fn add_folder(&self, path: &str) -> Result<Folder, DatabaseError> {
        self.add_folder_with_mode(path, "link", None, None, None, true)
    }

    /// Insert a new folder with explicit mode and optional pipeline parameters.
    pub fn add_folder_with_mode(
        &self,
        path: &str,
        folder_type: &str,
        source_path: Option<&str>,
        ingest_action: Option<&str>,
        grace_period_hours: Option<i32>,
        auto_harvest: bool,
    ) -> Result<Folder, DatabaseError> {
        self.conn.execute(
            "INSERT INTO folders (path, folder_type, source_path, ingest_action, grace_period_hours, auto_harvest)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                path,
                folder_type,
                source_path,
                ingest_action,
                grace_period_hours,
                if auto_harvest { 1 } else { 0 },
            ],
        )?;
        let id = self.conn.last_insert_rowid();
        self.find_folder_by_id(id)?
            .ok_or(DatabaseError::FolderNotFound(id))
    }

    /// All folders, ordered by id.
    pub fn list_folders(&self) -> Result<Vec<Folder>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, path, added_at, folder_type, source_path, ingest_action, grace_period_hours, auto_harvest
             FROM folders ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            let auto_harvest_int: i32 = row.get(7).unwrap_or(1);
            Ok(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
                added_at: row.get(2)?,
                folder_type: row.get(3).unwrap_or_else(|_| "link".to_string()),
                source_path: row.get(4).unwrap_or(None),
                ingest_action: row.get(5).unwrap_or(None),
                grace_period_hours: row.get(6).unwrap_or(None),
                auto_harvest: auto_harvest_int != 0,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    /// The folder registered at `path`, if any.
    pub fn find_folder_by_path(&self, path: &str) -> Result<Option<Folder>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, path, added_at, folder_type, source_path, ingest_action, grace_period_hours, auto_harvest
             FROM folders WHERE path = ?1",
        )?;
        let mut rows = stmt.query([path])?;
        match rows.next()? {
            Some(row) => {
                let auto_harvest_int: i32 = row.get(7).unwrap_or(1);
                Ok(Some(Folder {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    added_at: row.get(2)?,
                    folder_type: row.get(3).unwrap_or_else(|_| "link".to_string()),
                    source_path: row.get(4).unwrap_or(None),
                    ingest_action: row.get(5).unwrap_or(None),
                    grace_period_hours: row.get(6).unwrap_or(None),
                    auto_harvest: auto_harvest_int != 0,
                }))
            }
            None => Ok(None),
        }
    }

    /// The folder registered with `id`, if any.
    pub fn find_folder_by_id(&self, id: i64) -> Result<Option<Folder>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, path, added_at, folder_type, source_path, ingest_action, grace_period_hours, auto_harvest
             FROM folders WHERE id = ?1",
        )?;
        let mut rows = stmt.query([id])?;
        match rows.next()? {
            Some(row) => {
                let auto_harvest_int: i32 = row.get(7).unwrap_or(1);
                Ok(Some(Folder {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    added_at: row.get(2)?,
                    folder_type: row.get(3).unwrap_or_else(|_| "link".to_string()),
                    source_path: row.get(4).unwrap_or(None),
                    ingest_action: row.get(5).unwrap_or(None),
                    grace_period_hours: row.get(6).unwrap_or(None),
                    auto_harvest: auto_harvest_int != 0,
                }))
            }
            None => Ok(None),
        }
    }

    /// Delete a folder and cascade-delete its indexed files.
    ///
    /// Errors with [`DatabaseError::FolderNotFound`] if the id does not exist.
    pub fn remove_folder(&self, id: i64) -> Result<(), DatabaseError> {
        let affected = self
            .conn
            .execute("DELETE FROM folders WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(DatabaseError::FolderNotFound(id));
        }
        Ok(())
    }

    // --- Files ---

    /// Insert or update a file row keyed by its unique path, returning the id.
    ///
    /// `file.metadata` is serialized to JSON and stored in the `metadata`
    /// column; re-upserting the same path keeps a single row and refreshes its
    /// (size, mtime, metadata) — the incremental-scan cache.
    pub fn upsert_file(&self, file: &ImageFile) -> Result<i64, DatabaseError> {
        let metadata = file
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        self.conn.execute(
            UPSERT_FILE_SQL,
            params![
                file.folder_id,
                file.path,
                file.container.id(),
                file.size_bytes as i64,
                file.modified_at,
                metadata,
                file.rating.map(|r| r as i64),
                file.aesthetic_score,
                file.is_favorite as i64,
                file.is_nsfw as i64,
                file.stack_id,
                file.stack_order as i64,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert or update many files in a single transaction, returning the
    /// number of rows written.
    ///
    /// Prefer this over repeated [`upsert_file`](Self::upsert_file) calls when
    /// inserting in bulk (e.g. a folder scan): one commit per batch instead of
    /// one fsync per file.
    pub fn upsert_files(&self, files: &[ImageFile]) -> Result<u64, DatabaseError> {
        if files.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached(UPSERT_FILE_SQL)?;
            for file in files {
                let metadata = file
                    .metadata
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?;
                stmt.execute(params![
                    file.folder_id,
                    file.path,
                    file.container.id(),
                    file.size_bytes as i64,
                    file.modified_at,
                    metadata,
                    file.rating.map(|r| r as i64),
                    file.aesthetic_score,
                    file.is_favorite as i64,
                    file.is_nsfw as i64,
                    file.stack_id,
                    file.stack_order as i64,
                ])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Delete every file of `folder_id` whose path is not in `seen`.
    ///
    /// Used at the end of a scan to drop rows for files that were removed from
    /// disk. `seen` is passed as a JSON array and matched with SQLite's
    /// `json_each`, so any number of paths works without dynamic SQL.
    pub fn delete_files_not_in(
        &self,
        folder_id: i64,
        seen: &[String],
    ) -> Result<u64, DatabaseError> {
        let seen_json = serde_json::to_string(seen)?;
        let affected = self.conn.execute(
            "DELETE FROM files
             WHERE folder_id = ?1
               AND path NOT IN (SELECT value FROM json_each(?2))",
            params![folder_id, seen_json],
        )?;
        Ok(affected as u64)
    }

    /// Helper to deserialize an image file row.
    fn map_row(row: &rusqlite::Row<'_>) -> Result<ImageFile, DatabaseError> {
        let id: i64 = row.get(0)?;
        let folder_id: i64 = row.get(1)?;
        let path: String = row.get(2)?;
        let container_id: String = row.get(3)?;
        let size_bytes: i64 = row.get(4)?;
        let modified_at: i64 = row.get(5)?;
        let metadata: Option<String> = row.get(6)?;
        let rating: Option<i64> = row.get(7)?;
        let aesthetic_score: Option<f64> = row.get(8)?;
        let is_favorite: i64 = row.get(9).unwrap_or(0);
        let is_nsfw: i64 = row.get(10).unwrap_or(0);
        let stack_id: Option<String> = row.get(11).unwrap_or(None);
        let stack_order: i64 = row.get(12).unwrap_or(0);

        let container = Container::from_id(&container_id)
            .ok_or_else(|| DatabaseError::UnknownContainer(container_id))?;
        let metadata = metadata
            .map(|json| serde_json::from_str::<ExtractedMetadata>(&json))
            .transpose()?;

        Ok(ImageFile {
            id: Some(id),
            folder_id,
            path,
            size_bytes: size_bytes as u64,
            modified_at,
            container,
            metadata,
            rating: rating.map(|r| r as u8),
            aesthetic_score,
            is_favorite: is_favorite != 0,
            is_nsfw: is_nsfw != 0,
            stack_id,
            stack_order: stack_order as i32,
        })
    }

    /// Query files with optional folder filtering and multi-criteria sorting.
    pub fn query_files(
        &self,
        folder_id: Option<i64>,
        sort: FileSortField,
        direction: SortDirection,
    ) -> Result<Vec<ImageFile>, DatabaseError> {
        let criteria = SearchCriteria {
            folder_id,
            sort: Some(sort),
            direction: Some(direction),
            ..Default::default()
        };
        self.search_files(&criteria)
    }

    /// Retrieve a file by its file path.
    pub fn get_file_by_path(&self, path: &str) -> Result<Option<ImageFile>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order
             FROM files WHERE path = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query_and_then([path], Self::map_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// Retrieve a file by its database row ID.
    pub fn get_file_by_id(&self, id: i64) -> Result<Option<ImageFile>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order
             FROM files WHERE id = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query_and_then([id], Self::map_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// Update file path and folder_id when a file is moved.
    pub fn move_file_record(
        &self,
        old_path: &str,
        new_path: &str,
        new_folder_id: i64,
    ) -> Result<(), DatabaseError> {
        let affected = self.conn.execute(
            "UPDATE files SET path = ?1, folder_id = ?2 WHERE path = ?3",
            params![new_path, new_folder_id, old_path],
        )?;
        if affected == 0 {
            return Err(DatabaseError::FileNotFound(0));
        }
        Ok(())
    }

    /// Delete a file record by path (e.g. after trashing).
    pub fn delete_file_by_path(&self, path: &str) -> Result<(), DatabaseError> {
        self.conn
            .execute("DELETE FROM files WHERE path = ?1", [path])?;
        Ok(())
    }

    /// Delete a file record by id.
    pub fn delete_file_by_id(&self, id: i64) -> Result<(), DatabaseError> {
        self.conn.execute("DELETE FROM files WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Search files matching various criteria (text, parameters, ratings, folders, sorting, pagination).
    pub fn search_files(&self, criteria: &SearchCriteria) -> Result<Vec<ImageFile>, DatabaseError> {
        Ok(self.search_files_page(criteria)?.items)
    }

    /// Search a bounded page while returning the exact filtered row count in
    /// the same SQLite query. `COUNT(*) OVER()` avoids a second filter pass and
    /// does not materialize the complete result set.
    pub fn search_files_page(&self, criteria: &SearchCriteria) -> Result<FilePage, DatabaseError> {
        let mut conditions = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(fid) = criteria.folder_id {
            conditions.push("folder_id = ?".to_string());
            params.push(rusqlite::types::Value::Integer(fid));
        }

        if let Some(text) = &criteria.text {
            let pattern = format!("%{}%", text);
            conditions.push(
                "(path LIKE ? OR json_extract(metadata, '$.prompt') LIKE ? OR json_extract(metadata, '$.negative_prompt') LIKE ? OR json_extract(metadata, '$.model_name') LIKE ?)".to_string(),
            );
            params.push(rusqlite::types::Value::Text(pattern.clone()));
            params.push(rusqlite::types::Value::Text(pattern.clone()));
            params.push(rusqlite::types::Value::Text(pattern.clone()));
            params.push(rusqlite::types::Value::Text(pattern));
        }

        if let Some(prompt) = &criteria.prompt {
            conditions.push("json_extract(metadata, '$.prompt') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", prompt)));
        }

        if let Some(negative_prompt) = &criteria.negative_prompt {
            conditions.push("json_extract(metadata, '$.negative_prompt') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!(
                "%{}%",
                negative_prompt
            )));
        }

        if let Some(model_name) = &criteria.model_name {
            conditions.push("json_extract(metadata, '$.model_name') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", model_name)));
        }

        if let Some(model_hash) = &criteria.model_hash {
            conditions.push("json_extract(metadata, '$.model_hash') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", model_hash)));
        }

        if let Some(sampler) = &criteria.sampler {
            conditions.push("json_extract(metadata, '$.sampler') LIKE ?".to_string());
            params.push(rusqlite::types::Value::Text(format!("%{}%", sampler)));
        }

        if let Some(min_steps) = criteria.min_steps {
            conditions.push("CAST(json_extract(metadata, '$.steps') AS INTEGER) >= ?".to_string());
            params.push(rusqlite::types::Value::Integer(min_steps as i64));
        }

        if let Some(max_steps) = criteria.max_steps {
            conditions.push("CAST(json_extract(metadata, '$.steps') AS INTEGER) <= ?".to_string());
            params.push(rusqlite::types::Value::Integer(max_steps as i64));
        }

        if let Some(min_cfg) = criteria.min_cfg {
            conditions.push("CAST(json_extract(metadata, '$.cfg_scale') AS REAL) >= ?".to_string());
            params.push(rusqlite::types::Value::Real(min_cfg));
        }

        if let Some(max_cfg) = criteria.max_cfg {
            conditions.push("CAST(json_extract(metadata, '$.cfg_scale') AS REAL) <= ?".to_string());
            params.push(rusqlite::types::Value::Real(max_cfg));
        }

        if let Some(min_rating) = criteria.min_rating {
            conditions.push("rating >= ?".to_string());
            params.push(rusqlite::types::Value::Integer(min_rating as i64));
        }

        if let Some(max_rating) = criteria.max_rating {
            conditions.push("rating <= ?".to_string());
            params.push(rusqlite::types::Value::Integer(max_rating as i64));
        }

        if let Some(min_aesthetic) = criteria.min_aesthetic {
            conditions.push("aesthetic_score >= ?".to_string());
            params.push(rusqlite::types::Value::Real(min_aesthetic));
        }

        if let Some(max_aesthetic) = criteria.max_aesthetic {
            conditions.push("aesthetic_score <= ?".to_string());
            params.push(rusqlite::types::Value::Real(max_aesthetic));
        }

        if let Some(fav) = criteria.is_favorite {
            conditions.push("is_favorite = ?".to_string());
            params.push(rusqlite::types::Value::Integer(if fav { 1 } else { 0 }));
        }

        if let Some(nsfw) = criteria.is_nsfw {
            conditions.push("is_nsfw = ?".to_string());
            params.push(rusqlite::types::Value::Integer(if nsfw { 1 } else { 0 }));
        }

        if let Some(aid) = criteria.album_id {
            conditions
                .push("id IN (SELECT file_id FROM album_files WHERE album_id = ?)".to_string());
            params.push(rusqlite::types::Value::Integer(aid));
        }

        if let Some(tid) = criteria.tag_id {
            conditions.push("id IN (SELECT file_id FROM file_tags WHERE tag_id = ?)".to_string());
            params.push(rusqlite::types::Value::Integer(tid));
        }

        let sort = criteria.sort.unwrap_or(FileSortField::ModifiedAt);
        let direction = criteria.direction.unwrap_or(SortDirection::Desc);
        let order_clause = match (sort, direction) {
            (FileSortField::ModifiedAt, SortDirection::Asc) => "modified_at ASC, id ASC",
            (FileSortField::ModifiedAt, SortDirection::Desc) => "modified_at DESC, id DESC",
            (FileSortField::Path, SortDirection::Asc) => "path ASC",
            (FileSortField::Path, SortDirection::Desc) => "path DESC",
            (FileSortField::SizeBytes, SortDirection::Asc) => "size_bytes ASC, id ASC",
            (FileSortField::SizeBytes, SortDirection::Desc) => "size_bytes DESC, id DESC",
            (FileSortField::Rating, SortDirection::Asc) => {
                "rating ASC NULLS LAST, modified_at DESC, id DESC"
            }
            (FileSortField::Rating, SortDirection::Desc) => {
                "rating DESC NULLS LAST, modified_at DESC, id DESC"
            }
            (FileSortField::AestheticScore, SortDirection::Asc) => {
                "aesthetic_score ASC NULLS LAST, modified_at DESC, id DESC"
            }
            (FileSortField::AestheticScore, SortDirection::Desc) => {
                "aesthetic_score DESC NULLS LAST, modified_at DESC, id DESC"
            }
        };

        let limit_clause = match (criteria.limit, criteria.offset) {
            (Some(limit), Some(offset)) => {
                params.push(rusqlite::types::Value::Integer(limit as i64));
                params.push(rusqlite::types::Value::Integer(offset as i64));
                " LIMIT ? OFFSET ?"
            }
            (Some(limit), None) => {
                params.push(rusqlite::types::Value::Integer(limit as i64));
                " LIMIT ?"
            }
            (None, Some(offset)) => {
                params.push(rusqlite::types::Value::Integer(offset as i64));
                " LIMIT -1 OFFSET ?"
            }
            (None, None) => "",
        };

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT id, folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order, COUNT(*) OVER() AS total_count
             FROM files{where_clause} ORDER BY {order_clause}{limit_clause}"
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(rusqlite::params_from_iter(&params))?;
        let mut files = Vec::new();
        let mut total = 0usize;
        while let Some(row) = rows.next()? {
            if files.is_empty() {
                total = row.get::<_, i64>(13)? as usize;
            }
            files.push(Self::map_row(row)?);
        }
        let offset = criteria.offset.unwrap_or(0);
        let has_more = offset.saturating_add(files.len()) < total;
        Ok(FilePage {
            items: files,
            total,
            offset,
            has_more,
        })
    }

    /// List distinct non-empty model names present in indexed metadata.
    pub fn list_distinct_models(&self) -> Result<Vec<String>, DatabaseError> {
        let sql = "SELECT DISTINCT json_extract(metadata, '$.model_name') AS model
                   FROM files
                   WHERE json_extract(metadata, '$.model_name') IS NOT NULL
                     AND json_extract(metadata, '$.model_name') != ''
                   ORDER BY model ASC";
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut models = Vec::new();
        for model in rows {
            models.push(model?);
        }
        Ok(models)
    }

    /// List distinct non-empty sampler names present in indexed metadata.
    pub fn list_distinct_samplers(&self) -> Result<Vec<String>, DatabaseError> {
        let sql = "SELECT DISTINCT json_extract(metadata, '$.sampler') AS sampler
                   FROM files
                   WHERE json_extract(metadata, '$.sampler') IS NOT NULL
                     AND json_extract(metadata, '$.sampler') != ''
                   ORDER BY sampler ASC";
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut samplers = Vec::new();
        for sampler in rows {
            samplers.push(sampler?);
        }
        Ok(samplers)
    }

    /// Files of a folder, ordered by path, with `metadata` deserialized.
    pub fn list_files(&self, folder_id: i64) -> Result<Vec<ImageFile>, DatabaseError> {
        self.query_files(Some(folder_id), FileSortField::Path, SortDirection::Asc)
    }

    /// Update user rating (1–10, or None to clear) for an image file.
    pub fn set_file_rating(&self, file_id: i64, rating: Option<u8>) -> Result<(), DatabaseError> {
        if let Some(r) = rating {
            if !(1..=10).contains(&r) {
                return Err(DatabaseError::InvalidRating(r));
            }
        }
        let affected = self.conn.execute(
            "UPDATE files SET rating = ?1 WHERE id = ?2",
            params![rating.map(|r| r as i64), file_id],
        )?;
        if affected == 0 {
            return Err(DatabaseError::FileNotFound(file_id));
        }
        Ok(())
    }

    /// Update user rating (1–10, or None to clear) for multiple image files in a single transaction.
    pub fn set_files_rating(
        &self,
        file_ids: &[i64],
        rating: Option<u8>,
    ) -> Result<usize, DatabaseError> {
        if let Some(r) = rating {
            if !(1..=10).contains(&r) {
                return Err(DatabaseError::InvalidRating(r));
            }
        }
        if file_ids.is_empty() {
            return Ok(0);
        }

        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached("UPDATE files SET rating = ?1 WHERE id = ?2")?;
            let r_val = rating.map(|r| r as i64);
            for &id in file_ids {
                count += stmt.execute(params![r_val, id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Number of indexed files in a folder.
    pub fn count_files(&self, folder_id: i64) -> Result<i64, DatabaseError> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM files WHERE folder_id = ?1",
            [folder_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Total number of indexed files across all folders.
    pub fn count_all_files(&self) -> Result<i64, DatabaseError> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Counts of indexed files per folder.
    pub fn get_folder_file_counts(&self) -> Result<HashMap<i64, i64>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT folder_id, COUNT(*) FROM files GROUP BY folder_id")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
        let mut counts = HashMap::new();
        for row in rows {
            let (folder_id, count) = row?;
            counts.insert(folder_id, count);
        }
        Ok(counts)
    }

    // --- Albums ---

    /// Create a new album with a unique name.
    pub fn create_album(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<Album, DatabaseError> {
        self.conn.execute(
            "INSERT INTO albums (name, description) VALUES (?1, ?2)",
            params![name.trim(), description.map(|d| d.trim())],
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_album(id)?
            .ok_or_else(|| DatabaseError::AlbumNotFound(id))
    }

    /// Retrieve an album by its ID.
    pub fn get_album(&self, id: i64) -> Result<Option<Album>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id, name, description, created_at FROM albums WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// List all albums ordered by name.
    pub fn list_albums(&self) -> Result<Vec<Album>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, description, created_at FROM albums ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut albums = Vec::new();
        for album in rows {
            albums.push(album?);
        }
        Ok(albums)
    }

    /// Rename an album.
    pub fn rename_album(&self, id: i64, new_name: &str) -> Result<(), DatabaseError> {
        let affected = self.conn.execute(
            "UPDATE albums SET name = ?1 WHERE id = ?2",
            params![new_name.trim(), id],
        )?;
        if affected == 0 {
            return Err(DatabaseError::AlbumNotFound(id));
        }
        Ok(())
    }

    /// Delete an album (associated entries in album_files are automatically deleted via CASCADE).
    pub fn delete_album(&self, id: i64) -> Result<(), DatabaseError> {
        let affected = self
            .conn
            .execute("DELETE FROM albums WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(DatabaseError::AlbumNotFound(id));
        }
        Ok(())
    }

    /// Add a single file to an album.
    pub fn add_file_to_album(&self, album_id: i64, file_id: i64) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO album_files (album_id, file_id) VALUES (?1, ?2)",
            params![album_id, file_id],
        )?;
        Ok(())
    }

    /// Add multiple files to an album in a single transaction.
    pub fn add_files_to_album(
        &self,
        album_id: i64,
        file_ids: &[i64],
    ) -> Result<usize, DatabaseError> {
        if file_ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR IGNORE INTO album_files (album_id, file_id) VALUES (?1, ?2)",
            )?;
            for &fid in file_ids {
                count += stmt.execute(params![album_id, fid])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Remove a file from an album.
    pub fn remove_file_from_album(&self, album_id: i64, file_id: i64) -> Result<(), DatabaseError> {
        self.conn.execute(
            "DELETE FROM album_files WHERE album_id = ?1 AND file_id = ?2",
            params![album_id, file_id],
        )?;
        Ok(())
    }

    /// Remove multiple files from an album in a single transaction.
    pub fn remove_files_from_album(
        &self,
        album_id: i64,
        file_ids: &[i64],
    ) -> Result<usize, DatabaseError> {
        if file_ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt =
                tx.prepare_cached("DELETE FROM album_files WHERE album_id = ?1 AND file_id = ?2")?;
            for &fid in file_ids {
                count += stmt.execute(params![album_id, fid])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Count how many files are in an album.
    pub fn count_album_files(&self, album_id: i64) -> Result<i64, DatabaseError> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM album_files WHERE album_id = ?1",
            [album_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// List all files in an album.
    pub fn list_album_files(&self, album_id: i64) -> Result<Vec<ImageFile>, DatabaseError> {
        let criteria = SearchCriteria {
            album_id: Some(album_id),
            ..Default::default()
        };
        self.search_files(&criteria)
    }

    // --- Tags ---

    /// Create a new tag with an optional color code.
    pub fn create_tag(&self, name: &str, color: Option<&str>) -> Result<Tag, DatabaseError> {
        self.conn.execute(
            "INSERT INTO tags (name, color) VALUES (?1, ?2)",
            params![name.trim(), color.map(|c| c.trim())],
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_tag(id)?
            .ok_or_else(|| DatabaseError::TagNotFound(id))
    }

    /// Retrieve a tag by its ID.
    pub fn get_tag(&self, id: i64) -> Result<Option<Tag>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id, name, color, created_at FROM tags WHERE id = ?1")?;
        let mut rows = stmt.query_map([id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Retrieve a tag by its name (case-insensitive).
    pub fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, color, created_at FROM tags WHERE LOWER(name) = LOWER(?1)",
        )?;
        let mut rows = stmt.query_map([name.trim()], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Retrieve an existing tag by name or create it if not present.
    pub fn get_or_create_tag(&self, name: &str, color: Option<&str>) -> Result<Tag, DatabaseError> {
        let trimmed = name.trim();
        if let Some(existing) = self.get_tag_by_name(trimmed)? {
            return Ok(existing);
        }
        self.create_tag(trimmed, color)
    }

    /// List all tags ordered by name.
    pub fn list_tags(&self) -> Result<Vec<Tag>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id, name, color, created_at FROM tags ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut tags = Vec::new();
        for tag in rows {
            tags.push(tag?);
        }
        Ok(tags)
    }

    /// Delete a tag (associated entries in file_tags are automatically deleted via CASCADE).
    pub fn delete_tag(&self, id: i64) -> Result<(), DatabaseError> {
        let affected = self.conn.execute("DELETE FROM tags WHERE id = ?1", [id])?;
        if affected == 0 {
            return Err(DatabaseError::TagNotFound(id));
        }
        Ok(())
    }

    /// Attach a tag to a file.
    pub fn tag_file(&self, file_id: i64, tag_id: i64) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            params![file_id, tag_id],
        )?;
        Ok(())
    }

    /// Attach a tag to multiple files in a single transaction.
    pub fn tag_files(&self, file_ids: &[i64], tag_id: i64) -> Result<usize, DatabaseError> {
        if file_ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            )?;
            for &fid in file_ids {
                count += stmt.execute(params![fid, tag_id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Remove a tag from a file.
    pub fn untag_file(&self, file_id: i64, tag_id: i64) -> Result<(), DatabaseError> {
        self.conn.execute(
            "DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
            params![file_id, tag_id],
        )?;
        Ok(())
    }

    /// Remove a tag from multiple files in a single transaction.
    pub fn untag_files(&self, file_ids: &[i64], tag_id: i64) -> Result<usize, DatabaseError> {
        if file_ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt =
                tx.prepare_cached("DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2")?;
            for &fid in file_ids {
                count += stmt.execute(params![fid, tag_id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Get all tags attached to a specific file.
    pub fn get_file_tags(&self, file_id: i64) -> Result<Vec<Tag>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM tags t
             JOIN file_tags ft ON t.id = ft.tag_id
             WHERE ft.file_id = ?1
             ORDER BY t.name ASC",
        )?;
        let rows = stmt.query_map([file_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut tags = Vec::new();
        for tag in rows {
            tags.push(tag?);
        }
        Ok(tags)
    }

    /// List all files that have a specific tag.
    pub fn list_files_by_tag(&self, tag_id: i64) -> Result<Vec<ImageFile>, DatabaseError> {
        let criteria = SearchCriteria {
            tag_id: Some(tag_id),
            ..Default::default()
        };
        self.search_files(&criteria)
    }

    // --- Favorites and NSFW ---

    /// Set favorite status for a single file.
    pub fn set_file_favorite(&self, file_id: i64, is_favorite: bool) -> Result<(), DatabaseError> {
        let affected = self.conn.execute(
            "UPDATE files SET is_favorite = ?1 WHERE id = ?2",
            params![is_favorite as i64, file_id],
        )?;
        if affected == 0 {
            return Err(DatabaseError::FileNotFound(file_id));
        }
        Ok(())
    }

    /// Set favorite status for multiple files in a single transaction.
    pub fn set_files_favorite(
        &self,
        file_ids: &[i64],
        is_favorite: bool,
    ) -> Result<usize, DatabaseError> {
        if file_ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached("UPDATE files SET is_favorite = ?1 WHERE id = ?2")?;
            let val = is_favorite as i64;
            for &id in file_ids {
                count += stmt.execute(params![val, id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Set NSFW status for a single file.
    pub fn set_file_nsfw(&self, file_id: i64, is_nsfw: bool) -> Result<(), DatabaseError> {
        let affected = self.conn.execute(
            "UPDATE files SET is_nsfw = ?1 WHERE id = ?2",
            params![is_nsfw as i64, file_id],
        )?;
        if affected == 0 {
            return Err(DatabaseError::FileNotFound(file_id));
        }
        Ok(())
    }

    /// Set NSFW status for multiple files in a single transaction.
    pub fn set_files_nsfw(&self, file_ids: &[i64], is_nsfw: bool) -> Result<usize, DatabaseError> {
        if file_ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached("UPDATE files SET is_nsfw = ?1 WHERE id = ?2")?;
            let val = is_nsfw as i64;
            for &id in file_ids {
                count += stmt.execute(params![val, id])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    // --- Prompt Statistics ---

    /// Extract and rank prompt tags by occurrence across all indexed files.
    pub fn get_prompt_stats(
        &self,
        is_negative: bool,
        limit: usize,
    ) -> Result<Vec<PromptStat>, DatabaseError> {
        let field = if is_negative {
            "$.negative_prompt"
        } else {
            "$.prompt"
        };
        let sql = format!(
            "SELECT json_extract(metadata, '{field}') AS p
             FROM files
             WHERE json_extract(metadata, '{field}') IS NOT NULL
               AND json_extract(metadata, '{field}') != ''"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut frequencies: HashMap<String, usize> = HashMap::new();
        for p in rows {
            let prompt = p?;
            for part in prompt.split(',') {
                let tag = part.trim().trim_matches('"').trim();
                if !tag.is_empty() && tag.len() > 1 {
                    *frequencies.entry(tag.to_string()).or_insert(0) += 1;
                }
            }
        }

        let mut stats: Vec<PromptStat> = frequencies
            .into_iter()
            .map(|(text, count)| PromptStat { text, count })
            .collect();

        stats.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.text.cmp(&b.text)));
        if stats.len() > limit {
            stats.truncate(limit);
        }
        Ok(stats)
    }

    // --- Checkpoints and Model Cache ---

    /// Retrieve all distinct checkpoint models and their hashes, counts, sorted by usage descending.
    pub fn get_checkpoint_models(&self) -> Result<Vec<CheckpointModelStat>, DatabaseError> {
        let sql = "
            SELECT
                json_extract(metadata, '$.model_name') AS m_name,
                json_extract(metadata, '$.model_hash') AS m_hash,
                COUNT(*) AS cnt
            FROM files
            WHERE json_extract(metadata, '$.model_name') IS NOT NULL
              AND json_extract(metadata, '$.model_name') != ''
            GROUP BY m_name, m_hash
            ORDER BY cnt DESC, m_name ASC
        ";
        let mut stmt = self.conn.prepare_cached(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(CheckpointModelStat {
                model_name: row.get(0)?,
                model_hash: row.get(1)?,
                count: row.get::<_, i64>(2)? as usize,
            })
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// Import entries from an A1111 cache.json or model directory into the `model_cache` table.
    pub fn import_model_cache(&self, entries: &[ModelCacheEntry]) -> Result<usize, DatabaseError> {
        if entries.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR REPLACE INTO model_cache (hash, name, title, sha256) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for entry in entries {
                count += stmt.execute(params![
                    entry.hash.trim(),
                    entry.name.trim(),
                    entry.title.as_deref().map(|t| t.trim()),
                    entry.sha256.as_deref().map(|s| s.trim()),
                ])?;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Lookup a model entry in `model_cache` by short hash or sha256 or name.
    pub fn resolve_model_hash(
        &self,
        hash_or_name: &str,
    ) -> Result<Option<ModelCacheEntry>, DatabaseError> {
        let clean = hash_or_name.trim();
        if clean.is_empty() {
            return Ok(None);
        }
        let mut stmt = self.conn.prepare_cached(
            "SELECT hash, name, title, sha256 FROM model_cache
             WHERE hash = ?1
                OR name = ?1
                OR sha256 = ?1
                OR sha256 LIKE (?1 || '%')
                OR hash LIKE (?1 || '%')
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map([clean], |row| {
            Ok(ModelCacheEntry {
                hash: row.get(0)?,
                name: row.get(1)?,
                title: row.get(2)?,
                sha256: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// List all cached model mappings.
    pub fn list_model_cache(&self) -> Result<Vec<ModelCacheEntry>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT hash, name, title, sha256 FROM model_cache ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ModelCacheEntry {
                hash: row.get(0)?,
                name: row.get(1)?,
                title: row.get(2)?,
                sha256: row.get(3)?,
            })
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// List all saved LoRAs ordered alphabetically by name.
    pub fn list_loras(&self) -> Result<Vec<LoraModel>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, hash, trigger_words, preview_url, description, weight_default, created_at, updated_at
             FROM loras
             ORDER BY name COLLATE NOCASE ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let triggers_json: String = row.get(3)?;
            let trigger_words: Vec<String> =
                serde_json::from_str(&triggers_json).unwrap_or_default();
            Ok(LoraModel {
                id: row.get(0)?,
                name: row.get(1)?,
                hash: row.get(2)?,
                trigger_words,
                preview_url: row.get(4)?,
                description: row.get(5)?,
                weight_default: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }

    /// Retrieve a single LoRA by its ID.
    pub fn get_lora(&self, id: i64) -> Result<Option<LoraModel>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, hash, trigger_words, preview_url, description, weight_default, created_at, updated_at
             FROM loras
             WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map([id], |row| {
            let triggers_json: String = row.get(3)?;
            let trigger_words: Vec<String> =
                serde_json::from_str(&triggers_json).unwrap_or_default();
            Ok(LoraModel {
                id: row.get(0)?,
                name: row.get(1)?,
                hash: row.get(2)?,
                trigger_words,
                preview_url: row.get(4)?,
                description: row.get(5)?,
                weight_default: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// Lookup a LoRA by model name or hash (case-insensitive prefix / exact match).
    pub fn find_lora_by_name_or_hash(
        &self,
        query: &str,
    ) -> Result<Option<LoraModel>, DatabaseError> {
        let clean = query.trim();
        if clean.is_empty() {
            return Ok(None);
        }
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, name, hash, trigger_words, preview_url, description, weight_default, created_at, updated_at
             FROM loras
             WHERE name = ?1 COLLATE NOCASE
                OR hash = ?1 COLLATE NOCASE
                OR hash LIKE (?1 || '%')
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map([clean], |row| {
            let triggers_json: String = row.get(3)?;
            let trigger_words: Vec<String> =
                serde_json::from_str(&triggers_json).unwrap_or_default();
            Ok(LoraModel {
                id: row.get(0)?,
                name: row.get(1)?,
                hash: row.get(2)?,
                trigger_words,
                preview_url: row.get(4)?,
                description: row.get(5)?,
                weight_default: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    /// Save (insert or update) a LoRA model in the catalog.
    pub fn save_lora(&self, lora: &LoraModel) -> Result<LoraModel, DatabaseError> {
        let clean_name = lora.name.trim();
        let triggers_json = serde_json::to_string(&lora.trigger_words)?;

        if lora.id > 0 {
            let mut stmt = self.conn.prepare_cached(
                "UPDATE loras
                 SET name = ?1, hash = ?2, trigger_words = ?3, preview_url = ?4, description = ?5, weight_default = ?6, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
                 WHERE id = ?7",
            )?;
            let rows_affected = stmt.execute(params![
                clean_name,
                lora.hash.as_deref().map(|h| h.trim()),
                triggers_json,
                lora.preview_url.as_deref().map(|p| p.trim()),
                lora.description.as_deref().map(|d| d.trim()),
                lora.weight_default,
                lora.id,
            ])?;
            if rows_affected > 0 {
                return self
                    .get_lora(lora.id)?
                    .ok_or(DatabaseError::LoraNotFound(lora.id));
            }
        }

        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO loras (name, hash, trigger_words, preview_url, description, weight_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT(name) DO UPDATE SET
                hash = COALESCE(excluded.hash, loras.hash),
                trigger_words = excluded.trigger_words,
                preview_url = COALESCE(excluded.preview_url, loras.preview_url),
                description = COALESCE(excluded.description, loras.description),
                weight_default = excluded.weight_default,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
        )?;
        stmt.execute(params![
            clean_name,
            lora.hash.as_deref().map(|h| h.trim()),
            triggers_json,
            lora.preview_url.as_deref().map(|p| p.trim()),
            lora.description.as_deref().map(|d| d.trim()),
            lora.weight_default,
        ])?;

        let id = self.conn.last_insert_rowid();
        if let Some(created) = self.get_lora(id)? {
            Ok(created)
        } else {
            self.find_lora_by_name_or_hash(clean_name)?
                .ok_or(DatabaseError::LoraNotFound(id))
        }
    }

    /// Delete a LoRA by ID.
    pub fn delete_lora(&self, id: i64) -> Result<bool, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM loras WHERE id = ?1")?;
        let affected = stmt.execute([id])?;
        Ok(affected > 0)
    }

    /// Import multiple LoRAs into catalog.
    pub fn import_loras(&self, loras: &[LoraModel]) -> Result<usize, DatabaseError> {
        let mut count = 0;
        for l in loras {
            self.save_lora(l)?;
            count += 1;
        }
        Ok(count)
    }

    /// Returns `(path, size_bytes, modified_at, has_metadata)` tuples for all
    /// files in a folder — lightweight version of `list_files` that avoids
    /// deserializing the JSON `metadata` column.
    pub fn list_file_fingerprints(
        &self,
        folder_id: i64,
    ) -> Result<Vec<(String, u64, i64, bool)>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT path, size_bytes, modified_at, (metadata IS NOT NULL)
             FROM files WHERE folder_id = ?1",
        )?;
        let rows = stmt
            .query_map(params![folder_id], |row| {
                let size: i64 = row.get(1)?;
                let has_meta: i64 = row.get(3)?;
                Ok((
                    row.get::<_, String>(0)?,
                    size as u64,
                    row.get::<_, i64>(2)?,
                    has_meta != 0,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // --- File Embeddings and Similarity Search ---

    /// Insert or update an embedding vector for a file and model.
    ///
    /// The embedding vector is serialized as little-endian f32 bytes. If a record
    /// exists for `(file_id, model_id)`, its dimensions and embedding are updated,
    /// and `updated_at` is refreshed.
    pub fn upsert_file_embedding(
        &self,
        file_id: i64,
        model_id: &str,
        embedding: &[f32],
    ) -> Result<(), DatabaseError> {
        if model_id.trim().is_empty() {
            return Err(DatabaseError::BlankModelId);
        }
        if embedding.is_empty() {
            return Err(DatabaseError::EmptyVector);
        }
        for &val in embedding {
            if !val.is_finite() {
                return Err(DatabaseError::NonFiniteVectorValue);
            }
        }
        let mut sum_sq: f64 = 0.0;
        for &val in embedding {
            let vf = val as f64;
            sum_sq += vf * vf;
        }
        let norm = sum_sq.sqrt();
        if norm <= 0.0 || !norm.is_finite() {
            return Err(DatabaseError::ZeroVectorNorm);
        }

        let dim_i64 =
            i64::try_from(embedding.len()).map_err(|_| DatabaseError::InvalidStoredDimensions {
                file_id,
                dimensions: -1,
            })?;
        let expected_bytes = embedding
            .len()
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or(DatabaseError::InvalidStoredDimensions {
                file_id,
                dimensions: dim_i64,
            })?;

        let mut blob = Vec::with_capacity(expected_bytes);
        for &val in embedding {
            blob.extend_from_slice(&val.to_le_bytes());
        }

        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO file_embeddings (file_id, model_id, dimensions, embedding, updated_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'))
             ON CONFLICT(file_id, model_id) DO UPDATE SET
                 dimensions = excluded.dimensions,
                 embedding  = excluded.embedding,
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')",
        )?;
        stmt.execute(params![file_id, model_id, dim_i64, blob])?;
        Ok(())
    }

    /// Remove an embedding record for a file and model.
    ///
    /// Returns `true` if a record was removed, or `false` if none existed.
    pub fn remove_file_embedding(
        &self,
        file_id: i64,
        model_id: &str,
    ) -> Result<bool, DatabaseError> {
        if model_id.trim().is_empty() {
            return Err(DatabaseError::BlankModelId);
        }
        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM file_embeddings WHERE file_id = ?1 AND model_id = ?2")?;
        let affected = stmt.execute(params![file_id, model_id])?;
        Ok(affected > 0)
    }

    /// Search for files similar to `query` vector using cosine similarity.
    ///
    /// Only embeddings with the exact `model_id` are compared.
    /// Similarity scores are computed with f64 accumulation and clamped to `[-1.0, 1.0]`.
    /// Results are sorted descending by score, with ties broken deterministically by
    /// ascending `file_id`, and truncated to `limit`.
    pub fn search_similar_files(
        &self,
        model_id: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<SimilarityMatch>, DatabaseError> {
        if model_id.trim().is_empty() {
            return Err(DatabaseError::BlankModelId);
        }
        if limit == 0 {
            return Err(DatabaseError::ZeroResultLimit);
        }
        if query.is_empty() {
            return Err(DatabaseError::EmptyVector);
        }
        for &val in query {
            if !val.is_finite() {
                return Err(DatabaseError::NonFiniteVectorValue);
            }
        }
        let mut query_sum_sq: f64 = 0.0;
        for &val in query {
            let qf = val as f64;
            query_sum_sq += qf * qf;
        }
        let query_norm = query_sum_sq.sqrt();
        if query_norm <= 0.0 || !query_norm.is_finite() {
            return Err(DatabaseError::ZeroVectorNorm);
        }

        let mut stmt = self.conn.prepare_cached(
            "SELECT file_id, dimensions, embedding FROM file_embeddings WHERE model_id = ?1",
        )?;
        let mut rows = stmt.query([model_id])?;
        let mut matches = Vec::new();

        while let Some(row) = rows.next()? {
            let file_id: i64 = row.get(0)?;
            let stored_dim_i64: i64 = row.get(1)?;
            let blob: Vec<u8> = row.get(2)?;

            if stored_dim_i64 <= 0 {
                return Err(DatabaseError::InvalidStoredDimensions {
                    file_id,
                    dimensions: stored_dim_i64,
                });
            }

            let stored_dim = usize::try_from(stored_dim_i64).map_err(|_| {
                DatabaseError::InvalidStoredDimensions {
                    file_id,
                    dimensions: stored_dim_i64,
                }
            })?;

            let expected_bytes = stored_dim.checked_mul(std::mem::size_of::<f32>()).ok_or(
                DatabaseError::InvalidStoredDimensions {
                    file_id,
                    dimensions: stored_dim_i64,
                },
            )?;

            if blob.len() != expected_bytes {
                return Err(DatabaseError::MalformedStoredBlob {
                    file_id,
                    dimensions: stored_dim,
                    expected_bytes,
                    actual_bytes: blob.len(),
                });
            }

            if stored_dim != query.len() {
                return Err(DatabaseError::DimensionMismatch {
                    file_id,
                    query: query.len(),
                    stored: stored_dim,
                });
            }

            let mut dot_product: f64 = 0.0;
            let mut stored_sum_sq: f64 = 0.0;

            for (i, chunk) in blob.chunks_exact(4).enumerate() {
                let bytes: [u8; 4] =
                    chunk
                        .try_into()
                        .map_err(|_| DatabaseError::MalformedStoredBlob {
                            file_id,
                            dimensions: stored_dim,
                            expected_bytes,
                            actual_bytes: blob.len(),
                        })?;
                let val = f32::from_le_bytes(bytes);
                if !val.is_finite() {
                    return Err(DatabaseError::CorruptStoredVectorValue { file_id });
                }
                let s_f64 = val as f64;
                let q_f64 = query[i] as f64;
                dot_product += q_f64 * s_f64;
                stored_sum_sq += s_f64 * s_f64;
            }

            let stored_norm = stored_sum_sq.sqrt();
            if stored_norm <= 0.0 || !stored_norm.is_finite() {
                return Err(DatabaseError::CorruptStoredZeroNorm { file_id });
            }

            let denom = query_norm * stored_norm;
            let cosine_sim = if denom <= 0.0 || !denom.is_finite() {
                0.0
            } else {
                (dot_product / denom).clamp(-1.0, 1.0)
            };

            matches.push(SimilarityMatch {
                file_id,
                score: cosine_sim as f32,
            });
        }

        matches.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.file_id.cmp(&b.file_id))
        });
        matches.truncate(limit);

        Ok(matches)
    }

    /// Retrieve the stored embedding vector for a given file and model.
    pub fn get_file_embedding(
        &self,
        file_id: i64,
        model_id: &str,
    ) -> Result<Option<Vec<f32>>, DatabaseError> {
        if model_id.trim().is_empty() {
            return Err(DatabaseError::BlankModelId);
        }
        let mut stmt = self.conn.prepare_cached(
            "SELECT dimensions, embedding FROM file_embeddings WHERE file_id = ?1 AND model_id = ?2 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![file_id, model_id])?;
        if let Some(row) = rows.next()? {
            let stored_dim_i64: i64 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;

            if stored_dim_i64 <= 0 {
                return Err(DatabaseError::InvalidStoredDimensions {
                    file_id,
                    dimensions: stored_dim_i64,
                });
            }

            let stored_dim = usize::try_from(stored_dim_i64).map_err(|_| {
                DatabaseError::InvalidStoredDimensions {
                    file_id,
                    dimensions: stored_dim_i64,
                }
            })?;

            let expected_bytes = stored_dim.checked_mul(std::mem::size_of::<f32>()).ok_or(
                DatabaseError::InvalidStoredDimensions {
                    file_id,
                    dimensions: stored_dim_i64,
                },
            )?;

            if blob.len() != expected_bytes {
                return Err(DatabaseError::MalformedStoredBlob {
                    file_id,
                    dimensions: stored_dim,
                    expected_bytes,
                    actual_bytes: blob.len(),
                });
            }

            let mut vec = Vec::with_capacity(stored_dim);
            for chunk in blob.chunks_exact(4) {
                let bytes: [u8; 4] =
                    chunk
                        .try_into()
                        .map_err(|_| DatabaseError::MalformedStoredBlob {
                            file_id,
                            dimensions: stored_dim,
                            expected_bytes,
                            actual_bytes: blob.len(),
                        })?;
                let val = f32::from_le_bytes(bytes);
                if !val.is_finite() {
                    return Err(DatabaseError::CorruptStoredVectorValue { file_id });
                }
                vec.push(val);
            }
            Ok(Some(vec))
        } else {
            Ok(None)
        }
    }

    /// List all model IDs for which embeddings exist for a given file.
    pub fn get_file_embedding_models(&self, file_id: i64) -> Result<Vec<String>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT model_id FROM file_embeddings WHERE file_id = ?1 ORDER BY model_id ASC",
        )?;
        let rows = stmt.query_map([file_id], |r| r.get(0))?;
        let mut models = Vec::new();
        for m in rows {
            models.push(m?);
        }
        Ok(models)
    }

    /// Find files visually similar to an existing indexed file.
    ///
    /// If `model_id` is provided, that model's embedding is used. If `None`, the first
    /// available model embedding associated with `file_id` is used.
    /// Returns up to `limit` matches, excluding the source file itself.
    pub fn find_similar_to_file(
        &self,
        file_id: i64,
        model_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SimilarityMatch>, DatabaseError> {
        if limit == 0 {
            return Err(DatabaseError::ZeroResultLimit);
        }

        let model = match model_id {
            Some(m) => {
                if m.trim().is_empty() {
                    return Err(DatabaseError::BlankModelId);
                }
                m.to_string()
            }
            None => {
                let models = self.get_file_embedding_models(file_id)?;
                if models.is_empty() {
                    return Ok(vec![]);
                }
                models[0].clone()
            }
        };

        let embedding = match self.get_file_embedding(file_id, &model)? {
            Some(vec) => vec,
            None => return Ok(vec![]),
        };

        // Query with limit + 1 so we can filter out the source file if returned
        let search_limit = limit.saturating_add(1);
        let mut matches = self.search_similar_files(&model, &embedding, search_limit)?;
        matches.retain(|m| m.file_id != file_id);
        matches.truncate(limit);

        Ok(matches)
    }

    /// Returns the embedding index statistics for a given model:
    /// `(indexed_count, total_count)` where `total_count` is the total number of images in `files`.
    pub fn get_embedding_index_stats(
        &self,
        model_id: &str,
    ) -> Result<(usize, usize), DatabaseError> {
        if model_id.trim().is_empty() {
            return Err(DatabaseError::BlankModelId);
        }

        let total_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;

        let indexed_count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM file_embeddings WHERE model_id = ?1",
            [model_id],
            |row| row.get(0),
        )?;

        Ok((indexed_count as usize, total_count as usize))
    }

    /// Returns files that have not yet been indexed by `model_id`, up to `limit`.
    pub fn get_unindexed_files(
        &self,
        model_id: &str,
        limit: usize,
    ) -> Result<Vec<ImageFile>, DatabaseError> {
        if model_id.trim().is_empty() {
            return Err(DatabaseError::BlankModelId);
        }
        if limit == 0 {
            return Err(DatabaseError::ZeroResultLimit);
        }

        let mut stmt = self.conn.prepare_cached(
            "SELECT id, folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order
             FROM files
             WHERE id NOT IN (SELECT file_id FROM file_embeddings WHERE model_id = ?1)
             ORDER BY id ASC
             LIMIT ?2",
        )?;

        let rows = stmt.query_and_then(rusqlite::params![model_id, limit as i64], Self::map_row)?;

        rows.collect()
    }

    // --- Image Stacking ---

    /// Group a list of image IDs into a stack with the given or generated stack_id.
    /// The first image in `file_ids` becomes the hero (stack_order = 0), subsequent images have stack_order = 1, 2, ...
    pub fn stack_images(
        &self,
        file_ids: &[i64],
        custom_stack_id: Option<&str>,
    ) -> Result<String, DatabaseError> {
        if file_ids.is_empty() {
            return Ok(String::new());
        }

        let sid = custom_stack_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx
                .prepare_cached("UPDATE files SET stack_id = ?1, stack_order = ?2 WHERE id = ?3")?;
            for (idx, file_id) in file_ids.iter().enumerate() {
                stmt.execute(params![sid, idx as i64, file_id])?;
            }
        }
        tx.commit()?;
        Ok(sid)
    }

    /// Flatten complete source stacks and standalone images into `target_stack_id`.
    ///
    /// Existing target members retain their order and hero. Complete source stacks
    /// follow in the caller-provided order, then standalone images. Every image is
    /// deduplicated and reassigned in one transaction, so a source stack can never be
    /// left partially populated.
    pub fn merge_stacks(
        &self,
        target_stack_id: &str,
        source_stack_ids: &[String],
        standalone_file_ids: &[i64],
    ) -> Result<usize, DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        let mut member_ids = Vec::new();
        let mut seen_file_ids = HashSet::new();

        {
            let mut append_stack_members = |stack_id: &str| -> Result<(), DatabaseError> {
                let mut stmt = tx.prepare(
                    "SELECT id FROM files WHERE stack_id = ?1 ORDER BY stack_order ASC, id ASC",
                )?;
                let ids = stmt
                    .query_map([stack_id], |row| row.get::<_, i64>(0))?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                if ids.is_empty() {
                    return Err(DatabaseError::StackNotFound(stack_id.to_string()));
                }
                for id in ids {
                    if seen_file_ids.insert(id) {
                        member_ids.push(id);
                    }
                }
                Ok(())
            };

            append_stack_members(target_stack_id)?;
            let mut seen_stack_ids = HashSet::from([target_stack_id.to_string()]);
            for stack_id in source_stack_ids {
                if seen_stack_ids.insert(stack_id.clone()) {
                    append_stack_members(stack_id)?;
                }
            }
        }

        for file_id in standalone_file_ids {
            if seen_file_ids.contains(file_id) {
                continue;
            }
            let mut stmt = tx.prepare("SELECT stack_id FROM files WHERE id = ?1")?;
            let mut rows = stmt.query([file_id])?;
            let Some(row) = rows.next()? else {
                return Err(DatabaseError::FileNotFound(*file_id));
            };
            if let Some(stack_id) = row.get::<_, Option<String>>(0)? {
                return Err(DatabaseError::FileAlreadyStacked {
                    file_id: *file_id,
                    stack_id,
                });
            }
            seen_file_ids.insert(*file_id);
            member_ids.push(*file_id);
        }

        {
            let mut update = tx
                .prepare_cached("UPDATE files SET stack_id = ?1, stack_order = ?2 WHERE id = ?3")?;
            for (order, file_id) in member_ids.iter().enumerate() {
                update.execute(params![target_stack_id, order as i64, file_id])?;
            }
        }
        tx.commit()?;
        Ok(member_ids.len())
    }

    /// Dissolve a stack by clearing stack_id and resetting stack_order for all its member images.
    pub fn unstack_images(&self, stack_id: &str) -> Result<u64, DatabaseError> {
        let affected = self.conn.execute(
            "UPDATE files SET stack_id = NULL, stack_order = 0 WHERE stack_id = ?1",
            [stack_id],
        )?;
        Ok(affected as u64)
    }

    /// Set a specific image in a stack as the Hero Cover (stack_order = 0), shifting others.
    pub fn set_stack_hero(&self, stack_id: &str, hero_file_id: i64) -> Result<(), DatabaseError> {
        let tx = self.conn.unchecked_transaction()?;
        {
            // Shift all existing images in the stack
            tx.execute(
                "UPDATE files SET stack_order = stack_order + 1 WHERE stack_id = ?1",
                [stack_id],
            )?;
            // Set the target hero image to stack_order = 0
            tx.execute(
                "UPDATE files SET stack_order = 0 WHERE id = ?1 AND stack_id = ?2",
                params![hero_file_id, stack_id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Get all images belonging to a specific stack, ordered by stack_order ASC, id ASC.
    pub fn get_stack_members(&self, stack_id: &str) -> Result<Vec<ImageFile>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, folder_id, path, container, size_bytes, modified_at, metadata, rating, aesthetic_score, is_favorite, is_nsfw, stack_id, stack_order
             FROM files WHERE stack_id = ?1 ORDER BY stack_order ASC, id ASC",
        )?;
        let rows = stmt.query_and_then([stack_id], Self::map_row)?;
        rows.collect()
    }

    /// List summary stats for all active stacks, optionally filtered by folder_id.
    pub fn list_stacks(&self, folder_id: Option<i64>) -> Result<Vec<StackSummary>, DatabaseError> {
        let (sql, params_vec): (&str, Vec<rusqlite::types::Value>) = match folder_id {
            Some(fid) => (
                "SELECT stack_id, COUNT(*) as count,
                        (SELECT id FROM files f2 WHERE f2.stack_id = f1.stack_id ORDER BY f2.stack_order ASC, f2.id ASC LIMIT 1) as hero_id
                 FROM files f1
                 WHERE stack_id IS NOT NULL AND folder_id = ?1
                 GROUP BY stack_id
                 ORDER BY count DESC",
                vec![rusqlite::types::Value::Integer(fid)],
            ),
            None => (
                "SELECT stack_id, COUNT(*) as count,
                        (SELECT id FROM files f2 WHERE f2.stack_id = f1.stack_id ORDER BY f2.stack_order ASC, f2.id ASC LIMIT 1) as hero_id
                 FROM files f1
                 WHERE stack_id IS NOT NULL
                 GROUP BY stack_id
                 ORDER BY count DESC",
                vec![],
            ),
        };

        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(&params_vec), |row| {
            Ok(StackSummary {
                stack_id: row.get(0)?,
                count: row.get::<_, i64>(1)? as usize,
                hero_image_id: row.get(2)?,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    /// Get non-hero images in a stack whose rating is below `min_rating`, for batch culling.
    pub fn get_stack_cull_candidate_ids(
        &self,
        stack_id: &str,
        min_rating: u8,
    ) -> Result<Vec<i64>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id FROM files
             WHERE stack_id = ?1 AND stack_order > 0 AND (rating IS NULL OR rating < ?2)",
        )?;
        let rows = stmt.query_map(params![stack_id, min_rating as i64], |row| row.get(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    // --- Pipeline Cleanup Queue ---

    /// Enqueue a source file for delayed cleanup.
    pub fn enqueue_cleanup(
        &self,
        source_path: &str,
        target_file_id: i64,
        grace_period_hours: i32,
    ) -> Result<i64, DatabaseError> {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let scheduled_delete_at = created_at + (grace_period_hours.max(0) as i64 * 3600);

        self.conn.execute(
            "INSERT INTO pipeline_cleanup_queue (source_file_path, target_file_id, scheduled_delete_at, created_at, status)
             VALUES (?1, ?2, ?3, ?4, 'pending')
             ON CONFLICT(source_file_path) DO UPDATE SET
                 scheduled_delete_at = excluded.scheduled_delete_at,
                 status = 'pending'",
            params![source_path, target_file_id, scheduled_delete_at, created_at],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// List pending cleanup items that are due for deletion.
    pub fn list_due_cleanups(
        &self,
        now_timestamp: i64,
    ) -> Result<Vec<CleanupQueueItem>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, source_file_path, target_file_id, scheduled_delete_at, created_at, status
             FROM pipeline_cleanup_queue
             WHERE status = 'pending' AND scheduled_delete_at <= ?1
             ORDER BY scheduled_delete_at ASC",
        )?;
        let rows = stmt.query_map([now_timestamp], |row| {
            Ok(CleanupQueueItem {
                id: row.get(0)?,
                source_file_path: row.get(1)?,
                target_image_id: row.get(2)?,
                scheduled_delete_at: row.get(3)?,
                created_at: row.get(4)?,
                status: row.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    /// Update status of a cleanup queue item.
    pub fn update_cleanup_status(&self, id: i64, status: &str) -> Result<(), DatabaseError> {
        self.conn.execute(
            "UPDATE pipeline_cleanup_queue SET status = ?1 WHERE id = ?2",
            params![status, id],
        )?;
        Ok(())
    }

    /// Get current cleanup queue list with limit.
    pub fn get_cleanup_queue(&self, limit: usize) -> Result<Vec<CleanupQueueItem>, DatabaseError> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, source_file_path, target_file_id, scheduled_delete_at, created_at, status
             FROM pipeline_cleanup_queue
             ORDER BY id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            Ok(CleanupQueueItem {
                id: row.get(0)?,
                source_file_path: row.get(1)?,
                target_image_id: row.get(2)?,
                scheduled_delete_at: row.get(3)?,
                created_at: row.get(4)?,
                status: row.get(5)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let _ = self.conn.execute("PRAGMA optimize;", []);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_database_migrates_to_latest() {
        let db = Database::connect_in_memory().unwrap();
        assert_eq!(db.user_version().unwrap(), LATEST_VERSION);
        assert!(db.path().is_none());
    }

    #[test]
    fn migrations_are_idempotent() {
        // Re-running the runner on an already-migrated DB must be a no-op.
        let mut db = Database::connect_in_memory().unwrap();
        db.migrate().unwrap();
        assert_eq!(db.user_version().unwrap(), LATEST_VERSION);
    }

    #[test]
    fn migration_created_meta_table() {
        let db = Database::connect_in_memory().unwrap();
        assert_eq!(db.meta_get("anything").unwrap(), None);
    }

    #[test]
    fn file_database_persists_and_tracks_path() {
        let dir = std::env::temp_dir().join(format!("berry-storage-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.db");

        {
            let db = Database::connect(&path).unwrap();
            assert_eq!(db.path(), Some(path.as_path()));
            assert_eq!(db.user_version().unwrap(), LATEST_VERSION);
        }

        // Reopening the same file sees the persisted schema.
        let db = Database::connect(&path).unwrap();
        assert_eq!(db.user_version().unwrap(), LATEST_VERSION);

        // Drop the connection so SQLite releases its file locks before the
        // directory is removed (required on Windows).
        drop(db);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // --- Folder repository ---

    fn image(folder_id: i64, path: &str) -> ImageFile {
        ImageFile {
            id: None,
            folder_id,
            path: path.to_string(),
            size_bytes: 1,
            modified_at: 1,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        }
    }

    #[test]
    fn folder_crud_roundtrip() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/tmp/img").unwrap();
        assert_eq!(folder.path, "/tmp/img");
        assert!(folder.id > 0);
        assert!(!folder.added_at.is_empty());

        assert_eq!(db.list_folders().unwrap(), vec![folder.clone()]);
        assert_eq!(
            db.find_folder_by_path("/tmp/img").unwrap(),
            Some(folder.clone())
        );
        assert_eq!(db.find_folder_by_path("/nope").unwrap(), None);

        db.remove_folder(folder.id).unwrap();
        assert!(db.list_folders().unwrap().is_empty());
    }

    #[test]
    fn add_folder_rejects_duplicate_path() {
        let db = Database::connect_in_memory().unwrap();
        db.add_folder("/dup").unwrap();
        let err = db.add_folder("/dup").unwrap_err();
        assert!(matches!(err, DatabaseError::Sqlite(_)));
    }

    #[test]
    fn remove_missing_folder_errors() {
        let db = Database::connect_in_memory().unwrap();
        assert!(matches!(
            db.remove_folder(42),
            Err(DatabaseError::FolderNotFound(42))
        ));
    }

    // --- File repository ---

    #[test]
    fn file_upsert_and_list() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();

        let id_a = db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();
        let id_b = db.upsert_file(&image(folder.id, "/img/b.png")).unwrap();
        assert_ne!(id_a, id_b);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "/img/a.png");
        assert_eq!(files[0].id, Some(id_a));
        assert_eq!(files[0].container, Container::Png);
        assert_eq!(files[1].path, "/img/b.png");

        assert_eq!(db.count_files(folder.id).unwrap(), 2);
    }

    #[test]
    fn file_upsert_is_idempotent_by_path() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();

        let mut file = image(folder.id, "/img/a.png");
        file.size_bytes = 100;
        let first = db.upsert_file(&file).unwrap();

        file.size_bytes = 200;
        let second = db.upsert_file(&file).unwrap();
        assert_eq!(first, second, "re-upserting the same path keeps one row");

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].size_bytes, 200);
    }

    #[test]
    fn metadata_roundtrips_through_json_column() {
        use berry_domain::MetadataFormat;

        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let meta = ExtractedMetadata {
            format: MetadataFormat::A1111,
            parameters: Some("a masterpiece, Steps: 20, Seed: 123".to_string()),
            raw: None,
            prompt: Some("a masterpiece".to_string()),
            negative_prompt: Some("blurry".to_string()),
            width: Some(512),
            height: Some(768),
            seed: Some("123".to_string()),
            steps: Some(20),
            cfg_scale: Some(7.0),
            sampler: Some("DPM++ 2M Karras".to_string()),
            model_name: Some("dreamshaper".to_string()),
            model_hash: Some("abc123".to_string()),
        };

        let mut file = image(folder.id, "/img/a.png");
        file.metadata = Some(meta.clone());
        db.upsert_file(&file).unwrap();

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].metadata, Some(meta));
    }

    #[test]
    fn remove_folder_cascades_to_files() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();

        db.remove_folder(folder.id).unwrap();
        assert!(db.list_files(folder.id).unwrap().is_empty());
    }

    #[test]
    fn delete_files_not_in_removes_orphans() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();
        db.upsert_file(&image(folder.id, "/img/b.png")).unwrap();

        let removed = db
            .delete_files_not_in(folder.id, &["/img/a.png".to_string()])
            .unwrap();
        assert_eq!(removed, 1);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "/img/a.png");
    }

    #[test]
    fn query_files_sorts_and_filters() {
        let db = Database::connect_in_memory().unwrap();
        let f1 = db.add_folder("/folder1").unwrap();
        let f2 = db.add_folder("/folder2").unwrap();

        let mut f1_a = image(f1.id, "/folder1/a.png");
        f1_a.modified_at = 100;
        f1_a.size_bytes = 500;
        f1_a.rating = Some(8);
        f1_a.aesthetic_score = Some(7.5);

        let mut f1_b = image(f1.id, "/folder1/b.png");
        f1_b.modified_at = 200;
        f1_b.size_bytes = 300;
        f1_b.rating = Some(9);
        f1_b.aesthetic_score = None;

        let mut f2_c = image(f2.id, "/folder2/c.png");
        f2_c.modified_at = 150;
        f2_c.size_bytes = 800;
        f2_c.rating = None;
        f2_c.aesthetic_score = Some(8.2);

        db.upsert_files(&[f1_a, f1_b, f2_c]).unwrap();

        // 1. Filter by folder
        let folder1_files = db
            .query_files(Some(f1.id), FileSortField::Path, SortDirection::Asc)
            .unwrap();
        assert_eq!(folder1_files.len(), 2);
        assert_eq!(folder1_files[0].path, "/folder1/a.png");
        assert_eq!(folder1_files[1].path, "/folder1/b.png");

        // 2. All folders (folder_id: None), sorted by modified_at DESC
        let all_by_date_desc = db
            .query_files(None, FileSortField::ModifiedAt, SortDirection::Desc)
            .unwrap();
        assert_eq!(all_by_date_desc.len(), 3);
        assert_eq!(all_by_date_desc[0].path, "/folder1/b.png"); // 200
        assert_eq!(all_by_date_desc[1].path, "/folder2/c.png"); // 150
        assert_eq!(all_by_date_desc[2].path, "/folder1/a.png"); // 100

        // 3. Sorted by size ASC
        let all_by_size = db
            .query_files(None, FileSortField::SizeBytes, SortDirection::Asc)
            .unwrap();
        assert_eq!(all_by_size[0].path, "/folder1/b.png"); // 300
        assert_eq!(all_by_size[1].path, "/folder1/a.png"); // 500
        assert_eq!(all_by_size[2].path, "/folder2/c.png"); // 800

        // 4. Sorted by rating DESC (NULLs last)
        let all_by_rating = db
            .query_files(None, FileSortField::Rating, SortDirection::Desc)
            .unwrap();
        assert_eq!(all_by_rating[0].path, "/folder1/b.png"); // rating 9
        assert_eq!(all_by_rating[1].path, "/folder1/a.png"); // rating 8
        assert_eq!(all_by_rating[2].path, "/folder2/c.png"); // rating None (nulls last)

        // 5. Sorted by aesthetic_score DESC (NULLs last)
        let all_by_aesthetic = db
            .query_files(None, FileSortField::AestheticScore, SortDirection::Desc)
            .unwrap();
        assert_eq!(all_by_aesthetic[0].path, "/folder2/c.png"); // 8.2
        assert_eq!(all_by_aesthetic[1].path, "/folder1/a.png"); // 7.5
        assert_eq!(all_by_aesthetic[2].path, "/folder1/b.png"); // None (nulls last)
    }

    #[test]
    fn rating_updates_and_preservation_on_reupsert() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let id = db.upsert_file(&image(folder.id, "/img/a.png")).unwrap();

        // Initially no rating
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, None);

        // Set valid rating
        db.set_file_rating(id, Some(7)).unwrap();
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, Some(7));

        // Invalid rating rejected
        assert!(matches!(
            db.set_file_rating(id, Some(0)),
            Err(DatabaseError::InvalidRating(0))
        ));
        assert!(matches!(
            db.set_file_rating(id, Some(11)),
            Err(DatabaseError::InvalidRating(11))
        ));

        // Setting rating on nonexistent file returns error
        assert!(matches!(
            db.set_file_rating(999, Some(5)),
            Err(DatabaseError::FileNotFound(999))
        ));

        // Re-scanning (upserting without rating) preserves the previously set rating
        let re_scan_file = image(folder.id, "/img/a.png"); // has rating: None
        db.upsert_file(&re_scan_file).unwrap();
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, Some(7), "re-upsert preserves user rating");

        // Clearing rating with None works
        db.set_file_rating(id, None).unwrap();
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, None);
    }

    #[test]
    fn file_counts_per_folder_and_total() {
        let db = Database::connect_in_memory().unwrap();
        let f1 = db.add_folder("/f1").unwrap();
        let f2 = db.add_folder("/f2").unwrap();

        assert_eq!(db.count_all_files().unwrap(), 0);

        db.upsert_file(&image(f1.id, "/f1/1.png")).unwrap();
        db.upsert_file(&image(f1.id, "/f1/2.png")).unwrap();
        db.upsert_file(&image(f2.id, "/f2/1.png")).unwrap();

        assert_eq!(db.count_all_files().unwrap(), 3);
        let counts = db.get_folder_file_counts().unwrap();
        assert_eq!(counts.get(&f1.id), Some(&2));
        assert_eq!(counts.get(&f2.id), Some(&1));
    }

    fn sample_meta(
        prompt: &str,
        neg: &str,
        model: &str,
        sampler: &str,
        steps: u32,
        cfg: f64,
        seed: &str,
    ) -> ExtractedMetadata {
        ExtractedMetadata {
            format: berry_domain::MetadataFormat::A1111,
            parameters: None,
            raw: None,
            prompt: Some(prompt.to_string()),
            negative_prompt: Some(neg.to_string()),
            width: Some(512),
            height: Some(768),
            seed: Some(seed.to_string()),
            steps: Some(steps),
            cfg_scale: Some(cfg),
            sampler: Some(sampler.to_string()),
            model_name: Some(model.to_string()),
            model_hash: Some("abc12345".to_string()),
        }
    }

    #[test]
    fn search_files_multi_criteria() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/library").unwrap();

        let mut file1 = image(folder.id, "/library/cyberpunk_cat.png");
        file1.rating = Some(9);
        file1.aesthetic_score = Some(8.5);
        file1.metadata = Some(sample_meta(
            "a neon cyberpunk cat, 8k resolution, photorealistic",
            "blurry, low quality",
            "dreamshaper_xl",
            "Euler a",
            25,
            7.0,
            "12345",
        ));

        let mut file2 = image(folder.id, "/library/nature_forest.png");
        file2.rating = Some(6);
        file2.aesthetic_score = Some(6.2);
        file2.metadata = Some(sample_meta(
            "serene green forest, river stream, sunlight",
            "ugly, artifacts",
            "realisticVision_v5",
            "DPM++ 2M Karras",
            30,
            5.5,
            "67890",
        ));

        let mut file3 = image(folder.id, "/library/anime_portrait.png");
        file3.rating = Some(8);
        file3.aesthetic_score = Some(7.8);
        file3.metadata = Some(sample_meta(
            "anime girl in city, colorful lights",
            "bad anatomy",
            "dreamshaper_xl",
            "Euler a",
            20,
            8.0,
            "99999",
        ));

        db.upsert_files(&[file1, file2, file3]).unwrap();

        // 1. Broad text search for "cyberpunk"
        let res = db
            .search_files(&SearchCriteria {
                text: Some("cyberpunk".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].path, "/library/cyberpunk_cat.png");

        // 2. Broad text search matching model name "dreamshaper"
        let res = db
            .search_files(&SearchCriteria {
                text: Some("dreamshaper".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2);

        // 3. Prompt specific search
        let res = db
            .search_files(&SearchCriteria {
                prompt: Some("forest".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].path, "/library/nature_forest.png");

        // 4. Rating range >= 8
        let res = db
            .search_files(&SearchCriteria {
                min_rating: Some(8),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2);

        // 5. Steps between 22 and 35
        let res = db
            .search_files(&SearchCriteria {
                min_steps: Some(22),
                max_steps: Some(35),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2); // file1 (25) and file2 (30)

        // 6. CFG scale >= 7.0 and model_name dreamshaper_xl
        let res = db
            .search_files(&SearchCriteria {
                model_name: Some("dreamshaper_xl".to_string()),
                min_cfg: Some(7.0),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 2);

        // 7. Sampler filter
        let res = db
            .search_files(&SearchCriteria {
                sampler: Some("DPM++".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].path, "/library/nature_forest.png");

        // 8. Pagination (limit 1, offset 1) sorted by rating DESC
        let res = db
            .search_files(&SearchCriteria {
                sort: Some(FileSortField::Rating),
                direction: Some(SortDirection::Desc),
                limit: Some(1),
                offset: Some(1),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].rating, Some(8)); // 2nd highest rating (after 9)

        let page = db
            .search_files_page(&SearchCriteria {
                sort: Some(FileSortField::Rating),
                direction: Some(SortDirection::Desc),
                limit: Some(2),
                offset: Some(0),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.total, 3);
        assert_eq!(page.offset, 0);
        assert!(page.has_more);

        let final_page = db
            .search_files_page(&SearchCriteria {
                sort: Some(FileSortField::Rating),
                direction: Some(SortDirection::Desc),
                limit: Some(2),
                offset: Some(2),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(final_page.items.len(), 1);
        assert_eq!(final_page.total, 3);
        assert!(!final_page.has_more);

        // 9. Distinct models and samplers
        let models = db.list_distinct_models().unwrap();
        assert_eq!(models, vec!["dreamshaper_xl", "realisticVision_v5"]);

        let samplers = db.list_distinct_samplers().unwrap();
        assert_eq!(samplers, vec!["DPM++ 2M Karras", "Euler a"]);
    }

    #[test]
    fn set_files_rating_batch() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let id1 = db.upsert_file(&image(folder.id, "/img/1.png")).unwrap();
        let id2 = db.upsert_file(&image(folder.id, "/img/2.png")).unwrap();
        let _id3 = db.upsert_file(&image(folder.id, "/img/3.png")).unwrap();

        // Batch update id1 and id2 to rating 9
        let updated = db.set_files_rating(&[id1, id2], Some(9)).unwrap();
        assert_eq!(updated, 2);

        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, Some(9));
        assert_eq!(files[1].rating, Some(9));
        assert_eq!(files[2].rating, None);

        // Batch clear ratings
        let cleared = db.set_files_rating(&[id1, id2], None).unwrap();
        assert_eq!(cleared, 2);
        let files = db.list_files(folder.id).unwrap();
        assert_eq!(files[0].rating, None);
        assert_eq!(files[1].rating, None);
    }

    #[test]
    fn album_crud_and_association() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let id1 = db.upsert_file(&image(folder.id, "/img/1.png")).unwrap();
        let id2 = db.upsert_file(&image(folder.id, "/img/2.png")).unwrap();
        let id3 = db.upsert_file(&image(folder.id, "/img/3.png")).unwrap();

        // 1. Create album
        let album = db
            .create_album("Cyberpunk", Some("Sci-fi aesthetic collection"))
            .unwrap();
        assert_eq!(album.name, "Cyberpunk");
        assert_eq!(
            album.description.as_deref(),
            Some("Sci-fi aesthetic collection")
        );

        // 2. Add files to album
        db.add_file_to_album(album.id, id1).unwrap();
        db.add_files_to_album(album.id, &[id2, id3]).unwrap();
        assert_eq!(db.count_album_files(album.id).unwrap(), 3);

        // 3. List album files
        let album_files = db.list_album_files(album.id).unwrap();
        assert_eq!(album_files.len(), 3);

        // 4. Remove a file from album
        db.remove_file_from_album(album.id, id2).unwrap();
        assert_eq!(db.count_album_files(album.id).unwrap(), 2);

        // 5. Rename album
        db.rename_album(album.id, "Cyberpunk 2077").unwrap();
        let renamed = db.get_album(album.id).unwrap().unwrap();
        assert_eq!(renamed.name, "Cyberpunk 2077");

        // 6. Delete album (cascades)
        db.delete_album(album.id).unwrap();
        assert_eq!(db.get_album(album.id).unwrap(), None);
        assert_eq!(db.count_album_files(album.id).unwrap(), 0);
    }

    #[test]
    fn tag_crud_and_association() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let id1 = db.upsert_file(&image(folder.id, "/img/1.png")).unwrap();
        let id2 = db.upsert_file(&image(folder.id, "/img/2.png")).unwrap();

        // 1. Create tag
        let tag1 = db.create_tag("featured", Some("#3b82f6")).unwrap();
        let tag2 = db.create_tag("wallpaper", None).unwrap();
        assert_eq!(tag1.name, "featured");
        assert_eq!(tag1.color.as_deref(), Some("#3b82f6"));

        let all_tags = db.list_tags().unwrap();
        assert_eq!(all_tags.len(), 2);

        // 2. Tag files
        db.tag_file(id1, tag1.id).unwrap();
        db.tag_files(&[id1, id2], tag2.id).unwrap();

        let id1_tags = db.get_file_tags(id1).unwrap();
        assert_eq!(id1_tags.len(), 2);

        let tagged_with_wallpaper = db.list_files_by_tag(tag2.id).unwrap();
        assert_eq!(tagged_with_wallpaper.len(), 2);

        // 3. Untag file
        db.untag_file(id1, tag1.id).unwrap();
        let id1_tags = db.get_file_tags(id1).unwrap();
        assert_eq!(id1_tags.len(), 1);

        // 4. Delete tag (cascades)
        db.delete_tag(tag2.id).unwrap();
        let id1_tags = db.get_file_tags(id1).unwrap();
        assert_eq!(id1_tags.len(), 0);
    }

    #[test]
    fn favorites_and_nsfw_flags() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();
        let id1 = db.upsert_file(&image(folder.id, "/img/1.png")).unwrap();
        let id2 = db.upsert_file(&image(folder.id, "/img/2.png")).unwrap();
        let id3 = db.upsert_file(&image(folder.id, "/img/3.png")).unwrap();

        // Initially both false
        let f1 = db.list_files(folder.id).unwrap();
        assert!(!f1[0].is_favorite);
        assert!(!f1[0].is_nsfw);

        // Set favorite
        db.set_file_favorite(id1, true).unwrap();
        db.set_files_favorite(&[id2, id3], true).unwrap();
        let favs = db
            .search_files(&SearchCriteria {
                is_favorite: Some(true),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(favs.len(), 3);

        // Set NSFW
        db.set_file_nsfw(id3, true).unwrap();
        let nsfw_files = db
            .search_files(&SearchCriteria {
                is_nsfw: Some(true),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(nsfw_files.len(), 1);
        assert_eq!(nsfw_files[0].id, Some(id3));

        let sfw_files = db
            .search_files(&SearchCriteria {
                is_nsfw: Some(false),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(sfw_files.len(), 2);
    }

    #[test]
    fn prompt_statistics() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();

        let mut f1 = image(folder.id, "/img/1.png");
        f1.metadata = Some(sample_meta(
            "masterpiece, 1girl, solo, cherry blossoms",
            "blurry, low quality",
            "model_a",
            "Euler",
            20,
            7.0,
            "123",
        ));
        db.upsert_file(&f1).unwrap();

        let mut f2 = image(folder.id, "/img/2.png");
        f2.metadata = Some(sample_meta(
            "masterpiece, 1girl, futuristic city",
            "blurry, watermark",
            "model_a",
            "Euler",
            20,
            7.0,
            "456",
        ));
        db.upsert_file(&f2).unwrap();

        let stats = db.get_prompt_stats(false, 10).unwrap();
        // "masterpiece" and "1girl" appeared in both images (count 2)
        assert_eq!(stats[0].count, 2);
        assert_eq!(stats[1].count, 2);

        let neg_stats = db.get_prompt_stats(true, 10).unwrap();
        // "blurry" appeared in both images (count 2)
        assert_eq!(neg_stats[0].text, "blurry");
        assert_eq!(neg_stats[0].count, 2);
    }

    #[test]
    fn checkpoint_models_and_cache() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/img").unwrap();

        let mut f1 = image(folder.id, "/img/1.png");
        f1.metadata = Some(sample_meta(
            "prompt1",
            "neg1",
            "v1-5-pruned.safetensors",
            "Euler",
            20,
            7.0,
            "123",
        ));
        db.upsert_file(&f1).unwrap();

        let mut f2 = image(folder.id, "/img/2.png");
        f2.metadata = Some(sample_meta(
            "prompt2",
            "neg2",
            "v1-5-pruned.safetensors",
            "Euler",
            20,
            7.0,
            "456",
        ));
        db.upsert_file(&f2).unwrap();

        let mut f3 = image(folder.id, "/img/3.png");
        f3.metadata = Some(sample_meta(
            "prompt3",
            "neg3",
            "sd_xl_base_1.0.safetensors",
            "Euler",
            20,
            7.0,
            "789",
        ));
        db.upsert_file(&f3).unwrap();

        // 1. Checkpoint model stats
        let models = db.get_checkpoint_models().unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].model_name, "v1-5-pruned.safetensors");
        assert_eq!(models[0].count, 2);
        assert_eq!(models[1].model_name, "sd_xl_base_1.0.safetensors");
        assert_eq!(models[1].count, 1);

        // 2. Import cache entries
        let entries = vec![
            ModelCacheEntry {
                hash: "e4a30e46".to_string(),
                name: "v1-5-pruned.safetensors".to_string(),
                title: Some("Stable Diffusion v1.5".to_string()),
                sha256: Some(
                    "e4a30e4620f1c21a4bf600d9f66f3ba7608e0b31ff8c64e9ddda4b57a37e44ec".to_string(),
                ),
            },
            ModelCacheEntry {
                hash: "31e35c80".to_string(),
                name: "sd_xl_base_1.0.safetensors".to_string(),
                title: Some("SDXL Base 1.0".to_string()),
                sha256: Some(
                    "31e35c80fc4829d14f90153f40f6cd80c400577640de36d3933cfa1be5d7d84d".to_string(),
                ),
            },
        ];
        let imported = db.import_model_cache(&entries).unwrap();
        assert_eq!(imported, 2);

        // 3. Resolve hash
        let resolved = db.resolve_model_hash("e4a30e46").unwrap().expect("found");
        assert_eq!(resolved.name, "v1-5-pruned.safetensors");
        assert_eq!(resolved.title.as_deref(), Some("Stable Diffusion v1.5"));

        let resolved_sha = db
            .resolve_model_hash("31e35c80fc48")
            .unwrap()
            .expect("found prefix");
        assert_eq!(resolved_sha.name, "sd_xl_base_1.0.safetensors");

        // 4. List cache
        let list = db.list_model_cache().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn file_move_and_delete_operations() {
        let db = Database::connect_in_memory().unwrap();
        let f1 = db.add_folder("/folder1").unwrap();
        let f2 = db.add_folder("/folder2").unwrap();

        let id = db.upsert_file(&image(f1.id, "/folder1/cat.png")).unwrap();

        let retrieved = db
            .get_file_by_path("/folder1/cat.png")
            .unwrap()
            .expect("exists");
        assert_eq!(retrieved.id, Some(id));
        assert_eq!(retrieved.folder_id, f1.id);

        // Move to folder2
        db.move_file_record("/folder1/cat.png", "/folder2/cat.png", f2.id)
            .unwrap();

        assert_eq!(db.get_file_by_path("/folder1/cat.png").unwrap(), None);
        let moved = db
            .get_file_by_path("/folder2/cat.png")
            .unwrap()
            .expect("moved exists");
        assert_eq!(moved.id, Some(id));
        assert_eq!(moved.folder_id, f2.id);

        // Delete file
        db.delete_file_by_path("/folder2/cat.png").unwrap();
        assert_eq!(db.get_file_by_path("/folder2/cat.png").unwrap(), None);
    }

    #[test]
    fn database_stats_and_vacuum_backup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::connect(&db_path).unwrap();

        let f = db.add_folder("/test_folder").unwrap();
        db.upsert_file(&image(f.id, "/test_folder/img1.png"))
            .unwrap();
        db.create_album("Test Album", None).unwrap();
        db.create_tag("Tag1", None).unwrap();

        let stats = db.get_database_stats().unwrap();
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.folder_count, 1);
        assert_eq!(stats.album_count, 1);
        assert_eq!(stats.tag_count, 1);
        assert!(stats.db_size_bytes > 0);

        // Test vacuum
        db.vacuum_database().unwrap();

        // Test backup
        let backup_path = temp_dir.path().join("backup.db");
        db.backup_database(&backup_path.to_string_lossy()).unwrap();
        assert!(backup_path.exists());

        // Verify backup database is a valid SQLite db
        let backup_db = Database::connect(&backup_path).unwrap();
        let backup_stats = backup_db.get_database_stats().unwrap();
        assert_eq!(backup_stats.file_count, 1);
    }

    // --- File Embeddings and Similarity Search Tests ---

    #[test]
    fn migration_reaches_schema_version_9() {
        let db = Database::connect_in_memory().unwrap();
        assert_eq!(db.user_version().unwrap(), 9);
        assert_eq!(LATEST_VERSION, 9);
    }

    #[test]
    fn file_embeddings_table_and_model_index_exist() {
        let db = Database::connect_in_memory().unwrap();
        let table_exists: bool = db
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'file_embeddings')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists);

        let loras_exists: bool = db
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'loras')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(loras_exists);
        assert!(table_exists);

        let index_exists: bool = db
            .conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'index' AND name = 'idx_file_embeddings_model')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);
    }

    #[test]
    fn foreign_key_cascade_cleanup() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img1.png"))
            .unwrap();

        db.upsert_file_embedding(file_id, "clip-vit-b32", &[1.0, 0.0, 0.0])
            .unwrap();
        db.upsert_file_embedding(file_id, "siglip-base", &[0.0, 1.0, 0.0])
            .unwrap();

        let count: i64 = db
            .conn
            .query_row(
                "SELECT count(*) FROM file_embeddings WHERE file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);

        // Deleting the file should cascade-delete all of its embeddings
        db.delete_file_by_path("/test_folder/img1.png").unwrap();

        let count_after: i64 = db
            .conn
            .query_row(
                "SELECT count(*) FROM file_embeddings WHERE file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count_after, 0);

        // Also test folder removal cascades through files to embeddings
        let file2_id = db
            .upsert_file(&image(folder.id, "/test_folder/img2.png"))
            .unwrap();
        db.upsert_file_embedding(file2_id, "clip-vit-b32", &[1.0, 1.0, 1.0])
            .unwrap();
        db.remove_folder(folder.id).unwrap();

        let count_folder_after: i64 = db
            .conn
            .query_row(
                "SELECT count(*) FROM file_embeddings WHERE file_id = ?1",
                params![file2_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count_folder_after, 0);
    }

    #[test]
    fn embedding_insertion() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        let vec = [0.1f32, 0.2, 0.3, 0.4];
        db.upsert_file_embedding(file_id, "test-model", &vec)
            .unwrap();

        let matches = db.search_similar_files("test-model", &vec, 10).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].file_id, file_id);
        assert!((matches[0].score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn upsert_replacement() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        db.upsert_file_embedding(file_id, "clip", &[1.0, 0.0])
            .unwrap();

        // Stored dimensions = 2
        let (dim, blob, _updated_at_1): (i64, Vec<u8>, String) = db
            .conn
            .query_row(
                "SELECT dimensions, embedding, updated_at FROM file_embeddings WHERE file_id = ?1 AND model_id = ?2",
                params![file_id, "clip"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(dim, 2);
        assert_eq!(blob.len(), 8);

        // Replace with 3-dimensional vector
        // Update updated_at to a past timestamp first to ensure refreshed timestamp differs
        db.conn
            .execute(
                "UPDATE file_embeddings SET updated_at = '2000-01-01T00:00:00.000Z' WHERE file_id = ?1 AND model_id = ?2",
                params![file_id, "clip"],
            )
            .unwrap();

        db.upsert_file_embedding(file_id, "clip", &[0.0, 1.0, 0.0])
            .unwrap();

        let (dim2, blob2, updated_at_2): (i64, Vec<u8>, String) = db
            .conn
            .query_row(
                "SELECT dimensions, embedding, updated_at FROM file_embeddings WHERE file_id = ?1 AND model_id = ?2",
                params![file_id, "clip"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(dim2, 3);
        assert_eq!(blob2.len(), 12);
        assert_ne!(updated_at_2, "2000-01-01T00:00:00.000Z");

        // Verify search with new vector matches score ~ 1.0
        let matches = db
            .search_similar_files("clip", &[0.0, 1.0, 0.0], 10)
            .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].file_id, file_id);
        assert!((matches[0].score - 1.0).abs() < 1e-5);
    }

    #[test]
    fn independent_embeddings_for_different_models() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        db.upsert_file_embedding(file_id, "model-a", &[1.0, 0.0])
            .unwrap();
        db.upsert_file_embedding(file_id, "model-b", &[0.0, 1.0])
            .unwrap();

        let matches_a = db.search_similar_files("model-a", &[1.0, 0.0], 10).unwrap();
        assert_eq!(matches_a.len(), 1);
        assert_eq!(matches_a[0].file_id, file_id);
        assert!((matches_a[0].score - 1.0).abs() < 1e-5);

        let matches_b = db.search_similar_files("model-b", &[0.0, 1.0], 10).unwrap();
        assert_eq!(matches_b.len(), 1);
        assert_eq!(matches_b[0].file_id, file_id);
        assert!((matches_b[0].score - 1.0).abs() < 1e-5);

        // Removing model-a leaves model-b intact
        assert!(db.remove_file_embedding(file_id, "model-a").unwrap());
        assert_eq!(
            db.search_similar_files("model-a", &[1.0, 0.0], 10).unwrap(),
            vec![]
        );
        let matches_b_still = db.search_similar_files("model-b", &[0.0, 1.0], 10).unwrap();
        assert_eq!(matches_b_still.len(), 1);
    }

    #[test]
    fn removal_and_removal_of_missing_row() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        // Removing missing row returns Ok(false)
        assert!(!db.remove_file_embedding(file_id, "clip").unwrap());
        assert!(!db.remove_file_embedding(99999, "clip").unwrap());

        // Insert and remove
        db.upsert_file_embedding(file_id, "clip", &[1.0, 2.0])
            .unwrap();
        assert!(db.remove_file_embedding(file_id, "clip").unwrap());
        // Removing again returns Ok(false)
        assert!(!db.remove_file_embedding(file_id, "clip").unwrap());
    }

    #[test]
    fn cosine_similarity_ranking() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();

        let id1 = db
            .upsert_file(&image(folder.id, "/test_folder/img1.png"))
            .unwrap();
        let id2 = db
            .upsert_file(&image(folder.id, "/test_folder/img2.png"))
            .unwrap();
        let id3 = db
            .upsert_file(&image(folder.id, "/test_folder/img3.png"))
            .unwrap();
        let id4 = db
            .upsert_file(&image(folder.id, "/test_folder/img4.png"))
            .unwrap();

        // Vector 1: identical direction to query [1.0, 0.0] -> cosine = 1.0
        db.upsert_file_embedding(id1, "clip", &[10.0, 0.0]).unwrap();
        // Vector 2: 45 degrees [1.0, 1.0] -> cosine ~ 0.7071
        db.upsert_file_embedding(id2, "clip", &[1.0, 1.0]).unwrap();
        // Vector 3: orthogonal [0.0, 5.0] -> cosine = 0.0
        db.upsert_file_embedding(id3, "clip", &[0.0, 5.0]).unwrap();
        // Vector 4: opposite direction [-2.0, 0.0] -> cosine = -1.0
        db.upsert_file_embedding(id4, "clip", &[-2.0, 0.0]).unwrap();

        let matches = db.search_similar_files("clip", &[1.0, 0.0], 10).unwrap();
        assert_eq!(matches.len(), 4);
        assert_eq!(matches[0].file_id, id1);
        assert!((matches[0].score - 1.0).abs() < 1e-5);
        assert_eq!(matches[1].file_id, id2);
        assert!((matches[1].score - (2.0f32.sqrt() / 2.0)).abs() < 1e-5);
        assert_eq!(matches[2].file_id, id3);
        assert!((matches[2].score - 0.0).abs() < 1e-5);
        assert_eq!(matches[3].file_id, id4);
        assert!((matches[3].score - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn deterministic_tie_ordering_by_file_id() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();

        let id_a = db
            .upsert_file(&image(folder.id, "/test_folder/a.png"))
            .unwrap();
        let id_b = db
            .upsert_file(&image(folder.id, "/test_folder/b.png"))
            .unwrap();
        let id_c = db
            .upsert_file(&image(folder.id, "/test_folder/c.png"))
            .unwrap();

        // Ensure IDs are strictly increasing
        assert!(id_a < id_b);
        assert!(id_b < id_c);

        // Insert identical vectors for all three
        db.upsert_file_embedding(id_c, "clip", &[1.0, 0.0]).unwrap();
        db.upsert_file_embedding(id_a, "clip", &[1.0, 0.0]).unwrap();
        db.upsert_file_embedding(id_b, "clip", &[1.0, 0.0]).unwrap();

        let matches = db.search_similar_files("clip", &[1.0, 0.0], 10).unwrap();
        assert_eq!(matches.len(), 3);
        assert_eq!(matches[0].file_id, id_a);
        assert_eq!(matches[1].file_id, id_b);
        assert_eq!(matches[2].file_id, id_c);
    }

    #[test]
    fn result_limits() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();

        for i in 1..=5 {
            let id = db
                .upsert_file(&image(folder.id, &format!("/test_folder/img{i}.png")))
                .unwrap();
            db.upsert_file_embedding(id, "clip", &[i as f32, 1.0])
                .unwrap();
        }

        let matches = db.search_similar_files("clip", &[1.0, 0.0], 2).unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn dimension_mismatch() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        db.upsert_file_embedding(file_id, "clip", &[1.0, 2.0, 3.0])
            .unwrap();

        let err = db
            .search_similar_files("clip", &[1.0, 2.0], 10)
            .unwrap_err();
        match err {
            DatabaseError::DimensionMismatch {
                file_id: matched_id,
                query,
                stored,
            } => {
                assert_eq!(matched_id, file_id);
                assert_eq!(query, 2);
                assert_eq!(stored, 3);
            }
            other => panic!("expected DimensionMismatch, got {other:?}"),
        }
    }

    #[test]
    fn blank_model_ids() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        for blank in ["", "   ", "\t\n"] {
            assert!(matches!(
                db.upsert_file_embedding(file_id, blank, &[1.0]),
                Err(DatabaseError::BlankModelId)
            ));
            assert!(matches!(
                db.remove_file_embedding(file_id, blank),
                Err(DatabaseError::BlankModelId)
            ));
            assert!(matches!(
                db.search_similar_files(blank, &[1.0], 10),
                Err(DatabaseError::BlankModelId)
            ));
        }
    }

    #[test]
    fn empty_non_finite_and_zero_norm_vectors() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        // Empty vector
        assert!(matches!(
            db.upsert_file_embedding(file_id, "clip", &[]),
            Err(DatabaseError::EmptyVector)
        ));
        assert!(matches!(
            db.search_similar_files("clip", &[], 10),
            Err(DatabaseError::EmptyVector)
        ));

        // Non-finite values
        assert!(matches!(
            db.upsert_file_embedding(file_id, "clip", &[1.0, f32::NAN]),
            Err(DatabaseError::NonFiniteVectorValue)
        ));
        assert!(matches!(
            db.upsert_file_embedding(file_id, "clip", &[1.0, f32::INFINITY]),
            Err(DatabaseError::NonFiniteVectorValue)
        ));
        assert!(matches!(
            db.upsert_file_embedding(file_id, "clip", &[1.0, f32::NEG_INFINITY]),
            Err(DatabaseError::NonFiniteVectorValue)
        ));
        assert!(matches!(
            db.search_similar_files("clip", &[1.0, f32::NAN], 10),
            Err(DatabaseError::NonFiniteVectorValue)
        ));
        assert!(matches!(
            db.search_similar_files("clip", &[1.0, f32::INFINITY], 10),
            Err(DatabaseError::NonFiniteVectorValue)
        ));

        // Zero-norm vector
        assert!(matches!(
            db.upsert_file_embedding(file_id, "clip", &[0.0, 0.0]),
            Err(DatabaseError::ZeroVectorNorm)
        ));
        assert!(matches!(
            db.search_similar_files("clip", &[0.0, 0.0], 10),
            Err(DatabaseError::ZeroVectorNorm)
        ));
    }

    #[test]
    fn zero_limit() {
        let db = Database::connect_in_memory().unwrap();
        let err = db.search_similar_files("clip", &[1.0, 2.0], 0).unwrap_err();
        assert!(matches!(err, DatabaseError::ZeroResultLimit));
    }

    #[test]
    fn malformed_stored_blob_data() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        // Insert row with dimensions = 2 (expected 8 bytes), but blob has only 5 bytes
        let malformed_blob = vec![1u8, 2, 3, 4, 5];
        db.conn
            .execute(
                "INSERT INTO file_embeddings (file_id, model_id, dimensions, embedding) VALUES (?1, ?2, ?3, ?4)",
                params![file_id, "clip", 2i64, malformed_blob],
            )
            .unwrap();

        let err = db
            .search_similar_files("clip", &[1.0, 0.0], 10)
            .unwrap_err();
        match err {
            DatabaseError::MalformedStoredBlob {
                file_id: err_file_id,
                dimensions,
                expected_bytes,
                actual_bytes,
            } => {
                assert_eq!(err_file_id, file_id);
                assert_eq!(dimensions, 2);
                assert_eq!(expected_bytes, 8);
                assert_eq!(actual_bytes, 5);
            }
            other => panic!("expected MalformedStoredBlob, got {other:?}"),
        }
    }

    #[test]
    fn corrupt_stored_dimensions_or_values() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img.png"))
            .unwrap();

        // Test 1: Corrupt stored dimensions <= 0 (disable check constraints temporarily)
        db.conn
            .execute("PRAGMA ignore_check_constraints = ON", [])
            .unwrap();
        db.conn
            .execute(
                "INSERT INTO file_embeddings (file_id, model_id, dimensions, embedding) VALUES (?1, ?2, ?3, ?4)",
                params![file_id, "corrupt-dim", 0i64, vec![]],
            )
            .unwrap();
        db.conn
            .execute("PRAGMA ignore_check_constraints = OFF", [])
            .unwrap();

        let err_dim = db
            .search_similar_files("corrupt-dim", &[1.0, 0.0], 10)
            .unwrap_err();
        assert!(matches!(
            err_dim,
            DatabaseError::InvalidStoredDimensions {
                file_id: f_id,
                dimensions: 0,
            } if f_id == file_id
        ));

        // Test 2: Corrupt stored non-finite float value (NaN)
        let mut nan_blob = Vec::new();
        nan_blob.extend_from_slice(&f32::NAN.to_le_bytes());
        nan_blob.extend_from_slice(&1.0f32.to_le_bytes());
        db.conn
            .execute(
                "INSERT INTO file_embeddings (file_id, model_id, dimensions, embedding) VALUES (?1, ?2, ?3, ?4)",
                params![file_id, "corrupt-val", 2i64, nan_blob],
            )
            .unwrap();

        let err_val = db
            .search_similar_files("corrupt-val", &[1.0, 0.0], 10)
            .unwrap_err();
        assert!(matches!(
            err_val,
            DatabaseError::CorruptStoredVectorValue { file_id: f_id } if f_id == file_id
        ));

        // Test 3: Corrupt stored zero-norm vector
        let mut zero_blob = Vec::new();
        zero_blob.extend_from_slice(&0.0f32.to_le_bytes());
        zero_blob.extend_from_slice(&0.0f32.to_le_bytes());
        db.conn
            .execute(
                "INSERT INTO file_embeddings (file_id, model_id, dimensions, embedding) VALUES (?1, ?2, ?3, ?4)",
                params![file_id, "zero-norm", 2i64, zero_blob],
            )
            .unwrap();

        let err_norm = db
            .search_similar_files("zero-norm", &[1.0, 0.0], 10)
            .unwrap_err();
        assert!(matches!(
            err_norm,
            DatabaseError::CorruptStoredZeroNorm { file_id: f_id } if f_id == file_id
        ));
    }

    #[test]
    fn get_file_by_id_works() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();
        let file_id = db
            .upsert_file(&image(folder.id, "/test_folder/img1.png"))
            .unwrap();

        let file = db.get_file_by_id(file_id).unwrap().expect("file exists");
        assert_eq!(file.id, Some(file_id));
        assert_eq!(file.path, "/test_folder/img1.png");

        assert!(db.get_file_by_id(9999).unwrap().is_none());
    }

    #[test]
    fn get_file_embedding_and_models_and_find_similar_to_file() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();

        let id1 = db
            .upsert_file(&image(folder.id, "/test_folder/img1.png"))
            .unwrap();
        let id2 = db
            .upsert_file(&image(folder.id, "/test_folder/img2.png"))
            .unwrap();
        let id3 = db
            .upsert_file(&image(folder.id, "/test_folder/img3.png"))
            .unwrap();

        db.upsert_file_embedding(id1, "clip-vit-b32", &[1.0, 0.0])
            .unwrap();
        db.upsert_file_embedding(id1, "siglip-base", &[0.0, 1.0])
            .unwrap();
        db.upsert_file_embedding(id2, "clip-vit-b32", &[0.9, 0.1])
            .unwrap();
        db.upsert_file_embedding(id3, "clip-vit-b32", &[0.0, 1.0])
            .unwrap();

        // Check get_file_embedding
        let emb1 = db
            .get_file_embedding(id1, "clip-vit-b32")
            .unwrap()
            .expect("embedding exists");
        assert_eq!(emb1, vec![1.0, 0.0]);
        assert!(db
            .get_file_embedding(id1, "nonexistent-model")
            .unwrap()
            .is_none());
        assert!(db
            .get_file_embedding(9999, "clip-vit-b32")
            .unwrap()
            .is_none());

        // Check get_file_embedding_models
        let models1 = db.get_file_embedding_models(id1).unwrap();
        assert_eq!(models1, vec!["clip-vit-b32", "siglip-base"]);
        let models_none = db.get_file_embedding_models(9999).unwrap();
        assert!(models_none.is_empty());

        // Check find_similar_to_file: id1 should find id2 (high score) and id3 (orthogonal), but NOT id1 itself
        let similar = db
            .find_similar_to_file(id1, Some("clip-vit-b32"), 10)
            .unwrap();
        assert_eq!(similar.len(), 2);
        assert_eq!(similar[0].file_id, id2);
        assert_eq!(similar[1].file_id, id3);
        assert!(similar.iter().all(|m| m.file_id != id1));

        // When model_id is None, it defaults to the first available model
        let similar_auto = db.find_similar_to_file(id1, None, 10).unwrap();
        assert_eq!(similar_auto.len(), 2);
        assert_eq!(similar_auto[0].file_id, id2);

        // Limit works
        let similar_limited = db
            .find_similar_to_file(id1, Some("clip-vit-b32"), 1)
            .unwrap();
        assert_eq!(similar_limited.len(), 1);
        assert_eq!(similar_limited[0].file_id, id2);

        // When file has no embeddings
        let id_empty = db
            .upsert_file(&image(folder.id, "/test_folder/no_emb.png"))
            .unwrap();
        let similar_empty = db.find_similar_to_file(id_empty, None, 10).unwrap();
        assert!(similar_empty.is_empty());
    }

    #[test]
    fn get_or_create_tag_works() {
        let db = Database::connect_in_memory().unwrap();
        let tag1 = db.get_or_create_tag("1girl", Some("#ff00ff")).unwrap();
        assert_eq!(tag1.name, "1girl");
        assert_eq!(tag1.color.as_deref(), Some("#ff00ff"));

        // Second call should return existing tag without duplicate error
        let tag2 = db.get_or_create_tag("1girl", None).unwrap();
        assert_eq!(tag1.id, tag2.id);
        assert_eq!(tag2.name, "1girl");

        // Case-insensitive match
        let tag3 = db.get_or_create_tag("1GIRL", None).unwrap();
        assert_eq!(tag1.id, tag3.id);
    }

    #[test]
    fn get_embedding_index_stats_and_unindexed_files_works() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test_folder").unwrap();

        let id1 = db
            .upsert_file(&image(folder.id, "/test_folder/img1.png"))
            .unwrap();
        let id2 = db
            .upsert_file(&image(folder.id, "/test_folder/img2.png"))
            .unwrap();
        let id3 = db
            .upsert_file(&image(folder.id, "/test_folder/img3.png"))
            .unwrap();

        let (indexed, total) = db.get_embedding_index_stats("clip-vit-b32").unwrap();
        assert_eq!(total, 3);
        assert_eq!(indexed, 0);

        let unindexed = db.get_unindexed_files("clip-vit-b32", 10).unwrap();
        assert_eq!(unindexed.len(), 3);
        assert_eq!(unindexed[0].id, Some(id1));
        assert_eq!(unindexed[1].id, Some(id2));
        assert_eq!(unindexed[2].id, Some(id3));

        // Index 2 of them
        db.upsert_file_embedding(id1, "clip-vit-b32", &[1.0, 0.0])
            .unwrap();
        db.upsert_file_embedding(id3, "clip-vit-b32", &[0.0, 1.0])
            .unwrap();

        let (indexed, total) = db.get_embedding_index_stats("clip-vit-b32").unwrap();
        assert_eq!(total, 3);
        assert_eq!(indexed, 2);

        let unindexed_after = db.get_unindexed_files("clip-vit-b32", 10).unwrap();
        assert_eq!(unindexed_after.len(), 1);
        assert_eq!(unindexed_after[0].id, Some(id2));

        // Limit works
        let unindexed_limit = db.get_unindexed_files("clip-vit-b32", 1).unwrap();
        assert_eq!(unindexed_limit.len(), 1);
        assert_eq!(unindexed_limit[0].id, Some(id2));
    }

    #[test]
    fn lora_storage_crud_works() {
        let db = Database::connect_in_memory().unwrap();

        // 1. Initial list is empty
        let initial = db.list_loras().unwrap();
        assert!(initial.is_empty());

        // 2. Save a new LoRA
        let lora_to_save = LoraModel {
            id: 0,
            name: "detail_tweaker_v1".to_string(),
            hash: Some("9b42e7".to_string()),
            trigger_words: vec!["highly detailed".to_string(), "sharp focus".to_string()],
            preview_url: None,
            description: Some("Enhances details and textures".to_string()),
            weight_default: 0.8,
            created_at: String::new(),
            updated_at: String::new(),
        };
        let saved = db.save_lora(&lora_to_save).unwrap();
        assert!(saved.id > 0);
        assert_eq!(saved.name, "detail_tweaker_v1");
        assert_eq!(saved.trigger_words.len(), 2);
        assert_eq!(saved.weight_default, 0.8);

        // 3. Find by name (case-insensitive)
        let found_name = db.find_lora_by_name_or_hash("DETAIL_TWEAKER_V1").unwrap();
        assert!(found_name.is_some());
        assert_eq!(found_name.unwrap().id, saved.id);

        // 4. Find by hash prefix
        let found_hash = db.find_lora_by_name_or_hash("9b42").unwrap();
        assert!(found_hash.is_some());
        assert_eq!(found_hash.unwrap().id, saved.id);

        // 5. Update LoRA
        let mut updated_data = saved.clone();
        updated_data.weight_default = 0.65;
        updated_data.trigger_words.push("masterpiece".to_string());
        let updated = db.save_lora(&updated_data).unwrap();
        assert_eq!(updated.id, saved.id);
        assert_eq!(updated.weight_default, 0.65);
        assert_eq!(updated.trigger_words.len(), 3);

        // 6. Delete LoRA
        let deleted = db.delete_lora(saved.id).unwrap();
        assert!(deleted);
        assert!(db.get_lora(saved.id).unwrap().is_none());
    }

    #[test]
    fn multi_mode_folders_work() {
        let db = Database::connect_in_memory().unwrap();

        // 1. Link mode (default)
        let link_folder = db.add_folder("/path/to/link").unwrap();
        assert_eq!(link_folder.folder_type, "link");
        assert!(link_folder.source_path.is_none());

        // 2. Managed mode
        let managed_folder = db
            .add_folder_with_mode(
                "/path/to/managed",
                "managed",
                None,
                Some("copy"),
                None,
                false,
            )
            .unwrap();
        assert_eq!(managed_folder.folder_type, "managed");
        assert_eq!(managed_folder.ingest_action.as_deref(), Some("copy"));

        // 3. Pipeline mode
        let pipeline_folder = db
            .add_folder_with_mode(
                "/path/to/vault",
                "pipeline",
                Some("/path/to/webui_output"),
                Some("move"),
                Some(48),
                true,
            )
            .unwrap();
        assert_eq!(pipeline_folder.folder_type, "pipeline");
        assert_eq!(
            pipeline_folder.source_path.as_deref(),
            Some("/path/to/webui_output")
        );
        assert_eq!(pipeline_folder.grace_period_hours, Some(48));
        assert!(pipeline_folder.auto_harvest);

        // 4. List all folders
        let all = db.list_folders().unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].id, link_folder.id);
        assert_eq!(all[1].id, managed_folder.id);
        assert_eq!(all[2].id, pipeline_folder.id);

        // 5. Find by id
        let found = db.find_folder_by_id(pipeline_folder.id).unwrap().unwrap();
        assert_eq!(found.path, "/path/to/vault");
        assert_eq!(found.folder_type, "pipeline");
    }

    #[test]
    fn image_stacking_and_cleanup_queue_work() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test/folder").unwrap();

        // Seed 3 test files
        let file1 = ImageFile {
            id: None,
            folder_id: folder.id,
            path: "/test/folder/img1.png".to_string(),
            size_bytes: 1024,
            modified_at: 100,
            container: Container::Png,
            metadata: None,
            rating: Some(4),
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let mut file2 = file1.clone();
        file2.path = "/test/folder/img2.png".to_string();
        file2.rating = Some(2);

        let mut file3 = file1.clone();
        file3.path = "/test/folder/img3.png".to_string();
        file3.rating = None;

        let id1 = db.upsert_file(&file1).unwrap();
        let id2 = db.upsert_file(&file2).unwrap();
        let id3 = db.upsert_file(&file3).unwrap();

        // 1. Stack images [id1, id2, id3]
        let stack_id = db.stack_images(&[id1, id2, id3], None).unwrap();
        assert!(!stack_id.is_empty());

        // Verify members
        let members = db.get_stack_members(&stack_id).unwrap();
        assert_eq!(members.len(), 3);
        assert_eq!(members[0].id, Some(id1));
        assert_eq!(members[0].stack_order, 0); // Hero
        assert_eq!(members[1].id, Some(id2));
        assert_eq!(members[1].stack_order, 1);
        assert_eq!(members[2].id, Some(id3));
        assert_eq!(members[2].stack_order, 2);

        // 2. Set hero cover to id2
        db.set_stack_hero(&stack_id, id2).unwrap();
        let members_after_hero = db.get_stack_members(&stack_id).unwrap();
        assert_eq!(members_after_hero[0].id, Some(id2));
        assert_eq!(members_after_hero[0].stack_order, 0);

        // 3. List stacks
        let stacks = db.list_stacks(None).unwrap();
        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].stack_id, stack_id);
        assert_eq!(stacks[0].count, 3);
        assert_eq!(stacks[0].hero_image_id, Some(id2));

        // 4. Cull candidates: non-hero with rating < 3 or unrated
        let cull_ids = db.get_stack_cull_candidate_ids(&stack_id, 3).unwrap();
        // id2 is hero (exempt), id1 has rating 4 (>=3 exempt), id3 has no rating (<3 cull candidate)
        assert_eq!(cull_ids, vec![id3]);

        // 5. Cleanup queue
        let q_id = db.enqueue_cleanup("/source/temp.png", id1, 24).unwrap();
        assert!(q_id > 0);

        let due_now = db.list_due_cleanups(0).unwrap();
        assert!(due_now.is_empty());

        let future_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 100_000;
        let due_future = db.list_due_cleanups(future_timestamp).unwrap();
        assert_eq!(due_future.len(), 1);
        assert_eq!(due_future[0].source_file_path, "/source/temp.png");

        db.update_cleanup_status(q_id, "deleted").unwrap();
        let queue = db.get_cleanup_queue(10).unwrap();
        assert_eq!(queue[0].status, "deleted");

        // 6. Unstack
        let unstacked = db.unstack_images(&stack_id).unwrap();
        assert_eq!(unstacked, 3);
        assert!(db.get_stack_members(&stack_id).unwrap().is_empty());
    }

    #[test]
    fn merge_stacks_flattens_members_and_preserves_target_hero() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test/merge").unwrap();
        let base = ImageFile {
            id: None,
            folder_id: folder.id,
            path: String::new(),
            size_bytes: 1024,
            modified_at: 100,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let ids = (0..6)
            .map(|index| {
                let mut file = base.clone();
                file.path = format!("/test/merge/img{index}.png");
                db.upsert_file(&file).unwrap()
            })
            .collect::<Vec<_>>();

        let target = db.stack_images(&ids[0..2], Some("target")).unwrap();
        let source = db.stack_images(&ids[2..4], Some("source")).unwrap();
        db.set_stack_hero(&target, ids[1]).unwrap();

        let count = db
            .merge_stacks(
                &target,
                &[source.clone(), source],
                &[ids[4], ids[4], ids[5]],
            )
            .unwrap();

        let members = db.get_stack_members(&target).unwrap();
        assert_eq!(count, 6);
        assert_eq!(
            members
                .iter()
                .filter_map(|file| file.id)
                .collect::<Vec<_>>(),
            vec![ids[1], ids[0], ids[2], ids[3], ids[4], ids[5]],
        );
        assert_eq!(members[0].stack_order, 0);
        assert!(db.get_stack_members("source").unwrap().is_empty());
    }

    #[test]
    fn merge_stacks_rejects_an_unexpanded_stacked_file() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/test/merge-validation").unwrap();
        let mut file = ImageFile {
            id: None,
            folder_id: folder.id,
            path: "/test/merge-validation/a.png".to_string(),
            size_bytes: 1,
            modified_at: 1,
            container: Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        let id1 = db.upsert_file(&file).unwrap();
        file.path = "/test/merge-validation/b.png".to_string();
        let id2 = db.upsert_file(&file).unwrap();
        file.path = "/test/merge-validation/c.png".to_string();
        let id3 = db.upsert_file(&file).unwrap();
        let target = db.stack_images(&[id1], Some("target")).unwrap();
        db.stack_images(&[id2, id3], Some("source")).unwrap();

        let error = db.merge_stacks(&target, &[], &[id2]).unwrap_err();
        assert!(matches!(
            error,
            DatabaseError::FileAlreadyStacked { file_id, .. } if file_id == id2
        ));
        assert_eq!(db.get_stack_members(&target).unwrap().len(), 1);
        assert_eq!(db.get_stack_members("source").unwrap().len(), 2);
    }
}
