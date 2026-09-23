import assert from "node:assert/strict";
import test from "node:test";

// Helpers representing the navigation and criteria logic in App.vue & Sidebar.vue
function isTargetActive(activeTarget, target) {
  if (activeTarget.type !== target.type) return false;
  if (target.type === "folder" && activeTarget.type === "folder") {
    if (activeTarget.folder.id !== target.folder.id) return false;
    const activeSub = activeTarget.subfolderPath || "";
    const targetSub = target.subfolderPath || "";
    return activeSub === targetSub;
  }
  if (target.type === "album" && activeTarget.type === "album") {
    return activeTarget.album.id === target.album.id;
  }
  if (target.type === "tag" && activeTarget.type === "tag") {
    return activeTarget.tag.id === target.tag.id;
  }
  return true;
}

function buildFolderCriteria(activeTarget, sortField = "created_at", sortDirection = "desc") {
  const criteria = {
    sort: sortField,
    direction: sortDirection,
    limit: 50,
    offset: 0,
  };
  if (activeTarget.type === "folder") {
    criteria.folder_id = activeTarget.folder.id;
    if (activeTarget.subfolderPath) {
      criteria.folder_path = activeTarget.subfolderPath;
    }
    if (activeTarget.recursive !== undefined) {
      criteria.recursive = activeTarget.recursive;
    }
  }
  return criteria;
}

function computeTargetTitle(activeTarget) {
  if (activeTarget.type === "folder") {
    const rootName = activeTarget.folder.path.split(/[\\/]/).pop() || activeTarget.folder.path;
    if (activeTarget.subfolderPath) {
      const subName = activeTarget.subfolderPath.split(/[\\/]/).pop() || activeTarget.subfolderPath;
      return `📁 ${rootName} / ${subName}`;
    }
    return rootName;
  }
  return "All Images";
}

function computeGalleryContextKey(activeTarget) {
  return JSON.stringify({
    target:
      activeTarget.type === "folder"
        ? ["folder", activeTarget.folder.id, activeTarget.subfolderPath, activeTarget.recursive]
        : activeTarget.type,
  });
}

test("folder target matching distinguishes root from subfolder", () => {
  const rootTarget = {
    type: "folder",
    folder: { id: 1, path: "/vault/photos" },
  };

  const subTarget = {
    type: "folder",
    folder: { id: 1, path: "/vault/photos" },
    subfolderPath: "/vault/photos/nature",
    recursive: false,
  };

  const otherSubTarget = {
    type: "folder",
    folder: { id: 1, path: "/vault/photos" },
    subfolderPath: "/vault/photos/portraits",
    recursive: false,
  };

  // When active target is root
  assert.equal(isTargetActive(rootTarget, rootTarget), true);
  assert.equal(isTargetActive(rootTarget, subTarget), false);
  assert.equal(isTargetActive(rootTarget, otherSubTarget), false);

  // When active target is subfolder
  assert.equal(isTargetActive(subTarget, rootTarget), false);
  assert.equal(isTargetActive(subTarget, subTarget), true);
  assert.equal(isTargetActive(subTarget, otherSubTarget), false);
});

test("criteria builder includes folder_path and recursive flag", () => {
  const rootTarget = {
    type: "folder",
    folder: { id: 1, path: "/vault/photos" },
    recursive: true,
  };

  const rootCriteria = buildFolderCriteria(rootTarget);
  assert.equal(rootCriteria.folder_id, 1);
  assert.equal(rootCriteria.folder_path, undefined);
  assert.equal(rootCriteria.recursive, true);

  const subSingleTarget = {
    type: "folder",
    folder: { id: 1, path: "/vault/photos" },
    subfolderPath: "/vault/photos/nature",
    recursive: false,
  };

  const singleCriteria = buildFolderCriteria(subSingleTarget);
  assert.equal(singleCriteria.folder_id, 1);
  assert.equal(singleCriteria.folder_path, "/vault/photos/nature");
  assert.equal(singleCriteria.recursive, false);

  const subRecursiveTarget = {
    type: "folder",
    folder: { id: 1, path: "/vault/photos" },
    subfolderPath: "/vault/photos/nature",
    recursive: true,
  };

  const recursiveCriteria = buildFolderCriteria(subRecursiveTarget);
  assert.equal(recursiveCriteria.folder_id, 1);
  assert.equal(recursiveCriteria.folder_path, "/vault/photos/nature");
  assert.equal(recursiveCriteria.recursive, true);
});

test("targetTitle generates breadcrumb for subfolder", () => {
  const rootTarget = {
    type: "folder",
    folder: { id: 1, path: "C:\\Users\\Artist\\Pictures" },
  };
  assert.equal(computeTargetTitle(rootTarget), "Pictures");

  const subTarget = {
    type: "folder",
    folder: { id: 1, path: "C:\\Users\\Artist\\Pictures" },
    subfolderPath: "C:\\Users\\Artist\\Pictures\\Landscapes",
  };
  assert.equal(computeTargetTitle(subTarget), "📁 Pictures / Landscapes");
});

test("galleryContextKey differentiates recursive vs single-level and subfolders", () => {
  const rootRecursive = {
    type: "folder",
    folder: { id: 1, path: "/media" },
    recursive: true,
  };
  const rootSingle = {
    type: "folder",
    folder: { id: 1, path: "/media" },
    recursive: false,
  };
  const subTarget = {
    type: "folder",
    folder: { id: 1, path: "/media" },
    subfolderPath: "/media/2026",
    recursive: false,
  };

  const key1 = computeGalleryContextKey(rootRecursive);
  const key2 = computeGalleryContextKey(rootSingle);
  const key3 = computeGalleryContextKey(subTarget);

  assert.notEqual(key1, key2);
  assert.notEqual(key1, key3);
  assert.notEqual(key2, key3);
});

test("expand and collapse state transitions", () => {
  const expandedPaths = new Set();

  function toggle(path) {
    if (expandedPaths.has(path)) {
      expandedPaths.delete(path);
    } else {
      expandedPaths.add(path);
    }
  }

  toggle("/vault/root");
  assert.equal(expandedPaths.has("/vault/root"), true);

  toggle("/vault/root/sub1");
  assert.equal(expandedPaths.has("/vault/root"), true);
  assert.equal(expandedPaths.has("/vault/root/sub1"), true);

  toggle("/vault/root");
  assert.equal(expandedPaths.has("/vault/root"), false);
  // Child state preserved or manageable independently
  assert.equal(expandedPaths.has("/vault/root/sub1"), true);
});
