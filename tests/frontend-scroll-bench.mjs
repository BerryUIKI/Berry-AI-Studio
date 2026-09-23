import assert from "node:assert/strict";
import test from "node:test";
import { performance } from "node:perf_hooks";
import { WaterfallGeometry, visibleWaterfallItems, GalleryPages } from "../src/utils/gallery-state.ts";

const file = (id) => ({
  id,
  path: `/${id}.png`,
  container: "png",
  size_bytes: 1000,
  modified_at: 1000,
  metadata: { width: 512, height: id % 2 ? 768 : 512 },
});

test("production waterfall window matches full-scan oracle and preserves append geometry", () => {
  const files = Array.from({ length: 1000 }, (_, i) => file(i));
  const geometry = new WaterfallGeometry().update(files, 6, 220, 16);
  const first = geometry.items[0];

  // Append 1 more file: items[0] must remain unchanged
  files.push(file(1000));
  geometry.update(files, 6, 220, 16);
  assert.equal(geometry.items[0], first);

  // Binary search visible items matches linear filter oracle at various scroll offsets
  for (const top of [0, 999, 9000, geometry.height - 900]) {
    const expected = geometry.items.filter(
      (item) => item.top + item.height >= top && item.top <= top + 900,
    );
    assert.deepEqual(visibleWaterfallItems(geometry.columns, top, top + 900), expected);
  }

  // Layout change (column count changes from 6 to 3) invalidates geometry
  geometry.update(files, 3, 220, 16);
  assert.equal(geometry.columns.length, 3);
  assert.notEqual(geometry.items[0], first);
});

test("production page append deduplicates and replaces a fallback stack hero", () => {
  const pages = new GalleryPages();
  const first = { ...file(1), stack_id: "s" };
  const files = [first];
  const hero = { ...file(2), stack_id: "s" };

  // Page 2 contains the actual hero (heroId: 2) -> should replace fallback first
  assert.equal(
    pages.append(files, [first, hero, file(3)], { s: { count: 2, heroId: 2 } }, new Set()),
    true,
  );
  assert.deepEqual(files.map((f) => f.id), [2, 3]);

  // Page 3 contains duplicate item 3 and new item 4 -> deduplicates item 3
  pages.append(files, [file(3), file(4)], {}, new Set());
  assert.deepEqual(files.map((f) => f.id), [2, 3, 4]);
});

test("measure production waterfall visibility at 1k, 10k and 50k items", () => {
  for (const size of [1000, 10000, 50000]) {
    const files = Array.from({ length: size }, (_, i) => file(i));
    const geometry = new WaterfallGeometry().update(files, 6, 220, 16);
    const times = [];
    for (let step = 0; step < 200; step++) {
      const top = (step / 199) * Math.max(0, geometry.height - 900);
      const start = performance.now();
      const visible = visibleWaterfallItems(geometry.columns, top, top + 900);
      times.push(performance.now() - start);
      assert.ok(visible.length > 0 && visible.length < 60);
    }
    times.sort((a, b) => a - b);
    console.log(`${size} items: visibility p50=${times[100].toFixed(3)}ms p95=${times[190].toFixed(3)}ms`);
  }
});

test("VirtualGrid and FileList enforce incremental append without unbounded rebuilds", async () => {
  const fs = await import("node:fs/promises");
  const gridSource = await fs.readFile(new URL("../src/components/VirtualGrid.vue", import.meta.url), "utf8");
  const appSource = await fs.readFile(new URL("../src/App.vue", import.meta.url), "utf8");

  // 1. App.vue uses galleryPages.append in loadMoreFiles instead of full-array concatenation & collapse
  assert.match(appSource, /const replaced = galleryPages\.append\(/);
  assert.doesNotMatch(appSource, /files\.value = collapseInactiveStacks\(\[\.\.\.files\.value, \.\.\.appended\]\);/);

  // 2. VirtualGrid uses WaterfallGeometry and computes height without spreading items
  assert.match(gridSource, /const geometry = new WaterfallGeometry\(\);/);
  assert.match(gridSource, /geometry\.update\(/);
  assert.doesNotMatch(gridSource, /Math\.max\(\.\.\.masonryItems\.value\.map/);
});
