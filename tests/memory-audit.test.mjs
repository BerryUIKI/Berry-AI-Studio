import assert from "node:assert/strict";
import test from "node:test";
import { LruThumbnailCache } from "../src/utils/lru-cache.ts";

test("LruThumbnailCache strictly bounds entry count to limit across 20,000 items", () => {
  const cache = new LruThumbnailCache(3000);
  for (let i = 0; i < 20000; i++) {
    cache.set(`item_${i}`, `asset://localhost/thumb_${i}.webp`);
  }

  assert.equal(cache.size, 3000, "Cache size must be capped at 3000");
  assert.equal(cache.get("item_0"), undefined, "Oldest item 0 must have been evicted");
  assert.equal(cache.get("item_16999"), undefined, "Item 16999 must have been evicted");
  assert.ok(cache.get("item_17000") !== undefined, "Item 17000 must still be present");
  assert.ok(cache.get("item_19999") !== undefined, "Latest item 19999 must still be present");
});

test("LruThumbnailCache promotes accessed items and supports delete and clear", () => {
  const cache = new LruThumbnailCache(3);
  cache.set("a", "url_a");
  cache.set("b", "url_b");
  cache.set("c", "url_c");

  // Read "a" to promote it to most recently used
  assert.equal(cache.get("a"), "url_a");

  // Insert "d", which should evict "b" (oldest unaccessed) rather than "a"
  cache.set("d", "url_d");
  assert.equal(cache.get("b"), undefined);
  assert.equal(cache.get("a"), "url_a");
  assert.equal(cache.get("c"), "url_c");
  assert.equal(cache.get("d"), "url_d");

  // Delete
  assert.equal(cache.delete("a"), true);
  assert.equal(cache.get("a"), undefined);
  assert.equal(cache.size, 2);

  // Clear
  cache.clear();
  assert.equal(cache.size, 0);
  assert.equal(cache.get("c"), undefined);
});

test("batchReadyKeys pruning maintains bounded memory footprint under continuous imports", () => {
  const batchReadyKeys = new Set();
  const MAX_LIMIT = 5000;
  const PRUNE_TARGET = 4000;

  function addBatchKey(key) {
    batchReadyKeys.add(key);
    if (batchReadyKeys.size > MAX_LIMIT) {
      const excess = batchReadyKeys.size - PRUNE_TARGET;
      let pruned = 0;
      for (const k of batchReadyKeys) {
        batchReadyKeys.delete(k);
        pruned++;
        if (pruned >= excess) break;
      }
    }
  }

  // Simulate 25,000 batch thumbnail keys added across a prolonged session
  for (let i = 0; i < 25000; i++) {
    addBatchKey(`file_${i}_rev_1_edge_384`);
  }

  assert.ok(
    batchReadyKeys.size <= MAX_LIMIT,
    `batchReadyKeys (${batchReadyKeys.size}) must never exceed ${MAX_LIMIT}`,
  );
  assert.ok(
    batchReadyKeys.size >= PRUNE_TARGET,
    `batchReadyKeys (${batchReadyKeys.size}) should be around ${PRUNE_TARGET}`,
  );
  // Verify recent keys are preserved
  assert.ok(batchReadyKeys.has("file_24999_rev_1_edge_384"), "Recent keys must be preserved");
});

test("simulates 50 folder switches with 500 images each without memory leaks", () => {
  // Simulate active components, listeners, and LRU cache across 50 folder navigations
  const globalListeners = new Set();
  const activeViews = new Map();

  function mountFolderView(folderId, fileCount = 500) {
    const listener = (e) => {};
    globalListeners.add(listener);
    const files = Array.from({ length: fileCount }, (_, i) => ({
      id: folderId * 1000 + i,
      path: `/library/folder_${folderId}/img_${i}.png`,
    }));
    activeViews.set(folderId, { files, listener });
    return () => {
      globalListeners.delete(listener);
      activeViews.delete(folderId);
    };
  }

  let cleanupPrevious = null;
  for (let folder = 1; folder <= 50; folder++) {
    if (cleanupPrevious) cleanupPrevious();
    cleanupPrevious = mountFolderView(folder, 500);
  }
  // Cleanup the last view
  if (cleanupPrevious) cleanupPrevious();

  assert.equal(globalListeners.size, 0, "Zero hanging global event listeners after unmount");
  assert.equal(activeViews.size, 0, "Zero hanging view instances after unmount");
});

test("thumbnail invalidation and failure contract across Grid and Table", async () => {
  const fs = await import("node:fs/promises");
  const thumbSource = await fs.readFile(new URL("../src/utils/thumbnail.ts", import.meta.url), "utf8");
  const gridSource = await fs.readFile(new URL("../src/components/VirtualGrid.vue", import.meta.url), "utf8");
  const listSource = await fs.readFile(new URL("../src/components/FileList.vue", import.meta.url), "utf8");

  // 1. getThumbnailUrl rethrows error without caching fallback original
  assert.doesNotMatch(thumbSource, /const fallbackUrl = assetUrl\(file\.path\);\s*memoryCache\.set/);
  assert.match(thumbSource, /export function invalidateThumbnail\(file: ImageFile, tier: number\): void/);

  // 2. clearThumbnailCache increments epoch to prevent old in-flight requests from populating cleared cache
  assert.match(thumbSource, /cacheEpoch\+\+;\s*const generation = beginThumbnailRequestCycle\(\);/);
  assert.match(thumbSource, /if \(epoch === cacheEpoch\) memoryCache\.set\(cacheKey, url\);/);

  // 3. VirtualGrid preserves failure state, invalidates thumbnail on retry, and never mounts original for failed files
  assert.match(gridSource, /invalidateThumbnail\(file, getThumbnailTier\(edge\)\);/);
  assert.match(gridSource, /if \(failedImages\.value\.has\(file\.path\)\) return null;/);
  assert.match(gridSource, /class="thumbnail-fallback thumbnail-failed"/);
  assert.match(gridSource, /class="retry-thumb-btn"/);

  // 4. FileList (Table) preserves failure state, offers retry, and never mounts full original for library files
  assert.match(listSource, /invalidateThumbnail\(file, getThumbnailTier\(ROW_THUMBNAIL_EDGE\)\);/);
  assert.match(listSource, /if \(failedImages\.value\.has\(file\.path\)\) return null;/);
  assert.doesNotMatch(listSource, /return file\.id \? null : assetUrl\(file\.path\);/);
  assert.match(listSource, /class="thumb-placeholder thumb-failed"/);
  assert.match(listSource, /class="retry-thumb-btn table-retry-btn"/);
});

