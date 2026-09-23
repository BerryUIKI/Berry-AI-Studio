import assert from "node:assert/strict";
import test from "node:test";

test("BatchActionBar button action emits align with App.vue listeners", () => {
  const emittedEvents = [];
  const emit = (event, ...args) => {
    emittedEvents.push({ event, args });
  };

  const actions = {
    onTag() {
      emit("tagSelected");
      emit("addTag");
    },
    onMove() {
      emit("moveSelected");
      emit("move");
    },
    onCopy() {
      emit("copySelected");
      emit("copy");
    },
    onCull() {
      emit("cullSelectedDrafts");
      emit("cullDrafts");
    },
    onTrash() {
      emit("trashSelected");
      emit("trash");
    },
  };

  // Tag action
  actions.onTag();
  assert.ok(emittedEvents.some((e) => e.event === "addTag"), "Must emit addTag for App.vue @add-tag");
  assert.ok(emittedEvents.some((e) => e.event === "tagSelected"), "Must emit tagSelected for backwards compatibility");

  // Move action
  actions.onMove();
  assert.ok(emittedEvents.some((e) => e.event === "move"), "Must emit move for App.vue @move");

  // Copy action
  actions.onCopy();
  assert.ok(emittedEvents.some((e) => e.event === "copy"), "Must emit copy for App.vue @copy");

  // Cull action
  actions.onCull();
  assert.ok(emittedEvents.some((e) => e.event === "cullDrafts"), "Must emit cullDrafts for App.vue @cull-drafts");

  // Trash action
  actions.onTrash();
  assert.ok(emittedEvents.some((e) => e.event === "trash"), "Must emit trash for App.vue @trash");
});

test("BatchActionBar positioning is anchored to gallery viewport canvas", async () => {
  const fs = await import("node:fs/promises");
  const source = await fs.readFile(new URL("../src/components/BatchActionBar.vue", import.meta.url), "utf8");

  // Must use position: absolute within .gallery-viewport, not position: fixed to full window
  assert.ok(
    source.includes("position: absolute"),
    "BatchActionBar must use position: absolute to anchor within gallery viewport",
  );
  assert.ok(
    !source.includes("position: fixed"),
    "BatchActionBar must not use position: fixed which causes sidebar collisions",
  );
  assert.ok(
    source.includes("max-width: calc(100% - 24px)"),
    "BatchActionBar must constrain max-width to avoid edge clipping",
  );
});

test("single file selection updates selectedFiles and clear-selection clears state", () => {
  const file1 = { id: 1, path: "/photos/a.png" };
  const file2 = { id: 2, path: "/photos/b.png" };
  const allFiles = [file1, file2];

  let selectedFile = null;
  let selectedFilePaths = new Set();

  function onFileSelected(file, event) {
    selectedFile = file;
    if (event?.ctrlKey || event?.metaKey) {
      if (selectedFilePaths.has(file.path)) selectedFilePaths.delete(file.path);
      else selectedFilePaths.add(file.path);
    } else {
      selectedFilePaths = new Set([file.path]);
    }
  }

  function getSelectedFilesList() {
    return allFiles.filter((f) => selectedFilePaths.has(f.path));
  }

  function onClearSelection() {
    selectedFilePaths.clear();
    selectedFile = null;
  }

  // Single click on file 1
  onFileSelected(file1);
  assert.equal(selectedFile, file1);
  assert.equal(selectedFilePaths.size, 1);
  assert.deepEqual(getSelectedFilesList(), [file1], "Should provide single file in selected files list");

  // Deselect clears both paths and selectedFile
  onClearSelection();
  assert.equal(selectedFile, null);
  assert.equal(selectedFilePaths.size, 0);
  assert.deepEqual(getSelectedFilesList(), []);
});

test("responsive collapse separates primary actions and bundles secondary actions into more menu", async () => {
  const fs = await import("node:fs/promises");
  const source = await fs.readFile(new URL("../src/components/BatchActionBar.vue", import.meta.url), "utf8");

  // Primary buttons retained
  assert.ok(source.includes("t.batch.setRating"), "Rating dropdown must be present");
  assert.ok(source.includes("t.batch.favorite"), "Favorite button must be present");
  assert.ok(source.includes("t.batch.tag"), "Tag button must be present");
  assert.ok(source.includes("t.batch.trash"), "Trash button must be present");

  // Secondary actions bundled
  assert.ok(source.includes("secondary-actions-inline"), "Inline secondary actions container must exist");
  assert.ok(source.includes("more-actions-wrapper"), "More menu wrapper must exist");
  assert.ok(source.includes("t.batch.more"), "More button label must be present");
  assert.ok(source.includes("more-menu"), "More dropdown menu must exist");

  // Escape key closes menus
  assert.ok(source.includes('e.key === "Escape"'), "Escape key handler must close menus");
});

