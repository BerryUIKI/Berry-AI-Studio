//! SQLite persistence with schema versioning for Omera.
//!
//! Owns the SQLite connection and applies an ordered list of embedded
//! migrations tracked by SQLite's `PRAGMA user_version`. All schema changes
//! go through `migrations`, never through ad-hoc DDL.

#![allow(unknown_lints)]
#![allow(clippy::chunks_exact_to_as_chunks)]

mod db;
mod engine;
pub mod legacy_migration;
mod migration_export;
mod migrations;
pub mod recovery;

pub use db::{Database, DatabaseError, ThumbnailCacheEntry};
pub use engine::{DatabaseDialect, StorageEngine};
