<div align="center">

# 🍇 Berry AI Studio

**The open-source asset manager & prompt studio for AI-generated images.**

Organize, search, compare, and export tens of thousands of AIGC artworks — all from a blazing-fast local desktop app.

<br/>

**[English](README.md)** | **[简体中文](README.zh-CN.md)** | **[繁體中文](README.zh-TW.md)** | **[日本語](README.ja.md)**

<br/>

<a href="https://github.com/BerryUIKI/Berry-AI-Studio/stargazers"><img src="https://img.shields.io/github/stars/BerryUIKI/Berry-AI-Studio?style=social" alt="GitHub Stars"></a>&nbsp;&nbsp;
<a href="https://github.com/BerryUIKI/Berry-AI-Studio/network/members"><img src="https://img.shields.io/github/forks/BerryUIKI/Berry-AI-Studio?style=social" alt="GitHub Forks"></a>&nbsp;&nbsp;
<a href="https://github.com/BerryUIKI/Berry-AI-Studio/issues"><img src="https://img.shields.io/github/issues/BerryUIKI/Berry-AI-Studio?style=social&logo=github" alt="GitHub Issues"></a>

<br/>

[![Release](https://img.shields.io/github/v/release/BerryUIKI/Berry-AI-Studio?display_name=tag&style=flat-square&color=blue)](https://github.com/BerryUIKI/Berry-AI-Studio/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/BerryUIKI/Berry-AI-Studio/total?style=flat-square&color=green)](https://github.com/BerryUIKI/Berry-AI-Studio/releases)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Website](https://img.shields.io/badge/website-GitHub%20Pages-12b5cb?style=flat-square)](https://berryuiki.github.io/Berry-AI-Studio/)

[![Tauri 2](https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?style=flat-square&logo=vue.js&logoColor=white)](https://vuejs.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)](#-downloads)

<br/>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/gui_preview_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/screenshots/gui_preview_light.svg">
  <img alt="Berry AI Studio — 3-Pane Studio Workspace" src="docs/screenshots/gui_preview_dark.svg" width="100%">
</picture>

</div>

<br/>

## 🚀 Quick Start

1. **Download** — Grab the [latest release](https://github.com/BerryUIKI/Berry-AI-Studio/releases/latest) for your platform.
2. **Add a Folder** — Point Berry at your WebUI / ComfyUI / NovelAI output directory. Choose *External Link* (zero-copy) or *AIGC Pipeline* (auto-harvest) mode.
3. **Browse & Create** — Your library is instantly searchable. Rate, tag, compare, and export with full metadata preserved.

---

## 🆕 What's New in v0.3.0

> **Cloud Sync & Export**, **Keyset Deep Pagination**, and **Multi-Database Team Studio** — [Read the full changelog →](CHANGELOG.md)

- ☁️ **S3 / WebDAV Snapshot Backup & Restore** — Automated hot `VACUUM INTO` snapshots to AWS S3, R2, MinIO, Backblaze B2, and WebDAV (Nextcloud / Synology).
- 📤 **Standalone HTML Showcase Export** — Zero-dependency single `index.html` with dark-theme gallery, fullscreen lightbox, prompt inspector, and keyword filter.
- 🔄 **Incremental Remote Delta Sync** — ETag + streaming SHA-256 bidirectional sync with token-bucket bandwidth throttle.
- ⚡ **O(1) Keyset Cursor Pagination** — < 1 ms page traversal at 500,000+ assets (84× faster than OFFSET).
- 👥 **Multi-Database Team Studio** — Abstract `StorageEngine` supporting SQLite, MySQL 8.0+, and PostgreSQL 14+ with optimistic concurrency and real-time collaboration sync.
- 📦 **Batch Transcoding & Privacy Strip** — Multi-threaded WebP/JPEG/PNG conversion with 4-tier metadata stripping (`KeepAll` → `StripAll`).

---

## ✨ Key Features

### 🎨 All-in-One 3-Pane Studio Workspace

- **Frameless Window with Native Quality** — Custom title bar with integrated desktop menu bar (`File`, `Edit`, `View`, `Tools`, `Help`), drag region, and window controls.
- **Left Navigation Sidebar** — Quick filters (All Images, Favorites, Sensitive 18+), hierarchical folder tree with real-time scan indicators, color-coded tags, and smart albums.
- **Center Canvas — Grid · Waterfall · Table** — Ultra-fast virtual scrolling rendering tens of thousands of images. Masonry Waterfall view, smooth thumbnail zoom slider (130 px – 360 px), and one-click ⊞ / ▦ / ☰ view switcher.
- **Stable Responsive Density** — Fixed card widths during resize; columns are added or removed instead of stretching images.
- **Right Property Inspector** — Large preview card, star ratings (0–5), favorite toggle, tokenized prompt chips with one-click copy, LoRA tags, generation parameters, and collapsible raw workflow JSON.
- **Immersive Lightbox (Quick Look)** — Full-screen viewer (`Space` / `Enter`) with smooth mouse-wheel zoom, pan, and keyboard navigation.

### 🔍 Lossless AIGC Metadata Parsers

Automatically extracts and indexes Prompt, Negative Prompt, Model, Hash, Sampler, Steps, CFG, Seed, Dimensions, and full Workflow JSON:

| Platform | Source |
|:---|:---|
| **AUTOMATIC1111 / SD.Next** | PNG `tEXt`/`iTXt` parameters, WebP EXIF |
| **ComfyUI** | Full prompt & workflow graph JSON, LoRA Loader detection |
| **NovelAI** | Comment & Description signature decoding |
| **Fooocus / Fooocus-MRE** | Parameter parsing & model resolution |
| **InvokeAI & EasyDiffusion** | Embedded metadata & JSON sidecars |
| **Sidecar Files** | `.txt` companion metadata |

**Supported formats:** PNG · JPG/JPEG · WebP · MP4

### 🗃️ Multi-Mode Ingestion & Smart Stacking

- **External Link** — Reference in-place, zero-copy.
- **Managed Vault** — Dedicated app-managed storage.
- **AIGC Pipeline** — Automatic background harvest from WebUI/ComfyUI output folders with debounced ingestion.
- **Intelligent Burst Stacking** — Auto-groups sequential generations with similar prompts.
- **Interactive Stack Cards** — Poker-deck visual cards with badge counts, inline expand/collapse, cover selection (`Alt+S`), side-by-side compare (`C`), and safe transactional stack merging.

### 🧠 AI Intelligence

- **Local CLIP / SigLIP Semantic Search** — Search images using natural language via local ONNX models — no cloud required.
- **WD14 / Danbooru Auto-Tagging** — Automated anime & realistic tag extraction with confidence thresholds.
- **Visual Similarity Search** — Find compositionally similar images from any starting point.
- **LoRA Trigger Word Library** — Scan LoRA directories, match Civitai metadata, and one-click inject trigger words.
- **Prompt Keyword Insights** — Frequency analysis of prompt tokens with average rating correlations.
- **Checkpoint Model Manager** — Civitai SHA256 cache sync, reverse hash lookup, and one-click model filtering.

### ☁️ Cloud Sync & Export

- **S3 / WebDAV Snapshot Backup** — Hot `VACUUM INTO` snapshots to AWS S3, Cloudflare R2, MinIO, Backblaze B2, and WebDAV servers with schema verification and rollback safety.
- **Incremental Remote Delta Sync** — ETag + streaming SHA-256 change detection, bidirectional mirroring, token-bucket bandwidth throttle, and atomic cancellation.
- **Batch Export & Transcoding** — Multi-threaded WebP/JPEG/PNG conversion with 4-tier privacy metadata stripping and customizable filename templates.
- **HTML Showcase Generator** — Standalone zero-dependency `index.html` with responsive dark gallery, fullscreen lightbox, prompt inspector, and instant keyword filter.

### 👥 Multi-Database Team Studio

- **Storage Engine Abstraction** — `StorageEngine` trait supporting SQLite (default), MySQL 8.0+, and PostgreSQL 14+.
- **Cross-Platform Storage Roots** — Path normalization for multi-user collaboration across Windows, macOS, and Linux.
- **Optimistic Concurrency** — Row-level version tracking with last-write-wins and set-union conflict resolution.
- **Real-Time Collaboration** — Zero-DevOps change log journal polling engine with automatic cadence adaptation.

### 🌐 Internationalization & Auto-Update

- **7 Languages** — English · 简体中文 · 繁體中文 · 日本語 · Deutsch · Français · Español
- **Auto OS Language Detection** — Follows your system language by default.
- **Built-in Updater** — Check for updates directly from `Help > Check for Updates…` with release notes and one-click download.

---

## ⚡ Performance

Berry AI Studio is built for large libraries. The gallery renders from the indexed SQLite database before any filesystem reconciliation begins — no blocking startup scan.

| Metric | 1 k images | 10 k images | 50 k images |
|:---|:---:|:---:|:---:|
| Time to first usable gallery (warm DB) | **4.48 ms** | **23.65 ms** | **143.55 ms** |
| Main-thread long tasks (> 50 ms) during rapid scrollbar drag | **0** | **0** | **0** |
| Thumbnail manifest adoption | — | — | **3.26 s** (15,300 files/sec) |

**Additional safeguards:**

- **O(1) Keyset Cursor Pagination** — < 1 ms per page at 500 k+ assets (84× faster than `OFFSET`).
- **Frame-Coalesced Scrolling** — At most 1 reactive update per `requestAnimationFrame`.
- **Per-Column Binary Search** — Waterfall visibility computed via binary search, not full-library scan.
- **Demand-Driven Thumbnails** — Visible images decoded first; look-ahead starts only after scrolling settles.
- **Directory Fingerprint Gating** — Parent `mtime` check delivers 2.5× local / 5.6× network speedup on re-scans.

📖 See [Performance Architecture & Benchmarks](docs/PERFORMANCE.md) for full details.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Berry AI Studio (Desktop)                    │
├─────────────────────────────────────────────────────────────────┤
│  Vue 3 + TypeScript          │  Tauri 2 IPC Commands           │
│  ─ VirtualGrid / Waterfall   │  ─ Thin adapters, input         │
│  ─ Inspector / Lightbox      │    validation, lock release      │
│  ─ 37 components, 11 utils   │  ─ 50+ endpoints                │
├──────────────────────────────┼──────────────────────────────────┤
│              Modular Rust Crate Workspace                       │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────┐           │
│  │ berry-domain │ │berry-metadata│ │  berry-scan   │           │
│  │ Pure models  │ │ PNG/EXIF/    │ │ Indexer,      │           │
│  │ Zero I/O     │ │ ComfyUI/     │ │ Thumbnails,   │           │
│  │              │ │ NovelAI/     │ │ Export, HTML   │           │
│  │              │ │ Fooocus/...  │ │ Showcase       │           │
│  └──────────────┘ └──────────────┘ └───────────────┘           │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────┐           │
│  │berry-storage │ │ berry-tagger │ │  berry-clip   │           │
│  │ SQLite/MySQL │ │ WD14 ONNX    │ │ CLIP/SigLIP   │           │
│  │ PostgreSQL   │ │ Danbooru     │ │ Text & Vision │           │
│  │ Migrations   │ │ Tagging      │ │ Embeddings    │           │
│  └──────────────┘ └──────────────┘ └───────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

**Tech Stack:** Tauri 2 · Rust (2021 edition) · Vue 3.5 · TypeScript 5.6 · Vite 6 · SQLite (WAL) · Rayon · ONNX Runtime

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action | Shortcut | Action |
|:---|:---|:---|:---|
| `Space` / `Enter` | Open / Close Lightbox | `0` – `5` | Set Star Rating (0 = Clear) |
| `F` | Toggle Favorite | `B` | Toggle Sidebar |
| `I` | Toggle Inspector | `/` or `Ctrl+F` | Focus Search Bar |
| `Ctrl+A` | Select All | `Esc` | Clear Selection / Close Modal |
| `Ctrl+O` | Folder Wizard | `Ctrl+,` | Settings |
| `Ctrl+G` | Merge Stacks | `Ctrl+Shift+G` | Unstack |
| `Alt+S` | Set Stack Cover | `C` | Side-by-Side Compare |
| `Delete` | Move to Trash | `?` | Shortcuts Guide |

---

## 📥 Downloads

Get the latest pre-built binaries from **[GitHub Releases](https://github.com/BerryUIKI/Berry-AI-Studio/releases/latest)**:

| Platform | Architecture | Format | File Name |
|:---|:---|:---|:---|
| **Windows** | x86_64 | NSIS Installer | `Berry-AI-Studio_Windows_x64.exe` |
| **Windows** | x86_64 | Portable Zip | `Berry-AI-Studio_Windows_x64.zip` |
| **macOS** | Apple Silicon | DMG | `Berry-AI-Studio_macOS_aarch64.dmg` |
| **macOS** | Intel x86_64 | DMG | `Berry-AI-Studio_macOS_x64.dmg` |
| **Linux** | x86_64 | AppImage | `Berry-AI-Studio_Linux_x64.AppImage` |
| **Linux** | x86_64 | Debian | `Berry-AI-Studio_Linux_x64.deb` |

---

## 🛠️ Building from Source

### Prerequisites

- **Node.js** v18+ & **pnpm** — `npm install -g pnpm`
- **Rust** 1.75+ — Install via [rustup.rs](https://rustup.rs/)
- **Platform Toolchain** — MSVC Build Tools (Windows), Xcode CLI (macOS), `libwebkit2gtk-4.1` (Linux)

### Build

```bash
# Clone
git clone https://github.com/BerryUIKI/Berry-AI-Studio.git
cd Berry-AI-Studio

# Install dependencies
pnpm install

# Development (hot-reload)
pnpm run tauri dev

# Production build
pnpm run tauri build
```

Output: `src-tauri/target/release/bundle/`

---

## 🗺️ Roadmap

| Milestone | Planned |
|:---|:---|
| 🎬 Video & Animation AIGC Ingestion (AnimateDiff, Wan2.1, HunyuanVideo) | v0.4.0 |
| 🧬 Next-Gen Model Architectures (Flux.1, SD3.5, Civitai API) | v0.5.0 |
| 🔗 Bi-Directional ComfyUI Studio (WebSocket telemetry, prompt diff) | v0.6.0 |
| 📱 Local LAN Web Companion ("Berry Remote" for tablets & phones) | v0.7.0 |
| 🎨 Dominant Color Palette Indexing & Faceted Analytics | v0.8.0 |

📖 See [docs/ROADMAP.md](docs/ROADMAP.md) for full details.

---

## 🤝 Contributing

We welcome bug reports, feature requests, new metadata parsers, and translation contributions!

1. Fork the repo and branch from `dev` (not `main`).
2. Read the [Contributing Guide](CONTRIBUTING.md) and [Agent Guidelines](AGENTS.md).
3. Ensure all checks pass before opening a PR:

```bash
pnpm run build          # Frontend build
pnpm run test:stack     # Behavior tests
cargo clippy --workspace -- -D warnings
cargo test --workspace  # Rust tests
```

---

## 📄 License

This project is licensed under the **[AGPL-3.0 License](LICENSE)**.

Copyright © 2026 [BerryUIKI](https://github.com/BerryUIKI).

---

## 🙏 Acknowledgments

Berry AI Studio is built on the shoulders of these excellent open-source projects:

- [Tauri](https://tauri.app) — Lightweight cross-platform app framework
- [Vue.js](https://vuejs.org) — Progressive JavaScript framework
- [Rust](https://www.rust-lang.org) & [Rayon](https://github.com/rayon-rs/rayon) — Fearless concurrency
- [rusqlite](https://github.com/rusqlite/rusqlite) — Ergonomic SQLite bindings
- [ONNX Runtime](https://onnxruntime.ai) — Cross-platform ML inference
- [CLIP](https://github.com/openai/CLIP) & [SigLIP](https://arxiv.org/abs/2303.15343) — Vision-language models
- [WD14 Tagger](https://huggingface.co/SmilingWolf) — Anime image classification

---

<div align="center">

**[⬆ Back to Top](#-berry-ai-studio)**

If you find Berry AI Studio useful, please consider giving it a ⭐ — it helps others discover the project!

<a href="https://star-history.com/#BerryUIKI/Berry-AI-Studio&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=BerryUIKI/Berry-AI-Studio&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=BerryUIKI/Berry-AI-Studio&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=BerryUIKI/Berry-AI-Studio&type=Date" width="600" />
  </picture>
</a>

<sub>Made with ❤️ by the Berry AI Studio community</sub>

</div>
