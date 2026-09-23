<script setup lang="ts">
import { computed } from "vue";
import type { Folder, NavTarget, SubdirectoryEntry } from "../types";

defineOptions({
  name: "FolderTreeNode",
});

const props = defineProps<{
  folder: Folder;
  entry: SubdirectoryEntry;
  depth: number;
  activeTarget: NavTarget;
  expandedPaths: Set<string>;
  subdirectories: Record<string, SubdirectoryEntry[]>;
  loadingPaths: Set<string>;
}>();

const emit = defineEmits<{
  (e: "toggleExpand", folder: Folder, path: string): void;
  (e: "selectSubfolder", folder: Folder, path: string): void;
}>();

const isTargetActive = computed(() => {
  return (
    props.activeTarget.type === "folder" &&
    props.activeTarget.folder.id === props.folder.id &&
    props.activeTarget.subfolderPath === props.entry.path
  );
});

const isExpanded = computed(() => props.expandedPaths.has(props.entry.path));
const isLoading = computed(() => props.loadingPaths.has(props.entry.path));
const children = computed(() => props.subdirectories[props.entry.path] ?? []);
</script>

<template>
  <li class="subfolder-node" :class="{ active: isTargetActive }">
    <div
      class="subfolder-header"
      :style="{ paddingLeft: `${18 + depth * 14}px` }"
      :title="entry.path"
      @click="emit('selectSubfolder', folder, entry.path)"
    >
      <button
        v-if="entry.has_children"
        type="button"
        class="tree-arrow-btn"
        :class="{ expanded: isExpanded }"
        :title="isExpanded ? 'Collapse' : 'Expand'"
        @click.stop="emit('toggleExpand', folder, entry.path)"
      >
        <span v-if="isLoading" class="tree-loading-dot">…</span>
        <span v-else>{{ isExpanded ? '▼' : '▶' }}</span>
      </button>
      <span v-else class="tree-arrow-spacer"></span>

      <span class="item-icon">
        {{ isExpanded ? '📂' : '📁' }}
      </span>

      <span class="item-label truncate">{{ entry.name }}</span>

      <span v-if="entry.file_count > 0" class="item-badge">{{ entry.file_count }}</span>
    </div>

    <!-- Recursive Subfolder Children -->
    <ul v-if="isExpanded && children.length > 0" class="subfolder-tree-list">
      <FolderTreeNode
        v-for="child in children"
        :key="child.path"
        :folder="folder"
        :entry="child"
        :depth="depth + 1"
        :active-target="activeTarget"
        :expanded-paths="expandedPaths"
        :subdirectories="subdirectories"
        :loading-paths="loadingPaths"
        @toggle-expand="(f, p) => emit('toggleExpand', f, p)"
        @select-subfolder="(f, p) => emit('selectSubfolder', f, p)"
      />
    </ul>
  </li>
</template>

<style scoped>
.subfolder-node {
  display: flex;
  flex-direction: column;
  width: 100%;
}

.subfolder-header {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding-top: 4px;
  padding-bottom: 4px;
  padding-right: 8px;
  border-radius: 6px;
  font-size: 0.8rem;
  color: #94a3b8;
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
  box-sizing: border-box;
}

.subfolder-header:hover {
  background: var(--color-bg-hover, rgba(255, 255, 255, 0.05));
  color: var(--color-text-primary, #f8fafc);
}

.subfolder-node.active > .subfolder-header {
  background: var(--color-bg-active, rgba(139, 92, 246, 0.2));
  color: var(--color-text-primary, #f8fafc);
  font-weight: 600;
}

.tree-arrow-btn {
  background: transparent;
  border: none;
  color: #64748b;
  font-size: 0.65rem;
  width: 16px;
  height: 16px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.12s ease;
  flex-shrink: 0;
}

.tree-arrow-btn:hover {
  color: #f8fafc;
  background: rgba(255, 255, 255, 0.08);
}

.tree-arrow-spacer {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.tree-loading-dot {
  font-size: 0.65rem;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.9rem;
  flex-shrink: 0;
}

.item-label {
  flex: 1;
  min-width: 0;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-badge {
  font-size: 0.68rem;
  padding: 1px 6px;
  border-radius: 9999px;
  background: rgba(255, 255, 255, 0.08);
  color: #94a3b8;
  font-weight: 500;
  flex-shrink: 0;
}

.subfolder-tree-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
</style>
