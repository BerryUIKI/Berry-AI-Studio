<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import type { ImageFile, Tag } from "../types";
import {
  assetUrl,
  formatBytes,
  getFileName,
  isVideoContainer,
  normalizePath,
} from "../utils/image";

const props = defineProps<{
  file: ImageFile;
  files: ImageFile[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "navigate", file: ImageFile): void;
  (e: "rate", fileId: number, rating: number | null): void;
  (e: "updateFile", file: ImageFile): void;
  (e: "openTagModal", fileId: number): void;
  (e: "openAlbumModal", fileId: number): void;
}>();

const showInspector = ref(true);
const promptCopied = ref(false);
const negativePromptCopied = ref(false);
const showRaw = ref(false);
const ratingSaving = ref(false);
const fileTags = ref<Tag[]>([]);
const revealedNsfw = ref(false);

watch(
  () => props.file.path,
  () => {
    revealedNsfw.value = false;
    loadTags();
  },
  { immediate: true },
);

async function loadTags() {
  if (!props.file.id) {
    fileTags.value = [];
    return;
  }
  try {
    fileTags.value = await invoke<Tag[]>("get_file_tags", { fileId: props.file.id });
  } catch (err) {
    console.error("Failed to load tags:", err);
  }
}

async function toggleFavorite() {
  if (!props.file.id) return;
  const nextVal = !props.file.is_favorite;
  try {
    await invoke("set_file_favorite", { fileId: props.file.id, isFavorite: nextVal });
    props.file.is_favorite = nextVal;
    emit("updateFile", props.file);
  } catch (err) {
    console.error("Failed to toggle favorite:", err);
  }
}

async function toggleNsfw() {
  if (!props.file.id) return;
  const nextVal = !props.file.is_nsfw;
  try {
    await invoke("set_file_nsfw", { fileId: props.file.id, isNsfw: nextVal });
    props.file.is_nsfw = nextVal;
    emit("updateFile", props.file);
  } catch (err) {
    console.error("Failed to toggle nsfw:", err);
  }
}

async function removeTag(tagId: number) {
  if (!props.file.id) return;
  try {
    await invoke("untag_file", { fileId: props.file.id, tagId });
    await loadTags();
  } catch (err) {
    console.error("Failed to untag:", err);
  }
}

const currentIndex = computed(() =>
  props.files.findIndex((f) => f.path === props.file.path),
);

const hasPrev = computed(() => currentIndex.value > 0);
const hasNext = computed(() => currentIndex.value < props.files.length - 1);

function prev() {
  if (hasPrev.value) {
    emit("navigate", props.files[currentIndex.value - 1]);
  }
}

function next() {
  if (hasNext.value) {
    emit("navigate", props.files[currentIndex.value + 1]);
  }
}

function toggleInspector() {
  showInspector.value = !showInspector.value;
}

async function copyPrompt(text: string | null | undefined, type: "prompt" | "negative") {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    if (type === "prompt") {
      promptCopied.value = true;
      setTimeout(() => {
        promptCopied.value = false;
      }, 1500);
    } else {
      negativePromptCopied.value = true;
      setTimeout(() => {
        negativePromptCopied.value = false;
      }, 1500);
    }
  } catch (err) {
    console.error("Failed to copy text:", err);
  }
}

async function setRating(r: number) {
  if (!props.file.id || ratingSaving.value) return;
  const newRating = props.file.rating === r ? null : r;
  ratingSaving.value = true;
  try {
    await invoke("set_file_rating", {
      fileId: props.file.id,
      rating: newRating,
    });
    emit("rate", props.file.id, newRating);
  } catch (err) {
    console.error("Failed to save rating:", err);
  } finally {
    ratingSaving.value = false;
  }
}

function handleKeyDown(e: KeyboardEvent) {
  // Ignore when typing inside an input/textarea
  const tag = (document.activeElement?.tagName ?? "").toLowerCase();
  if (tag === "input" || tag === "textarea") return;

  if (e.key === "Escape") {
    e.preventDefault();
    emit("close");
  } else if (e.key === "ArrowLeft") {
    e.preventDefault();
    prev();
  } else if (e.key === "ArrowRight") {
    e.preventDefault();
    next();
  } else if (e.key === "i" || e.key === "I") {
    e.preventDefault();
    toggleInspector();
  }
}

async function revealInFinder() {
  try {
    await invoke("reveal_in_file_manager", { path: props.file.path });
  } catch (e) {
    console.error("Reveal failed:", e);
  }
}

async function trashCurrentFile() {
  if (!confirm(`Are you sure you want to move "${getFileName(props.file.path)}" to the trash?`)) {
    return;
  }
  try {
    await invoke("trash_files", { filePaths: [props.file.path] });
    emit("close");
  } catch (e) {
    console.error("Trash failed:", e);
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});

// Reset copy state when navigating files
watch(
  () => props.file.path,
  () => {
    promptCopied.value = false;
    negativePromptCopied.value = false;
  },
);
</script>

<template>
  <div class="preview-backdrop" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="preview-dialog">
      <!-- Top header bar -->
      <header class="dialog-header">
        <div class="header-left">
          <span class="file-name" :title="normalizePath(file.path)">
            {{ getFileName(file.path) }}
          </span>
          <span class="file-counter" v-if="files.length > 1">
            {{ currentIndex + 1 }} / {{ files.length }}
          </span>
        </div>

        <div class="header-center">
          <div class="rating-bar" title="Click a star to rate (1–10), click again to clear">
            <button
              v-for="star in 10"
              :key="star"
              type="button"
              class="star-btn"
              :class="{ active: (file.rating ?? 0) >= star }"
              :disabled="ratingSaving"
              @click="setRating(star)"
            >
              ★
            </button>
            <span v-if="file.rating" class="rating-text">{{ file.rating }}/10</span>
          </div>

          <!-- Favorite toggle -->
          <button
            type="button"
            class="header-btn"
            :class="{ 'fav-active': file.is_favorite }"
            :title="file.is_favorite ? 'Favorited (Click to remove)' : 'Mark as Favorite'"
            @click="toggleFavorite"
          >
            {{ file.is_favorite ? t.batch.favorited : t.batch.favorite }}
          </button>

          <!-- NSFW toggle -->
          <button
            type="button"
            class="header-btn"
            :class="{ 'nsfw-btn-active': file.is_nsfw }"
            :title="file.is_nsfw ? 'Marked as NSFW (Click to unmark)' : 'Mark as NSFW'"
            @click="toggleNsfw"
          >
            {{ file.is_nsfw ? t.batch.nsfw : t.batch.sfw }}
          </button>
        </div>

        <div class="header-right">
          <button
            type="button"
            class="header-btn"
            title="Reveal in File Manager / Finder"
            @click="revealInFinder"
          >
            📁 {{ t.preview.reveal }}
          </button>
          <button
            type="button"
            class="header-btn"
            :class="{ active: showInspector }"
            title="Toggle Metadata Inspector (Hotkey: I)"
            @click="toggleInspector"
          >
            ℹ {{ t.preview.inspector }}
          </button>
          <button
            type="button"
            class="header-btn trash-btn"
            title="Move file to system trash"
            @click="trashCurrentFile"
          >
            🗑
          </button>
          <button
            type="button"
            class="header-btn close-btn"
            title="Close (Esc)"
            @click="emit('close')"
          >
            ✕
          </button>
        </div>
      </header>

      <!-- Main body: image viewport + collapsible inspector -->
      <div class="dialog-body">
        <div class="viewport-area">
          <button
            v-if="hasPrev"
            type="button"
            class="nav-btn prev-btn"
            title="Previous (←)"
            @click="prev"
          >
            ‹
          </button>

          <div class="image-container">
            <img
              v-if="!isVideoContainer(file.container) && file.container !== 'txt'"
              :src="assetUrl(file.path)"
              :alt="getFileName(file.path)"
              class="main-image"
              :class="{ 'nsfw-blurred': file.is_nsfw && !revealedNsfw }"
            />

            <!-- Native HTML5 video player for video files -->
            <video
              v-else-if="isVideoContainer(file.container)"
              :src="assetUrl(file.path)"
              class="main-image main-video"
              :class="{ 'nsfw-blurred': file.is_nsfw && !revealedNsfw }"
              controls
              autoplay
              loop
              playsinline
            />

            <!-- NSFW Preview Overlay -->
            <div
              v-if="file.is_nsfw && !revealedNsfw"
              class="preview-nsfw-overlay"
              @click="revealedNsfw = true"
            >
              <div class="preview-nsfw-card">
                <span class="preview-nsfw-icon">🔞</span>
                <h3>Sensitive Content (18+ NSFW)</h3>
                <p>Click anywhere or tap below to reveal this image.</p>
                <button
                  type="button"
                  class="btn-reveal"
                  @click.stop="revealedNsfw = true"
                >
                  Reveal Image
                </button>
              </div>
            </div>
            <div v-else-if="file.container === 'txt'" class="non-image-placeholder">
              <span class="placeholder-icon">TXT</span>
              <p>Preview not available for text sidecars.</p>
            </div>
          </div>

          <button
            v-if="hasNext"
            type="button"
            class="nav-btn next-btn"
            title="Next (→)"
            @click="next"
          >
            ›
          </button>
        </div>

        <!-- Metadata Inspector Sidebar -->
        <aside v-if="showInspector" class="inspector-sidebar">
          <div class="inspector-header">
            <h3>{{ t.preview.inspector }}</h3>
            <button
              type="button"
              class="close-inspector-btn"
              title="Hide Inspector"
              @click="toggleInspector"
            >
              ✕
            </button>
          </div>

          <div class="inspector-body">
            <!-- Albums & Tags Section -->
            <div class="meta-section org-section">
              <div class="section-heading">
                <h4>{{ t.nav.albums }} & {{ t.nav.tags }}</h4>
                <div class="org-actions">
                  <button
                    type="button"
                    class="copy-btn"
                    title="Add to Album"
                    @click="file.id && emit('openAlbumModal', file.id)"
                  >
                    📁 + {{ t.batch.album }}
                  </button>
                  <button
                    type="button"
                    class="copy-btn"
                    title="Manage Tags"
                    @click="file.id && emit('openTagModal', file.id)"
                  >
                    🏷 + {{ t.batch.tag }}
                  </button>
                </div>
              </div>

              <!-- Tags chips -->
              <div v-if="fileTags.length > 0" class="tag-chips-list">
                <span
                  v-for="t_tag in fileTags"
                  :key="t_tag.id"
                  class="preview-tag-chip"
                  :style="{ borderColor: t_tag.color || '#3b82f6' }"
                >
                  <span
                    class="preview-tag-dot"
                    :style="{ backgroundColor: t_tag.color || '#3b82f6' }"
                  />
                  {{ t_tag.name }}
                  <button
                    type="button"
                    class="preview-tag-remove"
                    title="Remove tag from file"
                    @click="removeTag(t_tag.id)"
                  >
                    ✕
                  </button>
                </span>
              </div>
              <div v-else class="no-tags-text">
                {{ t.nav.noTags }}
              </div>
            </div>

            <!-- Basic file specs -->
            <div class="spec-grid">
              <div class="spec-item">
                <span class="spec-label">{{ t.preview.dimensions }}</span>
                <span class="spec-value">
                  {{
                    file.metadata?.width && file.metadata?.height
                      ? `${file.metadata.width} × ${file.metadata.height}`
                      : "—"
                  }}
                </span>
              </div>
              <div class="spec-item">
                <span class="spec-label">{{ t.view.files }}</span>
                <span class="spec-value">{{ formatBytes(file.size_bytes) }}</span>
              </div>
              <div class="spec-item">
                <span class="spec-label">{{ t.preview.format }}</span>
                <span class="spec-value">{{ file.container.toUpperCase() }}</span>
              </div>
              <div class="spec-item">
                <span class="spec-label">{{ t.preview.rating }}</span>
                <span class="spec-value">{{ file.rating ? `${file.rating} ★` : t.preview.unrated }}</span>
              </div>
            </div>

            <!-- Prompt section -->
            <div v-if="file.metadata?.prompt" class="meta-section">
              <div class="section-heading">
                <h4>{{ t.preview.prompt }}</h4>
                <button
                  type="button"
                  class="copy-btn"
                  @click="copyPrompt(file.metadata?.prompt, 'prompt')"
                >
                  {{ promptCopied ? "✓ " + t.preview.copied : t.preview.copyPrompt }}
                </button>
              </div>
              <div class="text-block prompt-text">
                {{ file.metadata.prompt }}
              </div>
            </div>

            <!-- Negative Prompt section -->
            <div v-if="file.metadata?.negative_prompt" class="meta-section">
              <div class="section-heading">
                <h4>{{ t.preview.negativePrompt }}</h4>
                <button
                  type="button"
                  class="copy-btn"
                  @click="copyPrompt(file.metadata?.negative_prompt, 'negative')"
                >
                  {{ negativePromptCopied ? "✓ " + t.preview.copied : t.preview.copyNegative }}
                </button>
              </div>
              <div class="text-block negative-text">
                {{ file.metadata.negative_prompt }}
              </div>
            </div>

            <!-- Generation parameters -->
            <div v-if="file.metadata" class="meta-section">
              <div class="section-heading">
                <h4>{{ t.preview.generationParams }}</h4>
              </div>
              <div class="params-table">
                <div v-if="file.metadata.model_name" class="param-row">
                  <span class="param-key">{{ t.preview.modelName }}</span>
                  <span class="param-val" :title="file.metadata.model_name">
                    {{ file.metadata.model_name }}
                    <span v-if="file.metadata.model_hash" class="hash-tag">
                      ({{ file.metadata.model_hash }})
                    </span>
                  </span>
                </div>
                <div v-if="file.metadata.sampler" class="param-row">
                  <span class="param-key">{{ t.preview.sampler }}</span>
                  <span class="param-val">{{ file.metadata.sampler }}</span>
                </div>
                <div v-if="file.metadata.steps" class="param-row">
                  <span class="param-key">{{ t.preview.steps }}</span>
                  <span class="param-val">{{ file.metadata.steps }}</span>
                </div>
                <div v-if="file.metadata.cfg_scale" class="param-row">
                  <span class="param-key">{{ t.preview.cfgScale }}</span>
                  <span class="param-val">{{ file.metadata.cfg_scale }}</span>
                </div>
                <div v-if="file.metadata.seed" class="param-row">
                  <span class="param-key">{{ t.preview.seed }}</span>
                  <span class="param-val selectable">{{ file.metadata.seed }}</span>
                </div>
              </div>
            </div>

            <!-- Raw parameter accordion -->
            <div v-if="file.metadata?.parameters || file.metadata?.raw" class="meta-section">
              <button
                type="button"
                class="raw-toggle-btn"
                @click="showRaw = !showRaw"
              >
                <span>{{ showRaw ? "▼ Hide Raw Metadata" : "▶ Show Raw Metadata" }}</span>
              </button>
              <pre v-if="showRaw" class="raw-pre">{{
                file.metadata.parameters || file.metadata.raw
              }}</pre>
            </div>

            <!-- Path info -->
            <div class="spec-item path-item">
              <span class="spec-label">File Path</span>
              <span class="spec-value path-val" :title="normalizePath(file.path)">
                {{ normalizePath(file.path) }}
              </span>
            </div>
          </div>
        </aside>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preview-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(6px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-dialog {
  width: 96vw;
  height: 94vh;
  background: #1e1e1e;
  color: #eee;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.dialog-header {
  height: 52px;
  padding: 0 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #181818;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
  max-width: 40%;
}

.file-name {
  font-size: 0.95em;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-counter {
  font-size: 0.8em;
  color: #888;
  flex-shrink: 0;
}

.header-center {
  display: flex;
  align-items: center;
}

.rating-bar {
  display: flex;
  align-items: center;
  gap: 2px;
}

.star-btn {
  background: transparent;
  border: none;
  font-size: 1.15rem;
  color: #555;
  cursor: pointer;
  padding: 0 1px;
  line-height: 1;
  transition: color 0.15s ease, transform 0.1s ease;
}

.star-btn:hover {
  transform: scale(1.15);
  color: #fbbf24;
}

.star-btn.active {
  color: #eab308;
}

.rating-text {
  font-size: 0.8em;
  font-weight: 600;
  color: #eab308;
  margin-left: 0.4rem;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.header-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #ccc;
  padding: 0.35rem 0.75rem;
  border-radius: 6px;
  font-size: 0.85em;
  cursor: pointer;
  transition: all 0.15s ease;
}

.header-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.header-btn.active {
  background: #2f6fed;
  border-color: #2f6fed;
  color: #fff;
}

.close-btn {
  font-size: 1.1em;
  padding: 0.25rem 0.65rem;
}

.close-btn:hover {
  background: #dc2626;
  border-color: #dc2626;
}

.dialog-body {
  flex: 1;
  display: flex;
  min-height: 0;
  position: relative;
}

.viewport-area {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  background: #121212;
}

.image-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1.5rem;
  box-sizing: border-box;
}

.main-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 4px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  user-select: none;
}

.main-video {
  background: #000;
  outline: none;
}

.non-image-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  color: #888;
}

.placeholder-icon {
  font-size: 3rem;
}

.placeholder-text {
  font-size: 1.1em;
  font-weight: 600;
}

.placeholder-path {
  font-size: 0.8em;
  font-family: monospace;
}

.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 44px;
  height: 64px;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  font-size: 2.2rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
  z-index: 10;
}

.nav-btn:hover {
  background: rgba(47, 111, 237, 0.85);
  border-color: #2f6fed;
}

.prev-btn {
  left: 1rem;
}

.next-btn {
  right: 1rem;
}

/* Inspector Sidebar */
.inspector-sidebar {
  width: 360px;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  background: #181818;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  overflow: hidden;
}

.inspector-header {
  padding: 0.85rem 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.inspector-header h3 {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
}

.format-pill {
  padding: 0.15rem 0.55rem;
  border-radius: 999px;
  font-size: 0.72em;
  font-weight: 600;
  background: rgba(47, 111, 237, 0.2);
  color: #60a5fa;
  border: 1px solid rgba(47, 111, 237, 0.4);
}

.inspector-content {
  flex: 1;
  overflow-y: auto;
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.spec-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.75rem;
  background: rgba(255, 255, 255, 0.03);
  padding: 0.75rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.spec-item {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.spec-label {
  font-size: 0.72em;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.spec-value {
  font-size: 0.85em;
  font-weight: 500;
  color: #ddd;
}

.meta-section {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.section-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-heading h4 {
  margin: 0;
  font-size: 0.8em;
  font-weight: 600;
  color: #aaa;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.copy-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #aaa;
  font-size: 0.72em;
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.copy-btn:hover {
  background: #2f6fed;
  color: #fff;
  border-color: #2f6fed;
}

.text-block {
  background: rgba(0, 0, 0, 0.35);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0.65rem 0.75rem;
  font-size: 0.82em;
  line-height: 1.45;
  color: #ddd;
  max-height: 160px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-word;
}

.params-table {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  background: rgba(255, 255, 255, 0.03);
  padding: 0.75rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.param-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.8em;
  gap: 0.5rem;
}

.param-key {
  color: #888;
}

.param-val {
  font-weight: 500;
  color: #ddd;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
}

.hash-tag {
  color: #888;
  font-size: 0.9em;
}

.selectable {
  user-select: all;
  font-family: monospace;
}

.raw-toggle-btn {
  background: transparent;
  border: none;
  color: #888;
  font-size: 0.78em;
  cursor: pointer;
  padding: 0;
  text-align: left;
  transition: color 0.15s ease;
}

.raw-toggle-btn:hover {
  color: #bbb;
}

.raw-pre {
  margin: 0;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0.6rem;
  font-size: 0.72em;
  font-family: monospace;
  max-height: 180px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  color: #aaa;
}

.path-item {
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 0.75rem;
}

.path-val {
  font-family: monospace;
  font-size: 0.75em;
  word-break: break-all;
  color: #888;
}

.header-btn.fav-active {
  background: rgba(234, 179, 8, 0.2);
  border-color: #eab308;
  color: #eab308;
}

.header-btn.nsfw-btn-active {
  background: rgba(220, 38, 38, 0.2);
  border-color: #dc2626;
  color: #ef4444;
}

.main-image.nsfw-blurred {
  filter: blur(36px) brightness(0.6);
  transform: scale(1.05);
  transition: filter 0.2s ease, transform 0.2s ease;
}

.preview-nsfw-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  cursor: pointer;
  z-index: 5;
}

.preview-nsfw-card {
  background: #202020;
  border: 1px solid rgba(220, 38, 38, 0.4);
  border-radius: 12px;
  padding: 1.5rem 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  max-width: 320px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.6);
}

.preview-nsfw-icon {
  font-size: 2.5rem;
  margin-bottom: 0.5rem;
}

.preview-nsfw-card h3 {
  margin: 0 0 0.5rem 0;
  font-size: 1rem;
  color: #ef4444;
}

.preview-nsfw-card p {
  margin: 0 0 1rem 0;
  font-size: 0.82rem;
  color: #aaa;
}

.btn-reveal {
  background: #dc2626;
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 0.45rem 1rem;
  font-weight: 600;
  font-size: 0.85rem;
  cursor: pointer;
}

.btn-reveal:hover {
  background: #b91c1c;
}

.org-actions {
  display: flex;
  gap: 0.4rem;
}

.tag-chips-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  margin-top: 0.4rem;
}

.preview-tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.2rem 0.55rem;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid transparent;
  font-size: 0.78em;
  color: #eee;
}

.preview-tag-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.preview-tag-remove {
  background: transparent;
  border: none;
  color: #888;
  font-size: 0.75em;
  cursor: pointer;
  padding: 0 0.1rem;
  line-height: 1;
}

.preview-tag-remove:hover {
  color: #ef4444;
}

.no-tags-text {
  font-size: 0.8em;
  color: #666;
  font-style: italic;
  margin-top: 0.2rem;
}
</style>
