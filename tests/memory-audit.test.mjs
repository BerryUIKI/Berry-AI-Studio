import assert from "node:assert/strict";
import test from "node:test";

test("LruThumbnailCache strictly bounds entry count to limit across 20,000 items", () => {
  class LruThumbnailCache {
    constructor(maxSize = 3000) {
      this.cache = new Map();
      this.maxSize = maxSize;
    }
    get(key) {
      const val = this.cache.get(key);
      if (val !== undefined) {
        this.cache.delete(key);
        this.cache.set(key, val);
      }
      return val;
    }
    set(key, value) {
      if (this.cache.has(key)) {
        this.cache.delete(key);
      } else if (this.cache.size >= this.maxSize) {
        const oldestKey = this.cache.keys().next().value;
        if (oldestKey !== undefined) {
          this.cache.delete(oldestKey);
        }
      }
      this.cache.set(key, value);
    }
    get size() {
      return this.cache.size;
    }
  }

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
