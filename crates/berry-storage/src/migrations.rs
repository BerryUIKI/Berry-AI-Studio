//! Ordered SQL migrations.
//!
//! Each entry is applied inside a single transaction and advances
//! `PRAGMA user_version` by one. **Never reorder, edit, or remove an applied
//! migration** — append a new one instead. Deployed databases rely on this.

/// The ordered list of migrations. Index `i` migrates the schema from version
/// `i` to version `i + 1`.
pub const MIGRATIONS: &[&str] = &[
    // v1: foundation tables.
    r#"
    CREATE TABLE meta (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    ) STRICT;
    "#,
    // v2: folder and file indexing.
    r#"
    CREATE TABLE folders (
        id        INTEGER PRIMARY KEY,
        path      TEXT NOT NULL UNIQUE,
        added_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE TABLE files (
        id          INTEGER PRIMARY KEY,
        folder_id   INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
        path        TEXT NOT NULL UNIQUE,
        container   TEXT NOT NULL,      -- stable Container id: png|jpg|webp|mp4
        size_bytes  INTEGER NOT NULL,
        modified_at INTEGER NOT NULL,   -- unix seconds; incremental-scan cache
        metadata    TEXT,               -- JSON ExtractedMetadata, NULL until extracted
        indexed_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE INDEX idx_files_folder ON files(folder_id);
    "#,
    // v3: ratings, aesthetics score, and browsing indexes.
    r#"
    ALTER TABLE files ADD COLUMN rating INTEGER;
    ALTER TABLE files ADD COLUMN aesthetic_score REAL;

    CREATE INDEX idx_files_modified ON files(modified_at);
    CREATE INDEX idx_files_rating ON files(rating);
    CREATE INDEX idx_files_aesthetic ON files(aesthetic_score);
    "#,
    // v4: albums, tags, favorites, and nsfw flag.
    r#"
    CREATE TABLE albums (
        id          INTEGER PRIMARY KEY,
        name        TEXT NOT NULL UNIQUE,
        description TEXT,
        created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE TABLE album_files (
        album_id    INTEGER NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
        file_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
        added_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
        PRIMARY KEY (album_id, file_id)
    ) STRICT;

    CREATE INDEX idx_album_files_file ON album_files(file_id);

    CREATE TABLE tags (
        id          INTEGER PRIMARY KEY,
        name        TEXT NOT NULL UNIQUE,
        color       TEXT,
        created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE TABLE file_tags (
        file_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
        tag_id      INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
        PRIMARY KEY (file_id, tag_id)
    ) STRICT;

    CREATE INDEX idx_file_tags_tag ON file_tags(tag_id);

    ALTER TABLE files ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE files ADD COLUMN is_nsfw INTEGER NOT NULL DEFAULT 0;

    CREATE INDEX idx_files_favorite ON files(is_favorite);
    CREATE INDEX idx_files_nsfw ON files(is_nsfw);
    "#,
    // v5: checkpoint model cache table.
    r#"
    CREATE TABLE model_cache (
        hash        TEXT PRIMARY KEY,
        name        TEXT NOT NULL,
        title       TEXT,
        sha256      TEXT,
        updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE INDEX idx_model_cache_name ON model_cache(name);
    "#,
    // v6: performance indexing for composite and size_bytes sorting.
    r#"
    CREATE INDEX IF NOT EXISTS idx_files_folder_modified ON files(folder_id, modified_at DESC);
    CREATE INDEX IF NOT EXISTS idx_files_size ON files(size_bytes DESC);
    "#,
    // v7: file embeddings table and index.
    r#"
    CREATE TABLE file_embeddings (
        file_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
        model_id    TEXT NOT NULL,
        dimensions  INTEGER NOT NULL CHECK (dimensions > 0),
        embedding   BLOB NOT NULL,
        updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
        PRIMARY KEY (file_id, model_id)
    ) STRICT;

    CREATE INDEX idx_file_embeddings_model ON file_embeddings(model_id);
    "#,
    // v8: loras table for trigger words and catalog.
    r#"
    CREATE TABLE loras (
        id              INTEGER PRIMARY KEY,
        name            TEXT NOT NULL UNIQUE,
        hash            TEXT,
        trigger_words   TEXT NOT NULL,
        preview_url     TEXT,
        description     TEXT,
        weight_default  REAL NOT NULL DEFAULT 1.0,
        created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
        updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    ) STRICT;

    CREATE INDEX idx_loras_name ON loras(name);
    CREATE INDEX idx_loras_hash ON loras(hash);
    "#,
    // v9: multi-mode folders, image stacking, and pipeline cleanup queue.
    r#"
    ALTER TABLE folders ADD COLUMN folder_type TEXT NOT NULL DEFAULT 'link';
    ALTER TABLE folders ADD COLUMN source_path TEXT;
    ALTER TABLE folders ADD COLUMN ingest_action TEXT DEFAULT 'copy';
    ALTER TABLE folders ADD COLUMN grace_period_hours INTEGER DEFAULT 24;
    ALTER TABLE folders ADD COLUMN auto_harvest INTEGER NOT NULL DEFAULT 1;

    ALTER TABLE files ADD COLUMN stack_id TEXT;
    ALTER TABLE files ADD COLUMN stack_order INTEGER NOT NULL DEFAULT 0;

    CREATE INDEX IF NOT EXISTS idx_files_stack_id ON files(stack_id);
    CREATE INDEX IF NOT EXISTS idx_folders_type ON folders(folder_type);

    CREATE TABLE pipeline_cleanup_queue (
        id                  INTEGER PRIMARY KEY,
        source_file_path    TEXT NOT NULL UNIQUE,
        target_file_id      INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
        scheduled_delete_at INTEGER NOT NULL,
        created_at          INTEGER NOT NULL,
        status              TEXT NOT NULL DEFAULT 'pending'
    ) STRICT;

    CREATE INDEX idx_cleanup_schedule ON pipeline_cleanup_queue(scheduled_delete_at, status);
    "#,
    // v10: durable coalesced filesystem events for targeted reconciliation.
    r#"
    CREATE TABLE filesystem_change_journal (
        folder_id   INTEGER NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
        path        TEXT NOT NULL,
        event_kind  TEXT NOT NULL,
        observed_at INTEGER NOT NULL,
        PRIMARY KEY (folder_id, path)
    ) STRICT;

    CREATE INDEX idx_filesystem_change_observed
        ON filesystem_change_journal(observed_at);
    "#,
    // v11: persistent thumbnail cache manifest for size tiers and LRU cleanup.
    r#"
    CREATE TABLE thumbnail_cache_entries (
        file_id          INTEGER NOT NULL,
        modified_at      INTEGER NOT NULL,
        max_edge         INTEGER NOT NULL,
        codec            TEXT NOT NULL,
        path             TEXT NOT NULL UNIQUE,
        size_bytes       INTEGER NOT NULL,
        last_accessed_at INTEGER NOT NULL,
        PRIMARY KEY (file_id, modified_at, max_edge, codec)
    ) STRICT;

    CREATE INDEX idx_thumbnail_cache_lru
        ON thumbnail_cache_entries(last_accessed_at, path);
    "#,
];

/// The schema version the current code migrates databases to.
pub const LATEST_VERSION: i64 = MIGRATIONS.len() as i64;
