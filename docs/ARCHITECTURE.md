# 🏗️ Berry AI Studio Architecture

**Berry AI Studio** is a high-performance desktop application built on **Tauri 2**, **Rust**, **Vue 3**, and **SQLite**. It uses a multi-crate Rust backend to handle heavy I/O, file system operations, and metadata extraction, while providing a modern Eagle-style 3-Pane Studio UI in the frontend webview.

```
┌─────────────────────────────────────────────────────────────┐
│                 Frontend (Vue 3 + TypeScript)               │
│  - App.vue (Studio Layout: Sidebar + Gallery + Inspector)   │
│  - TitleBar.vue & MenuBar.vue (Frameless Desktop Navigation)│
│  - VirtualGrid.vue (Virtualized Waterfall Scrolling Canvas) │
│  - LightboxModal.vue (Fullscreen Immersive Quick Look)      │
│  - InspectorPane.vue (Tokenized Prompts & Parameters)       │
│  - UpdateModal.vue (GitHub Releases SemVer Updater)         │
│  - Modals: Settings, FilterDrawer, TagModal, AlbumModal     │
│  - i18n Localization Engine (7 Locales + Auto OS Tracking)   │
└──────────────────────────────┬──────────────────────────────┘
                               │ Tauri IPC (@tauri-apps/api/core)
┌──────────────────────────────▼──────────────────────────────┐
│             App Shell Crate (`src-tauri/`)                  │
│  - lib.rs (Tauri setup, SQLite connection lifecycle)        │
│  - commands.rs (40+ type-safe IPC command endpoints)        │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
┌──────────────▼──────────────┐┌──────────────▼───────────────┐
│        `berry-scan`         ││       `berry-metadata`       │
│ - Multi-threaded directory  ││ - PNGInfo parameters parser  │
│   scanner (walkdir)         ││ - ComfyUI workflow JSON tree │
│ - Incremental mtime cache   ││ - NovelAI comment signatures │
│ - Batch SQLite transactions ││ - Fooocus & InvokeAI formats │
│ - Progress event streaming  ││ - EXIF extraction & Sidecars │
└──────────────┬──────────────┘└──────────────┬───────────────┘
               │                              │
┌──────────────▼──────────────────────────────▼───────────────┐
│                        `berry-domain`                       │
│  - Shared domain models: ImageFile, Folder, Album, Tag      │
│  - ExtractedMetadata, CheckpointModelStat, SearchCriteria   │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                       `berry-storage`                       │
│  - Embedded SQLite database engine (rusqlite)               │
│  - Version-controlled schema migrations (`PRAGMA user_ver`) │
│  - Full-text & structured metadata search engine            │
│  - Database maintenance: VACUUM compaction & VACUUM INTO    │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Multi-Crate Workspace Layout

The Rust backend is structured as a modular Cargo workspace rooted at `/Cargo.toml`:

| Crate Path | Role & Responsibility | Key Dependencies |
| :--- | :--- | :--- |
| **`src-tauri/`** | Thin application shell. Manages window state, frameless window decorations, app lifecycle, and exposes IPC endpoints to the frontend. | `tauri`, `tauri-plugin-dialog`, `trash`, internal crates |
| **`crates/berry-domain/`** | Pure domain models, value objects, metadata formats (`MetadataFormat`), sort criteria, and error types. Zero I/O dependencies. | `serde`, `serde_json` |
| **`crates/berry-metadata/`** | Container sniffers (`detect_container`) and metadata extractors for WebUI (A1111/SD.Next), ComfyUI, NovelAI, Fooocus, InvokeAI, EasyDiffusion, and `.txt` sidecars. | `berry-domain`, `kamadak-exif`, `serde_json` |
| **`crates/berry-scan/`** | Fast recursive filesystem scanner (`Scanner`). Uses incremental fingerprinting `(size_bytes, modified_at)` to skip unchanged files, batches database upserts, and emits progress events. | `berry-domain`, `berry-metadata`, `berry-storage`, `walkdir` |
| **`crates/berry-storage/`** | SQLite persistence layer. Owns schema migrations (`MIGRATIONS`), structured metadata indexing, multi-term query builder, model hash reverse cache, and live database backup/restore. | `berry-domain`, `rusqlite` |

---

## 🗄️ Database & Schema Management

All user metadata, albums, tags, and cached checkpoint models are persisted in a local SQLite database (`berry.db`) located in the OS standard application data directory.

- **Schema Evolution**: Handled via `PRAGMA user_version` migrations. All schema transitions are strictly incremental, atomic, and defined in `crates/berry-storage/src/migrations.rs`.
- **Concurrency & WAL**: SQLite operates in `WAL` (Write-Ahead Logging) mode, enabling non-blocking reads during background filesystem scanning.
- **Maintenance & Safety**: Supports live runtime `VACUUM` compaction and non-locking snapshot export via `VACUUM INTO`.

---

## 🎨 Frontend Architecture

The frontend is built with **Vue 3 Composition API** + **TypeScript** + **Vite**:

### 1. Studio Layout Architecture
- **Frameless Window (`TitleBar.vue`)**: Implements custom Windows/macOS/Linux frameless window controls with `@tauri-apps/api/window` and titlebar dragging.
- **Integrated MenuBar (`MenuBar.vue`)**: Desktop dropdown menus (`File`, `Edit`, `View`, `Tools`, `Help`) with clean single-language rendering.
- **Left Navigation (`Sidebar.vue`)**: Collapsible navigation bar managing library views, recursive folder trees, color tags, and smart albums.
- **Center Canvas (`VirtualGrid.vue` & `FileList.vue`)**: Virtualized grid rendering tens of thousands of items with dynamic column resizing, smooth thumbnail zoom slider, and list view toggle.
- **Right Property Inspector (`InspectorPane.vue`)**: Tokenized positive/negative prompt chips with one-click copy, model specs table, and collapsible raw JSON viewer.
- **Quick Look Lightbox (`LightboxModal.vue`)**: Immersive fullscreen viewer with pan, zoom, and keyboard navigation.

### 2. State & Localization
- **Reactive i18n (`src/i18n/`)**: Lightweight reactive internationalization supporting 7 locales (`en`, `zh-CN`, `zh-TW`, `ja`, `de`, `fr`, `es`) and automatic OS language detection (`auto`).
- **Updater (`src/utils/updater.ts` & `UpdateModal.vue`)**: SemVer comparison against GitHub Releases API with automated asset matching and release notes rendering.

---

## ⚡ Multi-Mode Folders & AIGC Ingestion Pipeline

Berry AI Studio extends conventional folder management into three high-performance modes (see [INGESTION_AND_STACKING.md](INGESTION_AND_STACKING.md) for full specifications):
- **Link Folders (`link`)**: Zero-copy in-place file surveillance without file movement.
- **Managed Vaults (`managed`)**: Managed repository supporting direct drag-and-drop Copy or Move ingestion.
- **AIGC Ingestion Pipeline (`pipeline`)**: Automated surveillance of WebUI, ComfyUI, and Fooocus output directories with write-lock debouncing, atomic ingest to the library, and non-destructive delayed cleanup (grace period) moving aged source files to the OS Recycle Bin.

---

## 🗃️ Image Stacking Architecture

To solve the "AI Burst / Roll" gallery clutter problem:
- **Burst Clustering**: Groups batch generation variants by exact or high-similarity prompts (Jaccard threshold) constrained within a temporal generation window.
- **Manual Stacking**: Full keyboard-driven grouping via `Ctrl+G` (stack) and `Ctrl+Shift+G` (unstack).
- **Poker Deck Presentation**: Collapsed stack presentation with item count badges (`📚 N`), inline expansion, hero cover selection (`Alt+S`), and side-by-side comparison (`C`).

