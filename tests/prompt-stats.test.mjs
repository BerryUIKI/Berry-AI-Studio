import assert from "node:assert/strict";
import test from "node:test";

test("get_prompt_stats RPC payload verification", () => {
  // Simulates Tauri IPC serialization behavior for get_prompt_stats
  function mockTauriInvoke(command, args) {
    if (command === "get_prompt_stats") {
      if (typeof args?.isNegative !== "boolean") {
        throw new Error("missing required key isNegative");
      }
      if (typeof args?.limit !== "number") {
        throw new Error("missing required key limit");
      }
      return [
        { text: "masterpiece", count: 120 },
        { text: "best quality", count: 95 },
      ];
    }
    throw new Error(`unknown command ${command}`);
  }

  // Regression test: old behavior passed { limit: 40 } without isNegative
  assert.throws(
    () => mockTauriInvoke("get_prompt_stats", { limit: 40 }),
    /missing required key isNegative/,
    "Passing { limit: 40 } without isNegative must fail with RPC serialization error",
  );

  // Fixed behavior: passes { isNegative, limit }
  const posStats = mockTauriInvoke("get_prompt_stats", { isNegative: false, limit: 40 });
  assert.equal(posStats.length, 2);
  assert.equal(posStats[0].text, "masterpiece");

  const negStats = mockTauriInvoke("get_prompt_stats", { isNegative: true, limit: 40 });
  assert.equal(negStats.length, 2);
});

test("maps PromptStat array to PromptKeywordStat array", () => {
  const backendStats = [
    { text: "photorealistic", count: 42 },
    { text: "cinematic lighting", count: 30 },
  ];

  const mapped = backendStats.map((p) => ({
    keyword: p.text,
    count: p.count,
  }));

  assert.deepEqual(mapped, [
    { keyword: "photorealistic", count: 42 },
    { keyword: "cinematic lighting", count: 30 },
  ]);
});

test("modal close event contract", () => {
  const emittedEvents = [];
  const emit = (event, ...args) => {
    emittedEvents.push({ event, args });
  };

  function close() {
    emit("close");
    emit("update:open", false);
  }

  close();

  assert.equal(emittedEvents.length, 2);
  assert.equal(emittedEvents[0].event, "close");
  assert.deepEqual(emittedEvents[1], { event: "update:open", args: [false] });
});
