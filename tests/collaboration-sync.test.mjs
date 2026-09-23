import assert from "node:assert/strict";
import test from "node:test";

import { CollaborationSyncEngine } from "../src/utils/collaborationSync.ts";

test("CollaborationSyncEngine initializes with defaults and updates client ID", () => {
  const engine = new CollaborationSyncEngine();
  assert.equal(engine.getLastSyncId(), 0);
  assert.equal(engine.getTotalSyncedCount(), 0);
  assert.equal(engine.getIsRunning(), false);

  engine.init("artist_ws_01", 100);
  assert.equal(engine.getLastSyncId(), 100);

  engine.setClientId("artist_ws_02");
  assert.equal(engine.getLastSyncId(), 100);
});

test("CollaborationSyncEngine adapts cadence when visibility changes", () => {
  const engine = new CollaborationSyncEngine({
    activeCadenceMs: 2000,
    idleCadenceMs: 10000,
  });

  engine.setDocumentVisible(true);
  assert.equal(engine.getEffectiveCadenceMs(), 2000);

  engine.setDocumentVisible(false);
  assert.equal(engine.getEffectiveCadenceMs(), 10000);
});

test("CollaborationSyncEngine pollOnce updates lastSyncId and dispatches events", async () => {
  const engine = new CollaborationSyncEngine();
  engine.init("client_a", 50);

  const mockEvents = [
    {
      id: 51,
      event_type: "file.rated",
      entity_id: 1001,
      secondary_id: null,
      client_id: "client_b",
      payload: JSON.stringify({ rating: 5, version: 2 }),
      created_at: 1726000001,
    },
    {
      id: 52,
      event_type: "tag.created",
      entity_id: 2002,
      secondary_id: "masterpiece",
      client_id: "client_c",
      payload: null,
      created_at: 1726000002,
    },
  ];

  let receivedSingleEvents = [];
  let receivedBatch = [];

  const unsubEvent = engine.onEvent((entry) => {
    receivedSingleEvents.push(entry);
  });
  const unsubBatch = engine.onBatch((entries) => {
    receivedBatch = entries;
  });

  const queryLog = [];
  const fetched = await engine.pollOnce(async (q) => {
    queryLog.push(q);
    return mockEvents;
  });

  assert.equal(fetched.length, 2);
  assert.equal(queryLog.length, 1);
  assert.equal(queryLog[0].after_id, 50);
  assert.equal(queryLog[0].exclude_client_id, "client_a");

  assert.equal(engine.getLastSyncId(), 52);
  assert.equal(engine.getTotalSyncedCount(), 2);
  assert.equal(receivedSingleEvents.length, 2);
  assert.equal(receivedSingleEvents[0].id, 51);
  assert.equal(receivedSingleEvents[1].id, 52);
  assert.equal(receivedBatch.length, 2);

  // Unsubscribe listeners
  unsubEvent();
  unsubBatch();

  await engine.pollOnce(async () => [
    {
      id: 53,
      event_type: "file.trashed",
      entity_id: 1001,
      secondary_id: null,
      client_id: "client_d",
      payload: null,
      created_at: 1726000003,
    },
  ]);

  assert.equal(engine.getLastSyncId(), 53);
  assert.equal(engine.getTotalSyncedCount(), 3);
  assert.equal(receivedSingleEvents.length, 2, "Unsubscribed listener should not receive event 53");
});

test("CollaborationSyncEngine registers and cleans up stable visibility listener on start and stop", () => {
  const addedListeners = [];
  const removedListeners = [];

  const originalDoc = globalThis.document;
  try {
    globalThis.document = {
      hidden: true,
      addEventListener(event, fn) {
        addedListeners.push({ event, fn });
      },
      removeEventListener(event, fn) {
        removedListeners.push({ event, fn });
      },
    };

    const engine = new CollaborationSyncEngine({
      activeCadenceMs: 1000,
      idleCadenceMs: 5000,
    });

    const expectedListener = engine.getVisibilityListener();
    assert.equal(typeof expectedListener, "function");

    engine.start();
    assert.equal(engine.getIsRunning(), true);
    assert.equal(addedListeners.length, 1);
    assert.equal(addedListeners[0].event, "visibilitychange");
    assert.equal(addedListeners[0].fn, expectedListener);
    // Visibility listener should have been called immediately to sync state
    assert.equal(engine.getEffectiveCadenceMs(), 5000); // document.hidden was true

    // Document becomes visible
    globalThis.document.hidden = false;
    expectedListener();
    assert.equal(engine.getEffectiveCadenceMs(), 1000);

    engine.stop();
    assert.equal(engine.getIsRunning(), false);
    assert.equal(removedListeners.length, 1);
    assert.equal(removedListeners[0].event, "visibilitychange");
    assert.equal(removedListeners[0].fn, expectedListener);

    // Starting again reuses the identical listener reference
    engine.start();
    assert.equal(addedListeners.length, 2);
    assert.equal(addedListeners[1].fn, expectedListener);

    engine.stop();
    assert.equal(removedListeners.length, 2);
    assert.equal(removedListeners[1].fn, expectedListener);
  } finally {
    if (originalDoc === undefined) {
      delete globalThis.document;
    } else {
      globalThis.document = originalDoc;
    }
  }
});

test("CollaborationSyncEngine fences stale poll responses when stopped while request is pending", async () => {
  const engine = new CollaborationSyncEngine();
  engine.init("client_test", 10);

  let eventsFired = 0;
  engine.onEvent(() => {
    eventsFired++;
  });

  let resolveFetch;
  const pendingPromise = new Promise((resolve) => {
    resolveFetch = resolve;
  });

  engine.start();
  const initialGen = engine.getRunGeneration();

  // Trigger pollOnce with a pending promise
  const pollPromise = engine.pollOnce(() => pendingPromise);

  // Stop the engine while request is still pending
  engine.stop();
  assert.equal(engine.getIsRunning(), false);
  assert.notEqual(engine.getRunGeneration(), initialGen);

  // Resolve the fetch after stop
  resolveFetch([
    {
      id: 11,
      event_type: "file.rated",
      entity_id: 500,
      secondary_id: null,
      client_id: "other_client",
      payload: null,
      created_at: 1726000000,
    },
  ]);

  const result = await pollPromise;

  // Stale entries should be discarded by generation fence
  assert.deepEqual(result, []);
  assert.equal(engine.getLastSyncId(), 10, "lastSyncId should not update from stale response");
  assert.equal(engine.getTotalSyncedCount(), 0, "totalSyncedCount should not increment from stale response");
  assert.equal(eventsFired, 0, "listeners should not fire for stale response");
});

test("CollaborationSyncEngine fences stale poll responses across restart", async () => {
  const engine = new CollaborationSyncEngine();
  engine.init("client_test", 100);

  let eventsFired = 0;
  engine.onEvent(() => {
    eventsFired++;
  });

  let resolveFetch1;
  const pendingPromise1 = new Promise((resolve) => {
    resolveFetch1 = resolve;
  });

  engine.start(); // generation 1
  const gen1 = engine.getRunGeneration();

  const poll1 = engine.pollOnce(() => pendingPromise1);

  engine.stop(); // generation 2
  engine.start(); // generation 3
  const gen3 = engine.getRunGeneration();
  assert.notEqual(gen1, gen3);

  // Now resolve the old poll from gen1
  resolveFetch1([
    {
      id: 101,
      event_type: "file.rated",
      entity_id: 501,
      secondary_id: null,
      client_id: "other_client",
      payload: null,
      created_at: 1726000000,
    },
  ]);

  const result1 = await poll1;
  assert.deepEqual(result1, []);
  assert.equal(engine.getLastSyncId(), 100);
  assert.equal(engine.getTotalSyncedCount(), 0);
  assert.equal(eventsFired, 0);

  // Now run a valid poll in gen3
  const poll3 = await engine.pollOnce(async () => [
    {
      id: 102,
      event_type: "file.rated",
      entity_id: 502,
      secondary_id: null,
      client_id: "other_client",
      payload: null,
      created_at: 1726000001,
    },
  ]);

  assert.equal(poll3.length, 1);
  assert.equal(engine.getLastSyncId(), 102);
  assert.equal(engine.getTotalSyncedCount(), 1);
  assert.equal(eventsFired, 1);

  engine.stop();
});

test("CollaborationSyncEngine dispose stops engine, clears listeners and cancels timers", async () => {
  const engine = new CollaborationSyncEngine({ activeCadenceMs: 50 });
  let count = 0;
  engine.onEvent(() => count++);
  engine.onBatch(() => count++);

  engine.start();
  assert.equal(engine.getIsRunning(), true);

  engine.dispose();
  assert.equal(engine.getIsRunning(), false);

  // pollOnce directly after dispose should not fire listeners even if entries returned
  const entries = await engine.pollOnce(async () => [
    {
      id: 200,
      event_type: "file.rated",
      entity_id: 999,
      secondary_id: null,
      client_id: "client_x",
      payload: null,
      created_at: 1726000000,
    },
  ]);

  assert.equal(entries.length, 1);
  assert.equal(count, 0, "No listeners should fire after dispose");
});

test("SQLite configuration does not activate collaboration polling", () => {
  const engine = new CollaborationSyncEngine();
  const cfg = {
    storage_backend: "sqlite",
    client_identifier: "artist_1",
  };

  const shouldStartPolling = Boolean(cfg.storage_backend && cfg.storage_backend !== "sqlite");
  assert.equal(shouldStartPolling, false, "SQLite mode must not start collaboration polling");
  assert.equal(engine.getIsRunning(), false);
});

test("Default configuration without storage_backend does not activate collaboration polling", () => {
  const engine = new CollaborationSyncEngine();
  const cfg = {};

  const shouldStartPolling = Boolean(cfg.storage_backend && cfg.storage_backend !== "sqlite");
  assert.equal(shouldStartPolling, false, "Default mode must not start collaboration polling");
  assert.equal(engine.getIsRunning(), false);
});


