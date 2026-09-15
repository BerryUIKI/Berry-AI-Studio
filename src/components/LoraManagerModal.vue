<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type { LoraModel } from "../types";

const props = defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "inject-prompt", text: string): void;
}>();

const loras = ref<LoraModel[]>([]);
const searchQuery = ref("");
const loading = ref(false);
const message = ref<{ type: "success" | "error"; text: string } | null>(null);

// Editing state
const isEditing = ref(false);
const editForm = ref<{
  id: number;
  name: string;
  hash: string;
  trigger_words: string;
  weight_default: number;
  description: string;
}>({
  id: 0,
  name: "",
  hash: "",
  trigger_words: "",
  weight_default: 1.0,
  description: "",
});

const isImporting = ref(false);

async function loadLoras() {
  loading.value = true;
  message.value = null;
  try {
    loras.value = await invoke<LoraModel[]>("list_loras");
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    loading.value = false;
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      loadLoras();
      isEditing.value = false;
    }
  }
);

onMounted(() => {
  if (props.show) {
    loadLoras();
  }
});

const filteredLoras = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return loras.value;
  return loras.value.filter((l) => {
    const matchName = l.name.toLowerCase().includes(q);
    const matchHash = l.hash?.toLowerCase().includes(q) ?? false;
    const matchTrigger = l.trigger_words.some((tw) => tw.toLowerCase().includes(q));
    const matchDesc = l.description?.toLowerCase().includes(q) ?? false;
    return matchName || matchHash || matchTrigger || matchDesc;
  });
});

const totalTriggersCount = computed(() => {
  return loras.value.reduce((acc, l) => acc + l.trigger_words.length, 0);
});

function openAddModal() {
  editForm.value = {
    id: 0,
    name: "",
    hash: "",
    trigger_words: "",
    weight_default: 1.0,
    description: "",
  };
  isEditing.value = true;
}

function openEditModal(lora: LoraModel) {
  editForm.value = {
    id: lora.id,
    name: lora.name,
    hash: lora.hash || "",
    trigger_words: lora.trigger_words.join(", "),
    weight_default: lora.weight_default,
    description: lora.description || "",
  };
  isEditing.value = true;
}

function cancelEdit() {
  isEditing.value = false;
}

async function saveEdit() {
  if (!editForm.value.name.trim()) {
    message.value = { type: "error", text: t.value.loraModal.nameRequired };
    return;
  }

  const triggers = editForm.value.trigger_words
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);

  const payload: LoraModel = {
    id: editForm.value.id,
    name: editForm.value.name.trim(),
    hash: editForm.value.hash.trim() || null,
    trigger_words: triggers,
    preview_url: null,
    description: editForm.value.description.trim() || null,
    weight_default: editForm.value.weight_default,
    created_at: "",
    updated_at: "",
  };

  try {
    await invoke<LoraModel>("save_lora", { lora: payload });
    message.value = { type: "success", text: t.value.loraModal.savedSuccess };
    isEditing.value = false;
    await loadLoras();
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  }
}

async function handleDelete(lora: LoraModel) {
  if (!confirm(`${t.value.loraModal.deleteConfirm} "${lora.name}"?`)) {
    return;
  }
  try {
    await invoke("delete_lora", { id: lora.id });
    await loadLoras();
    message.value = { type: "success", text: t.value.loraModal.deletedSuccess };
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  }
}

async function handleImportCivitaiInfo() {
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [
        {
          name: "Civitai Info / JSON",
          extensions: ["info", "json"],
        },
      ],
    });
    if (!selected || typeof selected !== "string") return;

    isImporting.value = true;
    message.value = null;
    const imported = await invoke<LoraModel>("import_lora_civitai_info", {
      filePath: selected,
    });
    message.value = {
      type: "success",
      text: `${t.value.loraModal.importedSuccess}: ${imported.name} (${imported.trigger_words.length} triggers)`,
    };
    await loadLoras();
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isImporting.value = false;
  }
}

async function handleScanDirectory() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
    });
    if (!selected || typeof selected !== "string") return;

    isImporting.value = true;
    message.value = null;
    const count = await invoke<number>("scan_loras_directory", {
      dirPath: selected,
    });
    message.value = {
      type: "success",
      text: `${t.value.loraModal.scanCompleted}: ${count} LoRA(s) imported/updated.`,
    };
    await loadLoras();
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isImporting.value = false;
  }
}

const copyFeedback = ref<string | null>(null);
let copyTimeout: any = null;

async function copyText(text: string, label: string) {
  try {
    await navigator.clipboard.writeText(text);
    copyFeedback.value = `${t.value.loraModal.copiedTrigger}: "${label}"`;
    if (copyTimeout) clearTimeout(copyTimeout);
    copyTimeout = setTimeout(() => {
      copyFeedback.value = null;
    }, 2200);
  } catch {
    // fallback
  }
}

function copyAllTriggers(lora: LoraModel) {
  if (!lora.trigger_words.length) return;
  const joined = lora.trigger_words.join(", ");
  copyText(joined, lora.name);
}

function injectPrompt(lora: LoraModel) {
  const promptTag = `<lora:${lora.name}:${lora.weight_default}>`;
  const fullText = lora.trigger_words.length
    ? `${promptTag}, ${lora.trigger_words.join(", ")}`
    : promptTag;
  emit("inject-prompt", fullText);
  copyText(fullText, promptTag);
}
</script>

<template>
  <div v-if="show" class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-container">
      <!-- Header -->
      <div class="modal-header">
        <div class="modal-title-wrap">
          <svg class="modal-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
          </svg>
          <h2>{{ t.loraModal.title }}</h2>
          <span class="badge-count">{{ loras.length }} {{ t.loraModal.modelsCount }}</span>
          <span v-if="totalTriggersCount > 0" class="badge-triggers">
            {{ totalTriggersCount }} {{ t.loraModal.triggersCount }}
          </span>
        </div>
        <button class="close-btn" @click="emit('close')">&times;</button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        <!-- Message Notification -->
        <div v-if="message" class="alert-msg" :class="message.type">
          {{ message.text }}
        </div>

        <div v-if="copyFeedback" class="toast-feedback">
          {{ copyFeedback }}
        </div>

        <!-- Controls Toolbar -->
        <div class="toolbar">
          <div class="search-box">
            <svg class="search-icon" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
            </svg>
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="t.loraModal.searchPlaceholder"
              class="search-input"
            />
          </div>

          <div class="btn-group">
            <button class="secondary-btn" :disabled="isImporting" @click="handleImportCivitaiInfo">
              <svg class="btn-icon" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" />
              </svg>
              {{ t.loraModal.importInfoBtn }}
            </button>

            <button class="secondary-btn" :disabled="isImporting" @click="handleScanDirectory">
              <svg class="btn-icon" viewBox="0 0 20 20" fill="currentColor">
                <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
              </svg>
              {{ t.loraModal.scanFolderBtn }}
            </button>

            <button class="primary-btn" @click="openAddModal">
              <svg class="btn-icon" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
              </svg>
              {{ t.loraModal.addLoraBtn }}
            </button>
          </div>
        </div>

        <!-- Inline Add / Edit Drawer Form -->
        <div v-if="isEditing" class="edit-card">
          <div class="edit-header">
            <h3>{{ editForm.id > 0 ? t.loraModal.editTitle : t.loraModal.addTitle }}</h3>
            <button class="text-btn" @click="cancelEdit">{{ t.loraModal.cancel }}</button>
          </div>

          <div class="form-grid">
            <div class="form-group">
              <label>{{ t.loraModal.loraName }} *</label>
              <input v-model="editForm.name" type="text" class="input-field" placeholder="e.g. anime_outline_v1" />
            </div>

            <div class="form-group">
              <label>{{ t.loraModal.hash }} (AutoV2 / SHA256)</label>
              <input v-model="editForm.hash" type="text" class="input-field" placeholder="e.g. 9b42e7" />
            </div>

            <div class="form-group full-width">
              <label>{{ t.loraModal.triggerWords }} ({{ t.loraModal.commaSeparated }})</label>
              <input
                v-model="editForm.trigger_words"
                type="text"
                class="input-field"
                placeholder="e.g. masterpiece, sharp lineart, high contrast"
              />
            </div>

            <div class="form-group">
              <label>{{ t.loraModal.defaultWeight }}: {{ editForm.weight_default }}</label>
              <input
                v-model.number="editForm.weight_default"
                type="range"
                min="0.1"
                max="2.0"
                step="0.05"
                class="range-slider"
              />
            </div>

            <div class="form-group full-width">
              <label>{{ t.loraModal.description }}</label>
              <textarea
                v-model="editForm.description"
                rows="2"
                class="textarea-field"
                placeholder="Optional notes or Civitai tags..."
              ></textarea>
            </div>
          </div>

          <div class="form-actions">
            <button class="secondary-btn" @click="cancelEdit">{{ t.loraModal.cancel }}</button>
            <button class="primary-btn" @click="saveEdit">{{ t.loraModal.save }}</button>
          </div>
        </div>

        <!-- LoRA List View -->
        <div v-if="loading" class="empty-state">
          <div class="spinner"></div>
          <p>{{ t.loraModal.loading }}</p>
        </div>

        <div v-else-if="filteredLoras.length === 0" class="empty-state">
          <p>{{ searchQuery ? t.loraModal.noMatches : t.loraModal.emptyLibrary }}</p>
          <button v-if="!searchQuery" class="primary-btn" style="margin-top: 12px;" @click="openAddModal">
            {{ t.loraModal.addFirstLora }}
          </button>
        </div>

        <div v-else class="lora-grid">
          <div v-for="lora in filteredLoras" :key="lora.id" class="lora-card">
            <div class="lora-card-header">
              <div class="lora-title-row">
                <span class="lora-badge">LoRA</span>
                <span class="lora-name" :title="lora.name">{{ lora.name }}</span>
                <span class="lora-weight-tag">×{{ lora.weight_default }}</span>
              </div>

              <div class="card-menu">
                <button class="icon-btn" :title="t.loraModal.editTooltip" @click="openEditModal(lora)">
                  <svg viewBox="0 0 20 20" fill="currentColor">
                    <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                  </svg>
                </button>
                <button class="icon-btn danger" :title="t.loraModal.deleteTooltip" @click="handleDelete(lora)">
                  <svg viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                  </svg>
                </button>
              </div>
            </div>

            <!-- Hash & Notes -->
            <div v-if="lora.hash || lora.description" class="lora-meta-row">
              <span v-if="lora.hash" class="hash-chip">#{{ lora.hash }}</span>
              <span v-if="lora.description" class="desc-text" :title="lora.description">
                {{ lora.description }}
              </span>
            </div>

            <!-- Trigger Words Chips -->
            <div class="triggers-container">
              <div class="triggers-label">{{ t.loraModal.triggerWords }}:</div>
              <div v-if="lora.trigger_words.length === 0" class="no-triggers">
                {{ t.loraModal.noTriggersRegistered }}
              </div>
              <div v-else class="chips-list">
                <span
                  v-for="(word, idx) in lora.trigger_words"
                  :key="idx"
                  class="trigger-chip"
                  :title="t.loraModal.clickToCopy"
                  @click="copyText(word, word)"
                >
                  {{ word }}
                  <span class="copy-icon">📋</span>
                </span>
              </div>
            </div>

            <!-- Card Action Footer -->
            <div class="card-footer">
              <button
                v-if="lora.trigger_words.length > 0"
                class="footer-action-btn"
                @click="copyAllTriggers(lora)"
              >
                {{ t.loraModal.copyAllTriggers }}
              </button>
              <button class="footer-action-btn highlight" @click="injectPrompt(lora)">
                {{ t.loraModal.injectPrompt }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
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
  max-width: 820px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
  color: var(--text-primary, #eee);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, #333);
}

.modal-title-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
}

.modal-icon {
  width: 22px;
  height: 22px;
  color: #ec4899;
}

.modal-header h2 {
  font-size: 1.15rem;
  font-weight: 600;
  margin: 0;
}

.badge-count,
.badge-triggers {
  font-size: 0.75rem;
  padding: 2px 8px;
  border-radius: 12px;
  font-weight: 500;
}

.badge-count {
  background: rgba(236, 72, 153, 0.15);
  color: #f472b6;
  border: 1px solid rgba(236, 72, 153, 0.3);
}

.badge-triggers {
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
  border: 1px solid rgba(59, 130, 246, 0.3);
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
  padding: 18px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.alert-msg {
  padding: 10px 14px;
  border-radius: 6px;
  font-size: 0.85rem;
}

.alert-msg.success {
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #34d399;
}

.alert-msg.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.toast-feedback {
  background: #ec4899;
  color: #fff;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 500;
  text-align: center;
  box-shadow: 0 4px 12px rgba(236, 72, 153, 0.4);
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 220px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: #888;
}

.search-input {
  width: 100%;
  box-sizing: border-box;
  background: #141418;
  border: 1px solid var(--border-color, #333);
  border-radius: 6px;
  padding: 7px 10px 7px 32px;
  font-size: 0.85rem;
  color: #fff;
}

.search-input:focus {
  outline: none;
  border-color: #ec4899;
}

.btn-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.primary-btn,
.secondary-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.primary-btn {
  background: #ec4899;
  color: #fff;
}

.primary-btn:hover {
  background: #db2777;
}

.secondary-btn {
  background: #282832;
  border-color: #3e3e4a;
  color: #ddd;
}

.secondary-btn:hover:not(:disabled) {
  background: #343442;
  color: #fff;
}

.secondary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-icon {
  width: 16px;
  height: 16px;
}

/* Edit Drawer Card */
.edit-card {
  background: #252530;
  border: 1px solid #3f3f50;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.edit-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #333344;
  padding-bottom: 8px;
}

.edit-header h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: #f472b6;
}

.text-btn {
  background: none;
  border: none;
  color: #aaa;
  cursor: pointer;
  font-size: 0.85rem;
}

.text-btn:hover {
  color: #fff;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group.full-width {
  grid-column: span 2;
}

.form-group label {
  font-size: 0.78rem;
  color: #aaa;
}

.input-field,
.textarea-field {
  background: #181820;
  border: 1px solid #3a3a4a;
  border-radius: 6px;
  padding: 7px 10px;
  font-size: 0.85rem;
  color: #fff;
}

.input-field:focus,
.textarea-field:focus {
  outline: none;
  border-color: #ec4899;
}

.range-slider {
  accent-color: #ec4899;
  cursor: pointer;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

/* LoRA Grid */
.lora-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.lora-card {
  background: #23232c;
  border: 1px solid #353544;
  border-radius: 8px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: border-color 0.15s ease, background 0.15s ease;
}

.lora-card:hover {
  border-color: #4f4f66;
  background: #272732;
}

.lora-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.lora-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.lora-badge {
  background: rgba(236, 72, 153, 0.2);
  color: #f472b6;
  font-size: 0.7rem;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid rgba(236, 72, 153, 0.4);
}

.lora-name {
  font-weight: 600;
  font-size: 0.95rem;
  color: #fff;
}

.lora-weight-tag {
  font-size: 0.75rem;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.12);
  padding: 1px 6px;
  border-radius: 4px;
}

.card-menu {
  display: flex;
  gap: 4px;
}

.icon-btn {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.icon-btn svg {
  width: 15px;
  height: 15px;
}

.icon-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.08);
}

.icon-btn.danger:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.15);
}

.lora-meta-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.78rem;
}

.hash-chip {
  background: #1c1c24;
  border: 1px solid #333342;
  color: #999;
  padding: 1px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.desc-text {
  color: #aaa;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 450px;
}

/* Triggers Container */
.triggers-container {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.triggers-label {
  font-size: 0.75rem;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.no-triggers {
  font-size: 0.8rem;
  color: #666;
  font-style: italic;
}

.chips-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.trigger-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: #181822;
  border: 1px solid #38384a;
  color: #e2e8f0;
  padding: 3px 8px;
  border-radius: 5px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.trigger-chip:hover {
  background: #252538;
  border-color: #ec4899;
  color: #f472b6;
}

.copy-icon {
  font-size: 0.7rem;
  opacity: 0.6;
}

.trigger-chip:hover .copy-icon {
  opacity: 1;
}

.card-footer {
  display: flex;
  gap: 8px;
  margin-top: 2px;
  padding-top: 8px;
  border-top: 1px solid #2e2e3c;
}

.footer-action-btn {
  background: #1a1a24;
  border: 1px solid #333342;
  color: #bbb;
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 0.78rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.footer-action-btn:hover {
  background: #282836;
  color: #fff;
  border-color: #4a4a5e;
}

.footer-action-btn.highlight {
  background: rgba(236, 72, 153, 0.12);
  border-color: rgba(236, 72, 153, 0.3);
  color: #f472b6;
}

.footer-action-btn.highlight:hover {
  background: rgba(236, 72, 153, 0.22);
  border-color: #ec4899;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: #888;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 2px solid #333;
  border-top-color: #ec4899;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
