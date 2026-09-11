<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type { ClipModelSummary, ClipIndexStatus, ClipBatchIndexResult } from "../types";

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "indexed"): void;
}>();

const models = ref<ClipModelSummary[]>([]);
const selectedDirPath = ref<string>("");
const loadedModel = ref<{ name: string; model_id: string } | null>(null);
const indexStatus = ref<ClipIndexStatus | null>(null);

const batchSize = ref<number>(20);
const isIndexing = ref<boolean>(false);
const shouldStopIndexing = ref<boolean>(false);
const message = ref<{ type: "success" | "error"; text: string } | null>(null);

const percentIndexed = computed(() => {
  if (!indexStatus.value || indexStatus.value.total_images === 0) return 0;
  return Math.min(
    100,
    Math.round((indexStatus.value.indexed_images / indexStatus.value.total_images) * 100),
  );
});

async function loadModelsAndStatus() {
  try {
    const list = await invoke<ClipModelSummary[]>("list_clip_models");
    models.value = list;

    const current = await invoke<{ name: string; model_id: string } | null>("get_loaded_clip_model");
    loadedModel.value = current;

    if (current) {
      selectedDirPath.value = current.model_id;
    } else if (list.length > 0) {
      selectedDirPath.value = list[0].dir_path;
      // Auto load first model if available
      await onSelectModel(list[0]);
    }

    await refreshIndexStatus();
  } catch (err: any) {
    console.error("Failed to load CLIP models:", err);
  }
}

async function refreshIndexStatus() {
  try {
    const status = await invoke<ClipIndexStatus>("get_clip_index_status", {
      modelId: loadedModel.value?.model_id || null,
    });
    indexStatus.value = status;
  } catch (err: any) {
    console.error("Failed to fetch index status:", err);
  }
}

async function onSelectModel(model: ClipModelSummary) {
  try {
    message.value = null;
    selectedDirPath.value = model.dir_path;
    const info = await invoke<{ name: string; model_id: string }>("load_clip_model", {
      dirPath: model.dir_path,
    });
    loadedModel.value = info;
    models.value = models.value.map((m) => ({
      ...m,
      is_loaded: m.dir_path === model.dir_path,
    }));
    await refreshIndexStatus();
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  }
}

async function onBrowseFolder() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: t.value.clipModal.browseFolder,
    });

    if (selected && typeof selected === "string") {
      message.value = null;
      const info = await invoke<{ name: string; model_id: string }>("load_clip_model", {
        dirPath: selected,
      });
      loadedModel.value = info;
      await loadModelsAndStatus();
      message.value = { type: "success", text: t.value.clipModal.modelLoaded };
    }
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  }
}

async function runBatchIndexingLoop() {
  if (!loadedModel.value) return;
  isIndexing.value = true;
  shouldStopIndexing.value = false;
  message.value = null;

  try {
    while (isIndexing.value && !shouldStopIndexing.value) {
      const res = await invoke<ClipBatchIndexResult>("index_clip_images_batch", {
        batchSize: batchSize.value,
      });

      await refreshIndexStatus();
      emit("indexed");

      if (res.indexed_count === 0 || res.remaining_count === 0) {
        message.value = { type: "success", text: t.value.clipModal.indexingComplete };
        break;
      }

      // Small delay to allow UI render
      await new Promise((r) => setTimeout(r, 80));
    }
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isIndexing.value = false;
    shouldStopIndexing.value = false;
  }
}

function stopIndexing() {
  shouldStopIndexing.value = true;
}

onMounted(() => {
  loadModelsAndStatus();
});
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-dialog">
      <div class="modal-header">
        <div class="modal-title">
          <span class="title-icon">🧠</span>
          <h3>{{ t.clipModal.title }}</h3>
        </div>
        <button class="close-btn" @click="emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <!-- Model Selection Section -->
        <div class="section-card">
          <div class="section-title">{{ t.clipModal.modelLabel }}</div>

          <div v-if="models.length > 0" class="model-select-row">
            <select
              class="form-select"
              :value="selectedDirPath"
              @change="
                (e) => {
                  const target = e.target as HTMLSelectElement;
                  const found = models.find((m) => m.dir_path === target.value);
                  if (found) onSelectModel(found);
                }
              "
            >
              <option
                v-for="m in models"
                :key="m.dir_path"
                :value="m.dir_path"
              >
                {{ m.name }} {{ m.is_loaded ? `(${t.clipModal.modelLoaded})` : '' }}
              </option>
            </select>
            <button class="action-btn secondary" @click="onBrowseFolder">
              📂 {{ t.clipModal.browseFolder }}
            </button>
          </div>

          <div v-else class="empty-hint">
            <p>{{ t.clipModal.noModels }}</p>
            <button class="action-btn primary" @click="onBrowseFolder">
              📂 {{ t.clipModal.browseFolder }}
            </button>
          </div>

          <div v-if="loadedModel" class="active-badge">
            <span class="dot"></span>
            <span>{{ t.clipModal.modelLoaded }}: <strong>{{ loadedModel.name }}</strong></span>
          </div>
        </div>

        <!-- Embedding Progress & Batch Indexing Section -->
        <div class="section-card">
          <div class="section-title">{{ t.clipModal.indexStatus }}</div>

          <div class="progress-container">
            <div class="progress-bar-wrapper">
              <div
                class="progress-bar-fill"
                :style="{ width: `${percentIndexed}%` }"
              ></div>
            </div>
            <div class="progress-text">
              {{
                t.clipModal.indexedRatio
                  .replace('{indexed}', String(indexStatus?.indexed_images ?? 0))
                  .replace('{total}', String(indexStatus?.total_images ?? 0))
                  .replace('{percent}', String(percentIndexed))
              }}
            </div>
          </div>

          <div class="batch-controls">
            <div class="input-group">
              <label>{{ t.clipModal.batchSize }}</label>
              <select v-model.number="batchSize" class="form-select-sm" :disabled="isIndexing">
                <option :value="10">10</option>
                <option :value="20">20</option>
                <option :value="50">50</option>
                <option :value="100">100</option>
              </select>
            </div>

            <div class="buttons-group">
              <button
                v-if="!isIndexing"
                class="action-btn primary"
                :disabled="!loadedModel || Boolean(indexStatus && indexStatus.indexed_images >= indexStatus.total_images)"
                @click="runBatchIndexingLoop"
              >
                ⚡ {{ t.clipModal.startIndexing }}
              </button>
              <button
                v-else
                class="action-btn danger"
                @click="stopIndexing"
              >
                🛑 {{ t.clipModal.stopIndexing }}
              </button>
            </div>
          </div>

          <div v-if="isIndexing" class="indexing-indicator">
            <span class="spinner">⏳</span>
            <span>{{ t.clipModal.indexing }}</span>
          </div>

          <div class="search-tip">
            {{ t.clipModal.searchTip }}
          </div>
        </div>

        <div v-if="message" :class="['alert-message', message.type]">
          {{ message.text }}
        </div>
      </div>

      <div class="modal-footer">
        <button class="action-btn secondary" @click="emit('close')">
          {{ t.clipModal.close }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-dialog {
  background: #18181b;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 580px;
  max-width: 90vw;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-title h3 {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: #f1f5f9;
}

.title-icon {
  font-size: 1.2rem;
}

.close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.close-btn:hover {
  color: #f1f5f9;
  background: rgba(255, 255, 255, 0.08);
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-height: 75vh;
  overflow-y: auto;
}

.section-card {
  background: #202024;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 8px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 0.85rem;
  font-weight: 600;
  color: #a1a1aa;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.model-select-row {
  display: flex;
  gap: 10px;
  align-items: center;
}

.form-select {
  flex: 1;
  background: #27272a;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: #f1f5f9;
  padding: 6px 10px;
  font-size: 0.85rem;
  outline: none;
}

.form-select-sm {
  background: #27272a;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: #f1f5f9;
  padding: 4px 8px;
  font-size: 0.82rem;
  outline: none;
}

.empty-hint {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 0.82rem;
  color: #a1a1aa;
}

.active-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  color: #10b981;
}

.dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #10b981;
  box-shadow: 0 0 6px #10b981;
}

.progress-container {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-bar-wrapper {
  height: 8px;
  background: #27272a;
  border-radius: 4px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #8b5cf6, #3b82f6);
  transition: width 0.3s ease;
}

.progress-text {
  font-size: 0.8rem;
  color: #94a3b8;
}

.batch-controls {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 4px;
}

.input-group {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.82rem;
  color: #cbd5e1;
}

.buttons-group {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.action-btn.primary {
  background: #6366f1;
  color: #ffffff;
}

.action-btn.primary:hover:not(:disabled) {
  background: #4f46e5;
}

.action-btn.primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.secondary {
  background: rgba(255, 255, 255, 0.08);
  color: #cbd5e1;
  border-color: rgba(255, 255, 255, 0.1);
}

.action-btn.secondary:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #ffffff;
}

.action-btn.danger {
  background: #ef4444;
  color: #ffffff;
}

.action-btn.danger:hover {
  background: #dc2626;
}

.indexing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.82rem;
  color: #a855f7;
  font-weight: 500;
}

.search-tip {
  font-size: 0.78rem;
  color: #94a3b8;
  background: rgba(139, 92, 246, 0.08);
  border-left: 3px solid #8b5cf6;
  padding: 6px 10px;
  border-radius: 0 4px 4px 0;
}

.alert-message {
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.82rem;
}

.alert-message.success {
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #34d399;
}

.alert-message.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.modal-footer {
  padding: 12px 20px;
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}
</style>
