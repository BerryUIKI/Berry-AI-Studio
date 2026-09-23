import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";

import { FileDetailsManager } from "../src/utils/file-details.ts";

function createMockFile(id, modified_at = 1000, overrides = {}) {
  return {
    id,
    folder_id: 1,
    path: `/media/image_${id}.png`,
    size_bytes: 1024,
    modified_at,
    rating: undefined,
    is_favorite: false,
    is_nsfw: false,
    metadata: null,
    ...overrides,
  };
}

test("Select A -> Mutate A -> Select B -> Reselect A preserves mutated ratings, favorites, and NSFW", async () => {
  const manager = new FileDetailsManager(64);

  // 1. Initial selection of A and hydration
  const fileA = createMockFile(1, 1000);
  const hydratedA = createMockFile(1, 1000, {
    metadata: { prompt: "a cute cat", model: "sdxl" },
    rating: undefined,
    is_favorite: false,
  });

  const resA = await manager.hydrate(fileA, async () => hydratedA);
  assert.ok(resA);
  assert.equal(resA.details.metadata?.prompt, "a cute cat");

  // 2. User mutates A (sets rating = 5, favorite = true, nsfw = true)
  manager.update(1, { rating: 5, is_favorite: true, is_nsfw: true });

  // 3. User navigates to file B
  const fileB = createMockFile(2, 1000);
  const hydratedB = createMockFile(2, 1000, {
    metadata: { prompt: "a dog", model: "sdxl" },
  });
  await manager.hydrate(fileB, async () => hydratedB);

  // 4. User navigates back to file A
  const cachedA = manager.get(fileA);
  assert.ok(cachedA, "A should be retrieved from cache");
  assert.equal(cachedA.rating, 5, "Cached entry must reflect mutated rating");
  assert.equal(cachedA.is_favorite, true, "Cached entry must reflect mutated favorite");
  assert.equal(cachedA.is_nsfw, true, "Cached entry must reflect mutated NSFW");
  assert.equal(cachedA.metadata?.prompt, "a cute cat", "Cached entry must retain parsed metadata");
});

test("Updating A's source revision while remaining selected invalidates cache and triggers fresh hydration", async () => {
  const manager = new FileDetailsManager(64);

  const fileA = createMockFile(1, 1000);
  const initialDetails = createMockFile(1, 1000, {
    metadata: { prompt: "initial version" },
  });
  await manager.hydrate(fileA, async () => initialDetails);
  assert.ok(manager.get(fileA));

  // External modification changes modified_at to 2000
  const updatedA = createMockFile(1, 2000);
  assert.notEqual(
    FileDetailsManager.revisionKey(fileA),
    FileDetailsManager.revisionKey(updatedA),
    "Revision key must change when modified_at changes",
  );

  // Cache lookup for updated file revision must miss
  const cacheHit = manager.get(updatedA);
  assert.equal(cacheHit, null, "Cache must invalidate when revision changes");

  // Re-hydrating fetches the updated version
  const freshDetails = createMockFile(1, 2000, {
    metadata: { prompt: "re-generated version" },
  });
  const res = await manager.hydrate(updatedA, async () => freshDetails);
  assert.ok(res);
  assert.equal(res.details.metadata?.prompt, "re-generated version");
});

test("Fences stale in-flight detail requests across reset and revision mismatch", async () => {
  const manager = new FileDetailsManager(64);

  let resolveRequest;
  const delayedFetch = () =>
    new Promise((resolve) => {
      resolveRequest = resolve;
    });

  const fileA = createMockFile(1, 1000);
  const hydrationPromise = manager.hydrate(fileA, delayedFetch);

  // While request is in-flight, library resets (e.g. folder change or reload)
  manager.reset();

  // Delayed response finally resolves with old data
  resolveRequest(createMockFile(1, 1000, { metadata: { prompt: "stale" } }));
  const result = await hydrationPromise;

  assert.equal(result, null, "Stale in-flight response must be fenced and return null");
  assert.equal(manager.get(fileA), null, "Stale response must not populate cache");
});

test("Safe merge guarantees current user modifications are never overwritten by hydrated responses", () => {
  const manager = new FileDetailsManager(64);

  const currentSelection = createMockFile(1, 1000, {
    rating: 4,
    is_favorite: true,
    is_nsfw: false,
  });

  // Older or background hydrated object has rating undefined and favorite false
  const hydratedDetails = createMockFile(1, 1000, {
    rating: undefined,
    is_favorite: false,
    is_nsfw: true,
    metadata: { prompt: "complex prompt", steps: 30 },
  });

  const merged = manager.merge(currentSelection, hydratedDetails);

  assert.equal(merged.rating, 4, "Current selection rating must be preserved");
  assert.equal(merged.is_favorite, true, "Current selection favorite must be preserved");
  assert.equal(merged.is_nsfw, false, "Current selection NSFW must be preserved");
  assert.equal(merged.metadata?.prompt, "complex prompt", "Hydrated metadata must be populated");
  assert.equal(merged.metadata?.steps, 30);
});

test("Implements true LRU promotion and bounded eviction", async () => {
  const manager = new FileDetailsManager(3);

  const file1 = createMockFile(1);
  const file2 = createMockFile(2);
  const file3 = createMockFile(3);
  const file4 = createMockFile(4);

  await manager.hydrate(file1, async () => file1);
  await manager.hydrate(file2, async () => file2);
  await manager.hydrate(file3, async () => file3);
  assert.equal(manager.size, 3);

  // Access file1 -> promotes file1 to most recently used
  assert.ok(manager.get(file1));

  // Add file4 -> should evict file2 (since file1 was promoted, file2 became least recently used)
  await manager.hydrate(file4, async () => file4);
  assert.equal(manager.size, 3);

  assert.ok(manager.get(file1), "file1 should still be present (promoted)");
  assert.equal(manager.get(file2), null, "file2 should have been evicted as LRU");
  assert.ok(manager.get(file3), "file3 should still be present");
  assert.ok(manager.get(file4), "file4 should still be present");
});

test("App.vue integrates FileDetailsManager and watches full revision identity", async () => {
  const appSource = await fs.readFile(new URL("../src/App.vue", import.meta.url), "utf8");

  // Watches full revision identity, not just id
  assert.match(
    appSource,
    /const selectedRevisionKey = computed\(\(\)\s*=>\s*selectedFile\.value \? FileDetailsManager\.revisionKey\(selectedFile\.value\) : null,?\s*\);/s,
    "App.vue must compute complete revision identity for selectedFile",
  );
  assert.match(
    appSource,
    /watch\(selectedRevisionKey,\s*\(\)\s*=>\s*\{/,
    "App.vue must watch complete revision identity",
  );

  // Mutation updates
  assert.match(
    appSource,
    /fileDetailsManager\.update\(id,\s*\{\s*rating:\s*rating\s*\?\?\s*undefined\s*\}\);/,
    "onBatchRate updates fileDetailsManager",
  );
  assert.match(
    appSource,
    /fileDetailsManager\.update\(fileId,\s*\{\s*rating:\s*rating\s*\?\?\s*undefined\s*\}\);/,
    "onFileRated updates fileDetailsManager",
  );
  assert.match(
    appSource,
    /fileDetailsManager\.update\(id,\s*\{\s*is_favorite:\s*isFavorite\s*\}\);/,
    "onBatchToggleFavorite updates fileDetailsManager",
  );
  assert.match(
    appSource,
    /fileDetailsManager\.update\(id,\s*\{\s*is_nsfw:\s*isNsfw\s*\}\);/,
    "onBatchToggleNsfw updates fileDetailsManager",
  );
  assert.match(
    appSource,
    /fileDetailsManager\.update\(file\.id,\s*file\);/,
    "onUpdateFile updates fileDetailsManager",
  );
  assert.match(
    appSource,
    /fileDetailsManager\.reset\(\);/,
    "loadFiles resets fileDetailsManager",
  );
});
