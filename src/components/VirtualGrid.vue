<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { ImageFile } from "../types";
import {
  assetUrl,
  formatBytes,
  formatPlatformName,
  getFileName,
  normalizePath,
} from "../utils/image";
import {
  getThumbnailUrl,
  getThumbnailUrlSync,
  requestBatchThumbnails,
} from "../utils/thumbnail";
import { t } from "../i18n";

const props = withDefaults(
  defineProps<{
    files: ImageFile[];
    selectedFile?: ImageFile | null;
    selectedFilePaths?: Set<string>;
    loading?: boolean;
    itemMinWidth?: number;
    gap?: number;
    overscan?: number;
    blurNsfw?: boolean;
    showCardBadges?: boolean;
    stackMap?: Record<string, { count: number; heroId: number | null }>;
    expandedStacks?: Set<string>;
  }>(),
  {
    selectedFile: null,
    loading: false,
    itemMinWidth: 180,
    gap: 16,
    overscan: 4,
    blurNsfw: true,
    showCardBadges: true,
  },
);

const emit = defineEmits<{
  (e: "select", file: ImageFile, event?: MouseEvent): void;
  (e: "activate", file: ImageFile): void;
  (e: "toggleSelect", file: ImageFile): void;
  (e: "findSimilar", file: ImageFile): void;
  (e: "toggleStackExpand", stackId: string): void;
  (e: "compareStack", stackId: string): void;
}>();

const containerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerWidth = ref(800);
const containerHeight = ref(600);

// Image loading error tracker
const failedImages = ref<Set<string>>(new Set());
const revealedNsfw = ref<Set<string>>(new Set());

function onImageError(path: string) {
  failedImages.value.add(path);
}

function toggleNsfwReveal(path: string) {
  if (revealedNsfw.value.has(path)) {
    revealedNsfw.value.delete(path);
  } else {
    revealedNsfw.value.add(path);
  }
}

// Update container dimensions
function updateDimensions() {
  if (!containerRef.value) return;
  containerWidth.value = containerRef.value.clientWidth;
  containerHeight.value = containerRef.value.clientHeight;
}

let resizeObserver: ResizeObserver | null = null;
let resizeDebounceTimer: ReturnType<typeof setTimeout> | null = null;

onMounted(() => {
  if (containerRef.value) {
    updateDimensions();
    resizeObserver = new ResizeObserver(() => {
      if (resizeDebounceTimer) clearTimeout(resizeDebounceTimer);
      resizeDebounceTimer = setTimeout(() => {
        updateDimensions();
      }, 50);
    });
    resizeObserver.observe(containerRef.value);
  }
  window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  if (resizeDebounceTimer) clearTimeout(resizeDebounceTimer);
  if (prefetchDebounceTimer) clearTimeout(prefetchDebounceTimer);
  resizeObserver?.disconnect();
  window.removeEventListener("keydown", handleKeyDown);
});

function onScroll(e: Event) {
  const target = e.target as HTMLElement;
  scrollTop.value = target.scrollTop;
}

// Columns count based on container width
const cols = computed(() => {
  const available = containerWidth.value - 2; // small padding offset
  const minWidth = props.itemMinWidth;
  const count = Math.floor((available + props.gap) / (minWidth + props.gap));
  return Math.max(1, count);
});

// Single item dimensions
const itemWidth = computed(() => {
  const totalGaps = (cols.value - 1) * props.gap;
  return Math.floor((containerWidth.value - totalGaps) / cols.value);
});

// Card height = 1:1 square image + 56px info footer
const CARD_INFO_HEIGHT = 56;
const cardHeight = computed(() => itemWidth.value + CARD_INFO_HEIGHT);
const rowHeight = computed(() => cardHeight.value + props.gap);

// Total grid rows and phantom scroll height
const totalRows = computed(() => Math.ceil(props.files.length / cols.value));
const totalHeight = computed(() => {
  if (totalRows.value === 0) return 0;
  return totalRows.value * rowHeight.value - props.gap;
});

// Visible row range
const startRow = computed(() => {
  const raw = Math.floor(scrollTop.value / rowHeight.value);
  return Math.max(0, raw - props.overscan);
});

const endRow = computed(() => {
  const visibleCount = Math.ceil(containerHeight.value / rowHeight.value);
  const raw = startRow.value + visibleCount + props.overscan * 2;
  return Math.min(totalRows.value - 1, raw);
});

// Sliced visible items with their absolute row offset
const startIndex = computed(() => startRow.value * cols.value);
const endIndex = computed(() =>
  Math.min(props.files.length - 1, (endRow.value + 1) * cols.value - 1),
);

const visibleFiles = computed(() => {
  if (props.files.length === 0) return [];
  return props.files.slice(startIndex.value, endIndex.value + 1);
});

const translateY = computed(() => startRow.value * rowHeight.value);

const thumbnailMap = ref<Record<number, string>>({});

function getCardImageSrc(file: ImageFile): string {
  if (!file.id) return assetUrl(file.path);
  return thumbnailMap.value[file.id] || getThumbnailUrlSync(file) || assetUrl(file.path);
}

async function loadThumbnailFor(file: ImageFile) {
  if (!file.id || thumbnailMap.value[file.id]) return;
  const url = await getThumbnailUrl(file);
  if (file.id) {
    thumbnailMap.value[file.id] = url;
  }
}

let prefetchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  visibleFiles,
  (files) => {
    if (!files || files.length === 0) return;

    // 1. Immediately request thumbnails for currently visible items
    for (const f of files) {
      if (f.id && !thumbnailMap.value[f.id]) {
        void loadThumbnailFor(f);
      }
    }
    void requestBatchThumbnails(files);

    // 2. Proactive Lookahead Preload: pre-generate next 100 items in background
    if (prefetchDebounceTimer) clearTimeout(prefetchDebounceTimer);
    prefetchDebounceTimer = setTimeout(() => {
      const aheadStart = endIndex.value + 1;
      const aheadEnd = Math.min(props.files.length, endIndex.value + 101);
      if (aheadStart < aheadEnd) {
        const aheadSlice = props.files.slice(aheadStart, aheadEnd);
        void requestBatchThumbnails(aheadSlice);
      }
      const behindStart = Math.max(0, startIndex.value - 40);
      if (behindStart < startIndex.value) {
        const behindSlice = props.files.slice(behindStart, startIndex.value);
        void requestBatchThumbnails(behindSlice);
      }
    }, 40);
  },
  { immediate: true },
);

function selectFile(file: ImageFile, event?: MouseEvent) {
  emit("select", file, event);
}

function toggleSelect(file: ImageFile) {
  emit("toggleSelect", file);
}

function activateFile(file: ImageFile) {
  emit("activate", file);
}

// Format dimensions helper
function formatDimensions(file: ImageFile): string {
  if (file.metadata?.width && file.metadata?.height) {
    return `${file.metadata.width} × ${file.metadata.height}`;
  }
  return formatBytes(file.size_bytes);
}

// Selected index tracking for O(1) keyboard navigation
const selectedIndex = ref(-1);
watch(
  () => props.selectedFile,
  (file) => {
    if (!file) {
      selectedIndex.value = -1;
      return;
    }
    if (selectedIndex.value >= 0 && selectedIndex.value < props.files.length) {
      if (props.files[selectedIndex.value]?.path === file.path) {
        return;
      }
    }
    selectedIndex.value = props.files.findIndex((f) => f.path === file.path);
  },
  { immediate: true },
);

// Keyboard navigation
function handleKeyDown(e: KeyboardEvent) {
  // Only handle navigation if active element is not an input or textarea
  const tag = (document.activeElement?.tagName ?? "").toLowerCase();
  if (tag === "input" || tag === "textarea") return;

  if (!props.files.length) return;

  const currentIndex = selectedIndex.value;

  let nextIndex = currentIndex;

  switch (e.key) {
    case "ArrowRight":
      nextIndex = currentIndex < props.files.length - 1 ? currentIndex + 1 : 0;
      break;
    case "ArrowLeft":
      nextIndex = currentIndex > 0 ? currentIndex - 1 : props.files.length - 1;
      break;
    case "ArrowDown":
      if (currentIndex + cols.value < props.files.length) {
        nextIndex = currentIndex + cols.value;
      }
      break;
    case "ArrowUp":
      if (currentIndex - cols.value >= 0) {
        nextIndex = currentIndex - cols.value;
      }
      break;
    default:
      return;
  }

  if (nextIndex >= 0 && nextIndex < props.files.length && nextIndex !== currentIndex) {
    e.preventDefault();
    selectedIndex.value = nextIndex;
    selectFile(props.files[nextIndex]);
    scrollToIndex(nextIndex);
  }
}

// Ensure the selected item is scrolled into visible viewport
function scrollToIndex(index: number) {
  if (!containerRef.value) return;
  const targetRow = Math.floor(index / cols.value);
  const targetTop = targetRow * rowHeight.value;
  const targetBottom = targetTop + cardHeight.value;

  const currentScrollTop = containerRef.value.scrollTop;
  const viewportHeight = containerRef.value.clientHeight;

  if (targetTop < currentScrollTop) {
    containerRef.value.scrollTop = targetTop;
  } else if (targetBottom > currentScrollTop + viewportHeight) {
    containerRef.value.scrollTop = targetBottom - viewportHeight;
  }
}

function onDragStart(e: DragEvent, file: ImageFile) {
  const selectedPaths = props.selectedFilePaths && props.selectedFilePaths.has(file.path)
    ? Array.from(props.selectedFilePaths)
    : [file.path];

  const payload = {
    file_paths: selectedPaths,
    file_ids: file.id ? [file.id] : [],
  };

  e.dataTransfer?.setData("application/json", JSON.stringify(payload));
  e.dataTransfer?.setData("text/plain", selectedPaths.join("\n"));
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = "copyMove";
  }
}

</script>

<template>
  <div class="virtual-grid-wrapper">
    <div v-if="loading" class="grid-placeholder">{{ t.view.loading }}</div>
    <div v-else-if="!files.length" class="grid-placeholder">
      {{ t.view.selectFolderPrompt }}
    </div>

    <div
      v-else
      ref="containerRef"
      class="virtual-grid-container"
      role="grid"
      aria-label="Image gallery grid"
      tabindex="0"
      @scroll.passive="onScroll"
    >
      <div class="virtual-phantom" :style="{ height: `${totalHeight}px` }">
        <div
          class="virtual-content"
          :style="{
            transform: `translateY(${translateY}px)`,
            gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
            gap: `${gap}px`,
          }"
        >
          <div
            v-for="file in visibleFiles"
            :key="file.id ?? file.path"
            class="grid-card"
            role="gridcell"
            :aria-selected="selectedFile?.path === file.path"
            :class="{
              active: selectedFile?.path === file.path,
              'multi-selected': selectedFilePaths?.has(file.path),
              'is-stacked': file.stack_id && (stackMap?.[file.stack_id]?.count ?? 1) > 1,
              'stack-expanded': file.stack_id && expandedStacks?.has(file.stack_id),
            }"
            draggable="true"
            @dragstart="onDragStart($event, file)"
            @click="selectFile(file, $event)"
            @dblclick="activateFile(file)"
            @contextmenu.prevent="emit('findSimilar', file)"
          >
            <div class="thumbnail-wrapper">
              <button
                type="button"
                class="card-select-btn"
                :class="{ checked: selectedFilePaths?.has(file.path) }"
                :aria-label="selectedFilePaths?.has(file.path) ? t.view.deselect : t.view.selectAll"
                :title="selectedFilePaths?.has(file.path) ? t.view.deselect : t.view.selectAll"
                @click.stop="toggleSelect(file)"
              >
                {{ selectedFilePaths?.has(file.path) ? "✓" : "" }}
              </button>
              <button
                v-if="file.id"
                type="button"
                class="card-similar-btn"
                :title="t.preview.findSimilar"
                @click.stop="emit('findSimilar', file)"
              >
                🔍
              </button>
              <img
                v-if="
                  file.container !== 'mp4' &&
                  file.container !== 'txt' &&
                  !failedImages.has(file.path)
                "
                :src="getCardImageSrc(file)"
                :alt="getFileName(file.path)"
                class="thumbnail-img"
                :class="{ 'nsfw-blurred': blurNsfw && file.is_nsfw && !revealedNsfw.has(file.path) }"
                loading="lazy"
                decoding="async"
                @error="onImageError(file.path)"
              />
              <video
                v-else-if="file.container === 'mp4'"
                :src="assetUrl(file.path)"
                class="thumbnail-img thumbnail-video"
                :class="{ 'nsfw-blurred': blurNsfw && file.is_nsfw && !revealedNsfw.has(file.path) }"
                muted
                preload="metadata"
                playsinline
              />
              <div v-else class="thumbnail-fallback">
                <span class="fallback-text">{{ file.container.toUpperCase() }}</span>
              </div>

              <!-- Video badge -->
              <span
                v-if="file.container === 'mp4'"
                class="card-badge badge-video"
                title="Video (MP4)"
              >
                ▶ MP4
              </span>

              <!-- NSFW blur overlay -->
              <div
                v-if="blurNsfw && file.is_nsfw && !revealedNsfw.has(file.path)"
                class="nsfw-overlay"
                :title="t.preview.clickToReveal"
                @click.stop="toggleNsfwReveal(file.path)"
              >
                <div class="nsfw-overlay-content">
                  <span class="nsfw-icon">🔞</span>
                  <span class="nsfw-text">NSFW</span>
                </div>
              </div>

              <!-- Badges Container -->
              <template v-if="showCardBadges">
                <!-- Favorite badge -->
                <span
                  v-if="file.is_favorite"
                  class="card-badge badge-fav"
                  title="Favorite"
                >
                  ★
                </span>

                <!-- NSFW badge -->
                <span
                  v-if="file.is_nsfw"
                  class="card-badge badge-nsfw"
                  title="18+ NSFW Content"
                >
                  18+
                </span>

                <!-- Format badge -->
                <span
                  v-if="file.metadata?.format"
                  class="card-badge badge-format"
                  :title="`Format: ${formatPlatformName(file.metadata.format)}`"
                >
                  {{ formatPlatformName(file.metadata.format) }}
                </span>

                <!-- Rating badge -->
                <span
                  v-if="file.rating"
                  class="card-badge badge-rating"
                  :title="`Rating: ${file.rating}/10`"
                >
                  ★ {{ file.rating }}
                </span>

                <!-- Similarity score badge -->
                <span
                  v-if="file.similarity_score !== undefined && file.similarity_score !== null"
                  class="card-badge badge-similarity"
                  :title="`${t.preview.similarityScore}: ${Math.round(file.similarity_score * 100)}%`"
                >
                  ⚡ {{ Math.round(file.similarity_score * 100) }}%
                </span>

                <!-- Stacking badge -->
                <button
                  v-if="file.stack_id && (stackMap?.[file.stack_id]?.count ?? 1) > 1"
                  type="button"
                  class="card-badge badge-stack"
                  :class="{ expanded: expandedStacks?.has(file.stack_id) }"
                  :title="t.stack.toggleExpand || 'Toggle Stack Expansion'"
                  @click.stop="emit('toggleStackExpand', file.stack_id)"
                >
                  📚 {{ stackMap?.[file.stack_id]?.count }}
                </button>

                <!-- Stack compare trigger button -->
                <button
                  v-if="file.stack_id && (stackMap?.[file.stack_id]?.count ?? 1) > 1"
                  type="button"
                  class="card-stack-compare-btn"
                  :title="t.compare.title || 'Compare Stack'"
                  @click.stop="emit('compareStack', file.stack_id)"
                >
                  ⚖️
                </button>
              </template>
            </div>

            <div class="card-info">
              <div class="card-title" :title="normalizePath(file.path)">
                {{ getFileName(file.path) }}
              </div>
              <div class="card-meta">
                <span>{{ formatDimensions(file) }}</span>
                <span class="card-container">{{ file.container.toUpperCase() }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.virtual-grid-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.virtual-grid-container {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  position: relative;
  outline: none;
  border-radius: 8px;
}

.virtual-grid-container:focus-visible {
  box-shadow: 0 0 0 2px rgba(47, 111, 237, 0.4);
}

.grid-placeholder {
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
  font-size: 0.9em;
}

.virtual-phantom {
  width: 100%;
  position: relative;
}

.virtual-content {
  display: grid;
  width: 100%;
  position: absolute;
  top: 0;
  left: 0;
}

.grid-card {
  display: flex;
  flex-direction: column;
  background: #fff;
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.15s ease;
  user-select: none;
}

@media (prefers-color-scheme: dark) {
  .grid-card {
    background: #252525;
    border-color: rgba(255, 255, 255, 0.12);
  }
}

.grid-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  border-color: rgba(47, 111, 237, 0.4);
}

/* Poker Deck Stack Shadow Effect */
.grid-card.is-stacked {
  box-shadow: 3px 3px 0 0 rgba(0, 0, 0, 0.12), 6px 6px 0 0 rgba(0, 0, 0, 0.06);
  border-color: rgba(47, 111, 237, 0.35);
}

@media (prefers-color-scheme: dark) {
  .grid-card.is-stacked {
    box-shadow: 3px 3px 0 0 rgba(255, 255, 255, 0.08), 6px 6px 0 0 rgba(255, 255, 255, 0.04);
    border-color: rgba(47, 111, 237, 0.45);
  }
}

.grid-card.is-stacked:hover {
  transform: translateY(-3px);
  box-shadow: 4px 4px 0 0 rgba(0, 0, 0, 0.15), 8px 8px 0 0 rgba(0, 0, 0, 0.08);
}

.grid-card.stack-expanded {
  border-style: dashed;
  border-color: #2f6fed;
}

.grid-card.active {
  border-color: #2f6fed;
  box-shadow: 0 0 0 2px #2f6fed, 0 4px 14px rgba(47, 111, 237, 0.25);
}

.grid-card.multi-selected {
  border-color: #2f6fed;
  background: rgba(47, 111, 237, 0.05);
}

.card-select-btn {
  position: absolute;
  top: 8px;
  left: 8px;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  border: 1px solid rgba(255, 255, 255, 0.6);
  background: rgba(0, 0, 0, 0.4);
  color: #fff;
  font-size: 0.8em;
  font-weight: bold;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 2;
  opacity: 0;
  transition: all 0.15s ease;
  padding: 0;
}

.grid-card:hover .card-select-btn,
.card-select-btn.checked {
  opacity: 1;
}

.card-select-btn.checked {
  background: #2f6fed;
  border-color: #2f6fed;
}

.thumbnail-wrapper {
  position: relative;
  width: 100%;
  aspect-ratio: 1 / 1;
  background: rgba(0, 0, 0, 0.04);
  overflow: hidden;
}

@media (prefers-color-scheme: dark) {
  .thumbnail-wrapper {
    background: rgba(0, 0, 0, 0.25);
  }
}

.thumbnail-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.thumbnail-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
}

.fallback-text {
  font-size: 0.85em;
  font-weight: 600;
  letter-spacing: 0.05em;
}

.card-badge {
  position: absolute;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-size: 0.7em;
  font-weight: 600;
  line-height: 1.2;
  backdrop-filter: blur(4px);
}

.badge-format {
  top: 6px;
  left: 6px;
  background: rgba(0, 0, 0, 0.65);
  color: #fff;
}

.badge-video {
  top: 6px;
  left: 6px;
  background: rgba(18, 181, 203, 0.85);
  color: #fff;
  font-weight: 700;
  letter-spacing: 0.05em;
}

.thumbnail-video {
  background: #000;
  pointer-events: none;
}

.badge-rating {
  top: 6px;
  right: 6px;
  background: rgba(234, 179, 8, 0.9);
  color: #000;
}

.badge-fav {
  bottom: 6px;
  right: 6px;
  background: rgba(234, 179, 8, 0.9);
  color: #000;
  font-size: 0.85em;
  padding: 0.05rem 0.35rem;
}

.badge-nsfw {
  bottom: 6px;
  left: 6px;
  background: rgba(220, 38, 38, 0.9);
  color: #fff;
  font-weight: 700;
  font-size: 0.65em;
  padding: 0.05rem 0.35rem;
}

.badge-similarity {
  top: 6px;
  right: 6px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  color: #fff;
  font-weight: 700;
  box-shadow: 0 2px 6px rgba(99, 102, 241, 0.4);
}

.badge-stack {
  top: 6px;
  right: 6px;
  background: rgba(30, 41, 59, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.2);
  color: #38bdf8;
  font-weight: 700;
  font-size: 0.72em;
  padding: 0.15rem 0.45rem;
  border-radius: 4px;
  cursor: pointer;
  z-index: 2;
  transition: all 0.15s ease;
}

.badge-stack:hover {
  background: #0284c7;
  color: #fff;
  border-color: #38bdf8;
}

.badge-stack.expanded {
  background: #2563eb;
  color: #fff;
  border-color: #60a5fa;
}

.card-stack-compare-btn {
  position: absolute;
  top: 6px;
  right: 64px;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.55);
  border: 1px solid rgba(255, 255, 255, 0.25);
  color: #fff;
  font-size: 0.75em;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  z-index: 2;
  transition: all 0.15s ease;
}

.grid-card:hover .card-stack-compare-btn {
  opacity: 1;
}

.card-stack-compare-btn:hover {
  background: #4f46e5;
  border-color: #818cf8;
}

.card-similar-btn {
  position: absolute;
  bottom: 8px;
  left: 8px;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(4px);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: #fff;
  font-size: 0.75em;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transform: scale(0.85);
  transition: opacity 0.15s ease, transform 0.15s ease, background 0.15s ease;
  z-index: 2;
}

.grid-card:hover .card-similar-btn {
  opacity: 1;
  transform: scale(1);
}

.card-similar-btn:hover {
  background: rgba(99, 102, 241, 0.9);
  border-color: rgba(255, 255, 255, 0.7);
}

.thumbnail-img.nsfw-blurred {
  filter: blur(24px) brightness(0.7);
  transform: scale(1.1);
  transition: filter 0.2s ease, transform 0.2s ease;
}

.nsfw-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  cursor: pointer;
  z-index: 1;
  transition: background 0.15s ease;
}

.nsfw-overlay:hover {
  background: rgba(0, 0, 0, 0.6);
}

.nsfw-overlay-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.2rem;
  color: #fff;
}

.nsfw-icon {
  font-size: 1.5em;
}

.nsfw-text {
  font-size: 0.72em;
  font-weight: 700;
  letter-spacing: 0.08em;
  background: #dc2626;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
}

.card-info {
  padding: 0.5rem 0.6rem;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  min-height: 56px;
  box-sizing: border-box;
}

.card-title {
  font-size: 0.8em;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: inherit;
}

.card-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.72em;
  color: #888;
}

.card-container {
  font-weight: 600;
  text-transform: uppercase;
  font-size: 0.9em;
}
</style>
