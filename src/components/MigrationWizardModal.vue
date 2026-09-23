<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type { DatabaseStats, MigrationOptions, MigrationSummary } from "../types";
import { formatBytes } from "../utils/image";

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const currentStep = ref<1 | 2 | 3 | 4>(1);
const stats = ref<DatabaseStats | null>(null);
const loadingStats = ref(false);

const dialect = ref<"mysql" | "postgres">("mysql");
const targetRootUuid = ref(crypto.randomUUID());
const destPath = ref("");

const exporting = ref(false);
const exportError = ref<string | null>(null);
const exportSummary = ref<MigrationSummary | null>(null);
const copied = ref(false);

async function loadStats() {
  loadingStats.value = true;
  try {
    stats.value = await invoke<DatabaseStats>("get_database_stats");
  } catch (err) {
    console.error("Failed to load database stats for migration:", err);
  } finally {
    loadingStats.value = false;
  }
}

watch(
  () => props.show,
  (newVal) => {
    if (newVal) {
      currentStep.value = 1;
      exportError.value = null;
      exportSummary.value = null;
      copied.value = false;
      loadStats();
    }
  },
  { immediate: true }
);

function generateNewUuid() {
  targetRootUuid.value = crypto.randomUUID();
}

async function handleBrowseDestination() {
  try {
    const defaultName = `berry_central_migration_${dialect.value}_${new Date().toISOString().slice(0, 10)}.sql`;
    const selected = await save({
      defaultPath: defaultName,
      filters: [{ name: "SQL Script", extensions: ["sql"] }],
      title: t.value.migrationWizard.destFile,
    });
    if (selected) {
      destPath.value = selected;
    }
  } catch (err) {
    console.error("Failed to choose migration file destination:", err);
  }
}

async function handleStartExport() {
  if (!destPath.value.trim()) return;

  currentStep.value = 3;
  exporting.value = true;
  exportError.value = null;

  try {
    const options: MigrationOptions = {
      target_dialect: dialect.value,
      target_root_uuid: targetRootUuid.value.trim(),
      destination: destPath.value.trim(),
    };

    const summary = await invoke<MigrationSummary>("export_sqlite_to_central_migration", {
      options,
    });

    exportSummary.value = summary;
    currentStep.value = 4;
  } catch (err: any) {
    exportError.value = typeof err === "string" ? err : err?.message || String(err);
    currentStep.value = 2;
  } finally {
    exporting.value = false;
  }
}

const cliCommand = computed(() => {
  if (!exportSummary.value) return "";
  const path = exportSummary.value.output_path;
  if (dialect.value === "mysql") {
    return `mysql -u berry_user -p -h db.internal -P 3306 berry_team < "${path}"`;
  } else {
    return `psql -U berry_user -h db.internal -p 5432 -d berry_team -f "${path}"`;
  }
});

async function copyCliCommand() {
  if (!cliCommand.value) return;
  try {
    await navigator.clipboard.writeText(cliCommand.value);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2500);
  } catch (err) {
    console.error("Failed to copy CLI command:", err);
  }
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="modal-dialog">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-titles">
          <h3 class="modal-title">📦 {{ t.migrationWizard.title }}</h3>
          <p class="modal-subtitle">{{ t.migrationWizard.subtitle }}</p>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Step Indicator Bar -->
      <div class="step-indicator-bar">
        <div class="step-badge" :class="{ active: currentStep === 1, completed: currentStep > 1 }">
          <span class="step-num">1</span>
          <span class="step-label">Audit</span>
        </div>
        <div class="step-line" :class="{ filled: currentStep > 1 }"></div>
        <div class="step-badge" :class="{ active: currentStep === 2, completed: currentStep > 2 }">
          <span class="step-num">2</span>
          <span class="step-label">Config</span>
        </div>
        <div class="step-line" :class="{ filled: currentStep > 2 }"></div>
        <div class="step-badge" :class="{ active: currentStep === 3, completed: currentStep > 3 }">
          <span class="step-num">3</span>
          <span class="step-label">Generate</span>
        </div>
        <div class="step-line" :class="{ filled: currentStep > 3 }"></div>
        <div class="step-badge" :class="{ active: currentStep === 4, completed: currentStep === 4 }">
          <span class="step-num">4</span>
          <span class="step-label">Ready</span>
        </div>
      </div>

      <!-- Error Alert -->
      <div v-if="exportError" class="alert-box error">
        <span>⚠️ {{ exportError }}</span>
      </div>

      <!-- Modal Body -->
      <div class="modal-body">
        <!-- Step 1: Library Audit -->
        <div v-if="currentStep === 1" class="step-content">
          <div class="step-header">
            <h4 class="step-title">{{ t.migrationWizard.step1Title }}</h4>
            <p class="step-desc">{{ t.migrationWizard.step1Desc }}</p>
          </div>

          <div class="stats-grid">
            <div class="stat-card">
              <span class="stat-icon">🖼️</span>
              <div class="stat-meta">
                <span class="stat-val">{{ stats ? stats.file_count.toLocaleString() : '...' }}</span>
                <span class="stat-name">{{ t.migrationWizard.totalFiles }}</span>
              </div>
            </div>

            <div class="stat-card">
              <span class="stat-icon">📁</span>
              <div class="stat-meta">
                <span class="stat-val">{{ stats ? stats.album_count.toLocaleString() : '...' }}</span>
                <span class="stat-name">{{ t.migrationWizard.totalAlbums }}</span>
              </div>
            </div>

            <div class="stat-card">
              <span class="stat-icon">🏷️</span>
              <div class="stat-meta">
                <span class="stat-val">{{ stats ? stats.tag_count.toLocaleString() : '...' }}</span>
                <span class="stat-name">{{ t.migrationWizard.totalTags }}</span>
              </div>
            </div>

            <div class="stat-card">
              <span class="stat-icon">💾</span>
              <div class="stat-meta">
                <span class="stat-val">{{ stats ? formatBytes(stats.db_size_bytes) : '...' }}</span>
                <span class="stat-name">Local SQLite Size</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 2: Target Dialect & Storage Configuration -->
        <div v-if="currentStep === 2" class="step-content">
          <div class="step-header">
            <h4 class="step-title">{{ t.migrationWizard.step2Title }}</h4>
            <p class="step-desc">{{ t.migrationWizard.step2Desc }}</p>
          </div>

          <div class="form-group">
            <label class="form-label">{{ t.migrationWizard.targetDialect }}</label>
            <select v-model="dialect" class="form-select">
              <option value="mysql">{{ t.migrationWizard.dialectMysql }}</option>
              <option value="postgres">{{ t.migrationWizard.dialectPostgres }}</option>
            </select>
          </div>

          <div class="form-group">
            <div class="label-with-hint">
              <label class="form-label">{{ t.migrationWizard.targetRootUuid }}</label>
              <button type="button" class="btn-text-action" @click="generateNewUuid">🎲 New UUID</button>
            </div>
            <input v-model="targetRootUuid" type="text" class="form-input mono" placeholder="Canonical UUID" />
            <span class="input-hint">{{ t.migrationWizard.targetRootUuidDesc }}</span>
          </div>

          <div class="form-group">
            <label class="form-label">{{ t.migrationWizard.destFile }}</label>
            <div class="path-input-group">
              <input
                v-model="destPath"
                type="text"
                class="form-input path-input"
                :placeholder="t.migrationWizard.noDestSelected"
                readonly
              />
              <button type="button" class="btn-browse" @click="handleBrowseDestination">
                {{ t.migrationWizard.browseDestination }}
              </button>
            </div>
          </div>
        </div>

        <!-- Step 3: Export in Progress -->
        <div v-if="currentStep === 3" class="step-content centered">
          <div class="loading-spinner"></div>
          <h4 class="step-title">{{ t.migrationWizard.step3Title }}</h4>
          <p class="step-desc">{{ t.migrationWizard.step3Desc }}</p>
        </div>

        <!-- Step 4: Ready -->
        <div v-if="currentStep === 4 && exportSummary" class="step-content">
          <div class="success-banner">
            <span class="banner-icon">✅</span>
            <span class="banner-text">
              {{ t.migrationWizard.successNotice.replace('{duration}', String(exportSummary.duration_ms)) }}
            </span>
          </div>

          <div class="summary-breakdown">
            <div class="summary-pill">
              <strong>{{ exportSummary.target_dialect.toUpperCase() }}</strong> target dialect
            </div>
            <div class="summary-pill">
              <strong>{{ exportSummary.total_files.toLocaleString() }}</strong> files
            </div>
            <div class="summary-pill">
              <strong>{{ exportSummary.total_albums }}</strong> albums
            </div>
            <div class="summary-pill">
              <strong>{{ exportSummary.total_tags }}</strong> tags
            </div>
            <div class="summary-pill">
              <strong>{{ exportSummary.total_tag_associations }}</strong> tag associations
            </div>
          </div>

          <div class="cli-section">
            <span class="cli-label">{{ t.migrationWizard.cliInstruction }}</span>
            <div class="cli-code-container">
              <code class="cli-code">{{ cliCommand }}</code>
              <button type="button" class="btn-copy" @click="copyCliCommand">
                {{ copied ? t.migrationWizard.copied : t.migrationWizard.copyCommand }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="modal-footer">
        <div class="footer-left">
          <button v-if="currentStep === 2" type="button" class="btn-secondary" @click="currentStep = 1">
            {{ t.migrationWizard.back }}
          </button>
        </div>

        <div class="footer-right">
          <button v-if="currentStep === 1" type="button" class="btn-primary" @click="currentStep = 2">
            {{ t.migrationWizard.next }} ➔
          </button>

          <button
            v-if="currentStep === 2"
            type="button"
            class="btn-primary"
            :disabled="!destPath.trim()"
            @click="handleStartExport"
          >
            🚀 {{ t.migrationWizard.startExport }}
          </button>

          <button v-if="currentStep === 4" type="button" class="btn-primary" @click="emit('close')">
            {{ t.migrationWizard.close }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.72);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
  padding: 1.5rem;
}

.modal-dialog {
  background: #181920;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 100%;
  max-width: 680px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.65);
  color: #e2e8f0;
  overflow: hidden;
}

.modal-header {
  padding: 1.25rem 1.5rem;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.modal-title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 600;
  color: #f8fafc;
}

.modal-subtitle {
  margin: 0.35rem 0 0 0;
  font-size: 0.82rem;
  color: #94a3b8;
}

.close-btn {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
}
.close-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

.step-indicator-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.9rem 2.5rem;
  background: rgba(0, 0, 0, 0.25);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.step-badge {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  opacity: 0.45;
  transition: opacity 0.2s ease;
}
.step-badge.active {
  opacity: 1;
  color: #818cf8;
}
.step-badge.completed {
  opacity: 0.85;
  color: #34d399;
}

.step-num {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 1.5px solid currentColor;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.72rem;
  font-weight: 600;
}

.step-label {
  font-size: 0.8rem;
  font-weight: 500;
}

.step-line {
  flex: 1;
  height: 2px;
  background: rgba(255, 255, 255, 0.1);
  margin: 0 0.75rem;
  transition: background 0.2s ease;
}
.step-line.filled {
  background: #34d399;
}

.alert-box {
  margin: 1rem 1.5rem 0 1.5rem;
  padding: 0.75rem 1rem;
  border-radius: 6px;
  font-size: 0.85rem;
}
.alert-box.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.35);
  color: #fca5a5;
}

.modal-body {
  padding: 1.5rem;
  overflow-y: auto;
  min-height: 260px;
}

.step-header {
  margin-bottom: 1.25rem;
}

.step-title {
  margin: 0 0 0.3rem 0;
  font-size: 1.02rem;
  font-weight: 600;
  color: #f1f5f9;
}

.step-desc {
  margin: 0;
  font-size: 0.82rem;
  color: #94a3b8;
  line-height: 1.45;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  padding: 1rem;
}

.stat-icon {
  font-size: 1.8rem;
}

.stat-meta {
  display: flex;
  flex-direction: column;
}

.stat-val {
  font-size: 1.25rem;
  font-weight: 700;
  color: #f8fafc;
}

.stat-name {
  font-size: 0.78rem;
  color: #94a3b8;
  margin-top: 2px;
}

.form-group {
  margin-bottom: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.label-with-hint {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.form-label {
  font-size: 0.85rem;
  font-weight: 500;
  color: #cbd5e1;
}

.btn-text-action {
  background: none;
  border: none;
  color: #818cf8;
  font-size: 0.78rem;
  cursor: pointer;
  padding: 0;
}
.btn-text-action:hover {
  text-decoration: underline;
}

.form-select,
.form-input {
  background: #121317;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 0.55rem 0.75rem;
  color: #f8fafc;
  font-size: 0.88rem;
  outline: none;
  transition: border-color 0.2s ease;
}
.form-select:focus,
.form-input:focus {
  border-color: #6366f1;
}

.form-input.mono {
  font-family: monospace;
  font-size: 0.82rem;
}

.input-hint {
  font-size: 0.75rem;
  color: #64748b;
}

.path-input-group {
  display: flex;
  gap: 0.5rem;
}

.path-input {
  flex: 1;
}

.btn-browse {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  color: #e2e8f0;
  font-size: 0.82rem;
  padding: 0 0.85rem;
  cursor: pointer;
  white-space: nowrap;
}
.btn-browse:hover {
  background: rgba(255, 255, 255, 0.14);
}

.centered {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 2.5rem 1rem;
}

.loading-spinner {
  width: 36px;
  height: 36px;
  border: 3px solid rgba(99, 102, 241, 0.2);
  border-top-color: #6366f1;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 1.25rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.success-banner {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  background: rgba(52, 211, 153, 0.12);
  border: 1px solid rgba(52, 211, 153, 0.35);
  border-radius: 8px;
  padding: 0.85rem 1rem;
  color: #6ee7b7;
  font-size: 0.9rem;
  font-weight: 500;
  margin-bottom: 1.25rem;
}

.summary-breakdown {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
}

.summary-pill {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0.4rem 0.65rem;
  font-size: 0.8rem;
  color: #cbd5e1;
}

.cli-section {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}

.cli-label {
  font-size: 0.8rem;
  color: #94a3b8;
}

.cli-code-container {
  display: flex;
  align-items: center;
  background: #0f1013;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 0.45rem 0.75rem;
  gap: 0.75rem;
}

.cli-code {
  flex: 1;
  font-family: monospace;
  font-size: 0.82rem;
  color: #a5b4fc;
  overflow-x: auto;
  white-space: nowrap;
}

.btn-copy {
  background: rgba(99, 102, 241, 0.15);
  border: 1px solid rgba(99, 102, 241, 0.3);
  color: #c7d2fe;
  border-radius: 4px;
  padding: 0.35rem 0.75rem;
  font-size: 0.78rem;
  cursor: pointer;
  white-space: nowrap;
}
.btn-copy:hover {
  background: rgba(99, 102, 241, 0.28);
}

.modal-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-right {
  display: flex;
  gap: 0.75rem;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #cbd5e1;
  border-radius: 6px;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  cursor: pointer;
}
.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.1);
}

.btn-primary {
  background: #4f46e5;
  border: 1px solid #6366f1;
  color: #ffffff;
  border-radius: 6px;
  padding: 0.5rem 1.1rem;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease;
}
.btn-primary:hover:not(:disabled) {
  background: #4338ca;
}
.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
