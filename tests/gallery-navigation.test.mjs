import assert from "node:assert/strict";
import test from "node:test";
import { ref } from "vue";

import {
  useGalleryNavigation,
  clearGalleryNavigationAnchors,
  getGalleryNavigationAnchor,
  setGalleryNavigationAnchor,
  MAX_RESTORE_PAGES,
} from "../src/utils/gallery-navigation.ts";

function createMockElement(initialScrollTop = 0) {
  return {
    scrollTop: initialScrollTop,
  };
}

function createFile(id, name) {
  return {
    id,
    path: `/media/${name ?? `img_${id}`}.jpg`,
    container: "jpg",
    size_bytes: 1024,
    modified_at: 1700000000,
  };
}

// Ensure clean anchor map before each test
test.beforeEach(() => {
  clearGalleryNavigationAnchors();
});

test("saves anchor file id and row offset on user scroll", async () => {
  const element = ref(createMockElement(350));
  const keyRef = ref("folder-1");
  const filesRef = ref(Array.from({ length: 20 }, (_, i) => createFile(i + 1)));
  const revisionRef = ref(0);
  const loadingRef = ref(false);
  const hasMoreRef = ref(true);
  const loadingMoreRef = ref(false);

  // Suppose row height is 100px. At scrollTop = 350, firstVisible is index 3.
  // top(3) = 300, offset = 350 - 300 = 50px.
  const nav = useGalleryNavigation({
    element,
    key: () => keyRef.value,
    files: () => filesRef.value,
    revision: () => revisionRef.value,
    loading: () => loadingRef.value,
    hasMore: () => hasMoreRef.value,
    loadingMore: () => loadingMoreRef.value,
    top: (index) => index * 100,
    firstVisible: () => Math.floor(element.value.scrollTop / 100),
    loadMore: () => {},
    onRestore: () => {},
  });

  nav.save();

  const saved = getGalleryNavigationAnchor("folder-1");
  assert.ok(saved, "Anchor should be saved");
  assert.equal(saved.id, 4); // File at index 3 is id 4
  assert.equal(saved.offset, 50); // 350 - 300 = 50
  assert.equal(saved.index, 3);
});

test("restores anchor position with offset across different view modes", async () => {
  // Pre-set anchor from a previous Grid session: file id 5 at offset 20px
  setGalleryNavigationAnchor("folder-1", { id: 5, offset: 20, index: 4 });

  const element = ref(createMockElement(0));
  const keyRef = ref("folder-1");
  const filesRef = ref(Array.from({ length: 20 }, (_, i) => createFile(i + 1)));
  const revisionRef = ref(0);
  const loadingRef = ref(false);
  const hasMoreRef = ref(false);
  const loadingMoreRef = ref(false);

  let restoredTop = null;

  // Now mounting Table view: ROW_HEIGHT = 46px
  useGalleryNavigation({
    element,
    key: () => keyRef.value,
    files: () => filesRef.value,
    revision: () => revisionRef.value,
    loading: () => loadingRef.value,
    hasMore: () => hasMoreRef.value,
    loadingMore: () => loadingMoreRef.value,
    top: (index) => index * 46,
    itemHeight: () => 46,
    firstVisible: () => Math.floor(element.value.scrollTop / 46),
    loadMore: () => {},
    onRestore: (top) => {
      restoredTop = top;
    },
  });

  // Allow requestAnimationFrame / microtasks to settle
  await new Promise((resolve) => setTimeout(resolve, 30));

  // File id 5 is at index 4 in filesRef.
  // In Table mode, top(4) = 4 * 46 = 184. With offset 20, target is 184 + 20 = 204.
  assert.equal(restoredTop, 204);
  assert.equal(element.value.scrollTop, 204);
});

test("restores anchor beyond first page by requesting loadMore", async () => {
  // Anchor points to file id 25, which is not in the first page (only 10 items)
  setGalleryNavigationAnchor("folder-1", { id: 25, offset: 15, index: 24 });

  const element = ref(createMockElement(0));
  const keyRef = ref("folder-1");
  // Initial page 1: 10 items (ids 1..10)
  const filesRef = ref(Array.from({ length: 10 }, (_, i) => createFile(i + 1)));
  const revisionRef = ref(0);
  const loadingRef = ref(false);
  const hasMoreRef = ref(true);
  const loadingMoreRef = ref(false);

  let loadMoreCalls = 0;
  let restoredTop = null;

  useGalleryNavigation({
    element,
    key: () => keyRef.value,
    files: () => filesRef.value,
    revision: () => revisionRef.value,
    loading: () => loadingRef.value,
    hasMore: () => hasMoreRef.value,
    loadingMore: () => loadingMoreRef.value,
    top: (index) => index * 50,
    firstVisible: () => Math.floor(element.value.scrollTop / 50),
    loadMore: () => {
      loadMoreCalls++;
    },
    onRestore: (top) => {
      restoredTop = top;
    },
  });

  await new Promise((resolve) => setTimeout(resolve, 30));
  assert.equal(loadMoreCalls, 1, "Should request next page when anchor not found in page 1");
  assert.equal(restoredTop, null, "Should not restore until anchor is found");

  // Page 2 arrives: now 20 items (ids 1..20), still not file 25
  filesRef.value = Array.from({ length: 20 }, (_, i) => createFile(i + 1));
  revisionRef.value++;

  await new Promise((resolve) => setTimeout(resolve, 30));
  assert.equal(loadMoreCalls, 2, "Should request another page");

  // Page 3 arrives: now 30 items (ids 1..30), file 25 is at index 24!
  filesRef.value = Array.from({ length: 30 }, (_, i) => createFile(i + 1));
  revisionRef.value++;

  await new Promise((resolve) => setTimeout(resolve, 30));

  // Found file 25 at index 24! top(24) = 24 * 50 = 1200 + offset 15 = 1215.
  assert.equal(restoredTop, 1215);
  assert.equal(element.value.scrollTop, 1215);
});

test("handles missing anchor without infinite paging", async () => {
  // Anchor points to deleted file id 9999
  setGalleryNavigationAnchor("folder-1", { id: 9999, offset: 0, index: 100 });

  const element = ref(createMockElement(500));
  const keyRef = ref("folder-1");
  const filesRef = ref(Array.from({ length: 10 }, (_, i) => createFile(i + 1)));
  const revisionRef = ref(0);
  const loadingRef = ref(false);
  const hasMoreRef = ref(true);
  const loadingMoreRef = ref(false);

  let loadMoreCalls = 0;
  let restoredTop = null;

  useGalleryNavigation({
    element,
    key: () => keyRef.value,
    files: () => filesRef.value,
    revision: () => revisionRef.value,
    loading: () => loadingRef.value,
    hasMore: () => hasMoreRef.value,
    loadingMore: () => loadingMoreRef.value,
    top: (index) => index * 50,
    firstVisible: () => Math.floor(element.value.scrollTop / 50),
    loadMore: () => {
      loadMoreCalls++;
      // Simulate subsequent pages arriving without the deleted file
      filesRef.value = Array.from({ length: filesRef.value.length + 10 }, (_, i) =>
        createFile(i + 1),
      );
      revisionRef.value++;
    },
    onRestore: (top) => {
      restoredTop = top;
    },
    maxRestorePages: 3, // Bounded budget of 3 pages for this test
  });

  // Give enough time for bounded page requests to execute
  await new Promise((resolve) => setTimeout(resolve, 150));

  // Should stop after maxRestorePages (3), not page indefinitely!
  assert.equal(loadMoreCalls, 3, "Must not exceed max restore page budget");
  assert.equal(restoredTop, 0, "Should fall back to top (0) when anchor is missing");
  assert.equal(element.value.scrollTop, 0);
  assert.equal(getGalleryNavigationAnchor("folder-1"), undefined, "Stale anchor should be purged");
});

test("stops immediately when hasMore is false for a missing anchor", async () => {
  // Anchor points to deleted file id 9999, but folder only has 5 files and hasMore is false
  setGalleryNavigationAnchor("folder-1", { id: 9999, offset: 10 });

  const element = ref(createMockElement(100));
  const keyRef = ref("folder-1");
  const filesRef = ref(Array.from({ length: 5 }, (_, i) => createFile(i + 1)));
  const revisionRef = ref(0);
  const loadingRef = ref(false);
  const hasMoreRef = ref(false); // No more files in folder
  const loadingMoreRef = ref(false);

  let loadMoreCalls = 0;
  let restoredTop = null;

  useGalleryNavigation({
    element,
    key: () => keyRef.value,
    files: () => filesRef.value,
    revision: () => revisionRef.value,
    loading: () => loadingRef.value,
    hasMore: () => hasMoreRef.value,
    loadingMore: () => loadingMoreRef.value,
    top: (index) => index * 50,
    firstVisible: () => Math.floor(element.value.scrollTop / 50),
    loadMore: () => {
      loadMoreCalls++;
    },
    onRestore: (top) => {
      restoredTop = top;
    },
  });

  await new Promise((resolve) => setTimeout(resolve, 30));

  assert.equal(loadMoreCalls, 0, "Must not call loadMore when hasMore is false");
  assert.equal(restoredTop, 0, "Should fall back to top (0)");
  assert.equal(element.value.scrollTop, 0);
  assert.equal(getGalleryNavigationAnchor("folder-1"), undefined);
});

test("saves previous context on rapid key switch", async () => {
  const element = ref(createMockElement(250));
  const keyRef = ref("folder-A");
  const filesRef = ref(Array.from({ length: 10 }, (_, i) => createFile(i + 1, `A_${i + 1}`)));
  const revisionRef = ref(0);
  const loadingRef = ref(false);
  const hasMoreRef = ref(false);
  const loadingMoreRef = ref(false);

  useGalleryNavigation({
    element,
    key: () => keyRef.value,
    files: () => filesRef.value,
    revision: () => revisionRef.value,
    loading: () => loadingRef.value,
    hasMore: () => hasMoreRef.value,
    loadingMore: () => loadingMoreRef.value,
    top: (index) => index * 100,
    firstVisible: () => Math.floor(element.value.scrollTop / 100),
    loadMore: () => {},
    onRestore: () => {},
  });

  await new Promise((resolve) => setTimeout(resolve, 20));

  // User was at scrollTop = 250 in folder-A. Now switches to folder-B:
  keyRef.value = "folder-B";
  filesRef.value = Array.from({ length: 10 }, (_, i) => createFile(i + 100, `B_${i + 1}`));
  element.value.scrollTop = 0;
  revisionRef.value++;

  await new Promise((resolve) => setTimeout(resolve, 30));

  // Folder A anchor should have been saved automatically when key switched
  const anchorA = getGalleryNavigationAnchor("folder-A");
  assert.ok(anchorA, "folder-A anchor should be saved upon switching away");
  assert.equal(anchorA.id, 3); // index 2 -> file id 3
  assert.equal(anchorA.offset, 50); // 250 - 200 = 50
});

test("clamps offset to itemHeight when switching from tall cards to short table rows", async () => {
  // Pre-set anchor from tall grid cards (height = 300px), with offset 150px into item 2
  setGalleryNavigationAnchor("folder-1", { id: 2, offset: 150, index: 1 });

  const element = ref(createMockElement(0));
  const keyRef = ref("folder-1");
  const filesRef = ref(Array.from({ length: 10 }, (_, i) => createFile(i + 1)));
  const revisionRef = ref(0);
  const loadingRef = ref(false);
  const hasMoreRef = ref(false);
  const loadingMoreRef = ref(false);

  let restoredTop = null;

  // Mount in Table view: itemHeight = 46px
  useGalleryNavigation({
    element,
    key: () => keyRef.value,
    files: () => filesRef.value,
    revision: () => revisionRef.value,
    loading: () => loadingRef.value,
    hasMore: () => hasMoreRef.value,
    loadingMore: () => loadingMoreRef.value,
    top: (index) => index * 46,
    itemHeight: () => 46,
    firstVisible: () => Math.floor(element.value.scrollTop / 46),
    loadMore: () => {},
    onRestore: (top) => {
      restoredTop = top;
    },
  });

  await new Promise((resolve) => setTimeout(resolve, 30));

  // Item 2 is at index 1. top(1) = 46.
  // Offset 150 clamped to itemHeight - 1 = 45.
  // targetTop = 46 + 45 = 91.
  assert.equal(restoredTop, 91);
  assert.equal(element.value.scrollTop, 91);
});

test("empty gallery message and recovery selection state transitions", () => {
  const t = {
    review: {
      emptyLibrary: "Add a folder to start your library.",
      emptyFolder: "This folder is empty.",
      noMatches: "No matching images",
      clearFilters: "Clear filters",
      retry: "Retry",
    },
  };

  function computeEmptyMessage({ error, foldersCount, searchQuery, activeTargetType, activeFilterCount }) {
    if (error) return error;
    if (foldersCount === 0) return t.review.emptyLibrary;
    if (searchQuery.trim() || activeTargetType !== "folder" || activeFilterCount > 0) {
      return t.review.noMatches;
    }
    return t.review.emptyFolder;
  }

  function computeEmptyAction({ error, foldersCount, searchQuery, activeFilterCount }) {
    if (error) return t.review.retry;
    if (foldersCount === 0) return t.review.retry;
    if (searchQuery.trim() || activeFilterCount > 0) {
      return t.review.clearFilters;
    }
    return t.review.retry;
  }

  // Case 1: Empty library
  assert.equal(
    computeEmptyMessage({ error: "", foldersCount: 0, searchQuery: "", activeTargetType: "folder", activeFilterCount: 0 }),
    "Add a folder to start your library.",
  );
  assert.equal(
    computeEmptyAction({ error: "", foldersCount: 0, searchQuery: "", activeFilterCount: 0 }),
    "Retry",
  );

  // Case 2: Empty folder (folder exists, no filters, but 0 files)
  assert.equal(
    computeEmptyMessage({ error: "", foldersCount: 1, searchQuery: "", activeTargetType: "folder", activeFilterCount: 0 }),
    "This folder is empty.",
  );
  assert.equal(
    computeEmptyAction({ error: "", foldersCount: 1, searchQuery: "", activeFilterCount: 0 }),
    "Retry",
  );

  // Case 3: Search with no matches
  assert.equal(
    computeEmptyMessage({ error: "", foldersCount: 1, searchQuery: "cat", activeTargetType: "folder", activeFilterCount: 0 }),
    "No matching images",
  );
  assert.equal(
    computeEmptyAction({ error: "", foldersCount: 1, searchQuery: "cat", activeFilterCount: 0 }),
    "Clear filters",
  );

  // Case 4: Query error
  assert.equal(
    computeEmptyMessage({ error: "Failed to read database", foldersCount: 1, searchQuery: "", activeTargetType: "folder", activeFilterCount: 0 }),
    "Failed to read database",
  );
  assert.equal(
    computeEmptyAction({ error: "Failed to read database", foldersCount: 1, searchQuery: "", activeFilterCount: 0 }),
    "Retry",
  );

  // Recovery clears selection without stale state
  const selectedFile = { value: createFile(1) };
  const selectedFilePaths = { value: new Set(["/media/img_1.jpg"]) };

  function recoverGallery() {
    selectedFile.value = null;
    selectedFilePaths.value.clear();
  }

  recoverGallery();
  assert.equal(selectedFile.value, null, "selectedFile should be cleared on recover");
  assert.equal(selectedFilePaths.value.size, 0, "selectedFilePaths should be cleared on recover");
});

