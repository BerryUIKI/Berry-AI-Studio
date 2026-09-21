//! Core domain types shared across Berry AI Studio crates.
//!
//! These types describe the *problem domain* — AI-generated image files and
//! their embedded metadata — independent of any storage or extraction concern.
//! Crates may depend on `berry-domain`, but it depends on nothing else.

mod album;
mod checkpoint;
mod cloud_backup;
mod collaboration;
mod database_stats;
mod export;
mod extracted_metadata;
mod filesystem_change;
mod folder;
mod image_file;
mod lora;
mod metadata_format;
mod pipeline;
mod prompt_stacking;
mod prompt_stat;
mod search;
mod search_parser;
mod similarity;
mod stack;
mod storage_root;
mod tag;

pub use album::Album;
pub use checkpoint::{CheckpointModelStat, ModelCacheEntry};
pub use cloud_backup::{
    CloudBackupConfig, CloudBackupResult, CloudPingResult, CloudRestoreResult, CloudSnapshotMeta,
    CloudStorageProvider,
};
pub use collaboration::{
    ChangeLogEntry, ChangeLogSyncQuery, DatabasePingResult, MigrationOptions, MigrationSummary,
    MutationResult,
};
pub use database_stats::DatabaseStats;
pub use export::{
    ExportFormat, ExportOptions, ExportProgressEvent, ExportSidecar, ExportSummary,
    MetadataPrivacyMode,
};
pub use extracted_metadata::ExtractedMetadata;
pub use filesystem_change::FilesystemChange;
pub use folder::Folder;
pub use image_file::{Container, FileSortField, ImageFile, SortDirection};
pub use lora::{DetectedLora, LoraModel};
pub use metadata_format::MetadataFormat;
pub use pipeline::{CleanupQueueItem, PipelineDetectedPath};
pub use prompt_stacking::{plan_prompt_stacks, PromptStackCandidate, PromptStackPlan};
pub use prompt_stat::PromptStat;
pub use search::{CursorFilePage, FilePage, PageCursor, SearchCriteria};
pub use search_parser::parse_query;
pub use similarity::SimilarityMatch;
pub use stack::StackSummary;
pub use storage_root::{NormalizedPath, PathResolver, StorageRoot};
pub use tag::Tag;
