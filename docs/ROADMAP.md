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
  - Virtualized Waterfall gallery layout mode alongside standard grid.

---

## 🔮 Upcoming Milestones (v0.3.0+)

### 🎯 Milestone 11: Generation Interop & Workflows
- [ ] Drag-and-drop workflow send-to-WebUI / send-to-ComfyUI via WebSocket or Local HTTP API.
- [ ] One-click batch cull (keep hero/top-rated images and trash remaining drafts).

### 🎯 Milestone 12: Cloud Sync & Export Utilities
- [ ] Encrypted WebDAV / S3 / LAN database synchronization.
- [ ] Batch format conversion (WebP lossless / JPG) and metadata stripping for publishing.
- [ ] Export curated collections into HTML galleries and ZIP archives.

### 🎯 Milestone 13: Large-Library Performance

- [x] Fixed-width responsive Grid and Waterfall columns.
- [x] Animation-frame scroll coalescing and per-column Waterfall visibility search.
- [x] Deduplicated, serialized thumbnail look-ahead scheduling.
- [x] Startup scan cooldown with opt-in scanning for new installations.
- [x] Bounded incremental gallery queries with exact totals and stale-response protection.
- [ ] Replace offset traversal with keyset cursors if deep-page benchmarks require it.
- [ ] Introduce lightweight gallery DTOs and load full metadata on selection.
- [x] Persistent filesystem watcher journal with targeted path reconciliation and optional recovery scans.
- [x] Persistent size-tiered thumbnail manifest with configurable LRU disk budget.
- [x] Cancelable viewport generations with visible-first and directional look-ahead priority.
- [x] Zoom-aware thumbnail tier selection for Gallery and Table surfaces.
- [ ] Per-job thumbnail queue diagnostics and cached-tier fallback.
