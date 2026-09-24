# Checkpoint 模型庫與 LoRA 觸發詞庫

在複雜的生成式 AI 工作流程中，妥善管理數以百計的 Stable Diffusion 大底模型 (Checkpoint) 與微調 LoRA 是一項重大挑戰。Omera 提供內建的模型編目、雜湊解析以及觸發詞庫集中管理機制。

---

## 1. Checkpoint 模型庫與快取 (`ModelManagerModal.vue`)

Omera 會在索引圖庫的同時，自動追蹤並編目作品所使用過的每一個 Checkpoint 大模型。

### 自動模型探索：
- 遍歷圖庫時，Omera 會自動從內嵌的 PNGInfo 或 EXIF 元數據中提取模型名稱與雜湊值。
- 開啟 **工具 > 模型管理器與快取...**，即可查閱全圖庫已收錄的所有模型列表、短雜湊、完整 SHA256 校驗碼以及關聯圖片總數。

### 透過 AUTOMATIC1111 `cache.json` 解析模型雜湊：
- 很多生圖工具僅在圖片中記錄 8 碼短雜湊（例如 `31e35c80`）。
- 若您電腦中已安裝 AUTOMATIC1111 / SD.Next：
  1. 在模型管理器中點擊 **「匯入 A1111 cache.json」**。
  2. 選取 WebUI 根目錄下的 `cache.json` 檔案。
  3. Omera 會立即將對應映射表寫入本機 `model_cache` 資料表中，將全圖庫中晦澀難懂的短雜湊瞬間轉換為易讀的模型完整名稱。

### Civitai SHA256 雜湊反向查詢：
- 針對本機尚未收錄名稱的新模型，可在屬性檢查器中直接點擊 Civitai 查詢按鈕，系統會透過雜湊在 Civitai 公開 API 中反查模型發布名稱與版本資訊。

---

## 2. LoRA 觸發詞庫 (`LoraManagerModal.vue`)

Low-Rank Adaptations (LoRA) 通常需要特定的啟用標籤或觸發詞 (Trigger Words)，方可在生圖時準確召喚角色細節、畫風或服裝特徵。

```
┌────────────────────────────────────────────────────────────────────────┐
│ LoRA 觸發詞庫管理                                                  [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ [🔍 搜尋模型名稱、Hash 或觸發詞... ]         [+ 新增 LoRA] [📁 掃描資料夾]│
├────────────────────────────────────────────────────────────────────────┤
│ LoRA 模型名稱           │ 建議權重   │ 觸發詞 (Activation Tags)│ 操作   │
├─────────────────────────┼────────────┼────────────────────────┼────────┤
│ CyberpunkCityStyle      │ 0.80       │ cyberpunk, neon signs, │ [複製] │
│                         │            │ futuristic alleys      │ [注入] │
│ GenshinRaidenShogun     │ 0.85       │ raiden shogun, purple  │ [複製] │
│                         │            │ braid, glowing katana  │ [注入] │
│ StudioGhibliVintage     │ 0.70       │ ghibli style, vintage  │ [複製] │
│                         │            │ watercolor, cel shaded │ [注入] │
└────────────────────────────────────────────────────────────────────────┘
```

### 核心功能亮點：
1. **檢查器中的自動辨識**：
   - 當檢視任一圖片時，右側屬性檢查器會自動剖析提示詞中的 `<lora:name:weight>` 標籤以及 ComfyUI 的 `LoraLoader` 節點。
   - 清楚列出作品引用的 LoRA 名稱、套用權重與登記在冊的觸發標籤。
2. **一鍵提示詞注入**：
   - 點擊 **「帶 <lora> 格式複製」**，即可複製標準格式（如 `<lora:name:0.8>`）至剪貼簿。
   - 點擊任一觸發詞膠囊，即可單獨複製該觸發詞，隨時貼入生圖工具中。
3. **自動解析 Civitai 側車檔案 (`.civitai.info`)**：
   - 若您從 Civitai 下載帶有附屬資訊檔案（`.civitai.info` 或 `.json`）的 LoRA，Omera 能自動提取模型 Hash、基底模型版本（SD 1.5、SDXL、Pony、Flux）、官方訓練觸發詞以及封面預覽圖。
4. **掃描本機 LoRA 目錄**：
   - 將 Omera 指向您的本機 LoRA 存放資料夾（例如 `models/Lora/`）。
   - Omera 會自動遞迴掃描所有的 `.safetensors` 檔案及同名預覽圖，在本地建置起完善可搜尋的離線 LoRA 知識手冊。
