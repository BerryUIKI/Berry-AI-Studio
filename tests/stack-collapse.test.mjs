import assert from "node:assert/strict";
import test from "node:test";
import { collapseStackMembers, resolveStackHeroPaths } from "../src/utils/stack.ts";

function image(id, path, stackOrder) {
  return {
    id,
    folder_id: 1,
    path,
    size_bytes: 1,
    modified_at: 1,
    container: "png",
    metadata: null,
    stack_id: "stack-a",
    stack_order: stackOrder,
  };
}

test("collapse keeps only the summary hero when stack orders are duplicated", () => {
  const files = [image(1, "first.png", 0), image(2, "hero.png", 0), image(3, "third.png", 1)];
  const stackMap = { "stack-a": { count: 3, heroId: 2 } };

  assert.deepEqual(collapseStackMembers(files, stackMap).map((file) => file.path), ["hero.png"]);
  assert.equal(resolveStackHeroPaths(files, stackMap).get("stack-a"), "hero.png");
});

test("collapse deterministically falls back to the first zero-order member", () => {
  const files = [image(1, "first.png", 0), image(2, "second.png", 0), image(3, "third.png", 1)];
  const stackMap = { "stack-a": { count: 3, heroId: null } };

  assert.deepEqual(collapseStackMembers(files, stackMap).map((file) => file.path), ["first.png"]);
});

test("targeted collapse leaves other stacks and standalone images unchanged", () => {
  const first = image(1, "first.png", 0);
  const hidden = image(2, "hidden.png", 1);
  const other = { ...image(3, "other.png", 0), stack_id: "stack-b" };
  const standalone = { ...image(4, "standalone.png", 0), stack_id: null };
  const stackMap = {
    "stack-a": { count: 2, heroId: 1 },
    "stack-b": { count: 2, heroId: 3 },
  };

  assert.deepEqual(
    collapseStackMembers([first, hidden, other, standalone], stackMap, "stack-a").map((file) => file.path),
    ["first.png", "other.png", "standalone.png"],
  );
});
