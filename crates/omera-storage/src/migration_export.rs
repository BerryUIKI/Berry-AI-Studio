//! Export utility to migrate an existing SQLite library into MySQL 8.0+ or PostgreSQL 14+ schemas.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use omera_domain::{MigrationOptions, MigrationSummary};
use rusqlite::Connection;

use crate::DatabaseError;

/// Escape string for SQL literal insertion.
fn escape_sql(val: &str, is_mysql: bool) -> String {
    if is_mysql {
        let mut out = String::with_capacity(val.len() + 8);
        for c in val.chars() {
            match c {
                '\'' => out.push_str("''"),
                '\\' => out.push_str("\\\\"),
                '\0' => out.push_str("\\0"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                other => out.push(other),
            }
        }
        out
    } else {
        val.replace('\'', "''")
    }
}

/// Format an optional string as `'escaped'` or `NULL`.
fn sql_opt_str(val: Option<&str>, is_mysql: bool) -> String {
    match val {
        Some(s) => format!("'{}'", escape_sql(s, is_mysql)),
        None => "NULL".to_string(),
    }
}

/// Format boolean for dialect: 1/0 for MySQL, TRUE/FALSE for PostgreSQL.
fn sql_bool(val: bool, is_mysql: bool) -> &'static str {
    if is_mysql {
        if val {
            "1"
        } else {
            "0"
        }
    } else if val {
        "TRUE"
    } else {
        "FALSE"
    }
}

/// Export the current SQLite database to a central MySQL or PostgreSQL SQL migration file.
pub fn export_migration_sql(
    conn: &Connection,
    options: &MigrationOptions,
) -> Result<MigrationSummary, DatabaseError> {
    let start_time = Instant::now();
    let dialect_lower = options.target_dialect.trim().to_lowercase();
    let is_mysql = dialect_lower == "mysql";

    let file = File::create(&options.destination).map_err(DatabaseError::Io)?;
    let mut writer = BufWriter::new(file);

    let now_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // --- 1. Header ---
    writeln!(
        writer,
        "-- ========================================================="
    )?;
    writeln!(
        writer,
        "-- Berry AI Studio Central Database Migration Script"
    )?;
    writeln!(
        writer,
        "-- Target Dialect: {}",
        if is_mysql {
            "MySQL 8.0+"
        } else {
            "PostgreSQL 14+"
        }
    )?;
    writeln!(
        writer,
        "-- Target Storage Root UUID: {}",
        options.target_root_uuid
    )?;
    writeln!(writer, "-- Generated At Unix Epoch: {}", now_sec)?;
    writeln!(
        writer,
        "-- ========================================================="
    )?;
    writeln!(writer)?;

    if is_mysql {
        writeln!(writer, "SET FOREIGN_KEY_CHECKS = 0;")?;
        writeln!(writer, "SET NAMES utf8mb4;")?;
        writeln!(writer, "START TRANSACTION;")?;
    } else {
        writeln!(writer, "BEGIN;")?;
        writeln!(writer, "SET client_encoding = 'UTF8';")?;
    }
    writeln!(writer)?;

    // --- 2. DDL Table Schemas ---
    write_ddl(&mut writer, is_mysql)?;

    // --- 3. Storage Roots ---
    let _total_roots = write_storage_roots(conn, &mut writer, options, now_sec, is_mysql)?;

    // --- 4. Albums & Album Files ---
    let total_albums = write_albums(conn, &mut writer, now_sec, is_mysql)?;
    let _total_album_files = write_album_files(conn, &mut writer, now_sec, is_mysql)?;

    // --- 5. Tags & File Tags ---
    let total_tags = write_tags(conn, &mut writer, now_sec, is_mysql)?;
    let total_tag_associations = write_file_tags(conn, &mut writer, is_mysql)?;

    // --- 6. Files & Metadata ---
    let total_files = write_files(conn, &mut writer, options, now_sec, is_mysql)?;

    // --- 7. Initial Change Log Entry ---
    write_initial_change_log(&mut writer, total_files, now_sec, is_mysql)?;

    // --- 8. Commit & Footer ---
    if is_mysql {
        writeln!(writer, "COMMIT;")?;
        writeln!(writer, "SET FOREIGN_KEY_CHECKS = 1;")?;
    } else {
        writeln!(writer, "COMMIT;")?;
    }
    writer.flush().map_err(DatabaseError::Io)?;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    Ok(MigrationSummary {
        success: true,
        target_dialect: if is_mysql {
            "mysql".to_string()
        } else {
            "postgres".to_string()
        },
        total_files,
        total_albums,
        total_tags,
        total_tag_associations,
        output_path: Some(options.destination.clone()),
        error_message: None,
        duration_ms,
    })
}

fn write_ddl<W: Write>(writer: &mut W, is_mysql: bool) -> Result<(), DatabaseError> {
    if is_mysql {
        writeln!(
            writer,
            r#"
CREATE TABLE IF NOT EXISTS `storage_roots` (
    `root_uuid` VARCHAR(36) PRIMARY KEY,
    `display_name` VARCHAR(128) NOT NULL,
    `root_type` VARCHAR(32) NOT NULL DEFAULT 'local_mount',
    `created_at` BIGINT NOT NULL,
    `updated_at` BIGINT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS `files` (
    `id` BIGINT PRIMARY KEY,
    `root_uuid` VARCHAR(36) NOT NULL,
    `relative_path` VARCHAR(1024) NOT NULL,
    `file_name` VARCHAR(255) NOT NULL,
    `file_extension` VARCHAR(16) NOT NULL,
    `size_bytes` BIGINT NOT NULL,
    `modified_at` BIGINT NOT NULL,
    `rating` TINYINT UNSIGNED NULL,
    `aesthetic_score` DOUBLE NULL,
    `is_favorite` TINYINT(1) NOT NULL DEFAULT 0,
    `is_nsfw` TINYINT(1) NOT NULL DEFAULT 0,
    `is_deleted` TINYINT(1) NOT NULL DEFAULT 0,
    `stack_id` VARCHAR(36) NULL,
    `stack_order` INT NOT NULL DEFAULT 0,
    `metadata_format` VARCHAR(32) NULL,
    `metadata_json` JSON NULL,
    `version` BIGINT NOT NULL DEFAULT 1,
    `created_at` BIGINT NOT NULL,
    `updated_at` BIGINT NOT NULL,
    `updated_by` VARCHAR(64) NOT NULL,
    INDEX `idx_files_cursor_mtime` (`is_deleted`, `modified_at` DESC, `id` DESC),
    INDEX `idx_files_cursor_size` (`is_deleted`, `size_bytes` DESC, `id` DESC),
    INDEX `idx_files_cursor_rating` (`is_deleted`, `rating` DESC, `id` DESC),
    INDEX `idx_files_stack` (`stack_id`, `stack_order`),
    INDEX `idx_files_root_rel` (`root_uuid`, `relative_path`(255))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS `albums` (
    `id` BIGINT PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL UNIQUE,
    `description` TEXT NULL,
    `created_at` BIGINT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS `album_files` (
    `album_id` BIGINT NOT NULL,
    `file_id` BIGINT NOT NULL,
    `added_at` BIGINT NOT NULL,
    PRIMARY KEY (`album_id`, `file_id`),
    INDEX `idx_album_files_file` (`file_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS `tags` (
    `id` BIGINT PRIMARY KEY,
    `name` VARCHAR(255) NOT NULL UNIQUE,
    `color` VARCHAR(32) NULL,
    `created_at` BIGINT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS `file_tags` (
    `file_id` BIGINT NOT NULL,
    `tag_id` BIGINT NOT NULL,
    PRIMARY KEY (`file_id`, `tag_id`),
    INDEX `idx_file_tags_tag` (`tag_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

CREATE TABLE IF NOT EXISTS `change_log` (
    `id` BIGINT PRIMARY KEY AUTO_INCREMENT,
    `event_type` VARCHAR(32) NOT NULL,
    `entity_id` BIGINT NOT NULL,
    `secondary_id` VARCHAR(64) NULL,
    `client_id` VARCHAR(64) NOT NULL,
    `payload` JSON NULL,
    `created_at` BIGINT NOT NULL,
    INDEX `idx_change_log_sync` (`id`, `client_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
"#
        )?;
    } else {
        writeln!(
            writer,
            r#"
CREATE TABLE IF NOT EXISTS "storage_roots" (
    "root_uuid" VARCHAR(36) PRIMARY KEY,
    "display_name" VARCHAR(128) NOT NULL,
    "root_type" VARCHAR(32) NOT NULL DEFAULT 'local_mount',
    "created_at" BIGINT NOT NULL,
    "updated_at" BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS "files" (
    "id" BIGINT PRIMARY KEY,
    "root_uuid" VARCHAR(36) NOT NULL,
    "relative_path" VARCHAR(1024) NOT NULL,
    "file_name" VARCHAR(255) NOT NULL,
    "file_extension" VARCHAR(16) NOT NULL,
    "size_bytes" BIGINT NOT NULL,
    "modified_at" BIGINT NOT NULL,
    "rating" SMALLINT NULL,
    "aesthetic_score" DOUBLE PRECISION NULL,
    "is_favorite" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_nsfw" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_deleted" BOOLEAN NOT NULL DEFAULT FALSE,
    "stack_id" VARCHAR(36) NULL,
    "stack_order" INT NOT NULL DEFAULT 0,
    "metadata_format" VARCHAR(32) NULL,
    "metadata_json" JSONB NULL,
    "version" BIGINT NOT NULL DEFAULT 1,
    "created_at" BIGINT NOT NULL,
    "updated_at" BIGINT NOT NULL,
    "updated_by" VARCHAR(64) NOT NULL
);

CREATE INDEX IF NOT EXISTS "idx_files_cursor_mtime" ON "files" ("is_deleted", "modified_at" DESC, "id" DESC);
CREATE INDEX IF NOT EXISTS "idx_files_cursor_size" ON "files" ("is_deleted", "size_bytes" DESC, "id" DESC);
CREATE INDEX IF NOT EXISTS "idx_files_cursor_rating" ON "files" ("is_deleted", "rating" DESC, "id" DESC);
CREATE INDEX IF NOT EXISTS "idx_files_stack" ON "files" ("stack_id", "stack_order");
CREATE INDEX IF NOT EXISTS "idx_files_root_rel" ON "files" ("root_uuid", "relative_path");

CREATE TABLE IF NOT EXISTS "albums" (
    "id" BIGINT PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL UNIQUE,
    "description" TEXT NULL,
    "created_at" BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS "album_files" (
    "album_id" BIGINT NOT NULL,
    "file_id" BIGINT NOT NULL,
    "added_at" BIGINT NOT NULL,
    PRIMARY KEY ("album_id", "file_id")
);
CREATE INDEX IF NOT EXISTS "idx_album_files_file" ON "album_files" ("file_id");

CREATE TABLE IF NOT EXISTS "tags" (
    "id" BIGINT PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL UNIQUE,
    "color" VARCHAR(32) NULL,
    "created_at" BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS "file_tags" (
    "file_id" BIGINT NOT NULL,
    "tag_id" BIGINT NOT NULL,
    PRIMARY KEY ("file_id", "tag_id")
);
CREATE INDEX IF NOT EXISTS "idx_file_tags_tag" ON "file_tags" ("tag_id");

CREATE TABLE IF NOT EXISTS "change_log" (
    "id" BIGSERIAL PRIMARY KEY,
    "event_type" VARCHAR(32) NOT NULL,
    "entity_id" BIGINT NOT NULL,
    "secondary_id" VARCHAR(64) NULL,
    "client_id" VARCHAR(64) NOT NULL,
    "payload" JSONB NULL,
    "created_at" BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS "idx_change_log_sync" ON "change_log" ("id", "client_id");
"#
        )?;
    }
    Ok(())
}

fn write_storage_roots<W: Write>(
    conn: &Connection,
    writer: &mut W,
    options: &MigrationOptions,
    now_sec: i64,
    is_mysql: bool,
) -> Result<u64, DatabaseError> {
    writeln!(writer, "-- Data: Storage Roots")?;
    let mut stmt = conn.prepare(
        "SELECT root_uuid, display_name, root_type, created_at, updated_at FROM storage_roots",
    )?;

    let mut roots: Vec<(String, String, String, i64, i64)> = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    })?;

    for r in rows {
        roots.push(r?);
    }

    let target_uuid = options.target_root_uuid.trim();
    if !target_uuid.is_empty() && !roots.iter().any(|(u, ..)| u == target_uuid) {
        roots.push((
            target_uuid.to_string(),
            "Primary Shared Vault".to_string(),
            "local_mount".to_string(),
            now_sec,
            now_sec,
        ));
    }

    if roots.is_empty() {
        roots.push((
            "default_vault".to_string(),
            "Default Vault".to_string(),
            "local_mount".to_string(),
            now_sec,
            now_sec,
        ));
    }

    let total = roots.len() as u64;
    for chunk in roots.chunks(100) {
        let table = if is_mysql {
            "`storage_roots`"
        } else {
            "\"storage_roots\""
        };
        let mut sql = format!(
            "INSERT INTO {} (root_uuid, display_name, root_type, created_at, updated_at) VALUES\n",
            table
        );
        for (i, (uuid, name, rtype, c_at, u_at)) in chunk.iter().enumerate() {
            if i > 0 {
                sql.push_str(",\n");
            }
            sql.push_str(&format!(
                "('{}', '{}', '{}', {}, {})",
                escape_sql(uuid, is_mysql),
                escape_sql(name, is_mysql),
                escape_sql(rtype, is_mysql),
                c_at,
                u_at
            ));
        }
        if is_mysql {
            sql.push_str(" ON DUPLICATE KEY UPDATE updated_at = VALUES(updated_at);\n");
        } else {
            sql.push_str(
                " ON CONFLICT (root_uuid) DO UPDATE SET updated_at = EXCLUDED.updated_at;\n",
            );
        }
        writer.write_all(sql.as_bytes())?;
    }
    writeln!(writer)?;
    Ok(total)
}

fn write_albums<W: Write>(
    conn: &Connection,
    writer: &mut W,
    now_sec: i64,
    is_mysql: bool,
) -> Result<u64, DatabaseError> {
    writeln!(writer, "-- Data: Albums")?;
    let mut stmt = conn.prepare("SELECT id, name, description FROM albums ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    let mut albums = Vec::new();
    for r in rows {
        albums.push(r?);
    }

    let total = albums.len() as u64;
    for chunk in albums.chunks(100) {
        let table = if is_mysql { "`albums`" } else { "\"albums\"" };
        let mut sql = format!(
            "INSERT INTO {} (id, name, description, created_at) VALUES\n",
            table
        );
        for (i, (id, name, desc)) in chunk.iter().enumerate() {
            if i > 0 {
                sql.push_str(",\n");
            }
            sql.push_str(&format!(
                "({}, '{}', {}, {})",
                id,
                escape_sql(name, is_mysql),
                sql_opt_str(desc.as_deref(), is_mysql),
                now_sec
            ));
        }
        if is_mysql {
            sql.push_str(" ON DUPLICATE KEY UPDATE name = VALUES(name);\n");
        } else {
            sql.push_str(" ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;\n");
        }
        writer.write_all(sql.as_bytes())?;
    }
    writeln!(writer)?;
    Ok(total)
}

fn write_album_files<W: Write>(
    conn: &Connection,
    writer: &mut W,
    now_sec: i64,
    is_mysql: bool,
) -> Result<u64, DatabaseError> {
    writeln!(writer, "-- Data: Album Files")?;
    let mut stmt = conn.prepare("SELECT album_id, file_id FROM album_files")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;

    let mut pairs = Vec::new();
    for r in rows {
        pairs.push(r?);
    }

    let total = pairs.len() as u64;
    for chunk in pairs.chunks(200) {
        let table = if is_mysql {
            "`album_files`"
        } else {
            "\"album_files\""
        };
        let mut sql = format!(
            "INSERT INTO {} (album_id, file_id, added_at) VALUES\n",
            table
        );
        for (i, (a_id, f_id)) in chunk.iter().enumerate() {
            if i > 0 {
                sql.push_str(",\n");
            }
            sql.push_str(&format!("({}, {}, {})", a_id, f_id, now_sec));
        }
        if is_mysql {
            sql.push_str(" ON DUPLICATE KEY UPDATE added_at = VALUES(added_at);\n");
        } else {
            sql.push_str(" ON CONFLICT (album_id, file_id) DO NOTHING;\n");
        }
        writer.write_all(sql.as_bytes())?;
    }
    writeln!(writer)?;
    Ok(total)
}

fn write_tags<W: Write>(
    conn: &Connection,
    writer: &mut W,
    now_sec: i64,
    is_mysql: bool,
) -> Result<u64, DatabaseError> {
    writeln!(writer, "-- Data: Tags")?;
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY id ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    let mut tags = Vec::new();
    for r in rows {
        tags.push(r?);
    }

    let total = tags.len() as u64;
    for chunk in tags.chunks(100) {
        let table = if is_mysql { "`tags`" } else { "\"tags\"" };
        let mut sql = format!(
            "INSERT INTO {} (id, name, color, created_at) VALUES\n",
            table
        );
        for (i, (id, name, color)) in chunk.iter().enumerate() {
            if i > 0 {
                sql.push_str(",\n");
            }
            sql.push_str(&format!(
                "({}, '{}', {}, {})",
                id,
                escape_sql(name, is_mysql),
                sql_opt_str(color.as_deref(), is_mysql),
                now_sec
            ));
        }
        if is_mysql {
            sql.push_str(" ON DUPLICATE KEY UPDATE name = VALUES(name);\n");
        } else {
            sql.push_str(" ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;\n");
        }
        writer.write_all(sql.as_bytes())?;
    }
    writeln!(writer)?;
    Ok(total)
}

fn write_file_tags<W: Write>(
    conn: &Connection,
    writer: &mut W,
    is_mysql: bool,
) -> Result<u64, DatabaseError> {
    writeln!(writer, "-- Data: File Tags")?;
    let mut stmt = conn.prepare("SELECT file_id, tag_id FROM file_tags")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;

    let mut pairs = Vec::new();
    for r in rows {
        pairs.push(r?);
    }

    let total = pairs.len() as u64;
    for chunk in pairs.chunks(200) {
        let table = if is_mysql {
            "`file_tags`"
        } else {
            "\"file_tags\""
        };
        let mut sql = format!("INSERT INTO {} (file_id, tag_id) VALUES\n", table);
        for (i, (f_id, t_id)) in chunk.iter().enumerate() {
            if i > 0 {
                sql.push_str(",\n");
            }
            sql.push_str(&format!("({}, {})", f_id, t_id));
        }
        if is_mysql {
            sql.push_str(" ON DUPLICATE KEY UPDATE tag_id = VALUES(tag_id);\n");
        } else {
            sql.push_str(" ON CONFLICT (file_id, tag_id) DO NOTHING;\n");
        }
        writer.write_all(sql.as_bytes())?;
    }
    writeln!(writer)?;
    Ok(total)
}

#[derive(Debug)]
struct FileRow {
    id: i64,
    path: String,
    container: String,
    size_bytes: i64,
    modified_at: i64,
    rating: Option<i64>,
    aesthetic_score: Option<f64>,
    is_favorite: bool,
    is_nsfw: bool,
    stack_id: Option<String>,
    stack_order: i64,
    metadata: Option<String>,
    version: i64,
}

fn write_files<W: Write>(
    conn: &Connection,
    writer: &mut W,
    options: &MigrationOptions,
    now_sec: i64,
    is_mysql: bool,
) -> Result<u64, DatabaseError> {
    writeln!(writer, "-- Data: Files")?;

    let target_root = if options.target_root_uuid.trim().is_empty() {
        "default_vault"
    } else {
        options.target_root_uuid.trim()
    };

    let mut stmt = conn.prepare(
        "SELECT id, path, container, size_bytes, modified_at, rating, aesthetic_score,
                is_favorite, is_nsfw, stack_id, stack_order, metadata, version
         FROM files
         ORDER BY id ASC",
    )?;

    let file_iter = stmt.query_map([], |row| {
        Ok(FileRow {
            id: row.get(0)?,
            path: row.get(1)?,
            container: row.get(2)?,
            size_bytes: row.get(3)?,
            modified_at: row.get(4)?,
            rating: row.get(5)?,
            aesthetic_score: row.get(6)?,
            is_favorite: row.get::<_, i64>(7)? != 0,
            is_nsfw: row.get::<_, i64>(8)? != 0,
            stack_id: row.get(9)?,
            stack_order: row.get(10)?,
            metadata: row.get(11)?,
            version: row.get::<_, Option<i64>>(12)?.unwrap_or(1),
        })
    })?;

    let mut total: u64 = 0;
    let mut batch = Vec::with_capacity(100);

    for item in file_iter {
        let row = item?;
        batch.push(row);
        if batch.len() >= 100 {
            flush_files_batch(writer, &batch, target_root, now_sec, is_mysql)?;
            total += batch.len() as u64;
            batch.clear();
        }
    }

    if !batch.is_empty() {
        flush_files_batch(writer, &batch, target_root, now_sec, is_mysql)?;
        total += batch.len() as u64;
        batch.clear();
    }

    writeln!(writer)?;
    Ok(total)
}

fn flush_files_batch<W: Write>(
    writer: &mut W,
    batch: &[FileRow],
    target_root: &str,
    now_sec: i64,
    is_mysql: bool,
) -> Result<(), DatabaseError> {
    let table = if is_mysql { "`files`" } else { "\"files\"" };
    let mut sql = format!(
        "INSERT INTO {} (
    id, root_uuid, relative_path, file_name, file_extension, size_bytes, modified_at,
    rating, aesthetic_score, is_favorite, is_nsfw, is_deleted, stack_id, stack_order,
    metadata_format, metadata_json, version, created_at, updated_at, updated_by
) VALUES\n",
        table
    );

    for (i, row) in batch.iter().enumerate() {
        if i > 0 {
            sql.push_str(",\n");
        }

        let p = Path::new(&row.path);
        let file_name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed_file");

        let ext = p
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or(&row.container);

        // Normalize relative path forward-slashes
        let clean_path = row.path.replace('\\', "/");
        let rel_path = clean_path
            .trim_start_matches(|c: char| c.is_alphabetic() || c == ':' || c == '/')
            .to_string();

        let rating_str = match row.rating {
            Some(r) => r.to_string(),
            None => "NULL".to_string(),
        };

        let score_str = match row.aesthetic_score {
            Some(s) => format!("{:.4}", s),
            None => "NULL".to_string(),
        };

        let meta_json_str = match &row.metadata {
            Some(json) => {
                if is_mysql {
                    format!("'{}'", escape_sql(json, is_mysql))
                } else {
                    format!("'{}'::jsonb", escape_sql(json, is_mysql))
                }
            }
            None => "NULL".to_string(),
        };

        let format_str = match &row.metadata {
            Some(_) => "'auto_extracted'".to_string(),
            None => "NULL".to_string(),
        };

        sql.push_str(&format!(
            "({}, '{}', '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, '{}')",
            row.id,
            escape_sql(target_root, is_mysql),
            escape_sql(&rel_path, is_mysql),
            escape_sql(file_name, is_mysql),
            escape_sql(ext, is_mysql),
            row.size_bytes,
            row.modified_at,
            rating_str,
            score_str,
            sql_bool(row.is_favorite, is_mysql),
            sql_bool(row.is_nsfw, is_mysql),
            sql_bool(false, is_mysql), // is_deleted
            sql_opt_str(row.stack_id.as_deref(), is_mysql),
            row.stack_order,
            format_str,
            meta_json_str,
            row.version,
            row.modified_at, // created_at
            now_sec,         // updated_at
            "migration_wizard"
        ));
    }

    if is_mysql {
        sql.push_str(" ON DUPLICATE KEY UPDATE updated_at = VALUES(updated_at);\n");
    } else {
        sql.push_str(" ON CONFLICT (id) DO UPDATE SET updated_at = EXCLUDED.updated_at;\n");
    }

    writer.write_all(sql.as_bytes())?;
    Ok(())
}

fn write_initial_change_log<W: Write>(
    writer: &mut W,
    total_files: u64,
    now_sec: i64,
    is_mysql: bool,
) -> Result<(), DatabaseError> {
    writeln!(writer, "-- Data: Initial Migration Journal Entry")?;
    let table = if is_mysql {
        "`change_log`"
    } else {
        "\"change_log\""
    };
    let payload = format!(r#"{{"migrated_assets": {total_files}}}"#);
    let payload_val = if is_mysql {
        format!("'{}'", escape_sql(&payload, is_mysql))
    } else {
        format!("'{}'::jsonb", escape_sql(&payload, is_mysql))
    };

    let sql = format!(
        "INSERT INTO {} (event_type, entity_id, secondary_id, client_id, payload, created_at)
         VALUES ('migration.completed', {}, NULL, 'migration_wizard', {}, {});\n\n",
        table, total_files, payload_val, now_sec
    );
    writer.write_all(sql.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_escape_sql() {
        assert_eq!(escape_sql("O'Reilly", false), "O''Reilly");
        assert_eq!(escape_sql("O'Reilly\\Test", true), "O''Reilly\\\\Test");
    }

    #[test]
    fn test_export_migration_sql_mysql_and_postgres() {
        let conn = Connection::open_in_memory().unwrap();
        // Init schema with v1..v14 migrations
        crate::migrations::MIGRATIONS.iter().for_each(|m| {
            conn.execute_batch(m).unwrap();
        });

        // Seed some data
        conn.execute("INSERT INTO folders (id, path) VALUES (1, 'C:\\Vault')", [])
            .unwrap();
        conn.execute(
            "INSERT INTO files (id, folder_id, path, container, size_bytes, modified_at, rating, is_favorite, is_nsfw, version)
             VALUES (101, 1, 'C:\\Vault\\hero.png', 'png', 1048576, 1726000000, 5, 1, 0, 3)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tags (id, name, color) VALUES (1, 'Cyberpunk', '#38bdf8')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO file_tags (file_id, tag_id) VALUES (101, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO albums (id, name, description) VALUES (1, 'Best Of 2026', 'Curated assets')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO album_files (album_id, file_id) VALUES (1, 101)",
            [],
        )
        .unwrap();

        // 1. Test MySQL export
        let tmp_mysql = NamedTempFile::new().unwrap();
        let mysql_dest = tmp_mysql.path().to_str().unwrap().to_string();
        let summary_mysql = export_migration_sql(
            &conn,
            &MigrationOptions {
                target_dialect: "mysql".to_string(),
                target_root_uuid: "nas_vault_01".to_string(),
                destination: mysql_dest.clone(),
            },
        )
        .unwrap();

        assert!(summary_mysql.success);
        assert_eq!(summary_mysql.target_dialect, "mysql");
        assert_eq!(summary_mysql.total_files, 1);
        assert_eq!(summary_mysql.total_albums, 1);
        assert_eq!(summary_mysql.total_tags, 1);
        assert_eq!(summary_mysql.total_tag_associations, 1);

        let mysql_content = std::fs::read_to_string(&mysql_dest).unwrap();
        assert!(mysql_content.contains("SET FOREIGN_KEY_CHECKS = 0;"));
        assert!(mysql_content.contains("CREATE TABLE IF NOT EXISTS `files`"));
        assert!(mysql_content.contains("nas_vault_01"));
        assert!(mysql_content.contains("Cyberpunk"));
        assert!(mysql_content.contains("Best Of 2026"));

        // 2. Test PostgreSQL export
        let tmp_pg = NamedTempFile::new().unwrap();
        let pg_dest = tmp_pg.path().to_str().unwrap().to_string();
        let summary_pg = export_migration_sql(
            &conn,
            &MigrationOptions {
                target_dialect: "postgres".to_string(),
                target_root_uuid: "nas_vault_01".to_string(),
                destination: pg_dest.clone(),
            },
        )
        .unwrap();

        assert!(summary_pg.success);
        assert_eq!(summary_pg.target_dialect, "postgres");
        assert_eq!(summary_pg.total_files, 1);
        assert_eq!(summary_pg.total_albums, 1);
        assert_eq!(summary_pg.total_tags, 1);
        assert_eq!(summary_pg.total_tag_associations, 1);

        let pg_content = std::fs::read_to_string(&pg_dest).unwrap();
        assert!(pg_content.contains("BEGIN;"));
        assert!(pg_content.contains("CREATE TABLE IF NOT EXISTS \"files\""));
        assert!(pg_content.contains("nas_vault_01"));
    }
}
