# 🍇 Berry AI Studio

<div align="center">

**[English](README.md)** | **[简体中文](README.zh-CN.md)** | **[繁體中文](README.zh-TW.md)** | **[日本語](README.ja.md)**

[![Website](https://img.shields.io/badge/website-GitHub%20Pages-12b5cb.svg)](https://berryuiki.github.io/Berry-AI-Studio/)
[![Release](https://img.shields.io/badge/release-v0.2.0-blue.svg)](https://github.com/BerryUIKI/Berry-AI-Studio/releases/tag/v0.2.0)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-24c8db)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.7+-orange)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-42b883)](https://vuejs.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/BerryUIKI/Berry-AI-Studio/releases)

*A high-performance, open-source metadata indexer and asset management studio for AI-generated images.*

<br/>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/gui_preview_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/screenshots/gui_preview_light.svg">
  <img alt="Berry AI Studio Preview" src="docs/screenshots/gui_preview_dark.svg" width="100%">
</picture>

</div>

---

## 🌟 Overview

**Berry AI Studio** is a modern, desktop-first asset manager built for digital artists, AI creators, and prompt engineers. It indexes and parses prompt metadata and generation parameters across all major AI image platforms into a fast, local SQLite database, providing an **All-in-One 3-Pane Studio Workspace** with smooth virtual grid navigation, tokenized prompt inspectors, instant full-screen lightbox preview, and smart categorization.

> 🚀 **v0.2.0 Release**: Introducing local CLIP semantic search, AI auto-tagging, LoRA trigger word library, multi-mode ingestion pipelines, burst image stacking, and a virtualized waterfall gallery.

---

## ✨ Key Features

### 🎨 All-in-One 3-Pane Studio Workspace
- **Frameless Window with Native Quality**: Custom frameless title bar with integrated desktop menu bar (`File`, `Edit`, `View`, `Tools`, `Help`), drag region, and window controls.
- **Left Navigation Sidebar**: Quick filters (All Images, Favorites, Sensitive 18+), hierarchical folder tree with real-time scan indicators, color-coded tags, and smart albums.
- **Center Canvas, Virtual Grid & Waterfall**: Ultra-fast virtual scrolling rendering tens of thousands of images, masonry Waterfall view mode, smooth thumbnail zoom slider (130px–360px), and Grid (⊞) / Waterfall (▦) / Table (☰) view switcher.
- **Stable Responsive Density**: Grid and Waterfall preserve the selected card width as the window changes, adding or removing columns instead of stretching images.
- **Right Property Inspector**: Dedicated inspector pane displaying large preview cards, star ratings (0–5), favorite toggle, tokenized prompt chips with one-click copy, detected LoRA tags, generation specs, and collapsible raw workflow JSON.
- **Immersive Quick Look (Lightbox)**: Full-screen viewer (`Space` / `Enter`) with smooth mouse-wheel zoom, pan, and keyboard navigation.

### 🔍 Lossless AIGC Metadata Parsers
Automatically extracts and indexes generation parameters (Prompt, Negative Prompt, Model Name, Model Hash, Sampler, Steps, CFG Scale, Seed, Dimensions, Workflow JSON):
- **WebUI (AUTOMATIC1111 / SD.Next)**: PNG `tEXt`/`iTXt` parameters chunks, WebP EXIF.
- **ComfyUI**: Full prompt and workflow graph JSON parsing with LoRA loader detection.
- **NovelAI**: Comment and description signature decoding.
- **Fooocus / Fooocus-MRE**: Parameter parsing and model resolution.
- **InvokeAI & EasyDiffusion**: Embedded metadata & JSON sidecars.
- **Supported Formats**: PNG, JPG/JPEG, WebP, MP4, and `.txt` sidecar metadata.

### 🗃️ Multi-Mode Ingestion & Smart Stacking
- **Multi-Mode Folders**: External Link mode (reference in-place), Managed Vaults (dedicated storage), and AIGC Pipeline mode (automatic background harvest from WebUI/ComfyUI output folders).
- **Intelligent Burst Stacking**: Automatically detects and groups sequential generations with similar prompts.
- **Interactive Stack Cards**: Poker deck visual cards with badge counts, inline expand/collapse, cover selection (`Alt+S`), side-by-side compare mode (`C`), and transactional stack flattening with safety confirmation.

### 🧠 Model, Prompt & AI Intelligence
- **Local CLIP Semantic Search**: Search images using natural language prompts powered by local ONNX CLIP/SigLIP models.
- **AI Tagging & Visual Similarity**: Automated WD14 / Danbooru anime tagging and reverse image similarity search.
- **LoRA Trigger Word Library**: Scan LoRA directories, match Civitai metadata, and auto-inject trigger words into prompts.
- **Prompt Keyword Insights**: Statistical frequency analysis of prompt tokens with average rating correlations.
- **Checkpoint Model Manager**: Civitai SHA256 cache synchronization, reverse hash lookup, and one-click filtering by model.
- **Database Maintenance**: Built-in SQLite VACUUM optimization, backup export, and restoration.

### 🌐 Internationalization & Auto-Update
- **7 Languages Supported**: English, 简体中文, 繁體中文, 日本語, Deutsch, Français, Español.
- **Auto System Language Detection**: Follows OS language by default (`Auto`).
- **GitHub Releases Updater**: Check for updates directly from **Help > Check for Updates...** with release notes and one-click download.

### ⚡ Large-Library Performance

- **Fast First Paint**: The indexed SQLite library is shown before optional filesystem reconciliation begins.
- **Controlled Startup Scans**: Startup scanning is opt-in for new installations and uses a configurable per-folder cooldown.
- **Demand-Driven Thumbnails**: Visible images are prioritized; deduplicated background look-ahead starts only after scrolling settles.
- **Bounded Virtualization**: Scroll updates are frame-coalesced, and Waterfall visibility uses per-column search rather than a full-library scan.

See the [performance architecture and optimization plan](docs/PERFORMANCE.md) for benchmarks, tradeoffs, and prioritized follow-up work.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action | Shortcut | Action |
| :--- | :--- | :--- | :--- |
| `Space` / `Enter` | Open / Close Fullscreen Lightbox | `0` ~ `5` | Set Star Rating (0 = Clear) |
| `F` | Toggle Favorite | `B` | Toggle Left Navigation Sidebar |
| `I` | Toggle Right Property Inspector | `/` or `Ctrl+F` | Focus Search Bar |
| `Ctrl+A` | Select All Visible Images | `Esc` | Clear Selection / Close Modals |
| `Ctrl+O` | Folder Creation Wizard | `Ctrl+,` | Preferences & Settings |
| `Ctrl+G` | Group / Merge Stacks | `Ctrl+Shift+G` | Unstack / Dissolve Stack |
| `Alt+S` | Set Stack Hero Cover | `C` | Side-by-Side Compare Mode |
| `Delete` | Move Selected to Recycle Bin | `?` | Keyboard Shortcuts Guide |

---

## 📦 Release Package Naming Convention

Official pre-built binaries on [GitHub Releases](https://github.com/BerryUIKI/Berry-AI-Studio/releases) follow the standardized naming convention:

$$\text{<AppName>}\_\text{<OS>}\_\text{<Architecture>}.\text{<extension>}$$

| Platform / OS | Architecture | Package Format | Release Asset File Name |
| :--- | :--- | :--- | :--- |
| **Windows** | x86_64 (64-bit) | NSIS Installer | `Berry-AI-Studio_Windows_x64.exe` |
| **Windows** | x86_64 (64-bit) | Portable Zip | `Berry-AI-Studio_Windows_x64.zip` |
| **macOS** | Apple Silicon (ARM64) | DMG Disk Image | `Berry-AI-Studio_macOS_aarch64.dmg` |
| **macOS** | Intel (x86_64) | DMG Disk Image | `Berry-AI-Studio_macOS_x64.dmg` |
| **Linux** | x86_64 (64-bit) | AppImage | `Berry-AI-Studio_Linux_x64.AppImage` |
| **Linux** | x86_64 (64-bit) | Debian Package | `Berry-AI-Studio_Linux_x64.deb` |

---

## 🛠️ Building from Source

### Prerequisites
1. **Node.js** (v18+) & **pnpm** (`npm install -g pnpm`)
2. **Rust** (1.75+): Install via [rustup.rs](https://rustup.rs/)
3. **C++ Build Tools**: MSVC Build Tools on Windows, Xcode CLI Tools on macOS, `libwebkit2gtk-4.1` on Linux.

### Steps
```bash
# 1. Clone the repository
git clone https://github.com/BerryUIKI/Berry-AI-Studio.git
cd Berry-AI-Studio

# 2. Install frontend dependencies
pnpm install

# 3. Run development mode (Hot-Reload)
pnpm run tauri dev

# 4. Build production installer
pnpm run tauri build
```

The production output will be located in `src-tauri/target/release/bundle/`.

---

## 📄 License

This project is licensed under the **AGPL-3.0 License**. See the [LICENSE](LICENSE) file for details.
