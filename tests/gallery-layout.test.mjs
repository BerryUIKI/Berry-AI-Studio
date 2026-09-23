import assert from "node:assert/strict";
import test from "node:test";

import { calculateGalleryColumns, calculateGalleryTrackOffset } from "../src/utils/gallery-layout.ts";

test("adds and removes fixed-width columns as the gallery changes width", () => {
  assert.equal(calculateGalleryColumns(520, 200, 16), 2);
  assert.equal(calculateGalleryColumns(800, 200, 16), 3);
  assert.equal(calculateGalleryColumns(1080, 200, 16), 5);
});

test("keeps at least one column before measurement or in a narrow viewport", () => {
  assert.equal(calculateGalleryColumns(0, 200, 16), 1);
  assert.equal(calculateGalleryColumns(-100, 200, 16), 1);
  assert.equal(calculateGalleryColumns(120, 200, 16), 1);
  assert.equal(calculateGalleryColumns(Number.NaN, 200, 16), 1);
  assert.equal(calculateGalleryColumns(Number.POSITIVE_INFINITY, 200, 16), 1);
});

test("handles exact column fit boundaries and zero gap", () => {
  // Exactly 3 columns: 3 * 200 + 2 * 16 = 632
  assert.equal(calculateGalleryColumns(632, 200, 16), 3);
  // Just 1px below 3 columns: 631
  assert.equal(calculateGalleryColumns(631, 200, 16), 2);
  // Zero gap: 800 / 200 = 4 columns
  assert.equal(calculateGalleryColumns(800, 200, 0), 4);
  // Ultra-wide 4K display: 3840 width, 200 card, 16 gap -> floor((3840+16)/(216)) = 17
  assert.equal(calculateGalleryColumns(3840, 200, 16), 17);
});

test("calculates symmetrical track offset to center fixed-width columns and eliminate right gaps", () => {
  // 1000px container, 200px cards, 16px gap -> 4 cols (track = 4*200 + 3*16 = 848)
  // leftover = 152px -> offset = 76px left, 76px right
  assert.equal(calculateGalleryTrackOffset(1000, 4, 200, 16), 76);

  // Exact fit: 3 cols at 200px + 2*16 = 632px -> offset = 0
  assert.equal(calculateGalleryTrackOffset(632, 3, 200, 16), 0);

  // Narrow viewport smaller than 1 card: 150px container, 1 col at 200px -> offset = 0 (no negative shift)
  assert.equal(calculateGalleryTrackOffset(150, 1, 200, 16), 0);

  // Zero / negative / non-finite inputs return 0 safely
  assert.equal(calculateGalleryTrackOffset(0, 1, 200, 16), 0);
  assert.equal(calculateGalleryTrackOffset(-100, 1, 200, 16), 0);
  assert.equal(calculateGalleryTrackOffset(Number.NaN, 1, 200, 16), 0);
});

test("VirtualGrid enforces fixed card width and centered alignment across Grid and Masonry", async () => {
  const fs = await import("node:fs/promises");
  const gridSource = await fs.readFile(new URL("../src/components/VirtualGrid.vue", import.meta.url), "utf8");

  // 1. Preserves fixed card width per AGENTS.md performance rule (no stretching cards)
  assert.match(gridSource, /itemWidth\s*=\s*computed\(\(\)\s*=>\s*\{[^}]*Math\.min\(props\.itemMinWidth,\s*containerWidth\.value\)/s);

  // 2. Computes horizontalOffset from calculateGalleryTrackOffset
  assert.match(gridSource, /horizontalOffset\s*=\s*computed\(\(\)\s*=>\s*\{\s*return calculateGalleryTrackOffset/);

  // 3. Grid mode uses safe center to center column tracks symmetrically without right voids
  assert.match(gridSource, /justifyContent:\s*layout === ["']grid["'] \? ["']safe center["'] : undefined/);
  assert.match(gridSource, /\.virtual-content\s*\{[^}]*justify-content:\s*safe center;/s);

  // 4. Masonry mode applies horizontalOffset to item left positions
  assert.match(gridSource, /left:\s*horizontalOffset\.value \+ column \* \(itemWidth\.value \+ props\.gap\)/);
});


