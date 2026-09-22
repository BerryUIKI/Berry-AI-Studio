# Large-Library Performance Benchmark Report (1k, 10k, 50k Items)

**Date:** 2026-09-20  
**Test Machine:** Windows 11, AMD Ryzen 9 / SSD  
**Target Applications:** `berry-scan`, `berry-storage`, `VirtualGrid.vue`, `FileList.vue`

---

## 1. Executive Summary

This report documents reproducible benchmark measurements for Omera across small (1,000 items), medium (10,000 items), and large (50,000 items) media libraries.

The tests evaluate:
1. **Time to first usable gallery** from a warm SQLite database.
2. **Deep pagination latency** comparing existing `LIMIT 400 OFFSET N` with `COUNT(*) OVER()` versus sort-aware keyset cursors.
3. **50,000 thumbnail manifest synchronization and legacy cache adoption throughput**.
4. **Main-thread long tasks (>50ms)** and memory growth during continuous rapid scrolling.

All measurement harnesses are fully automated and reproducible via:
```bash
# Rust storage & scanner benchmarks
cargo bench --bench large_library -p berry-scan

# Frontend virtualized scroll and memory benchmarks
node --experimental-strip-types --test tests/frontend-scroll-bench.mjs
```

---

## 2. Benchmark Results

### 2.1 Time to First Usable Gallery Page (400 items + Exact Filtered Total)

| Library Size | Ingest Throughput | Warm Query (P50) | Warm Query (P90) | Target | Status |
|---|---|---|---|---|---|
| **1,000 items** | 40,711 rows/s | **4.48 ms** | 4.66 ms | < 1,000 ms | **PASS** |
| **10,000 items** | 22,148 rows/s | **23.65 ms** | 27.08 ms | < 1,000 ms | **PASS** |
| **50,000 items** | 17,981 rows/s | **143.55 ms** | 155.16 ms | < 1,000 ms | **PASS** |

> **Analysis:** Even on a massive 50k item library, the initial usable gallery view renders in ~143ms from a warm SQLite cache, well under the 1-second design budget.

---

### 2.2 Deep Pagination Traversal

Queries executed: `SELECT ... COUNT(*) OVER() AS total_count FROM files WHERE folder_id = ? ORDER BY modified_at DESC, id DESC LIMIT 400 OFFSET N`.

| Library Size | Offset / Page | Offset Query Latency (Median) | Keyset Cursor Latency (Median) | Keyset Speedup |
|---|---|---|---|---|
| **1,000 items** | Offset 0 (Page 1) | 4.09 ms | - | - |
| | Offset 400 (Page 2) | 5.32 ms | - | - |
| | Offset 600 (Page 2) | 6.16 ms | - | - |
| **10,000 items** | Offset 0 (Page 1) | 24.41 ms | - | - |
| | Offset 4,000 (Page 11) | 34.55 ms | - | - |
| | Offset 8,000 (Page 21) | 49.96 ms | **351.60 µs** (raw) | **18.3x** (vs 6.44ms raw) |
| | Offset 9,600 (Page 25) | 55.57 ms | - | - |
| **50,000 items** | Offset 0 (Page 1) | 135.74 ms | - | - |
| | Offset 10,000 (Page 26) | 166.21 ms | - | - |
| | Offset 25,000 (Page 63) | 219.19 ms | - | - |
| | Offset 40,000 (Page 101) | 266.48 ms | **617.90 µs** (raw) | **84.3x** (vs 52.11ms raw) |
| | Offset 49,600 (Page 125) | 299.52 ms | - | - |

---

### 2.3 50k Thumbnail Manifest Synchronization

Tests `synchronize_thumbnail_manifest` against a cache directory containing 50,000 pre-existing WebP thumbnail files:

- **Disk file generation:** 50,000 files created in 16.45 s (3,039 files/s).
- **Manifest synchronization time:** **3.26 seconds**.
- **Sync throughput:** **15,347 files/s**.
- **Cache budget verification:** All 50,000 records persisted in `thumbnail_cache_entries` with accurate disk sizing and LRU timestamps.

---

### 2.4 Frontend Rapid Scrolling & Memory Audit

Measured across 200 consecutive scroll frames (0px to bottom) on a 1440x900 viewport (6 columns):

| Library Size | Mode | Total Scroll Time | Avg Frame Time | Worst Frame Time | Tasks > 50 ms |
|---|---|---|---|---|---|
| **1,000 items** | Grid | 0.99 ms | 0.005 ms | 0.742 ms | **0** |
| | Waterfall | 0.90 ms | 0.005 ms | 0.057 ms | **0** |
| **10,000 items** | Grid | 1.46 ms | 0.007 ms | 1.296 ms | **0** |
| | Waterfall | 0.29 ms | 0.001 ms | 0.024 ms | **0** |
| **50,000 items** | Grid | 0.12 ms | 0.001 ms | 0.005 ms | **0** |
| | Waterfall | 0.43 ms | 0.002 ms | 0.011 ms | **0** |

- **Waterfall Initial Geometry Calculation:** 50,000 items computed in **6.61 ms** (cached thereafter).
- **Memory LRU Audit:** Simulated 10,000 thumbnail views with 3,000 LRU limit: exactly 3,000 retained, 7,000 revoked. Zero uncollected object URLs.

---

## 3. Decision Matrix: Keyset Cursors and Gallery DTO

### 3.1 Keyset Cursors (`WHERE (sort_val, id) < (?, ?)`)
- **Empirical finding:** Keyset cursor reduces page row traversal from **52.11 ms to 617.90 µs** (84x faster).
- However, total latency for offset queries in 50k items is 266–299ms because `COUNT(*) OVER()` takes ~135ms to calculate the exact total count across 50,000 rows on every query.
- **Decision:**
  1. Keyset cursors should **not** be introduced blindly as a replacement for offset pagination without decoupling `COUNT(*) OVER()`.
  2. For libraries up to 10k items, offset queries take <55ms, making keyset unnecessary.
  3. Keyset cursors are approved for a future phase when paginated infinite scroll caches the total count on the initial page and uses cursor tokens for subsequent loads.

### 3.2 Dedicated Gallery DTO
- **Empirical finding:** PR `d1006075` already strips `parameters` and `raw` JSON blobs from gallery responses. The remaining structured fields (`prompt`, `negative_prompt`, `model_name`, `sampler`, `width`, `height`, `seed`, `cfg_scale`, `steps`) average ~450 bytes per item.
- For a 400-item page, payload size is ~180 KB. Transferring 180 KB over Tauri's in-process IPC takes under 2 ms.
- **Decision:**
  - A specialized DTO would save only ~80 KB per page and would require duplicating types and mapper logic across Rust and TypeScript.
  - Therefore, retaining the current projected `ImageFile` is recommended; no DTO refactoring is needed at this time.
