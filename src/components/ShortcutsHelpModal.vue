<script setup lang="ts">
import { t } from "../i18n";

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const shortcuts = [
  { key: "Space / Enter", desc: "Open preview / full inspector for selected image" },
  { key: "Esc", desc: "Close preview or modal, clear multi-selection" },
  { key: "← / → / ↑ / ↓", desc: "Navigate between images in grid or list" },
  { key: "Cmd/Ctrl + A", desc: "Select all images in current view" },
  { key: "/ or Cmd/Ctrl + F", desc: "Focus search bar" },
  { key: "1 - 5", desc: "Quickly rate selected image(s) 1 to 5 stars" },
  { key: "0", desc: "Clear rating on selected image(s)" },
  { key: "F", desc: "Toggle favorite on selected image(s)" },
  { key: "I", desc: "Toggle metadata inspector panel inside preview" },
  { key: "Delete / Backspace", desc: "Move selected image(s) to Trash" },
  { key: "?", desc: "Show this keyboard shortcuts guide" },
];
</script>

<template>
  <div v-if="show" class="modal-backdrop" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="modal-container">
      <div class="modal-header">
        <div class="header-left">
          <span class="header-icon">⌨️</span>
          <h2>{{ t.shortcutsModal.title }}</h2>
        </div>
        <button class="close-btn" @click="emit('close')" :title="t.shortcutsModal.close">✕</button>
      </div>

      <div class="modal-body">
        <table class="shortcuts-table">
          <thead>
            <tr>
              <th>{{ t.shortcutsModal.shortcut }}</th>
              <th>{{ t.shortcutsModal.action }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(item, idx) in shortcuts" :key="idx">
              <td class="key-cell">
                <kbd class="key-badge">{{ item.key }}</kbd>
              </td>
              <td class="desc-cell">{{ item.desc }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" @click="emit('close')">
          {{ t.shortcutsModal.close }}
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
  max-width: 580px;
  max-height: 85vh;
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
  padding: 16px 20px;
  overflow-y: auto;
}

.shortcuts-table {
  width: 100%;
  border-collapse: collapse;
}

.shortcuts-table th {
  text-align: left;
  padding: 8px 12px;
  font-size: 0.8rem;
  color: var(--text-secondary, #888);
  border-bottom: 1px solid var(--border-color, #333);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.shortcuts-table td {
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 0.88rem;
}

.key-cell {
  width: 35%;
  white-space: nowrap;
}

.key-badge {
  display: inline-block;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  padding: 2px 8px;
  font-family: inherit;
  font-size: 0.8rem;
  font-weight: 600;
  color: #60a5fa;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}

.desc-cell {
  color: #ccc;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
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
</style>
