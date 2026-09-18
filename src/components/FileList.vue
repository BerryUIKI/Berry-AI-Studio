<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { ImageFile } from "../types";
import {
  assetUrl,
  formatBytes,
  formatDateTime,
  formatPlatformName,
  getFileName,
  normalizePath,
} from "../utils/image";
import {
  beginThumbnailRequestCycle,
  cancelThumbnailRequests,
  getThumbnailTier,
  getThumbnailUrl,
  getThumbnailUrlSync,
} from "../utils/thumbnail";
import { t } from "../i18n";

const props = defineProps<{
  files: ImageFile[];
  loading: boolean;
  loadingMore?: boolean;
  hasMore?: boolean;
  selectedFile?: ImageFile | null;
  selectedFilePaths?: Set<string>;
}>();

const emit = defineEmits<{
  (e: "select", file: ImageFile, event?: MouseEvent): void;
  (e: "activate", file: ImageFile): void;
  (e: "toggleSelect", file: ImageFile): void;
  (e: "toggleAll"): void;
  (e: "loadMore"): void;
}>();

const ROW_HEIGHT = 46;
const OVERSCAN = 6;
const ROW_THUMBNAIL_EDGE = 36;

const containerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerHeight = ref(600);
let scrollFrame: number | null = null;

const startRow = computed(() => {
  const raw = Math.floor(scrollTop.value / ROW_HEIGHT);
  return Math.max(0, raw - OVERSCAN);
});

const endRow = computed(() => {
  const visibleCount = Math.ceil(containerHeight.value / ROW_HEIGHT);
  const raw = startRow.value + visibleCount + OVERSCAN * 2;
  return Math.min(props.files.length - 1, raw);
});

const visibleFiles = computed(() => {
  if (props.files.length === 0) return [];
  return props.files.slice(startRow.value, endRow.value + 1);
});

const topSpacerHeight = computed(() => startRow.value * ROW_HEIGHT);
const bottomSpacerHeight = computed(() =>
  Math.max(0, (props.files.length - (endRow.value + 1)) * ROW_HEIGHT),
);

function onScroll(e: Event) {
  const target = e.target as HTMLElement;
  if (scrollFrame !== null) return;
  scrollFrame = requestAnimationFrame(() => {
    scrollTop.value = target.scrollTop;
    scrollFrame = null;
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

function updateHeight() {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight || 600;
  }
}

// Keep the table reactive without duplicating the shared bounded thumbnail LRU.
const thumbnailRevision = ref(0);

// Fast sync or async lookup for row image
function getRowImageSrc(file: ImageFile): string | null {
  void thumbnailRevision.value;
  const syncCached = getThumbnailUrlSync(file, getThumbnailTier(ROW_THUMBNAIL_EDGE));
  if (syncCached) return syncCached;
  return file.id ? null : assetUrl(file.path);
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
          .catch(() => {
            // The row left the visible window before decoding began.
          });
      }
    }
  },
  { immediate: true },
);

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  updateHeight();
  if (containerRef.value && typeof ResizeObserver !== "undefined") {
    resizeObserver = new ResizeObserver(() => {
      updateHeight();
    });
    resizeObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  if (scrollFrame !== null) cancelAnimationFrame(scrollFrame);
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
    <p v-else-if="!files.length" class="empty">{{ t.view.selectFolderPrompt }}</p>

    <div v-else ref="containerRef" class="scroll" @scroll.passive="onScroll">
      <table class="table">
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
                  file.container !== 'mp4' &&
                  file.container !== 'txt' &&
                  getRowImageSrc(file)
                "
                :src="getRowImageSrc(file) || undefined"
                :alt="getFileName(file.path)"
                class="thumb"
                loading="lazy"
                decoding="async"
              />
              <div
                v-else-if="file.container !== 'mp4' && file.container !== 'txt'"
                class="thumb-placeholder thumb-pending"
                aria-hidden="true"
              />
              <video
                v-else-if="file.container === 'mp4'"
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
  color: #71717a;
  font-size: 0.85em;
  padding: 2rem;
  text-align: center;
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
