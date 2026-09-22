<div align="center">

# 🍇 Berry AI Studio

**专为 AI 图像创作者与 Prompt 工程师打造的开源资产管理与提示词工作台**

毫秒级检索、智能整理、双图对比与批量导出数以万计的 AIGC 艺术作品 —— 一切都在极致流畅的本地桌面端完成。

<br/>

**[English](README.md)** | **[简体中文](README.zh-CN.md)** | **[繁體中文](README.zh-TW.md)** | **[日本語](README.ja.md)**

<br/>

<a href="https://github.com/BerryUIKI/Berry-AI-Studio/stargazers"><img src="https://img.shields.io/github/stars/BerryUIKI/Berry-AI-Studio?style=social" alt="GitHub Stars"></a>&nbsp;&nbsp;
<a href="https://github.com/BerryUIKI/Berry-AI-Studio/network/members"><img src="https://img.shields.io/github/forks/BerryUIKI/Berry-AI-Studio?style=social" alt="GitHub Forks"></a>&nbsp;&nbsp;
<a href="https://github.com/BerryUIKI/Berry-AI-Studio/issues"><img src="https://img.shields.io/github/issues/BerryUIKI/Berry-AI-Studio?style=social&logo=github" alt="GitHub Issues"></a>

<br/>

[![Release](https://img.shields.io/github/v/release/BerryUIKI/Berry-AI-Studio?display_name=tag&style=flat-square&color=blue)](https://github.com/BerryUIKI/Berry-AI-Studio/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/BerryUIKI/Berry-AI-Studio/total?style=flat-square&color=green)](https://github.com/BerryUIKI/Berry-AI-Studio/releases)
[![License](https://img.shields.io/badge/协议-AGPL--3.0-blue?style=flat-square)](LICENSE)
[![Website](https://img.shields.io/badge/官方网站-GitHub%20Pages-12b5cb?style=flat-square)](https://berryuiki.github.io/Berry-AI-Studio/)

[![Tauri 2](https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.75+-f74c00?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Vue 3](https://img.shields.io/badge/Vue-3-42b883?style=flat-square&logo=vue.js&logoColor=white)](https://vuejs.org)
[![Platform](https://img.shields.io/badge/平台-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)](#-下载安装)

<br/>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/gui_preview_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/screenshots/gui_preview_light.svg">
  <img alt="Berry AI Studio — 三栏现代工作台" src="docs/screenshots/gui_preview_dark.svg" width="100%">
</picture>

</div>

<br/>

## 🚀 快速上手

1. **下载安装** — 前往 **[GitHub Releases 官方发布页](https://github.com/BerryUIKI/Berry-AI-Studio/releases/latest)** 获取对应系统的安装包。
2. **添加目录** — 将 Berry 指向您的 WebUI、ComfyUI 或 NovelAI 输出目录。选择 *外链模式*（零拷贝就地引用）或 *AIGC 管道模式*（后台防抖自动收割）。
3. **浏览与创作** — 您的海量图库已被瞬间索引。打星评分、打标管理、多图对比、分词提取与无损导出尽在指尖。

---

## 🆕 v0.3.0 重大更新

> **云端同步与导出、游标深度分页与多数据库团队协作工作室** — [阅读完整更新日志 →](CHANGELOG.md)

- ☁️ **S3 / WebDAV 快照备份与恢复** — 自动化热 `VACUUM INTO` 备份，原生支持 AWS S3、Cloudflare R2、MinIO、Backblaze B2 与 WebDAV（群晖 / Nextcloud）。
- 📤 **独立单文件 HTML Showcase 导出** — 零外部依赖单 HTML 文件，内嵌自适应暗色画廊、全屏缩放灯箱、提示词分词检查器与即时关键词过滤器。
- 🔄 **增量式远程差异镜像同步** — 基于 ETag 与 SHA-256 流式校验的双向资产同步，配备令牌桶带宽限速与原子取消机制。
- ⚡ **O(1) 键集游标深度分页** — 彻底告别 SQLite `OFFSET` 慢查询，在 50 万级资产下实现 < 1ms 的超快翻页（性能提升 84 倍）。
- 👥 **多数据库团队协作 (Team Studio)** — 抽象 `StorageEngine` 引擎，支持 SQLite、MySQL 8.0+ 与 PostgreSQL 14+，支持乐观并发版本控制与实时同步日志。
- 📦 **多线程批量格式转码与隐私脱敏** — 基于 Rayon 的 WebP/JPEG/PNG 并行转码，内置 4 级元数据脱敏策略（`KeepAll` → `StripAll`）。

---

## ✨ 核心特性

### 🎨 沉浸式三栏现代工作台

- **原生质感无边框视窗** — 精致自定义标题栏，集成桌面原生菜单栏（`文件`、`编辑`、`视图`、`工具`、`帮助`）、窗口拖拽区与精致控制按键。
- **左侧层级导航栏** — 媒体库快捷入口（全部图片、收藏夹、敏感内容 18+ 分级）、实时扫描状态反馈的文件夹目录树、彩色标签库与智能相册。
- **中央画廊 — 网格 · 瀑布流 · 列表** — 支持毫秒级渲染数万张图像的虚拟滚动体系，Masonry 瀑布流自适应布局、平滑缩略图缩放滑块（130 px – 360 px），以及 ⊞ / ▦ / ☰ 一键切换。
- **自适应稳定列宽** — 窗口拉伸时不拉伸变形卡片，通过动态增减列数保持视觉密度一致。
- **右侧属性检查器** — 大图预览卡片、`0~5` 星级盲打评分、收藏切换、分词高亮芯片与一键复制、识别 LoRA 标签、生成参数展台及原始 JSON 工作流折叠查看器。
- **全屏沉浸灯箱 (Quick Look)** — 按空格（`Space`）或回车（`Enter`）瞬间呼出，支持滚轮平滑缩放、拖拽平移与键盘切图。

### 🔍 全平台 AIGC 元数据无损解析

自动提取并索引正向提示词、反向提示词、模型名称、哈希、采样器、步数、CFG、Seed、分辨率与完整工作流 JSON：

| 生成平台 | 解析来源 |
|:---|:---|
| **AUTOMATIC1111 / SD.Next** | PNG `tEXt`/`iTXt` parameters 块、WebP EXIF |
| **ComfyUI** | 完整 Prompt 与 Workflow 节点图 JSON、LoRA 加载器识别 |
| **NovelAI** | Comment 与 Description 签名格式解码 |
| **Fooocus / Fooocus-MRE** | 专有参数解析与基底模型提取 |
| **InvokeAI & EasyDiffusion** | 内嵌元数据与 JSON Sidecar 伴生文件 |
| **Sidecar 伴生文本** | `.txt` 伴生同名元数据文本 |

**支持的文件容器：** PNG · JPG/JPEG · WebP · MP4 视频

### 🗃️ 多模式导入与智能连拍堆叠

- **外链模式 (External Link)** — 就地索引，零磁盘拷贝占用。
- **托管保险库模式 (Managed Vault)** — 专用应用内部存储空间。
- **AIGC 管道模式 (Pipeline)** — 自动监听 WebUI/ComfyUI 输出目录并防抖收割入库。
- **智能连拍堆叠** — 基于提示词相似度与时间窗口，自动聚合相似迭代版本。
- **互动叠放卡片** — 扑克牌叠放视觉卡片，带数量徽章、行内展开/折叠、设定封面图（`Alt+S`）、双图并排对比（`C`）以及安全事务性合并展开。

### 🧠 本地 AI 智能引擎

- **本地 CLIP / SigLIP 语义搜索** — 纯本地 ONNX 模型驱动，无需联网即可用自然语言搜图。
- **WD14 / Danbooru 动漫自动打标** — 自动反推动漫及写实风格标签，支持置信度阈值调谐。
- **以图搜图视觉相似度检索** — 快速寻找风格、构图相似的画面。
- **LoRA 触发词库与 Civitai 同步** — 一键扫描 LoRA 模型目录、匹配 Civitai 元数据，触发词一键复制注入。
- **提示词词频与表现洞察** — 正负提示词关键词频次统计，关联平均评分表现。
- **模型库管理与哈希反查** — Civitai SHA256 缓存自动同步、模型哈希反查与一键模型筛选。

### ☁️ 云端同步与导出工具

- **S3 / WebDAV 快照备份** — 热 `VACUUM INTO` 备份到 AWS S3、Cloudflare R2、MinIO、Backblaze B2 与各类 WebDAV 服务器，具备架构校验与安全回滚保障。
- **增量式远程差异同步** — ETag + 流式 SHA-256 变动侦测，双向镜像同步，令牌桶带宽限制与原子任务取消。
- **多线程批量格式转码** — 快速将图片并行转码为 WebP/JPEG/PNG，提供 4 级隐私脱敏与自定义文件命名模板。
- **单文件 HTML Showcase 生成器** — 导出独立运行的 `index.html`，无需任何 Web 服务即可向客户展示带暗色主题、灯箱与提示词面板的作品集。

### 👥 多数据库团队协作工作室

- **存储引擎抽象层** — `StorageEngine` 抽象特性，无缝切换 SQLite（默认）、MySQL 8.0+ 与 PostgreSQL 14+。
- **跨平台存储根路径映射** — 针对 Windows、macOS 与 Linux 异构网络挂载点自动规范化相对路径。
- **乐观并发版本控制** — 行级版本审计，智能处理多人编辑时的冲突合并。
- **实时协作变更同步** — 零 DevOps 维护负担的变更日志轮询引擎，自适应节奏同步各客户端状态。

### 🌐 国际化与自动更新

- **7 种原生语言** — 简体中文 · 繁體中文 · English · 日本語 · Deutsch · Français · Español
- **操作系统语言自适应** — 默认跟随系统语言（`Auto`）。
- **内置检查更新** — 在 **帮助 > 检查更新…** 中即刻获取最新版本说明与官方下载链接。

---

## ⚡ 卓越性能

Berry AI Studio 为十万级海量图库专门设计。应用启动时直接渲染 SQLite 本地索引库，绝不让文件目录遍历阻塞首屏渲染：

| 评估指标 | 1,000 张图像 | 10,000 张图像 | 50,000 张图像 |
|:---|:---:|:---:|:---:|
| 首屏可用画廊渲染时间（热数据库） | **4.48 ms** | **23.65 ms** | **143.55 ms** |
| 极速拖动滚动条时主线程长任务 (> 50 ms) | **0 次** | **0 次** | **0 次** |
| 缩略图持久化清单加载耗时 | — | — | **3.26 s** (15,300 文件/秒) |

**底层架构保障：**

- **O(1) 键集游标分页** — 50 万张图片深度翻页仅需 < 1ms（较传统 `OFFSET` 提速 84 倍）。
- **帧合并渲染调度** — 每次 `requestAnimationFrame` 最多只触发一次响应式重绘。
- **按列二分查找瀑布流** — 毫秒级计算视口内可见项，避免全量数据集遍历。
- **按需驱动缩略图管线** — 仅优先解码视口可见卡片，滚动静止后才启动预加载。
- **目录指纹拦截机制** — 父目录 `mtime` 比对机制在二次扫描时本地提速 2.5 倍，网络存储提速 5.6 倍。

📖 详见 [性能架构设计与基准测试报告](docs/PERFORMANCE.md)。

---

## 🏗️ 架构分层

```
┌─────────────────────────────────────────────────────────────────┐
│                    Berry AI Studio (桌面客户端)                 │
├─────────────────────────────────────────────────────────────────┤
│  Vue 3 + TypeScript          │  Tauri 2 IPC 命令通道            │
│  ─ 虚拟网格 / 瀑布流         │  ─ 轻量适配器、入参校验、        │
│  ─ 属性检查器 / 全屏灯箱     │    锁快速释放                    │
│  ─ 37 个组件、11 个工具集    │  ─ 50+ 个核心接口                │
├──────────────────────────────┼──────────────────────────────────┤
│              模块化 Rust 工作区 (Cargo Workspace)               │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────┐           │
│  │ berry-domain │ │berry-metadata│ │  berry-scan   │           │
│  │ 核心领域模型 │ │ PNG/EXIF/    │ │ 扫描索引、    │           │
│  │ 零 I/O 契约  │ │ ComfyUI/     │ │ 缩略图引擎、  │           │
│  │              │ │ NovelAI/...  │ │ 导出与展示页  │           │
│  └──────────────┘ └──────────────┘ └───────────────┘           │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────┐           │
│  │berry-storage │ │ berry-tagger │ │  berry-clip   │           │
│  │ SQLite/MySQL │ │ WD14 ONNX    │ │ CLIP/SigLIP   │           │
│  │ PostgreSQL   │ │ Danbooru     │ │ 文本与图像    │           │
│  │ 增量迁移系统 │ │ 反推打标     │ │ 多模态嵌入    │           │
│  └──────────────┘ └──────────────┘ └───────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

**核心技术栈：** Tauri 2 · Rust (2021 edition) · Vue 3.5 · TypeScript 5.6 · Vite 6 · SQLite (WAL) · Rayon · ONNX Runtime

---

## ⌨️ 常用快捷键

| 快捷键 | 功能说明 | 快捷键 | 功能说明 |
|:---|:---|:---|:---|
| `Space` / `Enter` | 打开 / 关闭全屏灯箱 | `0` – `5` | 设置星级评分 (0 为清除) |
| `F` | 切换收藏状态 | `B` | 显示 / 隐藏左侧导航栏 |
| `I` | 显示 / 隐藏属性检查器 | `/` 或 `Ctrl+F` | 聚焦顶部搜索框 |
| `Ctrl+A` | 全选当前全部图像 | `Esc` | 取消选择 / 关闭弹窗或灯箱 |
| `Ctrl+O` | 打开文件夹创建向导 | `Ctrl+,` | 首选项与设置 |
| `Ctrl+G` | 手动叠放 / 合并成堆 | `Ctrl+Shift+G` | 解除叠放 / 展开全部图像 |
| `Alt+S` | 设定当前图像为封面 | `C` | 双图并排对比模式 |
| `Delete` | 将选中图片移入回收站 | `?` | 呼出快捷键指南 |

---

## 📥 下载安装

前往 **[GitHub Releases 官方发布页](https://github.com/BerryUIKI/Berry-AI-Studio/releases/latest)** 获取最新预编译安装包：

| 操作系统平台 | 处理器架构 | 格式类型 | 安装包文件名 |
|:---|:---|:---|:---|
| **Windows** | x86_64 (64位) | NSIS 安装包 | `Berry-AI-Studio_Windows_x64.exe` |
| **Windows** | x86_64 (64位) | 免安装便携版 | `Berry-AI-Studio_Windows_x64.zip` |
| **macOS** | Apple Silicon (ARM64) | DMG 磁盘镜像 | `Berry-AI-Studio_macOS_aarch64.dmg` |
| **macOS** | Intel (x86_64) | DMG 磁盘镜像 | `Berry-AI-Studio_macOS_x64.dmg` |
| **Linux** | x86_64 (64位) | AppImage | `Berry-AI-Studio_Linux_x64.AppImage` |
| **Linux** | x86_64 (64位) | Debian 软件包 | `Berry-AI-Studio_Linux_x64.deb` |

---

## 🛠️ 源码构建指南

### 环境准备

- **Node.js** v18+ 与 **pnpm** — `npm install -g pnpm`
- **Rust** 1.75+ — 推荐通过 [rustup.rs](https://rustup.rs/) 安装
- **编译工具链** — Windows 下为 MSVC Build Tools，macOS 下为 Xcode CLI，Linux 下为 `libwebkit2gtk-4.1`

### 构建步骤

```bash
# 1. 克隆代码仓库
git clone https://github.com/BerryUIKI/Berry-AI-Studio.git
cd Berry-AI-Studio

# 2. 安装前端依赖
pnpm install

# 3. 运行本地开发模式（支持热重载）
pnpm run tauri dev

# 4. 打包正式版安装程序
pnpm run tauri build
```

编译输出目录：`src-tauri/target/release/bundle/`

---

## 🗺️ 开发路线图

| 规划里程碑 | 目标版本 |
|:---|:---|
| 🎬 视频与动态 AIGC 支持 (ComfyUI AnimateDiff, Wan2.1, HunyuanVideo 逐帧预览与灯箱播放) | v0.4.0 |
| 🧬 新一代模型架构适配 (Flux.1, SD3.5 Guidance 与双文本编码器提取, Civitai API) | v0.5.0 |
| 🔗 双向 ComfyUI 深度集成 (WebSocket 遥测监控, 零延迟入库, 提示词 Diff 对比) | v0.6.0 |
| 📱 局域网 Web 伴侣 ("Berry Remote" 平板与手机端无线浏览) | v0.7.0 |
| 🎨 主导色彩提取与多维 SQL 分析仪表盘 | v0.8.0 |

📖 查阅完整开发计划：[docs/ROADMAP.md](docs/ROADMAP.md)。

---

## 🤝 参与贡献

我们热烈欢迎各类 Issue 反馈、功能建议、新元数据解析器适配以及多语言翻译！

1. Fork 本仓库并从 `dev` 分支创建您的特性分支（请勿直接向 `main` 提交）。
2. 请阅读 [贡献指南](CONTRIBUTING.md) 与 [代码规范指引](AGENTS.md)。
3. 在提交 Pull Request 前请确保以下验证指令全部通过：

```bash
pnpm run build          # 前端 TypeScript 校验与构建
pnpm run test:stack     # 堆叠与画廊逻辑测试
cargo clippy --workspace -- -D warnings
cargo test --workspace  # Rust 后端单元测试
```

---

## 📄 开源许可证

本项目基于 **[AGPL-3.0 开源协议](LICENSE)** 发布。

Copyright © 2026 [BerryUIKI](https://github.com/BerryUIKI).

---

## 🙏 特别致谢

Berry AI Studio 的诞生离不开以下优秀的开源项目：

- [Tauri](https://tauri.app) — 超轻量跨平台桌面应用开发框架
- [Vue.js](https://vuejs.org) — 渐进式前端 JavaScript 框架
- [Rust](https://www.rust-lang.org) & [Rayon](https://github.com/rayon-rs/rayon) — 兼具高性能与并发安全的原生语言
- [rusqlite](https://github.com/rusqlite/rusqlite) — 优雅好用的 SQLite 绑定
- [ONNX Runtime](https://onnxruntime.ai) — 跨平台机器学习高性能推理引擎
- [CLIP](https://github.com/openai/CLIP) & [SigLIP](https://arxiv.org/abs/2303.15343) — 多模态视觉-语言大模型
- [WD14 Tagger](https://huggingface.co/SmilingWolf) — 动漫领域高质量反推打标模型

---

<div align="center">

**[⬆ 返回顶部](#-berry-ai-studio)**

如果您觉得 Berry AI Studio 对您的创作有所帮助，请为我们点亮一颗 ⭐，这能让更多创作者发现这个项目！

<a href="https://star-history.com/#BerryUIKI/Berry-AI-Studio&Date">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=BerryUIKI/Berry-AI-Studio&type=Date&theme=dark" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=BerryUIKI/Berry-AI-Studio&type=Date" />
    <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=BerryUIKI/Berry-AI-Studio&type=Date" width="600" />
  </picture>
</a>

<sub>Made with ❤️ by the Berry AI Studio community</sub>

</div>
