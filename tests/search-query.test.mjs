import assert from "node:assert/strict";
import test from "node:test";
import { criteriaToQuery } from "../src/utils/search.ts";

test("serializes model hashes without dropping the structured filter", () => {
  assert.equal(criteriaToQuery({ model_hash: "abc12345" }), "hash:abc12345");
  assert.equal(criteriaToQuery({ model_hash: "hash with spaces" }), 'hash:"hash with spaces"');
});

test("serializes video search criteria correctly", () => {
  assert.equal(criteriaToQuery({ media_type: "video" }), "type:video");
  assert.equal(criteriaToQuery({ min_duration: 5, max_duration: 15 }), "duration:5..15");
  assert.equal(criteriaToQuery({ min_duration: 10 }), "duration:>=10");
  assert.equal(criteriaToQuery({ max_duration: 60 }), "duration:<=60");
  assert.equal(criteriaToQuery({ min_fps: 30, max_fps: 60 }), "fps:30..60");
  assert.equal(criteriaToQuery({ min_fps: 24 }), "fps:>=24");
});

