import assert from "node:assert/strict";
import test from "node:test";

import {
  extractWorkflowJson,
  hasComfyWorkflow,
  hasPromptData,
} from "../src/utils/generation.ts";

test("extractWorkflowJson parses raw ComfyUI JSON chunk correctly", () => {
  const comfyPrompt = JSON.stringify({
    "3": { class_type: "KSampler", inputs: { seed: 123456 } },
  });
  const file = {
    id: 1,
    path: "/test.png",
    metadata: {
      format: "ComfyUi",
      raw: comfyPrompt,
      prompt: "masterpiece, 1girl",
    },
  };

  const extracted = extractWorkflowJson(file);
  assert.equal(extracted, comfyPrompt);
  assert.equal(hasComfyWorkflow(file), true);
  assert.equal(hasPromptData(file), true);
});

test("extractWorkflowJson fallback to parameters when raw is empty", () => {
  const comfyWorkflow = JSON.stringify({
    nodes: [{ id: 1, type: "KSampler" }],
  });
  const file = {
    id: 2,
    path: "/test2.png",
    metadata: {
      format: "Unknown",
      raw: null,
      parameters: comfyWorkflow,
      prompt: null,
    },
  };

  const extracted = extractWorkflowJson(file);
  assert.equal(extracted, comfyWorkflow);
  assert.equal(hasComfyWorkflow(file), true);
  assert.equal(hasPromptData(file), false);
});

test("hasPromptData detects standard A1111 prompts", () => {
  const file = {
    id: 3,
    path: "/a1111.png",
    metadata: {
      format: "Automatic1111",
      raw: null,
      prompt: "cyberpunk city, neon lighting",
      negative_prompt: "low quality, blurry",
      steps: 25,
      sampler: "DPM++ 2M Karras",
      cfg_scale: 7.5,
    },
  };

  assert.equal(hasPromptData(file), true);
  assert.equal(hasComfyWorkflow(file), false);
  assert.equal(extractWorkflowJson(file), null);
});

test("handles null file and empty metadata safely", () => {
  assert.equal(extractWorkflowJson(null), null);
  assert.equal(hasComfyWorkflow(null), false);
  assert.equal(hasPromptData(null), false);

  const emptyFile = { id: 4, path: "/empty.png", metadata: null };
  assert.equal(extractWorkflowJson(emptyFile), null);
  assert.equal(hasComfyWorkflow(emptyFile), false);
  assert.equal(hasPromptData(emptyFile), false);
});
