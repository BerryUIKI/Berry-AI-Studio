<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { AppInfo } from "../types";
import {
  clearThumbnailCache,
  getThumbnailCacheBudgetMb,
  getThumbnailCacheStats,
  getThumbnailMaxEdge,
  setThumbnailMaxEdge,
  setThumbnailCacheBudgetMb,
  type ThumbnailCacheStats,
} from "../utils/thumbnail";
import { formatBytes } from "../utils/image";
import {
  currentLocaleSetting,
  setLocale,
  SUPPORTED_LOCALES,
  t,
  type LocaleSetting,
} from "../i18n";
import {
  loadAppConfig,
  saveAppConfig,
  getStoragePaths,
  openStorageDir,
  resetSuppressedWarnings,
  type StoragePaths,
} from "../utils/config";
import { applyTheme, normalizeTheme, type AppTheme } from "../utils/theme";

const props = defineProps<{
  show: boolean;
  info: AppInfo | null;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save", settings: {
    locale: LocaleSetting;
    autoScan: boolean;
    startupScanIntervalMinutes: number;
    theme: AppTheme;
    blurNsfw: boolean;
    showCardBadges: boolean;
    defaultView: "grid" | "masonry" | "table";
    thumbnailMaxEdge: number;
    thumbnailCacheBudgetMb: number;
    autoCheckUpdate: boolean;
    allowMultipleStacksOpen: boolean;
  }): void;
}>();

const activeTab = ref<"general" | "display" | "stacking" | "parsers" | "about">("general");

// Settings state (backed by persistent config.json)
const selectedLocale = ref<LocaleSetting>(currentLocaleSetting.value);
const autoScanOnStartup = ref(false);
const startupScanIntervalMinutes = ref(360);
const selectedTheme = ref<AppTheme>(normalizeTheme(localStorage.getItem("berry_theme")));
const autoCheckUpdate = ref(true);
const blurNsfwDefault = ref(true);
const showCardBadges = ref(true);
const defaultView = ref<"grid" | "masonry" | "table">("grid");
const thumbnailMaxEdge = ref(getThumbnailMaxEdge());
const thumbnailCacheBudgetMb = ref(getThumbnailCacheBudgetMb());
const autoStack = ref(false);
const stackSimilarityThreshold = ref(0.85);
const stackTimeWindowMinutes = ref(180);
const allowMultipleStacksOpen = ref(false);
const suppressedWarningCount = ref(0);
const resettingWarnings = ref(false);
const warningResetMessage = ref("");

// Storage paths state
const storagePaths = ref<StoragePaths | null>(null);

// Cache stats
const cacheStats = ref<ThumbnailCacheStats | null>(null);
const clearingCache = ref(false);
const cacheMessage = ref("");

async function loadCacheStats() {
  try {
    cacheStats.value = await getThumbnailCacheStats();
  } catch (e) {
    console.error("Failed to load thumbnail cache stats:", e);
  }
}

async function loadSettingsAndPaths() {
  try {
    const config = await loadAppConfig();
    selectedLocale.value = (config.locale as LocaleSetting) || currentLocaleSetting.value;
    autoScanOnStartup.value = config.auto_scan;
    startupScanIntervalMinutes.value = config.startup_scan_interval_minutes ?? 360;
    selectedTheme.value = normalizeTheme(config.theme);
    autoCheckUpdate.value = config.auto_check_update;
    blurNsfwDefault.value = config.blur_nsfw;
    showCardBadges.value = config.show_card_badges;
    defaultView.value = config.default_view || "grid";
    thumbnailMaxEdge.value = config.thumbnail_max_edge || getThumbnailMaxEdge();
    thumbnailCacheBudgetMb.value =
      config.thumbnail_cache_budget_mb || getThumbnailCacheBudgetMb();
    autoStack.value = config.auto_stack ?? false;
    stackSimilarityThreshold.value = config.stack_similarity_threshold ?? 0.85;
    stackTimeWindowMinutes.value = config.stack_time_window_minutes ?? 180;
    allowMultipleStacksOpen.value = config.allow_multiple_open_stacks ?? false;
    suppressedWarningCount.value = config.suppressed_warnings.length;
    warningResetMessage.value = "";

    storagePaths.value = await getStoragePaths();
  } catch (e) {
    console.warn("Failed to load config from config.json:", e);
  }
}

async function handleClearCache() {
  clearingCache.value = true;
  cacheMessage.value = "";
  try {
    const count = await clearThumbnailCache();
    cacheMessage.value = `✓ ${count} ${t.value.settings.thumbnailsCount}`;
    await loadCacheStats();
  } catch (e) {
    cacheMessage.value = `${e}`;
  } finally {
    clearingCache.value = false;
  }
}

async function handleResetWarnings() {
  resettingWarnings.value = true;
  warningResetMessage.value = "";
  try {
    const resetCount = await resetSuppressedWarnings();
    suppressedWarningCount.value = 0;
    warningResetMessage.value = resetCount > 0
      ? t.value.settings.warningsReset
      : t.value.settings.noSuppressedWarnings;
  } catch (e) {
    warningResetMessage.value = String(e);
  } finally {
    resettingWarnings.value = false;
  }
}

function handleOpenDir(target: "config" | "database" | "thumbnails" | "models" | "data") {
  void openStorageDir(target);
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      void loadSettingsAndPaths();
      void loadCacheStats();
    }
  },
);

onMounted(() => {
  if (props.show) {
    void loadSettingsAndPaths();
    void loadCacheStats();
  }
});

async function saveSettings() {
  setLocale(selectedLocale.value);
  setThumbnailMaxEdge(thumbnailMaxEdge.value);
  setThumbnailCacheBudgetMb(thumbnailCacheBudgetMb.value);
  applyTheme(selectedTheme.value);

  // Write to persistent config.json
  try {
    const existing = await loadAppConfig();
    await saveAppConfig({
      ...existing,
      locale: selectedLocale.value,
      auto_scan: autoScanOnStartup.value,
      startup_scan_interval_minutes: startupScanIntervalMinutes.value,
      theme: selectedTheme.value,
      blur_nsfw: blurNsfwDefault.value,
      show_card_badges: showCardBadges.value,
      default_view: defaultView.value,
      thumbnail_max_edge: thumbnailMaxEdge.value,
      thumbnail_cache_budget_mb: thumbnailCacheBudgetMb.value,
      similarity_limit: Number(localStorage.getItem("berry_similarity_limit")) || 50,
      auto_check_update: autoCheckUpdate.value,
      silent_install: localStorage.getItem("berry_silent_install") === "true",
      auto_stack: autoStack.value,
      stack_similarity_threshold: stackSimilarityThreshold.value,
      stack_time_window_minutes: stackTimeWindowMinutes.value,
      allow_multiple_open_stacks: allowMultipleStacksOpen.value,
    });
  } catch (e) {
    console.error("Failed to save config.json:", e);
  }
  void loadCacheStats();

  emit("save", {
    locale: selectedLocale.value,
    autoScan: autoScanOnStartup.value,
    startupScanIntervalMinutes: startupScanIntervalMinutes.value,
    theme: selectedTheme.value,
    blurNsfw: blurNsfwDefault.value,
    showCardBadges: showCardBadges.value,
    defaultView: defaultView.value,
    thumbnailMaxEdge: thumbnailMaxEdge.value,
    thumbnailCacheBudgetMb: thumbnailCacheBudgetMb.value,
    autoCheckUpdate: autoCheckUpdate.value,
    allowMultipleStacksOpen: allowMultipleStacksOpen.value,
  });
  emit("close");
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="settings-dialog" role="dialog" aria-modal="true" :aria-label="t.settings.title">
      <!-- Header -->
      <div class="dialog-header">
        <div class="header-left">
          <span class="dialog-icon">⚙️</span>
          <h3 class="dialog-title">{{ t.settings.title }}</h3>
        </div>
        <button type="button" class="close-btn" :aria-label="t.settings.cancel" @click="emit('close')">✕</button>
      </div>

      <!-- Body: Left Tabs + Right Content -->
      <div class="dialog-body">
        <aside class="settings-tabs" role="tablist" :aria-label="t.settings.title">
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'general' }"
            role="tab"
            :aria-selected="activeTab === 'general'"
            @click="activeTab = 'general'"
          >
            <span aria-hidden="true">⚙</span><span>{{ t.settings.tabs.general }}</span>
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'display' }"
            role="tab"
            :aria-selected="activeTab === 'display'"
            @click="activeTab = 'display'"
          >
            <span aria-hidden="true">▦</span><span>{{ t.settings.tabs.display }}</span>
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'stacking' }"
            role="tab"
            :aria-selected="activeTab === 'stacking'"
            @click="activeTab = 'stacking'"
          >
            <span aria-hidden="true">▱</span><span>{{ t.settings.tabs.stacking || 'Stacks' }}</span>
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'parsers' }"
            role="tab"
            :aria-selected="activeTab === 'parsers'"
            @click="activeTab = 'parsers'"
          >
            <span aria-hidden="true">⌘</span><span>{{ t.settings.tabs.parsers }}</span>
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'about' }"
            role="tab"
            :aria-selected="activeTab === 'about'"
            @click="activeTab = 'about'"
          >
            <span aria-hidden="true">ⓘ</span><span>{{ t.settings.tabs.about }}</span>
          </button>
        </aside>

        <section class="settings-content">
          <!-- Tab: General -->
          <div v-if="activeTab === 'general'" class="settings-panel">
            <div class="panel-heading">
              <h4 class="panel-title">{{ t.settings.generalTitle }}</h4>
              <p class="panel-subtitle">{{ t.settings.generalSubtitle }}</p>
            </div>

            <!-- Language Setting -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.language }}</span>
                <span class="row-desc">{{ t.settings.languageDesc }}</span>
              </div>
              <select v-model="selectedLocale" class="select-input">
                <option
                  v-for="loc in SUPPORTED_LOCALES"
                  :key="loc.key"
                  :value="loc.key"
                >
                  {{ loc.label }}
                </option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.defaultView }}</span>
                <span class="row-desc">{{ t.settings.defaultViewDesc }}</span>
              </div>
              <select v-model="defaultView" class="select-input">
                <option value="grid">{{ t.settings.viewGrid }}</option>
                <option value="masonry">{{ t.settings.viewMasonry }}</option>
                <option value="table">{{ t.settings.viewTable }}</option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.autoScan }}</span>
                <span class="row-desc">{{ t.settings.autoScanDesc }}</span>
              </div>
              <input v-model="autoScanOnStartup" type="checkbox" class="toggle-checkbox" />
            </div>

            <div v-if="autoScanOnStartup" class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.startupScanInterval }}</span>
                <span class="row-desc">{{ t.settings.startupScanIntervalDesc }}</span>
              </div>
              <select v-model.number="startupScanIntervalMinutes" class="select-input">
                <option :value="30">{{ t.settings.scan30Minutes }}</option>
                <option :value="60">{{ t.settings.scan1Hour }}</option>
                <option :value="360">{{ t.settings.scan6Hours }}</option>
                <option :value="1440">{{ t.settings.scan24Hours }}</option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.autoCheckUpdate }}</span>
                <span class="row-desc">{{ t.settings.autoCheckUpdateDesc }}</span>
              </div>
              <input v-model="autoCheckUpdate" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row immediate-action-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.suppressedWarnings }}</span>
                <span class="row-desc">{{ t.settings.suppressedWarningsDesc }}</span>
                <span v-if="warningResetMessage" class="setting-feedback">
                  {{ warningResetMessage }}
                </span>
              </div>
              <button
                type="button"
                class="btn secondary"
                :disabled="resettingWarnings || suppressedWarningCount === 0"
                @click="handleResetWarnings"
              >
                {{ t.settings.resetWarnings }}
              </button>
            </div>
          </div>

          <!-- Tab: Display & Safety -->
          <div v-if="activeTab === 'display'" class="settings-panel">
            <div class="panel-heading">
              <h4 class="panel-title">{{ t.settings.displayTitle }}</h4>
              <p class="panel-subtitle">{{ t.settings.displaySubtitle }}</p>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.theme }}</span>
                <span class="row-desc">{{ t.settings.themeDesc }}</span>
              </div>
              <select v-model="selectedTheme" class="select-input">
                <option value="system">{{ t.settings.themeSystem }}</option>
                <option value="midnight">{{ t.settings.themeMidnight }}</option>
                <option value="graphite">{{ t.settings.themeGraphite }}</option>
                <option value="violet">{{ t.settings.themeViolet }}</option>
                <option value="light">{{ t.settings.themeLight }}</option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.blurNsfw }}</span>
                <span class="row-desc">{{ t.settings.blurNsfwDesc }}</span>
              </div>
              <input v-model="blurNsfwDefault" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.showBadges }}</span>
                <span class="row-desc">{{ t.settings.showBadgesDesc }}</span>
              </div>
              <input v-model="showCardBadges" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.thumbResolution }}</span>
                <span class="row-desc">{{ t.settings.thumbResolutionDesc }}</span>
              </div>
              <select v-model.number="thumbnailMaxEdge" class="select-input">
                <option :value="256">{{ t.settings.thumbCompact }}</option>
                <option :value="384">{{ t.settings.thumbStandard }}</option>
                <option :value="448">{{ t.settings.thumbHd }}</option>
                <option :value="512">{{ t.settings.thumbUltra }}</option>
              </select>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.thumbnailCacheBudget }}</span>
                <span class="row-desc">{{ t.settings.thumbnailCacheBudgetDesc }}</span>
              </div>
              <select v-model.number="thumbnailCacheBudgetMb" class="select-input">
                <option :value="512">512 MB</option>
                <option :value="1024">1 GB</option>
                <option :value="2048">2 GB</option>
                <option :value="4096">4 GB</option>
                <option :value="8192">8 GB</option>
                <option :value="16384">16 GB</option>
              </select>
            </div>

            <div class="setting-row immediate-action-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.cacheManagement }}</span>
                <span class="row-desc">
                  {{ t.settings.currentUsage }}
                  <strong style="color:#12b5cb;">
                    {{ cacheStats ? `${formatBytes(cacheStats.total_bytes)} / ${formatBytes(cacheStats.budget_bytes)} (${cacheStats.file_count} ${t.settings.thumbnailsCount})` : t.settings.calculating }}
                  </strong>
                  <span v-if="cacheMessage" style="margin-left: 8px; color: #4ade80;">{{ cacheMessage }}</span>
                </span>
              </div>
              <button
                type="button"
                class="btn secondary"
                :disabled="clearingCache"
                @click="handleClearCache"
              >
                {{ clearingCache ? t.settings.clearing : t.settings.clearCache }}
              </button>
            </div>
          </div>

          <!-- Tab: Stacking & Bursts -->
          <div v-if="activeTab === 'stacking'" class="settings-panel">
            <div class="panel-heading">
              <h4 class="panel-title">{{ t.settings.stackingTitle || 'Image Stacking & Burst Grouping' }}</h4>
              <p class="panel-subtitle">{{ t.settings.stackingSubtitle }}</p>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.allowMultipleStacks }}</span>
                <span class="row-desc">{{ t.settings.allowMultipleStacksDesc }}</span>
              </div>
              <input v-model="allowMultipleStacksOpen" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.autoStack || 'Enable Automatic Stacking' }}</span>
                <span class="row-desc">{{ t.settings.autoStackDesc || 'Automatically group consecutive images generated with identical or similar prompts into stacked cards' }}</span>
              </div>
              <input v-model="autoStack" type="checkbox" class="toggle-checkbox" />
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.stackThreshold || 'Prompt Similarity Threshold' }}</span>
                <span class="row-desc">{{ t.settings.stackThresholdDesc || 'Minimum tokenized prompt similarity to group images (Current: ' + Math.round(stackSimilarityThreshold * 100) + '%)' }}</span>
              </div>
              <div style="display: flex; align-items: center; gap: 8px;">
                <input
                  v-model.number="stackSimilarityThreshold"
                  type="range"
                  min="0.5"
                  max="1.0"
                  step="0.05"
                  class="range-input"
                />
                <span style="font-size: 0.85em; min-width: 40px;">{{ Math.round(stackSimilarityThreshold * 100) }}%</span>
              </div>
            </div>

            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.stackTimeWindow || 'Max Time Window Between Generations' }}</span>
                <span class="row-desc">{{ t.settings.stackTimeWindowDesc || 'Group images only if generated within this time range' }}</span>
              </div>
              <select v-model.number="stackTimeWindowMinutes" class="select-input">
                <option :value="30">30 minutes</option>
                <option :value="60">1 hour</option>
                <option :value="180">3 hours</option>
                <option :value="360">6 hours</option>
                <option :value="1440">24 hours</option>
              </select>
            </div>
          </div>

          <!-- Tab: Parsers -->
          <div v-if="activeTab === 'parsers'" class="settings-panel">
            <h4 class="panel-title">{{ t.settings.parsersTitle }}</h4>
            <p class="panel-subtitle">{{ t.settings.parsersSubtitle }}</p>

            <div class="parser-list">
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">WebUI (AUTOMATIC1111 / SD.Next)</span>
                <span class="parser-desc">PNG tEXt/iTXt (parameters), WebP EXIF</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">ComfyUI</span>
                <span class="parser-desc">Prompt & Workflow JSON Graph</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">NovelAI</span>
                <span class="parser-desc">Comment / Description / Software Signature</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">Fooocus / Fooocus-MRE</span>
                <span class="parser-desc">Fooocus Parameters & Base Model Parser</span>
              </div>
              <div class="parser-item">
                <span class="parser-badge active">{{ t.settings.enabled }}</span>
                <span class="parser-name">InvokeAI & EasyDiffusion</span>
                <span class="parser-desc">Invoke Metadata & JSON Sidecar</span>
              </div>
            </div>
          </div>

          <!-- Tab: About & Storage -->
          <div v-if="activeTab === 'about'" class="settings-panel">
            <div class="panel-heading">
              <h4 class="panel-title">{{ t.settings.aboutTitle }}</h4>
              <p class="panel-subtitle">{{ t.settings.aboutSubtitle }}</p>
            </div>

            <div class="about-card">
              <div class="about-logo">
                <img src="../assets/logo.png" alt="Berry Logo" width="48" height="48" class="about-logo-img" />
              </div>
              <div class="about-details">
                <h5 class="about-name">Berry AI Studio</h5>
                <p class="about-ver">v{{ info?.app_version || '0.1.3' }}</p>
                <p class="about-desc">{{ t.settings.aboutDesc }}</p>
              </div>
            </div>

            <!-- Storage Locations Card -->
            <div class="storage-section">
              <h5 class="storage-section-title">{{ t.settings.storageTitle }}</h5>

              <div class="storage-notice-box">
                <span class="notice-icon">🛡️</span>
                <span>{{ t.settings.storageNotice }}</span>
              </div>

              <!-- Config File -->
              <div class="storage-item-row">
                <div class="storage-item-info">
                  <span class="storage-item-label">{{ t.settings.configFile }}</span>
                  <span class="storage-item-path" :title="storagePaths?.config_file">{{ storagePaths?.config_file || '—' }}</span>
                </div>
                <button type="button" class="btn secondary mini-action-btn" @click="handleOpenDir('config')">
                  📁 {{ t.settings.openFolder }}
                </button>
              </div>

              <!-- Database File -->
              <div class="storage-item-row">
                <div class="storage-item-info">
                  <span class="storage-item-label">{{ t.settings.databaseFile }}</span>
                  <span class="storage-item-path" :title="storagePaths?.database_file">{{ storagePaths?.database_file || info?.database_path || '—' }}</span>
                </div>
                <button type="button" class="btn secondary mini-action-btn" @click="handleOpenDir('database')">
                  📁 {{ t.settings.openFolder }}
                </button>
              </div>

              <!-- Thumbnails Cache -->
              <div class="storage-item-row">
                <div class="storage-item-info">
                  <span class="storage-item-label">{{ t.settings.thumbnailsDir }}</span>
                  <span class="storage-item-path" :title="storagePaths?.thumbnails_dir">{{ storagePaths?.thumbnails_dir || '—' }}</span>
                </div>
                <button type="button" class="btn secondary mini-action-btn" @click="handleOpenDir('thumbnails')">
                  📁 {{ t.settings.openFolder }}
                </button>
              </div>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="dialog-footer">
        <button type="button" class="btn secondary" @click="emit('close')">{{ t.settings.cancel }}</button>
        <button type="button" class="btn primary" @click="saveSettings">{{ t.settings.save }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  user-select: none;
}

.settings-dialog {
  width: min(840px, 92vw);
  height: min(640px, 88vh);
  background: var(--color-bg-primary);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
  overflow: hidden;
}

.dialog-header {
  min-height: 62px;
  padding: 0 22px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dialog-icon {
  display: grid;
  place-items: center;
  width: 30px;
  height: 30px;
  border-radius: 8px;
  background: rgba(18, 181, 203, 0.14);
  font-size: 1rem;
}

.dialog-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: #f1f5f9;
}

.close-btn {
  background: transparent;
  border: none;
  color: #71717a;
  cursor: pointer;
  width: 32px;
  height: 32px;
  border-radius: 7px;
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s;
}

.close-btn:hover {
  color: #ffffff;
  background: rgba(255, 255, 255, 0.07);
}

.dialog-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

.settings-tabs {
  width: 205px;
  background: var(--color-bg-app);
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  padding: 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.tab-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  padding: 10px 12px;
  border-radius: 8px;
  font-size: 0.8rem;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  transition: all 0.12s ease;
  display: flex;
  align-items: center;
  gap: 10px;
}

.tab-btn > span:first-child {
  width: 18px;
  color: #64748b;
  text-align: center;
  font-size: 0.9rem;
}

.tab-btn:hover {
  background: rgba(255, 255, 255, 0.04);
  color: #f1f5f9;
}

.tab-btn.active {
  background: rgba(18, 181, 203, 0.18);
  color: #67e8f9;
  font-weight: 600;
}

.tab-btn.active > span:first-child {
  color: #67e8f9;
}

.tab-btn:focus-visible,
.close-btn:focus-visible,
.btn:focus-visible,
.select-input:focus-visible,
.toggle-checkbox:focus-visible,
.range-input:focus-visible {
  outline: 2px solid #22d3ee;
  outline-offset: 2px;
}

.settings-content {
  flex: 1;
  padding: 24px 26px 30px;
  overflow-y: auto;
}

.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 660px;
}

.panel-heading {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-bottom: 4px;
}

.panel-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: #f8fafc;
}

.panel-subtitle {
  margin: 0;
  font-size: 0.76rem;
  line-height: 1.45;
  color: #8b95a7;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 54px;
  padding: 12px 14px;
  background: var(--color-bg-secondary);
  border-radius: 9px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.setting-row:hover {
  border-color: rgba(255, 255, 255, 0.1);
}

.setting-row.immediate-action-row {
  margin-top: 6px;
  background: rgba(18, 181, 203, 0.045);
  border-style: dashed;
}

.row-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  padding-right: 8px;
}

.row-label {
  font-size: 0.8rem;
  font-weight: 500;
  color: #f1f5f9;
}

.row-desc {
  font-size: 0.71rem;
  line-height: 1.35;
  color: #8b95a7;
}

.setting-feedback {
  margin-top: 3px;
  font-size: 0.68rem;
  color: #4ade80;
}

.btn:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.path-code {
  font-family: monospace;
  word-break: break-all;
}

.select-input {
  background: var(--color-bg-primary);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #e2e8f0;
  border-radius: 5px;
  min-width: 178px;
  padding: 7px 30px 7px 10px;
  font-size: 0.75rem;
  outline: none;
}

.toggle-checkbox {
  appearance: none;
  position: relative;
  flex: 0 0 auto;
  width: 38px;
  height: 22px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 999px;
  background: #34343b;
  cursor: pointer;
  transition: background 0.16s ease, border-color 0.16s ease;
}

.toggle-checkbox::after {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #cbd5e1;
  transition: transform 0.16s ease;
}

.toggle-checkbox:checked {
  background: #0891a5;
  border-color: #22d3ee;
}

.toggle-checkbox:checked::after {
  background: #fff;
  transform: translateX(16px);
}

.parser-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.parser-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  background: var(--color-bg-secondary);
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 0.74rem;
}

.parser-badge {
  font-size: 0.66rem;
  padding: 1px 6px;
  border-radius: 4px;
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
  font-weight: 600;
}

.parser-name {
  font-weight: 500;
  color: #f1f5f9;
}

.parser-desc {
  color: #71717a;
  margin-left: auto;
  font-size: 0.68rem;
}

.about-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px;
  background: var(--color-bg-secondary);
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.about-logo {
  display: flex;
  align-items: center;
  justify-content: center;
}

.about-logo-img {
  display: block;
  object-fit: contain;
}

.about-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.about-name {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 700;
  color: #f8fafc;
}

.about-ver {
  margin: 0;
  font-size: 0.72rem;
  color: #12b5cb;
  font-weight: 500;
}

.about-desc {
  margin: 2px 0 0;
  font-size: 0.72rem;
  color: #94a3b8;
}

.dialog-footer {
  min-height: 60px;
  padding: 0 22px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: var(--color-bg-app);
}

.btn {
  min-height: 32px;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.76rem;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.12s ease;
  border: none;
}

.btn.secondary {
  background: rgba(255, 255, 255, 0.05);
  color: #cbd5e1;
}

.btn.secondary:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
}

.btn.primary {
  background: #12b5cb;
  color: #ffffff;
  font-weight: 500;
  min-width: 88px;
}

.btn.primary:hover {
  background: #0e9aa7;
}

/* Storage Section in Settings */
.storage-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  background: var(--color-bg-primary);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 14px;
}

.storage-section-title {
  margin: 0;
  font-size: 0.82rem;
  font-weight: 600;
  color: #f1f5f9;
}

.storage-notice-box {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: rgba(16, 185, 129, 0.08);
  border: 1px solid rgba(16, 185, 129, 0.2);
  border-radius: 6px;
  padding: 8px 12px;
  font-size: 0.72rem;
  color: #a7f3d0;
  line-height: 1.4;
}

.storage-item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 10px;
  background: var(--color-bg-secondary);
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.04);
}

.storage-item-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.storage-item-label {
  font-size: 0.74rem;
  font-weight: 500;
  color: #e2e8f0;
}

.storage-item-path {
  font-size: 0.68rem;
  font-family: monospace;
  color: #12b5cb;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mini-action-btn {
  font-size: 0.7rem;
  padding: 4px 10px;
  flex-shrink: 0;
  white-space: nowrap;
}

@media (max-width: 720px) {
  .settings-dialog {
    width: 94vw;
    height: 92vh;
  }

  .dialog-body {
    flex-direction: column;
  }

  .settings-tabs {
    box-sizing: border-box;
    width: 100%;
    flex-direction: row;
    overflow-x: auto;
    padding: 9px 12px;
    border-right: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .tab-btn {
    flex: 0 0 auto;
    white-space: nowrap;
  }

  .settings-content {
    padding: 18px;
  }

  .setting-row,
  .storage-item-row {
    align-items: flex-start;
    flex-direction: column;
  }

  .select-input {
    width: 100%;
  }
}

@media (prefers-reduced-motion: reduce) {
  .tab-btn,
  .btn,
  .toggle-checkbox,
  .toggle-checkbox::after,
  .close-btn {
    transition: none;
  }
}
</style>
