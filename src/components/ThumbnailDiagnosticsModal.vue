<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import {
  getThumbnailDiagnostics,
  resetThumbnailDiagnostics,
  type ThumbnailDiagnosticsSummary,
} from "../utils/thumbnail";

defineProps<{
  show: boolean;
}>();


const emit = defineEmits<{
  (e: "close"): void;
}>();

const diag = ref<ThumbnailDiagnosticsSummary | null>(null);
const loading = ref(false);
const autoRefresh = ref(true);
let refreshTimer: ReturnType<typeof setInterval> | null = null;

async function refresh() {
  try {
    diag.value = await getThumbnailDiagnostics();
  } catch (e) {
    console.warn("Failed to fetch thumbnail diagnostics:", e);
  }
}

async function handleReset() {
  loading.value = true;
  try {
    await resetThumbnailDiagnostics();
    await refresh();
  } finally {
    loading.value = false;
  }
}

function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value;
  if (autoRefresh.value) {
    refreshTimer = setInterval(refresh, 1000);
  } else if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
}

onMounted(() => {
  refresh();
  if (autoRefresh.value) {
    refreshTimer = setInterval(refresh, 1000);
  }
});

onUnmounted(() => {
  if (refreshTimer) clearInterval(refreshTimer);
});
</script>

<template>
  <div v-if="show" class="diag-overlay" @click.self="emit('close')">
    <div class="diag-dialog">
      <div class="diag-header">
        <div class="diag-title-area">
          <h3 class="diag-title">⚡ Thumbnail Queue Diagnostics</h3>
          <span class="diag-badge">Dev / Diagnostic Mode</span>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div class="diag-body">
        <!-- Backend Queue Section -->
        <div class="diag-section">
          <h4 class="section-title">Backend Queue & Workers (Rayon Pool)</h4>
          <div class="metrics-grid">
            <div class="metric-card">
              <span class="metric-label">Queued Jobs</span>
              <span class="metric-val" :class="{ highlight: (diag?.backend.queued ?? 0) > 0 }">
                {{ diag?.backend.queued ?? 0 }}
              </span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Running Workers</span>
              <span class="metric-val running">{{ diag?.backend.running ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Completed</span>
              <span class="metric-val success">{{ diag?.backend.completed ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Canceled</span>
              <span class="metric-val warning">{{ diag?.backend.canceled ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Failed</span>
              <span class="metric-val danger">{{ diag?.backend.failed ?? 0 }}</span>
            </div>
          </div>
        </div>

        <!-- Generation & Cache Hits Section -->
        <div class="diag-section">
          <h4 class="section-title">Viewport Generation & Cache Hits</h4>
          <div class="metrics-grid">
            <div class="metric-card">
              <span class="metric-label">Active Generation</span>
              <span class="metric-val info">#{{ diag?.backend.active_generation ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Manifest Hits</span>
              <span class="metric-val">{{ diag?.backend.manifest_hits ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Larger Tier Reuses</span>
              <span class="metric-val success">{{ diag?.backend.reused_tier_hits ?? 0 }}</span>
            </div>
          </div>
        </div>

        <!-- Frontend Slicing & Memory LRU Section -->
        <div class="diag-section">
          <h4 class="section-title">Frontend Pipeline & In-Memory LRU</h4>
          <div class="metrics-grid">
            <div class="metric-card">
              <span class="metric-label">In-Flight Promises</span>
              <span class="metric-val">{{ diag?.frontend.inFlightCount ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Speculative Batch Queue</span>
              <span class="metric-val">{{ diag?.frontend.queuedBatchCount ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Memory LRU Cached</span>
              <span class="metric-val info">{{ diag?.frontend.memoryCacheSize ?? 0 }} / 3000</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Memory Cache Hits</span>
              <span class="metric-val success">{{ diag?.frontend.memoryCacheHits ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">In-Flight Dedupe Hits</span>
              <span class="metric-val success">{{ diag?.frontend.dedupeHits ?? 0 }}</span>
            </div>
            <div class="metric-card">
              <span class="metric-label">Dispatched Requests</span>
              <span class="metric-val">{{ diag?.frontend.requestsDispatched ?? 0 }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="diag-footer">
        <div class="footer-left">
          <button
            type="button"
            class="action-btn auto-btn"
            :class="{ active: autoRefresh }"
            @click="toggleAutoRefresh"
          >
            {{ autoRefresh ? "⏸ Pause Auto-Refresh" : "▶ Resume Auto-Refresh" }}
          </button>
        </div>
        <div class="footer-right">
          <button type="button" class="action-btn" :disabled="loading" @click="refresh">
            🔄 Refresh
          </button>
          <button type="button" class="action-btn danger-btn" :disabled="loading" @click="handleReset">
            🗑 Reset Counters
          </button>
          <button type="button" class="action-btn primary" @click="emit('close')">
            Done
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.diag-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2500;
  user-select: none;
}

.diag-dialog {
  width: min(720px, 94vw);
  max-height: 85vh;
  background: #1e1e24;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
  color: #e2e8f0;
  font-family: inherit;
  overflow: hidden;
}

.diag-header {
  padding: 14px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.diag-title-area {
  display: flex;
  align-items: center;
  gap: 10px;
}

.diag-title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: #f8fafc;
}

.diag-badge {
  font-size: 0.72rem;
  background: rgba(99, 102, 241, 0.2);
  color: #a5b4fc;
  border: 1px solid rgba(99, 102, 241, 0.4);
  padding: 2px 8px;
  border-radius: 9999px;
  text-transform: uppercase;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.06);
}

.diag-body {
  padding: 16px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.diag-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  margin: 0;
  font-size: 0.85rem;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 10px;
}

.metric-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metric-label {
  font-size: 0.75rem;
  color: #94a3b8;
}

.metric-val {
  font-size: 1.25rem;
  font-weight: 700;
  color: #f1f5f9;
}

.metric-val.running { color: #38bdf8; }
.metric-val.success { color: #4ade80; }
.metric-val.warning { color: #fbbf24; }
.metric-val.danger { color: #f87171; }
.metric-val.info { color: #818cf8; }
.metric-val.highlight { color: #f43f5e; }

.diag-footer {
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.2);
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.footer-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-btn {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #e2e8f0;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.primary {
  background: #4f46e5;
  border-color: #6366f1;
  color: #fff;
}

.action-btn.primary:hover:not(:disabled) {
  background: #4338ca;
}

.action-btn.danger-btn:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.2);
  border-color: #ef4444;
  color: #fca5a5;
}

.action-btn.auto-btn.active {
  color: #38bdf8;
  border-color: rgba(56, 189, 248, 0.4);
}
</style>
