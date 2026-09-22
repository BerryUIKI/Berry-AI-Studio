# Performance Architecture and Optimization Plan

## Omera storage direction

See [STORAGE_EVOLUTION.md](STORAGE_EVOLUTION.md) for measured SQLite optimization and [OMERA_MIGRATION.md](OMERA_MIGRATION.md) for pre-1.0 compatibility. Renaming a database is not a performance improvement. Migration precedes opening an imported library; ordinary startup must continue to render indexed data before optional scans, cache migration, or model verification.

## Performance Goals

Omera should remain interactive with large local libraries while keeping CPU, memory, and cache growth predictable. The UI should render indexed data immediately, perform filesystem reconciliation in the background, and limit image decoding to the visible working set.

## Implemented Safeguards

### Gallery scrolling

- Grid and Waterfall cards keep the zoom-selected width. Window resizing changes the column count rather than stretching images.
- Scroll events are coalesced through `requestAnimationFrame`, limiting reactive updates to one per rendered frame.
- Waterfall geometry is cached until files, card width, gap, or viewport width changes.
- Visible Waterfall items are found with a binary search in each column instead of filtering the entire result set on every scroll update.
- Only the viewport plus overscan is mounted.
- Library and structured-search results arrive in 400-item pages. Grid, Waterfall, and Table request another page near the loaded boundary while retaining an exact filtered total.
- Stale page responses are discarded when the user changes folder, search, or sort context.

### Initial UI bundle

- Eighteen infrequent modals and drawers are loaded through async Vue component boundaries only when opened.
- The production entry bundle decreased from approximately 465 KB to 362 KB uncompressed JavaScript and from 136 KB to 49 KB uncompressed CSS. Gzip sizes decreased from 140.5 KB to 114.9 KB for JavaScript and from 22.1 KB to 9.2 KB for CSS.

### Gallery payloads

- Paginated gallery queries apply every text, structured metadata, album, tag, favorite, NSFW, folder, sort, and range filter against the complete SQLite row before projecting the response.
- Gallery responses preserve structured metadata required by Grid, Waterfall, Table, prompt copying, and grouping, along with stack identity and order. Large raw parameter strings and workflow graphs are omitted from page IPC.
- Selecting or previewing a file fetches its complete record by ID. The frontend deduplicates concurrent requests and retains a 64-entry revision-aware LRU, so Inspector raw metadata remains available without repeating detail IPC.
- Free-form queries retain the active folder, album, tag, favorite, or sensitivity scope instead of widening back to the full library. Stack summaries use the same SQL filter predicate as the page, and filtered expansion loads only matching members.
- Semantic-search grouping is derived in one pass over its bounded result set, so stack counts and heroes never include images outside the semantic result.

### Thumbnail pipeline

- Visible thumbnails use the single-item path for the shortest latency.
- Look-ahead generation waits until scrolling settles, so dragging the scrollbar does not enqueue work for intermediate positions.
- Batch requests are deduplicated, serialized, and split into bounded chunks.
- Grid, Waterfall, and Table advance a shared viewport generation when their visible range changes. Rust checks that generation again inside the bounded decode pool, so stale queued files are discarded before decoding begins.
- Near look-ahead outranks backward look-ahead. Cache misses display lightweight placeholders rather than loading full-resolution originals, and placeholder motion follows `prefers-reduced-motion`.
- Grid and Waterfall requests select the smallest cache tier that covers the rendered card at the current device scale, capped by the user's resolution setting. Table rows use the compact tier instead of generating gallery-sized previews.
- A visible request can reuse the smallest existing larger tier for the same source revision. Missing files referenced by the manifest are removed and the next sufficient tier is considered before new decoding begins.
- The Rust decoder uses a dedicated bounded Rayon pool, and the frontend keeps a bounded in-memory URL LRU.
- SQLite schema v11 persists a thumbnail manifest keyed by file ID, source modification time, size tier, and codec. Access timestamps are written at most once per thumbnail per hour.
- A configurable disk budget defaults to 2 GB. Generation and background legacy-cache synchronization remove least-recently used tiers in bounded batches when usage exceeds the budget.
- Thumbnails are generated lazily. Import-time generation of the entire library is intentionally avoided because it delays ingest, creates cache entries that may never be viewed, and causes a CPU and disk spike. A future opt-in idle prewarm mode can be added for users who prefer disk usage over first-view latency.

### Startup scanning

- Indexed SQLite data is loaded before filesystem scanning begins.
- Registered roots use a long-lived native watcher. Events are coalesced by path in a durable SQLite journal and reconciled after a 750 ms quiet period.
- Ordinary file changes re-index only the reported path. Directory changes are limited to the reported subtree, and deleted subtrees are removed with a prefix-bounded query.
- Startup scanning is disabled for new installations by default.
- Users who enable it receive a per-folder cooldown (six hours by default), preventing a full tree walk on every launch.
- Manual scans remain available when immediate reconciliation is required.
- Scan progress is coalesced to at most one update per 64 processed files or 100 ms, with an unconditional completion update. Large scans therefore avoid one Tauri event and reactive UI update per file.
- Full scans stream media entries from the directory walker into bounded database batches. They do not retain a complete `MediaFile` tree or a duplicate list of seen paths; unmatched database fingerprints identify removals after traversal.

### Prompt-based organization

- Explicit whole-library organization is allowed to read the complete candidate set.
- Candidates are sorted once by folder and timestamp; comparisons stop after the configured time window, and empty prompts are discarded before similarity work.

## Measurement Checklist

Record these values against representative libraries (1k, 10k, and 50k items) before changing performance-sensitive code:

| Metric | Target | 1k Items | 10k Items | 50k Items | Status |
| --- | --- | --- | --- | --- | --- |
| Time to first usable gallery from a warm database | Under 1 second on a typical SSD | 4.48 ms | 23.65 ms | 143.55 ms | Met |
| Main-thread long tasks during scrollbar drag | No task over 50 ms | 0 tasks | 0 tasks | 0 tasks | Met |
| Mounted gallery cards | Viewport plus bounded overscan only | Viewport only | Viewport only | Viewport only | Met |
| 50k thumbnail manifest adoption time | Under 10 seconds | - | - | 3.26 s (15.3k/s) | Met |
| Concurrent thumbnail batches | One frontend batch; bounded Rust workers | 1 batch | 1 batch | 1 batch | Met |
| Duplicate thumbnail generation for the same fingerprint | Zero under normal operation | 0 | 0 | 0 | Met |
| Startup directory walks inside the cooldown | Zero | 0 | 0 | 0 | Met |

Detailed benchmark methodology and reproducible test logs are documented in [`docs/benchmarks/LARGE_LIBRARY_BENCHMARK.md`](benchmarks/LARGE_LIBRARY_BENCHMARK.md).

Use browser performance traces for WebView work, Rust timing spans for commands, and database query plans for search regressions. Avoid judging scrolling solely from average frame rate; inspect worst-frame latency and long tasks.

## Prioritized Follow-Up Work

### P0: Query pagination and incremental result delivery — Phase 2 complete

The gallery fetches bounded pages and extends them near the viewport boundary. Virtual scrolling seamlessly connects directly to keyset cursor deep pagination (`search_files_cursor_page` and `search_files_by_query_cursor_page`). The initial page computes the exact filtered window total, and subsequent scroll requests use `PageCursor` (sort value + row ID) to bypass offset traversal and avoid repeating window counts. Keyset cursor access achieves O(1) row traversal (< 1 ms at 40,000+ items, an 84x speedup over offset paging).

### P0: Filesystem change journal or watcher — Phase 2 complete

Registered roots use the platform watcher, a durable coalesced journal, and path-level reconciliation. The worker receiver non-blockingly batch-drains channel events (up to 1,024 events per batch) to prevent transaction storms during massive batch file additions or unzips, completing 10,000 coalesced event writes in under 80 ms. Runtime watcher health metrics (`is_active`, `watched_roots_count`, `pending_journal_count`, `last_reconcile_time`, `last_error`) are exposed via IPC and monitored in the Background Activity panel.

### P1: Persistent thumbnail manifest and cache budget — Phase 2 complete

The cache now has a persistent size-tiered manifest, rate-limited access tracking, background adoption of legacy files, configurable usage reporting, and bounded LRU enforcement. Gallery zoom and table density select the smallest sufficient tier while respecting the configured quality ceiling, and existing larger tiers are reused rather than generating redundant smaller files. Manifest synchronization for 50k cached files runs at over 15,300 files/sec, completing in 3.26 seconds.

### P1: Cancelable thumbnail priority queue — Phase 2 complete

IPC now carries monotonic viewport generations, the backend skips stale work inside the bounded decode pool, and the frontend orders near look-ahead before backward look-ahead. Visible requests begin before the debounced speculative queue. Per-job runtime queue diagnostics are now fully implemented in both the Rust Rayon worker pool and the frontend LRU cache, tracking real-time queued, running, canceled, completed, failed, and deduplication hit metrics without production console overhead.


### P1: Faster scan reconciliation & Directory Fingerprints — Phase 2 complete

Progress-event coalescing and streaming full-folder traversal are complete. Comprehensive benchmark analysis (`docs/benchmarks/DIRECTORY_FINGERPRINT_BENCHMARK.md`) proves that parent directory `mtime` gating reduces filesystem operations by over 80% on local storage and slashes remote network RPC roundtrips by 5.6x on SMB/NFS/WebDAV shares.

### P2: Component and payload reduction

- [Completed] Split infrequent modal bundles with dynamic imports.
- [Phase 1 complete] Exclude raw workflow payloads from gallery pages and fetch complete metadata on selection.
- [Empirically Evaluated] A dedicated gallery DTO would only save ~80 KB per 400-item page (less than 2 ms transfer over localhost IPC). Retaining the current projected `ImageFile` is optimal and avoids duplicating schema types.
- Move expensive filter aggregation to indexed SQL and cache stable facet counts.
- [Completed] Audit object URL and decoded-image lifetime after long browsing sessions. Memory cache strictly bounded to 3,000 LRU entries, batch keys capped at 5,000 items, and unmounted event listeners cleaned up.


## GUI Recommendations — Implemented

- [x] Keep Grid, Waterfall, and Table as explicit modes, with the current mode and zoom persisted.
- [x] Add a compact density control that changes card width in fixed steps, not fluid stretching.
- [x] Show a subtle placeholder while a thumbnail is queued and a distinct retry affordance after a decode failure.
- [x] Keep stack transitions short (roughly 180–220 ms), spatially explain expansion, and disable them when reduced motion is requested.
- [x] Provide System, Midnight, Graphite, Violet, and Light themes. Use semantic color tokens so every panel follows the selected theme.
- [x] Add a small background-activity popover for scans, thumbnail generation, tagging, and embeddings, with pause/cancel controls where supported.
- [x] Preserve scroll position independently per folder/search context so navigation does not force users back to the beginning.
