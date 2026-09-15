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
];

/// The schema version the current code migrates databases to.
pub const LATEST_VERSION: i64 = MIGRATIONS.len() as i64;
