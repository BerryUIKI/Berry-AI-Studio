<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import type { Tag } from "../types";

const props = defineProps<{
  open: boolean;
  fileIds?: number[]; // If provided, allows toggling tags on these files
}>();

const emit = defineEmits<{
  (e: "update:open", val: boolean): void;
  (e: "tagsChanged"): void;
}>();

const tags = ref<Tag[]>([]);
const loading = ref(false);
const error = ref("");

const PRESET_COLORS = [
  "#3b82f6", // Blue
  "#10b981", // Emerald
  "#ec4899", // Pink
  "#f59e0b", // Amber
  "#8b5cf6", // Purple
  "#ef4444", // Red
  "#06b6d4", // Cyan
  "#6b7280", // Gray
];

const newTagName = ref("");
const selectedColor = ref(PRESET_COLORS[0]);
const isCreating = ref(false);

onMounted(async () => {
  await loadTags();
});

async function loadTags() {
  loading.value = true;
  error.value = "";
  try {
    tags.value = await invoke<Tag[]>("list_tags");
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function onCreateTag() {
  const name = newTagName.value.trim();
  if (!name) return;
  try {
    const created = await invoke<Tag>("create_tag", {
      name,
      color: selectedColor.value,
    });
    newTagName.value = "";
    isCreating.value = false;
    await loadTags();
    emit("tagsChanged");

    // If tagging files, immediately attach newly created tag
    if (props.fileIds && props.fileIds.length > 0) {
      await onApplyTag(created);
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function onDeleteTag(tagId: number) {
  if (!confirm(t.value.tagsModal.deleteConfirm)) {
    return;
  }
  try {
    await invoke("delete_tag", { id: tagId });
    await loadTags();
    emit("tagsChanged");
  } catch (e) {
    error.value = String(e);
  }
}

async function onApplyTag(tag: Tag) {
  if (!props.fileIds || props.fileIds.length === 0) {
    return;
  }
  try {
    await invoke("tag_files", {
      fileIds: props.fileIds,
      tagId: tag.id,
    });
    emit("tagsChanged");
    emit("update:open", false);
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div v-if="open" class="modal-backdrop" @click="emit('update:open', false)" v-dialog="() => emit('update:open', false)">
    <div class="modal-dialog" role="dialog" aria-modal="true" @click.stop>
      <div class="modal-header">
        <h2>
          {{ fileIds && fileIds.length > 0 ? `${t.batch.tag} (${fileIds.length})` : t.tagsModal.title }}
        </h2>
        <button
          type="button"
          class="btn-close"
          aria-label="Close"
          @click="emit('update:open', false)"
        >
          ✕
        </button>
      </div>

      <div v-if="error" class="modal-error">
        {{ error }}
      </div>

      <div class="modal-body">
        <!-- New Tag Form -->
        <div v-if="isCreating" class="create-form">
          <h3>{{ t.tagsModal.createBtn }}</h3>
          <div class="form-group">
            <label for="tag-name">{{ t.tagsModal.createPlaceholder }}</label>
            <input
              id="tag-name"
              v-model="newTagName"
              type="text"
              :placeholder="t.tagsModal.createPlaceholder"
              autofocus
              @keydown.enter.prevent="onCreateTag"
            />
          </div>

          <div class="form-group">
            <label>Color</label>
            <div class="color-presets">
              <button
                v-for="color in PRESET_COLORS"
                :key="color"
                type="button"
                class="color-dot"
                :class="{ active: selectedColor === color }"
                :style="{ backgroundColor: color }"
                @click="selectedColor = color"
              />
            </div>
          </div>

          <div class="form-actions">
            <button type="button" class="btn-cancel" @click="isCreating = false">
              {{ t.fileOpModal.cancel }}
            </button>
            <button
              type="button"
              class="btn-primary"
              :disabled="!newTagName.trim()"
              @click="onCreateTag"
            >
              {{ t.tagsModal.createBtn }}
            </button>
          </div>
        </div>

        <div v-else class="header-action">
          <button type="button" class="btn-create" @click="isCreating = true">
            + {{ t.nav.newTag }}
          </button>
        </div>

        <!-- Tag list -->
        <div v-if="loading" class="tag-loading">Loading...</div>
        <div v-else-if="tags.length === 0" class="tag-empty">
          {{ t.tagsModal.noTags }}
        </div>
        <div v-else class="tag-chips-container">
          <div
            v-for="tag in tags"
            :key="tag.id"
            class="tag-chip"
            :class="{ 'selectable': fileIds && fileIds.length > 0 }"
            :style="{ borderColor: tag.color || '#3b82f6' }"
            @click="onApplyTag(tag)"
          >
            <span
              class="tag-dot"
              :style="{ backgroundColor: tag.color || '#3b82f6' }"
            />
            <span class="tag-label">{{ tag.name }}</span>
            <button
              v-if="!fileIds || fileIds.length === 0"
              type="button"
              class="tag-delete-btn"
              title="Delete Tag"
              @click.stop="onDeleteTag(tag.id)"
            >
              ✕
            </button>
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
  background: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(4px);
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-dialog {
  background: #1e1e1e;
  color: #fff;
  width: 90%;
  max-width: 440px;
  max-height: 80vh;
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
    transform: scale(0.95);
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
  padding: 1rem 1.2rem;
  border-bottom: 1px solid rgba(128, 128, 128, 0.2);
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
  padding: 0.5rem 1rem;
  font-size: 0.85em;
  border-bottom: 1px solid rgba(239, 68, 68, 0.3);
}

.modal-body {
  padding: 1rem 1.2rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.header-action {
  display: flex;
  justify-content: flex-end;
}

.btn-create {
  background: #2f6fed;
  color: #fff;
  border: none;
  border-radius: 6px;
  padding: 0.4rem 0.85rem;
  font: inherit;
  font-size: 0.85em;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.btn-create:hover {
  opacity: 0.9;
}

.create-form {
  background: rgba(128, 128, 128, 0.08);
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  padding: 0.8rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.create-form h3 {
  margin: 0 0 0.3rem 0;
  font-size: 0.95em;
  font-weight: 600;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.form-group label {
  font-size: 0.78em;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.form-group input {
  padding: 0.45rem 0.6rem;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: rgba(0, 0, 0, 0.15);
  color: inherit;
  font: inherit;
  font-size: 0.88em;
}

@media (prefers-color-scheme: light) {
  .form-group input {
    background: #fff;
  }
}

.color-presets {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding-top: 0.2rem;
}

.color-dot {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  transition: transform 0.15s ease;
}

.color-dot:hover {
  transform: scale(1.15);
}

.color-dot.active {
  border-color: #fff;
  transform: scale(1.2);
  box-shadow: 0 0 6px rgba(0, 0, 0, 0.5);
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-top: 0.4rem;
}

.btn-cancel,
.btn-primary {
  padding: 0.35rem 0.75rem;
  border-radius: 6px;
  font: inherit;
  font-size: 0.82em;
  cursor: pointer;
}

.btn-cancel {
  background: transparent;
  border: 1px solid rgba(128, 128, 128, 0.3);
  color: inherit;
}

.btn-primary {
  background: #2f6fed;
  color: #fff;
  border: none;
  font-weight: 500;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.tag-chips-container {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.tag-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.3rem 0.65rem;
  border-radius: 999px;
  background: rgba(128, 128, 128, 0.1);
  border: 1px solid transparent;
  font-size: 0.85em;
  transition: all 0.15s ease;
}

.tag-chip.selectable {
  cursor: pointer;
}

.tag-chip.selectable:hover {
  background: rgba(128, 128, 128, 0.25);
  transform: translateY(-1px);
}

.tag-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.tag-label {
  font-weight: 500;
}

.tag-delete-btn {
  background: transparent;
  border: none;
  color: #888;
  font-size: 0.8em;
  padding: 0 0.15rem;
  cursor: pointer;
  border-radius: 50%;
  line-height: 1;
}

.tag-delete-btn:hover {
  color: #ef4444;
}

.tag-empty,
.tag-loading {
  text-align: center;
  color: #888;
  font-size: 0.88em;
  padding: 1.5rem 0;
}
</style>
