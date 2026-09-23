import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";

test("VirtualGrid source code populates file_ids with all selected file IDs on multi-selection drag", async () => {
  const gridSource = await fs.readFile(
    new URL("../src/components/VirtualGrid.vue", import.meta.url),
    "utf8"
  );

  // Checks that onDragStart does not discard other selected IDs
  assert.match(
    gridSource,
    /const isMulti = Boolean\(props\.selectedFilePaths && props\.selectedFilePaths\.has\(file\.path\)\);/
  );
  assert.match(
    gridSource,
    /for\s*\(const f of props\.files\)\s*\{\s*if\s*\(f\.id != null && pathSet\.has\(f\.path\)/
  );
  assert.match(
    gridSource,
    /file_ids:\s*selectedIds/
  );
});

test("Sidebar source code handles both internal multi-select drops and external OS file drops on albums", async () => {
  const sidebarSource = await fs.readFile(
    new URL("../src/components/Sidebar.vue", import.meta.url),
    "utf8"
  );

  // Emits declaration contains importExternalFilesToAlbum
  assert.match(
    sidebarSource,
    /importExternalFilesToAlbum:\s*\[payload:\s*\{\s*filePaths:\s*string\[\];\s*albumId:\s*number\s*\}\]/
  );

  // onDropOnAlbum handles dataTransfer.files
  assert.match(
    sidebarSource,
    /const droppedFiles = e\.dataTransfer\?\.files;/
  );
  assert.match(
    sidebarSource,
    /emit\("importExternalFilesToAlbum",\s*\{\s*filePaths,\s*albumId:\s*album\.id,\s*\}\);/
  );
});

test("multi-select drag payload algorithm preserves all selected file IDs in order", () => {
  function computeDragPayload(props, file) {
    const isMulti = Boolean(props.selectedFilePaths && props.selectedFilePaths.has(file.path));
    const selectedPaths = isMulti
      ? Array.from(props.selectedFilePaths)
      : [file.path];

    let selectedIds = [];
    if (isMulti) {
      const pathSet = props.selectedFilePaths;
      const seen = new Set();
      for (const f of props.files) {
        if (f.id != null && pathSet.has(f.path) && !seen.has(f.id)) {
          seen.add(f.id);
          selectedIds.push(f.id);
        }
      }
      if (selectedIds.length === 0 && file.id != null) {
        selectedIds = [file.id];
      }
    } else if (file.id != null) {
      selectedIds = [file.id];
    }

    return {
      file_paths: selectedPaths,
      file_ids: selectedIds,
    };
  }

  const files = [
    { id: 101, path: "/path/img1.png" },
    { id: 102, path: "/path/img2.png" },
    { id: 103, path: "/path/img3.png" },
    { id: 104, path: "/path/img4.png" },
    { id: 105, path: "/path/img5.png" },
  ];

  // Case 1: Multi-selection of 3 images, dragged by img3
  const selectedPaths = new Set(["/path/img1.png", "/path/img3.png", "/path/img5.png"]);
  const payloadMulti = computeDragPayload(
    { files, selectedFilePaths: selectedPaths },
    files[2] // img3
  );

  assert.deepEqual(payloadMulti.file_ids, [101, 103, 105], "All selected IDs must be included");
  assert.deepEqual(payloadMulti.file_paths, ["/path/img1.png", "/path/img3.png", "/path/img5.png"]);

  // Case 2: Dragging a non-selected file while other files are selected
  const payloadSingle = computeDragPayload(
    { files, selectedFilePaths: selectedPaths },
    files[1] // img2 (not selected)
  );

  assert.deepEqual(payloadSingle.file_ids, [102], "Only dragged file ID when not in selection");
  assert.deepEqual(payloadSingle.file_paths, ["/path/img2.png"]);

  // Case 3: No selection at all
  const payloadNoSel = computeDragPayload(
    { files, selectedFilePaths: null },
    files[3] // img4
  );

  assert.deepEqual(payloadNoSel.file_ids, [104]);
  assert.deepEqual(payloadNoSel.file_paths, ["/path/img4.png"]);
});

test("Sidebar onDropOnAlbum algorithm parses internal JSON payload and emits addFilesToAlbum", () => {
  const emitted = [];
  function emit(name, payload) {
    emitted.push({ name, payload });
  }

  function handleDrop(e, album) {
    const data = e.dataTransfer?.getData("application/json");
    if (data) {
      try {
        const payload = JSON.parse(data);
        if (payload.file_ids && payload.file_ids.length > 0) {
          emit("addFilesToAlbum", {
            fileIds: payload.file_ids,
            albumId: album.id,
          });
          return;
        }
      } catch {}
    }

    const droppedFiles = e.dataTransfer?.files;
    if (droppedFiles && droppedFiles.length > 0) {
      const filePaths = [];
      for (let i = 0; i < droppedFiles.length; i++) {
        const f = droppedFiles[i];
        if (f.path) filePaths.push(f.path);
      }
      if (filePaths.length > 0) {
        emit("importExternalFilesToAlbum", {
          filePaths,
          albumId: album.id,
        });
        return;
      }
    }
  }

  const album = { id: 7, name: "Test Album" };
  const mockEventInternal = {
    preventDefault() {},
    dataTransfer: {
      getData(type) {
        if (type === "application/json") {
          return JSON.stringify({ file_paths: ["/a", "/b"], file_ids: [10, 20] });
        }
        return "";
      },
      files: [],
    },
  };

  handleDrop(mockEventInternal, album);
  assert.equal(emitted.length, 1);
  assert.equal(emitted[0].name, "addFilesToAlbum");
  assert.deepEqual(emitted[0].payload, { fileIds: [10, 20], albumId: 7 });
});

test("Sidebar onDropOnAlbum algorithm detects external OS file drop and emits importExternalFilesToAlbum", () => {
  const emitted = [];
  function emit(name, payload) {
    emitted.push({ name, payload });
  }

  function handleDrop(e, album) {
    const data = e.dataTransfer?.getData("application/json");
    if (data) {
      try {
        const payload = JSON.parse(data);
        if (payload.file_ids && payload.file_ids.length > 0) {
          emit("addFilesToAlbum", {
            fileIds: payload.file_ids,
            albumId: album.id,
          });
          return;
        }
      } catch {}
    }

    const droppedFiles = e.dataTransfer?.files;
    if (droppedFiles && droppedFiles.length > 0) {
      const filePaths = [];
      for (let i = 0; i < droppedFiles.length; i++) {
        const f = droppedFiles[i];
        if (f.path) filePaths.push(f.path);
      }
      if (filePaths.length > 0) {
        emit("importExternalFilesToAlbum", {
          filePaths,
          albumId: album.id,
        });
        return;
      }
    }
  }

  const album = { id: 42, name: "Managed Vault Album" };
  const mockEventExternal = {
    preventDefault() {},
    dataTransfer: {
      getData() {
        return "";
      },
      files: [
        { path: "C:\\Users\\Artist\\Downloads\\render_01.png" },
        { path: "C:\\Users\\Artist\\Downloads\\render_02.png" },
      ],
    },
  };

  handleDrop(mockEventExternal, album);
  assert.equal(emitted.length, 1);
  assert.equal(emitted[0].name, "importExternalFilesToAlbum");
  assert.deepEqual(emitted[0].payload, {
    filePaths: [
      "C:\\Users\\Artist\\Downloads\\render_01.png",
      "C:\\Users\\Artist\\Downloads\\render_02.png",
    ],
    albumId: 42,
  });
});
