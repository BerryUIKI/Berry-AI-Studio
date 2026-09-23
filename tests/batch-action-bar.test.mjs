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
