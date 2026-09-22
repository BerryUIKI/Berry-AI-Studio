# Database & Cache Maintenance

Berry AI Studio is engineered for continuous low-maintenance operation. However, as you curate, delete, and modify tens of thousands of artworks, performing periodic database compaction and cache management helps maintain peak performance.

---

## 1. Database Management Modal (`DatabaseManagerModal.vue`)

Open the management window via **File > Database Management...** or from the quick-tools footer in the left navigation sidebar.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Database Management & Storage Compaction                           [✕] │
├────────────────────────────────────────────────────────────────────────┤
│ Storage Metrics                                                        │
│ • Database File:      berry.db (WAL Mode)                              │
│ • Indexed Files:      48,210 files across 6 folders                    │
│ • Total Disk Size:    128.4 MB (Database) / 1.42 GB (Thumbnails)       │
│ • Albums & Tags:      12 albums, 45 tags                               │
│ • Schema Version:     v14 (14 applied migrations)                      │
│ • Freelist Pages:     1,420 pages (~5.6 MB reclaimable)                │
├────────────────────────────────────────────────────────────────────────┤
│ Maintenance Actions                                                    │
│ [ 🧹 Compact Database (VACUUM) ]   [ 💾 Export Backup Snapshot ]       │
│ [ ↺ Restore Database from Backup ] [ 🗑 Purge Thumbnail Cache ]        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. SQLite Compaction (`VACUUM`)

### What is Database Fragmentation?
When you delete items, remove tags, or dissolve stacks, SQLite marks the underlying disk pages as "free" (freelist) rather than immediately shrinking the `.db` file on disk.

### Running `VACUUM`:
- Clicking **"Compact Database (VACUUM)"** triggers SQLite's native vacuuming routine.
- Berry rebuilds the database file into a clean, contiguous structure, eliminating freelist pages and reducing database file size on disk.
- **Safety**: Vacuuming is fully transactional. If power is lost during compaction, SQLite rolls back safely without data corruption.

---

## 3. Database Backup & Restoration

### Creating a Local Backup (`backup_database`)
- Click **"Export Backup Snapshot"** to create a verified, non-locking backup of your database.
- Berry uses SQLite's online backup API, allowing you to create backups while the application remains fully usable.

### Restoring from Backup (`restore_database`)
- If you need to restore your library on a new computer or revert accidental changes, click **"Restore Database from Backup"**.
- Berry creates a safety rollback of your current database, swaps in the backup file, and immediately hydrates the studio with the restored library state.

---

## 4. Thumbnail Cache Management & Eviction

Thumbnails are stored in `<app_data_dir>/thumbnails/` as high-efficiency WebP files.

### Configurable Disk Cache Budget:
- In **Settings > Display & Safety**, you can configure the **Thumbnail Cache Budget** (default: `2048 MB` / 2 GB).
- Berry tracks the access timestamp of every thumbnail file in the `thumbnail_cache_entries` database table.
- When total thumbnail size exceeds your budget, Berry automatically evicts the oldest accessed files using an **LRU (Least Recently Used)** policy.

### Purging the Cache:
- If you want to free up disk space immediately, click **"Purge Thumbnail Cache"**.
- Berry deletes all cached WebP files from disk and clears the manifest table. Thumbnails will be regenerated lazily on demand when you browse folders again.

---

## 5. Thumbnail Diagnostics Modal (`ThumbnailDiagnosticsModal.vue`)

For performance monitoring and debugging:
- Open **Settings > Display & Safety > Diagnostics** to view:
  - Active Rayon worker thread count (e.g. `4 workers running`).
  - Pending thumbnail generation queues.
  - Completed vs. canceled generation jobs.
  - In-memory LRU cache hit rates.
