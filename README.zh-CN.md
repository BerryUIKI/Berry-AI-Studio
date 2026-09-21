# 🍇 Berry AI Studio

<div align="center">

**[English](README.md)** | **[简体中文](README.zh-CN.md)** | **[繁體中文](README.zh-TW.md)** | **[日本語](README.ja.md)**

[![Website](https://img.shields.io/badge/网站-GitHub%20Pages-12b5cb.svg)](https://berryuiki.github.io/Berry-AI-Studio/)
[![Release](https://img.shields.io/badge/版本-v0.3.0-blue.svg)](https://github.com/BerryUIKI/Berry-AI-Studio/releases/tag/v0.3.0)
[![License](https://img.shields.io/badge/协议-AGPL--3.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-24c8db)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.7+-orange)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-42b883)](https://vuejs.org)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/BerryUIKI/Berry-AI-Studio/releases)

*专为 AI 图像创作者与 Prompt 工程师打造的高性能、本地化 AIGC 元数据索引与资产管理工作台。*

<br/>

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/screenshots/gui_preview_dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/screenshots/gui_preview_light.svg">
  <img alt="Berry AI Studio Preview" src="docs/screenshots/gui_preview_dark.svg" width="100%">
</picture>

</div>

---

## 🌟 项目概述

**Berry AI Studio** 是一款专为数字艺术家、AI 画师和提示词工程师设计的桌面级图像资产管理系统。软件能够毫秒级扫描并深度解析主流平台生成的图片元数据（提示词、模型、采样器、步数、CFG、Seed 及工作流 JSON），并构建本地高速 SQLite 索引，提供 **沉浸式三栏现代工作台**、流畅的虚拟网格瀑布流、分词高亮属性面板、沉浸式全屏灯箱以及丰富的批量分类能力。

> 🚀 **v0.2.0 版本发布**：全新支持本地 CLIP 语义搜索、AI 智能打标、LoRA 触发词库、多模式收割管道、智能连拍堆叠与虚拟滚动瀑布流。

---

## ✨ 核心特性

### 🎨 沉浸式三栏创作工作台
- **原生质感无边框窗口**：自定义标题栏集成桌面菜单栏（`文件`、`编辑`、`视图`、`工具`、`帮助`）、窗口拖拽区与精致控制按钮。
- **左侧分类导航栏**：媒体库快捷入口（全部图片、收藏夹、敏感内容 18+）、文件夹层级目录树（支持即时扫描状态反馈）、彩色标签库与智能相册。
- **中央画廊、虚拟网格与瀑布流**：支持顺滑渲染万级图像的虚拟滚动网格、Masonry 瀑布流布局、缩略图自由缩放滑块（130px–360px），以及网格（⊞）/ 瀑布流（▦）/ 列表（☰）一键切换。
- **右侧属性检查器**：大图预览卡片、`0~5` 星级盲打评分、收藏切换、正反向提示词分词高亮与一键复制、识别 LoRA 标签、生成参数展台及原始 JSON 工作流折叠查看器。
- **全屏沉浸灯箱 (Quick Look)**：按空格键（`Space`）或回车即刻呼出，支持滚轮平滑缩放、拖拽平移、键盘方向键快速切图。

### 🔍 全平台 AIGC 元数据无损解析
自动提取并索引正向提示词、负向提示词、模型名称、模型哈希、采样器、步数、CFG、Seed、分辨率与完整工作流：
- **WebUI (AUTOMATIC1111 / SD.Next)**：PNG `parameters` 块、WebP EXIF。
- **ComfyUI**：完整 Prompt 与 Workflow 工作流 JSON 语法树解析与 LoRA 加载器识别。
- **NovelAI**：Comment 与 Description 签名格式解析。
- **Fooocus / Fooocus-MRE**：专有参数与基底模型提取。
- **InvokeAI & EasyDiffusion**：内嵌元数据与 JSON Sidecar 伴生文件。
- **支持文件容器**：PNG、JPG/JPEG、WebP、MP4 视频及 `.txt` 伴生文本。

### 🗃️ 多模式导入与智能连拍堆叠
- **多模式文件夹**：外链模式（就地引用零拷贝）、托管保险库模式（应用管理存储）与 AIGC 管道模式（自动监控 WebUI/ComfyUI 输出目录并防抖收割）。
- **智能连拍堆叠**：根据提示词相似度与生成时间窗口自动聚合相近连拍版本。
- **互动叠放卡片**：扑克牌视觉叠放卡片，支持数量徽章、行内点击展开/折叠、设定封面图（`Alt+S`）、双图并排对比（`C`）与安全确认合并展开。

### 🧠 模型、提示词与 AI 智能
- **本地 CLIP 语义搜索**：基于本地 ONNX CLIP/SigLIP 模型，直接在搜索栏输入自然语言查找图像。
- **AI 智能打标与以图搜图**：内置 WD14 / Danbooru 动漫反推打标，支持多维度图像视觉相似度检索。
- **LoRA 触发词库**：一键扫描 LoRA 模型目录、匹配 Civitai 元数据，并支持将触发词一键复制注入提示词。
- **提示词词频统计**：统计正反向提示词中高频词汇并关联平均评分表现。
- **模型库管理与哈希反查**：Civitai SHA256 缓存自动同步、模型哈希反查与一键反向筛选。
- **数据库维护工具**：内置 SQLite VACUUM 空间压缩、数据库完整备份导出与一键还原。

### 🌐 国际化与检查更新
- **7 国语言原生支持**：简体中文、繁體中文、English、日本語、Deutsch、Français、Español。
- **跟随系统语言（Auto）**：默认自动匹配当前操作系统语言。
- **GitHub Releases 在线检查更新**：在 **帮助 > 检查更新...** 中一键获取最新版本、更新日志与官方安装包。

---

## ⌨️ 常用快捷键

| 快捷键 | 功能说明 | 快捷键 | 功能说明 |
| :--- | :--- | :--- | :--- |
| `Space` / `Enter` | 打开 / 关闭全屏灯箱预览 | `0` ~ `5` | 设置星级评分 (0 为清除) |
| `F` | 切换收藏状态 | `B` | 显示 / 隐藏左侧导航栏 |
| `I` | 显示 / 隐藏右侧属性检查器 | `/` 或 `Ctrl+F` | 聚焦顶部搜索框 |
| `Ctrl+A` | 全选当前视图全部图像 | `Esc` | 取消选择 / 关闭弹窗或灯箱 |
| `Ctrl+O` | 文件夹模式引导向导 | `Ctrl+,` | 打开首选项与设置 |
| `Ctrl+G` | 手动叠放 / 合并成堆 | `Ctrl+Shift+G` | 解除叠放 / 展开全部图像 |
| `Alt+S` | 设定当前图像为堆叠封面 | `C` | 进入多图并排对比模式 |
| `Delete` | 将选中图片移入系统回收站 | `?` | 呼出快捷键指南 |

---

## 📦 发行版安装包命名规范

GitHub Releases 官方发布的预编译二进制文件遵循标准命名格式：

$$\text{<应用名>}\_\text{<操作系统>}\_\text{<系统架构>}.\text{<文件后缀>}$$

| 操作系统平台 | 处理器架构 | 格式类型 | 安装包文件名 |
| :--- | :--- | :--- | :--- |
| **Windows** | x86_64 (64位) | NSIS 安装包 | `Berry-AI-Studio_Windows_x64.exe` |
| **Windows** | x86_64 (64位) | 免安装便携版 | `Berry-AI-Studio_Windows_x64.zip` |
| **macOS** | Apple Silicon (ARM64) | DMG 磁盘镜像 | `Berry-AI-Studio_macOS_aarch64.dmg` |
| **macOS** | Intel (x86_64) | DMG 磁盘镜像 | `Berry-AI-Studio_macOS_x64.dmg` |
| **Linux** | x86_64 (64位) | AppImage | `Berry-AI-Studio_Linux_x64.AppImage` |
| **Linux** | x86_64 (64位) | DEB 软件包 | `Berry-AI-Studio_Linux_x64.deb` |

---

## 🛠️ 源码构建指南

### 环境准备
1. **Node.js** (v18+) 与 **pnpm** (`npm install -g pnpm`)
2. **Rust** (1.75+): 推荐通过 [rustup.rs](https://rustup.rs/) 安装
3. **C++ 编译环境**: Windows 下为 MSVC Build Tools，macOS 下为 Xcode CLI，Linux 下为 `libwebkit2gtk-4.1`。

### 构建步骤
```bash
# 1. 克隆代码仓库
git clone https://github.com/BerryUIKI/Berry-AI-Studio.git
cd Berry-AI-Studio

# 2. 安装前端依赖
pnpm install

# 3. 运行本地开发热重载模式
pnpm run tauri dev

# 4. 打包正式版安装包
pnpm run tauri build
```

打包产物位于 `src-tauri/target/release/bundle/` 目录中。

---

## 📄 开源许可证

本项目基于 **AGPL-3.0 开源协议** 发布。详见 [LICENSE](LICENSE) 文件。
