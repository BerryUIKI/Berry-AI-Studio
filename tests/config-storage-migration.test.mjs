import test from 'node:test';
import assert from 'node:assert/strict';

// Mock localStorage for Node test environment
class MockLocalStorage {
  constructor() {
    this.store = new Map();
  }
  getItem(key) {
    return this.store.has(key) ? this.store.get(key) : null;
  }
  setItem(key, value) {
    this.store.set(key, String(value));
  }
  removeItem(key) {
    this.store.delete(key);
  }
  clear() {
    this.store.clear();
  }
}

globalThis.localStorage = new MockLocalStorage();

function getStorageItem(keySuffix) {
  try {
    const omeraKey = `omera_${keySuffix}`;
    const value = localStorage.getItem(omeraKey);
    if (value !== null) {
      return value;
    }
    const legacyKey = `berry_${keySuffix}`;
    const legacyValue = localStorage.getItem(legacyKey);
    if (legacyValue !== null) {
      localStorage.setItem(omeraKey, legacyValue);
      return legacyValue;
    }
    return null;
  } catch {
    return null;
  }
}

function setStorageItem(keySuffix, value) {
  try {
    localStorage.setItem(`omera_${keySuffix}`, value);
  } catch {
    // Ignore
  }
}

test('getStorageItem returns null when no key exists', () => {
  localStorage.clear();
  assert.equal(getStorageItem('locale'), null);
});

test('getStorageItem reads legacy berry_* key and migrates forward to omera_*', () => {
  localStorage.clear();
  localStorage.setItem('berry_locale', 'zh-CN');
  localStorage.setItem('berry_theme', 'midnight');

  assert.equal(getStorageItem('locale'), 'zh-CN');
  assert.equal(localStorage.getItem('omera_locale'), 'zh-CN');

  assert.equal(getStorageItem('theme'), 'midnight');
  assert.equal(localStorage.getItem('omera_theme'), 'midnight');
});

test('getStorageItem prioritizes omera_* over legacy berry_* even when false, 0 or empty', () => {
  localStorage.clear();
  localStorage.setItem('berry_similarity_limit', '100');
  localStorage.setItem('omera_similarity_limit', '0');

  assert.equal(getStorageItem('similarity_limit'), '0');

  localStorage.setItem('berry_blur_nsfw', 'true');
  localStorage.setItem('omera_blur_nsfw', 'false');

  assert.equal(getStorageItem('blur_nsfw'), 'false');

  localStorage.setItem('berry_comfyui_url', 'http://127.0.0.1:8188');
  localStorage.setItem('omera_comfyui_url', '');

  assert.equal(getStorageItem('comfyui_url'), '');
});

test('setStorageItem writes to omera_* without overwriting berry_*', () => {
  localStorage.clear();
  localStorage.setItem('berry_default_view', 'masonry');

  setStorageItem('default_view', 'grid');

  assert.equal(localStorage.getItem('omera_default_view'), 'grid');
  assert.equal(localStorage.getItem('berry_default_view'), 'masonry');
  assert.equal(getStorageItem('default_view'), 'grid');
});
