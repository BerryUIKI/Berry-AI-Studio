import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs/promises";

import { ClipIndexingController } from "../src/utils/clip-indexing.ts";

test("ClipIndexingController: Empty library completes cleanly with 0 failures", async () => {
  const calls = [];
  const mockInvoke = async (cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "index_clip_images_batch") {
      return {
        indexed_count: 0,
        remaining_count: 0,
        total_count: 0,
        failed_count: 0,
      };
    }
    throw new Error(`Unexpected command: ${cmd}`);
  };

  const controller = new ClipIndexingController(mockInvoke);
  await controller.startIndexing({ batchSize: 20 });

  assert.equal(controller.status, "completed");
  assert.equal(controller.failedCount, 0);
  assert.equal(controller.isIndexing, false);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].args.retryFailed, false);
});

test("ClipIndexingController: Successful indexing across multiple batches", async () => {
  const batches = [
    { indexed_count: 10, remaining_count: 15, total_count: 25, failed_count: 0 },
    { indexed_count: 10, remaining_count: 5, total_count: 25, failed_count: 0 },
    { indexed_count: 5, remaining_count: 0, total_count: 25, failed_count: 0 },
  ];
  let callIdx = 0;
  const mockInvoke = async (cmd, args) => {
    if (cmd === "index_clip_images_batch") {
      return batches[callIdx++];
    }
    throw new Error(`Unexpected command: ${cmd}`);
  };

  const batchResults = [];
  const controller = new ClipIndexingController(mockInvoke);
  await controller.startIndexing({
    batchSize: 10,
    onBatchIndexed: (res) => batchResults.push(res),
  });

  assert.equal(controller.status, "completed");
  assert.equal(controller.failedCount, 0);
  assert.equal(batchResults.length, 3);
  assert.equal(callIdx, 3);
});

test("ClipIndexingController: Failed first batch does NOT starve remaining images or exit prematurely", async () => {
  // Batch 1 has indexed_count = 0 (all 5 files failed to decode), but remaining_count = 10
  // Old bug: if (res.indexed_count === 0 || res.remaining_count === 0) break -> stopped here!
  // Fixed behavior: continues while remaining_count > 0
  const batches = [
    { indexed_count: 0, remaining_count: 10, total_count: 15, failed_count: 5 },
    { indexed_count: 10, remaining_count: 0, total_count: 15, failed_count: 5 },
  ];
  let callIdx = 0;
  const mockInvoke = async (cmd, args) => {
    if (cmd === "index_clip_images_batch") {
      return batches[callIdx++];
    }
    throw new Error(`Unexpected command: ${cmd}`);
  };

  const controller = new ClipIndexingController(mockInvoke);
  await controller.startIndexing({ batchSize: 10 });

  assert.equal(callIdx, 2, "Must proceed to batch 2 despite batch 1 having 0 indexed images");
  assert.equal(controller.status, "completed");
  assert.equal(controller.failedCount, 5, "Failure count must remain visible when remaining reaches 0");
});

test("ClipIndexingController: All-failed scenario terminates properly with failure visibility", async () => {
  const batches = [
    { indexed_count: 0, remaining_count: 5, total_count: 10, failed_count: 5 },
    { indexed_count: 0, remaining_count: 0, total_count: 10, failed_count: 10 },
  ];
  let callIdx = 0;
  const mockInvoke = async (cmd, args) => {
    if (cmd === "index_clip_images_batch") {
      return batches[callIdx++];
    }
    throw new Error(`Unexpected command: ${cmd}`);
  };

  const controller = new ClipIndexingController(mockInvoke);
  await controller.startIndexing({ batchSize: 5 });

  assert.equal(callIdx, 2);
  assert.equal(controller.status, "completed");
  assert.equal(controller.failedCount, 10);
});

test("ClipIndexingController: Cancellation separates cancel from success", async () => {
  let cancelCalled = false;
  const mockInvoke = async (cmd, args) => {
    if (cmd === "index_clip_images_batch") {
      // Trigger stop indexing inside first batch
      await controller.stopIndexing();
      return { indexed_count: 5, remaining_count: 15, total_count: 20, failed_count: 0 };
    }
    if (cmd === "cancel_clip_indexing") {
      cancelCalled = true;
      return null;
    }
    throw new Error(`Unexpected command: ${cmd}`);
  };

  const controller = new ClipIndexingController(mockInvoke);
  await controller.startIndexing({ batchSize: 5 });

  assert.equal(cancelCalled, true, "cancel_clip_indexing IPC must be invoked on stop");
  assert.equal(controller.status, "canceled", "Status must be canceled, never completed");
  assert.equal(controller.isIndexing, false);
});

test("ClipIndexingController: retryFailed is only true on first batch of a retry run and never continuously", async () => {
  const retryFlags = [];
  const batches = [
    { indexed_count: 5, remaining_count: 10, total_count: 15, failed_count: 0 },
    { indexed_count: 5, remaining_count: 5, total_count: 15, failed_count: 0 },
    { indexed_count: 5, remaining_count: 0, total_count: 15, failed_count: 0 },
  ];
  let callIdx = 0;
  const mockInvoke = async (cmd, args) => {
    if (cmd === "index_clip_images_batch") {
      retryFlags.push(args.retryFailed);
      return batches[callIdx++];
    }
    throw new Error(`Unexpected command: ${cmd}`);
  };

  const controller = new ClipIndexingController(mockInvoke);
  await controller.startIndexing({ batchSize: 5, retryFailed: true });

  assert.deepEqual(
    retryFlags,
    [true, false, false],
    "retryFailed must only be true on the initial retry batch, never continuously reset",
  );
});

test("ClipIndexingController: Model change resets failures and stops active indexing", async () => {
  const mockInvoke = async (cmd) => {
    if (cmd === "index_clip_images_batch") {
      return { indexed_count: 5, remaining_count: 0, total_count: 15, failed_count: 5 };
    }
    if (cmd === "cancel_clip_indexing") {
      return null;
    }
    return null;
  };

  const controller = new ClipIndexingController(mockInvoke);
  await controller.startIndexing({ batchSize: 5 });
  assert.equal(controller.failedCount, 5);

  controller.reset(true);
  assert.equal(controller.status, "idle");
  assert.equal(controller.failedCount, 0, "Model change must clear previous model failure counts");
});

test("Semantic search and find_similar consumers guard against stale responses in App.vue", async () => {
  const appSource = await fs.readFile(new URL("../src/App.vue", import.meta.url), "utf8");

  // Semantic search catch block requestVersion check
  assert.match(
    appSource,
    /\} catch \(clipErr\) \{\s*if \(requestVersion !== libraryRequestVersion\) return;/s,
    "Stale semantic search catch block must not reset files or open modal",
  );

  // handleFindSimilar requestVersion check
  assert.match(
    appSource,
    /let similarityRequestVersion = 0;\s*async function handleFindSimilar\(file: ImageFile\) \{\s*if \(!file\.id\) return;\s*const requestVersion = \+\+similarityRequestVersion;/s,
    "handleFindSimilar must use monotonic similarityRequestVersion fencing",
  );

  assert.match(
    appSource,
    /function exitSimilaritySearch\(\) \{\s*similarityRequestVersion\+\+;/s,
    "exitSimilaritySearch must invalidate in-flight similarity requests",
  );

  // SimilarFileItem mapping
  assert.match(
    appSource,
    /semanticSearchFiles\.value = matches\.map\(\(m\) => \(\{\s*\.\.\.m\.file,\s*similarity_score: m\.score,\s*\}\)\);/s,
    "App.vue must unpack m.file from SimilarFileItem payload",
  );
});
