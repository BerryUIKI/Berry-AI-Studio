//! Database engine abstraction and dialect definitions for multi-backend storage.
//!
//! Supports local zero-config SQLite as well as enterprise/team backends
//! (MySQL 8.0+ and PostgreSQL 14+).

use omera_domain::{CursorFilePage, SearchCriteria};
use serde::{Deserialize, Serialize};

use crate::db::DatabaseError;

/// Supported database dialects for Berry AI Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseDialect {
    Sqlite,
    MySql,
    Postgres,
}

impl DatabaseDialect {
    /// Returns the human-readable engine name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::MySql => "MySQL",
            Self::Postgres => "PostgreSQL",
        }
    }

    /// Returns the parameter placeholder for a 1-based parameter index.
    ///
    /// SQLite and MySQL use `?`, while PostgreSQL uses `$1`, `$2`, etc.
    pub fn parameter_placeholder(&self, index: usize) -> String {
        match self {
            Self::Sqlite | Self::MySql => "?".to_string(),
            Self::Postgres => format!("${index}"),
        }
    }

    /// Whether this dialect supports JSON functions natively.
    pub fn supports_native_json(&self) -> bool {
        true
    }
}

/// Abstract storage engine interface for Berry AI Studio.
pub trait StorageEngine: Send {
    /// Returns the database dialect of this storage backend.
    fn dialect(&self) -> DatabaseDialect;

    /// Execute a keyset cursor paginated query with full metadata.
    fn search_files_cursor_page(
        &self,
        criteria: &SearchCriteria,
    ) -> Result<CursorFilePage, DatabaseError>;

    /// Execute a keyset cursor paginated query optimized for gallery cards (omits raw prompt bloat).
    fn search_gallery_files_cursor_page(
        &self,
        criteria: &SearchCriteria,
    ) -> Result<CursorFilePage, DatabaseError>;
}
