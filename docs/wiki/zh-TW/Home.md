# Omera — 官方維基與使用者指南

歡迎查閱 **Omera** (`v0.3.0`) 的權威使用者文件與知識庫。

Omera 是一款開源、本機優先的資產管理器與提示詞工作台，專為生成式 AI 創作者、提示詞工程師與視覺設計工作室量身打造。基於 **Tauri v2**、**Rust** 與 **Vue 3** 構建，能輕鬆管理從數百張藝術作品到 500,000+ 檔案的超大圖庫，具備次毫秒級查詢延遲、零雲端依賴以及完整的生成元數據（元資料）解析能力。

---

## 🧭 導覽與目錄索引

### [第 1 章：新手入門與基礎操作](01-getting-started/installation.md)
- **[安裝與系統需求](01-getting-started/installation.md)**：硬體前置需求、Windows 安裝版/免安裝可攜版選項、macOS Universal 與 Apple Silicon 組建、Linux AppImage/deb，以及首次啟動歡迎設定精靈。
- **[工作區與介面結構](01-getting-started/workspace-layout.md)**：無框視窗、應用程式選單列、三欄式版面配置（左側側邊欄、中央畫廊、右側檢查器）、狀態列以及浮動批次操作列的深入解析。
- **[鍵盤快速鍵指南](01-getting-started/keyboard-shortcuts.md)**：全域快捷鍵、選取範圍錨點、盲打星級評分、快速查看預覽以及瀏覽導航快捷鍵。

### [第 2 章：媒體庫管理與瀏覽](02-library-management/folder-modes-and-import.md)
- **[匯入媒體與資料夾模式](02-library-management/folder-modes-and-import.md)**：對比模式 A（外部連結與監聽）、模式 B（代管專案庫）與模式 C（AIGC 自動管線，具備防抖監聽與延遲安全清理）。支援的影像（PNG、WebP、JPEG）與影片（MP4、WebM）格式。
- **[畫廊檢視與顯示選項](02-library-management/gallery-views.md)**：掌握等寬網格（具備 130px–360px 縮放）、瀑布流（原始不裁切寬高比）、清單檢視以及視覺相似度檢索檢視。卡片角標與 NSFW 敏感內容遮罩模糊。
- **[組織整理、評分與標籤](02-library-management/organization-and-tags.md)**：10 星評分刻度、我的最愛收藏、自訂相簿、8 色標籤分類體系、批次拖放整理與批次操作工具列。
- **[影片與動態媒體支援](02-library-management/video-support.md)**：AnimateDiff、Wan2.1、HunyuanVideo 與 SVD 播放；逐格前進/後退、循環播放與播放速度 HUD，以及內嵌影片工作流程檢視。

### [第 3 章：探索、搜尋與數據分析](03-discovery-and-analytics/search-and-filtering.md)
- **[搜尋語法與視覺化篩選](03-discovery-and-analytics/search-and-filtering.md)**：進階鍵值查詢語言（`prompt:`、`neg:`、`model:`、`cfg:>=7`、`steps:20..40`）、數值範圍查詢與滑出式抽屜篩選器。
- **[AIGC 元數據與提示詞檢查](03-discovery-and-analytics/metadata-and-prompts.md)**：針對 AUTOMATIC1111、ComfyUI、NovelAI、Fooocus、InvokeAI 的無損解析；分詞互動提示詞標籤與原始執行圖譜節點檢視。
- **[提示詞分析與洞察](03-discovery-and-analytics/prompt-insights.md)**：全圖庫詞元（Token）頻率分佈、高頻正向/負向提示詞排行，以及與使用者評分的關聯分析。

### [第 4 章：智慧策展與 AI 引擎](04-intelligent-curation/stacks-and-bursts.md)
- **[圖片堆疊、連拍與比對](04-intelligent-curation/stacks-and-bursts.md)**：自動連拍分群（Jaccard 提示詞相似度 + 時間窗口）、撲克牌折疊卡片、封面 Hero、安全堆疊解散、清理低分草稿工具以及雙圖並排深度比對（`C`）檢視。
- **[AI 語意搜尋與自動標籤](04-intelligent-curation/ai-semantic-and-tagger.md)**：本機 ONNX CLIP/SigLIP 文字搜圖自然語言查詢、圖搜圖視覺最近鄰檢索，以及 WD14 Danbooru 動漫自動標籤器。
- **[Checkpoint 模型庫與 LoRA 詞庫](04-intelligent-curation/models-and-loras.md)**：自動 Checkpoint 編目、A1111 `cache.json` 雜湊解析、Civitai 反向查詢、LoRA 觸發詞管理器與一鍵提示詞注入。
- **[生成工具互通整合](04-intelligent-curation/generation-interop.md)**：直接與 ComfyUI（`/prompt`）與 AUTOMATIC1111（`/sdapi/v1/txt2img`）進行 API 通訊，並具備即時連線健康狀態監測。

### [第 5 章：匯出、雲端備份與團隊協同](05-export-and-collaboration/export-and-web-showcase.md)
- **[批次匯出、轉碼與網頁畫冊](05-export-and-collaboration/export-and-web-showcase.md)**：多執行緒 Rayon 轉碼、4 級隱私元數據清洗脫敏、動態檔名範本、ZIP 壓縮封存，以及生成完全離線獨立互動單檔案 HTML 網頁畫冊。
- **[雲端快照備份與媒體映像](05-export-and-collaboration/cloud-backup-and-sync.md)**：熱 SQLite `VACUUM INTO` 快照備份至 AWS S3、Cloudflare R2、MinIO、WebDAV 或本機 NAS；基於 ETag/SHA-256 差異比對與頻寬限速的增量差異同步。
- **[多資料庫團隊工作室](05-export-and-collaboration/team-collaboration.md)**：超越單機 SQLite，邁向共享 MySQL 8.0+ 或 PostgreSQL 14+ 伺服器；跨平台儲存根目錄對應（規格化 Windows 磁碟機代號至 macOS/Linux 掛載路徑）、樂觀並行控制（OCC）與用戶端 NVMe 縮圖快取。

### [第 6 章：系統參考與維護](06-reference-and-maintenance/settings-reference.md)
- **[偏好設定完整指南](06-reference-and-maintenance/settings-reference.md)**：橫跨全部 8 個設定分頁的完整參數參考指南。
- **[資料庫與快取維護](06-reference-and-maintenance/database-maintenance.md)**：SQLite WAL 結構壓縮（`VACUUM`）、資料庫快照備份與復原、縮圖快取配額（LRU 淘汰機制）與背景工作佇列診斷。
- **[更新與生命週期](06-reference-and-maintenance/updating.md)**：應用程式內建自動更新、跨版本資料完全隔離儲存保證，以及手動發布版本覆蓋升級。
- **[隱私與安全架構](06-reference-and-maintenance/privacy-and-security.md)**：100% 本機離線優先架構、零遙測數據收集、本機 AI 推論安全隔離與 AGPL-3.0 授權條款。
- **[疑難排解與常見問題](06-reference-and-maintenance/troubleshooting-and-faq.md)**：常見疑難解決方案、效能最佳化建議與實用 FAQ。
- **[產品專有名詞詞彙表](06-reference-and-maintenance/glossary.md)**：核心領域專有名詞權威定義（Hero 封面圖、Ingestion Pipeline 收穫管線、Jaccard similarity 傑卡德相似度、Keyset cursor 鍵集游標、OCC 樂觀並行控制、Poker-deck card 撲克牌折疊卡等）。

---

## ⚡ 快速入門常用快速鍵

| 操作說明 | Windows / Linux | macOS |
| :--- | :--- | :--- |
| **開啟燈箱預覽（快速查看）** | `Space` / `Enter` | `Space` / `Return` |
| **雙圖並排深度比對** | `C` | `C` |
| **手動分組建立堆疊** | `Ctrl + G` | `Cmd + G` |
| **解散堆疊** | `Ctrl + Shift + G` | `Cmd + Shift + G` |
| **設為堆疊主封面圖 (Hero)** | `Alt + S` | `Option + S` |
| **為選取項目評分 1–5 星** | `1` – `5` (`0` 為清除) | `1` – `5` (`0` 為清除) |
| **切換我的最愛收藏** | `F` | `F` |
| **聚焦頂部搜尋框** | `/` 或 `Ctrl + F` | `/` 或 `Cmd + F` |
| **切換右側檢查器** | `I` | `I` |
| **切換左側側邊欄** | `B` | `B` |
| **開啟偏好設定** | `Ctrl + ,` | `Cmd + ,` |
