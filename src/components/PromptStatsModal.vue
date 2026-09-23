<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import type { DatabaseStats, PromptKeywordStat, PromptStat, PromptStats } from "../types";

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
  "update:open": [value: boolean];
  applySearch: [query: string];
}>();

const loading = ref(false);
const error = ref("");
const stats = ref<PromptStats | null>(null);
const activeTab = ref<"positive" | "negative" | "models" | "samplers">("positive");

async function loadStats() {
  loading.value = true;
  error.value = "";
  try {
    const [pos, neg, dbStats] = await Promise.all([
      invoke<PromptStat[]>("get_prompt_stats", { isNegative: false, limit: 40 }),
      invoke<PromptStat[]>("get_prompt_stats", { isNegative: true, limit: 40 }),
      invoke<DatabaseStats>("get_database_stats").catch(() => null),
    ]);
    const positiveWords: PromptKeywordStat[] = (pos || []).map((p) => ({
      keyword: p.text,
      count: p.count,
    }));
    const negativeWords: PromptKeywordStat[] = (neg || []).map((p) => ({
      keyword: p.text,
      count: p.count,
    }));
    stats.value = {
      total_analyzed: dbStats?.file_count ?? positiveWords.reduce((acc, p) => acc + p.count, 0),
      top_positive_words: positiveWords,
      top_negative_words: negativeWords,
      top_models: [],
      top_samplers: [],
    };
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function close() {
  emit("close");
  emit("update:open", false);
}

function handleBackdrop(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    close();
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.open) {
    e.stopPropagation();
    close();
  }
}

watch(
  () => props.open,
  (val) => {
    if (val) {
      void loadStats();
      window.addEventListener("keydown", handleKeydown);
    } else {
      window.removeEventListener("keydown", handleKeydown);
    }
  },
  { immediate: true },
);

onMounted(() => {
  if (props.open) {
    window.addEventListener("keydown", handleKeydown);
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});

const currentItems = computed<PromptKeywordStat[]>(() => {
  if (!stats.value) return [];
  switch (activeTab.value) {
    case "positive":
      return stats.value.top_positive_words;
    case "negative":
      return stats.value.top_negative_words;
    case "models":
      return stats.value.top_models;
    case "samplers":
      return stats.value.top_samplers;
  }
});

const maxCount = computed(() => {
  const items = currentItems.value;
  if (!items.length) return 1;
  return Math.max(...items.map((i) => i.count), 1);
});

function onSelectKeyword(item: PromptKeywordStat) {
  let query = "";
  switch (activeTab.value) {
    case "positive":
      query = item.keyword.includes(" ") ? `prompt:"${item.keyword}"` : `prompt:${item.keyword}`;
      break;
    case "negative":
      query = item.keyword.includes(" ") ? `neg:"${item.keyword}"` : `neg:${item.keyword}`;
      break;
    case "models":
      query = item.keyword.includes(" ") ? `model:"${item.keyword}"` : `model:${item.keyword}`;
      break;
    case "samplers":
      query = item.keyword.includes(" ") ? `sampler:"${item.keyword}"` : `sampler:${item.keyword}`;
      break;
  }
  emit("applySearch", query);
  close();
}
</script>

<template>
  <div
    v-if="open"
    class="stats-backdrop"
    @click="handleBackdrop"
  >
    <div
      class="stats-dialog"
      role="dialog"
      aria-modal="true"
      :aria-label="t.promptStatsModal.title"
    >
      <!-- Header -->
      <header class="stats-header">
        <div class="header-left">
          <span class="header-icon">📊</span>
          <h2>{{ t.promptStatsModal.title }}</h2>
        </div>
        <div class="header-actions">
          <button
            type="button"
            class="refresh-btn"
            title="Refresh statistics"
            :disabled="loading"
            @click="loadStats"
          >
            {{ loading ? "..." : "↻ Refresh" }}
          </button>
          <button
            type="button"
            class="close-btn"
            title="Close (Esc)"
            @click="close"
          >
            ✕
          </button>
        </div>
      </header>

      <!-- KPI Summary Cards -->
      <div v-if="stats" class="kpi-grid">
        <div class="kpi-card">
          <div class="kpi-title">{{ t.promptStatsModal.totalAnalyzed }}</div>
          <div class="kpi-value">{{ stats.total_analyzed.toLocaleString() }}</div>
          <div class="kpi-sub">{{ t.view.files }}</div>
        </div>

        <div class="kpi-card">
          <div class="kpi-title">{{ t.promptStatsModal.topPositive }}</div>
          <div class="kpi-value highlight-blue">
            {{ stats.top_positive_words[0]?.keyword || "—" }}
          </div>
          <div class="kpi-sub">
            {{ stats.top_positive_words[0]?.count ? `${stats.top_positive_words[0].count} ${t.promptStatsModal.occurrences}` : "—" }}
          </div>
        </div>

        <div class="kpi-card">
          <div class="kpi-title">{{ t.promptStatsModal.topNegative }}</div>
          <div class="kpi-value highlight-red">
            {{ stats.top_negative_words[0]?.keyword || "—" }}
          </div>
          <div class="kpi-sub">
            {{ stats.top_negative_words[0]?.count ? `${stats.top_negative_words[0].count} ${t.promptStatsModal.occurrences}` : "—" }}
          </div>
        </div>

        <div class="kpi-card">
          <div class="kpi-title">{{ t.promptStatsModal.topModels }}</div>
          <div class="kpi-value highlight-green" :title="stats.top_models[0]?.keyword">
            {{ stats.top_models[0]?.keyword || "—" }}
          </div>
          <div class="kpi-sub">
            {{ stats.top_models[0]?.count ? `${stats.top_models[0].count} ${t.view.files}` : "—" }}
          </div>
        </div>
      </div>

      <!-- Category Tabs -->
      <div class="tab-bar">
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'positive' }"
          @click="activeTab = 'positive'"
        >
          🌟 {{ t.promptStatsModal.topPositive }}
          <span v-if="stats?.top_positive_words.length" class="tab-badge">
            {{ stats.top_positive_words.length }}
          </span>
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'negative' }"
          @click="activeTab = 'negative'"
        >
          🚫 {{ t.promptStatsModal.topNegative }}
          <span v-if="stats?.top_negative_words.length" class="tab-badge">
            {{ stats.top_negative_words.length }}
          </span>
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'models' }"
          @click="activeTab = 'models'"
        >
          🤖 {{ t.promptStatsModal.topModels }}
          <span v-if="stats?.top_models.length" class="tab-badge">
            {{ stats.top_models.length }}
          </span>
        </button>
        <button
          type="button"
          class="tab-btn"
          :class="{ active: activeTab === 'samplers' }"
          @click="activeTab = 'samplers'"
        >
          ⚙️ {{ t.promptStatsModal.topSamplers }}
          <span v-if="stats?.top_samplers.length" class="tab-badge">
            {{ stats.top_samplers.length }}
          </span>
        </button>
      </div>

      <!-- Stats List / Chart Area -->
      <div class="stats-content">
        <div v-if="loading" class="loading-state">
          <div class="spinner"></div>
          <p>Extracting keyword distributions and ratings across your library…</p>
        </div>

        <div v-else-if="error" class="error-state">
          <p>{{ error }}</p>
        </div>

        <div v-else-if="currentItems.length === 0" class="empty-state">
          <p>No statistics available in this category.</p>
        </div>

        <div v-else class="items-list">
          <div
            v-for="(item, idx) in currentItems"
            :key="item.keyword"
            class="stat-row"
            title="Click to search this in library"
            @click="onSelectKeyword(item)"
          >
            <div class="stat-rank">{{ idx + 1 }}</div>
            <div class="stat-keyword" :title="item.keyword">{{ item.keyword }}</div>

            <!-- Bar representation -->
            <div class="stat-bar-container">
              <div
                class="stat-bar-fill"
                :class="activeTab"
                :style="{ width: `${Math.max(4, Math.round((item.count / maxCount) * 100))}%` }"
              ></div>
            </div>

            <!-- Rating badge if available -->
            <div v-if="item.avg_rating" class="stat-rating" title="Average Rating">
              ★ {{ item.avg_rating.toFixed(1) }}
            </div>

            <!-- Count badge -->
            <div class="stat-count-badge">
              {{ item.count }}
            </div>

            <button type="button" class="stat-search-hint" title="Filter library">
              🔍
            </button>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <footer class="stats-footer">
        <span class="footer-tip">
          💡 Click any keyword or model to filter library files.
        </span>
        <button type="button" class="btn-done" @click="close">
          Done
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.stats-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(4px);
  z-index: 1050;
  display: flex;
  align-items: center;
  justify-content: center;
}

.stats-dialog {
  width: 900px;
  max-width: 95vw;
  height: 85vh;
  background: #1e1e1e;
  color: #eee;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
  overflow: hidden;
}

.stats-header {
  height: 56px;
  padding: 0 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #181818;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.header-icon {
  font-size: 1.25rem;
}

.stats-header h2 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.refresh-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #ccc;
  font: inherit;
  font-size: 0.82em;
  padding: 0.35rem 0.75rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.refresh-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.close-btn {
  background: transparent;
  border: none;
  color: #888;
  font-size: 1.1em;
  cursor: pointer;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
}

.close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 0.75rem;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.kpi-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 0.75rem 1rem;
  display: flex;
  flex-direction: column;
}

.kpi-title {
  font-size: 0.75em;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #888;
}

.kpi-value {
  font-size: 1.25em;
  font-weight: 700;
  margin: 0.25rem 0 0.15rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.kpi-sub {
  font-size: 0.72em;
  color: #666;
}

.highlight-blue {
  color: #60a5fa;
}

.highlight-red {
  color: #f87171;
}

.highlight-green {
  color: #34d399;
}

.tab-bar {
  display: flex;
  gap: 0.5rem;
  padding: 0.75rem 1.25rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: #181818;
  flex-shrink: 0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #aaa;
  font: inherit;
  font-size: 0.82em;
  font-weight: 500;
  padding: 0.4rem 0.85rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  color: #eee;
  background: rgba(255, 255, 255, 0.05);
}

.tab-btn.active {
  background: #2f6fed;
  border-color: #2f6fed;
  color: #fff;
  font-weight: 600;
}

.tab-badge {
  background: rgba(0, 0, 0, 0.3);
  font-size: 0.8em;
  padding: 0.1rem 0.4rem;
  border-radius: 999px;
}

.stats-content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem 1.25rem;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.stat-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.45rem 0.75rem;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.stat-row:hover {
  background: rgba(47, 111, 237, 0.12);
  border-color: rgba(47, 111, 237, 0.4);
  transform: translateX(2px);
}

.stat-rank {
  width: 24px;
  font-size: 0.8em;
  font-weight: 600;
  color: #666;
  text-align: right;
  flex-shrink: 0;
}

.stat-keyword {
  width: 180px;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 0.85em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 0;
}

.stat-bar-container {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 999px;
  overflow: hidden;
}

.stat-bar-fill {
  height: 100%;
  border-radius: 999px;
  transition: width 0.3s ease-out;
}

.stat-bar-fill.positive {
  background: #3b82f6;
}

.stat-bar-fill.negative {
  background: #ef4444;
}

.stat-bar-fill.models {
  background: #10b981;
}

.stat-bar-fill.samplers {
  background: #8b5cf6;
}

.stat-rating {
  font-size: 0.76em;
  color: #fbbf24;
  background: rgba(245, 158, 11, 0.15);
  border: 1px solid rgba(245, 158, 11, 0.3);
  padding: 0.15rem 0.45rem;
  border-radius: 999px;
  white-space: nowrap;
  flex-shrink: 0;
}

.stat-count-badge {
  font-size: 0.8em;
  font-weight: 600;
  color: #ccc;
  background: rgba(255, 255, 255, 0.08);
  padding: 0.15rem 0.55rem;
  border-radius: 999px;
  white-space: nowrap;
  flex-shrink: 0;
  min-width: 36px;
  text-align: center;
}

.stat-search-hint {
  background: transparent;
  border: none;
  color: #666;
  font-size: 0.85em;
  cursor: pointer;
  padding: 0.15rem 0.3rem;
  opacity: 0.5;
  transition: opacity 0.15s ease;
}

.stat-row:hover .stat-search-hint {
  opacity: 1;
  color: #60a5fa;
}

.loading-state,
.error-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 1rem;
  color: #888;
  font-size: 0.9em;
}

.spinner {
  width: 28px;
  height: 28px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: #2f6fed;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.stats-footer {
  height: 52px;
  padding: 0 1.25rem;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #181818;
  flex-shrink: 0;
}

.footer-tip {
  font-size: 0.78em;
  color: #777;
}

.btn-done {
  background: #2f6fed;
  color: #fff;
  border: none;
  font: inherit;
  font-size: 0.85em;
  font-weight: 600;
  padding: 0.4rem 1.2rem;
  border-radius: 6px;
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.btn-done:hover {
  opacity: 0.9;
}
</style>
