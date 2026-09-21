import assert from "node:assert/strict";
import test from "node:test";

import { calculateGalleryColumns } from "../src/utils/gallery-layout.ts";

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

