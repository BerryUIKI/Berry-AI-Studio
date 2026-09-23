import type { ImageFile } from "../types";

export interface FileDetailsCacheEntry {
  file: ImageFile;
  revisionKey: string;
}

/**
 * Manages full file detail hydration, LRU caching, mutation invalidation,
 * and generation fencing to prevent stale responses from reverting user edits.
 */
export class FileDetailsManager {
  private cache = new Map<number, FileDetailsCacheEntry>();
  private inFlight = new Map<number, { promise: Promise<ImageFile>; generation: number }>();
  private generation = 0;
  private readonly maxSize: number;

  constructor(maxSize = 64) {
    this.maxSize = maxSize;
  }

  /**
   * Generates a stable revision identity string representing
   * file id, modification timestamp, and file path.
   */
  static revisionKey(file: ImageFile): string {
    return `${file.id ?? "null"}:${file.modified_at}:${file.path}`;
  }

  /**
   * Current global generation counter.
   */
  get currentGeneration(): number {
    return this.generation;
  }

  /**
   * Current number of entries in the LRU cache.
   */
  get size(): number {
    return this.cache.size;
  }

  /**
   * Reset all cached details and fence all currently in-flight requests.
   */
  reset(): void {
    this.generation++;
    this.cache.clear();
    this.inFlight.clear();
  }

  /**
   * Retrieve cached file details if matching the source revision.
   * Promotes the entry to MRU in LRU order on a cache hit.
   */
  get(file: ImageFile): ImageFile | null {
    if (file.id == null) return null;
    const entry = this.cache.get(file.id);
    if (!entry) return null;

    if (entry.revisionKey !== FileDetailsManager.revisionKey(file)) {
      this.cache.delete(file.id);
      return null;
    }

    // LRU promotion on hit
    this.cache.delete(file.id);
    this.cache.set(file.id, entry);
    return entry.file;
  }

  /**
   * Cache hydrated details and evict the oldest entry if exceeding maxSize.
   */
  set(details: ImageFile): void {
    if (details.id == null) return;
    this.cache.delete(details.id);
    this.cache.set(details.id, {
      file: { ...details },
      revisionKey: FileDetailsManager.revisionKey(details),
    });

    while (this.cache.size > this.maxSize) {
      const oldestKey = this.cache.keys().next().value;
      if (oldestKey === undefined) break;
      this.cache.delete(oldestKey);
    }
  }

  /**
   * Update cached file when ratings, favorites, NSFW, or metadata mutate.
   * Updates cached state in-place so subsequent cache hits preserve modifications.
   */
  update(fileId: number, patch: Partial<ImageFile>): void {
    const entry = this.cache.get(fileId);
    if (entry) {
      entry.file = { ...entry.file, ...patch };
      if (patch.modified_at != null || patch.path != null) {
        entry.revisionKey = FileDetailsManager.revisionKey(entry.file);
      }
    }
  }

  /**
   * Invalidate specific file details by ID.
   */
  invalidate(fileId: number): void {
    this.generation++;
    this.cache.delete(fileId);
    this.inFlight.delete(fileId);
  }

  /**
   * Safely merge hydrated details into the current selection.
   * The current selection's rating, favorite, and NSFW state are authoritative
   * and must never be reverted by an older hydrated row.
   */
  merge(current: ImageFile, hydrated: ImageFile): ImageFile {
    return {
      ...hydrated,
      rating: current.rating,
      is_favorite: current.is_favorite,
      is_nsfw: current.is_nsfw,
      path: current.path,
    };
  }

  /**
   * Hydrate full file details, deduplicating in-flight requests and
   * fencing stale responses across mutations or navigation.
   */
  async hydrate(
    file: ImageFile,
    fetchFn: (fileId: number) => Promise<ImageFile>,
    force = false,
  ): Promise<{ details: ImageFile; generation: number } | null> {
    if (file.id == null) return null;
    const fileId = file.id;

    if (!force) {
      const cached = this.get(file);
      if (cached) {
        return { details: cached, generation: this.generation };
      }
    }

    const requestGen = this.generation;
    let inflight = this.inFlight.get(fileId);
    if (!inflight || force) {
      const promise = fetchFn(fileId);
      inflight = { promise, generation: requestGen };
      this.inFlight.set(fileId, inflight);
    }

    try {
      const details = await inflight.promise;

      // Response fencing: discard if generation changed or request was superseded
      if (this.generation !== inflight.generation) {
        return null;
      }

      if (file.modified_at !== details.modified_at) {
        return null;
      }

      this.set(details);
      return { details, generation: inflight.generation };
    } finally {
      if (this.inFlight.get(fileId)?.promise === inflight.promise) {
        this.inFlight.delete(fileId);
      }
    }
  }
}
