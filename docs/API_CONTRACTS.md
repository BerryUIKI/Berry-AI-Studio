# Omera application interface contracts

Status: current source contract plus explicitly marked proposals. The current source includes unfinished review fixes; it is not a released Omera API. Application identity activation and migration commands below are not implemented yet.

## Sources of truth and compatibility

| Surface | Source of truth | Consumer |
| --- | --- | --- |
| Registered IPC commands | `src-tauri/src/lib.rs` `generate_handler!` and `src-tauri/src/commands.rs` | Vue `invoke` calls |
| Domain DTOs | `crates/berry-domain/src/` Serde definitions | `src/types.ts` |
| Configuration DTO/defaults | Rust `AppConfig` in commands; config persistence service | `src/utils/config.ts` |
| Thumbnail DTOs | `crates/berry-scan/src/thumbnail.rs` | `src/utils/thumbnail.ts` |
| Update DTOs | Rust `UpdateDownloadProgress` | `src/utils/updater.ts` |
| Watcher events | `src-tauri/src/watcher.rs` | App refresh scheduling |

[IPC_REFERENCE.md](IPC_REFERENCE.md) inventories all 138 currently registered commands, their actual request keys and Rust return types. Regenerate with `node scripts/generate-ipc-reference.mjs`; check with `node scripts/generate-ipc-reference.mjs --check`. The generator fails if a registered signature is not recognized. It does not validate nested DTOs; engineers must test Serde/TypeScript compatibility explicitly.

Current commands are local Tauri IPC, not HTTP endpoints. Do not invent REST routes or expose these commands through a network server. Commands are restricted to the configured application WebView; IPC arguments still require backend validation.

Top-level invoke arguments use camelCase (`fileId`, `batchSize`, `isNegative`). Nested DTO properties use their Serde field names, normally snake_case (`folder_id`, `modified_at`, `config_revision`). Injected `State`/`AppHandle` arguments are not caller input. `Result<T, String>` resolves to T and rejects with a string; it is **not** `{ success, data, error }`. Some explicit DTOs have their own `success` field; do not generalize that shape to all commands.

Enums serialize according to their Serde definitions, not Rust variant spelling. Optional fields accept null and, where Serde/Tauri allows, omission; required DTO fields must be supplied. File IDs are numbers today: validate safe integers in JS; a future string-ID transition needs an explicit contract change. Timestamps must retain each DTO's documented units, especially `modified_at` seconds versus UI elapsed milliseconds.

Additive optional fields require compatible defaults in Rust and TypeScript. Renaming/removing a command, changing response shape, event names, required fields or numeric units requires synchronized callers, tests, this document and the generated reference. Do not silently change contracts while implementing a UI issue.

## Gallery and search

Core wire shapes (simplified to the stable envelope; use source DTOs for all image metadata fields):

```ts
interface PageCursor { sort_value: string; id: number }
interface FilePage { items: ImageFile[]; total: number; offset: number; has_more: boolean }
interface CursorFilePage {
  items: ImageFile[];
  total: number;
  next_cursor: PageCursor | null;
  prev_cursor: PageCursor | null;
  has_more: boolean;
}
interface SimilarFileItem { file: ImageFile; score: number }
interface SimilarityMatch { file_id: number; score: number }
```

Use paginated commands for ordinary gallery browsing. Unbounded `list_files`, `query_files`, `search_files` and `search_files_by_query` remain registered for explicit full-result callers; they are not the default scrolling path.

```ts
const page = await invoke<CursorFilePage>('search_files_by_query_cursor_page', {
  query: 'landscape',
  context: {
    folder_id: 12,
    limit: 400,
    sort: 'modified_at',
    direction: 'desc',
    cursor: null,
  },
});
```

Verify the exact `SearchCriteria` properties against the Rust definition before extending this example. Cursor pages return `{ items, total, next_cursor, prev_cursor, has_more }`; offset pages return `{ items, total, offset, has_more }`. Cursors are valid only for the same filter/sort context. Discard in-flight responses after context changes. Pagination must terminate on `has_more: false`, and an empty page must not cause an infinite retry loop.

Gallery rows omit raw workflow/parameter payloads. `get_file_details({ fileId })` returns the full `ImageFile`. Cache details by file identity and revision, and use an invalidation generation for requests already in flight. A metadata hydration result must not overwrite newer mutable flags or ratings in the UI.

`list_filtered_stacks({ query, context })` and `get_filtered_stack_members({ stackId, query, context })` share gallery filters. Do not replace these with unfiltered stack APIs inside a scoped search.

## Semantic search and indexing

| Command | Response | Meaning |
| --- | --- | --- |
| `search_by_text_prompt({ prompt, limit })` | `SimilarFileItem[]` | `{ file: ImageFile, score: number }` per result in the working changes |
| `find_similar_to_file({ fileId, modelId, limit })` | `SimilarFileItem[]` | Hydrated image results |
| `search_similar_files({ modelId, query, limit })` | `SimilarityMatch[]` | Low-level match DTO; do not treat it as `ImageFile[]` |
| `index_clip_images_batch({ batchSize, retryFailed })` | `ClipBatchIndexResult` | `indexed_count` is work completed in this batch; `failed_count` tracks recorded model/source-revision failures; `remaining_count` governs continued work; `total_count` is the model indexing population |
| `cancel_clip_indexing()` | null | Cooperative cancellation between images; not immediate interruption of inference |

Current batch size is clamped to 1–20. A first explicit retry batch may set `retryFailed: true`; subsequent batches must not continuously clear the failure set. Cancellation alone does not imply success, and failure counts must remain visible when remaining work reaches zero. The count semantics and loaded-model changes require regression tests before this API is considered release-stable.

## Settings and storage paths

```ts
const config = await invoke<AppConfig>('get_app_config');
const saved = await invoke<AppConfig>('save_app_config', {
  config: { ...config, theme: 'light' },
});
// Retain saved.config_revision; do not keep submitting config.config_revision.
```

`get_app_config()` resolves `AppConfig`; `save_app_config({ config })` resolves the saved configuration with an incremented `config_revision`. The caller must retain the returned revision. A stale revision rejects rather than overwriting newer changes; reload and reconcile instead of retrying the same stale whole object.

The in-progress persistence service writes atomically, rejects corrupt JSON without silently replacing it, and stores secret references in configuration while resolving credentials for authorized internal use. Do not log full configs, credential values, or imported exports. Credentials are not safe browser localStorage content.

`legacy_migration_complete` currently refers to the old config/localStorage migration. It is **not** an Omera migration receipt, proof of database validation, or authorization to delete a directory. New Omera values take precedence over imported legacy values; false/zero/empty values are not evidence of absence.

`get_storage_paths()` returns `data_dir`, `config_file`, `database_file`, `thumbnails_dir`, `models_dir`, and `updates_dir`. UI code must use returned paths rather than reconstructing them from product names. `open_storage_dir({ target })` accepts the named categories in the reference. The current runtime still uses the legacy application identity; target Omera paths are activated by the lead-owned migration.

## File operations, backup and restore

`copy_files`/`move_files` take `{ filePaths, targetFolderId }`; `trash_files` takes `{ filePaths }`. Current working responses are a completed count on full success or a rejection string containing partial progress and recovery-journal information. This is a known transitional limitation (#100): never parse localized strings to infer which individual files succeeded, and never automatically retry the entire operation after partial failure. Refresh affected state and display the error.

The lead will stabilize a per-file result DTO before engineers add collision/retry UI. Source deletion, collision policies and filesystem rollback are backend responsibilities. Copy/move sidecars and trash behavior must not diverge from the primary-file outcome without an explicit partial result.

`backup_database({ destinationPath })` exports a database snapshot. `restore_database({ sourcePath })` stages a validated database and restarts the app on success; do not assume code after a successful invoke will run before process exit. Cloud restore has the same restart boundary. No component may replace `berry.db`/`omera.db` itself or delete WAL/SHM files.

The storage-only Rust function `recovery::migrate_copy(source, destination)` is **not an IPC command**. It snapshots committed SQLite/WAL content, validates the source, upgrades a staged copy, rejects existing destinations, and preserves the source. It does not migrate config/credentials, discover sources, track receipts, or authorize cleanup. Integrators must implement those steps under the migration contract.

## Updates and long-running operations

`download_update({ url, filename })` resolves a staged installer path after signature verification in the working code. `install_update({ installerPath, silent })` revalidates the local installer and launches installation. Missing embedded trust configuration is an error; engineers must not add a bypass. Platform/architecture asset selection and renamed-repository trust remain lead-owned work.

Subscribe to progress before starting an operation and always dispose listeners in `finally` and component teardown. Existing events are global and mostly lack operation IDs; concurrent unrelated jobs may interleave. Do not infer ownership by listening to every event. Adding operation IDs is an explicit cross-layer contract change, not a frontend-only patch.

| Event | Current payload/source | Lifecycle rule |
| --- | --- | --- |
| `scan-progress` | `ScanProgress`, scan crate | Coalesced progress; final command result supplies outcome |
| `library-files-changed` | `{ folder_id, stats: ScanStats }` | Invalidate affected query contexts; debounce/coalesce updates |
| `thumbnail-progress` | `{ current, total, done }` | Completion is not proof that every thumbnail succeeded; inspect batch results |
| `berry://export-progress` | `ExportProgressEvent` | Legacy name still active; rename producer and consumer together during activation |
| `update-download-progress` | `UpdateDownloadProgress` | Progress does not authorize execution; verified command result is required |
| `cloud-sync://progress` | `CloudSyncProgress` | Dispose on teardown and query final summary separately |

An `async fn` declaration alone does not ensure work is off the runtime thread. Blocking filesystem/decode/inference/network work belongs in bounded workers, and shared database locks must be released before that work. See L8/#102 for command-by-command ownership changes.

## Proposed migration IPC — NOT IMPLEMENTED

Owner: lead. These names and DTOs are a design proposal for review, not callable commands. Engineers may create typed UI mocks after confirming the proposal; production calls must wait for registration and contract tests.

| Proposed command | Request | Response and side effects |
| --- | --- | --- |
| `get_legacy_migration_status` | none | Versioned status, discovered source summaries, current receipt and available actions; no deletion |
| `preview_legacy_migration` | `{ sourceId }` | Backend-issued plan ID, source/destination summaries, conflicts, space estimate, exclusions and required user action; sourceId is an opaque discovered token, not a caller-selected arbitrary directory |
| `start_legacy_migration` | `{ planId }` | Job ID after revalidating plan and acquiring migration lock; preserve source |
| `get_legacy_migration_job` | `{ jobId }` | Durable state/progress/error plus retryability; usable after reconnect/restart |
| `preview_legacy_cleanup` | `{ receiptId }` | Expiring preview ID, exact eligible paths/categories/sizes, exclusions, source revision and destination verification state |
| `confirm_legacy_cleanup` | `{ previewId, confirmed: true }` | Per-artifact outcomes after revalidation; reject stale preview, concurrent writer, missing destination, changed source or absent confirmation |
| `defer_legacy_cleanup` | `{ receiptId }` | Persisted decision to retain old data; prevent repetitive startup prompts |

Proposed migration states: `discovered -> awaiting_source_choice -> planned -> copying -> validating -> activating -> migrated -> cleanup_available`. Failure records the failed stage and preserves retry/recovery evidence. Cleanup states are separate: `retained`, `cleaning`, `partially_cleaned`, `cleaned`. Restarting must not promote a partially activated migration to success.

Proposed structured errors: `{ code, message_key, retryable, context }`. Initial codes: `SOURCE_CHANGED`, `SOURCE_BUSY`, `MULTIPLE_SOURCES`, `DESTINATION_EXISTS`, `UNSUPPORTED_SCHEMA`, `INTEGRITY_FAILED`, `INSUFFICIENT_SPACE`, `CREDENTIAL_STORE_UNAVAILABLE`, `LEGACY_ORIGIN_UNAVAILABLE`, `PLAN_EXPIRED`, `CONFIRMATION_REQUIRED`, `CLEANUP_PARTIAL`. Context must be bounded and secret-free. Existing commands still reject strings; do not retrofit this envelope without a coordinated migration.

Confirmation must refer to the exact backend preview the user saw. A boolean alone is insufficient to authorize new paths. Backend cleanup is restricted to receipt-owned legacy artifacts, excludes media/vaults, rechecks destination health and source revision, and never falls back from trash to permanent deletion. See [OMERA_MIGRATION.md](OMERA_MIGRATION.md).

## Required contract tests

- Correct camelCase invocation and snake_case nested DTOs, including `get_prompt_stats({ isNegative, limit })` (#119).
- Rust DTO round trips and matching TypeScript fixtures for semantic results, pages, config revisions, progress and batch failure counts.
- Empty/null/invalid inputs, missing records, corrupt/future schema, rejected stale saves, concurrent operations and cancellation.
- Source preservation, interrupted migration/retry, stale cleanup previews, declined cleanup and partially successful trash.
- Event listener teardown and stale-response suppression after navigation/unmount.
- Regenerated command inventory and no undocumented command removal. A registered command does not certify its feature as supported.

## Interface review checklist for every PR

1. List the changed command/event/DTO and all producers and consumers, including tests and dormant settings panels.
2. State whether the change is additive, compatible through defaults, or breaking; specify old-data behavior and caller rollout order.
3. Document validation limits, side effects, lock/worker ownership, cancellation boundaries and partial outcomes. Reads that perform migration or cache writes must say so.
4. Supply request/success/error fixtures using actual serialized property names. Never include secrets or personal file paths in fixtures.
5. Update the command inventory and contract tests. Lead approval is required for identity, persistence, cleanup, signing and security boundaries.
