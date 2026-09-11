<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ImageFile } from "../types";
import { assetUrl, formatBytes, formatPlatformName, getFileName } from "../utils/image";
import { t } from "../i18n";

const props = defineProps<{
  file: ImageFile;
  files: ImageFile[];
}>();

const emit = defineEmits<{
  close: [];
  navigate: [file: ImageFile];
  updateFile: [file: ImageFile];
  findSimilar: [file: ImageFile];
}>();

const scale = ref(1);
const translateX = ref(0);
const translateY = ref(0);
const isDragging = ref(false);
const dragStartX = ref(0);
const dragStartY = ref(0);
const promptCopied = ref(false);
const revealedNsfw = ref(false);

const cachedIndex = ref(-1);

watch(
  () => props.file.path,
  (newPath) => {
    resetTransform();
    revealedNsfw.value = false;
    if (cachedIndex.value >= 0 && cachedIndex.value < props.files.length) {
      if (props.files[cachedIndex.value]?.path === newPath) return;
      if (cachedIndex.value + 1 < props.files.length && props.files[cachedIndex.value + 1]?.path === newPath) {
        cachedIndex.value++;
        return;
      }
      if (cachedIndex.value > 0 && props.files[cachedIndex.value - 1]?.path === newPath) {
        cachedIndex.value--;
        return;
      }
    }
    cachedIndex.value = props.files.findIndex((f) => f.path === newPath);
  },
  { immediate: true },
);

const currentIndex = computed(() => cachedIndex.value);
const hasPrev = computed(() => currentIndex.value > 0);
const hasNext = computed(() => currentIndex.value >= 0 && currentIndex.value < props.files.length - 1);

// Silent background preload for adjacent images
watch(
  currentIndex,
  (idx) => {
    if (idx > 0 && props.files[idx - 1]) {
      const prevImg = new Image();
      prevImg.src = assetUrl(props.files[idx - 1].path);
    }
    if (idx >= 0 && idx < props.files.length - 1 && props.files[idx + 1]) {
      const nextImg = new Image();
      nextImg.src = assetUrl(props.files[idx + 1].path);
    }
  },
  { immediate: true },
);

function resetTransform() {
  scale.value = 1;
  translateX.value = 0;
  translateY.value = 0;
}

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

function handleWheel(e: WheelEvent) {
  e.preventDefault();
  const delta = e.deltaY > 0 ? -0.15 : 0.15;
  const newScale = Math.min(Math.max(0.2, scale.value + delta), 5);
  scale.value = newScale;
  if (newScale <= 1) {
    translateX.value = 0;
    translateY.value = 0;
  }
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  isDragging.value = true;
  dragStartX.value = e.clientX - translateX.value;
  dragStartY.value = e.clientY - translateY.value;
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging.value) return;
  translateX.value = e.clientX - dragStartX.value;
  translateY.value = e.clientY - dragStartY.value;
}

function onMouseUp() {
  isDragging.value = false;
}

async function copyPrompt() {
  const p = props.file.metadata?.prompt;
  if (!p) return;
  try {
    await navigator.clipboard.writeText(p);
    promptCopied.value = true;
    setTimeout(() => (promptCopied.value = false), 1500);
  } catch (err) {
    console.error("Copy failed:", err);
  }
}

async function setRating(r: number) {
  if (!props.file.id) return;
  const newRating = props.file.rating === r ? null : r;
  try {
    await invoke("set_file_rating", { fileId: props.file.id, rating: newRating });
    props.file.rating = newRating ?? undefined;
    emit("updateFile", props.file);
  } catch (err) {
    console.error("Set rating error:", err);
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
    console.error("Toggle favorite error:", err);
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    emit("close");
    return;
  }
  if (e.key === "ArrowLeft") {
    e.preventDefault();
    prev();
    return;
  }
  if (e.key === "ArrowRight") {
    e.preventDefault();
    next();
    return;
  }
  if (e.key === "f" || e.key === "F") {
    e.preventDefault();
    void toggleFavorite();
    return;
  }
  if (["0", "1", "2", "3", "4", "5"].includes(e.key)) {
    e.preventDefault();
    void setRating(parseInt(e.key, 10));
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeyDown);
});
</script>

<template>
  <div
    class="lightbox-overlay"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @click.self="emit('close')"
  >
    <!-- Top HUD Bar -->
    <header class="lightbox-header">
      <div class="file-info">
        <span class="file-name">{{ getFileName(file.path) }}</span>
        <span v-if="file.metadata?.width && file.metadata?.height" class="badge">
          {{ file.metadata.width }} × {{ file.metadata.height }}
        </span>
        <span v-if="file.size_bytes" class="badge">
          {{ formatBytes(file.size_bytes) }}
        </span>
        <span class="badge uppercase">
          {{ formatPlatformName(file.metadata?.format) || file.container }}
        </span>
      </div>

      <div class="header-actions">
        <span class="index-indicator">
          {{ currentIndex + 1 }} / {{ files.length }}
        </span>
        <button
          type="button"
          class="hud-btn"
          :title="t.menu.resetZoom"
          @click="resetTransform"
        >
          {{ Math.round(scale * 100) }}%
        </button>
        <button
          type="button"
          class="hud-btn close-hud"
          :title="`${t.preview.close} (Esc)`"
          @click="emit('close')"
        >
          ✕
        </button>
      </div>
    </header>

    <!-- Center Viewer Canvas -->
    <div
      class="lightbox-canvas"
      :class="{ dragging: isDragging }"
      @wheel="handleWheel"
      @mousedown="onMouseDown"
      @click.self="emit('close')"
    >
      <!-- Navigation Prev -->
      <button
        v-if="hasPrev"
        type="button"
        class="nav-arrow prev"
        title="‹ (←)"
        @click.stop="prev"
      >
        ‹
      </button>

      <!-- Main Scaled Image -->
      <div
        class="img-container"
        :style="{
          transform: `translate(${translateX}px, ${translateY}px) scale(${scale})`,
        }"
      >
        <img
          :src="assetUrl(file.path)"
          :alt="getFileName(file.path)"
          :class="{ blurred: file.is_nsfw && !revealedNsfw }"
          draggable="false"
        />
        <div
          v-if="file.is_nsfw && !revealedNsfw"
          class="lightbox-nsfw-overlay"
          @click.stop="revealedNsfw = true"
        >
          <span>{{ t.preview.clickToReveal }}</span>
        </div>
      </div>

      <!-- Navigation Next -->
      <button
        v-if="hasNext"
        type="button"
        class="nav-arrow next"
        title="› (→)"
        @click.stop="next"
      >
        ›
      </button>
    </div>

    <!-- Bottom HUD Floating Bar -->
    <footer class="lightbox-footer">
      <div class="footer-center-hud">
        <!-- Quick Rating -->
        <div class="stars-row">
          <button
            v-for="star in 5"
            :key="star"
            type="button"
            class="star-hud-btn"
            :class="{ active: (file.rating ?? 0) >= star }"
            @click="setRating(star)"
          >
            ★
          </button>
        </div>

        <div class="divider"></div>

        <!-- Favorite -->
        <button
          type="button"
          class="hud-action-btn"
          :class="{ active: file.is_favorite }"
          :title="file.is_favorite ? t.preview.removeFavorite : t.preview.addFavorite"
          @click="toggleFavorite"
        >
          {{ file.is_favorite ? '❤️' : '🤍' }}
        </button>

        <!-- Find Similar -->
        <button
          type="button"
          class="hud-action-btn"
          :title="t.preview.findSimilar"
          @click="emit('findSimilar', file)"
        >
          <span>🔍 {{ t.preview.findSimilar }}</span>
        </button>

        <!-- Prompt Copy -->
        <button
          v-if="file.metadata?.prompt"
          type="button"
          class="hud-action-btn prompt-copy-btn"
          :title="file.metadata.prompt"
          @click="copyPrompt"
        >
          <span>📝 {{ promptCopied ? t.preview.copied : t.preview.copyPrompt }}</span>
        </button>
      </div>
    </footer>
  </div>
</template>

<style scoped>
.lightbox-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  background: rgba(0, 0, 0, 0.88);
  backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  user-select: none;
}

.lightbox-header {
  height: 46px;
  padding: 0 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: linear-gradient(to bottom, rgba(0, 0, 0, 0.75), transparent);
  z-index: 10;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-name {
  font-size: 0.84rem;
  font-weight: 600;
  color: #f8fafc;
}

.badge {
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: #94a3b8;
}

.badge.uppercase {
  text-transform: uppercase;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.index-indicator {
  font-size: 0.76rem;
  color: #94a3b8;
}

.hud-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #e2e8f0;
  padding: 3px 8px;
  border-radius: 5px;
  font-size: 0.76rem;
  cursor: pointer;
  transition: all 0.15s;
}

.hud-btn:hover {
  background: rgba(255, 255, 255, 0.18);
  color: #ffffff;
}

.close-hud {
  font-size: 0.9rem;
  padding: 3px 10px;
}

.close-hud:hover {
  background: #ef4444;
  border-color: #ef4444;
}

.lightbox-canvas {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  cursor: grab;
}

.lightbox-canvas.dragging {
  cursor: grabbing;
}

.img-container {
  max-width: 90vw;
  max-height: 82vh;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.05s ease-out;
  transform-origin: center center;
}

.img-container img {
  max-width: 90vw;
  max-height: 82vh;
  object-fit: contain;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.8);
  border-radius: 4px;
}

.img-container img.blurred {
  filter: blur(30px);
}

.lightbox-nsfw-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  color: #f87171;
  font-weight: 600;
  font-size: 0.88rem;
  cursor: pointer;
}

.nav-arrow {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 44px;
  height: 68px;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #f8fafc;
  font-size: 1.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 6px;
  backdrop-filter: blur(4px);
  transition: all 0.15s;
  z-index: 10;
}

.nav-arrow.prev {
  left: 18px;
}

.nav-arrow.next {
  right: 18px;
}

.nav-arrow:hover {
  background: rgba(255, 255, 255, 0.2);
  transform: translateY(-50%) scale(1.05);
}

.lightbox-footer {
  height: 54px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.75), transparent);
  z-index: 10;
}

.footer-center-hud {
  display: flex;
  align-items: center;
  gap: 10px;
  background: rgba(24, 24, 27, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(8px);
  padding: 5px 14px;
  border-radius: 999px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
}

.stars-row {
  display: flex;
  gap: 2px;
}

.star-hud-btn {
  background: transparent;
  border: none;
  color: #475569;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 0 2px;
  transition: all 0.1s;
}

.star-hud-btn:hover {
  transform: scale(1.2);
  color: #fbbf24;
}

.star-hud-btn.active {
  color: #f59e0b;
}

.divider {
  width: 1px;
  height: 16px;
  background: rgba(255, 255, 255, 0.15);
}

.hud-action-btn {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #e2e8f0;
  border-radius: 999px;
  padding: 3px 9px;
  font-size: 0.78rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: all 0.15s;
}

.hud-action-btn:hover {
  background: rgba(255, 255, 255, 0.15);
}

.hud-action-btn.active {
  background: rgba(236, 72, 153, 0.25);
  border-color: rgba(236, 72, 153, 0.5);
}

.prompt-copy-btn {
  max-width: 220px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
