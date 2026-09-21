# 🗺️ Berry AI Studio Roadmap

This roadmap documents completed milestones and future engineering goals for **Berry AI Studio**.

---

## 🏆 Completed Milestones (v0.1.0 & v0.1.1)

### ✅ Milestone 1: Core Foundation & Scaffolding
- [x] Multi-crate Rust workspace architecture (`berry-domain`, `berry-metadata`, `berry-scan`, `berry-storage`, `src-tauri`).
- [x] Embedded SQLite engine with `PRAGMA user_version` incremental migrations.
- [x] Cross-platform build configurations for Windows, macOS, and Linux.

### ✅ Milestone 2: Scanning & Indexing Engine
- [x] Multi-threaded recursive folder scanner with support for `PNG`, `JPG`/`JPEG`, `WebP`, `MP4`, and `.txt` sidecars.
- [x] Incremental indexing based on `(size_bytes, modified_at)` fingerprinting.
- [x] Real-time scanning progress event streaming to frontend.

### ✅ Milestone 3: Eagle-Style Studio Workspace & Browsing
- [x] Custom frameless desktop window with integrated menu bar (`File`, `Edit`, `View`, `Tools`, `Help`).
- [x] Collapsible 3-Pane Studio Layout (Left Sidebar + Center Canvas + Right Inspector).
- [x] High-performance virtualized grid view (`VirtualGrid.vue`) supporting tens of thousands of images.
- [x] Dynamic thumbnail zoom slider (130px–360px) and Table / Grid view switcher.
- [x] Fullscreen Quick Look Lightbox (`LightboxModal.vue`) with mouse-wheel zoom and keyboard navigation.

### ✅ Milestone 4: Comprehensive AIGC Metadata Parsers
- [x] **WebUI (AUTOMATIC1111 / SD.Next)**: PNG `parameters` chunk and WebP EXIF.
- [x] **ComfyUI**: Full Prompt and Workflow JSON graph syntax parsing.
- [x] **NovelAI**: Comment and Description signature parser.
- [x] **Fooocus & Fooocus-MRE**: Parameter parsing and base model resolution.
- [x] **InvokeAI & EasyDiffusion**: Embedded metadata & JSON sidecars.
- [x] Platform badge display standardization (`WebUI`, `ComfyUI`, `NovelAI`, `Fooocus`, `InvokeAI`, `SD`).

### ✅ Milestone 5: Organization, Search & Prompt Insights
- [x] Free-form search query parser supporting structured key-value tokens, quotes, and ranges.
- [x] Visual search filter drawer (`FilterDrawer.vue`).
- [x] Smart Albums and color-coded Tag taxonomy with drag-and-drop support.
- [x] Floating batch actions toolbar (`BatchActionBar.vue`).
- [x] Prompt keyword frequency & rating correlation analysis (`PromptStatsModal.vue`).
- [x] Checkpoint Model Manager with Civitai SHA256 cache import & hash reverse lookup (`ModelManagerModal.vue`).
- [x] Sensitive content (NSFW 18+) privacy protection with blur overlay and click-to-reveal.

### ✅ Milestone 6: Maintenance, Updates & Localization
- [x] SQLite database maintenance tools: live `VACUUM` compaction, backup export, and one-click restoration.
- [x] GitHub Releases auto-updater (`UpdateModal.vue`) with SemVer comparison and multi-state feedback.
- [x] Reactive i18n localization covering 7 languages (`en`, `zh-CN`, `zh-TW`, `ja`, `de`, `fr`, `es`) with OS auto-tracking.
- [x] Multilingual documentation and standardized release packaging naming `<AppName>_<OS>_<Architecture>.<extension>`.

---

## 🏆 Completed Milestones (v0.2.0)

### ✅ Milestone 8: AI-Assisted Tagging & Local CLIP Semantic Search
- [x] Persistent image embedding storage (schema v7 `file_embeddings`) and high-performance local cosine similarity ranking.
- [x] Visual similarity search (find similar compositions & styles) with real-time threshold slider and configurable result limits.
- [x] Local WD14 / Danbooru ONNX tagger for automated anime & realistic tag extraction with confidence filtering and batch tagging.
- [x] Local CLIP / SigLIP vision & text embedding pipeline with tokenization and batch library indexing dashboard (`ClipManagerModal.vue`).
- [x] Free-form natural language text-to-image semantic search toggle directly integrated into the search bar.

### ✅ Milestone 9: Advanced Generation Workflows & LoRA Catalog
- [x] LoRA trigger word library with Civitai info extraction and automatic prompt copy injection (`LoraManagerModal.vue`).
- [x] Full ComfyUI workflow node graph reconstruction and LoRA loader detection.
- [x] Detected LoRA chips in Inspector pane with one-click trigger injection.

### ✅ Milestone 10: Multi-Mode Folders, AIGC Ingest Pipeline & Image Stacking
- [x] **Multi-Mode Folder Architecture**:
  - Mode A: External Link mode (reference in-place, zero-copy, read-only watch).
  - Mode B: Managed Project Vaults (dedicated storage with drag-and-drop Copy/Move ingestion).
  - Mode C: AIGC Ingestion Pipeline (monitors WebUI / ComfyUI outputs, debounced harvest, and delayed recycle bin cleanup).
- [x] **WebUI & ComfyUI Autodetection & Onboarding Wizard**:
  - Heuristic auto-scan for local A1111/ComfyUI/Fooocus output directories.
  - Interactive First-Run Onboarding Wizard for instant setup.
  - Folder Creation Mode Wizard modal (`AddFolderModal.vue`).
- [x] **AIGC Image Stacking & Gallery Engine**:
  - Auto-stacking based on prompt similarity threshold and generation time window.
  - Discoverable scan-and-organize actions for the current folder or the full library, with localized progress and result feedback.
  - Empty-prompt safeguards, prompt-weight normalization, and folder/model boundaries for higher-quality automatic groups.
  - Manual grouping (`Ctrl+G`), unstacking (`Ctrl+Shift+G`), and hero cover selection (`Alt+S`).
  - Poker deck visual cards with badge counts, inline expand/collapse, and side-by-side compare mode (`C`).
  - Flat stack merge safety with warning confirmation dialog and suppressible settings.
  - Filter-aware stack counts, heroes, and expansion across navigation, structured, text, and semantic result contexts.
  - Virtualized Waterfall gallery layout mode alongside standard grid.

---

### ✅ Milestone 11: Generation Interop & Workflows
- [x] Drag-and-drop workflow transfer and send-to-WebUI / send-to-ComfyUI via local HTTP API backend proxy.
- [x] One-click batch cull (automatically keep hero/top-rated images and move drafts to system Trash with review modal).
- [x] Configurable ComfyUI and SD WebUI endpoints with real-time connectivity testing.

---

## 🔮 Upcoming Milestones (v0.3.0+)

### 🎯 Milestone 12: Cloud Sync & Export Utilities
- [x] **Milestone 12.1: Batch Transcoding, Privacy Stripping & Packaging**: Multi-threaded format conversion (WebP/JPEG/PNG), 4-tier privacy metadata stripping, downscaling constraints, customizable filename templates, sidecars (.txt/.json), and Directory / ZIP archive export.
- [ ] **Milestone 12.2: Standalone Interactive HTML Showcase Generator**: Self-contained zero-dependency HTML+CSS+JS photo album export with responsive gallery, lightbox preview, and prompt metadata viewer.
- [ ] **Milestone 12.3: S3 & WebDAV Snapshot Cloud Backup & Restore**: Automated and manual snapshots of SQLite database and configurations to AWS S3, Cloudflare R2, MinIO, or WebDAV servers.
- [ ] **Milestone 12.4: Incremental Remote Asset Mirroring & Delta Sync**: ETag / SHA-256 incremental media sync with background concurrency and bandwidth throttling.

### 🎯 Milestone 13: Large-Library Performance

- [x] Fixed-width responsive Grid and Waterfall columns.
- [x] Animation-frame scroll coalescing and per-column Waterfall visibility search.
- [x] Deduplicated, serialized thumbnail look-ahead scheduling.
- [x] Startup scan cooldown with opt-in scanning for new installations.
- [x] Bounded incremental gallery queries with exact totals and stale-response protection.
- [ ] Replace offset traversal with keyset cursors if deep-page benchmarks require it.
- [x] Persistent filesystem watcher journal with targeted path reconciliation and optional recovery scans.
- [x] Persistent size-tiered thumbnail manifest with configurable LRU disk budget.
- [x] Cancelable viewport generations with visible-first and directional look-ahead priority.
- [x] Zoom-aware thumbnail tier selection for Gallery and Table surfaces.
- [x] Reuse sufficient cached tiers and prune broken manifest paths during lookup.
- [x] Per-job thumbnail queue diagnostics.
- [x] Coalesce filesystem scan progress events by file count and elapsed time.
- [x] Stream full-scan directory entries without retaining the complete tree.
- [ ] Benchmark directory fingerprint strategies on local and network filesystems.
- [x] Lazy-load infrequent modals and drawers through async component boundaries.
- [x] Remove raw metadata blobs from paginated gallery IPC and fetch full details on selection.
- [x] Keep navigation filters and stack grouping aligned across paginated text and semantic searches.
- [x] Benchmark a dedicated gallery DTO for the remaining structured metadata fields.

### 🎯 Milestone 14: Multi-Database Support & Team Studio (MySQL & PostgreSQL)
*(See architectural design specification: [MULTI_DATABASE_COLLABORATION_RFC.md](./MULTI_DATABASE_COLLABORATION_RFC.md))*
- [x] Storage Engine trait abstraction with database dialect support for SQLite, MySQL 8.0+, and PostgreSQL 14+.
- [x] Keyset/cursor-based deep pagination for 500,000+ asset scale (`search_files_cursor_page`).
- [x] Cross-platform storage root mapping (`storage_roots` table and client mount configurations).
- [x] Optimistic concurrency control (`version` column) with Last-Write-Wins and set-union conflict resolution.
- [x] Tiered real-time change synchronization: default zero-DevOps change log journal polling engine (`CollaborationSyncEngine`).
- [x] Client-side on-demand local thumbnail caching preserving network storage bandwidth.
- [x] Team & Database settings panel with live latency testing and storage root mount mapping.


