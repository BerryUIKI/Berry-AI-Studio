<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import { getThumbnailDiagnostics, type ThumbnailDiagnosticsSummary } from "../utils/thumbnail";
import type { ScanProgress } from "../types";

export interface WatcherStatus {
  is_active: boolean;
  watched_roots_count: number;
  pending_journal_count: number;
  last_reconcile_time: number | null;
  last_error: string | null;
}

const props = defineProps<{
  open: boolean;
  scanProgress: ScanProgress | null;
  thumbProgress?: { current: number; total: number; active: boolean } | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "openThumbnailDiagnostics"): void;
}>();

const watcherStatus = ref<WatcherStatus | null>(null);
const thumbDiag = ref<ThumbnailDiagnosticsSummary | null>(null);
let pollTimer: ReturnType<typeof setInterval> | null = null;

async function refresh() {
  try {
    watcherStatus.value = await invoke<WatcherStatus>("get_watcher_status");
  } catch {
    // Graceful fallback if command fails
  }
  try {
    thumbDiag.value = await getThumbnailDiagnostics();
  } catch {
    // Ignore in case of dev/mock
  }
}

watch(
  () => props.open,
  (val) => {
    if (val) {
      refresh();
      if (!pollTimer) pollTimer = setInterval(refresh, 2000);
    } else if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  },
  { immediate: true },
);

onUnmounted(() => {
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
  }
});

const formattedLastReconcile = computed(() => {
  if (!watcherStatus.value?.last_reconcile_time) return "—";
  const date = new Date(watcherStatus.value.last_reconcile_time * 1000);
  return date.toLocaleTimeString();
});
</script>

<template>
  <div v-if="open" class="activity-popover-container" @click.stop>
    <div class="popover-header">
      <div class="header-title">
        <span class="icon">⚡</span>
        <span>{{ t.statusbar.activity }}</span>
      </div>
      <button type="button" class="close-btn" @click="emit('close')" aria-label="Close">✕</button>
    </div>

    <div class="popover-body">
      <!-- Section 1: Watcher Health -->
      <div class="activity-card">
        <div class="card-title">
          <div class="title-left">
            <span
              class="health-dot"
              :class="{
                healthy: watcherStatus?.is_active && !watcherStatus?.last_error,
                degraded: watcherStatus?.last_error,
                inactive: !watcherStatus?.is_active,
              }"
            ></span>
            <strong>{{ watcherStatus?.is_active ? t.statusbar.watcherActive : t.statusbar.ready }}</strong>
          </div>
          <span v-if="watcherStatus?.watched_roots_count" class="badge">
            {{ watcherStatus.watched_roots_count }} roots
          </span>
        </div>

        <div class="card-metrics">
          <div class="metric-row">
            <span class="label">Pending Changes:</span>
            <span class="value">{{ watcherStatus?.pending_journal_count ?? 0 }}</span>
          </div>
          <div class="metric-row">
            <span class="label">Last Reconciled:</span>
            <span class="value">{{ formattedLastReconcile }}</span>
          </div>
          <div v-if="watcherStatus?.last_error" class="metric-row error-row">
            <span class="label">Status:</span>
            <span class="value error-msg" :title="watcherStatus.last_error">
              {{ watcherStatus.last_error }}
            </span>
          </div>
        </div>
      </div>

      <!-- Section 2: Thumbnail Engine -->
      <div class="activity-card">
        <div class="card-title">
          <strong>Thumbnails</strong>
          <button
            type="button"
            class="action-link"
            @click="emit('openThumbnailDiagnostics'); emit('close')"
          >
            Diagnostics ↗
          </button>
        </div>

        <div class="card-metrics">
          <div class="metric-row">
            <span class="label">Active In-Flight:</span>
            <span class="value">{{ thumbDiag?.frontend.inFlightCount ?? 0 }}</span>
          </div>
          <div class="metric-row">
            <span class="label">Queued Batches:</span>
            <span class="value">{{ thumbDiag?.frontend.queuedBatchCount ?? 0 }}</span>
          </div>
          <div class="metric-row">
            <span class="label">Memory Cache:</span>
            <span class="value">{{ thumbDiag?.frontend.memoryCacheSize ?? 0 }} items</span>
          </div>
          <div class="metric-row">
            <span class="label">Cache Hits:</span>
            <span class="value">
              {{ (thumbDiag?.backend.reused_tier_hits ?? 0) + (thumbDiag?.backend.manifest_hits ?? 0) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Section 3: Library Scanner -->
      <div v-if="scanProgress && (scanProgress.discovering || scanProgress.found > 0)" class="activity-card">
        <div class="card-title">
          <strong>Scanner</strong>
          <span class="badge">{{ scanProgress.discovering ? "Discovering" : "Active" }}</span>
        </div>
        <div class="card-metrics">
          <div class="metric-row">
            <span class="label">Processed:</span>
            <span class="value">{{ scanProgress.scanned }} / {{ scanProgress.found }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.activity-popover-container {
  position: absolute;
  bottom: 32px;
  right: 12px;
  width: 320px;
  background: var(--color-bg-primary, #1e2025);
  border: 1px solid var(--border-color-strong, rgba(255, 255, 255, 0.16));
  border-radius: 8px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.45);
  z-index: 1000;
  overflow: hidden;
  font-family: inherit;
  color: var(--color-text-primary, #e2e8f0);
}

.popover-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--color-bg-secondary, #282b30);
  border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  font-size: 0.8rem;
  font-weight: 600;
}

.header-title {
  display: flex;
  align-items: center;
  gap: 6px;
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--color-text-muted, #94a3b8);
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--color-text-primary, #fff);
}

.popover-body {
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.activity-card {
  background: var(--color-bg-secondary, #25282e);
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.06));
  border-radius: 6px;
  padding: 8px 10px;
}

.card-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.78rem;
  margin-bottom: 6px;
}

.title-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.health-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #64748b;
}

.health-dot.healthy {
  background: #22c55e;
  box-shadow: 0 0 6px rgba(34, 197, 94, 0.6);
}

.health-dot.degraded {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.6);
}

.health-dot.inactive {
  background: #94a3b8;
}

.badge {
  font-size: 0.68rem;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.08);
  color: var(--color-text-secondary, #94a3b8);
}

.action-link {
  background: transparent;
  border: none;
  color: #38bdf8;
  font-size: 0.72rem;
  cursor: pointer;
  padding: 0;
}

.action-link:hover {
  text-decoration: underline;
}

.card-metrics {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.metric-row {
  display: flex;
  justify-content: space-between;
  font-size: 0.72rem;
}

.label {
  color: var(--color-text-muted, #94a3b8);
}

.value {
  font-weight: 500;
  color: var(--color-text-primary, #f1f5f9);
}

.error-msg {
  color: #f87171;
  max-width: 170px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
