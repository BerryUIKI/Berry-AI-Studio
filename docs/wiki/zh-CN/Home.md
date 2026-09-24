# Omera — 官方知识库与用户指南

欢迎阅读 **Omera**（`v0.3.0`）权威用户文档与知识库。

Omera 是一款开源、本地优先（Local-First）的媒体资产管理与提示词工作台，专为生成式 AI 创作者、提示词工程师和视觉设计工作室打造。应用基于 **Tauri v2**、**Rust** 和 **Vue 3** 构建，无论图库规模是数百张作品还是 500,000+ 文件，均可提供亚毫秒级查询延迟、零云端依赖以及全面的生成元数据无损提取。

---

## 🧭 目录导航与索引

### [第 1 章：新手入门与基础](01-getting-started/installation.md)
- **[系统配置要求与安装向导](01-getting-started/installation.md)**：硬件环境要求、Windows 安装版/便携版选项、macOS 通用二进制与 Apple Silicon 构建、Linux AppImage/deb，以及首次启动欢迎配置向导。
- **[工作区布局与交互架构](01-getting-started/workspace-layout.md)**：无边框窗口、原生应用菜单栏、标准三栏式布局（左侧导航栏、中央画廊网格、右侧属性检查器）、底部状态栏与悬浮批量操作栏的深度解析。
- **[键盘快捷键速查表](01-getting-started/keyboard-shortcuts.md)**：全局快捷键、选区锚定、盲打星级评分、快速查看与导航热键一览。

### [第 2 章：资产管理与画廊浏览](02-library-management/folder-modes-and-import.md)
- **[媒体导入与文件夹三大模式](02-library-management/folder-modes-and-import.md)**：外链模式（模式 A：外部只读引用）、托管项目库模式（模式 B：集中管理归档）与 AI 管线模式（模式 C：防抖实时监听与延迟安全清理）。支持的图像（PNG、WebP、JPEG）及视频（MP4、WebM）格式。
- **[画廊视图与显示设置](02-library-management/gallery-views.md)**：精通等宽网格（支持 130px–360px 缩放）、瀑布流视图（保留原始宽高比）、详细列表视图与视觉相似度检索视图。卡片状态徽章与敏感内容（NSFW）隐私模糊遮罩。
- **[资产整理、评分与标签](02-library-management/organization-and-tags.md)**：0–5 级半星级评分体系、一键收藏、自定义相册、8 色标签分类法、批量拖放与批量操作工具栏。
- **[动态媒体与视频支持](02-library-management/video-support.md)**：支持 AnimateDiff、Wan2.1、HunyuanVideo 和 SVD 视频回放；逐帧步进、循环/倍速控制面板与内嵌视频工作流无损解析。

### [第 3 章：检索、元数据与统计洞察](03-discovery-and-analytics/search-and-filtering.md)
- **[搜索语法与可视化筛选抽屉](03-discovery-and-analytics/search-and-filtering.md)**：高级键值对查询语法（`prompt:`、`neg:`、`model:`、`cfg:>=7`、`steps:20..40`）、数值区间搜索与侧滑式抽屉筛选器。
- **[AIGC 生成元数据与提示词解析](03-discovery-and-analytics/metadata-and-prompts.md)**：全面支持 AUTOMATIC1111、ComfyUI、NovelAI、Fooocus、InvokeAI 等无损解析；词元化交互式提示词标签与原始工作流执行图解析。
- **[提示词统计与词频洞察](03-discovery-and-analytics/prompt-insights.md)**：全库词元频率分布分析、正向/负向高频词排行榜，以及词元与用户星级评分的相关性分析。

### [第 4 章：AI 智能引擎与辅助创作](04-intelligent-curation/stacks-and-bursts.md)
- **[连拍智能堆叠、扑克卡片与双图对比](04-intelligent-curation/stacks-and-bursts.md)**：自动连拍聚合（Jaccard 提示词相似度 + 时间窗口）、扑克牌折叠卡片、主封面图（Hero）设定、安全解散堆栈、清理低分草稿工具，以及双图并排（`C`）深度比对模式。
- **[AI 语义搜索与自动打标](04-intelligent-curation/ai-semantic-and-tagger.md)**：本地 ONNX CLIP/SigLIP 文搜图自然语言检索、以图搜图视觉最近邻匹配，以及基于 WD14 的 Danbooru 二次元自动打标引擎。
- **[Checkpoint 模型库与 LoRA 触发词管理](04-intelligent-curation/models-and-loras.md)**：自动识别模型库、A1111 `cache.json` 哈希反查、Civitai 线上匹配、LoRA 触发词库与一键插入提示词。
- **[生图环境互通与双向工作流](04-intelligent-curation/generation-interop.md)**：直接对接 ComfyUI（`/prompt`）与 AUTOMATIC1111（`/sdapi/v1/txt2img`）本地 API，具备实时在线连通性检测。

### [第 5 章：导出、云端备份与团队协同](05-export-and-collaboration/export-and-web-showcase.md)
- **[批量导出、转码与静态网页画册](05-export-and-collaboration/export-and-web-showcase.md)**：多线程 Rayon 并发转码、4 级隐私元数据清洗脱敏、动态命名模板、ZIP 压缩打包，以及生成完全独立的单文件自适应交互 HTML 网页画册。
- **[云端快照备份与媒体差异镜像](05-export-and-collaboration/cloud-backup-and-sync.md)**：基于 SQLite 热快照（`VACUUM INTO`）备份至 AWS S3、Cloudflare R2、MinIO、WebDAV 或本地 NAS；基于 ETag/SHA-256 校验的增量差异镜像与带宽限速。
- **[多数据库团队协同工作站](05-export-and-collaboration/team-collaboration.md)**：无缝扩展至共享 MySQL 8.0+ 或 PostgreSQL 14+ 数据库；跨平台存储根映射（自动规范化 Windows 盘符与 macOS/Linux 挂载路径）、乐观并发锁（OCC）与客户端 NVMe 高速缩略图缓存。

### [第 6 章：系统参考与维护手册](06-reference-and-maintenance/settings-reference.md)
- **[首选项与设置完整参考](06-reference-and-maintenance/settings-reference.md)**：涵盖全部 8 大设置标签页的参数配置详解。
- **[数据库维护与缓存管理](06-reference-and-maintenance/database-maintenance.md)**：SQLite WAL 空间压缩（`VACUUM`）、数据库热备份与恢复、缩略图磁盘预算控制（LRU 淘汰算法）以及后台任务队列诊断。
- **[版本升级与数据保护机制](06-reference-and-maintenance/updating.md)**：应用内静默就地更新、跨版本数据零丢失架构保障与手动覆盖安装说明。
- **[隐私架构与安全规范](06-reference-and-maintenance/privacy-and-security.md)**：100% 离线优先架构、零用户遥测收集、本地 AI 推理沙箱隔离与 AGPL-3.0 开源协议。
- **[常见故障排除与 FAQ](06-reference-and-maintenance/troubleshooting-and-faq.md)**：高频问题诊断指南、性能调优技巧与常见问答。
- **[核心专有名词术语表](06-reference-and-maintenance/glossary.md)**：产品专属术语权威定义（Hero 封面、入库管线、Jaccard 相似度、游标分页、OCC 乐观并发控制、扑克牌卡片等）。

---

## ⚡ 常用快捷键速查

| 操作功能 | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **快速查看 / 全屏灯箱预览** | `Space` / `Enter` | `Space` / `Return` |
| **双图并排深度比对** | `C` | `C` |
| **新建图片堆栈** | `Ctrl + G` | `Cmd + G` |
| **解散图片堆栈** | `Ctrl + Shift + G` | `Cmd + Shift + G` |
| **设为堆栈主封面图 (Hero)** | `Alt + S` | `Option + S` |
| **评分 1–5 星** | `1` – `5`（按 `0` 清除） | `1` – `5`（按 `0` 清除） |
| **切换收藏状态** | `F` | `F` |
| **激活搜索输入框** | `/` 或 `Ctrl + F` | `/` 或 `Cmd + F` |
| **展开/收起属性检查器** | `I` | `I` |
| **展开/收起左侧导航栏** | `B` | `B` |
| **打开首选项设置** | `Ctrl + ,` | `Cmd + ,` |
