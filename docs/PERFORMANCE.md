# Performance Architecture and Optimization Plan

## Performance Goals

Berry AI Studio should remain interactive with large local libraries while keeping CPU, memory, and cache growth predictable. The UI should render indexed data immediately, perform filesystem reconciliation in the background, and limit image decoding to the visible working set.

## Implemented Safeguards

### Gallery scrolling

- Grid and Waterfall cards keep the zoom-selected width. Window resizing changes the column count rather than stretching images.
- Scroll events are coalesced through `requestAnimationFrame`, limiting reactive updates to one per rendered frame.
- Waterfall geometry is cached until files, card width, gap, or viewport width changes.
- Visible Waterfall items are found with a binary search in each column instead of filtering the entire result set on every scroll update.
- Only the viewport plus overscan is mounted.
- Library and structured-search results arrive in 400-item pages. Grid, Waterfall, and Table request another page near the loaded boundary while retaining an exact filtered total.
- Stale page responses are discarded when the user changes folder, search, or sort context.

### Thumbnail pipeline

- Visible thumbnails use the single-item path for the shortest latency.
- Look-ahead generation waits until scrolling settles, so dragging the scrollbar does not enqueue work for intermediate positions.
- Batch requests are deduplicated, serialized, and split into bounded chunks.
- Grid, Waterfall, and Table advance a shared viewport generation when their visible range changes. Rust checks that generation again inside the bounded decode pool, so stale queued files are discarded before decoding begins.
- Near look-ahead outranks backward look-ahead. Cache misses display lightweight placeholders rather than loading full-resolution originals, and placeholder motion follows `prefers-reduced-motion`.
- Grid and Waterfall requests select the smallest cache tier that covers the rendered card at the current device scale, capped by the user's resolution setting. Table rows use the compact tier instead of generating gallery-sized previews.
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

### Prompt-based organization

- Explicit whole-library organization is allowed to read the complete candidate set.
- Candidates are sorted once by folder and timestamp; comparisons stop after the configured time window, and empty prompts are discarded before similarity work.

## Measurement Checklist

Record these values against representative libraries (1k, 10k, and 50k items) before changing performance-sensitive code:

| Metric | Target |
| --- | --- |
| Time to first usable gallery from a warm database | Under 1 second on a typical SSD |
| Main-thread long tasks during scrollbar drag | No task over 50 ms |
| Mounted gallery cards | Viewport plus bounded overscan only |
| Concurrent thumbnail batches | One frontend batch; bounded Rust workers |
| Duplicate thumbnail generation for the same fingerprint | Zero under normal operation |
| Startup directory walks inside the cooldown | Zero |

Use browser performance traces for WebView work, Rust timing spans for commands, and database query plans for search regressions. Avoid judging scrolling solely from average frame rate; inspect worst-frame latency and long tasks.

## Prioritized Follow-Up Work

### P0: Query pagination and incremental result delivery — Phase 1 complete

The gallery now fetches bounded pages and extends them near the viewport boundary. SQLite returns the exact filtered total with the page through a window count, avoiding a second filter query and full IPC materialization. Offset paging remains intentionally isolated behind the page API; replace it with sort-aware keyset cursors after representative deep-page benchmarks show that SQLite offset traversal is material.

### P0: Filesystem change journal or watcher — Phase 1 complete

Registered roots now use the platform watcher, a durable coalesced journal, and path-level reconciliation. Optional cooldown scans remain as recovery for offline or missed events. Follow-up work should expose watcher health, add a polling fallback for unreliable network filesystems, and benchmark event storms on large batch imports.

### P1: Persistent thumbnail manifest and cache budget — Phase 2 complete

The cache now has a persistent size-tiered manifest, rate-limited access tracking, background adoption of legacy files, configurable usage reporting, and bounded LRU enforcement. Gallery zoom and table density select the smallest sufficient tier while respecting the configured quality ceiling. Follow-up work should reuse an already-cached larger tier when it avoids redundant generation, and benchmark manifest adoption with 50k cached files.

### P1: Cancelable thumbnail priority queue — Phase 1 complete

IPC now carries monotonic viewport generations, the backend skips stale work inside the bounded decode pool, and the frontend orders near look-ahead before backward look-ahead. Visible requests begin before the debounced speculative queue. Follow-up work should expose per-job diagnostics and measure cancellation latency with unusually slow network-backed image decoders.

### P1: Faster scan reconciliation

Stream directory entries instead of collecting the full tree before indexing, reduce progress-event frequency, and compare directory-level fingerprints where the platform provides reliable metadata. Benchmark network drives separately because traversal latency dominates there.

### P2: Component and payload reduction

- Split large modal bundles with dynamic imports.
- Return lightweight gallery DTOs and fetch full metadata only for the selected item.
- Move expensive filter aggregation to indexed SQL and cache stable facet counts.
- Audit object URL and decoded-image lifetime after long browsing sessions.

## GUI Recommendations

- Keep Grid, Waterfall, and Table as explicit modes, with the current mode and zoom persisted.
- Add a compact density control that changes card width in fixed steps, not fluid stretching.
- Show a subtle placeholder while a thumbnail is queued and a distinct retry affordance after a decode failure.
- Keep stack transitions short (roughly 180–220 ms), spatially explain expansion, and disable them when reduced motion is requested.
- Provide System, Midnight, Graphite, Violet, and Light themes. Use semantic color tokens so every panel follows the selected theme.
- Add a small background-activity popover for scans, thumbnail generation, tagging, and embeddings, with pause/cancel controls where supported.
- Preserve scroll position independently per folder/search context so navigation does not force users back to the beginning.
