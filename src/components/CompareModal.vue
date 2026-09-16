<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { assetUrl, getFileName } from "../utils/image";
import { t } from "../i18n";
import type { ImageFile } from "../types";

const props = defineProps<{
  open: boolean;
  images: ImageFile[];
}>();

const emit = defineEmits<{
  (e: "update:open", val: boolean): void;
  (e: "setHero", image: ImageFile): void;
}>();

const leftImage = ref<ImageFile | null>(null);
const rightImage = ref<ImageFile | null>(null);

const zoom = ref(1);
const panX = ref(0);
const panY = ref(0);
const isPanning = ref(false);
const startPan = ref({ x: 0, y: 0 });

onMounted(() => {
  resetComparison();
  window.addEventListener("keydown", onKeyDown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown);
});

function resetComparison() {
  if (props.images.length >= 2) {
    leftImage.value = props.images[0];
    rightImage.value = props.images[1];
  } else if (props.images.length === 1) {
    leftImage.value = props.images[0];
    rightImage.value = null;
  } else {
    leftImage.value = null;
    rightImage.value = null;
  }
  zoom.value = 1;
  panX.value = 0;
  panY.value = 0;
}

function onWheel(e: WheelEvent) {
  e.preventDefault();
  const delta = e.deltaY < 0 ? 0.15 : -0.15;
  zoom.value = Math.max(0.5, Math.min(5, zoom.value + delta));
}

function onMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;
  isPanning.value = true;
  startPan.value = { x: e.clientX - panX.value, y: e.clientY - panY.value };
}

function onMouseMove(e: MouseEvent) {
  if (!isPanning.value) return;
  panX.value = e.clientX - startPan.value.x;
  panY.value = e.clientY - startPan.value.y;
}

function onMouseUp() {
  isPanning.value = false;
}

function onKeyDown(e: KeyboardEvent) {
  if (!props.open) return;
  if (e.key === "Escape") {
    close();
  }
}

function close() {
  emit("update:open", false);
}

function setHero(img: ImageFile) {
  emit("setHero", img);
}
</script>

<template>
  <div v-if="open" class="compare-overlay" @click.self="close">
    <div class="compare-toolbar">
      <div class="toolbar-left">
        <span class="toolbar-icon">⚖️</span>
        <span class="toolbar-title">{{ t.compare.title }}</span>
        <span class="toolbar-hint">{{ t.compare.syncHint }}</span>
      </div>

      <div class="toolbar-center">
        <button type="button" class="tool-btn" @click="zoom = Math.max(0.5, zoom - 0.2)">－</button>
        <span class="zoom-display">{{ Math.round(zoom * 100) }}%</span>
        <button type="button" class="tool-btn" @click="zoom = Math.min(5, zoom + 0.2)">＋</button>
        <button type="button" class="tool-btn reset-btn" @click="zoom = 1; panX = 0; panY = 0">
          {{ t.compare.resetView }}
        </button>
      </div>

      <div class="toolbar-right">
        <button type="button" class="btn-close" @click="close">✕</button>
      </div>
    </div>

    <!-- Split View Containers -->
    <div
      class="compare-viewport"
      @wheel="onWheel"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
    >
      <!-- Left Image Pane -->
      <div class="split-pane">
        <div v-if="leftImage" class="pane-content">
          <div class="pane-header">
            <span class="pane-badge">A</span>
            <span class="pane-filename">{{ getFileName(leftImage.path) }}</span>
            <button type="button" class="btn-hero" :title="t.compare.setAsHero" @click="setHero(leftImage)">
              ★ {{ t.compare.hero }}
            </button>
          </div>
          <div class="image-stage">
            <img
              :src="assetUrl(leftImage.path)"
              :style="{
                transform: `translate(${panX}px, ${panY}px) scale(${zoom})`,
              }"
              draggable="false"
            />
          </div>
          <div class="metadata-hud">
            <span>{{ leftImage.metadata?.model_name || 'Unknown Model' }}</span>
            <span>Seed: {{ leftImage.metadata?.seed || '-' }}</span>
            <span>Steps: {{ leftImage.metadata?.steps || '-' }}</span>
          </div>
        </div>
        <div v-else class="pane-empty">
          {{ t.compare.noImage }}
        </div>
      </div>

      <div class="split-divider"></div>

      <!-- Right Image Pane -->
      <div class="split-pane">
        <div v-if="rightImage" class="pane-content">
          <div class="pane-header">
            <span class="pane-badge">B</span>
            <span class="pane-filename">{{ getFileName(rightImage.path) }}</span>
            <button type="button" class="btn-hero" :title="t.compare.setAsHero" @click="setHero(rightImage)">
              ★ {{ t.compare.hero }}
            </button>
          </div>
          <div class="image-stage">
            <img
              :src="assetUrl(rightImage.path)"
              :style="{
                transform: `translate(${panX}px, ${panY}px) scale(${zoom})`,
              }"
              draggable="false"
            />
          </div>
          <div class="metadata-hud">
            <span>{{ rightImage.metadata?.model_name || 'Unknown Model' }}</span>
            <span>Seed: {{ rightImage.metadata?.seed || '-' }}</span>
            <span>Steps: {{ rightImage.metadata?.steps || '-' }}</span>
          </div>
        </div>
        <div v-else class="pane-empty">
          {{ t.compare.noImage }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.compare-overlay {
  position: fixed;
  inset: 0;
  background: #121212;
  z-index: 500;
  display: flex;
  flex-direction: column;
}

.compare-toolbar {
  height: 48px;
  background: rgba(24, 24, 24, 0.95);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1.2rem;
  user-select: none;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.toolbar-icon {
  font-size: 1.1rem;
}

.toolbar-title {
  font-weight: 600;
  font-size: 0.95em;
  color: #fff;
}

.toolbar-hint {
  font-size: 0.78em;
  color: #888;
}

.toolbar-center {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.tool-btn {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  border-radius: 4px;
  padding: 0.2rem 0.55rem;
  cursor: pointer;
  font-size: 0.82em;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.reset-btn {
  margin-left: 0.4rem;
}

.zoom-display {
  font-size: 0.82em;
  color: #ccc;
  min-width: 42px;
  text-align: center;
}

.btn-close {
  background: transparent;
  border: none;
  color: #fff;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
}

.btn-close:hover {
  background: rgba(255, 255, 255, 0.15);
}

.compare-viewport {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
  cursor: grab;
}

.compare-viewport:active {
  cursor: grabbing;
}

.split-pane {
  flex: 1;
  position: relative;
  overflow: hidden;
  background: #0a0a0a;
}

.split-divider {
  width: 2px;
  background: rgba(255, 255, 255, 0.15);
  z-index: 10;
}

.pane-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.pane-header {
  position: absolute;
  top: 10px;
  left: 10px;
  right: 10px;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  z-index: 10;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  padding: 0.35rem 0.6rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.pane-badge {
  background: #2f6fed;
  color: #fff;
  font-size: 0.72em;
  font-weight: bold;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
}

.pane-filename {
  font-size: 0.82em;
  color: #ddd;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.btn-hero {
  background: rgba(255, 215, 0, 0.15);
  border: 1px solid rgba(255, 215, 0, 0.3);
  color: #ffd700;
  border-radius: 4px;
  font-size: 0.75em;
  padding: 0.15rem 0.45rem;
  cursor: pointer;
}

.btn-hero:hover {
  background: rgba(255, 215, 0, 0.3);
}

.image-stage {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.image-stage img {
  max-width: 90%;
  max-height: 90%;
  object-fit: contain;
  transition: transform 0.05s linear;
  user-select: none;
}

.metadata-hud {
  position: absolute;
  bottom: 10px;
  left: 10px;
  right: 10px;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  padding: 0.35rem 0.65rem;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  gap: 1rem;
  font-size: 0.76em;
  color: #aaa;
  z-index: 10;
}

.pane-empty {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #666;
  font-size: 0.9em;
}
</style>
