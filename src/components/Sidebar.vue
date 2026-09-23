<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import type { Album, Folder, LibraryCounts, NavTarget, ScanProgress, ScanStats, Tag } from "../types";

const props = defineProps<{
  folders: Folder[];
  counts?: LibraryCounts | null;
  albums?: Album[];
  albumCounts?: Record<number, number>;
  tags?: Tag[];
  activeTarget: NavTarget;
  progress: ScanProgress | null;
  collapsed?: boolean;
}>();

const emit = defineEmits<{
  folderAdded: [folder: Folder];
  removed: [folderId: number];
  scanned: [folderId: number];
  selectNav: [target: NavTarget];
  openAlbumModal: [];
  openTagModal: [];
  openPromptStats: [];
  openModelManager: [];
  openDbManager: [];
  openShortcutsHelp: [];
  openAddFolderModal: [];
  moveFilesToFolder: [payload: { filePaths: string[]; folderId: number }];
  addFilesToAlbum: [payload: { fileIds: number[]; albumId: number }];
  tagFiles: [payload: { fileIds: number[]; tagId: number }];
  toggleCollapse: [];
}>();

const addingFolder = ref(false);
const running = ref<{ id: number; action: "scan" | "rebuild" | "harvest" } | null>(null);
const error = ref("");

function displayPath(path: string): string {
  return path.replace(/^\\\\\?\\/, "");
}

function getFolderName(path: string): string {
  const clean = displayPath(path);
  const parts = clean.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] || clean;
}

function isBusy(id: number): boolean {
  return running.value?.id === id;
}

function pickFolder() {
  emit("openAddFolderModal");
}

async function harvest(folder: Folder, e: MouseEvent) {
  e.stopPropagation();
  error.value = "";
  running.value = { id: folder.id, action: "harvest" };
  try {
    await invoke<number>("harvest_pipeline_folder", { folderId: folder.id });
    emit("scanned", folder.id);
  } catch (e) {
    error.value = String(e);
  } finally {
    running.value = null;
  }
}

async function scan(folder: Folder, action: "scan" | "rebuild" = "scan") {
  error.value = "";
  running.value = { id: folder.id, action };
  try {
    await invoke<ScanStats>(
      action === "rebuild" ? "rebuild_metadata" : "scan_folder",
      { folderId: folder.id },
    );
    emit("scanned", folder.id);
  } catch (e) {
    error.value = String(e);
  } finally {
    running.value = null;
  }
}

async function removeFolder(folder: Folder, e: MouseEvent) {
  e.stopPropagation();
  error.value = "";
  try {
    await invoke("remove_folder", { folderId: folder.id });
    emit("removed", folder.id);
  } catch (e) {
    error.value = String(e);
  }
}

function isTargetActive(target: NavTarget): boolean {
  if (props.activeTarget.type !== target.type) return false;
  if (target.type === "folder" && props.activeTarget.type === "folder") {
    return props.activeTarget.folder.id === target.folder.id;
  }
  if (target.type === "album" && props.activeTarget.type === "album") {
    return props.activeTarget.album.id === target.album.id;
  }
  if (target.type === "tag" && props.activeTarget.type === "tag") {
    return props.activeTarget.tag.id === target.tag.id;
  }
  return true;
}

function onDropOnFolder(e: DragEvent, folder: Folder) {
  e.preventDefault();
  const data = e.dataTransfer?.getData("application/json");
  if (!data) return;
  try {
    const payload = JSON.parse(data);
    if (payload.file_paths && payload.file_paths.length > 0) {
      emit("moveFilesToFolder", {
        filePaths: payload.file_paths,
        folderId: folder.id,
      });
    }
  } catch (err) {
    console.error("Drop on folder parse error:", err);
  }
}

function onDropOnAlbum(e: DragEvent, album: Album) {
  e.preventDefault();
  const data = e.dataTransfer?.getData("application/json");
  if (!data) return;
  try {
    const payload = JSON.parse(data);
    if (payload.file_ids && payload.file_ids.length > 0) {
      emit("addFilesToAlbum", {
        fileIds: payload.file_ids,
        albumId: album.id,
      });
    }
  } catch (err) {
    console.error("Drop on album parse error:", err);
  }
}

function onDropOnTag(e: DragEvent, tag: Tag) {
  e.preventDefault();
  const data = e.dataTransfer?.getData("application/json");
  if (!data) return;
  try {
    const payload = JSON.parse(data);
    if (payload.file_ids && payload.file_ids.length > 0) {
      emit("tagFiles", {
        fileIds: payload.file_ids,
        tagId: tag.id,
      });
    }
  } catch (err) {
    console.error("Drop on tag parse error:", err);
  }
}
</script>

<template>
  <aside class="sidebar-eagle" :class="{ collapsed }">
    <div class="sidebar-scrollable">
      <!-- Section: Library -->
      <section class="nav-group">
        <div class="group-header">
          <span class="group-title">{{ t.nav.library }}</span>
        </div>
        <ul class="nav-list">
          <li
            class="nav-item"
            :class="{ active: isTargetActive({ type: 'all' }) }"
            @click="emit('selectNav', { type: 'all' })"
          >
            <span class="item-icon">
              <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
                <path d="M2 3a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V3zm1 0v10h10V3H3zm2 7.5l2-2.5 1.5 2 2.5-3.5 3 4H4l1-1.5z" />
              </svg>
            </span>
            <span class="item-label">{{ t.nav.allImages }}</span>
            <span v-if="counts" class="item-badge">{{ counts.total }}</span>
          </li>
          <li
            class="nav-item"
            :class="{ active: isTargetActive({ type: 'favorites' }) }"
            @click="emit('selectNav', { type: 'favorites' })"
          >
            <span class="item-icon star-icon">★</span>
            <span class="item-label">{{ t.nav.favorites }}</span>
          </li>
          <li
            class="nav-item"
            :class="{ active: isTargetActive({ type: 'nsfw' }) }"
            @click="emit('selectNav', { type: 'nsfw' })"
          >
            <span class="item-icon nsfw-icon">🔞</span>
            <span class="item-label">{{ t.nav.sensitive }}</span>
          </li>
        </ul>
      </section>

      <!-- Section: Folders -->
      <section class="nav-group">
        <div class="group-header">
          <span class="group-title">{{ t.nav.folders }}</span>
          <button
            type="button"
            class="group-action-btn"
            :disabled="addingFolder"
            :title="t.nav.scan"
            @click="pickFolder"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
              <path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/>
            </svg>
          </button>
        </div>
        <ul class="nav-list">
          <li
            v-for="folder in folders"
            :key="folder.id"
            class="nav-item folder-item"
            :class="{ active: isTargetActive({ type: 'folder', folder }) }"
            :title="displayPath(folder.path)"
            @click="emit('selectNav', { type: 'folder', folder })"
            @dragover.prevent
            @drop="onDropOnFolder($event, folder)"
          >
            <span class="item-icon">
              {{ folder.folder_type === 'pipeline' ? '⚡' : folder.folder_type === 'managed' ? '📦' : '📁' }}
            </span>
            <span class="item-label truncate">{{ getFolderName(folder.path) }}</span>
            <span v-if="counts?.folders" class="item-badge">{{ counts.folders[folder.id] ?? 0 }}</span>

            <div class="folder-actions" @click.stop>
              <button
                v-if="folder.folder_type === 'pipeline'"
                type="button"
                class="icon-btn harvest-btn"
                :disabled="isBusy(folder.id)"
                :title="t.nav.harvest || 'Harvest New Images'"
                @click="harvest(folder, $event)"
              >
                {{ isBusy(folder.id) ? '⏳' : '⚡' }}
              </button>
              <button
                type="button"
                class="icon-btn"
                :disabled="isBusy(folder.id)"
                :title="t.nav.scan"
                @click="scan(folder, 'scan')"
              >
                {{ isBusy(folder.id) ? '⏳' : '🔄' }}
              </button>
              <button
                type="button"
                class="icon-btn remove-btn"
                :title="t.nav.remove"
                @click="removeFolder(folder, $event)"
              >
                ✕
              </button>
            </div>
          </li>
          <li v-if="folders.length === 0" class="empty-hint">
            {{ t.nav.noFolders }}
          </li>
        </ul>
      </section>

      <!-- Section: Albums -->
      <section class="nav-group">
        <div class="group-header">
          <span class="group-title">{{ t.nav.albums }}</span>
          <button
            type="button"
            class="group-action-btn"
            :title="t.nav.newAlbum"
            @click="emit('openAlbumModal')"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
              <path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/>
            </svg>
          </button>
        </div>
        <ul class="nav-list">
          <li
            v-for="album in albums || []"
            :key="album.id"
            class="nav-item album-item"
            :class="{ active: isTargetActive({ type: 'album', album }) }"
            @click="emit('selectNav', { type: 'album', album })"
            @dragover.prevent
            @drop="onDropOnAlbum($event, album)"
          >
            <span class="item-icon">🗂️</span>
            <span class="item-label truncate">{{ album.name }}</span>
            <span class="item-badge">{{ albumCounts?.[album.id] ?? 0 }}</span>
          </li>
          <li v-if="!albums || albums.length === 0" class="empty-hint">
            {{ t.nav.noAlbums }}
          </li>
        </ul>
      </section>

      <!-- Section: Tags -->
      <section class="nav-group">
        <div class="group-header">
          <span class="group-title">{{ t.nav.tags }}</span>
          <button
            type="button"
            class="group-action-btn"
            :title="t.nav.newTag"
            @click="emit('openTagModal')"
          >
            <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
              <path d="M8 2a.75.75 0 0 1 .75.75v4.5h4.5a.75.75 0 0 1 0 1.5h-4.5v4.5a.75.75 0 0 1-1.5 0v-4.5h-4.5a.75.75 0 0 1 0-1.5h4.5v-4.5A.75.75 0 0 1 8 2z"/>
            </svg>
          </button>
        </div>
        <div class="tags-container">
          <div
            v-for="tag in tags || []"
            :key="tag.id"
            class="tag-chip-eagle"
            :class="{ active: isTargetActive({ type: 'tag', tag }) }"
            @click="emit('selectNav', { type: 'tag', tag })"
            @dragover.prevent
            @drop="onDropOnTag($event, tag)"
          >
            <span
              class="tag-dot"
              :style="{ backgroundColor: tag.color || '#8b5cf6' }"
            ></span>
            <span class="tag-name">{{ tag.name }}</span>
          </div>
          <p v-if="!tags || tags.length === 0" class="empty-hint">
            {{ t.nav.noTags }}
          </p>
        </div>
      </section>
    </div>

    <!-- Sidebar Bottom Action Bar (Eagle Style) -->
    <footer class="sidebar-footer">
      <div class="footer-tools">
        <button
          type="button"
          class="tool-btn"
          :title="t.search.insights"
          @click="emit('openPromptStats')"
        >
          <span class="tool-icon">📊</span>
          <span class="tool-text">{{ t.search.insights }}</span>
        </button>
        <button
          type="button"
          class="tool-btn"
          :title="t.search.models"
          @click="emit('openModelManager')"
        >
          <span class="tool-icon">🧠</span>
          <span class="tool-text">{{ t.search.models }}</span>
        </button>
        <button
          type="button"
          class="tool-btn"
          :title="t.search.database"
          @click="emit('openDbManager')"
        >
          <span class="tool-icon">🗄️</span>
          <span class="tool-text">{{ t.search.database }}</span>
        </button>
        <button
          type="button"
          class="tool-btn"
          :title="t.search.shortcuts"
          @click="emit('openShortcutsHelp')"
        >
          <span class="tool-icon">⌨️</span>
          <span class="tool-text">{{ t.search.shortcuts }}</span>
        </button>
      </div>
    </footer>
  </aside>
</template>

<style scoped>
.sidebar-eagle {
  width: 220px;
  min-width: 220px;
  max-width: 220px;
  flex-shrink: 0;
  height: 100%;
  background: var(--color-bg-primary);
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  user-select: none;
  overflow: hidden;
}

.sidebar-scrollable {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 10px 8px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.sidebar-scrollable::-webkit-scrollbar {
  width: 4px;
}
.sidebar-scrollable::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
}

.group-title {
  font-size: 0.7rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #64748b;
}

.group-action-btn {
  background: transparent;
  border: none;
  color: #64748b;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.12s;
}

.group-action-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 6px;
  font-size: 0.82rem;
  color: #94a3b8;
  cursor: pointer;
  transition: background-color 0.12s, color 0.12s;
  position: relative;
}

.nav-item:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.nav-item.active {
  background: var(--color-bg-active);
  color: var(--color-text-primary);
  font-weight: 600;
}

.item-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  color: #94a3b8;
  font-size: 0.85rem;
}

.nav-item.active .item-icon {
  color: #a855f7;
}

.item-icon.star-icon {
  color: #fbbf24;
}

.item-label {
  flex: 1;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-badge {
  font-size: 0.7rem;
  padding: 0 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.05);
  color: #64748b;
  font-weight: 500;
}

.nav-item.active .item-badge {
  background: rgba(168, 85, 247, 0.2);
  color: #e9d5ff;
}

.folder-actions {
  display: none;
  align-items: center;
  gap: 2px;
}

.folder-item:hover .folder-actions {
  display: flex;
}

.folder-item:hover .item-badge {
  display: none;
}

.icon-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 2px;
  border-radius: 3px;
  font-size: 0.72rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.icon-btn.remove-btn:hover {
  color: #ef4444;
}

.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 5px;
  padding: 4px 6px;
}

.tag-chip-eagle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 8px;
  border-radius: 4px;
  background: var(--color-bg-hover);
  font-size: 0.74rem;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.12s;
}

.tag-chip-eagle:hover {
  background: var(--color-bg-active);
  color: var(--color-text-primary);
}

.tag-chip-eagle.active {
  background: rgba(139, 92, 246, 0.2);
  color: #c4b5fd;
  font-weight: 600;
}

.tag-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.empty-hint {
  font-size: 0.74rem;
  color: #475569;
  padding: 4px 8px;
  margin: 0;
  font-style: italic;
}

.sidebar-footer {
  padding: 8px 8px 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  background: var(--color-bg-app);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.footer-tools {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
}

.tool-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 6px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 5px;
  color: #94a3b8;
  font-size: 0.72rem;
  cursor: pointer;
  transition: all 0.12s;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.07);
  color: #f1f5f9;
}

.tool-icon {
  font-size: 0.8rem;
}

.tool-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.footer-bottom-row {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}
</style>
