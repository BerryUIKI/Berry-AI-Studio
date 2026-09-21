//! Folder scanning and indexing orchestration.
//!
//! `berry-scan` ties the core crates together into a single operation the app
//! shell can call: walk a folder recursively, detect each media file's
//! container, extract metadata (via a pluggable extractor from
//! `berry-metadata`), persist rows through `berry-storage`, and drop rows for
//! files that disappeared from disk. The Tauri shell only wires this up.

pub mod export;
pub mod html_showcase;
pub mod scanner;
pub mod thumbnail;

pub use export::{
    execute_batch_export, format_export_filename, sanitize_filename_part, ProcessedExportItem,
};
pub use html_showcase::{generate_html_showcase, ShowcaseItemMetadata};
pub use scanner::{ScanError, ScanProgress, ScanStats, Scanner};
pub use thumbnail::{
    batch_generate_thumbnails, clear_thumbnail_cache, ensure_thumbnail, get_thumbnail_cache_stats,
    get_thumbnail_path, get_thumbnail_queue_diagnostics, reset_thumbnail_queue_diagnostics,
    synchronize_thumbnail_manifest, ThumbnailBatchResult, ThumbnailCacheStats, ThumbnailProgress,
    ThumbnailQueueDiagnostics, ThumbnailRequest,
};
