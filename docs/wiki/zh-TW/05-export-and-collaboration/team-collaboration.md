# 多資料庫團隊協同

針對需要多位藝術家在區域網路共享儲存（NAS、SMB、NFS）上並行協作的設計工作室、遊戲研發團隊與視覺特效公司，Berry AI Studio 可從單機版 SQLite 順暢升級為**多資料庫團隊協同工作室架構**。

---

## 1. 多資料庫儲存架構

Berry 透過非同步的 `StorageEngine` 抽象層，全面支援三種主流資料庫後端：

| 資料庫後端 | 建議團隊規模 | 並發並行模型 | 效能表現與特點 |
| :--- | :--- | :--- | :--- |
| **SQLite (預設)** | 單一使用者 | 單寫入者 / 多讀取者 WAL 模式 | 本機 NVMe SSD 上查詢延遲低於 0.5 毫秒。 |
| **MySQL 8.0+ / MariaDB** | 2 至 50+ 人同時在線 | 列級鎖定 (Row-level locking) 與 `ngram` 全文索引 | 在 500,000+ 龐大團隊資產庫中查詢延遲低於 5 毫秒。 |
| **PostgreSQL 14+** | 2 至 100+ 人同時在線 | MVCC、`tsvector` GIN 索引與 `LISTEN/NOTIFY` | 超低延遲廣播，支援即時協同通知事件。 |

```mermaid
graph TD
    NAS[(工作室共享 NAS: SMB / NFS / WebDAV)]
    DB[(中央團隊資料庫: PostgreSQL 14+ / MySQL 8+)]

    subgraph 工作站 A (Windows)
        A_UI[Berry UI 介面]
        A_Thumb[本機 NVMe 縮圖快取]
        A_UI --- A_Thumb
    end

    subgraph 工作站 B (macOS)
        B_UI[Berry UI 介面]
        B_Thumb[本機 NVMe 縮圖快取]
        B_UI --- B_Thumb
    end

    A_UI -->|磁碟掛載: Z:\ai_vault| NAS
    B_UI -->|磁碟掛載: /Volumes/ai_vault| NAS

    A_UI <-->|OCC 樂觀鎖版本校驗與同步| DB
    B_UI <-->|OCC 樂觀鎖版本校驗與同步| DB
```

---

## 2. 跨平台儲存根目錄對應 (Storage Root Mapping)

在跨平台團隊協作中，最大的阻礙是 Windows、macOS 與 Linux 對於同一個網路共享資料夾的路徑語法完全不同：
- Windows：`Z:\ai_vault\2026\character_01.png`
- macOS：`/Volumes/ai_vault/2026/character_01.png`
- Linux：`/mnt/nas/ai_vault/2026/character_01.png`

### Berry 的解決方案：
1. **平台無關的儲存根 UUID**：Berry 為共享根目錄分配一個全域唯一的 UUID（儲存在資料庫的 `storage_roots` 表中）。
2. **規格化 URI 儲存**：資料庫中一律以跨平台的標準格式保存路徑：
   ```
   berry://550e8400-e29b-41d4-a716-446655440000/2026/character_01.png
   ```
3. **用戶端掛載對應**：在 **偏好設定 > 團隊協同與資料庫** 中，每位藝術家只需將該「儲存根 UUID」對應到自己作業系統目前的掛載目錄。Berry 會在背景動態雙向轉譯路徑，使 macOS 藝術家標記的相簿與標籤，Windows 上的夥伴能即時無縫開啟。

---

## 3. 樂觀並行控制 (Optimistic Concurrency Control, OCC)

當多名團隊成員同時對同一批作品進行評分、打標籤或整理堆疊時，Berry 透過**樂觀並行控制 (OCC)** 杜絕資料衝突與互相覆蓋：

- 資料表中的每一筆資產記錄皆包含一個行級版本號 `version` 欄位。
- 當成員更新評分時，Berry 會提交原子操作：`set_file_rating_occ(file_id, new_rating, expected_version)`。
- **衝突仲裁策略**：
  - **星級評分與封面指定**：採最後寫入者獲勝 (Last-Write-Wins, LWW) 原則，並即時廣播更新介面。
  - **標籤與相簿**：採聯集合併 (Set-Union Merge) 原則（若成員 A 為圖片新增標籤「角色」，成員 B 新增標籤「概念圖」，兩者標籤均會被妥善保留）。
  - **檔案刪除**：採用軟刪除狀態旗標，防止幽靈記錄重建。

---

## 4. 三級即時協同同步架構

Berry 透過階梯式同步機制維持全團隊工作站的狀態一致：

1. **第 1 級：變更日誌輪詢 (Change Log Polling，預設 / 零運維)**：
   - 每個客戶端每隔 3 秒向資料庫的 `change_log` 交易日誌表輪詢新事件。無需額外配置複雜伺服器。
2. **第 2 級：PostgreSQL `LISTEN / NOTIFY` (<50 ms 低延遲)**：
   - 使用 PostgreSQL 時，Berry 自動建立非同步通知通道。任一工作站提交的變動會在 50 毫秒內廣播給所有協同客戶端。
3. **第 3 級：分散式 WebSocket 廣播中樞**：
   - 為大型企業級工作室提供的專用長連線廣播中繼服務。

---

## 5. 用戶端本機 NVMe 縮圖快取

若在 1Gbps 或 10Gbps 區域網路中頻繁傳輸海量高解析度縮圖，極易造成網路擁塞：
- Berry 將生成的 WebP 縮圖直接快取在各個工作站的**本機高速 NVMe SSD** 上（`thumbnails/` 目錄）。
- 當藝術家捲動共享圖庫時，縮圖會就地自快取讀取，網路流量趨近於零。
- 確保共享 NAS 的頻寬能完全釋放給重度渲染、LoRA 模型訓練與大檔案存取。

---

## 6. 中央資料庫遷移精靈 (`MigrationWizardModal.vue`)

若您原本使用單機版 SQLite，並希望升級為團隊共享資料庫：
1. 開啟 **偏好設定 > 團隊協同與資料庫**。
2. 點擊 **「開啟遷移精靈...」**。
3. Berry 會深入檢查您本機的 SQLite 庫，引導您選擇目標方言（MySQL 8.0+ 或 PostgreSQL 14+），並自動串流生成方針最佳化的 DDL 結構定義與批次 INSERT 交易指令碼。
4. 在資料庫伺服器上執行生成的 `.sql` 指令碼，即可完成圖庫升級。
