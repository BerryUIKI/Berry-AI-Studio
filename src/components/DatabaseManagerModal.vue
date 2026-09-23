<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save, open as openDialog } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type { DatabaseStats } from "../types";
import { formatBytes } from "../utils/image";

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "databaseChanged"): void;
}>();

const stats = ref<DatabaseStats | null>(null);
const loading = ref(false);
const processingAction = ref(false);
const statusMessage = ref<{ text: string; type: "success" | "error" } | null>(null);

async function loadStats() {
  loading.value = true;
  statusMessage.value = null;
  try {
    stats.value = await invoke<DatabaseStats>("get_database_stats");
  } catch (err: any) {
    statusMessage.value = { text: `Failed to load stats: ${err}`, type: "error" };
  } finally {
    loading.value = false;
  }
}

async function handleVacuum() {
  processingAction.value = true;
  statusMessage.value = null;
  try {
    await invoke("vacuum_database");
    await loadStats();
    statusMessage.value = {
      text: "Database optimized and compacted successfully! Freelist pages reclaimed.",
      type: "success",
    };
    emit("databaseChanged");
  } catch (err: any) {
    statusMessage.value = { text: `Vacuum failed: ${err}`, type: "error" };
  } finally {
    processingAction.value = false;
  }
}

async function handleBackup() {
  try {
    const defaultName = `berry_backup_${new Date().toISOString().slice(0, 10)}.db`;
    const destination = await save({
      defaultPath: defaultName,
      filters: [{ name: "SQLite Database", extensions: ["db", "sqlite"] }],
      title: t.value.dbModal.backupTitle,
    });

    if (!destination) return;

    processingAction.value = true;
    statusMessage.value = null;

    await invoke("backup_database", { destinationPath: destination });
    statusMessage.value = {
      text: `Backup exported successfully to: ${destination}`,
      type: "success",
    };
  } catch (err: any) {
    statusMessage.value = { text: `Backup failed: ${err}`, type: "error" };
  } finally {
    processingAction.value = false;
  }
}

async function handleRestore() {
  try {
    const selected = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: "SQLite Database", extensions: ["db", "sqlite"] }],
      title: t.value.dbModal.restoreTitle,
    });

    if (!selected || typeof selected !== "string") return;

    if (
      !confirm(
        "Are you sure you want to restore this database? The current database will be replaced with the backup file."
      )
    ) {
      return;
    }

    processingAction.value = true;
    statusMessage.value = null;

    await invoke("restore_database", { sourcePath: selected });
    await loadStats();
    statusMessage.value = {
      text: "Database restored successfully! Library data reloaded.",
      type: "success",
    };
    emit("databaseChanged");
  } catch (err: any) {
    statusMessage.value = { text: `Restore failed: ${err}`, type: "error" };
  } finally {
    processingAction.value = false;
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      void loadStats();
    }
  }
);

onMounted(() => {
  if (props.show) {
    void loadStats();
  }
});
</script>

<template>
  <div v-if="show" class="modal-backdrop" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="modal-container">
      <div class="modal-header">
        <div class="header-left">
          <span class="header-icon">🗄️</span>
          <h2>{{ t.dbModal.title }}</h2>
        </div>
        <button class="close-btn" @click="emit('close')" :title="t.dbModal.close">✕</button>
      </div>

      <div class="modal-body">
        <div v-if="statusMessage" class="status-banner" :class="statusMessage.type">
          {{ statusMessage.text }}
        </div>

        <!-- Metrics Overview Grid -->
        <section class="section">
          <div class="section-title">{{ t.dbModal.metricsTitle }}</div>
          <div v-if="loading" class="loading-state">Loading database statistics...</div>
          <div v-else-if="stats" class="stats-grid">
            <div class="stat-card">
              <span class="stat-value">{{ formatBytes(stats.db_size_bytes) }}</span>
              <span class="stat-label">{{ t.dbModal.dbSize }}</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.file_count.toLocaleString() }}</span>
              <span class="stat-label">{{ t.dbModal.indexedImages }}</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.folder_count }}</span>
              <span class="stat-label">{{ t.dbModal.rootFolders }}</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.album_count }}</span>
              <span class="stat-label">{{ t.dbModal.albums }}</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.tag_count }}</span>
              <span class="stat-label">{{ t.dbModal.tags }}</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.model_cache_count.toLocaleString() }}</span>
              <span class="stat-label">{{ t.dbModal.cachedHashes }}</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.page_count }}</span>
              <span class="stat-label">{{ t.dbModal.sqlitePages }}</span>
            </div>
            <div class="stat-card">
              <span class="stat-value">{{ stats.freelist_count }}</span>
              <span class="stat-label">{{ t.dbModal.freePages }}</span>
            </div>
          </div>
        </section>

        <!-- Actions -->
        <section class="section">
          <div class="section-title">{{ t.dbModal.maintenanceTitle }}</div>
          <div class="action-cards">
            <!-- Vacuum & Optimize -->
            <div class="action-card">
              <div class="action-info">
                <div class="action-name">⚡ {{ t.dbModal.compactTitle }}</div>
                <div class="action-desc">
                  {{ t.dbModal.compactDesc }}
                </div>
              </div>
              <button
                type="button"
                class="btn btn-primary"
                :disabled="processingAction || loading"
                @click="handleVacuum"
              >
                {{ processingAction ? "Processing..." : t.dbModal.compactBtn }}
              </button>
            </div>

            <!-- Backup Database -->
            <div class="action-card">
              <div class="action-info">
                <div class="action-name">💾 {{ t.dbModal.backupTitle }}</div>
                <div class="action-desc">
                  {{ t.dbModal.backupDesc }}
                </div>
              </div>
              <button
                type="button"
                class="btn btn-secondary"
                :disabled="processingAction || loading"
                @click="handleBackup"
              >
                {{ t.dbModal.backupBtn }}
              </button>
            </div>

            <!-- Restore Database -->
            <div class="action-card danger-card">
              <div class="action-info">
                <div class="action-name">🔄 {{ t.dbModal.restoreTitle }}</div>
                <div class="action-desc">
                  {{ t.dbModal.restoreDesc }}
                </div>
              </div>
              <button
                type="button"
                class="btn btn-danger"
                :disabled="processingAction || loading"
                @click="handleRestore"
              >
                {{ t.dbModal.restoreBtn }}
              </button>
            </div>
          </div>
        </section>
      </div>

      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" @click="emit('close')">
          {{ t.dbModal.close }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
}

.modal-container {
  background: var(--bg-surface, #1e1e24);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  width: 90%;
  max-width: 680px;
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.4);
  color: var(--text-primary, #eee);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, #333);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-icon {
  font-size: 1.3rem;
}

.modal-header h2 {
  font-size: 1.15rem;
  font-weight: 600;
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary, #999);
  font-size: 1.2rem;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
}

.close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.status-banner {
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 0.88rem;
  line-height: 1.4;
}

.status-banner.success {
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.4);
  color: #4ade80;
}

.status-banner.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.4);
  color: #f87171;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--text-secondary, #aaa);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.loading-state {
  color: var(--text-secondary, #888);
  font-size: 0.9rem;
  padding: 12px 0;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
}

@media (max-width: 600px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}

.stat-card {
  background: var(--bg-card, #25252d);
  border: 1px solid var(--border-color, #333);
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.stat-value {
  font-size: 1.15rem;
  font-weight: 700;
  color: #fff;
  margin-bottom: 2px;
}

.stat-label {
  font-size: 0.75rem;
  color: var(--text-secondary, #999);
}

.action-cards {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.action-card {
  background: var(--bg-card, #25252d);
  border: 1px solid var(--border-color, #333);
  border-radius: 8px;
  padding: 14px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.action-info {
  flex: 1;
}

.action-name {
  font-size: 0.95rem;
  font-weight: 600;
  color: #fff;
  margin-bottom: 4px;
}

.action-desc {
  font-size: 0.8rem;
  color: var(--text-secondary, #aaa);
  line-height: 1.35;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  padding: 14px 20px;
  border-top: 1px solid var(--border-color, #333);
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 0.88rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
  white-space: nowrap;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.08);
  color: #ccc;
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.btn-primary {
  background: #3b82f6;
  color: #fff;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-danger {
  background: rgba(239, 68, 68, 0.2);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.4);
}

.btn-danger:hover:not(:disabled) {
  background: #ef4444;
  color: #fff;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
