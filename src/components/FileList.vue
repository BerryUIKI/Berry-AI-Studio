<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import type { ImageFile } from "../types";
import {
  assetUrl,
  formatBytes,
  formatDateTime,
  formatPlatformName,
  getFileName,
  isVideoContainer,
  normalizePath,
} from "../utils/image";
import {
  beginThumbnailRequestCycle,
  cancelThumbnailRequests,
  getThumbnailTier,
  getThumbnailUrl,
  getThumbnailUrlSync,
  invalidateThumbnail,
} from "../utils/thumbnail";
import { t } from "../i18n";
import { useGalleryNavigation } from "../utils/gallery-navigation";

const props = defineProps<{
  files: ImageFile[];
  loading: boolean;
  loadingMore?: boolean;
  hasMore?: boolean;
  selectedFile?: ImageFile | null;
  selectedFilePaths?: Set<string>;
  contextKey?: string;
  fileRevision?: number;
  emptyMessage?: string;
  emptyActionText?: string;
}>();

const emit = defineEmits<{
  (e: "select", file: ImageFile, event?: MouseEvent): void;
  (e: "activate", file: ImageFile): void;
  (e: "toggleSelect", file: ImageFile): void;
  (e: "toggleAll"): void;
  (e: "loadMore"): void;
  (e: "recover"): void;
}>();

const ROW_HEIGHT = 46;
const OVERSCAN = 6;
const ROW_THUMBNAIL_EDGE = 36;

const containerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerHeight = ref(0);
let scrollFrame: number | null = null;
let resizeFrame: number | null = null;

const navigation = useGalleryNavigation({
  element: containerRef,
  key: () => props.contextKey ?? "all",
  files: () => props.files,
  revision: () => props.fileRevision ?? 0,
  loading: () => props.loading,
  hasMore: () => !!props.hasMore,
  loadingMore: () => !!props.loadingMore,
  top: (index) => index * ROW_HEIGHT,
  itemHeight: () => ROW_HEIGHT,
  firstVisible: () => Math.floor(scrollTop.value / ROW_HEIGHT),
  loadMore: () => emit("loadMore"),
  onRestore: (top) => {
    scrollTop.value = top;
  },
});

const startRow = computed(() => {
  const raw = Math.floor(scrollTop.value / ROW_HEIGHT);
  return Math.max(0, raw - OVERSCAN);
});

const endRow = computed(() => {
  void props.fileRevision;
  const visibleCount = Math.ceil(containerHeight.value / ROW_HEIGHT);
  const raw = startRow.value + visibleCount + OVERSCAN * 2;
  return Math.min(props.files.length - 1, raw);
});

const visibleFiles = computed(() => {
  void props.fileRevision;
  if (props.files.length === 0) return [];
  return props.files.slice(startRow.value, endRow.value + 1);
});

const topSpacerHeight = computed(() => startRow.value * ROW_HEIGHT);
const bottomSpacerHeight = computed(() => {
  void props.fileRevision;
  return Math.max(0, (props.files.length - (endRow.value + 1)) * ROW_HEIGHT);
});

function onScroll(e: Event) {
  const target = e.target as HTMLElement;
  if (scrollFrame !== null) return;
  scrollFrame = requestAnimationFrame(() => {
    scrollTop.value = target.scrollTop;
    scrollFrame = null;
    navigation.save();
    maybeRequestMore();
  });
}

function maybeRequestMore() {
  if (!props.hasMore || props.loading || props.loadingMore) return;
  const totalHeight = props.files.length * ROW_HEIGHT;
  if (totalHeight - scrollTop.value - containerHeight.value <= containerHeight.value * 2) {
    emit("loadMore");
  }
}

watch(
  [() => props.files.length, containerHeight, () => props.hasMore, () => props.loadingMore],
  () => queueMicrotask(maybeRequestMore),
);

// Keep the table reactive without duplicating the shared bounded thumbnail LRU.
const thumbnailRevision = ref(0);
const failedImages = ref<Set<string>>(new Set());

function onImageError(path: string) {
  failedImages.value.add(path);
}

function retryImage(file: ImageFile) {
  invalidateThumbnail(file, getThumbnailTier(ROW_THUMBNAIL_EDGE));
  failedImages.value.delete(file.path);
  const generation = beginThumbnailRequestCycle();
  void getThumbnailUrl(file, getThumbnailTier(ROW_THUMBNAIL_EDGE), generation)
    .then(() => {
      thumbnailRevision.value += 1;
    })
    .catch((error) => {
      if (!String(error).includes("thumbnail request canceled")) {
        failedImages.value.add(file.path);
      }
    });
}

// Fast sync or async lookup for row image
function getRowImageSrc(file: ImageFile): string | null {
  void thumbnailRevision.value;
  if (failedImages.value.has(file.path)) return null;
  const syncCached = getThumbnailUrlSync(file, getThumbnailTier(ROW_THUMBNAIL_EDGE));
  if (syncCached) return syncCached;
  return null;
}

// Prefetch thumbnails for visible rows
watch(
  visibleFiles,
  (batch) => {
    const generation = beginThumbnailRequestCycle();
    if (!batch || batch.length === 0) return;
    for (const file of batch) {
      if (
        file.container !== "mp4" && file.container !== "txt" &&
        !getThumbnailUrlSync(file, getThumbnailTier(ROW_THUMBNAIL_EDGE))
      ) {
        void getThumbnailUrl(file, getThumbnailTier(ROW_THUMBNAIL_EDGE), generation)
          .then(() => {
            thumbnailRevision.value += 1;
          })
          .catch((error) => {
            if (!String(error).includes("thumbnail request canceled") && !isVideoContainer(file.container)) {
              failedImages.value.add(file.path);
            }
          });
      }
    }
  },
  { immediate: true },
);

let resizeObserver: ResizeObserver | null = null;

watch(
  containerRef,
  (element) => {
    resizeObserver?.disconnect();
    resizeObserver = null;
    if (resizeFrame !== null) {
      cancelAnimationFrame(resizeFrame);
      resizeFrame = null;
    }
    if (!element) return;

    const updateHeight = (height: number) => {
      if (resizeFrame !== null) cancelAnimationFrame(resizeFrame);
      resizeFrame = requestAnimationFrame(() => {
        resizeFrame = null;
        if (containerRef.value === element) {
          containerHeight.value = Math.max(0, height);
        }
      });
    };
    updateHeight(element.clientHeight);
    if (typeof ResizeObserver === "undefined") return;
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries.find((candidate) => candidate.target === element);
      if (entry) updateHeight(entry.contentRect.height);
    });
    resizeObserver.observe(element);
  },
  { immediate: true, flush: "post" },
);

onUnmounted(() => {
  if (scrollFrame !== null) cancelAnimationFrame(scrollFrame);
  if (resizeFrame !== null) cancelAnimationFrame(resizeFrame);
  if (resizeObserver) {
    resizeObserver.disconnect();
    resizeObserver = null;
  }
  cancelThumbnailRequests();
});

function snippet(text: string | null | undefined, max = 48): string {
  if (!text) return "—";
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

function size(meta: ImageFile["metadata"]): string {
  if (!meta?.width || !meta?.height) return "—";
  return `${meta.width} × ${meta.height}`;
}
</script>

<template>
  <section class="files">
    <p v-if="loading" class="empty">{{ t.view.loading }}</p>
    <div v-else-if="!files.length" class="empty empty-state">
      <span class="empty-state-message">{{ emptyMessage || t.review.noMatches }}</span>
      <button
        type="button"
        class="empty-state-btn"
        @click="emit('recover')"
      >
        {{ emptyActionText || t.review.retry }}
      </button>
    </div>

    <div v-else ref="containerRef" class="scroll" @scroll.passive="onScroll">
      <table class="table" role="table" :aria-label="t.review.gallery">
        <thead class="sticky-header">
          <tr>
            <th class="th-checkbox">
              <input
                type="checkbox"
                :checked="files.length > 0 && selectedFilePaths?.size === files.length"
                :title="t.view.selectAll"
                @click.stop="emit('toggleAll')"
              />
            </th>
            <th class="th-preview">{{ t.preview.preview }}</th>
            <th>{{ t.sort.name }}</th>
            <th>{{ t.preview.container }}</th>
            <th>{{ t.sort.size }}</th>
            <th>{{ t.sort.modified }}</th>
            <th>{{ t.preview.platform }}</th>
            <th>{{ t.preview.prompt }}</th>
            <th>{{ t.preview.dimensions }}</th>
            <th>{{ t.preview.modelName }}</th>
          </tr>
        </thead>
        <tbody>
          <!-- Top Spacer for Virtualization -->
          <tr v-if="topSpacerHeight > 0" :style="{ height: `${topSpacerHeight}px` }" class="spacer-row">
            <td colspan="10" class="spacer-cell"></td>
          </tr>

          <!-- Visible Rows -->
          <tr
            v-for="file in visibleFiles"
            :key="file.id ?? file.path"
            class="data-row"
            :class="{
              'row-selected': selectedFile?.path === file.path,
              'row-multi-selected': selectedFilePaths?.has(file.path),
            }"
            @click="emit('select', file, $event)"
            @dblclick="emit('activate', file)"
          >
            <td class="td-checkbox">
              <input
                type="checkbox"
                :checked="selectedFilePaths?.has(file.path)"
                @click.stop="emit('toggleSelect', file)"
              />
            </td>
            <td class="preview-cell">
              <img
                v-if="
                  !isVideoContainer(file.container) &&
                  file.container !== 'txt' &&
                  getRowImageSrc(file)
                "
                :src="getRowImageSrc(file) || undefined"
                :alt="getFileName(file.path)"
                class="thumb"
                loading="lazy"
                decoding="async"
                @error="onImageError(file.path)"
              />
              <div
                v-else-if="failedImages.has(file.path)"
                class="thumb-placeholder thumb-failed"
              >
                <button
                  type="button"
                  class="retry-thumb-btn table-retry-btn"
                  :title="t.preview.retryThumbnail"
                  :aria-label="t.preview.retryThumbnail"
                  @click.stop="retryImage(file)"
                >
                  ↻
                </button>
              </div>
              <div
                v-else-if="!isVideoContainer(file.container) && file.container !== 'txt'"
                class="thumb-placeholder thumb-pending"
                aria-hidden="true"
              />
              <video
                v-else-if="isVideoContainer(file.container)"
                :src="assetUrl(file.path)"
                class="thumb thumb-video"
                muted
                preload="metadata"
                playsinline
              />
              <div v-else class="thumb-placeholder">
                {{ file.container.toUpperCase() }}
              </div>
            </td>
            <td class="name" :title="normalizePath(file.path)">{{ getFileName(file.path) }}</td>
            <td>{{ file.container }}</td>
            <td>{{ formatBytes(file.size_bytes) }}</td>
            <td class="date">{{ formatDateTime(file.modified_at) }}</td>
            <td>
              <span v-if="file.metadata" class="format">{{ formatPlatformName(file.metadata.format) }}</span>
              <span v-else class="none">—</span>
            </td>
            <td class="prompt" :title="file.metadata?.prompt ?? ''">
              {{ snippet(file.metadata?.prompt, 80) }}
            </td>
            <td class="nowrap">{{ size(file.metadata) }}</td>
            <td class="model" :title="file.metadata?.model_name ?? ''">
              {{ snippet(file.metadata?.model_name, 32) }}
            </td>
          </tr>

          <!-- Bottom Spacer for Virtualization -->
          <tr v-if="bottomSpacerHeight > 0" :style="{ height: `${bottomSpacerHeight}px` }" class="spacer-row">
            <td colspan="10" class="spacer-cell"></td>
          </tr>
        </tbody>
      </table>
      <div v-if="loadingMore" class="load-more-indicator" role="status">
        {{ t.view.loading }}
      </div>
    </div>
  </section>
</template>

<style scoped>
.files {
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.empty {
  color: var(--color-text-secondary);
  font-size: 0.85em;
  padding: 2rem;
  text-align: center;
}

.empty.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.empty-state-message {
  font-size: 0.95rem;
  color: var(--color-text-secondary);
}

.empty-state-btn {
  padding: 6px 16px;
  font-size: 0.85rem;
  font-weight: 500;
  border-radius: 6px;
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.empty-state-btn:hover {
  background: var(--color-bg-hover);
  border-color: var(--color-primary);
}

.scroll {
  flex: 1;
  overflow-y: auto;
  overflow-x: auto;
  width: 100%;
  height: 100%;
  position: relative;
}

.table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.82em;
}

.sticky-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background: #18181c;
  box-shadow: 0 1px 0 rgba(255, 255, 255, 0.08);
}

.table th,
.table td {
  text-align: left;
  padding: 0.4rem 0.6rem;
  border-bottom: 1px solid rgba(128, 128, 128, 0.15);
  white-space: nowrap;
}

.table th {
  color: #888;
  font-weight: 600;
  font-size: 0.8em;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.spacer-row {
  border: none !important;
  background: transparent !important;
}

.spacer-cell {
  padding: 0 !important;
  border: none !important;
  height: inherit;
}

.load-more-indicator {
  position: sticky;
  left: 50%;
  bottom: 12px;
  width: max-content;
  margin: 0 auto;
  padding: 5px 12px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--color-bg-tertiary) 88%, transparent);
  color: var(--color-text-secondary);
  font-size: 0.72rem;
  pointer-events: none;
}

.data-row {
  height: 46px;
  box-sizing: border-box;
}

.th-preview {
  width: 44px;
}

.preview-cell {
  width: 44px;
  padding: 0.25rem 0.4rem !important;
}

.thumb {
  width: 36px;
  height: 36px;
  object-fit: cover;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.1);
  display: block;
}

.thumb-video {
  background: #000;
  pointer-events: none;
}

.thumb-placeholder {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.7em;
  font-weight: 600;
  color: #999;
  background: rgba(0, 0, 0, 0.04);
  border-radius: 4px;
}

.thumb-failed {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(239, 68, 68, 0.12);
  border: 1px dashed rgba(239, 68, 68, 0.4);
}

.table-retry-btn {
  width: 22px;
  height: 22px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.3);
  color: #fff;
  font-size: 0.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.table-retry-btn:hover {
  background: var(--color-primary);
  border-color: var(--color-primary-hover);
}

.thumb-pending {
  background: linear-gradient(
    110deg,
    rgba(255, 255, 255, 0.02) 25%,
    rgba(255, 255, 255, 0.1) 45%,
    rgba(255, 255, 255, 0.02) 65%
  );
  background-size: 220% 100%;
  animation: table-thumbnail-queue-pulse 1.2s ease-in-out infinite;
}

@keyframes table-thumbnail-queue-pulse {
  from { background-position: 100% 0; }
  to { background-position: -100% 0; }
}

@media (prefers-reduced-motion: reduce) {
  .thumb-pending {
    animation: none;
    background: rgba(255, 255, 255, 0.04);
  }
}

.name {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  word-break: break-all;
  max-width: 22rem;
  overflow: hidden;
  text-overflow: ellipsis;
}

.date {
  font-size: 0.85em;
  color: #888;
}

.prompt {
  max-width: 24rem;
  overflow: hidden;
  text-overflow: ellipsis;
}

.model {
  max-width: 16rem;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 0.85em;
}

.format {
  display: inline-block;
  padding: 0.05rem 0.5rem;
  border-radius: 999px;
  font-size: 0.75em;
  font-weight: 600;
  background: rgba(47, 111, 237, 0.15);
  color: #2f6fed;
}

.none {
  color: #999;
}

tbody tr.data-row {
  cursor: pointer;
  transition: background-color 0.12s ease;
}

tbody tr.data-row:hover {
  background: rgba(255, 255, 255, 0.04);
}

tbody tr.row-selected {
  background: rgba(47, 111, 237, 0.18) !important;
}

tbody tr.row-multi-selected {
  background: rgba(47, 111, 237, 0.08);
}

.th-checkbox,
.td-checkbox {
  width: 32px;
  text-align: center !important;
  padding: 0.4rem 0.2rem !important;
}

.th-checkbox input,
.td-checkbox input {
  cursor: pointer;
  accent-color: #2f6fed;
}
</style>
