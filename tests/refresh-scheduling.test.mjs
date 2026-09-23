import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";

test("App startup renders initial gallery files without blocking on ancillary facets", async () => {
  const appSource = await fs.readFile(new URL("../src/App.vue", import.meta.url), "utf8");

  // Verify reloadFolders is awaited, followed immediately by loadFiles()
  assert.match(
    appSource,
    /await reloadFolders\(\);\s*const initialFilesPromise = loadFiles\(\);/s,
    "First gallery page must be initiated immediately after loading folders without awaiting facets",
  );

  // Verify ancillary hydration (counts, filters, albums & tags) is scheduled concurrently/in background
  assert.match(
    appSource,
    /void Promise\.all\(\[\s*refreshCounts\(\),\s*reloadFiltersMeta\(\),\s*loadAlbumsAndTags\(\),\s*\]\);/s,
    "Ancillary facets must not block the initial gallery files render",
  );
});

test("loadAlbumsAndTags and AlbumModal use aggregated get_album_counts instead of sequential N+1 queries", async () => {
  const appSource = await fs.readFile(new URL("../src/App.vue", import.meta.url), "utf8");
  const albumModalSource = await fs.readFile(new URL("../src/components/AlbumModal.vue", import.meta.url), "utf8");

  // App.vue loadAlbumsAndTags
  assert.match(
    appSource,
    /invoke<Record<number, number>>\("get_album_counts"\)/,
    "App.vue must use aggregated get_album_counts IPC command",
  );
  assert.doesNotMatch(
    appSource,
    /for\s*\([^)]*of\s*albums\.value\)\s*\{[^}]*count_album_files/s,
    "App.vue must not perform sequential count_album_files loops",
  );

  // AlbumModal.vue loadAlbums
  assert.match(
    albumModalSource,
    /invoke<Record<number, number>>\("get_album_counts"\)/,
    "AlbumModal.vue must use aggregated get_album_counts IPC command",
  );
  assert.doesNotMatch(
    albumModalSource,
    /for\s*\([^)]*of\s*albums\.value\)\s*\{[^}]*count_album_files/s,
    "AlbumModal.vue must not perform sequential count_album_files loops",
  );
});

test("scheduleLibraryRefresh and onFolderScanned are mutation-aware and avoid redundant reloads", async () => {
  const appSource = await fs.readFile(new URL("../src/App.vue", import.meta.url), "utf8");

  // scheduleLibraryRefresh uses pendingChangedFolders to determine target relevance
  assert.match(
    appSource,
    /const pendingChangedFolders = new Set<number>\(\);/,
    "Tracks changed folders to scope refresh events",
  );

  assert.match(
    appSource,
    /affectsActiveView =[\s\S]*?activeTarget\.value\.type === "all"[\s\S]*?changed\.has\(activeTarget\.value\.folder\.id\)/,
    "Only triggers loadFiles when active view is affected by changed folders",
  );

  // scheduleLibraryRefresh must NOT query albums/tags or models/samplers
  const refreshFnMatch = appSource.match(/function scheduleLibraryRefresh[\s\S]*?\n\}/);
  assert.ok(refreshFnMatch, "scheduleLibraryRefresh function found");
  assert.doesNotMatch(
    refreshFnMatch[0],
    /loadAlbumsAndTags/,
    "Filesystem watcher events must not query albums and tags",
  );
  assert.doesNotMatch(
    refreshFnMatch[0],
    /reloadFiltersMeta/,
    "Filesystem watcher events must not query filter models and samplers",
  );

  // onFolderScanned scopes file reloading to active target
  assert.match(
    appSource,
    /async function onFolderScanned\(folderId: number\)\s*\{[\s\S]*?activeTarget\.value\.folder\.id === folderId[\s\S]*?await loadFiles\(\);\s*\}/,
    "onFolderScanned must only reload files when current target matches scanned folder",
  );
});

test("Tauri backend registers get_album_counts command and performs asynchronous watcher folder setup", async () => {
  const libSource = await fs.readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const commandsSource = await fs.readFile(new URL("../src-tauri/src/commands.rs", import.meta.url), "utf8");

  // Commands
  assert.match(
    commandsSource,
    /pub fn get_album_counts/,
    "get_album_counts command defined in commands.rs",
  );
  assert.match(
    libSource,
    /commands::get_album_counts/,
    "get_album_counts registered in invoke_handler in lib.rs",
  );

  // Asynchronous folder registration for filesystem watcher
  assert.match(
    libSource,
    /std::thread::spawn\(move \|\|\s*\{[\s\S]*?state\.watcher\.lock\(\)[\s\S]*?watcher\.watch_folder\(&folder\)/,
    "Initial folder watching must run asynchronously without blocking app setup",
  );
});

test("simulates refresh scheduling scoping logic under single and multi-folder mutations", () => {
  function computeAffects(activeTarget, changed) {
    return (
      changed.size === 0 ||
      activeTarget.type === "all" ||
      activeTarget.type === "favorites" ||
      activeTarget.type === "nsfw" ||
      (activeTarget.type === "folder" && changed.has(activeTarget.folderId))
    );
  }

  // 1. Viewing all files -> always affected
  assert.equal(computeAffects({ type: "all" }, new Set([1])), true);
  assert.equal(computeAffects({ type: "all" }, new Set([2])), true);

  // 2. Viewing folder 1, mutation in folder 1 -> affected
  assert.equal(computeAffects({ type: "folder", folderId: 1 }, new Set([1])), true);

  // 3. Viewing folder 2, mutation in folder 1 -> NOT affected (preserves scroll & navigation)
  assert.equal(computeAffects({ type: "folder", folderId: 2 }, new Set([1])), false);

  // 4. Viewing folder 2, mutation in folders 1 and 2 -> affected
  assert.equal(computeAffects({ type: "folder", folderId: 2 }, new Set([1, 2])), true);

  // 5. Viewing album 5, mutation in folder 1 -> NOT affected
  assert.equal(computeAffects({ type: "album", albumId: 5 }, new Set([1])), false);

  // 6. General event with unknown folders (empty set) -> affected
  assert.equal(computeAffects({ type: "folder", folderId: 2 }, new Set()), true);
});
