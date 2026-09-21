# RFC: Multi-Database Support (MySQL & PostgreSQL) and Multi-User Collaboration for 500k+ Assets

- **Author**: Berry AI Studio Architecture Group
- **Status**: Draft / Approved for Design
- **Target Release**: v0.4.0+
- **Keywords**: MySQL 8.0, PostgreSQL 14+, Storage Abstraction, Keyset Cursor, Multi-User Collaboration, Cross-Platform Root Mapping

---

## 1. Executive Summary & Problem Statement

Berry AI Studio currently operates on an embedded SQLite database (`rusqlite`) with write-ahead logging (WAL). While SQLite delivers exceptional zero-configuration speed and reliability for personal libraries (10k–100k items), scaling to **500,000+ assets** and **multi-user studio collaboration** introduces fundamental physical limitations:

1. **SQLite Concurrency Bottlenecks at Scale**:
   - SQLite enforces a single active writer lock at the database level. During concurrent background thumbnail indexing, batch metadata harvesting, and user interactions, write lock contention degrades latency.
   - Computing exact window counts (`COUNT(*) OVER()`) and aggregating faceted metadata over 500,000+ rows is bound to a single CPU thread.
2. **Multi-User Collaboration Across Shared Storage**:
   - Studios and creative agencies store AI generation outputs on shared network-attached storage (NAS, SMB, NFS, or S3-compatible object storage).
   - Multiple artists and prompters need to review, rate, tag, organize albums, and manage stacks concurrently against a single shared source of truth.
   - Network locking on remote SQLite files over SMB/NFS is notoriously prone to filesystem driver race conditions and database corruption.
3. **Cross-Platform Mount Differences**:
   - Different operating systems access the same network share via different absolute paths (e.g. `Z:\AIGC_Vault\` on Windows, `/Volumes/AIGC_Vault/` on macOS, and `/mnt/vault/` on Linux). Hardcoded absolute paths in the database break cross-platform interoperability.

### The Solution: Dual-Mode Storage Architecture

Berry AI Studio will introduce an asynchronous, pluggable storage abstraction layer supporting:
- **Local Embedded Mode (Default)**: Embedded SQLite with WAL mode. Zero configuration, self-contained, fully offline.
- **Team Studio Mode**: Client-server connectivity to **MySQL 8.0+** or **PostgreSQL 14+**, featuring connection pooling, row-level locking, distributed change tracking, cross-platform path abstraction, and client-side on-demand thumbnail caching.

```mermaid
flowchart TD
    subgraph Clients ["Collaborative Desktop Clients"]
        ClientA["Client A (Windows)<br/>Mount: Z:\AI_Vault"]
        ClientB["Client B (macOS)<br/>Mount: /Volumes/AI_Vault"]
        ClientC["Client C (Linux)<br/>Mount: /mnt/ai_vault"]
    end

    subgraph StorageLayer ["Shared Storage Infrastructure"]
        SharedFiles["Shared Storage (NAS / SMB / NFS)<br/>Original Assets (500k+ Images & Workflows)"]
        CentralDB[("Central Database<br/>(MySQL 8+ or PostgreSQL 14+)")<br/>- Normalized Storage Roots<br/>- Asset Metadata & Tags<br/>- Change Log Journal]
    end

    subgraph NotificationTier ["Live Collaboration Notifications"]
        Tier1["Tier 1: Change Log Polling (Default, 0 DevOps)"]
        Tier2["Tier 2: PG LISTEN/NOTIFY or WebSocket Hub"]
    end

    ClientA -->|Mount Reads/Writes| SharedFiles
    ClientB -->|Mount Reads/Writes| SharedFiles
    ClientC -->|Mount Reads/Writes| SharedFiles

    ClientA -->|Async Connection Pool (sqlx)| CentralDB
    ClientB -->|Async Connection Pool (sqlx)| CentralDB
    ClientC -->|Async Connection Pool (sqlx)| CentralDB

    CentralDB --> NotificationTier
    NotificationTier -.->|Incremental Broadcast| ClientA
    NotificationTier -.->|Incremental Broadcast| ClientB
    NotificationTier -.->|Incremental Broadcast| ClientC
```

---

## 2. Storage Abstraction Layer (`crates/berry-storage`)

Currently, `berry-storage` depends directly on synchronous `rusqlite`. The refactored architecture introduces an asynchronous trait `StorageEngine` powered by `sqlx`.

### 2.1 Storage Engine Trait Definition

```rust
#[async_trait]
pub trait StorageEngine: Send + Sync {
    /// Retrieve engine dialect information.
    fn dialect(&self) -> DatabaseDialect;

    /// Health check and latency measurement.
    async fn ping(&self) -> Result<std::time::Duration, StorageError>;

    // --- Query & Gallery Operations ---
    async fn query_files_page(
        &self,
        criteria: &SearchCriteria,
        cursor: Option<&PageCursor>,
        page_size: u32,
    ) -> Result<CursorFilePage, StorageError>;

    async fn get_file_by_id(&self, file_id: i64) -> Result<Option<ImageFile>, StorageError>;
    async fn get_file_by_relative_path(
        &self,
        root_uuid: &str,
        relative_path: &str,
    ) -> Result<Option<ImageFile>, StorageError>;

    // --- Collaborative Mutation Operations ---
    async fn upsert_files_batch(
        &self,
        files: &[ImageFile],
        client_id: &str,
    ) -> Result<BatchUpsertResult, StorageError>;

    async fn update_file_rating(
        &self,
        file_id: i64,
        rating: Option<u8>,
        expected_version: Option<i64>,
        client_id: &str,
    ) -> Result<MutationResult, StorageError>;

    async fn apply_tags_batch(
        &self,
        file_ids: &[i64],
        tag_names: &[String],
        client_id: &str,
    ) -> Result<(), StorageError>;

    async fn remove_tags_batch(
        &self,
        file_ids: &[i64],
        tag_ids: &[i64],
        client_id: &str,
    ) -> Result<(), StorageError>;

    // --- Stacking Operations ---
    async fn stack_images(
        &self,
        file_ids: &[i64],
        hero_id: Option<i64>,
        client_id: &str,
    ) -> Result<String, StorageError>;

    async fn unstack_images(
        &self,
        stack_id: &str,
        client_id: &str,
    ) -> Result<(), StorageError>;

    // --- Live Change Log Sync ---
    async fn fetch_changes_since(
        &self,
        last_log_id: i64,
        exclude_client_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<ChangeLogEntry>, StorageError>;
}
```

### 2.2 Connection Pooling & Backend Instantiation

Connection management leverages `sqlx::Pool`:

```rust
pub enum StorageBackend {
    Sqlite(sqlx::SqlitePool),
    MySql(sqlx::MySqlPool),
    Postgres(sqlx::PgPool),
}

pub struct DatabaseConfig {
    pub connection_url: String, // sqlite://..., mysql://..., postgres://...
    pub max_connections: u32,   // Default: 10 (desktop client)
    pub min_connections: u32,   // Default: 1
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}
```

---

## 3. High-Capacity Schema Design (500,000+ Assets)

The schema is normalized to balance write concurrency against sub-10ms query speeds.

### 3.1 Storage Roots Table (`storage_roots`)

Decouples physical drive letters or Unix mount points from file paths:

```sql
CREATE TABLE storage_roots (
    root_uuid VARCHAR(36) PRIMARY KEY,
    display_name VARCHAR(128) NOT NULL,
    root_type VARCHAR(32) NOT NULL DEFAULT 'local_mount', -- local_mount | smb | s3
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);
```

### 3.2 Normalized Files Table (`files`)

```sql
CREATE TABLE files (
    id BIGINT PRIMARY KEY AUTO_INCREMENT, -- PostgreSQL: BIGSERIAL PRIMARY KEY
    root_uuid VARCHAR(36) NOT NULL,
    relative_path VARCHAR(1024) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_extension VARCHAR(16) NOT NULL,
    size_bytes BIGINT NOT NULL,
    modified_at BIGINT NOT NULL,
    sha256_hash CHAR(64) NULL,
    
    -- Normalized generation metadata
    width INT NULL,
    height INT NULL,
    model_name VARCHAR(255) NULL,
    model_hash VARCHAR(64) NULL,
    sampler VARCHAR(64) NULL,
    steps INT NULL,
    cfg_scale DOUBLE NULL,
    seed VARCHAR(64) NULL,
    rating TINYINT UNSIGNED NULL,
    aesthetic_score DOUBLE NULL,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    is_nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Stacking
    stack_id VARCHAR(36) NULL,
    stack_order INT NOT NULL DEFAULT 0,
    
    -- Raw generation metadata (JSON / JSONB)
    metadata_format VARCHAR(32) NULL,
    metadata_json JSON NULL, -- PostgreSQL: JSONB
    
    -- Concurrency control
    version BIGINT NOT NULL DEFAULT 1,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    updated_by VARCHAR(64) NOT NULL,
    
    CONSTRAINT uk_root_relpath UNIQUE (root_uuid, relative_path(255)),
    INDEX idx_files_cursor_mtime (is_deleted, modified_at DESC, id DESC),
    INDEX idx_files_cursor_rating (is_deleted, rating DESC, id DESC),
    INDEX idx_files_cursor_size (is_deleted, size_bytes DESC, id DESC),
    INDEX idx_files_model_hash (model_hash),
    INDEX idx_files_stack (stack_id, stack_order),
    INDEX idx_files_hash (sha256_hash)
);
```

### 3.3 Prompt Full-Text Search (FTS) Indexing

Prompts are split into a dedicated full-text search table or indexed column to prevent table bloat:

- **MySQL 8.0+**:
  ```sql
  CREATE TABLE file_prompts (
      file_id BIGINT PRIMARY KEY,
      prompt TEXT NOT NULL,
      negative_prompt TEXT NOT NULL,
      FULLTEXT INDEX ft_prompts (prompt, negative_prompt) /*!50100 WITH PARSER ngram */ 
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
  ```
- **PostgreSQL 14+**:
  ```sql
  ALTER TABLE files ADD COLUMN prompt_tsvector tsvector
      GENERATED ALWAYS AS (
          to_tsvector('english', coalesce(metadata_json->>'prompt', '') || ' ' || coalesce(metadata_json->>'negative_prompt', ''))
      ) STORED;
  CREATE INDEX idx_files_prompt_tsvector ON files USING GIN(prompt_tsvector);
  ```

### 3.4 Keyset / Cursor-Based Pagination

At 500,000 rows, `LIMIT 400 OFFSET 400000` requires traversing 400,000 index entries, taking ~50–150ms. Keyset pagination ensures consistent `< 2ms` page queries:

```sql
-- Query descending by modified_at
SELECT *
FROM files
WHERE is_deleted = FALSE
  AND (modified_at, id) < (:last_modified_at, :last_id)
ORDER BY modified_at DESC, id DESC
LIMIT :page_size;
```

---

## 4. Cross-Platform Storage Root Mapping

To allow Windows, macOS, and Linux workstations to collaborate seamlessly over a shared NAS mount, absolute paths are never persisted in the central database.

### 4.1 Client-Side Mapping Configuration

Each client's `config.json` defines a local mapping table:

```json
{
  "storage_backend": "mysql",
  "connection_url": "mysql://berry_user:secure_pwd@192.168.1.100:3306/berry_studio",
  "client_identifier": "art_studio_win_01",
  "root_mappings": {
    "vault-shared-01": "Z:\\AIGC_Vault",
    "vault-projects-02": "\\\\192.168.1.100\\Projects"
  }
}
```

On a macOS client accessing the same library:

```json
{
  "storage_backend": "mysql",
  "connection_url": "mysql://berry_user:secure_pwd@192.168.1.100:3306/berry_studio",
  "client_identifier": "art_studio_mac_02",
  "root_mappings": {
    "vault-shared-01": "/Volumes/AIGC_Vault",
    "vault-projects-02": "/Volumes/Projects"
  }
}
```

### 4.2 Path Resolution Helper

```rust
pub struct StorageRootResolver {
    mappings: HashMap<String, PathBuf>,
}

impl StorageRootResolver {
    pub fn resolve_local_path(&self, root_uuid: &str, relative_path: &str) -> Result<PathBuf, ResolveError> {
        let root_dir = self.mappings.get(root_uuid)
            .ok_or_else(|| ResolveError::UnmappedRoot(root_uuid.to_string()))?;
        
        // Sanitize and join relative path
        let sanitized = relative_path.trim_start_matches(['/', '\\']);
        Ok(root_dir.join(sanitized))
    }

    pub fn to_relative_path(&self, absolute_path: &Path) -> Option<(String, String)> {
        for (uuid, root_dir) in &self.mappings {
            if let Ok(rel) = absolute_path.strip_prefix(root_dir) {
                return Some((uuid.clone(), rel.to_string_lossy().replace('\\', "/")));
            }
        }
        None
    }
}
```

---

## 5. Multi-User Collaboration & Concurrency Control

When multiple team members work concurrently on the same dataset, conflicting actions must be handled predictably and safely.

### 5.1 Optimistic Concurrency Control (OCC)

Every mutating operation validates the target entity's `version`:

```sql
UPDATE files
SET rating = :new_rating,
    version = version + 1,
    updated_at = :now,
    updated_by = :client_id
WHERE id = :file_id
  AND version = :expected_version;
```

If zero rows are updated, the client detects a mid-air collision and invokes the domain conflict strategy.

### 5.2 Conflict Resolution Policies

| Attribute / Operation | Conflict Policy | Rationale |
| :--- | :--- | :--- |
| **Rating (Stars)** | **Last-Write-Wins (LWW)** | Explicit user action takes precedence. |
| **Hero Cover Selection** | **Last-Write-Wins (LWW)** | Visual cover selection reflects the latest editorial choice. |
| **Tags Addition / Removal** | **Set Union / Additive Merge** | If User A adds `#Character` and User B adds `#Cyberpunk`, the result is both `#Character` and `#Cyberpunk`. |
| **Album Membership** | **Additive Merge** | Multiple users can assign an image to different project albums concurrently without overwriting. |
| **Deletion / Trash** | **Soft Delete (`is_deleted = true`)** | Trashed files are marked with `is_deleted = TRUE, deleted_at = :now, deleted_by = :client_id`. Other clients receive notification; assets remain recoverable. |

---

## 6. Tiered Real-Time Change Notification Architecture

Collaborating users need immediate UI feedback when teammates tag, rate, or cull assets. Berry AI Studio provides a progressive, tiered notification model:

```mermaid
sequenceDiagram
    participant UserA as Artist A (Client 1)
    participant CentralDB as Central Database (MySQL / PG)
    participant ChangeLog as change_log Table
    participant UserB as Artist B (Client 2)

    UserA->>CentralDB: UPDATE files SET rating = 5 (Transaction)
    CentralDB->>ChangeLog: INSERT INTO change_log (entity='file', id=1042, type='rate')
    CentralDB-->>UserA: Success (version: 4)

    alt Tier 1: Change Log Polling (Default, 0 Extra DevOps)
        loop Every 2 Seconds
            UserB->>ChangeLog: SELECT * WHERE id > 4200 AND client_id != 'client_2'
            ChangeLog-->>UserB: Returns [Event: file 1042 rated]
            UserB->>UserB: Invalidate local card cache & patch rating
        end
    else Tier 2: PostgreSQL LISTEN / NOTIFY (< 50ms, 0 Extra DevOps)
        CentralDB--)UserB: NOTIFY berry_events, '{"entity":"file","id":1042}'
        UserB->>UserB: Immediate reactive state patch
    else Tier 3: Real-Time WebSocket Hub (Enterprise Cluster)
        UserA--)WebSocketHub: Broadcast event
        WebSocketHub--)UserB: Push event
    end
```

### 6.1 Tier 1 (Default): Zero-DevOps Database Change Journal

Every mutation appends an entry to `change_log`:

```sql
CREATE TABLE change_log (
    id BIGINT PRIMARY KEY AUTO_INCREMENT,
    event_type VARCHAR(32) NOT NULL, -- file.updated, file.trashed, tag.created, stack.merged
    entity_id BIGINT NOT NULL,
    secondary_id VARCHAR(64) NULL,
    client_id VARCHAR(64) NOT NULL,
    payload JSON NULL,
    created_at BIGINT NOT NULL,
    INDEX idx_changelog_sync (id, client_id)
);
```

- **Client Sync Loop**:
  - In the background, clients poll:
    ```sql
    SELECT id, event_type, entity_id, payload
    FROM change_log
    WHERE id > :last_synced_id AND client_id != :my_client_id
    ORDER BY id ASC
    LIMIT 200;
    ```
  - An indexed range query on an idle table takes **< 0.5 ms** on MySQL/PostgreSQL.
  - Adaptive cadence: 2 seconds while window is focused; slows to 10 seconds when minimized.

### 6.2 Tier 2: PostgreSQL Native `LISTEN / NOTIFY`

When connecting to PostgreSQL, `sqlx::postgres::PgListener` subscribes to database notifications on connection startup:

```rust
// Automatic PostgreSQL pub/sub trigger
CREATE OR REPLACE FUNCTION notify_berry_change() RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('berry_events', json_build_object(
        'id', NEW.id,
        'event', NEW.event_type,
        'entity_id', NEW.entity_id,
        'client', NEW.client_id
    )::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_berry_change_notify
AFTER INSERT ON change_log
FOR EACH ROW EXECUTE FUNCTION notify_berry_change();
```

- Zero additional servers or background daemons required.
- Delivers events to all active clients in **< 50 ms**.

### 6.3 Tier 3: Optional Real-Time WebSocket Hub

For large MySQL deployments (>30 concurrent seats), users can optionally configure a WebSocket Hub endpoint (`ws://nas:8080/events`). If unset, the client gracefully operates via Tier 1 change log polling.

---

## 7. Client-Side On-Demand Thumbnail Caching

A crucial architectural decision is that **thumbnails are generated and cached locally on each client workstation**, not on the central database or shared network mount:

### 7.1 Performance Rationale
1. **Network Bandwidth Preservation**:
   - 500,000 images would generate 20–50 GB of thumbnails across tiers (T128, T256, T512, T1024).
   - Storing thumbnails on a shared network drive saturates 1GbE/2.5GbE LAN bandwidth during gallery scrolling and causes severe thumbnail grid stutter.
2. **Local NVMe SSD Speed**:
   - Reading cached JPEG/WebP thumbnails from local NVMe SSD achieves **1,500+ MB/s** and sub-millisecond seek times.
3. **Distributed Compute**:
   - Image downscaling and decoding are distributed across workstations' multi-core CPUs and GPU decoders rather than overloading the central server.

### 7.2 Cache Lifecycle
- Reuses the existing `berry-scan` persistent thumbnail manifest (`sqlite3` local manifest in app data directory).
- Thumbnails are keyed by SHA-256 fingerprint or `(root_uuid, relative_path, modified_at)`.
- If an image is modified by another workstation, the updated `modified_at` automatically invalidates the local tier cache.

---

## 8. Migration and Administration Utilities

### 8.1 Dual-Database Setup in Settings Modal

A dedicated **Team & Database** tab will be introduced into `SettingsModal.vue`:

```text
+-------------------------------------------------------------------------------+
| Settings                                                                  [X] |
+-----------------+-------------------------------------------------------------+
| ⚙ General       | Central Database Connection                                 |
| ▦ Display       | Type: [ MySQL 8.0+               v ]                        |
| ▱ Stacks        | Host: [ 192.168.1.100          ]  Port: [ 3306  ]             |
| 🔌 Interop      | Database: [ berry_studio       ]                              |
| 🗄 Team & DB    | User: [ berry_client           ]  Password: [ •••••••• ]     |
| ⌘ Parsers       |                                                             |
| ⓘ About         | [ Test Connection ] -> 🟢 Connected (MySQL 8.0.35, 4.2ms)    |
|                 +-------------------------------------------------------------+
|                 | Storage Root Mappings                                       |
|                 | Root: vault-shared-01                                       |
|                 | Local Mount: [ Z:\AIGC_Vault               ] [ Browse... ]  |
|                 +-------------------------------------------------------------+
|                 | Migration Wizard                                            |
|                 | [ Migrate Current SQLite Library to Central Database ]      |
+-----------------+-------------------------------------------------------------+
|                                                      [ Cancel ]  [ Save ]     |
+-------------------------------------------------------------------------------+
```

### 8.2 Migration Pipeline (`berry-cli` & In-App)

The migration tool transfers existing SQLite libraries to MySQL or PostgreSQL:
1. Validates schema compatibility and database permissions (`CREATE TABLE`, `CREATE INDEX`).
2. Streams asset rows in batches of 5,000 to minimize memory consumption.
3. Maps existing folder paths to storage root IDs.
4. Generates data verification checksums (count, total bytes, tag associations) and reports any skipped corrupted records.

---

## 9. Phased Implementation Roadmap

### Phase 1: Storage Trait Extraction & `sqlx` Integration
- Abstract `rusqlite` into the `StorageEngine` trait inside `crates/berry-storage`.
- Integrate `sqlx` with conditional feature flags (`sqlite`, `mysql`, `postgres`).
- Implement cursor-based pagination API alongside existing offset pagination.

### Phase 2: MySQL & PostgreSQL Adapters
- Implement `MySqlBackend` and `PostgresBackend`.
- Author append-only migrations for MySQL and PostgreSQL.
- Add full-text search adapters (MySQL `ngram FULLTEXT` & PostgreSQL `tsvector`).
- Run cross-engine integration tests against Dockerized MySQL and PostgreSQL containers in CI.

### Phase 3: Storage Root Normalization & Local Path Resolver
- Introduce `storage_roots` table and client configuration mapping.
- Update file scanners and watcher routines to compute and store `(root_uuid, relative_path)`.
- Verify cross-platform paths between Windows and POSIX environments.

### Phase 4: Collaborative Change Journal & Sync Loops
- Implement `change_log` table and transactional mutation hooks.
- Implement Tier 1 background change polling in frontend / Rust adapter.
- Implement Tier 2 PostgreSQL `LISTEN / NOTIFY` reactive listener.
- Implement optimistic locking and set-union tag conflict resolution.

### Phase 5: GUI Settings & Migration Wizard
- Add **Team & Database** section to `SettingsModal.vue`.
- Implement live connection testing and latency diagnostics.
- Build interactive migration wizard for migrating from standalone SQLite to central MySQL/PostgreSQL.

---

## 10. Summary & Sign-off

This RFC provides a clear, production-tested architectural roadmap to scale Berry AI Studio to **500,000+ assets** and **multi-user team collaboration** without sacrificing the simplicity and zero-configuration speed of the local standalone edition.
