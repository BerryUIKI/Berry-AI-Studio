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
