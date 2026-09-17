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

### Thumbnail pipeline

- Visible thumbnails use the single-item path for the shortest latency.
- Look-ahead generation waits until scrolling settles, so dragging the scrollbar does not enqueue work for intermediate positions.
- Batch requests are deduplicated, serialized, and split into bounded chunks.
- The Rust decoder uses a dedicated bounded Rayon pool, and the frontend keeps a bounded in-memory URL LRU.
- Thumbnails are generated lazily. Import-time generation of the entire library is intentionally avoided because it delays ingest, creates cache entries that may never be viewed, and causes a CPU and disk spike. A future opt-in idle prewarm mode can be added for users who prefer disk usage over first-view latency.

### Startup scanning

- Indexed SQLite data is loaded before filesystem scanning begins.
- Startup scanning is disabled for new installations by default.
- Users who enable it receive a per-folder cooldown (six hours by default), preventing a full tree walk on every launch.
- Manual scans remain available when immediate reconciliation is required.

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

### P0: Query pagination and incremental result delivery

Several library commands still materialize complete `Vec<ImageFile>` results and serialize them across IPC. Virtual rendering limits DOM cost but not database allocation, JSON serialization, transfer, or frontend memory. Add cursor or keyset pagination, fetch an initial window, and extend it as the viewport approaches the end. Keep exact total counts in separate lightweight queries.

### P0: Filesystem change journal or watcher

The cooldown reduces redundant startup work but does not remove the cost of a due full walk. Add a long-lived watcher for registered roots, persist a small change journal, and reconcile only affected paths. A periodic low-priority verification scan should remain as a recovery mechanism for missed watcher events or offline changes.

### P1: Persistent thumbnail manifest and cache budget

Add a manifest keyed by file ID, modification time, size tier, and codec. Track last access and enforce a configurable disk budget with LRU cleanup. Generate a larger tier only when the selected gallery zoom requires it; reuse smaller tiers when acceptable.

### P1: Cancelable thumbnail priority queue

Expose request priority and cancellation across IPC. Visible cards should outrank look-ahead work, and queued jobs that move far outside the viewport should be discarded before decoding begins.

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

