# Registered Tauri IPC reference

Generated from the current source tree by `node scripts/generate-ipc-reference.mjs`. Run with `--check` to detect drift. This inventory describes the implementation snapshot, including unmerged work; it is not a claim about a published release. See [API_CONTRACTS.md](API_CONTRACTS.md) for wire conventions, side effects, error behavior and proposals that are not implemented.

There are 138 registered commands. Arguments below use the default Tauri camelCase invoke keys, while nested Serde DTO fields retain their declared snake_case names. AppHandle and State are injected by Tauri and must not be sent by the caller. Rust types are retained to avoid inventing missing DTO definitions: `Option<T>` accepts null/omission, `Vec<T>` is an array, `HashMap` is a JSON object, `Result<T, String>` resolves T or rejects with a string, and `()` resolves null. See Rust/TypeScript DTO locations in API_CONTRACTS.md.

The generator validates command coverage, not Serde field compatibility, safety, or supported feature availability. In particular, registered remote-database commands do not imply a working remote backend. When command attributes or declaration syntax change, update the generator or use a proper Rust parser; never silently skip commands.

| Command | Caller arguments (Rust value types) | Rust return type |
| --- | --- | --- |
| `get_app_info` | (none) | `Result<AppInfo, String>` |
| `add_folder` | `path: String` | `Result<Folder, String>` |
| `list_folders` | (none) | `Result<Vec<Folder>, String>` |
| `remove_folder` | `folderId: i64` | `Result<(), String>` |
| `list_files` | `folderId: i64` | `Result<Vec<ImageFile>, String>` |
| `query_files` | `folderId: Option<i64>`<br>`sort: Option<FileSortField>`<br>`direction: Option<SortDirection>` | `Result<Vec<ImageFile>, String>` |
| `set_file_rating` | `fileId: i64`<br>`rating: Option<u8>` | `Result<(), String>` |
| `set_files_rating` | `fileIds: Vec<i64>`<br>`rating: Option<u8>` | `Result<usize, String>` |
| `get_library_counts` | (none) | `Result<LibraryCounts, String>` |
| `scan_folder` | `folderId: i64` | `Result<ScanStats, String>` |
| `rebuild_metadata` | `folderId: i64` | `Result<ScanStats, String>` |
| `search_files` | `criteria: SearchCriteria` | `Result<Vec<ImageFile>, String>` |
| `search_files_page` | `criteria: SearchCriteria` | `Result<FilePage, String>` |
| `search_files_cursor_page` | `criteria: SearchCriteria` | `Result<CursorFilePage, String>` |
| `get_file_details` | `fileId: i64` | `Result<ImageFile, String>` |
| `search_files_by_query` | `query: String`<br>`folderId: Option<i64>`<br>`sort: Option<FileSortField>`<br>`direction: Option<SortDirection>` | `Result<Vec<ImageFile>, String>` |
| `search_files_by_query_page` | `query: String`<br>`context: SearchCriteria` | `Result<FilePage, String>` |
| `search_files_by_query_cursor_page` | `query: String`<br>`context: SearchCriteria` | `Result<CursorFilePage, String>` |
| `list_filtered_stacks` | `query: String`<br>`context: SearchCriteria` | `Result<Vec<StackSummary>, String>` |
| `get_filtered_stack_members` | `stackId: String`<br>`query: String`<br>`context: SearchCriteria` | `Result<Vec<ImageFile>, String>` |
| `list_distinct_models` | (none) | `Result<Vec<String>, String>` |
| `list_distinct_samplers` | (none) | `Result<Vec<String>, String>` |
| `create_album` | `name: String`<br>`description: Option<String>` | `Result<Album, String>` |
| `get_album` | `id: i64` | `Result<Option<Album>, String>` |
| `list_albums` | (none) | `Result<Vec<Album>, String>` |
| `get_album_counts` | (none) | `Result<HashMap<i64, i64>, String>` |
| `rename_album` | `id: i64`<br>`newName: String` | `Result<(), String>` |
| `delete_album` | `id: i64` | `Result<(), String>` |
| `add_file_to_album` | `albumId: i64`<br>`fileId: i64` | `Result<(), String>` |
| `add_files_to_album` | `albumId: i64`<br>`fileIds: Vec<i64>` | `Result<usize, String>` |
| `remove_file_from_album` | `albumId: i64`<br>`fileId: i64` | `Result<(), String>` |
| `remove_files_from_album` | `albumId: i64`<br>`fileIds: Vec<i64>` | `Result<usize, String>` |
| `count_album_files` | `albumId: i64` | `Result<i64, String>` |
| `list_album_files` | `albumId: i64` | `Result<Vec<ImageFile>, String>` |
| `create_tag` | `name: String`<br>`color: Option<String>` | `Result<Tag, String>` |
| `list_tags` | (none) | `Result<Vec<Tag>, String>` |
| `delete_tag` | `id: i64` | `Result<(), String>` |
| `tag_file` | `fileId: i64`<br>`tagId: i64` | `Result<(), String>` |
| `tag_files` | `fileIds: Vec<i64>`<br>`tagId: i64` | `Result<usize, String>` |
| `untag_file` | `fileId: i64`<br>`tagId: i64` | `Result<(), String>` |
| `untag_files` | `fileIds: Vec<i64>`<br>`tagId: i64` | `Result<usize, String>` |
| `get_file_tags` | `fileId: i64` | `Result<Vec<Tag>, String>` |
| `list_files_by_tag` | `tagId: i64` | `Result<Vec<ImageFile>, String>` |
| `set_file_favorite` | `fileId: i64`<br>`isFavorite: bool` | `Result<(), String>` |
| `set_files_favorite` | `fileIds: Vec<i64>`<br>`isFavorite: bool` | `Result<usize, String>` |
| `set_file_nsfw` | `fileId: i64`<br>`isNsfw: bool` | `Result<(), String>` |
| `set_files_nsfw` | `fileIds: Vec<i64>`<br>`isNsfw: bool` | `Result<usize, String>` |
| `get_prompt_stats` | `isNegative: bool`<br>`limit: usize` | `Result<Vec<PromptStat>, String>` |
| `get_checkpoint_models` | (none) | `Result<Vec<CheckpointModelStat>, String>` |
| `import_model_cache_file` | `path: String` | `Result<usize, String>` |
| `resolve_model_hash` | `hash: String` | `Result<Option<ModelCacheEntry>, String>` |
| `list_model_cache` | (none) | `Result<Vec<ModelCacheEntry>, String>` |
| `move_files` | `filePaths: Vec<String>`<br>`targetFolderId: i64` | `Result<usize, String>` |
| `copy_files` | `filePaths: Vec<String>`<br>`targetFolderId: i64` | `Result<usize, String>` |
| `trash_files` | `filePaths: Vec<String>` | `Result<usize, String>` |
| `reveal_in_file_manager` | `path: String` | `Result<(), String>` |
| `vacuum_database` | (none) | `Result<(), String>` |
| `backup_database` | `destinationPath: String` | `Result<(), String>` |
| `get_database_stats` | (none) | `Result<DatabaseStats, String>` |
| `restore_database` | `sourcePath: String` | `Result<(), String>` |
| `list_storage_roots` | (none) | `Result<Vec<StorageRoot>, String>` |
| `get_storage_root` | `rootUuid: String` | `Result<Option<StorageRoot>, String>` |
| `create_storage_root` | `root: StorageRoot` | `Result<(), String>` |
| `update_storage_root` | `root: StorageRoot` | `Result<(), String>` |
| `delete_storage_root` | `rootUuid: String` | `Result<(), String>` |
| `resolve_normalized_path` | `rootUuid: String`<br>`relativePath: String` | `Result<Option<String>, String>` |
| `relativize_local_path` | `absolutePath: String` | `Result<Option<NormalizedPath>, String>` |
| `fetch_change_log` | `query: ChangeLogSyncQuery` | `Result<Vec<ChangeLogEntry>, String>` |
| `record_change_event` | `eventType: String`<br>`entityId: i64`<br>`secondaryId: Option<String>`<br>`clientId: String`<br>`payload: Option<String>` | `Result<i64, String>` |
| `set_file_rating_occ` | `fileId: i64`<br>`rating: Option<u8>`<br>`expectedVersion: Option<i64>`<br>`clientId: String` | `Result<MutationResult, String>` |
| `test_database_connection` | `backend: String`<br>`connectionUrl: String` | `Result<DatabasePingResult, String>` |
| `export_sqlite_to_central_migration` | `options: MigrationOptions` | `Result<MigrationSummary, String>` |
| `export_files_batch` | `options: ExportOptions` | `Result<ExportSummary, String>` |
| `open_external_url` | `url: String` | `Result<(), String>` |
| `get_or_create_thumbnail` | `request: ThumbnailRequestArgs` | `Result<String, String>` |
| `save_video_thumbnail` | `fileId: i64`<br>`modifiedAt: i64`<br>`maxEdge: u32`<br>`base64Data: String` | `Result<String, String>` |
| `batch_generate_thumbnails` | `items: Vec<BatchThumbnailItem>`<br>`maxEdge: Option<u32>`<br>`cacheBudgetMb: Option<u64>`<br>`generation: Option<u64>` | `Result<berry_scan::ThumbnailBatchResult, String>` |
| `cancel_thumbnail_requests` | `generation: u64` | `()` |
| `get_thumbnail_queue_diagnostics` | (none) | `berry_scan::ThumbnailQueueDiagnostics` |
| `reset_thumbnail_queue_diagnostics` | (none) | `()` |
| `get_watcher_status` | (none) | `crate::watcher::WatcherStatus` |
| `get_thumbnail_cache_stats` | `cacheBudgetMb: Option<u64>` | `Result<berry_scan::ThumbnailCacheStats, String>` |
| `clear_thumbnail_cache` | (none) | `Result<usize, String>` |
| `upsert_file_embedding` | `fileId: i64`<br>`modelId: String`<br>`embedding: Vec<f32>` | `Result<(), String>` |
| `remove_file_embedding` | `fileId: i64`<br>`modelId: String` | `Result<bool, String>` |
| `get_file_embedding` | `fileId: i64`<br>`modelId: String` | `Result<Option<Vec<f32>>, String>` |
| `get_file_embedding_models` | `fileId: i64` | `Result<Vec<String>, String>` |
| `search_similar_files` | `modelId: String`<br>`query: Vec<f32>`<br>`limit: usize` | `Result<Vec<SimilarityMatch>, String>` |
| `find_similar_to_file` | `fileId: i64`<br>`modelId: Option<String>`<br>`limit: Option<usize>` | `Result<Vec<SimilarFileItem>, String>` |
| `list_tagger_models` | (none) | `Result<Vec<TaggerModelSummary>, String>` |
| `load_tagger_model` | `modelPath: String`<br>`tagsPath: String` | `Result<ModelInfo, String>` |
| `get_loaded_tagger_model` | (none) | `Result<Option<ModelInfo>, String>` |
| `auto_tag_file` | `fileId: i64`<br>`config: TaggerConfig`<br>`applyTags: bool` | `Result<Vec<TagPrediction>, String>` |
| `batch_auto_tag_files` | `fileIds: Vec<i64>`<br>`config: TaggerConfig` | `Result<BatchTagResult, String>` |
| `list_clip_models` | (none) | `Result<Vec<ClipModelSummary>, String>` |
| `load_clip_model` | `dirPath: String` | `Result<ClipModelInfo, String>` |
| `get_loaded_clip_model` | (none) | `Result<Option<ClipModelInfo>, String>` |
| `get_clip_index_status` | `modelId: Option<String>` | `Result<ClipIndexStatus, String>` |
| `index_clip_images_batch` | `batchSize: usize`<br>`retryFailed: Option<bool>` | `Result<ClipBatchIndexResult, String>` |
| `cancel_clip_indexing` | (none) | `()` |
| `search_by_text_prompt` | `prompt: String`<br>`limit: usize` | `Result<Vec<SimilarFileItem>, String>` |
| `list_loras` | (none) | `Result<Vec<LoraModel>, String>` |
| `get_lora` | `id: i64` | `Result<Option<LoraModel>, String>` |
| `save_lora` | `lora: LoraModel` | `Result<LoraModel, String>` |
| `delete_lora` | `id: i64` | `Result<bool, String>` |
| `get_image_detected_loras` | `fileId: i64` | `Result<Vec<DetectedLora>, String>` |
| `import_lora_civitai_info` | `filePath: String` | `Result<LoraModel, String>` |
| `scan_loras_directory` | `dirPath: String` | `Result<usize, String>` |
| `get_app_config` | (none) | `Result<AppConfig, String>` |
| `save_app_config` | `config: AppConfig` | `Result<AppConfig, String>` |
| `get_storage_paths` | (none) | `Result<StoragePaths, String>` |
| `open_storage_dir` | `target: String` | `Result<(), String>` |
| `download_update` | `url: String`<br>`filename: String` | `Result<String, String>` |
| `install_update` | `installerPath: String`<br>`silent: Option<bool>` | `Result<(), String>` |
| `add_folder_with_options` | `path: String`<br>`folderType: Option<String>`<br>`sourcePath: Option<String>`<br>`ingestAction: Option<String>`<br>`gracePeriodHours: Option<i32>`<br>`autoHarvest: Option<bool>` | `Result<Folder, String>` |
| `autodetect_local_ai_paths` | (none) | `Result<Vec<PipelineDetectedPath>, String>` |
| `harvest_pipeline_folder` | `folderId: i64` | `Result<usize, String>` |
| `process_pipeline_cleanups` | (none) | `Result<u64, String>` |
| `get_pipeline_cleanup_queue` | `limit: Option<usize>` | `Result<Vec<CleanupQueueItem>, String>` |
| `stack_images` | `fileIds: Vec<i64>`<br>`customStackId: Option<String>` | `Result<String, String>` |
| `merge_stacks` | `targetStackId: String`<br>`sourceStackIds: Vec<String>`<br>`standaloneFileIds: Vec<i64>` | `Result<StackMergeResult, String>` |
| `unstack_images` | `stackId: String` | `Result<u64, String>` |
| `set_stack_hero` | `stackId: String`<br>`heroFileId: i64` | `Result<(), String>` |
| `get_stack_members` | `stackId: String` | `Result<Vec<ImageFile>, String>` |
| `list_stacks` | `folderId: Option<i64>` | `Result<Vec<StackSummary>, String>` |
| `cull_stack_drafts` | `stackId: String`<br>`minRating: u8` | `Result<u64, String>` |
| `auto_stack_images` | `folderId: Option<i64>`<br>`similarityThreshold: Option<f32>`<br>`timeWindowMinutes: Option<i64>` | `Result<AutoStackResult, String>` |
| `check_generation_service` | `endpoint: String`<br>`serviceType: String` | `Result<bool, String>` |
| `send_to_comfyui` | `endpoint: String`<br>`workflowJson: String` | `Result<serde_json::Value, String>` |
| `send_to_webui` | `endpoint: String`<br>`payload: serde_json::Value` | `Result<serde_json::Value, String>` |
| `cloud_backup_test_connection` | `config: berry_domain::CloudBackupConfig` | `Result<berry_domain::CloudPingResult, String>` |
| `cloud_backup_create_snapshot` | `config: berry_domain::CloudBackupConfig`<br>`description: Option<String>` | `Result<berry_domain::CloudBackupResult, String>` |
| `cloud_backup_list_snapshots` | `config: berry_domain::CloudBackupConfig` | `Result<Vec<berry_domain::CloudSnapshotMeta>, String>` |
| `cloud_backup_restore_snapshot` | `config: berry_domain::CloudBackupConfig`<br>`snapshotFilename: String` | `Result<berry_domain::CloudRestoreResult, String>` |
| `cloud_sync_start` | `config: berry_domain::CloudBackupConfig`<br>`options: berry_domain::CloudSyncOptions` | `Result<(), String>` |
| `cloud_sync_cancel` | (none) | `Result<(), String>` |
| `cloud_sync_get_progress` | (none) | `Result<berry_domain::CloudSyncProgress, String>` |
| `cloud_sync_get_summary` | (none) | `Result<Option<berry_domain::CloudSyncResult>, String>` |
