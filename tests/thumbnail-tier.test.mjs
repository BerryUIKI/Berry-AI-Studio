import assert from "node:assert/strict";
import test from "node:test";

import { selectThumbnailTier } from "../src/utils/thumbnail-tier.ts";

test("selects the smallest tier that covers rendered pixels", () => {
  assert.equal(selectThumbnailTier(130, 512, 1), 192);
  assert.equal(selectThumbnailTier(180, 512, 2), 384);
  assert.equal(selectThumbnailTier(32, 384, 2), 128);
});

test("respects the configured resolution ceiling", () => {
  assert.equal(selectThumbnailTier(360, 384, 2), 384);
  assert.equal(selectThumbnailTier(220, 448, 2), 448);
});

test("bounds invalid and unusually dense device scales", () => {
  assert.equal(selectThumbnailTier(120, 512, 0), 128);
  assert.equal(selectThumbnailTier(200, 512, 4), 448);
});
