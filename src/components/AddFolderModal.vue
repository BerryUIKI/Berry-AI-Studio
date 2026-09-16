<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type { Folder, FolderType, IngestAction, PipelineDetectedPath } from "../types";

defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", val: boolean): void;
  (e: "folderAdded", folder: Folder): void;
}>();

const selectedMode = ref<FolderType>("link");
const folderPath = ref("");
const sourcePath = ref("");
const ingestAction = ref<IngestAction>("copy");
const gracePeriodHours = ref<number>(24);
const autoHarvest = ref<boolean>(true);

const detectedPaths = ref<PipelineDetectedPath[]>([]);
const detecting = ref(false);
const submitting = ref(false);
const error = ref("");

onMounted(() => {
  void scanLocalAiPaths();
  window.addEventListener("keydown", onKeydown);
});

async function scanLocalAiPaths() {
  detecting.value = true;
  try {
    detectedPaths.value = await invoke<PipelineDetectedPath[]>("autodetect_local_ai_paths");
  } catch (e) {
    console.warn("Failed to auto-detect AI paths:", e);
  } finally {
    detecting.value = false;
  }
}

async function pickFolder(target: "folderPath" | "sourcePath") {
  error.value = "";
  try {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string" && selected) {
      if (target === "folderPath") {
        folderPath.value = selected;
      } else {
        sourcePath.value = selected;
      }
    }
  } catch (e) {
    error.value = String(e);
  }
}

function selectDetected(p: PipelineDetectedPath) {
  sourcePath.value = p.path;
  selectedMode.value = "pipeline";
}

async function onSubmit() {
  if (!folderPath.value.trim()) {
    error.value = t.value.addFolder.errorNoTarget;
    return;
  }
  if (selectedMode.value === "pipeline" && !sourcePath.value.trim()) {
    error.value = t.value.addFolder.errorNoSource;
    return;
  }

  error.value = "";
  submitting.value = true;

  try {
    const folder = await invoke<Folder>("add_folder_with_options", {
      path: folderPath.value.trim(),
      folderType: selectedMode.value,
      sourcePath: selectedMode.value === "pipeline" ? sourcePath.value.trim() : null,
      ingestAction: selectedMode.value === "pipeline" ? ingestAction.value : null,
      gracePeriodHours: selectedMode.value === "pipeline" ? gracePeriodHours.value : null,
      autoHarvest: selectedMode.value === "pipeline" ? autoHarvest.value : true,
    });

    emit("folderAdded", folder);
    close();
  } catch (e) {
    error.value = String(e);
  } finally {
    submitting.value = false;
  }
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    close();
  }
}

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});

function close() {
  folderPath.value = "";
  sourcePath.value = "";
  error.value = "";
  emit("update:open", false);
}
</script>

<template>
  <div v-if="open" class="modal-backdrop" @click.self="close">
    <div class="modal-dialog">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-title-wrap">
          <span class="header-icon">📁</span>
          <h2>{{ t.addFolder.title }}</h2>
        </div>
        <button type="button" class="btn-close" @click="close">✕</button>
      </div>

      <div v-if="error" class="modal-error">
        ⚠️ {{ error }}
      </div>

      <div class="modal-body">
        <!-- 3 Mode Cards -->
        <div class="mode-cards">
          <!-- Mode A: Link -->
          <div
            class="mode-card"
            :class="{ active: selectedMode === 'link' }"
            @click="selectedMode = 'link'"
          >
            <div class="card-radio">
              <span class="radio-circle" :class="{ selected: selectedMode === 'link' }"></span>
            </div>
            <div class="card-content">
              <div class="mode-header">
                <span class="mode-emoji">🔗</span>
                <strong>{{ t.addFolder.modeLinkTitle }}</strong>
              </div>
              <p class="mode-desc">{{ t.addFolder.modeLinkDesc }}</p>
            </div>
          </div>

          <!-- Mode B: Managed Vault -->
          <div
            class="mode-card"
            :class="{ active: selectedMode === 'managed' }"
            @click="selectedMode = 'managed'"
          >
            <div class="card-radio">
              <span class="radio-circle" :class="{ selected: selectedMode === 'managed' }"></span>
            </div>
            <div class="card-content">
              <div class="mode-header">
                <span class="mode-emoji">📦</span>
                <strong>{{ t.addFolder.modeManagedTitle }}</strong>
              </div>
              <p class="mode-desc">{{ t.addFolder.modeManagedDesc }}</p>
            </div>
          </div>

          <!-- Mode C: Ingest Pipeline -->
          <div
            class="mode-card"
            :class="{ active: selectedMode === 'pipeline' }"
            @click="selectedMode = 'pipeline'"
          >
            <div class="card-radio">
              <span class="radio-circle" :class="{ selected: selectedMode === 'pipeline' }"></span>
            </div>
            <div class="card-content">
              <div class="mode-header">
                <span class="mode-emoji">⚡</span>
                <strong>{{ t.addFolder.modePipelineTitle }}</strong>
                <span class="badge-tag">AIGC</span>
              </div>
              <p class="mode-desc">{{ t.addFolder.modePipelineDesc }}</p>
            </div>
          </div>
        </div>

        <!-- Mode-Specific Configuration Form -->
        <div class="form-section">
          <!-- Folder Path (Target Vault / Link Directory) -->
          <div class="form-group">
            <label>
              {{ selectedMode === 'pipeline' ? t.addFolder.libraryTargetPath : t.addFolder.folderPath }}
            </label>
            <div class="path-input-group">
              <input
                v-model="folderPath"
                type="text"
                :placeholder="selectedMode === 'pipeline' ? t.addFolder.libraryTargetPlaceholder : t.addFolder.folderPlaceholder"
              />
              <button type="button" class="btn-browse" @click="pickFolder('folderPath')">
                {{ t.addFolder.browse }}
              </button>
            </div>
          </div>

          <!-- Pipeline Specific Settings -->
          <template v-if="selectedMode === 'pipeline'">
            <div class="form-group">
              <label>{{ t.addFolder.pipelineSourcePath }}</label>
              <div class="path-input-group">
                <input
                  v-model="sourcePath"
                  type="text"
                  :placeholder="t.addFolder.pipelineSourcePlaceholder"
                />
                <button type="button" class="btn-browse" @click="pickFolder('sourcePath')">
                  {{ t.addFolder.browse }}
                </button>
              </div>
            </div>

            <!-- Detected AI Tool Quick Pick -->
            <div v-if="detectedPaths.length > 0" class="detected-box">
              <div class="detected-title">
                <span>🤖 {{ t.addFolder.detectedAiOutputs }}</span>
                <button type="button" class="btn-rescan" :disabled="detecting" @click="scanLocalAiPaths">
                  {{ detecting ? t.addFolder.detecting : t.addFolder.refresh }}
                </button>
              </div>
              <div class="detected-list">
                <button
                  v-for="(dp, idx) in detectedPaths"
                  :key="idx"
                  type="button"
                  class="detected-chip"
                  :class="{ active: sourcePath === dp.path }"
                  @click="selectDetected(dp)"
                >
                  <span class="chip-tool">{{ dp.tool_name }}</span>
                  <span class="chip-cat">({{ dp.category }})</span>
                </button>
              </div>
            </div>

            <!-- Ingest Action & Grace Period -->
            <div class="pipeline-grid">
              <div class="form-group">
                <label>{{ t.addFolder.action }}</label>
                <select v-model="ingestAction" class="select-field">
                  <option value="copy">{{ t.addFolder.actionCopy }}</option>
                  <option value="move">{{ t.addFolder.actionMove }}</option>
                </select>
              </div>

              <div v-if="ingestAction === 'move'" class="form-group">
                <label>{{ t.addFolder.gracePeriod }}</label>
                <select v-model.number="gracePeriodHours" class="select-field">
                  <option :value="0">{{ t.addFolder.graceImmediate }}</option>
                  <option :value="1">1 {{ t.addFolder.hour }}</option>
                  <option :value="24">24 {{ t.addFolder.hours }}</option>
                  <option :value="72">3 {{ t.addFolder.days }}</option>
                  <option :value="168">7 {{ t.addFolder.days }}</option>
                </select>
              </div>
            </div>

            <div class="toggle-row">
              <label class="toggle-label">
                <input v-model="autoHarvest" type="checkbox" />
                <span>{{ t.addFolder.autoHarvest }}</span>
              </label>
              <span class="toggle-hint">{{ t.addFolder.autoHarvestHint }}</span>
            </div>
          </template>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="modal-footer">
        <button type="button" class="btn-cancel" @click="close">
          {{ t.settings.cancel }}
        </button>
        <button
          type="button"
          class="btn-submit"
          :disabled="submitting || !folderPath"
          @click="onSubmit"
        >
          {{ submitting ? t.nav.scanning : t.addFolder.confirmAdd }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(5px);
  z-index: 300;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-dialog {
  background: #1e1e1e;
  color: #fff;
  width: 90%;
  max-width: 580px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: zoomIn 0.15s ease-out;
}

@media (prefers-color-scheme: light) {
  .modal-dialog {
    background: #ffffff;
    color: #1a1a1a;
    border: 1px solid rgba(0, 0, 0, 0.15);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.2);
  }
}

@keyframes zoomIn {
  from {
    transform: scale(0.96);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.4rem;
  border-bottom: 1px solid rgba(128, 128, 128, 0.2);
}

.header-title-wrap {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.header-icon {
  font-size: 1.2rem;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.15em;
  font-weight: 600;
}

.btn-close {
  background: transparent;
  border: none;
  color: inherit;
  font-size: 1.1em;
  cursor: pointer;
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
}

.btn-close:hover {
  background: rgba(128, 128, 128, 0.2);
}

.modal-error {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  padding: 0.6rem 1.2rem;
  font-size: 0.85em;
  border-bottom: 1px solid rgba(239, 68, 68, 0.3);
}

.modal-body {
  padding: 1.2rem 1.4rem;
  overflow-y: auto;
  max-height: 70vh;
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

.mode-cards {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}

.mode-card {
  display: flex;
  align-items: flex-start;
  gap: 0.8rem;
  padding: 0.8rem 1rem;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: rgba(128, 128, 128, 0.05);
  cursor: pointer;
  transition: all 0.15s ease;
}

.mode-card:hover {
  border-color: #2f6fed;
  background: rgba(47, 111, 237, 0.05);
}

.mode-card.active {
  border-color: #2f6fed;
  background: rgba(47, 111, 237, 0.12);
}

.card-radio {
  padding-top: 0.15rem;
}

.radio-circle {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid rgba(128, 128, 128, 0.6);
  position: relative;
}

.radio-circle.selected {
  border-color: #2f6fed;
  background: #2f6fed;
  box-shadow: inset 0 0 0 2px #fff;
}

.card-content {
  flex: 1;
}

.mode-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.95em;
}

.mode-emoji {
  font-size: 1.1em;
}

.badge-tag {
  font-size: 0.7em;
  background: #2f6fed;
  color: #fff;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  font-weight: bold;
}

.mode-desc {
  margin: 0.3rem 0 0;
  font-size: 0.82em;
  color: #888;
  line-height: 1.35;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  padding-top: 0.5rem;
  border-top: 1px solid rgba(128, 128, 128, 0.15);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.form-group label {
  font-size: 0.8em;
  color: #999;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  font-weight: 600;
}

.path-input-group {
  display: flex;
  gap: 0.5rem;
}

.path-input-group input,
.select-field {
  flex: 1;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: rgba(0, 0, 0, 0.2);
  color: inherit;
  font: inherit;
  font-size: 0.88em;
}

@media (prefers-color-scheme: light) {
  .path-input-group input,
  .select-field {
    background: #fff;
  }
}

.btn-browse {
  padding: 0.5rem 0.9rem;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: 0.85em;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-browse:hover {
  background: rgba(128, 128, 128, 0.15);
}

.detected-box {
  background: rgba(47, 111, 237, 0.08);
  border: 1px dashed rgba(47, 111, 237, 0.3);
  border-radius: 8px;
  padding: 0.65rem 0.8rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.detected-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.82em;
  font-weight: 600;
  color: #2f6fed;
}

.btn-rescan {
  background: transparent;
  border: none;
  color: inherit;
  cursor: pointer;
  font-size: 0.8em;
  text-decoration: underline;
  opacity: 0.8;
}

.detected-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
}

.detected-chip {
  padding: 0.3rem 0.6rem;
  border-radius: 6px;
  border: 1px solid rgba(47, 111, 237, 0.3);
  background: rgba(47, 111, 237, 0.1);
  color: inherit;
  font: inherit;
  font-size: 0.8em;
  cursor: pointer;
  display: flex;
  gap: 0.3rem;
  align-items: center;
  transition: all 0.15s ease;
}

.detected-chip:hover {
  background: rgba(47, 111, 237, 0.25);
}

.detected-chip.active {
  background: #2f6fed;
  color: #fff;
  border-color: #2f6fed;
}

.pipeline-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.8rem;
}

.toggle-row {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  margin-top: 0.2rem;
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.88em;
  cursor: pointer;
}

.toggle-hint {
  font-size: 0.78em;
  color: #888;
  margin-left: 1.5rem;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.6rem;
  padding: 1rem 1.4rem;
  border-top: 1px solid rgba(128, 128, 128, 0.2);
}

.btn-cancel,
.btn-submit {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font: inherit;
  font-size: 0.88em;
  cursor: pointer;
}

.btn-cancel {
  background: transparent;
  border: 1px solid rgba(128, 128, 128, 0.3);
  color: inherit;
}

.btn-submit {
  background: #2f6fed;
  color: #fff;
  border: none;
  font-weight: 500;
}

.btn-submit:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
