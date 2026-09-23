<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppInfo,
  CloudBackupConfig,
  CloudBackupResult,
  CloudPingResult,
  CloudRestoreResult,
  CloudSnapshotMeta,
  CloudStorageProvider,
  CloudSyncProgress,
  CloudSyncResult,
  CloudSyncStrategy,
} from "../types";
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
import ThumbnailDiagnosticsModal from "./ThumbnailDiagnosticsModal.vue";
import MigrationWizardModal from "./MigrationWizardModal.vue";
import { checkServiceStatus } from "../utils/generation";
import { open } from "@tauri-apps/plugin-dialog";
import { collaborationSync, pingDatabase } from "../utils/collaborationSync";

const showDiagnosticsModal = ref(false);
const showMigrationWizardModal = ref(false);

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

const activeTab = ref<"general" | "display" | "stacking" | "interop" | "collaboration" | "cloudBackup" | "parsers" | "about">("general");

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
const comfyuiUrl = ref("http://127.0.0.1:8188");
const webuiUrl = ref("http://127.0.0.1:7860");
const comfyStatus = ref<"unknown" | "checking" | "online" | "offline">("unknown");
const webuiStatus = ref<"unknown" | "checking" | "online" | "offline">("unknown");

// Collaboration & Database state
const storageBackend = ref<"sqlite" | "mysql" | "postgres">("sqlite");
const remoteConnectionUrl = ref("");
const clientIdentifier = ref("local_client");
const rootMappings = ref<Record<string, string>>({});
const pingStatus = ref<"unknown" | "testing" | "success" | "error">("unknown");
const pingLatency = ref<number | null>(null);
const pingMessage = ref("");
const newMappingUuid = ref("");
const newMappingPath = ref("");
const lastSyncDisplay = ref("never");

async function handlePingDatabase() {
  pingStatus.value = "testing";
  pingMessage.value = "";
  pingLatency.value = null;
  try {
    const res = await pingDatabase(storageBackend.value, remoteConnectionUrl.value);
    if (res.success) {
      pingStatus.value = "success";
      pingLatency.value = res.latency_ms;
      pingMessage.value = res.message;
    } else {
      pingStatus.value = "error";
      pingMessage.value = res.message;
    }
  } catch (err: any) {
    pingStatus.value = "error";
    pingMessage.value = String(err);
  }
}

async function handleBrowseMappingPath() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    newMappingPath.value = selected;
  }
}

function addRootMappingEntry() {
  const uuid = newMappingUuid.value.trim();
  const path = newMappingPath.value.trim();
  if (uuid && path) {
    rootMappings.value = {
      ...rootMappings.value,
      [uuid]: path,
    };
    newMappingUuid.value = "";
    newMappingPath.value = "";
  }
}

function removeRootMappingEntry(uuid: string) {
  const next = { ...rootMappings.value };
  delete next[uuid];
  rootMappings.value = next;
}

async function checkComfyConnection() {
  comfyStatus.value = "checking";
  const ok = await checkServiceStatus(comfyuiUrl.value, "comfyui");
  comfyStatus.value = ok ? "online" : "offline";
}

async function checkWebuiConnection() {
  webuiStatus.value = "checking";
  const ok = await checkServiceStatus(webuiUrl.value, "webui");
  webuiStatus.value = ok ? "online" : "offline";
}

// Cloud Backup state
const cloudProvider = ref<CloudStorageProvider>("local_path");
const cloudLocalPath = ref("");
const cloudWebdavEndpoint = ref("");
const cloudWebdavUsername = ref("");
const cloudWebdavPassword = ref("");
const cloudS3Endpoint = ref("");
const cloudS3Bucket = ref("");
const cloudS3Region = ref("auto");
const cloudS3AccessKey = ref("");
const cloudS3SecretKey = ref("");
const cloudS3Prefix = ref("backups/");
const cloudAutoBackup = ref(false);
const cloudAutoIntervalDays = ref(7);

const cloudPingStatus = ref<"unknown" | "testing" | "success" | "error">("unknown");
const cloudPingLatency = ref<number | null>(null);
const cloudPingMessage = ref("");

const cloudSnapshots = ref<CloudSnapshotMeta[]>([]);
const cloudSnapshotsLoading = ref(false);
const cloudCreatingSnapshot = ref(false);
const cloudRestoringSnapshot = ref(false);
const cloudActionMessage = ref("");
const cloudActionError = ref("");
const newSnapshotDescription = ref("");

function getCurrentCloudConfig(): CloudBackupConfig {
  return {
    provider: cloudProvider.value,
    local_path: cloudLocalPath.value.trim() || null,
    webdav_endpoint: cloudWebdavEndpoint.value.trim() || null,
    webdav_username: cloudWebdavUsername.value.trim() || null,
    webdav_password: cloudWebdavPassword.value || null,
    s3_endpoint: cloudS3Endpoint.value.trim() || null,
    s3_bucket: cloudS3Bucket.value.trim() || null,
    s3_region: cloudS3Region.value.trim() || "auto",
    s3_access_key: cloudS3AccessKey.value.trim() || null,
    s3_secret_key: cloudS3SecretKey.value.trim() || null,
    s3_prefix: cloudS3Prefix.value.trim() || "backups/",
    auto_backup_enabled: cloudAutoBackup.value,
    auto_backup_interval_days: cloudAutoIntervalDays.value,
  };
}

async function handleBrowseLocalBackupPath() {
  const selected = await open({ directory: true, multiple: false });
  if (typeof selected === "string") {
    cloudLocalPath.value = selected;
  }
}

async function handleTestCloudConnection() {
  cloudPingStatus.value = "testing";
  cloudPingMessage.value = "";
  cloudPingLatency.value = null;
  try {
    const res = await invoke<CloudPingResult>("cloud_backup_test_connection", {
      config: getCurrentCloudConfig(),
    });
    if (res.success) {
      cloudPingStatus.value = "success";
      cloudPingLatency.value = res.latency_ms;
      cloudPingMessage.value = res.message;
    } else {
      cloudPingStatus.value = "error";
      cloudPingMessage.value = res.message;
    }
  } catch (err: any) {
    cloudPingStatus.value = "error";
    cloudPingMessage.value = String(err);
  }
}

async function handleCreateSnapshot() {
  cloudCreatingSnapshot.value = true;
  cloudActionMessage.value = "";
  cloudActionError.value = "";
  try {
    const res = await invoke<CloudBackupResult>("cloud_backup_create_snapshot", {
      config: getCurrentCloudConfig(),
      description: newSnapshotDescription.value.trim() || null,
    });
    if (res.success && res.snapshot) {
      cloudActionMessage.value = `✓ Snapshot created: ${res.snapshot.filename} (${(res.snapshot.size_bytes / 1024 / 1024).toFixed(2)} MB)`;
      newSnapshotDescription.value = "";
      await handleListSnapshots();
    } else {
      cloudActionError.value = res.error || "Failed to create snapshot";
    }
  } catch (err: any) {
    cloudActionError.value = String(err);
  } finally {
    cloudCreatingSnapshot.value = false;
  }
}

async function handleListSnapshots() {
  cloudSnapshotsLoading.value = true;
  cloudActionError.value = "";
  try {
    cloudSnapshots.value = await invoke<CloudSnapshotMeta[]>("cloud_backup_list_snapshots", {
      config: getCurrentCloudConfig(),
    });
  } catch (err: any) {
    cloudActionError.value = String(err);
  } finally {
    cloudSnapshotsLoading.value = false;
  }
}

async function handleRestoreSnapshot(snapshot: CloudSnapshotMeta) {
  const confirmed = window.confirm(
    t.value.settings.cloudBackup.restoreConfirm.replace('{name}', snapshot.filename)
  );
  if (!confirmed) return;

  cloudRestoringSnapshot.value = true;
  cloudActionMessage.value = "";
  cloudActionError.value = "";
  try {
    const res = await invoke<CloudRestoreResult>("cloud_backup_restore_snapshot", {
      config: getCurrentCloudConfig(),
      snapshotFilename: snapshot.filename,
    });
    if (res.success) {
      cloudActionMessage.value = `✓ Successfully restored! (${res.restored_files_count} files in ${res.duration_ms} ms). Reloading...`;
      setTimeout(() => {
        window.location.reload();
      }, 1200);
    } else {
      cloudActionError.value = res.error || "Restore failed";
    }
  } catch (err: any) {
    cloudActionError.value = String(err);
  } finally {
    cloudRestoringSnapshot.value = false;
  }
}

// Cloud Media Delta Sync state
const syncStrategy = ref<CloudSyncStrategy>("fast_fingerprint");
const syncConcurrency = ref<number>(4);
const syncBandwidthLimit = ref<number>(0);
const syncDryRun = ref<boolean>(false);
const syncRemotePrefix = ref<string>("media/");
const syncProgress = ref<CloudSyncProgress | null>(null);
const syncSummary = ref<CloudSyncResult | null>(null);
const syncStarting = ref<boolean>(false);
let unlistenSyncProgress: UnlistenFn | null = null;

async function handleStartCloudSync() {
  syncStarting.value = true;
  syncSummary.value = null;
  try {
    await invoke("cloud_sync_start", {
      config: getCurrentCloudConfig(),
      options: {
        strategy: syncStrategy.value,
        concurrency: syncConcurrency.value,
        bandwidth_limit_kbs: syncBandwidthLimit.value > 0 ? syncBandwidthLimit.value : null,
        dry_run: syncDryRun.value,
        remote_prefix: syncRemotePrefix.value,
      },
    });
    await fetchSyncProgress();
  } catch (err: any) {
    window.alert(String(err));
  } finally {
    syncStarting.value = false;
  }
}

async function handleCancelCloudSync() {
  try {
    await invoke("cloud_sync_cancel");
    await fetchSyncProgress();
  } catch (err: any) {
    console.error("Failed to cancel cloud sync:", err);
  }
}

async function fetchSyncProgress() {
  try {
    syncProgress.value = await invoke<CloudSyncProgress>("cloud_sync_get_progress");
    if (
      syncProgress.value &&
      (syncProgress.value.phase === "completed" ||
        syncProgress.value.phase === "cancelled" ||
        syncProgress.value.phase === "failed")
    ) {
      syncSummary.value = await invoke<CloudSyncResult | null>("cloud_sync_get_summary");
    }
  } catch (err: any) {
    console.error("Failed to fetch cloud sync progress:", err);
  }
}

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
    comfyuiUrl.value = config.comfyui_url || "http://127.0.0.1:8188";
    webuiUrl.value = config.webui_url || "http://127.0.0.1:7860";
    void checkComfyConnection();
    void checkWebuiConnection();

    storageBackend.value = "sqlite";
    remoteConnectionUrl.value = config.remote_connection_url || "";
    clientIdentifier.value = config.client_identifier || "local_client";
    rootMappings.value = config.root_mappings ? { ...config.root_mappings } : {};
    const lastSync = collaborationSync.getLastSyncTime();
    lastSyncDisplay.value = lastSync > 0 ? new Date(lastSync).toLocaleTimeString() : "ready";

    const cb = config.cloud_backup;
    if (cb) {
      cloudProvider.value = cb.provider || "local_path";
      cloudLocalPath.value = cb.local_path || "";
      cloudWebdavEndpoint.value = cb.webdav_endpoint || "";
      cloudWebdavUsername.value = cb.webdav_username || "";
      cloudWebdavPassword.value = cb.webdav_password || "";
      cloudS3Endpoint.value = cb.s3_endpoint || "";
      cloudS3Bucket.value = cb.s3_bucket || "";
      cloudS3Region.value = cb.s3_region || "auto";
      cloudS3AccessKey.value = cb.s3_access_key || "";
      cloudS3SecretKey.value = cb.s3_secret_key || "";
      cloudS3Prefix.value = cb.s3_prefix || "backups/";
      cloudAutoBackup.value = cb.auto_backup_enabled ?? false;
      cloudAutoIntervalDays.value = cb.auto_backup_interval_days ?? 7;
    }

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

onMounted(async () => {
  if (props.show) {
    void loadSettingsAndPaths();
    void loadCacheStats();
    void fetchSyncProgress();
  }
  try {
    unlistenSyncProgress = await listen<CloudSyncProgress>("cloud-sync://progress", (event) => {
      syncProgress.value = event.payload;
      if (
        event.payload.phase === "completed" ||
        event.payload.phase === "cancelled" ||
        event.payload.phase === "failed"
      ) {
        void fetchSyncProgress();
      }
    });
  } catch (e) {
    console.error("Failed to register cloud-sync event listener:", e);
  }
});

onUnmounted(() => {
  if (unlistenSyncProgress) {
    unlistenSyncProgress();
    unlistenSyncProgress = null;
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
      comfyui_url: comfyuiUrl.value,
      webui_url: webuiUrl.value,
      storage_backend: "sqlite",
      remote_connection_url: remoteConnectionUrl.value,
      client_identifier: clientIdentifier.value,
      root_mappings: rootMappings.value,
      cloud_backup: getCurrentCloudConfig(),
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
            :class="{ active: activeTab === 'interop' }"
            role="tab"
            :aria-selected="activeTab === 'interop'"
            @click="activeTab = 'interop'"
          >
            <span aria-hidden="true">🔌</span><span>{{ t.interop.title }}</span>
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'collaboration' }"
            role="tab"
            :aria-selected="activeTab === 'collaboration'"
            @click="activeTab = 'collaboration'"
          >
            <span aria-hidden="true">👥</span><span>{{ t.settings.tabs.collaboration || 'Team & Database' }}</span>
          </button>
          <button
            type="button"
            class="tab-btn"
            :class="{ active: activeTab === 'cloudBackup' }"
            role="tab"
            :aria-selected="activeTab === 'cloudBackup'"
            @click="activeTab = 'cloudBackup'; handleListSnapshots();"
          >
            <span aria-hidden="true">☁️</span><span>{{ t.settings.tabs.cloudBackup }}</span>
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
              <div class="cache-actions" style="display: flex; gap: 8px;">
                <button
                  type="button"
                  class="btn secondary"
                  :disabled="clearingCache"
                  @click="handleClearCache"
                >
                  {{ clearingCache ? t.settings.clearing : t.settings.clearCache }}
                </button>
                <button
                  type="button"
                  class="btn secondary"
                  @click="showDiagnosticsModal = true"
                >
                  ⚡ {{ t.settings.diagnostics || 'Diagnostics' }}
                </button>
              </div>

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

          <!-- Tab: Generation Interop -->
          <div v-if="activeTab === 'interop'" class="settings-panel">
            <div class="panel-heading">
              <h4 class="panel-title">{{ t.interop.title }}</h4>
              <p class="panel-subtitle">Configure local WebUI and ComfyUI endpoints for generation interop and workflow execution.</p>
            </div>

            <!-- ComfyUI Base URL -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.interop.comfyUrl }}</span>
                <span class="row-desc">Default: http://127.0.0.1:8188</span>
              </div>
              <div class="interop-row-control">
                <input
                  v-model="comfyuiUrl"
                  type="text"
                  class="url-input"
                  placeholder="http://127.0.0.1:8188"
                />
                <button
                  type="button"
                  class="btn-test-conn"
                  :disabled="comfyStatus === 'checking'"
                  @click="checkComfyConnection"
                >
                  {{ comfyStatus === 'checking' ? t.interop.checking : t.interop.testConnection }}
                </button>
                <span
                  v-if="comfyStatus !== 'unknown'"
                  class="status-pill"
                  :class="comfyStatus"
                >
                  {{ comfyStatus === 'online' ? '🟢 ' + t.interop.online : '🔴 ' + t.interop.offline }}
                </span>
              </div>
            </div>

            <!-- SD WebUI Base URL -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.interop.webuiUrl }}</span>
                <span class="row-desc">Default: http://127.0.0.1:7860</span>
              </div>
              <div class="interop-row-control">
                <input
                  v-model="webuiUrl"
                  type="text"
                  class="url-input"
                  placeholder="http://127.0.0.1:7860"
                />
                <button
                  type="button"
                  class="btn-test-conn"
                  :disabled="webuiStatus === 'checking'"
                  @click="checkWebuiConnection"
                >
                  {{ webuiStatus === 'checking' ? t.interop.checking : t.interop.testConnection }}
                </button>
                <span
                  v-if="webuiStatus !== 'unknown'"
                  class="status-pill"
                  :class="webuiStatus"
                >
                  {{ webuiStatus === 'online' ? '🟢 ' + t.interop.online : '🔴 ' + t.interop.offline }}
                </span>
              </div>
            </div>
          </div>

          <!-- Tab: Team Collaboration & Database -->
          <div v-if="activeTab === 'collaboration'" class="settings-panel">
            <div class="panel-heading">
              <h4 class="panel-title">{{ t.settings.collaborationTitle }}</h4>
              <p class="panel-subtitle">{{ t.settings.collaborationSubtitle }}</p>
            </div>

            <!-- Database Engine Backend -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.storageBackend }}</span>
                <span class="row-desc">{{ t.review.remoteUnavailable }}</span>
              </div>
              <select v-model="storageBackend" class="select-input">
                <option value="sqlite">{{ t.settings.backendSqlite }}</option>
                <option value="mysql" disabled>{{ t.settings.backendMysql }}</option>
                <option value="postgres" disabled>{{ t.settings.backendPostgres }}</option>
              </select>
            </div>

            <!-- Client Identifier -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.clientId }}</span>
                <span class="row-desc">{{ t.settings.clientIdDesc }}</span>
              </div>
              <input
                v-model="clientIdentifier"
                type="text"
                class="url-input"
                :placeholder="t.settings.clientIdPlaceholder"
              />
            </div>

            <!-- Remote Connection URL (when MySQL or Postgres selected) -->
            <div v-if="storageBackend !== 'sqlite'" class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.remoteUrl }}</span>
                <span class="row-desc">{{ t.settings.remoteUrlDesc }}</span>
              </div>
              <div class="interop-row-control">
                <input
                  v-model="remoteConnectionUrl"
                  type="text"
                  class="url-input"
                  :placeholder="storageBackend === 'mysql' ? 'mysql://user:pass@192.168.1.100:3306/berry' : 'postgres://user:pass@192.168.1.100:5432/berry'"
                />
                <button
                  type="button"
                  class="btn-test-conn"
                  :disabled="pingStatus === 'testing'"
                  @click="handlePingDatabase"
                >
                  {{ pingStatus === 'testing' ? t.settings.testingConnection : t.settings.testConnection }}
                </button>
              </div>
            </div>

            <!-- Connection Ping Result Banner -->
            <div v-if="storageBackend !== 'sqlite' && pingStatus !== 'unknown'" class="setting-row ping-result-row">
              <span class="status-pill" :class="pingStatus === 'success' ? 'online' : 'offline'">
                {{ pingStatus === 'success' ? '🟢 ' + t.settings.connectionSuccess + (pingLatency !== null ? ' (' + pingLatency + ' ms)' : '') : '🔴 ' + t.settings.connectionFailed }}
              </span>
              <span class="ping-message">{{ pingMessage }}</span>
            </div>

            <!-- Real-time Sync Status -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.syncStatus }}</span>
                <span class="row-desc">{{ t.settings.lastSynced }}: {{ lastSyncDisplay }}</span>
              </div>
              <span class="status-pill" :class="storageBackend === 'sqlite' ? 'offline' : 'online'">
                {{ storageBackend === 'sqlite' ? t.settings.syncIdle : t.settings.syncActive }}
              </span>
            </div>

            <!-- Storage Roots & Local Mount Mappings -->
            <div class="settings-subsection">
              <h5 class="subsection-title">{{ t.settings.rootMappingsTitle }}</h5>
              <p class="panel-subtitle">{{ t.settings.rootMappingsDesc }}</p>

              <div v-if="Object.keys(rootMappings).length > 0" class="mappings-list">
                <div v-for="(path, uuid) in rootMappings" :key="uuid" class="mapping-item">
                  <span class="mapping-uuid">{{ uuid }}</span>
                  <span class="mapping-arrow">➔</span>
                  <span class="mapping-path">{{ path }}</span>
                  <button type="button" class="btn-remove-mapping" @click="removeRootMappingEntry(String(uuid))">
                    {{ t.settings.removeRootMapping }}
                  </button>
                </div>
              </div>

              <!-- Add Root Mapping Input Row -->
              <div class="add-mapping-row">
                <input
                  v-model="newMappingUuid"
                  type="text"
                  class="mapping-input-uuid"
                  :placeholder="t.settings.rootUuid"
                />
                <input
                  v-model="newMappingPath"
                  type="text"
                  class="mapping-input-path"
                  :placeholder="t.settings.localMountPath"
                />
                <button type="button" class="btn-browse-mapping" @click="handleBrowseMappingPath">
                  {{ t.settings.browseMount }}
                </button>
                <button
                  type="button"
                  class="btn-add-mapping"
                  :disabled="!newMappingUuid.trim() || !newMappingPath.trim()"
                  @click="addRootMappingEntry"
                >
                  {{ t.settings.addRootMapping }}
                </button>
              </div>
            </div>

            <!-- Migration Wizard Card -->
            <div class="settings-subsection migration-card-section">
              <div class="migration-card-header">
                <div>
                  <h5 class="subsection-title">{{ t.settings.migrationCardTitle }}</h5>
                  <p class="panel-subtitle">{{ t.settings.migrationCardDesc }}</p>
                </div>
                <button
                  type="button"
                  class="btn-open-wizard"
                  @click="showMigrationWizardModal = true"
                >
                  🚀 {{ t.settings.openMigrationWizard }}
                </button>
              </div>
            </div>
          </div>

          <!-- Tab: Cloud Snapshot Backup & Restore -->
          <div v-if="activeTab === 'cloudBackup'" class="settings-panel">
            <div class="panel-heading">
              <h4 class="panel-title">{{ t.settings.cloudBackup.title }}</h4>
              <p class="panel-subtitle">{{ t.settings.cloudBackup.subtitle }}</p>
            </div>

            <!-- Provider Selection -->
            <div class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.cloudBackup.provider }}</span>
              </div>
              <select v-model="cloudProvider" class="select-input">
                <option value="local_path">{{ t.settings.cloudBackup.providerLocal }}</option>
                <option value="webdav">{{ t.settings.cloudBackup.providerWebdav }}</option>
                <option value="s3">{{ t.settings.cloudBackup.providerS3 }}</option>
              </select>
            </div>

            <!-- Provider Options: Local / SMB / NFS -->
            <div v-if="cloudProvider === 'local_path'" class="setting-row">
              <div class="row-info">
                <span class="row-label">{{ t.settings.cloudBackup.localPath }}</span>
              </div>
              <div style="display: flex; gap: 8px; width: 60%;">
                <input
                  v-model="cloudLocalPath"
                  type="text"
                  class="url-input"
                  style="flex: 1;"
                  placeholder="D:\Backups or \\nas\berry_backups"
                />
                <button type="button" class="btn-browse-mapping" @click="handleBrowseLocalBackupPath">
                  {{ t.settings.cloudBackup.browse }}
                </button>
              </div>
            </div>

            <!-- Provider Options: WebDAV -->
            <template v-if="cloudProvider === 'webdav'">
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.webdavEndpoint }}</span>
                </div>
                <input
                  v-model="cloudWebdavEndpoint"
                  type="text"
                  class="url-input"
                  placeholder="https://nextcloud.example.com/remote.php/dav/files/user/backups/"
                />
              </div>
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.webdavUser }}</span>
                </div>
                <input
                  v-model="cloudWebdavUsername"
                  type="text"
                  class="url-input"
                  placeholder="admin"
                />
              </div>
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.webdavPassword }}</span>
                </div>
                <input
                  v-model="cloudWebdavPassword"
                  type="password"
                  class="url-input"
                  placeholder="••••••••"
                />
              </div>
            </template>

            <!-- Provider Options: S3 Compatible -->
            <template v-if="cloudProvider === 's3'">
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.s3Endpoint }}</span>
                </div>
                <input
                  v-model="cloudS3Endpoint"
                  type="text"
                  class="url-input"
                  placeholder="https://<account-id>.r2.cloudflarestorage.com or s3.amazonaws.com"
                />
              </div>
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.s3Bucket }}</span>
                </div>
                <input
                  v-model="cloudS3Bucket"
                  type="text"
                  class="url-input"
                  placeholder="my-berry-backups"
                />
              </div>
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.s3Region }}</span>
                </div>
                <input
                  v-model="cloudS3Region"
                  type="text"
                  class="url-input"
                  placeholder="auto or us-east-1"
                />
              </div>
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.s3AccessKey }}</span>
                </div>
                <input
                  v-model="cloudS3AccessKey"
                  type="text"
                  class="url-input"
                  placeholder="Access Key ID"
                />
              </div>
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.s3SecretKey }}</span>
                </div>
                <input
                  v-model="cloudS3SecretKey"
                  type="password"
                  class="url-input"
                  placeholder="Secret Access Key"
                />
              </div>
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.s3Prefix }}</span>
                </div>
                <input
                  v-model="cloudS3Prefix"
                  type="text"
                  class="url-input"
                  placeholder="backups/"
                />
              </div>
            </template>

            <!-- Test Connection & Feedback -->
            <div class="setting-row" style="align-items: center;">
              <div class="row-info">
                <button
                  type="button"
                  class="btn-browse-mapping"
                  :disabled="cloudPingStatus === 'testing'"
                  @click="handleTestCloudConnection"
                >
                  <span v-if="cloudPingStatus === 'testing'">⏳ Testing...</span>
                  <span v-else>📡 {{ t.settings.cloudBackup.testConnection }}</span>
                </button>
              </div>
              <div>
                <span
                  v-if="cloudPingStatus === 'success'"
                  style="color: #4ade80; font-size: 0.85rem; font-weight: 500;"
                >
                  ✓ {{ cloudPingMessage }} ({{ cloudPingLatency }} ms)
                </span>
                <span
                  v-else-if="cloudPingStatus === 'error'"
                  style="color: #f87171; font-size: 0.85rem;"
                >
                  ✕ {{ cloudPingMessage }}
                </span>
              </div>
            </div>

            <!-- Snapshot Creation Card -->
            <div class="settings-subsection" style="margin-top: 20px; padding: 16px; background: rgba(0,0,0,0.2); border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);">
              <h5 class="subsection-title" style="margin-bottom: 8px; font-size: 0.95rem; color: #f1f5f9;">
                📦 {{ t.settings.cloudBackup.createSnapshot }}
              </h5>
              <div style="display: flex; gap: 8px; margin-top: 8px;">
                <input
                  v-model="newSnapshotDescription"
                  type="text"
                  class="url-input"
                  style="flex: 1;"
                  :placeholder="t.settings.cloudBackup.snapshotDescription"
                />
                <button
                  type="button"
                  class="btn-add-mapping"
                  :disabled="cloudCreatingSnapshot"
                  @click="handleCreateSnapshot"
                >
                  <span v-if="cloudCreatingSnapshot">⏳ {{ t.settings.cloudBackup.creating }}</span>
                  <span v-else>🚀 {{ t.settings.cloudBackup.createSnapshot }}</span>
                </button>
              </div>
              <div v-if="cloudActionMessage" style="margin-top: 8px; color: #4ade80; font-size: 0.85rem;">
                {{ cloudActionMessage }}
              </div>
              <div v-if="cloudActionError" style="margin-top: 8px; color: #f87171; font-size: 0.85rem;">
                ✕ {{ cloudActionError }}
              </div>
            </div>

            <!-- Remote Snapshots List -->
            <div class="settings-subsection" style="margin-top: 20px;">
              <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                <h5 class="subsection-title" style="margin: 0; font-size: 0.95rem; color: #f1f5f9;">
                  ☁️ {{ t.settings.cloudBackup.snapshotsTitle }}
                </h5>
                <button
                  type="button"
                  class="btn-browse-mapping"
                  :disabled="cloudSnapshotsLoading"
                  @click="handleListSnapshots"
                >
                  {{ cloudSnapshotsLoading ? '...' : '↻ ' + t.settings.cloudBackup.refreshSnapshots }}
                </button>
              </div>

              <div v-if="cloudSnapshotsLoading" style="padding: 20px; text-align: center; color: #94a3b8; font-size: 0.85rem;">
                Loading snapshots...
              </div>
              <div v-else-if="cloudSnapshots.length === 0" style="padding: 20px; text-align: center; color: #94a3b8; font-size: 0.85rem;">
                {{ t.settings.cloudBackup.noSnapshots }}
              </div>
              <div v-else class="mapping-table-wrapper" style="max-height: 220px; overflow-y: auto;">
                <table class="root-mapping-table">
                  <thead>
                    <tr>
                      <th>{{ t.settings.cloudBackup.colName }}</th>
                      <th>{{ t.settings.cloudBackup.colDate }}</th>
                      <th>{{ t.settings.cloudBackup.colSize }}</th>
                      <th>{{ t.settings.cloudBackup.colFiles }}</th>
                      <th style="text-align: right;">{{ t.settings.cloudBackup.colActions }}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="item in cloudSnapshots" :key="item.filename">
                      <td style="font-family: monospace; font-size: 0.8rem;" :title="item.description || item.filename">
                        {{ item.filename }}
                        <span v-if="item.description" style="display: block; color: #94a3b8; font-size: 0.72rem;">{{ item.description }}</span>
                      </td>
                      <td style="font-size: 0.78rem; white-space: nowrap;">
                        {{ new Date(item.created_at * 1000).toLocaleString() }}
                      </td>
                      <td style="font-size: 0.78rem;">
                        {{ (item.size_bytes / (1024 * 1024)).toFixed(2) }} MB
                      </td>
                      <td style="font-size: 0.78rem;">
                        {{ item.file_count }}
                      </td>
                      <td style="text-align: right;">
                        <button
                          type="button"
                          class="btn-browse-mapping"
                          style="padding: 4px 10px; font-size: 0.75rem; border-color: rgba(239, 68, 68, 0.4); color: #fca5a5;"
                          :disabled="cloudRestoringSnapshot"
                          @click="handleRestoreSnapshot(item)"
                        >
                          {{ cloudRestoringSnapshot ? t.settings.cloudBackup.restoring : t.settings.cloudBackup.restore }}
                        </button>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>

            <!-- Subsection: Incremental Media Mirroring & Delta Sync -->
            <div class="settings-subsection" style="margin-top: 24px; padding-top: 20px; border-top: 1px solid rgba(255,255,255,0.08);">
              <div class="panel-heading" style="margin-bottom: 12px;">
                <h5 class="subsection-title" style="font-size: 1rem; color: #f1f5f9; margin-bottom: 4px;">
                  🔄 {{ t.settings.cloudBackup.mediaSyncTitle }}
                </h5>
                <p class="panel-subtitle" style="font-size: 0.8rem; color: #94a3b8; margin: 0;">
                  {{ t.settings.cloudBackup.mediaSyncSubtitle }}
                </p>
              </div>

              <!-- Strategy & Concurrency -->
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.syncStrategy }}</span>
                </div>
                <select v-model="syncStrategy" class="select-input">
                  <option value="fast_fingerprint">{{ t.settings.cloudBackup.strategyFast }}</option>
                  <option value="sha256_checksum">{{ t.settings.cloudBackup.strategySha256 }}</option>
                </select>
              </div>

              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.concurrency }}</span>
                </div>
                <div style="display: flex; align-items: center; gap: 8px;">
                  <input
                    v-model.number="syncConcurrency"
                    type="range"
                    min="1"
                    max="16"
                    step="1"
                    style="width: 140px;"
                  />
                  <span style="font-size: 0.85rem; color: #cbd5e1; min-width: 32px;">{{ syncConcurrency }}</span>
                </div>
              </div>

              <!-- Bandwidth limit & Remote Prefix -->
              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.bandwidthLimit }}</span>
                  <span class="row-desc">{{ t.settings.cloudBackup.bandwidthLimitDesc }}</span>
                </div>
                <input
                  v-model.number="syncBandwidthLimit"
                  type="number"
                  min="0"
                  step="128"
                  class="url-input"
                  style="width: 120px;"
                  placeholder="0"
                />
              </div>

              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.remoteMediaPrefix }}</span>
                </div>
                <input
                  v-model="syncRemotePrefix"
                  type="text"
                  class="url-input"
                  style="width: 200px;"
                  placeholder="media/"
                />
              </div>

              <div class="setting-row">
                <div class="row-info">
                  <span class="row-label">{{ t.settings.cloudBackup.dryRun }}</span>
                </div>
                <input
                  v-model="syncDryRun"
                  type="checkbox"
                  style="width: 18px; height: 18px; accent-color: var(--accent-color, #0284c7);"
                />
              </div>

              <!-- Sync Action & Live Progress -->
              <div style="margin-top: 14px; padding: 14px; background: rgba(0,0,0,0.25); border-radius: 8px; border: 1px solid rgba(255,255,255,0.08);">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <button
                    v-if="!syncProgress || syncProgress.phase === 'idle' || syncProgress.phase === 'completed' || syncProgress.phase === 'cancelled' || syncProgress.phase === 'failed'"
                    type="button"
                    class="btn-add-mapping"
                    :disabled="syncStarting"
                    @click="handleStartCloudSync"
                  >
                    🚀 {{ syncStarting ? t.settings.cloudBackup.syncing : t.settings.cloudBackup.startSync }}
                  </button>
                  <button
                    v-else
                    type="button"
                    class="btn-browse-mapping"
                    style="border-color: rgba(239, 68, 68, 0.4); color: #fca5a5;"
                    @click="handleCancelCloudSync"
                  >
                    ⏹ {{ t.settings.cloudBackup.cancelSync }}
                  </button>

                  <div v-if="syncProgress && syncProgress.phase !== 'idle'">
                    <span
                      :style="{
                        color:
                          syncProgress.phase === 'completed'
                            ? '#4ade80'
                            : syncProgress.phase === 'syncing' || syncProgress.phase === 'scanning'
                            ? '#38bdf8'
                            : syncProgress.phase === 'cancelled'
                            ? '#fbbf24'
                            : '#f87171',
                        fontSize: '0.85rem',
                        fontWeight: '500',
                      }"
                    >
                      ● {{ syncProgress.phase.toUpperCase() }}
                    </span>
                  </div>
                </div>

                <!-- Progress Bar & Details -->
                <div v-if="syncProgress && syncProgress.phase !== 'idle'" style="margin-top: 12px;">
                  <div style="width: 100%; height: 8px; background: rgba(255,255,255,0.1); border-radius: 4px; overflow: hidden;">
                    <div
                      :style="{
                        width: `${syncProgress.total_files > 0 ? Math.min(100, Math.round(((syncProgress.completed_files + syncProgress.skipped_files + syncProgress.failed_files) / syncProgress.total_files) * 100)) : 0}%`,
                        height: '100%',
                        background: 'var(--accent-color, #0284c7)',
                        transition: 'width 0.2s ease',
                      }"
                    ></div>
                  </div>

                  <div style="display: flex; justify-content: space-between; font-size: 0.78rem; color: #94a3b8; margin-top: 8px;">
                    <span>
                      {{
                        t.settings.cloudBackup.progressFiles
                          .replace('{synced}', String(syncProgress.completed_files))
                          .replace('{total}', String(syncProgress.total_files))
                          .replace('{skipped}', String(syncProgress.skipped_files))
                          .replace('{failed}', String(syncProgress.failed_files))
                      }}
                    </span>
                    <span v-if="syncProgress.phase === 'syncing'">
                      {{
                        t.settings.cloudBackup.progressSpeed
                          .replace('{speed}', `${(syncProgress.speed_bytes_per_sec / (1024 * 1024)).toFixed(2)} MB/s`)
                          .replace('{eta}', syncProgress.eta_seconds != null ? `${syncProgress.eta_seconds}s` : '--')
                      }}
                    </span>
                  </div>

                  <div v-if="syncProgress.current_file" style="font-size: 0.75rem; color: #64748b; margin-top: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                    {{ t.settings.cloudBackup.progressCurrent.replace('{file}', syncProgress.current_file) }}
                  </div>

                  <!-- Summary Box -->
                  <div v-if="syncSummary" style="margin-top: 10px; padding: 8px 12px; background: rgba(255,255,255,0.03); border-radius: 6px; font-size: 0.8rem; color: #e2e8f0;">
                    ✓ {{ syncSummary.dry_run ? '[Dry Run] ' : '' }}{{ t.settings.cloudBackup.syncCompleted }}:
                    {{ syncSummary.synced_files }} synced, {{ syncSummary.skipped_files }} skipped, {{ syncSummary.failed_files }} failed ({{ (syncSummary.transferred_bytes / (1024 * 1024)).toFixed(2) }} MB in {{ (syncSummary.duration_ms / 1000).toFixed(1) }}s).
                  </div>
                </div>
              </div>
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
    <ThumbnailDiagnosticsModal :show="showDiagnosticsModal" @close="showDiagnosticsModal = false" />
    <MigrationWizardModal :show="showMigrationWizardModal" @close="showMigrationWizardModal = false" />
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
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  border-radius: 5px;
  min-width: 178px;
  padding: 7px 30px 7px 10px;
  font-size: 0.75rem;
  outline: none;
}

.select-input option {
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
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

.interop-row-control {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.url-input {
  background: var(--color-bg-primary);
  border: 1px solid var(--border-color);
  color: var(--color-text-primary);
  border-radius: 5px;
  padding: 6px 10px;
  font-size: 0.78rem;
  width: 220px;
  outline: none;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

.url-input:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 1px;
}

.btn-test-conn {
  padding: 6px 12px;
  font-size: 0.75rem;
  border-radius: 5px;
  border: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.btn-test-conn:hover:not(:disabled) {
  background: var(--color-bg-tertiary);
  border-color: var(--border-color-strong);
}

.btn-test-conn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.status-pill {
  font-size: 0.72rem;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.status-pill.online {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.status-pill.offline {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.status-pill.checking {
  background: rgba(245, 158, 11, 0.15);
  color: #fbbf24;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.ping-result-row {
  display: flex;
  align-items: center;
  gap: 12px;
  background: rgba(255, 255, 255, 0.02);
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.ping-message {
  font-size: 0.8rem;
  color: var(--text-secondary, #94a3b8);
}

.settings-subsection {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.subsection-title {
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--text-primary, #f1f5f9);
  margin: 0 0 4px 0;
}

.mappings-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 12px 0;
}

.mapping-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 6px;
  font-size: 0.82rem;
}

.mapping-uuid {
  font-family: monospace;
  font-weight: 600;
  color: var(--accent-color, #38bdf8);
}

.mapping-arrow {
  color: rgba(255, 255, 255, 0.3);
}

.mapping-path {
  flex: 1;
  font-family: monospace;
  color: var(--text-secondary, #cbd5e1);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-remove-mapping {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75rem;
}

.btn-remove-mapping:hover {
  background: rgba(239, 68, 68, 0.25);
}

.add-mapping-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 10px;
}

.mapping-input-uuid {
  width: 140px;
  padding: 7px 10px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  color: var(--text-primary, #f1f5f9);
  font-size: 0.82rem;
}

.mapping-input-path {
  flex: 1;
  padding: 7px 10px;
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  color: var(--text-primary, #f1f5f9);
  font-size: 0.82rem;
}

.btn-browse-mapping {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  color: var(--text-primary, #f1f5f9);
  padding: 7px 12px;
  font-size: 0.8rem;
  cursor: pointer;
  white-space: nowrap;
}

.btn-browse-mapping:hover {
  background: rgba(255, 255, 255, 0.12);
}

.btn-add-mapping {
  background: var(--accent-color, #0284c7);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  color: #ffffff;
  padding: 7px 14px;
  font-size: 0.8rem;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
}

.btn-add-mapping:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.migration-card-section {
  background: rgba(99, 102, 241, 0.05);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 8px;
  padding: 1.1rem;
}

.migration-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.btn-open-wizard {
  background: #4f46e5;
  border: 1px solid #6366f1;
  border-radius: 6px;
  color: #ffffff;
  padding: 8px 16px;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.15s ease, transform 0.15s ease;
}

.btn-open-wizard:hover {
  background: #4338ca;
  transform: translateY(-1px);
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
