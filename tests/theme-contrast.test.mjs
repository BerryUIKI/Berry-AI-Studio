import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");

test("style.css declares light theme variables and readable select option styles", () => {
  const css = fs.readFileSync(path.join(rootDir, "src", "style.css"), "utf-8");

  // Check light theme variables
  assert.match(css, /:root\[data-theme="light"\]\s*\{[^}]*color-scheme:\s*light;/);
  assert.match(css, /--color-bg-primary:\s*#ffffff;/);
  assert.match(css, /--color-text-primary:\s*#111827;/);

  // Check system light theme media query
  assert.match(css, /@media\s*\(prefers-color-scheme:\s*light\)\s*\{[^}]*:root\[data-theme="system"\]/);

  // Check global select and option styling
  assert.match(css, /select\s*option\s*\{[^}]*color:\s*var\(--color-text-primary\);/);
  assert.match(css, /select\s*option\s*\{[^}]*background-color:\s*var\(--color-bg-primary\);/);

  // Check focus-visible outline for accessibility
  assert.match(css, /:focus-visible\s*\{[^}]*outline:\s*2px\s*solid\s*var\(--color-accent\);/);
});

test("SettingsModal.vue uses theme variables for select-input and url-input", () => {
  const vue = fs.readFileSync(path.join(rootDir, "src", "components", "SettingsModal.vue"), "utf-8");

  // .select-input should use var(--color-text-primary), not hardcoded #e2e8f0
  const selectMatch = vue.match(/\.select-input\s*\{([^}]+)\}/);
  assert.ok(selectMatch, ".select-input rule should exist");
  assert.match(selectMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(selectMatch[1], /color:\s*#e2e8f0/);

  // .url-input should also use var(--color-text-primary)
  const urlMatch = vue.match(/\.url-input\s*\{([^}]+)\}/);
  assert.ok(urlMatch, ".url-input rule should exist");
  assert.match(urlMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(urlMatch[1], /color:\s*#e2e8f0/);
});

test("SortBar.vue uses theme variables instead of hardcoded dark colors", () => {
  const vue = fs.readFileSync(path.join(rootDir, "src", "components", "SortBar.vue"), "utf-8");

  const sortSelectMatch = vue.match(/\.sort-select\s*\{([^}]+)\}/);
  assert.ok(sortSelectMatch, ".sort-select rule should exist");
  assert.match(sortSelectMatch[1], /background:\s*var\(--color-bg-secondary\)/);
  assert.match(sortSelectMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(sortSelectMatch[1], /background:\s*#202024/);
});

test("TitleBar.vue uses theme variables for brand title and window controls", () => {
  const vue = fs.readFileSync(path.join(rootDir, "src", "components", "TitleBar.vue"), "utf-8");

  const brandMatch = vue.match(/\.brand-title\s*\{([^}]+)\}/);
  assert.ok(brandMatch, ".brand-title rule should exist");
  assert.match(brandMatch[1], /color:\s*var\(--color-text-primary\)/);
  assert.doesNotMatch(brandMatch[1], /color:\s*#f1f5f9/);

  const controlMatch = vue.match(/\.control-btn:hover\s*\{([^}]+)\}/);
  assert.ok(controlMatch, ".control-btn:hover rule should exist");
  assert.match(controlMatch[1], /background:\s*var\(--color-bg-hover\)/);
});
