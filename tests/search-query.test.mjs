import assert from "node:assert/strict";
import test from "node:test";
import { criteriaToQuery } from "../src/utils/search.ts";

test("serializes model hashes without dropping the structured filter", () => {
  assert.equal(criteriaToQuery({ model_hash: "abc12345" }), "hash:abc12345");
  assert.equal(criteriaToQuery({ model_hash: "hash with spaces" }), 'hash:"hash with spaces"');
});
