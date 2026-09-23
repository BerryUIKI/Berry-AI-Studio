import assert from "node:assert/strict";
import test from "node:test";
import {
  collapseStackMembers,
  resolveStackHeroPaths,
  summarizeResultStacks,
} from "../src/utils/stack.ts";

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

test("result summaries count and select heroes only among matching members", () => {
  const matchingFirst = image(2, "matching-first.png", 1);
  const matchingSecond = image(3, "matching-second.png", 2);
  const other = { ...image(4, "other.png", 0), stack_id: "stack-b" };

  const summary = summarizeResultStacks([matchingSecond, matchingFirst, other]);
  assert.deepEqual(summary, {
    "stack-a": { count: 2, heroId: 2 },
  });
  assert.equal(summary["stack-a"].heroId, matchingFirst.id);
  assert.equal(summary["stack-a"].count, 2);
  assert.equal(summary["stack-b"], undefined);
});

test("expanded stack hero-only gating prevents badge and control spam across 23 members", () => {
  const members = Array.from({ length: 23 }, (_, i) => image(i + 1, `img_${i + 1}.png`, i));
  const stackMap = { "stack-a": { count: 23, heroId: 1 } };
  const heroPaths = resolveStackHeroPaths(members, stackMap);

  function isStackCover(file) {
    return Boolean(
      file.stack_id &&
      (stackMap[file.stack_id]?.count ?? 1) > 1 &&
      heroPaths.get(file.stack_id) === file.path,
    );
  }

  const covers = members.filter(isStackCover);
  assert.equal(covers.length, 1, "Only 1 item among 23 expanded members must be the stack cover");
  assert.equal(covers[0].path, "img_1.png");

  // Non-cover members must NOT display stack controls
  const nonCovers = members.filter((m) => !isStackCover(m));
  assert.equal(nonCovers.length, 22, "22 members must not be marked as stack cover");
});

test("VirtualGrid enforces hero-only stack controls and keyboard-accessible hover/focus styling", async () => {
  const fs = await import("node:fs/promises");
  const gridSource = await fs.readFile(new URL("../src/components/VirtualGrid.vue", import.meta.url), "utf8");

  // 1. Gating verification: badge-stack, compare-btn, cull-btn are only rendered on isStackCover(file)
  assert.match(gridSource, /<button\s+v-if="isStackCover\(file\)"\s+type="button"\s+class="card-badge badge-stack"/);
  assert.match(gridSource, /<button\s+v-if="isStackCover\(file\)"\s+type="button"\s+class="card-stack-compare-btn"/);
  assert.match(gridSource, /<button\s+v-if="isStackCover\(file\)"\s+type="button"\s+class="card-stack-cull-btn"/);

  // 2. Visual noise reduction: default badge-stack opacity is softened
  assert.match(gridSource, /\.badge-stack\s*\{[^}]*opacity:\s*0\.85/s);

  // 3. Hover / focus behavior: reveals on card hover, focus-within, and button focus-visible
  assert.match(gridSource, /\.grid-card:hover\s+\.badge-stack,\s*\.grid-card:focus-within\s+\.badge-stack,\s*\.badge-stack:focus-visible\s*\{/s);
  assert.match(gridSource, /\.grid-card:hover\s+\.card-stack-compare-btn,\s*\.grid-card:focus-within\s+\.card-stack-compare-btn,\s*\.card-stack-compare-btn:focus-visible\s*\{/s);
  assert.match(gridSource, /\.grid-card:hover\s+\.card-stack-cull-btn,\s*\.grid-card:focus-within\s+\.card-stack-cull-btn,\s*\.card-stack-cull-btn:focus-visible\s*\{/s);

  // 4. Focus rings for accessibility
  assert.match(gridSource, /\.card-stack-compare-btn:focus-visible\s*\{\s*outline:/);
  assert.match(gridSource, /\.card-stack-cull-btn:focus-visible\s*\{\s*outline:/);
  assert.match(gridSource, /\.badge-stack:focus-visible\s*\{/);
});
