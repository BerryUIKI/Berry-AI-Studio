<script setup lang="ts">
import { computed, ref } from "vue";
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
  /** Latest `scan-progress` event, forwarded from the App shell listener. */
  progress: ScanProgress | null;
}>();

const emit = defineEmits<{
  removed: [folderId: number];
  scanned: [folderId: number];
  selectNav: [target: NavTarget];
  openAlbumModal: [];
  openTagModal: [];
  openPromptStats: [];
  moveFilesToFolder: [payload: { filePaths: string[]; folderId: number }];
  addFilesToAlbum: [payload: { fileIds: number[]; albumId: number }];
  tagFiles: [payload: { fileIds: number[]; tagId: number }];
}>();

/** The folder currently being scanned, and whether it is a full scan or a
 * metadata rebuild. Only one scan runs at a time. */
const running = ref<{ id: number; action: "scan" | "rebuild" } | null>(null);
const stats = ref<ScanStats | null>(null);
const error = ref("");

/** Strip the Windows `\\?\` verbatim prefix, if present. */
function displayPath(path: string): string {
  return path.replace(/^\\\\\?\\/, "");
}

function isBusy(id: number): boolean {
  return running.value?.id === id;
}

async function scan(folder: Folder, action: "scan" | "rebuild" = "scan") {
  error.value = "";
  stats.value = null;
  running.value = { id: folder.id, action };
  try {
    stats.value = await invoke<ScanStats>(
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

async function remove(folder: Folder) {
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

const progressPercent = computed(() => {
  if (!props.progress || props.progress.discovering || props.progress.found === 0) return 0;
  return Math.round((props.progress.scanned / props.progress.found) * 100);
});

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
  <nav class="sidebar-nav" aria-label="Organization navigation">
    <!-- Library Section -->
    <div class="nav-section">
      <div class="section-title">{{ t.nav.library }}</div>
      <ul class="list">
        <li
          :class="['row', { active: isTargetActive({ type: 'all' }) }]"
          @click="emit('selectNav', { type: 'all' })"
        >
          <div class="path">
            <span class="row-icon">🖼️</span>
            <span class="name">{{ t.nav.allImages }}</span>
            <span class="count-badge">{{ counts?.total ?? 0 }}</span>
          </div>
        </li>

        <li
          :class="['row', { active: isTargetActive({ type: 'favorites' }) }]"
          @click="emit('selectNav', { type: 'favorites' })"
        >
          <div class="path">
            <span class="row-icon">★</span>
            <span class="name">{{ t.nav.favorites }}</span>
          </div>
        </li>

        <li
          :class="['row', { active: isTargetActive({ type: 'nsfw' }) }]"
          @click="emit('selectNav', { type: 'nsfw' })"
        >
          <div class="path">
            <span class="row-icon">🔞</span>
            <span class="name">{{ t.nav.sensitive }}</span>
          </div>
        </li>

        <li
          class="row stats-nav-row"
          title="Analyze prompt keywords, models, samplers, and ratings"
          @click="emit('openPromptStats')"
        >
          <div class="path">
            <span class="row-icon">📊</span>
            <span class="name">{{ t.nav.promptInsights }}</span>
          </div>
        </li>
      </ul>
    </div>

    <!-- Folders Section -->
    <div class="nav-section">
      <div class="section-title">{{ t.nav.folders }}</div>
      <ul class="list">
        <li
          v-for="folder in folders"
          :key="folder.id"
          :class="['row', { active: isTargetActive({ type: 'folder', folder }) }]"
          @dragover.prevent
          @drop="onDropOnFolder($event, folder)"
          @click="emit('selectNav', { type: 'folder', folder })"
        >
          <div class="path">
            <span class="row-icon">📁</span>
            <span class="name" :title="displayPath(folder.path)">
              {{ displayPath(folder.path).split(/[\\/]/).pop() || displayPath(folder.path) }}
            </span>
            <span class="count-badge">
              {{ counts?.folders[folder.id] ?? 0 }}
            </span>
            <button
              class="ghost scan"
              :disabled="isBusy(folder.id)"
              title="Incremental scan"
              @click.stop="scan(folder, 'scan')"
            >
              {{ running?.id === folder.id && running?.action === "scan" ? t.nav.scanning : t.nav.scan }}
            </button>
            <button
              class="ghost rebuild"
              :disabled="isBusy(folder.id)"
              title="Force rebuild metadata"
              @click.stop="scan(folder, 'rebuild')"
            >
              {{ running?.id === folder.id && running?.action === "rebuild" ? t.nav.rebuilding : t.nav.rebuild }}
            </button>
            <button class="ghost remove" title="Remove folder from library" @click.stop="remove(folder)">×</button>
          </div>

          <div
            v-if="isBusy(folder.id) && props.progress && (props.progress.discovering || props.progress.found > 0)"
            class="bar"
          >
            <div
              class="fill"
              :class="{ indeterminate: props.progress.discovering }"
              :style="props.progress.discovering ? undefined : { width: progressPercent + '%' }"
            ></div>
            <span class="bar-label">
              {{ props.progress.scanned }}
              <template v-if="!props.progress.discovering"> / {{ props.progress.found }}</template>
              <template v-if="props.progress.current"> · {{ props.progress.current }}</template>
            </span>
          </div>

          <p v-if="folder.id === running?.id && stats" class="stats">
            +{{ stats.added }} added · {{ stats.updated }} updated ·
            {{ stats.unchanged }} unchanged · {{ stats.removed }} removed ·
            {{ stats.failed }} failed · {{ stats.duration_ms }} ms
          </p>
        </li>
      </ul>
      <p v-if="!folders.length" class="empty">{{ t.nav.noFolders }}</p>
    </div>

    <!-- Albums Section -->
    <div class="nav-section">
      <div class="section-header-row">
        <div class="section-title">{{ t.nav.albums }}</div>
        <button
          type="button"
          class="section-add-btn"
          title="Manage or Create Albums"
          @click="emit('openAlbumModal')"
        >
          {{ t.nav.newAlbum }}
        </button>
      </div>

      <ul v-if="albums && albums.length > 0" class="list">
        <li
          v-for="album in albums"
          :key="album.id"
          :class="['row', { active: isTargetActive({ type: 'album', album }) }]"
          @dragover.prevent
          @drop="onDropOnAlbum($event, album)"
          @click="emit('selectNav', { type: 'album', album })"
        >
          <div class="path">
            <span class="row-icon">🗂️</span>
            <span class="name" :title="album.name">{{ album.name }}</span>
            <span v-if="albumCounts && albumCounts[album.id] !== undefined" class="count-badge">
              {{ albumCounts[album.id] }}
            </span>
          </div>
        </li>
      </ul>
      <p v-else class="empty">{{ t.nav.noAlbums }}</p>
    </div>

    <!-- Tags Section -->
    <div class="nav-section">
      <div class="section-header-row">
        <div class="section-title">{{ t.nav.tags }}</div>
        <button
          type="button"
          class="section-add-btn"
          title="Manage or Create Tags"
          @click="emit('openTagModal')"
        >
          {{ t.nav.newTag }}
        </button>
      </div>

      <div v-if="tags && tags.length > 0" class="tags-pill-list">
        <button
          v-for="tag in tags"
          :key="tag.id"
          type="button"
          class="tag-nav-chip"
          :class="{ active: isTargetActive({ type: 'tag', tag }) }"
          :style="{ borderColor: tag.color || '#3b82f6' }"
          @dragover.prevent
          @drop="onDropOnTag($event, tag)"
          @click="emit('selectNav', { type: 'tag', tag })"
        >
          <span
            class="tag-nav-dot"
            :style="{ backgroundColor: tag.color || '#3b82f6' }"
          />
          <span class="tag-nav-name">{{ tag.name }}</span>
        </button>
      </div>
      <p v-else class="empty">{{ t.nav.noTags }}</p>
    </div>

    <p v-if="error" class="error">{{ error }}</p>
  </nav>
</template>

<style scoped>
.sidebar-nav {
  margin-bottom: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.nav-section {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.section-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 0.25rem;
}

.section-title {
  font-size: 0.76em;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: #888;
}

.section-add-btn {
  background: transparent;
  border: none;
  color: #2f6fed;
  font: inherit;
  font-size: 0.78em;
  font-weight: 600;
  cursor: pointer;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
}

.section-add-btn:hover {
  background: rgba(47, 111, 237, 0.1);
}

.empty {
  color: #888;
  font-size: 0.82em;
  margin: 0.2rem 0 0.4rem 0.4rem;
  font-style: italic;
}

.list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.row {
  padding: 0.45rem 0.75rem;
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  margin-bottom: 0.4rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.row:hover {
  border-color: rgba(128, 128, 128, 0.4);
}

.row.active {
  border-color: #2f6fed;
  background: rgba(47, 111, 237, 0.08);
  font-weight: 500;
}

.path {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.name {
  flex: 1;
  word-break: break-all;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 0.88em;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.row-icon {
  font-size: 1.05em;
  line-height: 1;
  user-select: none;
}

.count-badge {
  font-size: 0.75em;
  font-weight: 600;
  padding: 0.12rem 0.45rem;
  border-radius: 999px;
  background: rgba(128, 128, 128, 0.15);
  color: #666;
  margin-right: 0.25rem;
}

@media (prefers-color-scheme: dark) {
  .count-badge {
    background: rgba(255, 255, 255, 0.12);
    color: #bbb;
  }
}

.tags-pill-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  padding: 0.2rem 0;
}

.tag-nav-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.25rem 0.6rem;
  border-radius: 999px;
  background: rgba(128, 128, 128, 0.1);
  border: 1px solid transparent;
  color: inherit;
  font: inherit;
  font-size: 0.8em;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tag-nav-chip:hover {
  background: rgba(128, 128, 128, 0.2);
  transform: translateY(-1px);
}

.tag-nav-chip.active {
  background: rgba(47, 111, 237, 0.15);
  border-color: #2f6fed !important;
  font-weight: 600;
}

.tag-nav-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.tag-nav-name {
  line-height: 1;
}

button.ghost {
  font: inherit;
  font-size: 0.82em;
  border: 1px solid rgba(128, 128, 128, 0.35);
  background: transparent;
  border-radius: 6px;
  padding: 0.12rem 0.5rem;
  cursor: pointer;
}

button.ghost:disabled {
  opacity: 0.6;
  cursor: default;
}

button.remove {
  color: #d33;
}

button.rebuild {
  color: #2f6fed;
}

.bar {
  position: relative;
  margin-top: 0.5rem;
  height: 1.1rem;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.15);
}

.fill {
  height: 100%;
  background: #2f6fed;
  transition: width 0.15s ease-out;
}

.fill.indeterminate {
  width: 35%;
  animation: scan-discovery 1.1s ease-in-out infinite;
}

@keyframes scan-discovery {
  from { transform: translateX(-110%); }
  to { transform: translateX(300%); }
}

@media (prefers-reduced-motion: reduce) {
  .fill.indeterminate {
    width: 45%;
    animation: none;
  }
}

.bar-label {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  padding: 0 0.5rem;
  font-size: 0.75em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stats {
  margin: 0.4rem 0 0;
  font-size: 0.8em;
  color: #666;
}

.error {
  color: #d33;
  font-size: 0.85em;
}
</style>
