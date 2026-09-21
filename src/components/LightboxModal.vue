<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ImageFile } from "../types";
import { assetUrl, formatBytes, formatDuration, formatPlatformName, getFileName, isVideoContainer } from "../utils/image";
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

// Video Player Controls & State
const videoRef = ref<HTMLVideoElement | null>(null);
const isPlaying = ref(true);
const isLooping = ref(true);
const isMuted = ref(false);
const playbackRate = ref(1.0);
const videoCurrentTime = ref(0);
const videoDuration = ref(0);
const showWorkflowInspector = ref(false);
const workflowCopied = ref(false);

const playbackRates = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0];

function togglePlay() {
  if (!videoRef.value) return;
  if (videoRef.value.paused) {
    videoRef.value.play().catch(() => {});
    isPlaying.value = true;
  } else {
    videoRef.value.pause();
    isPlaying.value = false;
  }
}

function toggleLoop() {
  isLooping.value = !isLooping.value;
  if (videoRef.value) {
    videoRef.value.loop = isLooping.value;
  }
}

function toggleMute() {
  isMuted.value = !isMuted.value;
  if (videoRef.value) {
    videoRef.value.muted = isMuted.value;
  }
}

function setPlaybackRate(rate: number) {
  playbackRate.value = rate;
  if (videoRef.value) {
    videoRef.value.playbackRate = rate;
  }
}

function stepFrame(forward: boolean) {
  if (!videoRef.value) return;
  videoRef.value.pause();
  isPlaying.value = false;
  const fps = props.file.metadata?.fps || 30;
  const step = 1 / fps;
  if (forward) {
    videoRef.value.currentTime = Math.min(videoRef.value.duration || 0, videoRef.value.currentTime + step);
  } else {
    videoRef.value.currentTime = Math.max(0, videoRef.value.currentTime - step);
  }
}

function onVideoTimeUpdate() {
  if (!videoRef.value) return;
  videoCurrentTime.value = videoRef.value.currentTime;
  if (!videoDuration.value && videoRef.value.duration) {
    videoDuration.value = videoRef.value.duration;
  }
}

function onVideoLoadedMetadata() {
  if (!videoRef.value) return;
  videoDuration.value = videoRef.value.duration;
  videoRef.value.playbackRate = playbackRate.value;
  videoRef.value.loop = isLooping.value;
  videoRef.value.muted = isMuted.value;
  if (isPlaying.value) {
    videoRef.value.play().catch(() => {});
  }
}

function seekVideo(e: MouseEvent) {
  if (!videoRef.value || !videoDuration.value) return;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  videoRef.value.currentTime = ratio * videoDuration.value;
  videoCurrentTime.value = videoRef.value.currentTime;
}

async function copyWorkflow() {
  const json = props.file.metadata?.parameters || props.file.metadata?.raw;
  if (!json) return;
  try {
    await navigator.clipboard.writeText(json);
    workflowCopied.value = true;
    setTimeout(() => (workflowCopied.value = false), 2000);
  } catch (err) {
    console.error("Workflow copy failed:", err);
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
    if (showWorkflowInspector.value) {
      showWorkflowInspector.value = false;
      return;
    }
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
    return;
  }
  if (isVideoContainer(props.file.container)) {
    if (e.code === "Space") {
      e.preventDefault();
      togglePlay();
      return;
    }
    if (e.key === "." || e.key === ">") {
      e.preventDefault();
      stepFrame(true);
      return;
    }
    if (e.key === "," || e.key === "<") {
      e.preventDefault();
      stepFrame(false);
      return;
    }
    if (e.key === "l" || e.key === "L") {
      e.preventDefault();
      toggleLoop();
      return;
    }
    if (e.key === "m" || e.key === "M") {
      e.preventDefault();
      toggleMute();
      return;
    }
    if (e.key === "i" || e.key === "I") {
      e.preventDefault();
      showWorkflowInspector.value = !showWorkflowInspector.value;
      return;
    }
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
        <span v-if="file.metadata?.duration_seconds" class="badge">
          ⏱ {{ formatDuration(file.metadata.duration_seconds) }}
        </span>
        <span v-if="file.metadata?.fps" class="badge">
          {{ Math.round(file.metadata.fps) }} fps
        </span>
        <span v-if="file.metadata?.video_codec" class="badge">
          {{ file.metadata.video_codec }}
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

      <!-- Main Scaled Media Container -->
      <div
        class="img-container"
        :style="{
          transform: `translate(${translateX}px, ${translateY}px) scale(${scale})`,
        }"
      >
        <video
          v-if="isVideoContainer(file.container)"
          ref="videoRef"
          :src="assetUrl(file.path)"
          :loop="isLooping"
          :muted="isMuted"
          playsinline
          class="lightbox-video"
          :class="{ blurred: file.is_nsfw && !revealedNsfw }"
          @timeupdate="onVideoTimeUpdate"
          @loadedmetadata="onVideoLoadedMetadata"
          @click.stop="togglePlay"
        />
        <img
          v-else
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

    <!-- Video Player Floating HUD -->
    <div v-if="isVideoContainer(file.container)" class="video-player-bar">
      <button
        type="button"
        class="v-ctrl-btn play-btn"
        :title="`${isPlaying ? t.preview.pause : t.preview.play} (Space)`"
        @click.stop="togglePlay"
      >
        {{ isPlaying ? '⏸' : '▶' }}
      </button>
      <button
        type="button"
        class="v-ctrl-btn"
        :title="`${t.preview.stepBackward} (,)`"
        @click.stop="stepFrame(false)"
      >
        ⏮
      </button>
      <button
        type="button"
        class="v-ctrl-btn"
        :title="`${t.preview.stepForward} (.)`"
        @click.stop="stepFrame(true)"
      >
        ⏭
      </button>

      <span class="video-time-display">
        {{ formatDuration(videoCurrentTime) }} / {{ formatDuration(videoDuration || file.metadata?.duration_seconds || 0) }}
      </span>

      <!-- Scrubber -->
      <div class="video-scrubber" @click.stop="seekVideo">
        <div
          class="video-scrubber-fill"
          :style="{ width: `${videoDuration ? (videoCurrentTime / videoDuration) * 100 : 0}%` }"
        ></div>
      </div>

      <!-- Loop Toggle -->
      <button
        type="button"
        class="v-ctrl-btn"
        :class="{ active: isLooping }"
        :title="`${t.preview.loop} (L)`"
        @click.stop="toggleLoop"
      >
        🔁
      </button>

      <!-- Playback Speed -->
      <select
        :value="playbackRate"
        class="v-speed-select"
        :title="t.preview.playbackSpeed"
        @change="setPlaybackRate(Number(($event.target as HTMLSelectElement).value))"
        @click.stop
      >
        <option v-for="rate in playbackRates" :key="rate" :value="rate">
          {{ rate }}x
        </option>
      </select>

      <!-- Mute Toggle -->
      <button
        type="button"
        class="v-ctrl-btn"
        :title="`${isMuted ? t.preview.unmute : t.preview.mute} (M)`"
        @click.stop="toggleMute"
      >
        {{ isMuted ? '🔇' : '🔊' }}
      </button>

      <!-- Workflow Inspector Toggle Button -->
      <button
        type="button"
        class="v-ctrl-btn workflow-btn"
        :class="{ active: showWorkflowInspector }"
        :title="`${t.preview.videoWorkflow} (I)`"
        @click.stop="showWorkflowInspector = !showWorkflowInspector"
      >
        <span>⚡ {{ t.preview.videoWorkflow }}</span>
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

    <!-- Video / Generation Workflow Inspector Slide-out Drawer -->
    <div
      v-if="showWorkflowInspector"
      class="workflow-inspector-panel"
      @click.stop
    >
      <div class="inspector-header">
        <h4>⚡ {{ t.preview.videoWorkflow }}</h4>
        <button
          type="button"
          class="hud-btn close-hud"
          @click="showWorkflowInspector = false"
        >
          ✕
        </button>
      </div>

      <div class="inspector-content">
        <div class="info-section">
          <div v-if="file.metadata?.model_name" class="info-item">
            <span class="info-key">{{ t.preview.modelName }}</span>
            <span class="info-val highlight">{{ file.metadata.model_name }}</span>
          </div>
          <div v-if="file.metadata?.sampler" class="info-item">
            <span class="info-key">{{ t.preview.sampler }}</span>
            <span class="info-val">{{ file.metadata.sampler }}</span>
          </div>
          <div v-if="file.metadata?.steps" class="info-item">
            <span class="info-key">{{ t.preview.steps }}</span>
            <span class="info-val">{{ file.metadata.steps }}</span>
          </div>
          <div v-if="file.metadata?.cfg_scale" class="info-item">
            <span class="info-key">{{ t.preview.cfgScale }}</span>
            <span class="info-val">{{ file.metadata.cfg_scale }}</span>
          </div>
          <div v-if="file.metadata?.seed" class="info-item">
            <span class="info-key">{{ t.preview.seed }}</span>
            <span class="info-val">{{ file.metadata.seed }}</span>
          </div>
          <div v-if="file.metadata?.duration_seconds" class="info-item">
            <span class="info-key">{{ t.preview.duration }}</span>
            <span class="info-val">{{ formatDuration(file.metadata.duration_seconds) }}</span>
          </div>
          <div v-if="file.metadata?.fps" class="info-item">
            <span class="info-key">{{ t.preview.fps }}</span>
            <span class="info-val">{{ Math.round(file.metadata.fps) }} fps</span>
          </div>
          <div v-if="file.metadata?.video_codec" class="info-item">
            <span class="info-key">{{ t.preview.codec }}</span>
            <span class="info-val">{{ file.metadata.video_codec }}</span>
          </div>
        </div>

        <!-- Prompts -->
        <div v-if="file.metadata?.prompt" class="prompt-box">
          <div class="prompt-box-header">
            <span>{{ t.preview.prompt }}</span>
            <button type="button" class="mini-copy-btn" @click="copyPrompt">
              {{ promptCopied ? t.preview.copied : t.preview.copyPrompt }}
            </button>
          </div>
          <p class="prompt-text">{{ file.metadata.prompt }}</p>
        </div>

        <div v-if="file.metadata?.negative_prompt" class="prompt-box">
          <div class="prompt-box-header">
            <span>{{ t.preview.negativePrompt }}</span>
          </div>
          <p class="prompt-text negative">{{ file.metadata.negative_prompt }}</p>
        </div>

        <!-- Raw JSON / Workflow Parameters -->
        <div v-if="file.metadata?.parameters || file.metadata?.raw" class="workflow-raw-box">
          <div class="prompt-box-header">
            <span>Workflow Graph / Metadata</span>
            <button type="button" class="mini-copy-btn" @click="copyWorkflow">
              {{ workflowCopied ? t.preview.workflowCopied : t.preview.copyWorkflow }}
            </button>
          </div>
          <pre class="raw-pre">{{ file.metadata.parameters || file.metadata.raw }}</pre>
        </div>
      </div>
    </div>
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

.img-container video {
  max-width: 90vw;
  max-height: 82vh;
  object-fit: contain;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.8);
  border-radius: 4px;
}

.img-container video.blurred {
  filter: blur(30px);
}

.video-player-bar {
  position: absolute;
  bottom: 64px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(18, 18, 22, 0.88);
  border: 1px solid rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(12px);
  padding: 6px 14px;
  border-radius: 999px;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.6);
  z-index: 20;
}

.v-ctrl-btn {
  background: transparent;
  border: 1px solid transparent;
  color: #e2e8f0;
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 0.85rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.v-ctrl-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}

.v-ctrl-btn.active {
  background: rgba(99, 102, 241, 0.3);
  border-color: rgba(99, 102, 241, 0.6);
  color: #818cf8;
}

.v-ctrl-btn.play-btn {
  font-size: 1rem;
  padding: 4px 10px;
}

.video-time-display {
  font-size: 0.76rem;
  color: #94a3b8;
  font-family: monospace;
  white-space: nowrap;
}

.video-scrubber {
  width: 140px;
  height: 6px;
  background: rgba(255, 255, 255, 0.18);
  border-radius: 999px;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.video-scrubber-fill {
  height: 100%;
  background: #6366f1;
  border-radius: 999px;
  transition: width 0.05s linear;
}

.v-speed-select {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.14);
  color: #e2e8f0;
  font-size: 0.74rem;
  border-radius: 4px;
  padding: 2px 4px;
  outline: none;
  cursor: pointer;
}

.v-speed-select option {
  background: #18181b;
  color: #f8fafc;
}

.workflow-btn {
  font-size: 0.76rem;
  padding: 4px 8px;
  border-radius: 999px;
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
  color: #a5b4fc;
}

.workflow-inspector-panel {
  position: absolute;
  top: 54px;
  right: 18px;
  bottom: 70px;
  width: 380px;
  max-width: calc(100vw - 36px);
  background: rgba(18, 18, 22, 0.94);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 12px;
  backdrop-filter: blur(16px);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
  z-index: 30;
  overflow: hidden;
  animation: slideInRight 0.2s ease-out;
}

@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateX(30px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.inspector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.inspector-header h4 {
  margin: 0;
  font-size: 0.92rem;
  font-weight: 600;
  color: #f8fafc;
}

.inspector-content {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: rgba(255, 255, 255, 0.04);
  padding: 10px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.78rem;
}

.info-key {
  color: #94a3b8;
}

.info-val {
  color: #e2e8f0;
  font-weight: 500;
}

.info-val.highlight {
  color: #818cf8;
  font-weight: 600;
}

.prompt-box,
.workflow-raw-box {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.prompt-box-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.74rem;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

.mini-copy-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #cbd5e1;
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.12s;
}

.mini-copy-btn:hover {
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
}

.prompt-text {
  margin: 0;
  font-size: 0.8rem;
  line-height: 1.4;
  color: #e2e8f0;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 140px;
  overflow-y: auto;
}

.prompt-text.negative {
  color: #fca5a5;
}

.raw-pre {
  margin: 0;
  font-size: 0.72rem;
  font-family: monospace;
  color: #a5b4fc;
  background: rgba(0, 0, 0, 0.3);
  padding: 8px;
  border-radius: 6px;
  max-height: 200px;
  overflow: auto;
  white-space: pre;
}
</style>
