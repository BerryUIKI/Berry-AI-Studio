<script setup lang="ts">
import type { FileSortField, SortDirection } from "../types";
import { t } from "../i18n";

const props = defineProps<{
  sortField: FileSortField;
  sortDirection: SortDirection;
}>();

const emit = defineEmits<{
  (e: "update:sortField", value: FileSortField): void;
  (e: "update:sortDirection", value: SortDirection): void;
  (e: "change"): void;
}>();

function onFieldChange(e: Event) {
  const target = e.target as HTMLSelectElement;
  emit("update:sortField", target.value as FileSortField);
  emit("change");
}

function toggleDirection() {
  const next = props.sortDirection === "desc" ? "asc" : "desc";
  emit("update:sortDirection", next);
  emit("change");
}
</script>

<template>
  <div class="sort-bar-eagle">
    <select :value="sortField" class="sort-select" :title="t.sort.title" @change="onFieldChange">
      <option value="modified_at">{{ t.sort.modified }}</option>
      <option value="path">{{ t.sort.name }}</option>
      <option value="size_bytes">{{ t.sort.size }}</option>
      <option value="rating">{{ t.sort.rating }}</option>
      <option value="aesthetic_score">{{ t.sort.aesthetic }}</option>
    </select>

    <button
      type="button"
      class="direction-btn"
      :title="sortDirection === 'desc' ? t.sort.desc : t.sort.asc"
      @click="toggleDirection"
    >
      {{ sortDirection === "desc" ? "↓" : "↑" }}
    </button>
  </div>
</template>

<style scoped>
.sort-bar-eagle {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  flex-shrink: 0;
}

.sort-select {
  font: inherit;
  font-size: 0.76rem;
  height: 28px;
  padding: 0 6px;
  border-radius: 5px;
  border: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  cursor: pointer;
  outline: none;
  transition: all 0.12s ease;
}

.sort-select:hover {
  background: var(--color-bg-tertiary);
  border-color: var(--border-color-strong);
}

.sort-select:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 1px;
}

.sort-select option {
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

.direction-btn {
  font: inherit;
  font-size: 0.8rem;
  height: 28px;
  width: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  border: 1px solid var(--border-color);
  background: var(--color-bg-secondary);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.12s ease;
}

.direction-btn:hover {
  background: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.direction-btn:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: 1px;
}
</style>
