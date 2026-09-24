# 隱私架構與安全機制

Omera 圍繞 **100% 本機優先 (Local-First)、零遙測數據收集 (Zero-Telemetry)** 的嚴格設計哲學而建構。在生成式 AI 創作深度融入商業原創風格、未公開角色設計與機密客戶委託的時代，Omera 確保您的創意成果始終封閉且安全地保存在您自己的電腦中。

---

## 1. 零遙測與純本機離線運作

### 絕無任何暗中外連回報行為 (No Phone-Home)
- Omera **完全不包含任何追蹤像素 (Tracking Pixels)、任何第三方數據分析 SDK 或任何自動崩潰日誌上報服務**（杜絕 Google Analytics、Sentry、Mixpanel 或 PostHog 等）。
- 您可以在完全中斷網際網路連線的離線環境，或嚴格受到實體隔離 (Air-Gapped) 的企業內網環境中流暢運行 Omera，所有核心功能分毫不受影響。

### 出站網路請求之明確邊界
Omera **僅且僅在**以下三種經使用者明確授權的場景下發起對外網路通訊：
1. **軟體版本檢查**：當開啟「啟動時自動檢查更新」（或手動點擊 `說明 > 檢查更新...`）時，Omera 會查詢公開的 GitHub Releases API（`https://api.github.com/repos/BerryUIKI/Omera/releases/latest`）。
2. **Civitai 模型雜湊解析**：當您在檢查器中主動點擊未知模型的「在 Civitai 中查詢」按鈕時，Omera 會向 Civitai 公開 API 發送單次 HTTP 查詢請求。
3. **雲端備份與團隊協同**：當您在設定中明確配置並啟用了 AWS S3、WebDAV 或遠端中央 PostgreSQL/MySQL 團隊資料庫時。

---

## 2. 100% 本機端 AI 模型推論

Omera 所有的類神經網路演算法皆透過內嵌的 **ONNX Runtime** 執行階段 (`ort`)，直接在您的本機 CPU 或 GPU 上進行計算：

- **CLIP / SigLIP 向量嵌入**：圖片前處理、特徵提取與分詞編碼全程在本地完成，絕不會將任何提示詞字串、搜尋字句或圖片像素上傳至外部伺服器。
- **WD14 動漫自動標籤器**：推論模型直接載入本地權重（`models/` 目錄），辨識預測出的標籤直接就地寫入本機 SQLite 資料庫中。

---

## 3. 隱私優先的匯出與脫敏擦除

當需將作品匯出打包分享至公開社群平台或交付外部客戶時，Omera 提供強大的 **4 級隱私脫敏引擎**：

- 在將圖片上傳至社群網站或 Discord 前，可一鍵徹底抹除內嵌的 ComfyUI 流程圖、正向/負向提示詞、生圖種子與 LoRA 標籤（詳見 [批次匯出與網頁畫冊](../05-export-and-collaboration/export-and-web-showcase.md)）。
- 提供「完全淨化」模式，可剔除所有非像素的 EXIF 標頭與 ICC 色彩描述檔，只保留乾淨純粹的像素資料。

---

## 4. 開源透明性與授權條款

Omera 是完全自由且開源的軟體，採用 **GNU Affero 通用公共授權條款第 3 版 (AGPL-3.0)** 進行發布：

- 每一行 Rust 後端架構程式碼、Tauri 跨行程橋接命令以及 Vue 3 前端組件程式碼，均完整公開於 [GitHub 倉庫](https://github.com/BerryUIKI/Omera) 供大眾審閱。
- 您享有完全的自由度在個人工作站或商業工作室環境中稽核、編譯、分支或部署 Omera。
