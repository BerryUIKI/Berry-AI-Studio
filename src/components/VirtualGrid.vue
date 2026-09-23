<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { ImageFile } from "../types";
import {
  assetUrl,
  formatBytes,
  formatDuration,
  formatPlatformName,
  getFileName,
  isVideoContainer,
  normalizePath,
} from "../utils/image";
import {
  beginThumbnailRequestCycle,
  cancelThumbnailRequests,
  captureAndSaveVideoThumbnail,
  getThumbnailTier,
  getThumbnailUrl,
  getThumbnailUrlSync,
  requestBatchThumbnails,
  THUMBNAIL_PRIORITY,
} from "../utils/thumbnail";
import { t } from "../i18n";
import { resolveStackHeroPaths } from "../utils/stack";
import { calculateGalleryColumns, calculateGalleryTrackOffset } from "../utils/gallery-layout";
import { hasActiveDialog, isEditableTarget } from "../utils/dialog";
import { useGalleryNavigation } from "../utils/gallery-navigation";

const props = withDefaults(
  defineProps<{
    files: ImageFile[];
    selectedFile?: ImageFile | null;
    selectedFilePaths?: Set<string>;
    loading?: boolean;
    loadingMore?: boolean;
    hasMore?: boolean;
    itemMinWidth?: number;
    gap?: number;
    overscan?: number;
    blurNsfw?: boolean;
    showCardBadges?: boolean;
    stackMap?: Record<string, { count: number; heroId: number | null }>;
    expandedStacks?: Set<string>;
    layout?: "grid" | "masonry";
    contextKey?: string;
    fileRevision?: number;
    emptyMessage?: string;
    emptyActionText?: string;
  }>(),
  {
    selectedFile: null,
    loading: false,
    loadingMore: false,
    hasMore: false,
    itemMinWidth: 180,
    gap: 16,
    overscan: 4,
    blurNsfw: true,
    showCardBadges: true,
    layout: "grid",
  },
);

const emit = defineEmits<{
  (e: "select", file: ImageFile, event?: MouseEvent): void;
  (e: "activate", file: ImageFile): void;
  (e: "toggleSelect", file: ImageFile): void;
  (e: "findSimilar", file: ImageFile): void;
  (e: "toggleStackExpand", stackId: string): void;
  (e: "compareStack", stackId: string): void;
  (e: "cullStack", stackId: string): void;
  (e: "loadMore"): void;
  (e: "recover"): void;
}>();

const containerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerWidth = ref(0);
const containerHeight = ref(0);

// Image loading error tracker
const failedImages = ref<Set<string>>(new Set());
const revealedNsfw = ref<Set<string>>(new Set());

function onImageError(path: string) {
  failedImages.value.add(path);
}

function retryImage(file: ImageFile) {
  failedImages.value.delete(file.path);
  getThumbnailUrl(file, Math.max(itemWidth.value, rowHeight.value)).catch(() => {});
}


function toggleNsfwReveal(path: string) {
  if (revealedNsfw.value.has(path)) {
    revealedNsfw.value.delete(path);
  } else {
    revealedNsfw.value.add(path);
  }
}

let resizeObserver: ResizeObserver | null = null;
let resizeFrame: number | null = null;
let pendingSize: { element: HTMLElement; width: number; height: number } | null = null;
let scrollFrame: number | null = null;
const stackClickTimers = new Map<string, ReturnType<typeof setTimeout>>();
const STACK_CLICK_DELAY_MS = 240;

function scheduleDimensionUpdate(element: HTMLElement, width: number, height: number) {
  pendingSize = { element, width, height };
  if (resizeFrame !== null) return;
  resizeFrame = requestAnimationFrame(() => {
    resizeFrame = null;
    const size = pendingSize;
    pendingSize = null;
    if (!size || containerRef.value !== size.element) return;
    containerWidth.value = Math.max(0, size.width);
    containerHeight.value = Math.max(0, size.height);
  });
}

watch(
  containerRef,
  (element) => {
    resizeObserver?.disconnect();
    resizeObserver = null;
    pendingSize = null;
    if (resizeFrame !== null) {
      cancelAnimationFrame(resizeFrame);
      resizeFrame = null;
    }
    if (!element) return;

    scheduleDimensionUpdate(element, element.clientWidth, element.clientHeight);
    if (typeof ResizeObserver === "undefined") return;
    resizeObserver = new ResizeObserver((entries) => {
      const entry = entries.find((candidate) => candidate.target === element);
      if (!entry) return;
      scheduleDimensionUpdate(element, entry.contentRect.width, entry.contentRect.height);
    });
    resizeObserver.observe(element);
  },
  { immediate: true, flush: "post" },
);

onMounted(() => {
  window.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
  if (resizeFrame !== null) cancelAnimationFrame(resizeFrame);
  if (prefetchDebounceTimer) clearTimeout(prefetchDebounceTimer);
  if (scrollFrame !== null) cancelAnimationFrame(scrollFrame);
  for (const timer of stackClickTimers.values()) clearTimeout(timer);
  stackClickTimers.clear();
  resizeObserver?.disconnect();
  window.removeEventListener("keydown", handleKeyDown);
  cancelThumbnailRequests();
});

function onScroll(e: Event) {
  const target = e.target as HTMLElement;
  // Scrollbar dragging can dispatch hundreds of events per second. Collapse
  // them to one reactive gallery update per animation frame.
  if (scrollFrame !== null) return;
  scrollFrame = requestAnimationFrame(() => {
    scrollTop.value = target.scrollTop;
    scrollFrame = null;
    navigation.save();
    maybeRequestMore();
  });
}

// Columns count based on container width
const cols = computed(() => {
  return calculateGalleryColumns(containerWidth.value, props.itemMinWidth, props.gap);
});

// Calculate symmetrical horizontal offset to center columns and eliminate right-hand whitespace gaps
const horizontalOffset = computed(() => {
  return calculateGalleryTrackOffset(containerWidth.value, cols.value, itemWidth.value, props.gap);
});

// Keep the user's chosen card width stable. Resizing the window changes the
// number of columns, not the image size (matching Eagle's gallery behavior).
const itemWidth = computed(() => {
  if (containerWidth.value <= 0) return props.itemMinWidth;
  return Math.max(1, Math.min(props.itemMinWidth, containerWidth.value));
});

// Card height = 1:1 square image + 56px info footer
const CARD_INFO_HEIGHT = 56;
const cardHeight = computed(() => itemWidth.value + CARD_INFO_HEIGHT);
const rowHeight = computed(() => cardHeight.value + props.gap);

interface MasonryItem {
  file: ImageFile;
  index: number;
  top: number;
  left: number;
  width: number;
  height: number;
  imageHeight: number;
  column: number;
}

const masonryItems = computed<MasonryItem[]>(() => {
  if (props.layout !== "masonry") return [];
  const columnHeights = Array.from({ length: cols.value }, () => 0);
  return props.files.map((file, index) => {
    const column = columnHeights.indexOf(Math.min(...columnHeights));
    const sourceWidth = file.metadata?.width ?? 1;
    const sourceHeight = file.metadata?.height ?? 1;
    const ratio = sourceWidth > 0 && sourceHeight > 0 ? sourceHeight / sourceWidth : 1;
    const imageHeight = Math.max(96, Math.round(itemWidth.value * ratio));
    const height = imageHeight + CARD_INFO_HEIGHT;
    const item = {
      file,
      index,
      top: columnHeights[column],
      left: horizontalOffset.value + column * (itemWidth.value + props.gap),
      width: itemWidth.value,
      height,
      imageHeight,
      column,
    };
    columnHeights[column] += height + props.gap;
    return item;
  });
});

const masonryColumns = computed(() => {
  const columns = Array.from({ length: cols.value }, () => [] as MasonryItem[]);
  for (const item of masonryItems.value) columns[item.column].push(item);
  return columns;
});

const masonryHeight = computed(() => {
  if (!masonryItems.value.length) return 0;
  return Math.max(...masonryItems.value.map((item) => item.top + item.height));
});

// Total grid rows and phantom scroll height
const totalRows = computed(() => Math.ceil(props.files.length / cols.value));
const totalHeight = computed(() => {
  if (props.layout === "masonry") return masonryHeight.value;
  if (totalRows.value === 0) return 0;
  return totalRows.value * rowHeight.value - props.gap;
});

function maybeRequestMore() {
  if (!props.hasMore || props.loading || props.loadingMore) return;
  const threshold = Math.max(800, containerHeight.value * 2);
  if (totalHeight.value - scrollTop.value - containerHeight.value <= threshold) {
    emit("loadMore");
  }
}

watch(
  [totalHeight, containerHeight, () => props.hasMore, () => props.loadingMore],
  () => queueMicrotask(maybeRequestMore),
);

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

const visibleMasonryItems = computed(() => {
  if (props.layout !== "masonry") return [];
  const buffer = Math.max(itemWidth.value, props.overscan * 100);
  const top = Math.max(0, scrollTop.value - buffer);
  const bottom = scrollTop.value + containerHeight.value + buffer;
  const visible: MasonryItem[] = [];

  for (const column of masonryColumns.value) {
    let low = 0;
    let high = column.length;
    while (low < high) {
      const mid = (low + high) >>> 1;
      if (column[mid].top + column[mid].height < top) low = mid + 1;
      else high = mid;
    }
    for (let index = low; index < column.length; index += 1) {
      const item = column[index];
      if (item.top > bottom) break;
      visible.push(item);
    }
  }

  return visible.sort((a, b) => a.index - b.index);
});

const visibleItems = computed(() => {
  if (props.layout === "masonry") return visibleMasonryItems.value;
  return visibleFiles.value.map((file, offset) => ({
    file,
    index: startIndex.value + offset,
    top: 0,
    left: 0,
    width: 0,
    height: cardHeight.value,
    imageHeight: itemWidth.value,
    column: 0,
  }));
});

const translateY = computed(() => startRow.value * rowHeight.value);

const navigation = useGalleryNavigation({
  element: containerRef,
  key: () => props.contextKey ?? "all",
  files: () => props.files,
  revision: () => props.fileRevision ?? 0,
  loading: () => props.loading,
  hasMore: () => props.hasMore,
  loadingMore: () => props.loadingMore,
  top: (index) => {
    if (props.layout === "masonry") {
      return masonryItems.value[index]?.top ?? 0;
    }
    return Math.floor(index / cols.value) * rowHeight.value;
  },
  itemHeight: (index) => {
    if (props.layout === "masonry") {
      return masonryItems.value[index]?.height ?? cardHeight.value;
    }
    return cardHeight.value;
  },
  firstVisible: () => {
    if (props.layout === "masonry") {
      return (
        visibleMasonryItems.value.find(
          (item) => item.top + item.height >= scrollTop.value,
        )?.index ?? 0
      );
    }
    return Math.floor(scrollTop.value / rowHeight.value) * cols.value;
  },
  loadMore: () => emit("loadMore"),
  onRestore: (top) => {
    scrollTop.value = top;
  },
});

// A revision signal keeps rendering reactive without retaining a second,
// unbounded URL map beside the shared LRU thumbnail cache.
const thumbnailRevision = ref(0);

function getCardImageSrc(file: ImageFile, displayEdge: number): string | null {
  void thumbnailRevision.value;
  if (!file.id) return assetUrl(file.path);
  return getThumbnailUrlSync(file, getThumbnailTier(displayEdge));
}

async function loadThumbnailFor(file: ImageFile, displayEdge: number, generation: number) {
  const thumbnailTier = getThumbnailTier(displayEdge);
  if (!file.id || getThumbnailUrlSync(file, thumbnailTier)) return;
  try {
    await getThumbnailUrl(file, thumbnailTier, generation);
    thumbnailRevision.value += 1;
  } catch {
    // The viewport moved before this queued request began decoding.
  }
}

let prefetchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  visibleItems,
  (items) => {
    if (prefetchDebounceTimer) clearTimeout(prefetchDebounceTimer);
    const generation = beginThumbnailRequestCycle();
    const files = items.map((item) => item.file);
    if (!files || files.length === 0) return;

    // Resolve visible thumbnails immediately. Do not also submit them to the
    // batch worker: that used to decode the same image twice during fast scroll.
    for (const item of items) {
      const thumbnailTier = getThumbnailTier(Math.max(item.width || itemWidth.value, item.imageHeight));
      if (item.file.id && !getThumbnailUrlSync(item.file, thumbnailTier)) {
        void loadThumbnailFor(item.file, Math.max(item.width || itemWidth.value, item.imageHeight), generation);
      }
    }
    // Prefetch only after scrolling settles. This avoids building an I/O queue
    // for every intermediate position while the scrollbar thumb is dragged.
    prefetchDebounceTimer = setTimeout(() => {
      const firstVisibleIndex = items[0]?.index ?? 0;
      const lastVisibleIndex = items[items.length - 1]?.index ?? 0;
      const aheadStart = lastVisibleIndex + 1;
      const aheadEnd = Math.min(props.files.length, lastVisibleIndex + 101);
      if (aheadStart < aheadEnd) {
        const aheadSlice = props.files.slice(aheadStart, aheadEnd);
        void requestBatchThumbnails(aheadSlice, getThumbnailTier(itemWidth.value), {
          generation,
          priority: THUMBNAIL_PRIORITY.NEAR_LOOKAHEAD,
        });
      }
      const behindStart = Math.max(0, firstVisibleIndex - 40);
      if (behindStart < firstVisibleIndex) {
        const behindSlice = props.files.slice(behindStart, firstVisibleIndex);
        void requestBatchThumbnails(behindSlice, getThumbnailTier(itemWidth.value), {
          generation,
          priority: THUMBNAIL_PRIORITY.FAR_LOOKAHEAD,
        });
      }
    }, 220);
  },
  { immediate: true },
);

function selectFile(file: ImageFile, event?: MouseEvent) {
  emit("select", file, event);
}

function toggleSelect(file: ImageFile) {
  emit("toggleSelect", file);
}

// Video hover scrubbing state
const hoveredVideoPath = ref<string | null>(null);
const hoveredVideoProgress = ref<number>(0);
const hoveredVideoCurrentTime = ref<number>(0);
const hoveredVideoDuration = ref<number>(0);
let videoHoverTimer: ReturnType<typeof setTimeout> | null = null;

function onVideoMouseEnter(file: ImageFile) {
  if (!isVideoContainer(file.container)) return;
  if (videoHoverTimer) clearTimeout(videoHoverTimer);
  videoHoverTimer = setTimeout(() => {
    hoveredVideoPath.value = file.path;
    hoveredVideoProgress.value = 0;
    hoveredVideoCurrentTime.value = 0;
    hoveredVideoDuration.value = file.metadata?.duration_seconds || 0;
  }, 60);
}

function onVideoMouseMove(e: MouseEvent, file: ImageFile) {
  if (!isVideoContainer(file.container) || hoveredVideoPath.value !== file.path) return;
  const target = e.currentTarget as HTMLElement | null;
  if (!target) return;
  const rect = target.getBoundingClientRect();
  const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / (rect.width || 1)));
  hoveredVideoProgress.value = ratio;

  const videoEl = target.querySelector("video") as HTMLVideoElement | null;
  if (videoEl && videoEl.duration && !isNaN(videoEl.duration)) {
    videoEl.currentTime = ratio * videoEl.duration;
    hoveredVideoCurrentTime.value = videoEl.currentTime;
    hoveredVideoDuration.value = videoEl.duration;
  } else if (file.metadata?.duration_seconds) {
    hoveredVideoCurrentTime.value = ratio * file.metadata.duration_seconds;
    hoveredVideoDuration.value = file.metadata.duration_seconds;
  }
}

function onVideoMouseLeave() {
  if (videoHoverTimer) clearTimeout(videoHoverTimer);
  hoveredVideoPath.value = null;
  hoveredVideoProgress.value = 0;
  hoveredVideoCurrentTime.value = 0;
}

function onVideoLoadedData(e: Event, file: ImageFile) {
  const video = e.target as HTMLVideoElement | null;
  if (!video) return;
  void captureAndSaveVideoThumbnail(file, video);
}

function isStacked(file: ImageFile): boolean {
  return Boolean(file.stack_id && (props.stackMap?.[file.stack_id]?.count ?? 1) > 1);
}

function isStackExpanded(file: ImageFile): boolean {
  return Boolean(file.stack_id && props.expandedStacks?.has(file.stack_id));
}

function isCollapsedStack(file: ImageFile): boolean {
  return isStacked(file) && !isStackExpanded(file);
}

const stackHeroPaths = computed(() => resolveStackHeroPaths(props.files, props.stackMap ?? {}));

function isStackCover(file: ImageFile): file is ImageFile & { stack_id: string } {
  return Boolean(
    file.stack_id &&
    isStacked(file) &&
    stackHeroPaths.value.get(file.stack_id) === file.path,
  );
}

function onCardClick(file: ImageFile, event: MouseEvent) {
  if (file.stack_id && isStackCover(file) && !event.ctrlKey && !event.metaKey && !event.shiftKey) {
    const existingTimer = stackClickTimers.get(file.stack_id);
    if (existingTimer) {
      clearTimeout(existingTimer);
      stackClickTimers.delete(file.stack_id!);
      return;
    }
    stackClickTimers.set(file.stack_id, setTimeout(() => {
      stackClickTimers.delete(file.stack_id!);
      emit("toggleStackExpand", file.stack_id!);
    }, STACK_CLICK_DELAY_MS));
    return;
  }
  selectFile(file, event);
}

function activateFile(file: ImageFile) {
  if (file.stack_id) {
    const pendingToggle = stackClickTimers.get(file.stack_id);
    if (pendingToggle) {
      clearTimeout(pendingToggle);
      stackClickTimers.delete(file.stack_id);
    }
  }
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
  if (
    e.defaultPrevented ||
    hasActiveDialog() ||
    isEditableTarget(e.target) ||
    isEditableTarget(document.activeElement)
  ) {
    return;
  }

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
  const masonryItem = props.layout === "masonry" ? masonryItems.value[index] : null;
  const targetRow = Math.floor(index / cols.value);
  const targetTop = masonryItem?.top ?? targetRow * rowHeight.value;
  const targetBottom = targetTop + (masonryItem?.height ?? cardHeight.value);

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
    <div v-else-if="!files.length" class="grid-placeholder empty-state">
      <div class="empty-state-message">{{ emptyMessage || t.review.noMatches }}</div>
      <button
        type="button"
        class="empty-state-btn"
        @click="emit('recover')"
      >
        {{ emptyActionText || t.review.retry }}
      </button>
    </div>

    <div
      v-else
      ref="containerRef"
      class="virtual-grid-container"
      :class="{ 'is-masonry': layout === 'masonry' }"
      role="grid"
      :aria-label="t.review.gallery"
      tabindex="0"
      @scroll.passive="onScroll"
    >
      <div class="virtual-phantom" :style="{ height: `${totalHeight}px` }">
        <div
          class="virtual-content"
          :class="{ 'masonry-content': layout === 'masonry' }"
          :style="{
            transform: layout === 'grid' ? `translateY(${translateY}px)` : undefined,
            gridTemplateColumns: layout === 'grid' ? `repeat(${cols}, ${itemWidth}px)` : undefined,
            gap: layout === 'grid' ? `${gap}px` : undefined,
            justifyContent: layout === 'grid' ? 'safe center' : undefined,
          }"
        >
          <div
            v-for="{ file, top, left, width, height, imageHeight } in visibleItems"
            :key="file.id ?? file.path"
            class="grid-card"
            :style="layout === 'masonry' ? {
              position: 'absolute',
              top: `${top}px`,
              left: `${left}px`,
              width: `${width}px`,
              height: `${height}px`,
            } : undefined"
            role="gridcell"
            :aria-selected="selectedFile?.path === file.path"
            :class="{
              active: selectedFile?.path === file.path,
              'multi-selected': selectedFilePaths?.has(file.path),
              'is-stacked': isStacked(file),
              'is-collapsed-stack': isCollapsedStack(file),
              'stack-expanded': isStackExpanded(file),
            }"
            draggable="true"
            @dragstart="onDragStart($event, file)"
            @click="onCardClick(file, $event)"
            @dblclick="activateFile(file)"
            @contextmenu.prevent="emit('findSimilar', file)"
            @mouseenter="onVideoMouseEnter(file)"
            @mousemove="onVideoMouseMove($event, file)"
            @mouseleave="onVideoMouseLeave"
          >
            <div
              class="thumbnail-wrapper"
              :style="layout === 'masonry' ? { height: `${imageHeight}px`, aspectRatio: 'auto' } : undefined"
            >
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

              <!-- Video actively scrubbing on hover -->
              <video
                v-if="isVideoContainer(file.container) && hoveredVideoPath === file.path"
                :src="assetUrl(file.path)"
                class="thumbnail-img thumbnail-video video-active"
                :class="{ 'nsfw-blurred': blurNsfw && file.is_nsfw && !revealedNsfw.has(file.path) }"
                muted
                playsinline
                preload="auto"
              />

              <!-- Cached WebP thumbnail or image -->
              <img
                v-else-if="
                  file.container !== 'txt' &&
                  !failedImages.has(file.path) &&
                  getCardImageSrc(file, Math.max(width || itemWidth, imageHeight))
                "
                :src="getCardImageSrc(file, Math.max(width || itemWidth, imageHeight)) || undefined"
                :alt="getFileName(file.path)"
                class="thumbnail-img"
                :class="{ 'nsfw-blurred': blurNsfw && file.is_nsfw && !revealedNsfw.has(file.path) }"
                loading="lazy"
                decoding="async"
                @error="onImageError(file.path)"
              />

              <!-- Video poster when not actively hovered and no cached thumbnail -->
              <video
                v-else-if="isVideoContainer(file.container)"
                :src="assetUrl(file.path)"
                class="thumbnail-img thumbnail-video video-poster"
                :class="{ 'nsfw-blurred': blurNsfw && file.is_nsfw && !revealedNsfw.has(file.path) }"
                muted
                preload="metadata"
                playsinline
                @loadeddata="onVideoLoadedData($event, file)"
              />

              <div
                v-else-if="
                  !isVideoContainer(file.container) &&
                  file.container !== 'txt' &&
                  !failedImages.has(file.path)
                "
                class="thumbnail-pending"
                aria-hidden="true"
              />

              <div
                v-else-if="failedImages.has(file.path)"
                class="thumbnail-fallback thumbnail-failed"
              >
                <span class="fallback-text">{{ file.container.toUpperCase() }}</span>
                <button
                  type="button"
                  class="retry-thumb-btn"
                  :title="t.preview.retryThumbnail"
                  :aria-label="t.preview.retryThumbnail"
                  @click.stop="retryImage(file)"
                >
                  ↻
                </button>
              </div>
              <div v-else class="thumbnail-fallback">
                <span class="fallback-text">{{ file.container.toUpperCase() }}</span>
              </div>

              <!-- Static Video badge -->
              <span
                v-if="isVideoContainer(file.container) && hoveredVideoPath !== file.path"
                class="card-badge badge-video"
                :title="`Video (${file.container.toUpperCase()})`"
              >
                ▶ {{ formatDuration(file.metadata?.duration_seconds) || file.container.toUpperCase() }}
              </span>

              <!-- Dynamic Hover Scrubbing Time Badge -->
              <span
                v-if="isVideoContainer(file.container) && hoveredVideoPath === file.path"
                class="card-badge badge-video-scrub"
              >
                {{ formatDuration(hoveredVideoCurrentTime) }} / {{ formatDuration(hoveredVideoDuration || file.metadata?.duration_seconds) }}
              </span>

              <!-- Mini Timeline Scrubber Track -->
              <div
                v-if="isVideoContainer(file.container) && hoveredVideoPath === file.path"
                class="card-video-scrubber"
              >
                <div
                  class="card-video-scrubber-progress"
                  :style="{ width: `${hoveredVideoProgress * 100}%` }"
                />
              </div>

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
                  v-if="isStackCover(file)"
                  type="button"
                  class="card-badge badge-stack"
                  :class="{ expanded: expandedStacks?.has(file.stack_id) }"
                  :title="t.stack.toggleExpand || 'Toggle Stack Expansion'"
                  :aria-label="`${stackMap?.[file.stack_id]?.count} ${t.stack.stackCount}`"
                  :aria-expanded="expandedStacks?.has(file.stack_id)"
                  @click.stop="emit('toggleStackExpand', file.stack_id)"
                  @dblclick.stop
                >
                  <span class="stack-badge-icon" aria-hidden="true"></span>
                  <span>{{ stackMap?.[file.stack_id]?.count }}</span>
                </button>

                <!-- Stack compare trigger button -->
                <button
                  v-if="isStackCover(file)"
                  type="button"
                  class="card-stack-compare-btn"
                  :title="t.compare.title || 'Compare Stack'"
                  @click.stop="emit('compareStack', file.stack_id)"
                >
                  ⚖️
                </button>

                <!-- Stack cull drafts trigger button -->
                <button
                  v-if="isStackCover(file)"
                  type="button"
                  class="card-stack-cull-btn"
                  :title="t.stack.cullDrafts || 'Cull Lower-Rated Drafts'"
                  @click.stop="emit('cullStack', file.stack_id)"
                >
                  🧹
                </button>
              </template>
            </div>

            <div class="card-info">
              <template v-if="isCollapsedStack(file)">
                <div class="card-title stack-cover-title" :title="t.stack.imageStack">
                  {{ t.stack.imageStack }}
                </div>
                <div class="card-meta">
                  <span>{{ stackMap?.[file.stack_id!]?.count }} {{ t.stack.stackCount }}</span>
                  <span class="card-container">STACK</span>
                </div>
              </template>
              <template v-else>
                <div class="card-title" :title="normalizePath(file.path)">
                  {{ getFileName(file.path) }}
                </div>
                <div class="card-meta">
                  <span>{{ formatDimensions(file) }}</span>
                  <span class="card-container">{{ file.container.toUpperCase() }}</span>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
      <div v-if="loadingMore" class="load-more-indicator" role="status">
        {{ t.view.loading }}
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
  z-index: 5;
}

.virtual-grid-container:focus-visible {
  box-shadow: 0 0 0 2px rgba(47, 111, 237, 0.4);
}

.grid-placeholder {
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  font-size: 0.9em;
}

.grid-placeholder.empty-state {
  flex-direction: column;
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
  justify-content: safe center;
}

.virtual-content.masonry-content {
  display: block;
}

.is-masonry .thumbnail-img {
  object-fit: contain;
}

.grid-card {
  position: relative;
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

/* Collapsed stacks use full card-shaped backing layers instead of a flat shadow. */
.grid-card.is-collapsed-stack {
  overflow: visible;
  isolation: isolate;
  border-color: rgba(47, 111, 237, 0.42);
  box-shadow: 0 5px 16px rgba(15, 23, 42, 0.16);
  animation: stack-collapse-in 180ms cubic-bezier(0.2, 0.8, 0.2, 1);
}

.grid-card.is-collapsed-stack::before,
.grid-card.is-collapsed-stack::after {
  content: "";
  position: absolute;
  inset: 0;
  border: 1px solid rgba(100, 116, 139, 0.34);
  border-radius: 8px;
  background: linear-gradient(145deg, #f8fafc, #e2e8f0);
  pointer-events: none;
}

.grid-card.is-collapsed-stack::before {
  z-index: -2;
  transform: translate(7px, 5px) rotate(1.8deg);
  opacity: 0.72;
}

.grid-card.is-collapsed-stack::after {
  z-index: -1;
  transform: translate(4px, 3px) rotate(0.8deg);
  opacity: 0.9;
}

.grid-card.is-collapsed-stack:hover {
  transform: translateY(-3px) rotate(-0.25deg);
  box-shadow: 0 8px 20px rgba(15, 23, 42, 0.2);
}

.grid-card.is-collapsed-stack .thumbnail-wrapper {
  border-radius: 7px 7px 0 0;
}

.grid-card.is-collapsed-stack .card-info {
  border-radius: 0 0 7px 7px;
}

.grid-card.stack-expanded {
  border-color: rgba(47, 111, 237, 0.5);
  box-shadow: inset 0 3px 0 rgba(47, 111, 237, 0.32);
  animation: stack-spread-in 220ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes stack-spread-in {
  from { opacity: 0; transform: translateY(-10px) scale(0.96); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

@keyframes stack-collapse-in {
  from { transform: scale(0.97); }
  to { transform: scale(1); }
}

@media (prefers-color-scheme: dark) {
  .grid-card.is-collapsed-stack {
    border-color: rgba(96, 165, 250, 0.58);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.42);
  }

  .grid-card.is-collapsed-stack::before,
  .grid-card.is-collapsed-stack::after {
    border-color: rgba(148, 163, 184, 0.3);
    background: linear-gradient(145deg, #334155, #1e293b);
  }
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

.stack-cover-title {
  font-weight: 650;
  letter-spacing: 0.01em;
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

.thumbnail-pending {
  width: 100%;
  height: 100%;
  background: linear-gradient(
    110deg,
    transparent 25%,
    rgba(255, 255, 255, 0.08) 45%,
    transparent 65%
  );
  background-size: 220% 100%;
  animation: thumbnail-queue-pulse 1.2s ease-in-out infinite;
}

@keyframes thumbnail-queue-pulse {
  from { background-position: 100% 0; }
  to { background-position: -100% 0; }
}

@media (prefers-reduced-motion: reduce) {
  .thumbnail-pending {
    animation: none;
    background: rgba(255, 255, 255, 0.04);
  }
}

.thumbnail-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
}

.thumbnail-fallback.thumbnail-failed {
  flex-direction: column;
  gap: 6px;
}

.retry-thumb-btn {
  background: rgba(0, 0, 0, 0.65);
  border: 1px solid rgba(255, 255, 255, 0.25);
  color: #38bdf8;
  border-radius: 4px;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 1rem;
  transition: all 0.15s ease;
}

.retry-thumb-btn:hover {
  background: #0284c7;
  color: #fff;
  border-color: #38bdf8;
  transform: rotate(90deg);
}

@media (prefers-reduced-motion: reduce) {
  .retry-thumb-btn:hover {
    transform: none;
  }
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
  background: rgba(14, 165, 233, 0.9);
  color: #fff;
  font-weight: 700;
  letter-spacing: 0.05em;
  backdrop-filter: blur(4px);
}

.badge-video-scrub {
  top: 6px;
  left: 6px;
  background: rgba(15, 23, 42, 0.85);
  color: #38bdf8;
  border: 1px solid rgba(56, 189, 248, 0.4);
  font-family: ui-monospace, monospace;
  font-weight: 600;
  letter-spacing: 0.04em;
  backdrop-filter: blur(4px);
  z-index: 4;
}

.card-video-scrubber {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: rgba(0, 0, 0, 0.45);
  z-index: 3;
}

.card-video-scrubber-progress {
  height: 100%;
  background: #38bdf8;
  border-radius: 0 2px 2px 0;
}

.thumbnail-video {
  background: #000;
  pointer-events: none;
}

.video-active {
  object-fit: cover;
  width: 100%;
  height: 100%;
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
  min-width: 42px;
  min-height: 25px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background: rgba(15, 23, 42, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.16);
  color: #e0f2fe;
  font-weight: 600;
  font-size: 0.72em;
  padding: 0.15rem 0.45rem;
  border-radius: 4px;
  cursor: pointer;
  z-index: 2;
  opacity: 0.85;
  transition: all 0.15s ease;
}

.grid-card:hover .badge-stack,
.grid-card:focus-within .badge-stack,
.badge-stack:focus-visible {
  opacity: 1;
}

.stack-badge-icon {
  position: relative;
  width: 11px;
  height: 9px;
  border: 1.5px solid currentColor;
  border-radius: 2px;
}

.stack-badge-icon::before,
.stack-badge-icon::after {
  content: "";
  position: absolute;
  width: 9px;
  height: 7px;
  border: 1px solid currentColor;
  border-radius: 2px;
  z-index: -1;
}

.stack-badge-icon::before {
  top: -4px;
  left: 2px;
}

.stack-badge-icon::after {
  top: -7px;
  left: 4px;
  opacity: 0.7;
}

.badge-stack:hover,
.badge-stack:focus-visible {
  background: #0284c7;
  color: #fff;
  border-color: #38bdf8;
  outline: 2px solid #38bdf8;
  outline-offset: 1px;
}

.badge-stack.expanded {
  background: rgba(37, 99, 235, 0.75);
  color: #f0f9ff;
  border-color: rgba(96, 165, 250, 0.55);
}

.badge-stack.expanded:hover,
.badge-stack.expanded:focus-visible {
  background: #2563eb;
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

.grid-card:hover .card-stack-compare-btn,
.grid-card:focus-within .card-stack-compare-btn,
.card-stack-compare-btn:focus-visible {
  opacity: 1;
}

.card-stack-compare-btn:focus-visible {
  outline: 2px solid #818cf8;
  outline-offset: 1px;
}

.card-stack-compare-btn:hover {
  background: #4f46e5;
  border-color: #818cf8;
}

.card-stack-cull-btn {
  position: absolute;
  top: 6px;
  right: 90px;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.55);
  border: 1px solid rgba(255, 255, 255, 0.25);
  color: #fbbf24;
  font-size: 0.75em;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  z-index: 2;
  transition: all 0.15s ease;
}

.grid-card:hover .card-stack-cull-btn,
.grid-card:focus-within .card-stack-cull-btn,
.card-stack-cull-btn:focus-visible {
  opacity: 1;
}

.card-stack-cull-btn:focus-visible {
  outline: 2px solid #f59e0b;
  outline-offset: 1px;
}

.card-stack-cull-btn:hover {
  background: #d97706;
  border-color: #f59e0b;
  color: #fff;
}

@media (prefers-reduced-motion: reduce) {
  .grid-card,
  .grid-card.is-collapsed-stack,
  .card-select-btn,
  .badge-stack,
  .card-stack-compare-btn,
  .card-stack-cull-btn,
  .card-similar-btn {
    transition: none;
    animation: none;
  }
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
