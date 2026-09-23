import { invoke } from "@tauri-apps/api/core";
import type { ChangeLogEntry, ChangeLogSyncQuery, DatabasePingResult } from "../types";

export type ChangeLogListener = (entry: ChangeLogEntry) => void;
export type BatchSyncListener = (entries: ChangeLogEntry[]) => void;

export interface CollaborationSyncOptions {
  activeCadenceMs?: number; // default: 2000
  idleCadenceMs?: number; // default: 10000
  batchLimit?: number; // default: 100
}

/**
 * Real-time collaboration change journal synchronization engine.
 * Adaptively polls the shared change log table with low overhead (<0.5ms query time),
 * dispatching live mutation events (ratings, tags, stacks, deletions) to the local UI.
 */
export class CollaborationSyncEngine {
  private lastSyncId: number = 0;
  private clientId: string = "local_client";
  private isRunning: boolean = false;
  private isDocumentVisible: boolean = true;
  private timer: any = null;
  private listeners: Set<ChangeLogListener> = new Set();
  private batchListeners: Set<BatchSyncListener> = new Set();
  private activeCadenceMs: number = 2000;
  private idleCadenceMs: number = 10000;
  private batchLimit: number = 100;
  private lastSyncTime: number = 0;
  private totalSyncedCount: number = 0;
  private runGeneration: number = 0;
  private readonly visibilityListener = () => {
    if (typeof document !== "undefined") {
      this.setDocumentVisible(!document.hidden);
    }
  };

  constructor(options?: CollaborationSyncOptions) {
    if (options?.activeCadenceMs) this.activeCadenceMs = options.activeCadenceMs;
    if (options?.idleCadenceMs) this.idleCadenceMs = options.idleCadenceMs;
    if (options?.batchLimit) this.batchLimit = options.batchLimit;
  }

  public init(clientId: string, initialAfterId: number = 0) {
    this.clientId = clientId;
    this.lastSyncId = initialAfterId;
  }

  public setClientId(clientId: string) {
    this.clientId = clientId;
  }

  public getLastSyncId(): number {
    return this.lastSyncId;
  }

  public getLastSyncTime(): number {
    return this.lastSyncTime;
  }

  public getTotalSyncedCount(): number {
    return this.totalSyncedCount;
  }

  public getIsRunning(): boolean {
    return this.isRunning;
  }

  public getRunGeneration(): number {
    return this.runGeneration;
  }

  public getVisibilityListener(): () => void {
    return this.visibilityListener;
  }

  public setDocumentVisible(visible: boolean) {
    this.isDocumentVisible = visible;
  }

  public getEffectiveCadenceMs(): number {
    return this.isDocumentVisible ? this.activeCadenceMs : this.idleCadenceMs;
  }

  public onEvent(listener: ChangeLogListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  public onBatch(listener: BatchSyncListener): () => void {
    this.batchListeners.add(listener);
    return () => {
      this.batchListeners.delete(listener);
    };
  }

  public start() {
    if (this.isRunning) return;
    this.isRunning = true;
    this.runGeneration++;
    this.setupVisibilityListeners();
    this.scheduleNextTick();
  }

  public stop() {
    this.isRunning = false;
    this.runGeneration++;
    if (typeof document !== "undefined" && typeof document.removeEventListener === "function") {
      document.removeEventListener("visibilitychange", this.visibilityListener);
    }
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }

  public dispose() {
    this.stop();
    this.listeners.clear();
    this.batchListeners.clear();
  }

  public async pollOnce(
    fetchFn?: (query: ChangeLogSyncQuery) => Promise<ChangeLogEntry[]>,
  ): Promise<ChangeLogEntry[]> {
    const generation = this.runGeneration;
    try {
      const query: ChangeLogSyncQuery = {
        after_id: this.lastSyncId,
        exclude_client_id: this.clientId,
        limit: this.batchLimit,
      };

      const entries = fetchFn
        ? await fetchFn(query)
        : await invoke<ChangeLogEntry[]>("fetch_change_log", { query });

      if (generation !== this.runGeneration) {
        return [];
      }

      this.lastSyncTime = Date.now();

      if (entries && entries.length > 0) {
        let maxId = this.lastSyncId;
        for (const entry of entries) {
          if (entry.id > maxId) {
            maxId = entry.id;
          }
        }
        this.lastSyncId = maxId;
        this.totalSyncedCount += entries.length;

        for (const entry of entries) {
          for (const listener of this.listeners) {
            try {
              listener(entry);
            } catch (err) {
              console.error("Error in collaboration change listener:", err);
            }
          }
        }

        for (const batchListener of this.batchListeners) {
          try {
            batchListener(entries);
          } catch (err) {
            console.error("Error in collaboration batch listener:", err);
          }
        }
      }

      return entries || [];
    } catch (err) {
      console.warn("Collaboration change sync poll error:", err);
      return [];
    }
  }

  private scheduleNextTick() {
    if (!this.isRunning) return;
    const generation = this.runGeneration;
    const cadence = this.getEffectiveCadenceMs();
    this.timer = setTimeout(async () => {
      if (!this.isRunning || generation !== this.runGeneration) return;
      await this.pollOnce();
      if (generation === this.runGeneration && this.isRunning) {
        this.scheduleNextTick();
      }
    }, cadence);
  }

  private setupVisibilityListeners() {
    if (typeof document !== "undefined" && typeof document.addEventListener === "function") {
      this.visibilityListener();
      document.addEventListener("visibilitychange", this.visibilityListener);
    }
  }
}

/**
 * Test connectivity and ping latency for local SQLite or remote MySQL/PostgreSQL databases.
 */
export async function pingDatabase(
  backend: string,
  connectionUrl: string,
): Promise<DatabasePingResult> {
  return await invoke<DatabasePingResult>("test_database_connection", {
    backend,
    connectionUrl,
  });
}

export const collaborationSync = new CollaborationSyncEngine();
