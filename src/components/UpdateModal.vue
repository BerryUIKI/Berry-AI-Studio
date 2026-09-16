<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { t } from "../i18n";
import {
  checkForUpdates,
  openUrl,
  downloadUpdateAsset,
  installUpdate,
  type UpdateCheckResult,
  type UpdateDownloadProgress,
} from "../utils/updater";
import { openStorageDir } from "../utils/config";
import { formatBytes } from "../utils/image";

const props = defineProps<{
  show: boolean;
  currentVersion: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const checking = ref(false);
const result = ref<UpdateCheckResult | null>(null);

// In-app auto update download & install state
const downloading = ref(false);
const downloadProgress = ref<UpdateDownloadProgress | null>(null);
const downloadError = ref<string | null>(null);
const downloadedFilePath = ref<string | null>(null);
const silentInstall = ref(false);
const installing = ref(false);

async function runCheck() {
  checking.value = true;
  result.value = null;
  downloading.value = false;
  downloadProgress.value = null;
  downloadError.value = null;
  downloadedFilePath.value = null;
  installing.value = false;
  try {
    const res = await checkForUpdates(props.currentVersion);
    result.value = res;
  } finally {
    checking.value = false;
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      void runCheck();
    }
  },
);

onMounted(() => {
  if (props.show) {
    void runCheck();
  }
});

function handleOpenRelease() {
  if (result.value?.release?.html_url) {
    void openUrl(result.value.release.html_url);
  }
}

function handleDownloadAsset() {
  if (result.value?.matchedAsset?.browser_download_url) {
    void openUrl(result.value.matchedAsset.browser_download_url);
  } else if (result.value?.release?.html_url) {
    void openUrl(result.value.release.html_url);
  }
}

async function handleAutoDownload() {
  if (!result.value?.matchedAsset?.browser_download_url) {
    handleDownloadAsset();
    return;
  }
  downloading.value = true;
  downloadError.value = null;
  downloadedFilePath.value = null;
  downloadProgress.value = {
    downloaded_bytes: 0,
    total_bytes: result.value.matchedAsset.size || 0,
    percent: 0,
    speed_bytes_per_sec: 0,
    done: false,
    target_file: null,
  };

  try {
    const filePath = await downloadUpdateAsset(
      result.value.matchedAsset.browser_download_url,
      result.value.matchedAsset.name,
      (progress) => {
        downloadProgress.value = progress;
      }
    );
    downloadedFilePath.value = filePath;
  } catch (err: any) {
    downloadError.value = err?.message || String(err);
  } finally {
    downloading.value = false;
  }
}

async function handleInstallAndRestart() {
  if (!downloadedFilePath.value) return;
  installing.value = true;
  try {
    await installUpdate(downloadedFilePath.value, silentInstall.value);
  } catch (err: any) {
    downloadError.value = err?.message || String(err);
    installing.value = false;
  }
}

function handleOpenUpdatesFolder() {
  void openStorageDir("updates");
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="update-dialog">
      <!-- Header -->
      <div class="dialog-header">
        <div class="header-left">
          <img src="../assets/logo.png" alt="Berry" width="20" height="20" class="dialog-logo" />
          <h3 class="dialog-title">{{ t.updater.title }}</h3>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Body -->
      <div class="dialog-body">
        <!-- 1. Checking State -->
        <div v-if="checking" class="state-container">
          <div class="spinner-pulse">
            <img src="../assets/logo.png" alt="Berry" width="40" height="40" class="pulsing-logo" />
          </div>
          <h4 class="state-title">{{ t.updater.checkingTitle }}</h4>
          <p class="state-desc">{{ t.updater.checkingDesc }}</p>
        </div>

        <!-- 2. Up to Date -->
        <div v-else-if="result?.status === 'up_to_date'" class="state-container">
          <div class="status-icon success">✓</div>
          <h4 class="state-title">{{ t.updater.upToDateTitle }}</h4>
          <p class="state-desc">
            {{ t.updater.upToDateDesc }} (v{{ props.currentVersion }})
          </p>
          <div v-if="result.release" class="release-pill">
            <span>{{ t.updater.latestRelease }}: <strong>v{{ result.latestVersion }}</strong></span>
            <span v-if="result.release.published_at" class="release-date">
              {{ new Date(result.release.published_at).toLocaleDateString() }}
            </span>
          </div>
        </div>

        <!-- 3. Update Available -->
        <div v-else-if="result?.status === 'update_available'" class="update-available-layout">
          <div class="update-banner">
            <div class="banner-left">
              <span class="update-pill">NEW</span>
              <div class="banner-titles">
                <h4 class="update-ver-title">
                  {{ t.updater.newVersionFound }}: v{{ result.latestVersion }}
                </h4>
                <span class="current-ver-sub">
                  {{ t.updater.currentInstalled }}: v{{ props.currentVersion }}
                </span>
              </div>
            </div>
            <span v-if="result.release?.published_at" class="publish-time">
              {{ new Date(result.release.published_at).toLocaleDateString() }}
            </span>
          </div>

          <!-- Safe Storage & No-Uninstall Guarantee Notice -->
          <div class="preserve-notice-box">
            <span class="notice-icon">🛡️</span>
            <span class="notice-text">{{ t.updater.preserveDataNotice }}</span>
          </div>

          <!-- In-App Download Progress Card -->
          <div v-if="downloading || downloadedFilePath || downloadError" class="download-progress-card">
            <!-- If download error -->
            <div v-if="downloadError" class="download-error-line">
              <span>⚠️ {{ downloadError }}</span>
            </div>

            <!-- If currently downloading -->
            <div v-if="downloading" class="progress-details">
              <div class="progress-header">
                <span class="progress-title">⚡ {{ t.updater.downloading }}</span>
                <span class="progress-speed">
                  {{ downloadProgress?.speed_bytes_per_sec ? `${formatBytes(downloadProgress.speed_bytes_per_sec)}/s` : '' }}
                </span>
              </div>
              <div class="progress-bar-track">
                <div class="progress-bar-fill" :style="{ width: `${downloadProgress?.percent || 0}%` }"></div>
              </div>
              <div class="progress-sub">
                <span>{{ formatBytes(downloadProgress?.downloaded_bytes || 0) }} / {{ formatBytes(downloadProgress?.total_bytes || 0) }}</span>
                <span class="progress-pct">{{ (downloadProgress?.percent || 0).toFixed(1) }}%</span>
              </div>
            </div>

            <!-- If downloaded and ready to install -->
            <div v-else-if="downloadedFilePath" class="ready-details">
              <div class="ready-title-line">
                <span class="ready-badge">✓</span>
                <span class="ready-text">{{ t.updater.downloadComplete }}</span>
              </div>
              <div class="silent-option">
                <label class="checkbox-label">
                  <input type="checkbox" v-model="silentInstall" />
                  <span>{{ t.updater.silentInstall }}</span>
                </label>
              </div>
            </div>
          </div>

          <!-- Release Notes -->
          <div class="changelog-container">
            <div class="changelog-header">
              <span>{{ t.updater.releaseNotes }}</span>
              <span v-if="result.matchedAsset" class="asset-name">
                📦 {{ result.matchedAsset.name }} ({{ formatBytes(result.matchedAsset.size) }})
              </span>
            </div>
            <div class="changelog-body">
              <pre class="changelog-text">{{ result.release?.body || t.updater.noChangelog }}</pre>
            </div>
          </div>
        </div>

        <!-- 4. Ahead of Release / Dev Build -->
        <div v-else-if="result?.status === 'ahead_of_release'" class="state-container">
          <div class="status-icon dev">⚡</div>
          <h4 class="state-title">{{ t.updater.aheadTitle }}</h4>
          <p class="state-desc">
            {{ t.updater.aheadDesc }}
          </p>
          <div class="version-compare-box">
            <div class="ver-col">
              <span class="ver-label">{{ t.updater.currentBuild }}</span>
              <span class="ver-value current">v{{ props.currentVersion }}</span>
            </div>
            <div class="ver-arrow">→</div>
            <div class="ver-col">
              <span class="ver-label">{{ t.updater.publicRelease }}</span>
              <span class="ver-value public">
                {{ result.latestVersion ? `v${result.latestVersion}` : 'None' }}
              </span>
            </div>
          </div>
        </div>

        <!-- 5. Error State -->
        <div v-else-if="result?.status === 'error'" class="state-container">
          <div class="status-icon error">⚠️</div>
          <h4 class="state-title">{{ t.updater.errorTitle }}</h4>
          <p class="state-desc error-text">
            {{ result.errorMessage || t.updater.errorDesc }}
          </p>
        </div>
      </div>

      <!-- Footer -->
      <div class="dialog-footer">
        <button
          v-if="!checking && (result?.status === 'error' || result?.status === 'up_to_date' || result?.status === 'ahead_of_release')"
          type="button"
          class="btn secondary"
          @click="runCheck"
        >
          🔄 {{ t.updater.checkAgain }}
        </button>

        <button
          v-if="result?.release?.html_url"
          type="button"
          class="btn secondary"
          @click="handleOpenRelease"
        >
          🌐 GitHub Releases
        </button>

        <!-- Browser fallback download button -->
        <button
          v-if="result?.status === 'update_available' && !downloadedFilePath && !downloading"
          type="button"
          class="btn secondary"
          @click="handleDownloadAsset"
        >
          🌐 {{ t.updater.downloadInBrowser }}
        </button>

        <!-- Auto Download Action Button -->
        <button
          v-if="result?.status === 'update_available' && !downloadedFilePath && !downloading"
          type="button"
          class="btn primary"
          @click="handleAutoDownload"
        >
          {{ t.updater.autoDownloadAndInstall }}
        </button>

        <!-- Downloading State Indicator -->
        <button
          v-if="downloading"
          type="button"
          class="btn primary"
          disabled
        >
          ⏳ {{ t.updater.downloading }}
        </button>

        <!-- Install & Restart Action -->
        <button
          v-if="downloadedFilePath"
          type="button"
          class="btn secondary"
          @click="handleOpenUpdatesFolder"
        >
          📁 {{ t.settings.openFolder }}
        </button>

        <button
          v-if="downloadedFilePath"
          type="button"
          class="btn primary install-btn"
          :disabled="installing"
          @click="handleInstallAndRestart"
        >
          {{ installing ? '⏳ ...' : t.updater.installAndRestart }}
        </button>

        <button
          type="button"
          class="btn"
          :class="{ 'primary': result?.status !== 'update_available' && !downloadedFilePath }"
          @click="emit('close')"
        >
          {{ t.updater.close }}
        </button>
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
  z-index: 2100;
  user-select: none;
}

.update-dialog {
  width: 560px;
  max-width: 90vw;
  background: #18181c;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
  overflow: hidden;
}

.dialog-header {
  height: 46px;
  padding: 0 16px;
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

.dialog-logo {
  display: block;
  object-fit: contain;
}

.dialog-title {
  margin: 0;
  font-size: 0.9rem;
  font-weight: 600;
  color: #f1f5f9;
}

.close-btn {
  background: transparent;
  border: none;
  color: #71717a;
  cursor: pointer;
  padding: 4px;
  font-size: 0.85rem;
  transition: all 0.12s;
}

.close-btn:hover {
  color: #ffffff;
}

.dialog-body {
  padding: 20px;
  min-height: 220px;
  max-height: 460px;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.state-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 20px 10px;
  gap: 10px;
  flex: 1;
}

.pulsing-logo {
  animation: pulseLogo 1.5s infinite ease-in-out;
}

@keyframes pulseLogo {
  0% {
    transform: scale(0.92);
    opacity: 0.7;
  }
  50% {
    transform: scale(1.08);
    opacity: 1;
  }
  100% {
    transform: scale(0.92);
    opacity: 0.7;
  }
}

.status-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.4rem;
  font-weight: 700;
}

.status-icon.success {
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.status-icon.dev {
  background: rgba(18, 181, 203, 0.15);
  color: #12b5cb;
  border: 1px solid rgba(18, 181, 203, 0.3);
}

.status-icon.error {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.state-title {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: #f8fafc;
}

.state-desc {
  margin: 0;
  font-size: 0.78rem;
  color: #94a3b8;
  max-width: 400px;
  line-height: 1.4;
}

.error-text {
  color: #fca5a5;
  font-family: monospace;
  font-size: 0.72rem;
  background: rgba(239, 68, 68, 0.08);
  padding: 6px 12px;
  border-radius: 6px;
}

.release-pill {
  display: flex;
  align-items: center;
  gap: 10px;
  background: #202024;
  padding: 6px 14px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 0.74rem;
  color: #cbd5e1;
}

.release-date {
  color: #64748b;
}

.version-compare-box {
  display: flex;
  align-items: center;
  gap: 16px;
  background: #202024;
  padding: 10px 20px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  margin-top: 4px;
}

.ver-col {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.ver-label {
  font-size: 0.68rem;
  color: #64748b;
}

.ver-value {
  font-size: 0.84rem;
  font-weight: 600;
  font-family: monospace;
}

.ver-value.current {
  color: #12b5cb;
}

.ver-value.public {
  color: #a1a1aa;
}

.ver-arrow {
  color: #64748b;
  font-weight: 600;
}

/* Update Available Layout */
.update-available-layout {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.update-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: linear-gradient(135deg, rgba(18, 181, 203, 0.15), rgba(250, 184, 43, 0.1));
  border: 1px solid rgba(18, 181, 203, 0.3);
  border-radius: 8px;
  padding: 12px 16px;
}

.banner-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.update-pill {
  font-size: 0.65rem;
  font-weight: 700;
  background: #12b5cb;
  color: #ffffff;
  padding: 2px 6px;
  border-radius: 4px;
}

.banner-titles {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.update-ver-title {
  margin: 0;
  font-size: 0.92rem;
  font-weight: 700;
  color: #f8fafc;
}

.current-ver-sub {
  font-size: 0.7rem;
  color: #94a3b8;
}

.publish-time {
  font-size: 0.72rem;
  color: #cbd5e1;
}

.changelog-container {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: #141417;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  overflow: hidden;
}

.changelog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #202024;
  font-size: 0.74rem;
  font-weight: 600;
  color: #e2e8f0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.asset-name {
  font-size: 0.7rem;
  color: #12b5cb;
  font-weight: 500;
}

.changelog-body {
  padding: 10px 12px;
  max-height: 180px;
  overflow-y: auto;
}

.changelog-text {
  margin: 0;
  font-family: inherit;
  font-size: 0.74rem;
  color: #cbd5e1;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.dialog-footer {
  height: 48px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: #141417;
}

.btn {
  padding: 5px 12px;
  border-radius: 6px;
  font-size: 0.76rem;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.12s ease;
  border: none;
  background: rgba(255, 255, 255, 0.05);
  color: #cbd5e1;
}

.btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
}

.btn.primary {
  background: #12b5cb;
  color: #ffffff;
  font-weight: 500;
}

.btn.primary:hover {
  background: #0e9aa7;
}

.btn.install-btn {
  background: linear-gradient(135deg, #10b981, #059669);
  font-weight: 600;
}

.btn.install-btn:hover {
  background: linear-gradient(135deg, #059669, #047857);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Safe Storage Notice Box */
.preserve-notice-box {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  background: rgba(16, 185, 129, 0.08);
  border: 1px solid rgba(16, 185, 129, 0.25);
  border-radius: 8px;
  padding: 10px 14px;
}

.notice-icon {
  font-size: 1.1rem;
  line-height: 1.2;
}

.notice-text {
  font-size: 0.76rem;
  line-height: 1.45;
  color: #a7f3d0;
}

/* Download Progress Card */
.download-progress-card {
  background: #1e1e24;
  border: 1px solid rgba(18, 181, 203, 0.3);
  border-radius: 8px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.download-error-line {
  color: #f87171;
  font-size: 0.76rem;
  background: rgba(239, 68, 68, 0.1);
  padding: 6px 10px;
  border-radius: 6px;
}

.progress-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.progress-title {
  font-size: 0.8rem;
  font-weight: 600;
  color: #f1f5f9;
}

.progress-speed {
  font-size: 0.74rem;
  font-family: monospace;
  color: #12b5cb;
}

.progress-bar-track {
  width: 100%;
  height: 8px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 999px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #12b5cb, #38bdf8);
  border-radius: 999px;
  transition: width 0.2s ease-out;
}

.progress-sub {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.72rem;
  color: #94a3b8;
  font-family: monospace;
}

.progress-pct {
  color: #f8fafc;
  font-weight: 600;
}

.ready-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.ready-title-line {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ready-badge {
  background: #10b981;
  color: #ffffff;
  font-weight: 700;
  font-size: 0.75rem;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ready-text {
  font-size: 0.82rem;
  font-weight: 600;
  color: #34d399;
}

.silent-option {
  margin-top: 4px;
}

.checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 0.75rem;
  color: #cbd5e1;
  cursor: pointer;
}

</style>
