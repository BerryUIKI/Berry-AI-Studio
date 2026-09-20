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
  assert.equal(calculateGalleryColumns(120, 200, 16), 1);
  assert.equal(calculateGalleryColumns(Number.NaN, 200, 16), 1);
});
