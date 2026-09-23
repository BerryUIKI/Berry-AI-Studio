<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import type { Folder, ImageFile } from "../types";

const props = defineProps<{
  open: boolean;
  mode: "move" | "copy" | "trash";
  files: ImageFile[];
  folders: Folder[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "completed"): void;
}>();

const selectedFolderId = ref<number | null>(props.folders[0]?.id ?? null);
const isProcessing = ref(false);
const errorMessage = ref<string | null>(null);

const modalTitle = computed(() => {
  switch (props.mode) {
    case "move":
      return `${t.value.fileOpModal.moveTitle} (${props.files.length})`;
    case "copy":
      return `${t.value.fileOpModal.copyTitle} (${props.files.length})`;
    case "trash":
      return `${t.value.fileOpModal.trashTitle} (${props.files.length})`;
  }
});

async function handleConfirm() {
  isProcessing.value = true;
  errorMessage.value = null;

  try {
    const filePaths = props.files.map((f) => f.path);
    if (props.mode === "move") {
      if (selectedFolderId.value === null) return;
      await invoke("move_files", {
        filePaths,
        targetFolderId: selectedFolderId.value,
      });
    } else if (props.mode === "copy") {
      if (selectedFolderId.value === null) return;
      await invoke("copy_files", {
        filePaths,
        targetFolderId: selectedFolderId.value,
      });
    } else if (props.mode === "trash") {
      await invoke("trash_files", {
        filePaths,
      });
    }
    emit("completed");
    emit("close");
  } catch (err: any) {
    errorMessage.value = String(err);
  } finally {
    isProcessing.value = false;
  }
}
</script>

<template>
  <div v-if="open" class="modal-backdrop" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="modal-container">
      <div class="modal-header">
        <div class="modal-title-wrap">
          <span class="modal-icon">{{ mode === 'trash' ? '🗑' : mode === 'move' ? '📂' : '📄' }}</span>
          <h2>{{ modalTitle }}</h2>
        </div>
        <button class="close-btn" @click="emit('close')" :title="t.fileOpModal.cancel">✕</button>
      </div>

      <div class="modal-body">
        <div v-if="errorMessage" class="error-banner">
          {{ errorMessage }}
        </div>

        <div v-if="mode === 'trash'" class="trash-warning">
          <p>{{ t.fileOpModal.trashWarning }}</p>
          <p class="trash-subtext">{{ t.fileOpModal.trashSubtext }}</p>
        </div>

        <div v-else class="folder-select-section">
          <label class="section-label">{{ t.fileOpModal.selectDest }}</label>
          <div class="folder-options">
            <div
              v-for="folder in folders"
              :key="folder.id"
              class="folder-option"
              :class="{ selected: selectedFolderId === folder.id }"
              @click="selectedFolderId = folder.id"
            >
              <div class="folder-radio">
                <input
                  type="radio"
                  :value="folder.id"
                  :checked="selectedFolderId === folder.id"
                  name="destination-folder"
                />
              </div>
              <div class="folder-text">
                <div class="folder-name">{{ folder.path.split(/[\\/]/).pop() || folder.path }}</div>
                <div class="folder-path" :title="folder.path">{{ folder.path }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Files Preview List -->
        <div class="file-list-preview">
          <div class="preview-header">{{ t.view.files }} ({{ files.length }}):</div>
          <div class="file-chips">
            <span v-for="f in files.slice(0, 10)" :key="f.path" class="file-chip" :title="f.path">
              {{ f.path.split(/[\\/]/).pop() }}
            </span>
            <span v-if="files.length > 10" class="file-chip more-chip">
              +{{ files.length - 10 }} more
            </span>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" :disabled="isProcessing" @click="emit('close')">
          {{ t.fileOpModal.cancel }}
        </button>
        <button
          type="button"
          class="btn"
          :class="mode === 'trash' ? 'btn-danger' : 'btn-primary'"
          :disabled="isProcessing || (mode !== 'trash' && selectedFolderId === null)"
          @click="handleConfirm"
        >
          {{ isProcessing ? t.fileOpModal.processing : mode === 'trash' ? t.fileOpModal.trashBtn : mode === 'move' ? t.fileOpModal.moveBtn : t.fileOpModal.copyBtn }}
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
  z-index: 1300;
}

.modal-container {
  background: var(--bg-surface, #1e1e24);
  border: 1px solid var(--border-color, #333);
  border-radius: 12px;
  width: 90%;
  max-width: 540px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.4);
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
  font-size: 1.25rem;
}

.modal-header h2 {
  font-size: 1.1rem;
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
  padding: 18px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.error-banner {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.4);
  color: #f87171;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
}

.trash-warning p {
  margin: 0 0 6px 0;
  font-size: 0.95rem;
  line-height: 1.4;
}

.trash-subtext {
  font-size: 0.82rem;
  color: var(--text-secondary, #aaa);
}

.folder-select-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-label {
  font-size: 0.85rem;
  color: var(--text-secondary, #aaa);
  font-weight: 500;
}

.folder-options {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 200px;
  overflow-y: auto;
}

.folder-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: var(--bg-card, #25252d);
  border: 1px solid var(--border-color, #3a3a46);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.folder-option:hover {
  border-color: #3b82f6;
}

.folder-option.selected {
  background: rgba(59, 130, 246, 0.15);
  border-color: #3b82f6;
}

.folder-text {
  min-width: 0;
  flex: 1;
}

.folder-name {
  font-weight: 500;
  font-size: 0.9rem;
  color: #fff;
}

.folder-path {
  font-size: 0.75rem;
  color: var(--text-secondary, #888);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-list-preview {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.preview-header {
  font-size: 0.8rem;
  color: var(--text-secondary, #888);
}

.file-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  max-height: 90px;
  overflow-y: auto;
}

.file-chip {
  font-size: 0.75rem;
  background: rgba(255, 255, 255, 0.08);
  padding: 3px 7px;
  border-radius: 4px;
  color: #ccc;
  white-space: nowrap;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.more-chip {
  background: rgba(59, 130, 246, 0.2);
  color: #93c5fa;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 20px;
  border-top: 1px solid var(--border-color, #333);
}

.btn {
  padding: 7px 16px;
  border-radius: 6px;
  font-size: 0.88rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.08);
  color: #ccc;
}

.btn-secondary:hover {
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
  background: #ef4444;
  color: #fff;
}

.btn-danger:hover:not(:disabled) {
  background: #dc2626;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
