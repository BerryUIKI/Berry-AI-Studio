import assert from "node:assert/strict";
import test from "node:test";
import { performance } from "node:perf_hooks";

import { calculateGalleryColumns } from "../src/utils/gallery-layout.ts";

function generateMockItems(count) {
  const items = new Array(count);
  for (let i = 0; i < count; i++) {
    const width = (i % 3 === 0) ? 512 : (i % 3 === 1) ? 768 : 1024;
    const height = (i % 2 === 0) ? 768 : 512;
    items[i] = {
      id: i + 1,
      path: `/images/test_${i}.png`,
      metadata: { width, height },
      size_bytes: 1500000,
    };
  }
  return items;
}

function computeWaterfallGeometry(items, cols, itemWidth, gap, cardInfoHeight) {
  const columnHeights = new Array(cols).fill(0);
  const masonryItems = new Array(items.length);
  for (let index = 0; index < items.length; index++) {
    const file = items[index];
    let col = 0;
    let minH = columnHeights[0];
    for (let c = 1; c < cols; c++) {
      if (columnHeights[c] < minH) {
        minH = columnHeights[c];
        col = c;
      }
    }
    const sourceWidth = file.metadata?.width ?? 1;
    const sourceHeight = file.metadata?.height ?? 1;
    const ratio = sourceWidth > 0 && sourceHeight > 0 ? sourceHeight / sourceWidth : 1;
    const imageHeight = Math.max(96, Math.round(itemWidth * ratio));
    const height = imageHeight + cardInfoHeight;
    masonryItems[index] = {
      index,
      top: columnHeights[col],
      left: col * (itemWidth + gap),
      width: itemWidth,
      height,
      column: col,
    };
    columnHeights[col] += height + gap;
  }

  const columns = Array.from({ length: cols }, () => []);
  for (let i = 0; i < masonryItems.length; i++) {
    columns[masonryItems[i].column].push(masonryItems[i]);
  }
  const totalHeight = Math.max(...columnHeights);
  return { masonryItems, columns, totalHeight };
}

function findVisibleWaterfallItems(columns, viewportTop, viewportBottom) {
  const visible = [];
  for (let c = 0; c < columns.length; c++) {
    const col = columns[c];
    let low = 0;
    let high = col.length;
    while (low < high) {
      const mid = (low + high) >>> 1;
      if (col[mid].top + col[mid].height < viewportTop) low = mid + 1;
      else high = mid;
    }
    for (let i = low; i < col.length; i++) {
      const item = col[i];
      if (item.top > viewportBottom) break;
      visible.push(item);
    }
  }
  return visible;
}

test("benchmarks virtualized grid and waterfall scroll performance (1k, 10k, 50k)", () => {
  const datasets = [1_000, 10_000, 50_000];
  const containerWidth = 1440;
  const itemMinWidth = 220;
  const gap = 16;
  const CARD_INFO_HEIGHT = 56;
  const cols = calculateGalleryColumns(containerWidth, itemMinWidth, gap);
  const itemWidth = Math.min(itemMinWidth, containerWidth);
  const cardHeight = itemWidth + CARD_INFO_HEIGHT;
  const rowHeight = cardHeight + gap;
  const viewportHeight = 900;
  const overscan = 2;

  console.log(`\n======================================================`);
  console.log(`>>> Frontend Virtualization Benchmark (${cols} columns, viewport ${viewportHeight}px)`);
  console.log(`======================================================`);

  for (const size of datasets) {
    const items = generateMockItems(size);

    // 1. Grid Slicing Benchmark over 200 scroll steps
    const totalRows = Math.ceil(size / cols);
    const totalGridHeight = totalRows * rowHeight;
    const maxScroll = Math.max(0, totalGridHeight - viewportHeight);
    const scrollSteps = 200;
    let gridWorstFrameMs = 0;
    let gridLongTasks = 0;

    const t0Grid = performance.now();
    for (let step = 0; step < scrollSteps; step++) {
      const frameStart = performance.now();
      const scrollTop = (step / (scrollSteps - 1)) * maxScroll;
      const rawStart = Math.floor(scrollTop / rowHeight);
      const startRow = Math.max(0, rawStart - overscan);
      const visibleCount = Math.ceil(viewportHeight / rowHeight);
      const endRow = Math.min(totalRows - 1, startRow + visibleCount + overscan * 2);
      const startIndex = startRow * cols;
      const endIndex = Math.min(size - 1, (endRow + 1) * cols - 1);
      const slice = items.slice(startIndex, endIndex + 1);
      assert.ok(slice.length <= (visibleCount + overscan * 3) * cols);

      const frameDuration = performance.now() - frameStart;
      if (frameDuration > gridWorstFrameMs) gridWorstFrameMs = frameDuration;
      if (frameDuration > 50) gridLongTasks++;
    }
    const gridTotalMs = performance.now() - t0Grid;

    // 2. Waterfall Geometry + 200 Scroll Steps
    const t0Geom = performance.now();
    const { columns, totalHeight } = computeWaterfallGeometry(items, cols, itemWidth, gap, CARD_INFO_HEIGHT);
    const geomMs = performance.now() - t0Geom;

    const maxWfScroll = Math.max(0, totalHeight - viewportHeight);
    let wfWorstFrameMs = 0;
    let wfLongTasks = 0;

    const t0WfScroll = performance.now();
    for (let step = 0; step < scrollSteps; step++) {
      const frameStart = performance.now();
      const scrollTop = (step / (scrollSteps - 1)) * maxWfScroll;
      const viewportTop = Math.max(0, scrollTop - overscan * rowHeight);
      const viewportBottom = scrollTop + viewportHeight + overscan * rowHeight;
      const visible = findVisibleWaterfallItems(columns, viewportTop, viewportBottom);
      assert.ok(visible.length > 0);

      const frameDuration = performance.now() - frameStart;
      if (frameDuration > wfWorstFrameMs) wfWorstFrameMs = frameDuration;
      if (frameDuration > 50) wfLongTasks++;
    }
    const wfScrollTotalMs = performance.now() - t0WfScroll;

    console.log(`[Library Size: ${size.toLocaleString()} items]`);
    console.log(`  Grid 200 scroll frames: Total = ${gridTotalMs.toFixed(2)}ms, Avg = ${(gridTotalMs / scrollSteps).toFixed(3)}ms/frame, Max = ${gridWorstFrameMs.toFixed(3)}ms, Long Tasks (>50ms) = ${gridLongTasks}`);
    console.log(`  Waterfall geometry calculation: ${geomMs.toFixed(2)}ms`);
    console.log(`  Waterfall 200 scroll frames: Total = ${wfScrollTotalMs.toFixed(2)}ms, Avg = ${(wfScrollTotalMs / scrollSteps).toFixed(3)}ms/frame, Max = ${wfWorstFrameMs.toFixed(3)}ms, Long Tasks (>50ms) = ${wfLongTasks}`);

    assert.equal(gridLongTasks, 0, "No main-thread grid scroll task should exceed 50ms");
    assert.equal(wfLongTasks, 0, "No main-thread waterfall scroll task should exceed 50ms");
  }
});

test("simulates thumbnail memory LRU and object URL lifecycle under continuous scroll", () => {
  const LRU_CAPACITY = 3000;
  const memoryCache = new Map();
  let revokedCount = 0;
  let allocatedCount = 0;

  function accessThumbnail(key) {
    if (memoryCache.has(key)) {
      const val = memoryCache.get(key);
      memoryCache.delete(key);
      memoryCache.set(key, val);
      return val;
    }
    if (memoryCache.size >= LRU_CAPACITY) {
      const oldestKey = memoryCache.keys().next().value;
      memoryCache.delete(oldestKey);
      revokedCount++;
    }
    const objectUrl = `blob:http://localhost/thumb_${key}`;
    allocatedCount++;
    memoryCache.set(key, objectUrl);
    return objectUrl;
  }

  // Simulate scrolling through 10,000 thumbnail views
  for (let i = 0; i < 10_000; i++) {
    accessThumbnail(`img_${i}_tier_384`);
  }

  assert.equal(memoryCache.size, LRU_CAPACITY);
  assert.equal(allocatedCount, 10_000);
  assert.equal(revokedCount, 7_000);
  console.log(`\nLRU Memory Audit Simulation:\n  Total thumbnails viewed: 10,000\n  Active LRU entries retained: ${memoryCache.size}\n  Evicted & revoked object URLs: ${revokedCount}\n  Bounded footprint guarantee: PASSED`);
});
