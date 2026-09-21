import assert from "node:assert/strict";
import test from "node:test";

import { identifyStackDrafts, identifyMultiStackDrafts } from "../src/utils/stack.ts";

test("identifies drafts when stack has a higher-rated hero", () => {
  const files = [
    { id: 1, path: "/img1.png", stack_id: "stack_1", stack_order: 0, rating: 5 },
    { id: 2, path: "/img2.png", stack_id: "stack_1", stack_order: 1, rating: 3 },
    { id: 3, path: "/img3.png", stack_id: "stack_1", stack_order: 2, rating: null },
    { id: 4, path: "/img4.png", stack_id: "stack_1", stack_order: 3, rating: 4 },
  ];

  const result = identifyStackDrafts(files, "stack_1");
  assert.ok(result !== null);
  assert.equal(result.hero.id, 1, "Image with 5 stars must be the hero");
  assert.equal(result.drafts.length, 3, "All 3 lower-rated images must be identified as drafts");
  assert.deepEqual(
    result.drafts.map((d) => d.id),
    [2, 3, 4],
  );
});

test("identifies drafts when all stack members are unrated", () => {
  const files = [
    { id: 10, path: "/img10.png", stack_id: "stack_2", stack_order: 0, rating: null },
    { id: 11, path: "/img11.png", stack_id: "stack_2", stack_order: 1, rating: null },
    { id: 12, path: "/img12.png", stack_id: "stack_2", stack_order: 2, rating: null },
  ];

  const result = identifyStackDrafts(files, "stack_2");
  assert.ok(result !== null);
  assert.equal(result.hero.id, 10, "Order 0 image must be the hero when all unrated");
  assert.equal(result.drafts.length, 2, "Other 2 unrated images must be drafts");
  assert.deepEqual(
    result.drafts.map((d) => d.id),
    [11, 12],
  );
});

test("returns null if stack has only 1 image or does not exist", () => {
  const files = [
    { id: 20, path: "/img20.png", stack_id: "single", stack_order: 0, rating: 5 },
  ];
  assert.equal(identifyStackDrafts(files, "single"), null);
  assert.equal(identifyStackDrafts(files, "nonexistent"), null);
});

test("multi-stack culling groups drafts across multiple selected stacks", () => {
  const files = [
    // Stack 1
    { id: 1, path: "/s1_1.png", stack_id: "s1", stack_order: 0, rating: 5 },
    { id: 2, path: "/s1_2.png", stack_id: "s1", stack_order: 1, rating: 2 },
    // Stack 2
    { id: 3, path: "/s2_1.png", stack_id: "s2", stack_order: 0, rating: 4 },
    { id: 4, path: "/s2_2.png", stack_id: "s2", stack_order: 1, rating: 1 },
    // Standalone
    { id: 5, path: "/standalone.png", stack_id: null, stack_order: 0, rating: 3 },
  ];

  const { heroes, drafts } = identifyMultiStackDrafts(files);
  assert.equal(heroes.length, 2, "Must identify 2 heroes");
  assert.equal(drafts.length, 2, "Must identify 2 drafts total");
  assert.deepEqual(
    drafts.map((d) => d.id),
    [2, 4],
  );
});
