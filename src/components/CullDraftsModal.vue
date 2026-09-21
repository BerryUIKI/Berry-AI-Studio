<script setup lang="ts">
import { ref, watch } from "vue";
import type { ImageFile } from "../types";
import { t } from "../i18n";
import { assetUrl, getFileName } from "../utils/image";

const props = defineProps<{
  open: boolean;
  heroes: ImageFile[];
  drafts: ImageFile[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "confirm", draftPaths: string[]): void;
}>();

const isSubmitting = ref(false);

function onConfirm() {
  if (props.drafts.length === 0) {
    emit("close");
    return;
  }
  isSubmitting.value = true;
  emit("confirm", props.drafts.map((d) => d.path));
  isSubmitting.value = false;
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.open) {
    emit("close");
  }
}

watch(
  () => props.open,
  (val) => {
    if (val) {
      window.addEventListener("keydown", handleKeydown);
    } else {
      window.removeEventListener("keydown", handleKeydown);
    }
  },
  { immediate: true },
);
</script>

<template>
  <div v-if="open" class="modal-overlay" @click.self="emit('close')">
    <div class="cull-modal" role="dialog" aria-modal="true" :aria-label="t.stack.cullDrafts">
      <div class="modal-header">
        <div class="header-icon">🗑️</div>
        <div class="header-text">
          <h3 class="modal-title">{{ t.stack.cullDrafts }}</h3>
          <p class="modal-desc">
            {{ t.stack.cullModalDesc.replace('{count}', String(drafts.length)) }}
          </p>
        </div>
        <button type="button" class="close-btn" @click="emit('close')" aria-label="Close">✕</button>
      </div>

      <div class="modal-body">
        <!-- Heroes section -->
        <div v-if="heroes.length > 0" class="cull-section">
          <div class="section-badge hero-badge">
            ⭐ {{ t.stack.cullHeroesKept.replace('{count}', String(heroes.length)) }}
          </div>
          <div class="thumbnail-strip">
            <div
              v-for="file in heroes.slice(0, 8)"
              :key="file.path"
              class="thumb-item hero-item"
              :title="getFileName(file.path)"
            >
              <img :src="assetUrl(file.path)" :alt="getFileName(file.path)" loading="lazy" />
              <span v-if="file.rating" class="item-rating">★ {{ file.rating }}</span>
            </div>
            <div v-if="heroes.length > 8" class="more-item">
              +{{ heroes.length - 8 }}
            </div>
          </div>
        </div>

        <!-- Drafts section -->
        <div class="cull-section">
          <div class="section-badge draft-badge">
            🗑️ {{ t.stack.cullDraftsToTrash.replace('{count}', String(drafts.length)) }}
          </div>
          <div class="thumbnail-strip">
            <div
              v-for="file in drafts.slice(0, 16)"
              :key="file.path"
              class="thumb-item draft-item"
              :title="getFileName(file.path)"
            >
              <img :src="assetUrl(file.path)" :alt="getFileName(file.path)" loading="lazy" />
              <span v-if="file.rating" class="item-rating draft-rating">★ {{ file.rating }}</span>
            </div>
            <div v-if="drafts.length > 16" class="more-item">
              +{{ drafts.length - 16 }}
            </div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button type="button" class="btn-cancel" @click="emit('close')">
          {{ t.stack.cullCancel }}
        </button>
        <button
          type="button"
          class="btn-danger"
          :disabled="isSubmitting || drafts.length === 0"
          @click="onConfirm"
        >
          🗑️ {{ t.stack.cullConfirm }} ({{ drafts.length }})
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
  z-index: 1100;
  padding: 16px;
}

.cull-modal {
  width: 520px;
  max-width: 95vw;
  max-height: 85vh;
  background: var(--color-bg-primary, #1e2025);
  border: 1px solid var(--border-color-strong, rgba(255, 255, 255, 0.16));
  border-radius: 12px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
}

.header-icon {
  font-size: 1.5rem;
  line-height: 1;
}

.header-text {
  flex: 1;
}

.modal-title {
  margin: 0 0 4px 0;
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--color-text-primary, #fff);
}

.modal-desc {
  margin: 0;
  font-size: 0.8rem;
  color: var(--color-text-muted, #94a3b8);
  line-height: 1.4;
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--color-text-muted, #94a3b8);
  cursor: pointer;
  font-size: 1rem;
  padding: 4px;
}

.close-btn:hover {
  color: #fff;
}

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.cull-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-badge {
  font-size: 0.76rem;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 4px;
  align-self: flex-start;
}

.hero-badge {
  background: rgba(34, 197, 94, 0.15);
  color: #4ade80;
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.draft-badge {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.thumbnail-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  max-height: 160px;
  overflow-y: auto;
}

.thumb-item {
  position: relative;
  width: 54px;
  height: 54px;
  border-radius: 6px;
  overflow: hidden;
  border: 1.5px solid rgba(255, 255, 255, 0.1);
  background: #000;
}

.thumb-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.hero-item {
  border-color: #22c55e;
}

.draft-item {
  border-color: rgba(239, 68, 68, 0.5);
  opacity: 0.8;
}

.item-rating {
  position: absolute;
  bottom: 2px;
  right: 2px;
  font-size: 0.6rem;
  font-weight: bold;
  background: rgba(0, 0, 0, 0.75);
  color: #facc15;
  padding: 0 3px;
  border-radius: 2px;
}

.more-item {
  width: 54px;
  height: 54px;
  border-radius: 6px;
  border: 1.5px dashed rgba(255, 255, 255, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  color: var(--color-text-muted, #94a3b8);
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 20px;
  background: var(--color-bg-secondary, #25282e);
  border-top: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
}

.btn-cancel {
  background: transparent;
  border: 1px solid var(--border-color-strong, rgba(255, 255, 255, 0.2));
  color: var(--color-text-primary, #e2e8f0);
  padding: 6px 14px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.8rem;
}

.btn-cancel:hover {
  background: rgba(255, 255, 255, 0.08);
}

.btn-danger {
  background: #dc2626;
  border: 1px solid #ef4444;
  color: #fff;
  padding: 6px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.8rem;
  font-weight: 600;
  transition: all 0.15s ease;
}

.btn-danger:hover:not(:disabled) {
  background: #b91c1c;
}

.btn-danger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
