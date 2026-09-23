import type { ClipBatchIndexResult } from "../types";

export type IndexingStatus = "idle" | "indexing" | "completed" | "canceled" | "error";

export interface ClipIndexingOptions {
  batchSize: number;
  retryFailed?: boolean;
  onBatchIndexed?: (res: ClipBatchIndexResult) => void | Promise<void>;
  delayMs?: number;
}

export class ClipIndexingController {
  private _status: IndexingStatus = "idle";
  private _failedCount: number = 0;
  private _errorMessage: string | null = null;
  private _isCanceled: boolean = false;
  private _invoke: (cmd: string, args?: Record<string, unknown>) => Promise<any>;

  constructor(invokeFn?: (cmd: string, args?: Record<string, unknown>) => Promise<any>) {
    this._invoke =
      invokeFn ??
      (async (cmd, args) => {
        const { invoke } = await import("@tauri-apps/api/core");
        return invoke(cmd, args);
      });
  }

  get status(): IndexingStatus {
    return this._status;
  }

  get isIndexing(): boolean {
    return this._status === "indexing";
  }

  get failedCount(): number {
    return this._failedCount;
  }

  set failedCount(val: number) {
    this._failedCount = Math.max(0, val);
  }

  get errorMessage(): string | null {
    return this._errorMessage;
  }

  async startIndexing(options: ClipIndexingOptions): Promise<void> {
    if (this.isIndexing) return;

    this._status = "indexing";
    this._isCanceled = false;
    this._errorMessage = null;

    const batchSize = Math.max(1, Math.min(20, options.batchSize || 20));
    let isFirstBatch = true;
    const retryFailed = Boolean(options.retryFailed);

    try {
      while (!this._isCanceled) {
        const res: ClipBatchIndexResult = await this._invoke("index_clip_images_batch", {
          batchSize,
          // Only pass retryFailed: true on the first batch, never continuously clear the failure set
          retryFailed: isFirstBatch ? retryFailed : false,
        });
        isFirstBatch = false;

        if (res.failed_count !== undefined) {
          this._failedCount = res.failed_count;
        }

        if (options.onBatchIndexed) {
          await options.onBatchIndexed(res);
        }

        if (this._isCanceled) {
          this._status = "canceled";
          break;
        }

        if (res.remaining_count === 0) {
          this._status = "completed";
          break;
        }

        if (options.delayMs && options.delayMs > 0) {
          await new Promise((resolve) => setTimeout(resolve, options.delayMs));
        }
      }

      if (this._isCanceled && this._status !== "canceled") {
        this._status = "canceled";
      }
    } catch (err: any) {
      this._errorMessage = String(err?.message || err);
      this._status = "error";
    }
  }

  async stopIndexing(): Promise<void> {
    this._isCanceled = true;
    try {
      await this._invoke("cancel_clip_indexing");
    } catch {
      // Best-effort cooperative cancellation
    }
    if (this._status === "indexing") {
      this._status = "canceled";
    }
  }

  reset(modelChanged = false): void {
    if (this.isIndexing) {
      void this.stopIndexing();
    }
    this._isCanceled = false;
    this._status = "idle";
    this._errorMessage = null;
    if (modelChanged) {
      this._failedCount = 0;
    }
  }
}
