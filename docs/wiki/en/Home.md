# Berry AI Studio — Official Wiki & User Guide

Welcome to the definitive user documentation and knowledge base for **Berry AI Studio** (`v0.3.0`).

Berry AI Studio is an open-source, local-first asset manager and prompt workbench engineered specifically for generative AI creators, prompt engineers, and visual design studios. Built on **Tauri v2**, **Rust**, and **Vue 3**, it handles libraries ranging from a few hundred artworks to 500,000+ files with sub-millisecond query latency, zero cloud dependency, and comprehensive generation metadata extraction.

---

## 🧭 Navigation & Table of Contents

### [Chapter 1: Getting Started & Fundamentals](01-getting-started/installation.md)
- **[Installation & System Requirements](01-getting-started/installation.md)**: Hardware prerequisites, Windows setup/portable options, macOS Universal & Apple Silicon builds, Linux AppImage/deb, and the First-Run Onboarding Wizard.
- **[Workspace & UI Anatomy](01-getting-started/workspace-layout.md)**: Detailed breakdown of the frameless window, application menu bar, 3-pane layout (Sidebar, Gallery, Inspector), Status Bar, and floating Batch Action Bar.
- **[Keyboard Shortcuts Cheatsheet](01-getting-started/keyboard-shortcuts.md)**: Global shortcuts, selection anchors, blind star-rating, quick inspection, and navigation hotkeys.

### [Chapter 2: Library Management & Browsing](02-library-management/folder-modes-and-import.md)
- **[Importing Media & Folder Modes](02-library-management/folder-modes-and-import.md)**: Comparing Mode A (External Link), Mode B (Managed Vault), and Mode C (AIGC Pipeline with debounced watch & delayed recycling). Supported image (PNG, WebP, JPEG) and video (MP4, WebM) formats.
- **[Gallery Modes & Display Options](02-library-management/gallery-views.md)**: Mastering Uniform Grid (with 130px–360px zoom), Masonry Waterfall (uncropped aspect ratios), Table View, and Visual Similarity match view. Card badges and NSFW privacy blur.
- **[Organization, Ratings & Tags](02-library-management/organization-and-tags.md)**: 10-star rating scale, favorites, custom albums, 8-color tag taxonomy, batch drag-and-drop, and batch action toolbar.
- **[Video & Motion Media Support](02-library-management/video-support.md)**: AnimateDiff, Wan2.1, HunyuanVideo, and SVD playback; frame stepping, loop/speed HUD, and embedded video workflow inspection.

### [Chapter 3: Discovery, Search & Analytics](03-discovery-and-analytics/search-and-filtering.md)
- **[Search Syntax & Visual Filtering](03-discovery-and-analytics/search-and-filtering.md)**: Advanced key-value query language (`prompt:`, `neg:`, `model:`, `cfg:>=7`, `steps:20..40`), numeric ranges, and the slide-out Filter Drawer.
- **[AIGC Metadata & Prompt Inspection](03-discovery-and-analytics/metadata-and-prompts.md)**: Lossless parsing for AUTOMATIC1111, ComfyUI, NovelAI, Fooocus, InvokeAI; tokenized interactive prompt chips, and raw execution graphs.
- **[Prompt Analytics & Insights](03-discovery-and-analytics/prompt-insights.md)**: Library-wide token frequency distributions, positive/negative prompt rankings, and correlation with user ratings.

### [Chapter 4: Intelligent Curation & AI Engines](04-intelligent-curation/stacks-and-bursts.md)
- **[Image Stacks, Bursts & Comparison](04-intelligent-curation/stacks-and-bursts.md)**: Automatic burst clustering (Jaccard prompt similarity + time windows), poker-deck stack cards, hero covers, safe stack flattening, Cull Drafts tool, and Side-by-Side (`C`) compare mode.
- **[AI Semantic Search & Auto-Tagging](04-intelligent-curation/ai-semantic-and-tagger.md)**: Local ONNX CLIP/SigLIP text-to-image natural language queries, image-to-image visual nearest neighbors, and WD14 Danbooru anime auto-tagging.
- **[Checkpoint Models & LoRA Library](04-intelligent-curation/models-and-loras.md)**: Automatic checkpoint cataloging, A1111 `cache.json` hash resolution, Civitai reverse lookup, LoRA trigger word manager, and 1-click prompt injection.
- **[Generation Tool Interoperability](04-intelligent-curation/generation-interop.md)**: Direct API communication with ComfyUI (`/prompt`) and AUTOMATIC1111 (`/sdapi/v1/txt2img`), with live connection health checking.

### [Chapter 5: Export, Cloud Backup & Collaboration](05-export-and-collaboration/export-and-web-showcase.md)
- **[Batch Export, Transcoding & Web Showcase](05-export-and-collaboration/export-and-web-showcase.md)**: Multi-threaded Rayon transcoding, 4-tier privacy metadata sanitization, dynamic filename templating, ZIP archives, and self-contained interactive single-file HTML showcase generation.
- **[Cloud Snapshot Backup & Media Mirroring](05-export-and-collaboration/cloud-backup-and-sync.md)**: Hot SQLite `VACUUM INTO` snapshots to AWS S3, Cloudflare R2, MinIO, WebDAV, or local NAS; incremental delta sync with ETag/SHA-256 detection and bandwidth throttling.
- **[Multi-Database Team Studio](05-export-and-collaboration/team-collaboration.md)**: Scaling beyond SQLite to shared MySQL 8.0+ or PostgreSQL 14+ servers; cross-platform storage root mapping (normalizing Windows drive letters to macOS/Linux paths), optimistic concurrency control, and client-side NVMe thumbnail caching.

### [Chapter 6: System Reference & Maintenance](06-reference-and-maintenance/settings-reference.md)
- **[Comprehensive Settings Reference](06-reference-and-maintenance/settings-reference.md)**: Complete parameter guide across all 8 preferences tabs.
- **[Database & Cache Maintenance](06-reference-and-maintenance/database-maintenance.md)**: SQLite WAL compaction (`VACUUM`), database backup/restoration, thumbnail cache budget (LRU eviction), and queue diagnostics.
- **[Updates & Lifecycle](06-reference-and-maintenance/updating.md)**: In-app auto-updater, data preservation guarantees across versions, and manual release upgrades.
- **[Privacy & Security Architecture](06-reference-and-maintenance/privacy-and-security.md)**: 100% offline-first model, zero telemetry, local AI inference isolation, and AGPL-3.0 licensing.
- **[Troubleshooting & FAQ](06-reference-and-maintenance/troubleshooting-and-faq.md)**: Solutions to common issues, performance optimization tips, and frequently asked questions.
- **[Product Glossary](06-reference-and-maintenance/glossary.md)**: Authoritative definitions for product domain terms (Hero, Ingestion Pipeline, Jaccard similarity, Keyset cursor, OCC, Poker-deck card, etc.).

---

## ⚡ Quick Start Shortcuts

| Action | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **Open Lightbox (Quick Look)** | `Space` / `Enter` | `Space` / `Return` |
| **Side-by-Side Compare** | `C` | `C` |
| **Group into Stack** | `Ctrl + G` | `Cmd + G` |
| **Ungroup Stack** | `Ctrl + Shift + G` | `Cmd + Shift + G` |
| **Set as Stack Hero** | `Alt + S` | `Option + S` |
| **Rate Selected 1–5 Stars** | `1` – `5` (`0` to clear) | `1` – `5` (`0` to clear) |
| **Toggle Favorite** | `F` | `F` |
| **Focus Search Bar** | `/` or `Ctrl + F` | `/` or `Cmd + F` |
| **Toggle Inspector Pane** | `I` | `I` |
| **Toggle Sidebar** | `B` | `B` |
| **Open Preferences** | `Ctrl + ,` | `Cmd + ,` |
