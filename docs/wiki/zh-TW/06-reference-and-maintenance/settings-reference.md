# 偏好設定完整指南

Berry AI Studio 的偏好設定視窗 (`SettingsModal.vue`) 可透過頂部選單 `檔案 > 偏好設定 / 設定...` 或快捷鍵 `Ctrl + ,` / `Cmd + ,` 開啟。所有設定項目皆即時持久化保存在本機資料目錄下的 `config.json` 設定檔中。

---

## 分頁 1：一般偏好 (General Preferences)

| 設定欄位 | `config.json` 鍵值 | 預設值 | 說明 |
| :--- | :--- | :--- | :--- |
| **介面顯示語言** | `locale` | `"auto"` | 可選：`auto`（跟隨系統語系）、`en`（英文）、`zh-CN`（簡體中文）、`zh-TW`（繁體中文）、`ja`（日文）、`de`（德文）、`fr`（法文）、`es`（西班牙文）。 |
| **預設畫廊檢視** | `default_view` | `"grid"` | 啟動軟體時預設使用的圖片展示方式：`"grid"`（等寬網格）、`"masonry"`（瀑布流原始比例）或 `"table"`（詳細清單）。 |
| **啟動時自動掃描** | `auto_scan` | `true` | 啟動時自動檢查已登記資料夾中的新增或變動檔案。 |
| **啟動掃描冷卻時間** | `startup_scan_interval_minutes`| `360` | 在此時間間隔（分鐘）內略過重複的完整實體資料夾掃描（可選：`30`、`60`、`360`、`1440`）。避免頻繁重啟應用程式時造成硬碟過度讀寫。 |
| **啟動時自動檢查更新** | `auto_check_update` | `true` | 啟動時於背景連線至 GitHub Releases 查詢是否有更新版本。 |

---

## 分頁 2：顯示與安全保護 (Display & Safety)

| 設定欄位 | `config.json` 鍵值 | 預設值 | 說明 |
| :--- | :--- | :--- | :--- |
| **主題配色** | `theme` | `"system"` | 介面外觀主題：`"system"`（跟隨系統）、`"midnight"`（深夜極黑）、`"graphite"`（石墨暗灰）、`"violet"`（紫羅蘭）或 `"light"`（淺色明亮）。 |
| **預設遮罩敏感內容 (NSFW)** | `blur_nsfw` | `true` | 自動模糊標記為成人或敏感內容的圖片，使用者點擊後方可臨時解鎖檢視。 |
| **顯示卡片角標** | `show_card_badges` | `true` | 在網格卡片上常駐顯示格式徽章（`PNG`、`MP4`）、原始尺寸解析度（`1024×1024`）、生成平台標識與星級評分。 |
| **縮圖解析度規格** | `thumbnail_max_edge` | `384` | 縮圖最長邊像素規格（64 的整數倍最佳化）：`256`（緊湊省記憶體）、`384`（標準推薦平衡）、`448`（高畫質清晰）、`512`（超高畫質大螢幕）。 |
| **縮圖快取配額** | `thumbnail_cache_budget_mb` | `2048` | 分配給 WebP 縮圖磁碟快取的最大容量（MB）。當超出上限時，系統將自動啟動 LRU 機制淘汰最久未讀取的縮圖檔案。 |
| **清理縮圖快取** | 不適用 | 不適用 | 操作按鈕：立即從磁碟清除所有已生成的快取縮圖檔案。 |
| **診斷資訊** | 不適用 | 不適用 | 操作按鈕：開啟 Rayon 背景工作執行緒池狀態與 LRU 快取命中率即時指標。 |

---

## 分頁 3：堆疊與批次 (Stacks & Bursts)

| 設定欄位 | `config.json` 鍵值 | 預設值 | 說明 |
| :--- | :--- | :--- | :--- |
| **啟用自動堆疊** | `auto_stack` | `true` | 自動將提示詞高度相似且連續生成的批次圖片合併為撲克牌折疊卡片。 |
| **提示詞相似度聚合門檻** | `stack_similarity_threshold` | `0.85` | 合併為同一堆疊所需的最低正向提示詞 Jaccard 分詞相似度（0.0 至 1.0）。 |
| **生成批次最大時間窗口** | `stack_time_window_minutes` | `180` | 判定為同一次連拍跑圖的相鄰生成最大時間間隔（分鐘）。 |
| **允許同時展開多個堆疊** | `allow_multiple_open_stacks` | `false` | 設為 `false` 時，展開任一堆疊會自動收合其他已展開堆疊；設為 `true` 則保留所有已展開的堆疊內容。 |
| **重設已隱藏的警告** | 不適用 | 不適用 | 操作按鈕：恢復所有先前勾選「不再顯示」的確認警告對話框（如堆疊合併警告）。 |

---

## 分頁 4：生成環境互通 (Generation Interop)

| 設定欄位 | `config.json` 鍵值 | 預設值 | 說明 |
| :--- | :--- | :--- | :--- |
| **ComfyUI 基礎位址** | `comfyui_url` | `"http://127.0.0.1:8188"` | 本機運行的 ComfyUI 服務 HTTP 端點。附帶即時連線測試按鈕。 |
| **SD WebUI 基礎位址** | `webui_url` | `"http://127.0.0.1:7860"` | 本機 AUTOMATIC1111 / Forge / SD.Next 服務端點。附帶即時連線測試按鈕。 |

---

## 分頁 5：團隊協同與資料庫 (Team & Collaboration)

| 設定欄位 | `config.json` 鍵值 | 預設值 | 說明 |
| :--- | :--- | :--- | :--- |
| **資料庫儲存引擎** | `storage_backend` | `"sqlite"` | 驅動圖庫的資料庫後端：`"sqlite"`（本機預設）、`"mysql"` 或 `"postgres"`。 |
| **遠端資料庫連線字串** | `remote_connection_url` | `""` | 遠端共享資料庫連線字串（例如 `postgres://user:pass@host:5432/berry_studio`）。 |
| **工作站用戶端識別碼** | `client_identifier` | `""` | 唯一標識目前電腦名稱，用於變更日誌記錄與樂觀並行鎖 (OCC) 追蹤。 |
| **共享儲存根目錄對應** | `root_mappings` | `{}` | 將中央儲存根 UUID 雙向對應至本機作業系統檔案系統掛載目錄的映射表。 |
| **測試連線與延遲** | 不適用 | 不適用 | 測試遠端資料庫通訊狀況並顯示網路來回延遲（毫秒）。 |
| **開啟遷移精靈** | 不適用 | 不適用 | 開啟單機 SQLite 匯出並升級至 MySQL/PostgreSQL 的遷移精靈。 |

---

## 分頁 6：雲端備份與差異同步 (Cloud Backup & Media Mirroring)

| 設定欄位 | `config.json` 鍵值 | 預設值 | 說明 |
| :--- | :--- | :--- | :--- |
| **備份儲存服務類型** | `cloud_backup.provider` | `"local_path"` | 遠端儲存協定：`"local_path"`、`"webdav"` 或 `"s3"`。 |
| **WebDAV 端點與憑證** | `cloud_backup.webdav_*` | `""` | Nextcloud / Synology 等 WebDAV 服務之 URL 根位址、使用者名稱與存取密碼。 |
| **S3 端點與金鑰** | `cloud_backup.s3_*` | `""` | S3 / Cloudflare R2 / MinIO 物件儲存端點、儲存庫 Bucket、區域、存取金鑰與密鑰。 |
| **差異比對策略** | `cloud_sync.strategy` | `"fingerprint"` | 增量媒體同步策略：`"fingerprint"`（快速指紋：大小+ETag）或 `"checksum"`（嚴格校驗：完整 SHA-256 雜湊）。 |
| **並發傳輸執行緒數** | `cloud_sync.threads` | `4` | 雲端媒體同步並行背景工作執行緒數（1 至 8）。 |
| **頻寬限速** | `cloud_sync.bandwidth_limit_kbs` | `0` | 最大上傳速率上限（KB/s，0 為不限速）。 |

---

## 分頁 7：內建元數據解析引擎概覽 (Metadata Engines Overview)

展示內建無損元數據剖析器的即時運作狀態：
- AUTOMATIC1111 / SD.Next PNG `parameters` 區塊剖析器（運作中 🟢）
- ComfyUI 節點執行圖譜與 `workflow` JSON 剖析器（運作中 🟢）
- NovelAI `Comment` 與 `Description` 剖析器（運作中 🟢）
- Fooocus / Fooocus-MRE 參數文字區塊剖析器（運作中 🟢）
- InvokeAI `sd-metadata` 與 `invokeai_metadata` 剖析器（運作中 🟢）
- MP4 ISOBMFF 與 WebM EBML 視訊串流剖析器（運作中 🟢）

---

## 分頁 8：資料儲存與關於 (Storage & About)

- **軟體版本**：顯示目前安裝執行的版本號（如 `v0.3.0`）。
- **資料庫結構版本**：顯示 SQLite 資料表結構遷移等級（如 `資料庫版本 v14`）。
- **核心資料庫路徑**：`berry.db` 於本機檔案系統中的絕對路徑。
- **快速開啟系統資料夾按鈕**：
  - `打開所在目錄 (設定檔)`：開啟包含 `config.json` 的系統目錄。
  - `打開所在目錄 (資料庫)`：開啟包含 `berry.db` 與 WAL 預寫日誌的目錄。
  - `打開所在目錄 (縮圖快取)`：開啟 WebP 縮圖快取目錄。
  - `打開所在目錄 (AI 模型)`：開啟本機 ONNX AI 模型權重目錄。
