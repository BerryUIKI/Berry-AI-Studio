import { invoke } from "@tauri-apps/api/core";
import { assetUrl } from "./image";
import type { ImageFile } from "../types";

export interface ThumbnailCacheStats {
  total_bytes: number;
  file_count: number;
  cache_dir: string;
}

const THUMBNAIL_SETTING_KEY = "berry_thumbnail_max_edge";
const DEFAULT_MAX_EDGE = 384; // 64 * 6, perfect balanced resolution for 130px~360px grid zoom
const MAX_MEMORY_CACHE_ENTRIES = 3000;

class LruThumbnailCache {
  private cache = new Map<string, string>();
  private maxSize: number;

  constructor(maxSize = MAX_MEMORY_CACHE_ENTRIES) {
    this.maxSize = maxSize;
  }

  get(key: string): string | undefined {
    const val = this.cache.get(key);
    if (val !== undefined) {
      this.cache.delete(key);
      this.cache.set(key, val);
    }
    return val;
  }

  set(key: string, value: string): void {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.maxSize) {
      const oldestKey = this.cache.keys().next().value;
      if (oldestKey !== undefined) {
        this.cache.delete(oldestKey);
      }
    }
    this.cache.set(key, value);
  }

  clear(): void {
    this.cache.clear();
  }

  get size(): number {
    return this.cache.size;
  }
}

// In-memory runtime LRU map of file revision + size tier -> asset URL.
const memoryCache = new LruThumbnailCache(3000);

// Active in-flight promises to deduplicate concurrent requests for the same file
const inFlightRequests = new Map<string, Promise<string>>();
interface QueuedThumbnail {
  cache_key: string;
  file_id: number;
  file_path: string;
  modified_at: number;
  max_edge: number;
}

const queuedBatchItems = new Map<string, QueuedThumbnail>();
const batchReadyKeys = new Set<string>();
let batchDrainPromise: Promise<number> | null = null;
const BATCH_CHUNK_SIZE = 48;

/**
 * Get the user-configured max edge resolution from localStorage.
 */
export function getThumbnailMaxEdge(): number {
  try {
    const val = localStorage.getItem(THUMBNAIL_SETTING_KEY);
    if (val) {
      const parsed = parseInt(val, 10);
      if (parsed >= 128 && parsed <= 1024) return parsed;
    }
  } catch {
    // Ignore localStorage access errors
  }
  return DEFAULT_MAX_EDGE;
}

/**
 * Save user-configured thumbnail resolution.
 */
export function setThumbnailMaxEdge(maxEdge: number): void {
  try {
    localStorage.setItem(THUMBNAIL_SETTING_KEY, String(maxEdge));
    // Clear in-memory cache so images request new resolution
    memoryCache.clear();
    queuedBatchItems.clear();
    batchReadyKeys.clear();
  } catch {
    // Ignore errors
  }
}

/** Stable cache identity for a particular file revision and thumbnail tier. */
export function getThumbnailCacheKey(
  file: ImageFile,
  maxEdge: number = getThumbnailMaxEdge(),
): string {
  return `${file.id ?? 0}:${file.modified_at}:${maxEdge}`;
}

/**
 * Check if thumbnail URL is already available in memory cache synchronously.
 */
export function getThumbnailUrlSync(file: ImageFile): string | null {
  const fileId = file.id ?? 0;
  if (!fileId) return null;
  return memoryCache.get(getThumbnailCacheKey(file)) ?? null;
}

/**
 * Get or asynchronously generate thumbnail URL for a given image file.
 */
export async function getThumbnailUrl(
  file: ImageFile,
  maxEdge: number = getThumbnailMaxEdge(),
): Promise<string> {
  const fileId = file.id ?? 0;
  if (!fileId) return assetUrl(file.path);
  const cacheKey = getThumbnailCacheKey(file, maxEdge);

  // Check memory cache first
  const cached = memoryCache.get(cacheKey);
  if (cached) return cached;

  // Deduplicate in-flight requests
  if (inFlightRequests.has(cacheKey)) {
    return inFlightRequests.get(cacheKey)!;
  }

  const promise = (async () => {
    try {
      const diskPath = await invoke<string>("get_or_create_thumbnail", {
        fileId,
        filePath: file.path,
        modifiedAt: file.modified_at,
        maxEdge,
      });
      const url = assetUrl(diskPath);
      memoryCache.set(cacheKey, url);
      return url;
    } catch {
      // Fallback to original image if downsampling fails (e.g. video)
      const fallbackUrl = assetUrl(file.path);
      memoryCache.set(cacheKey, fallbackUrl);
      return fallbackUrl;
    } finally {
      inFlightRequests.delete(cacheKey);
    }
  })();

  inFlightRequests.set(cacheKey, promise);
  return promise;
}

/**
 * Batch generate thumbnails in background for a list of files.
 */
export async function requestBatchThumbnails(
  files: ImageFile[],
  maxEdge: number = getThumbnailMaxEdge(),
): Promise<number> {
  if (!files || files.length === 0) return 0;

  for (const file of files) {
    const cacheKey = getThumbnailCacheKey(file, maxEdge);
    if (
      file.id == null ||
      batchReadyKeys.has(cacheKey) ||
      inFlightRequests.has(cacheKey) ||
      memoryCache.get(cacheKey)
    ) continue;
    queuedBatchItems.set(cacheKey, {
      cache_key: cacheKey,
      file_id: file.id,
      file_path: file.path,
      modified_at: file.modified_at,
      max_edge: maxEdge,
    });
  }

  if (queuedBatchItems.size === 0) return 0;
  if (batchDrainPromise) return batchDrainPromise;

  batchDrainPromise = (async () => {
    let generated = 0;
    while (queuedBatchItems.size > 0) {
      const nextEdge = queuedBatchItems.values().next().value?.max_edge;
      const items = Array.from(queuedBatchItems.values())
        .filter((item) => item.max_edge === nextEdge && !inFlightRequests.has(item.cache_key))
        .slice(0, BATCH_CHUNK_SIZE);
      if (items.length === 0) break;
      for (const item of items) queuedBatchItems.delete(item.cache_key);
      try {
        generated += await invoke<number>("batch_generate_thumbnails", {
          items: items.map(({ file_id, file_path, modified_at }) => ({ file_id, file_path, modified_at })),
          maxEdge: nextEdge,
        });
        for (const item of items) batchReadyKeys.add(item.cache_key);
      } catch {
        // Visible items can still recover through the single-thumbnail path.
      }
    }
    return generated;
  })().finally(() => {
    batchDrainPromise = null;
  });

  return batchDrainPromise;
}

/**
 * Fetch thumbnail cache statistics from disk.
 */
export async function getThumbnailCacheStats(): Promise<ThumbnailCacheStats> {
  return await invoke<ThumbnailCacheStats>("get_thumbnail_cache_stats");
}

/**
 * Clear all thumbnail cache files from disk and memory.
 */
export async function clearThumbnailCache(): Promise<number> {
  memoryCache.clear();
  inFlightRequests.clear();
  queuedBatchItems.clear();
  batchReadyKeys.clear();
  return await invoke<number>("clear_thumbnail_cache");
}
