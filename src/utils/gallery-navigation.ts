import { getCurrentInstance, onUnmounted, watch, type Ref } from "vue";
import type { ImageFile } from "../types";

export interface GalleryAnchor {
  id: number | string;
  offset: number;
  index?: number;
}

export interface GalleryNavigationOptions {
  element: Ref<HTMLElement | null>;
  key: () => string;
  files: () => ImageFile[];
  revision: () => number;
  loading: () => boolean;
  hasMore: () => boolean;
  loadingMore: () => boolean;
  top: (index: number) => number;
  itemHeight?: (index: number) => number;
  firstVisible: () => number;
  loadMore: () => void;
  onRestore: (top: number) => void;
  maxRestorePages?: number;
}

export const MAX_RESTORE_PAGES = 5;

const anchors = new Map<string, GalleryAnchor>();

export function clearGalleryNavigationAnchors(): void {
  anchors.clear();
}

export function getGalleryNavigationAnchor(key: string): GalleryAnchor | undefined {
  return anchors.get(key);
}

export function setGalleryNavigationAnchor(key: string, anchor: GalleryAnchor): void {
  anchors.set(key, anchor);
}

const raf = typeof requestAnimationFrame === "function"
  ? requestAnimationFrame
  : (cb: FrameRequestCallback) => setTimeout(cb, 0) as unknown as number;

const caf = typeof cancelAnimationFrame === "function"
  ? cancelAnimationFrame
  : (id: number) => clearTimeout(id);

/**
 * Context state survives view unmounts. Only the anchor is retained, never pages.
 * Preserves anchor file plus offset across view modes, folder switches, search queries,
 * and page appends, avoiding infinite paging when an anchor is missing.
 */
export function useGalleryNavigation(options: GalleryNavigationOptions) {
  let key = options.key();
  let restoring = anchors.has(key);
  let restorePagesRequested = 0;
  const maxPages = options.maxRestorePages ?? MAX_RESTORE_PAGES;
  let frame: number | null = null;

  function save() {
    if (restoring || options.loading()) return;
    const element = options.element.value;
    if (!element) return;
    const files = options.files();
    if (!files.length) return;

    const visibleIndex = options.firstVisible();
    const index = Math.min(Math.max(0, visibleIndex), files.length - 1);
    const file = files[index];
    if (!file) return;

    const id = file.id ?? file.path;
    const itemTop = options.top(index);
    const offset = Math.max(0, element.scrollTop - itemTop);

    anchors.delete(key);
    anchors.set(key, { id, offset, index });
    while (anchors.size > 64) {
      const firstKey = anchors.keys().next().value;
      if (firstKey !== undefined) anchors.delete(firstKey);
    }
  }

  watch(
    options.key,
    (next, prev) => {
      if (prev !== undefined && prev !== next) {
        save();
      }
      key = next;
      restoring = anchors.has(key);
      restorePagesRequested = 0;
      if (frame !== null) {
        caf(frame);
        frame = null;
      }
    },
    { flush: "sync" },
  );

  watch(
    [
      options.element,
      options.key,
      () => options.files().length,
      options.revision,
      options.loading,
      options.loadingMore,
    ],
    () => {
      if (frame !== null) caf(frame);
      frame = raf(() => {
        frame = null;
        if (options.loading() || !options.element.value) return;

        const anchor = anchors.get(key);
        if (!restoring || !anchor) return;

        const files = options.files();
        if (files.length === 0) {
          if (!options.hasMore()) {
            restoring = false;
            anchors.delete(key);
          }
          return;
        }

        const index = files.findIndex((file) => (file.id ?? file.path) === anchor.id);

        if (index < 0) {
          // Anchor not found in currently loaded files
          if (options.hasMore() && restorePagesRequested < maxPages) {
            if (!options.loadingMore()) {
              restorePagesRequested++;
              options.loadMore();
            }
            return;
          }

          // Anchor missing after exceeding page budget or end of dataset reached
          restoring = false;
          anchors.delete(key);
          if (options.element.value) {
            options.element.value.scrollTop = 0;
          }
          options.onRestore(0);
          return;
        }

        // Anchor found! Compute scroll position with offset
        let offset = Math.max(0, anchor.offset);
        if (options.itemHeight) {
          const height = options.itemHeight(index);
          if (height > 0) {
            offset = Math.min(offset, Math.max(0, height - 1));
          }
        }

        const targetTop = Math.max(0, options.top(index) + offset);
        if (options.element.value) {
          options.element.value.scrollTop = targetTop;
          options.onRestore(options.element.value.scrollTop);
        }
        restoring = false;
      });
    },
    { flush: "post", immediate: true },
  );

  function dispose() {
    save();
    if (frame !== null) {
      caf(frame);
      frame = null;
    }
  }

  if (getCurrentInstance()) {
    onUnmounted(dispose);
  }

  return { save, dispose };
}
