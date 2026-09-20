import { invoke } from "@tauri-apps/api/core";
import { assetUrl } from "./image";
import type { ImageFile } from "../types";
import { selectThumbnailTier } from "./thumbnail-tier";

export interface ThumbnailCacheStats {
  total_bytes: number;
  file_count: number;
  cache_dir: string;
  budget_bytes: number;
}

interface ThumbnailBatchResult {
  generated: number;
  canceled: number;
}

export const THUMBNAIL_PRIORITY = {
  FAR_LOOKAHEAD: 10,
  NEAR_LOOKAHEAD: 20,
} as const;

interface ThumbnailBatchOptions {
  generation?: number;
  priority?: number;
}

const THUMBNAIL_SETTING_KEY = "berry_thumbnail_max_edge";
const THUMBNAIL_BUDGET_SETTING_KEY = "berry_thumbnail_cache_budget_mb";
const DEFAULT_MAX_EDGE = 384; // 64 * 6, perfect balanced resolution for 130px~360px grid zoom
const DEFAULT_CACHE_BUDGET_MB = 2048;
const MAX_MEMORY_CACHE_ENTRIES = 3000;
let configuredThumbnailMaxEdge: number | null = null;

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

const frontendQueueCounters = {
  memoryCacheHits: 0,
  dedupeHits: 0,
  requestsDispatched: 0,
};

export interface BackendThumbnailDiagnostics {
  queued: number;
  running: number;
  completed: number;
  canceled: number;
  failed: number;
  reused_tier_hits: number;
  manifest_hits: number;
  active_generation: number;
}

export interface FrontendThumbnailDiagnostics {
  inFlightCount: number;
  queuedBatchCount: number;
  memoryCacheSize: number;
  memoryCacheHits: number;
  dedupeHits: number;
  activeGeneration: number;
  requestsDispatched: number;
}

export interface ThumbnailDiagnosticsSummary {
  backend: BackendThumbnailDiagnostics;
  frontend: FrontendThumbnailDiagnostics;
}

// Active requests are generation-aware so a new viewport never inherits a
// canceled promise from the previous scroll position.
const inFlightRequests = new Map<
  string,
  { generation?: number; promise: Promise<string> }
>();
interface QueuedThumbnail {
  cache_key: string;
  file_id: number;
  file_path: string;
  modified_at: number;
  max_edge: number;
  generation: number;
  priority: number;
  sequence: number;
}

const queuedBatchItems = new Map<string, QueuedThumbnail>();
const batchReadyKeys = new Set<string>();
let batchDrainPromise: Promise<number> | null = null;
let activeThumbnailGeneration = 0;
let thumbnailQueueSequence = 0;
let pendingCancellationGeneration = 0;
let sentCancellationGeneration = 0;
let cancellationPromise: Promise<void> | null = null;
const BATCH_CHUNK_SIZE = 48;
const CANCELED_REQUEST_MESSAGE = "thumbnail request canceled";


function scheduleThumbnailCancellation(generation: number): Promise<void> {
  pendingCancellationGeneration = Math.max(pendingCancellationGeneration, generation);
  if (cancellationPromise) return cancellationPromise;

  cancellationPromise = (async () => {
    while (sentCancellationGeneration < pendingCancellationGeneration) {
      const nextGeneration = pendingCancellationGeneration;
      try {
        await invoke("cancel_thumbnail_requests", { generation: nextGeneration });
      } finally {
        sentCancellationGeneration = nextGeneration;
      }
    }
  })().finally(() => {
    cancellationPromise = null;
    if (sentCancellationGeneration < pendingCancellationGeneration) {
      void scheduleThumbnailCancellation(pendingCancellationGeneration);
    }
  });
  return cancellationPromise;
}

/** Start a viewport generation and invalidate stale visible and look-ahead work. */
export function beginThumbnailRequestCycle(): number {
  activeThumbnailGeneration += 1;
  queuedBatchItems.clear();
  void scheduleThumbnailCancellation(activeThumbnailGeneration).catch(() => {
    // A later request also advances the backend generation.
  });
  return activeThumbnailGeneration;
}

/** Cancel pending work when a gallery surface is removed. */
export function cancelThumbnailRequests(): void {
  beginThumbnailRequestCycle();
}

/**
 * Get the user-configured max edge resolution from localStorage.
 */
export function getThumbnailMaxEdge(): number {
  if (configuredThumbnailMaxEdge !== null) return configuredThumbnailMaxEdge;
  try {
    const val = localStorage.getItem(THUMBNAIL_SETTING_KEY);
    if (val) {
      const parsed = parseInt(val, 10);
      if (parsed >= 128 && parsed <= 1024) {
        configuredThumbnailMaxEdge = parsed;
        return parsed;
      }
    }
  } catch {
    // Ignore localStorage access errors
  }
  configuredThumbnailMaxEdge = DEFAULT_MAX_EDGE;
  return configuredThumbnailMaxEdge;
}

/** Select a cache tier for a rendered thumbnail without exceeding user settings. */
export function getThumbnailTier(displayEdge: number): number {
  const deviceScale = typeof window === "undefined" ? 1 : window.devicePixelRatio;
  return selectThumbnailTier(displayEdge, getThumbnailMaxEdge(), deviceScale);
}

/**
 * Save user-configured thumbnail resolution.
 */
export function setThumbnailMaxEdge(maxEdge: number): void {
  configuredThumbnailMaxEdge = maxEdge;
  try {
    localStorage.setItem(THUMBNAIL_SETTING_KEY, String(maxEdge));
    // Clear in-memory cache so images request new resolution
    memoryCache.clear();
    batchReadyKeys.clear();
    beginThumbnailRequestCycle();
  } catch {
    // Ignore errors
  }
}

/** Read the configured persistent thumbnail disk budget. */
export function getThumbnailCacheBudgetMb(): number {
  try {
    const value = localStorage.getItem(THUMBNAIL_BUDGET_SETTING_KEY);
    if (value) {
      const parsed = parseInt(value, 10);
      if (parsed >= 256 && parsed <= 65_536) return parsed;
    }
  } catch {
    // Ignore localStorage access errors.
  }
  return DEFAULT_CACHE_BUDGET_MB;
}

/** Save the thumbnail disk budget for synchronous request scheduling. */
export function setThumbnailCacheBudgetMb(budgetMb: number): void {
  try {
    localStorage.setItem(THUMBNAIL_BUDGET_SETTING_KEY, String(budgetMb));
  } catch {
    // Ignore localStorage access errors.
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
export function getThumbnailUrlSync(
  file: ImageFile,
  maxEdge: number = getThumbnailMaxEdge(),
): string | null {
  const fileId = file.id ?? 0;
  if (!fileId) return null;
  const cached = memoryCache.get(getThumbnailCacheKey(file, maxEdge)) ?? null;
  if (cached) frontendQueueCounters.memoryCacheHits++;
  return cached;
}

/**
 * Get or asynchronously generate thumbnail URL for a given image file.
 */
export async function getThumbnailUrl(
  file: ImageFile,
  maxEdge: number = getThumbnailMaxEdge(),
  generation?: number,
): Promise<string> {
  const fileId = file.id ?? 0;
  if (!fileId) return assetUrl(file.path);
  const cacheKey = getThumbnailCacheKey(file, maxEdge);

  // Check memory cache first
  const cached = memoryCache.get(cacheKey);
  if (cached) {
    frontendQueueCounters.memoryCacheHits++;
    return cached;
  }

  // Deduplicate in-flight requests
  const existingRequest = inFlightRequests.get(cacheKey);
  if (
    existingRequest &&
    (existingRequest.generation === undefined || existingRequest.generation === generation)
  ) {
    frontendQueueCounters.dedupeHits++;
    return existingRequest.promise;
  }

  frontendQueueCounters.requestsDispatched++;
  let promise!: Promise<string>;

  promise = (async () => {
    try {
      const diskPath = await invoke<string>("get_or_create_thumbnail", {
        request: {
          fileId,
          filePath: file.path,
          modifiedAt: file.modified_at,
          maxEdge,
          cacheBudgetMb: getThumbnailCacheBudgetMb(),
          generation,
        },
      });
      const url = assetUrl(diskPath);
      memoryCache.set(cacheKey, url);
      return url;
    } catch (error) {
      if (generation !== undefined && String(error).includes(CANCELED_REQUEST_MESSAGE)) {
        throw error;
      }
      // Fallback to original image if downsampling fails (e.g. video)
      const fallbackUrl = assetUrl(file.path);
      memoryCache.set(cacheKey, fallbackUrl);
      return fallbackUrl;
    } finally {
      if (inFlightRequests.get(cacheKey)?.promise === promise) {
        inFlightRequests.delete(cacheKey);
      }
    }
  })();

  inFlightRequests.set(cacheKey, { generation, promise });
  return promise;
}

/**
 * Batch generate thumbnails in background for a list of files.
 */
export async function requestBatchThumbnails(
  files: ImageFile[],
  maxEdge: number = getThumbnailMaxEdge(),
  options: ThumbnailBatchOptions = {},
): Promise<number> {
  if (!files || files.length === 0) return 0;
  const generation = options.generation ?? activeThumbnailGeneration;
  const priority = options.priority ?? THUMBNAIL_PRIORITY.FAR_LOOKAHEAD;

  for (const file of files) {
    const cacheKey = getThumbnailCacheKey(file, maxEdge);
    if (
      file.id == null ||
      batchReadyKeys.has(cacheKey) ||
      inFlightRequests.has(cacheKey) ||
      memoryCache.get(cacheKey)
    ) continue;
    const queued = queuedBatchItems.get(cacheKey);
    if (
      queued &&
      (queued.generation > generation ||
        (queued.generation === generation && queued.priority >= priority))
    ) continue;
    queuedBatchItems.set(cacheKey, {
      cache_key: cacheKey,
      file_id: file.id,
      file_path: file.path,
      modified_at: file.modified_at,
      max_edge: maxEdge,
      generation,
      priority,
      sequence: thumbnailQueueSequence++,
    });
  }

  if (queuedBatchItems.size === 0) return 0;
  if (batchDrainPromise) return batchDrainPromise;

  batchDrainPromise = (async () => {
    let generated = 0;
    while (queuedBatchItems.size > 0) {
      const ordered = Array.from(queuedBatchItems.values()).sort(
        (left, right) =>
          right.generation - left.generation ||
          right.priority - left.priority ||
          left.sequence - right.sequence,
      );
      const next = ordered[0];
      if (!next) break;
      const items = ordered
        .filter(
          (item) =>
            item.generation === next.generation &&
            item.max_edge === next.max_edge &&
            !inFlightRequests.has(item.cache_key),
        )
        .slice(0, BATCH_CHUNK_SIZE);
      if (items.length === 0) break;
      for (const item of items) queuedBatchItems.delete(item.cache_key);
      try {
        const result = await invoke<ThumbnailBatchResult>("batch_generate_thumbnails", {
          items: items.map(({ file_id, file_path, modified_at }) => ({ file_id, file_path, modified_at })),
          maxEdge: next.max_edge,
          cacheBudgetMb: getThumbnailCacheBudgetMb(),
          generation: next.generation,
        });
        generated += result.generated;
        if (result.canceled === 0) {
          for (const item of items) batchReadyKeys.add(item.cache_key);
        }
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
  return await invoke<ThumbnailCacheStats>("get_thumbnail_cache_stats", {
    cacheBudgetMb: getThumbnailCacheBudgetMb(),
  });
}

/**
 * Clear all thumbnail cache files from disk and memory.
 */
export async function clearThumbnailCache(): Promise<number> {
  const generation = beginThumbnailRequestCycle();
  await scheduleThumbnailCancellation(generation);
  memoryCache.clear();
  inFlightRequests.clear();
  queuedBatchItems.clear();
  batchReadyKeys.clear();
  return await invoke<number>("clear_thumbnail_cache");
}

/**
 * Fetch snapshot of runtime thumbnail queue diagnostics (both backend and frontend).
 */
export async function getThumbnailDiagnostics(): Promise<ThumbnailDiagnosticsSummary> {
  const backend = await invoke<BackendThumbnailDiagnostics>("get_thumbnail_queue_diagnostics");
  const frontend: FrontendThumbnailDiagnostics = {
    inFlightCount: inFlightRequests.size,
    queuedBatchCount: queuedBatchItems.size,
    memoryCacheSize: memoryCache.size,
    memoryCacheHits: frontendQueueCounters.memoryCacheHits,
    dedupeHits: frontendQueueCounters.dedupeHits,
    activeGeneration: activeThumbnailGeneration,
    requestsDispatched: frontendQueueCounters.requestsDispatched,
  };
  return { backend, frontend };
}

/**
 * Reset runtime thumbnail queue diagnostics counters.
 */
export async function resetThumbnailDiagnostics(): Promise<void> {
  frontendQueueCounters.memoryCacheHits = 0;
  frontendQueueCounters.dedupeHits = 0;
  frontendQueueCounters.requestsDispatched = 0;
  await invoke("reset_thumbnail_queue_diagnostics");
}

// In development mode, attach diagnostics to window for manual inspections without console noise.
if (typeof window !== "undefined" && import.meta.env?.DEV) {
  (window as unknown as { __BERRY_THUMBNAIL_DIAGNOSTICS__?: unknown }).__BERRY_THUMBNAIL_DIAGNOSTICS__ = {
    get: getThumbnailDiagnostics,
    reset: resetThumbnailDiagnostics,
  };
}

