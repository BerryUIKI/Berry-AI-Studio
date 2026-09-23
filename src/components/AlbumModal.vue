<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";
import type { Album } from "../types";

const props = defineProps<{
  open: boolean;
  fileIds?: number[]; // If provided, modal acts as "Add to Album" selector
}>();

const emit = defineEmits<{
  (e: "update:open", val: boolean): void;
  (e: "selectAlbum", album: Album): void;
  (e: "addedToAlbum", album: Album): void;
  (e: "albumsChanged"): void;
}>();

const albums = ref<Album[]>([]);
const albumCounts = ref<Record<number, number>>({});
const loading = ref(false);
const error = ref("");

const newAlbumName = ref("");
const newAlbumDesc = ref("");
const isCreating = ref(false);
const editingAlbumId = ref<number | null>(null);
const editingAlbumName = ref("");

onMounted(async () => {
  await loadAlbums();
});

async function loadAlbums() {
  loading.value = true;
  error.value = "";
  try {
    const [fetchedAlbums, counts] = await Promise.all([
      invoke<Album[]>("list_albums"),
      invoke<Record<number, number>>("get_album_counts"),
    ]);
    albums.value = fetchedAlbums;
    albumCounts.value = counts;
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

async function onCreateAlbum() {
  const name = newAlbumName.value.trim();
  if (!name) return;
  try {
    const created = await invoke<Album>("create_album", {
      name,
      description: newAlbumDesc.value.trim() || null,
    });
    newAlbumName.value = "";
    newAlbumDesc.value = "";
    isCreating.value = false;
    await loadAlbums();
    emit("albumsChanged");

    // If adding files, immediately add to newly created album
    if (props.fileIds && props.fileIds.length > 0) {
      await onAddFiles(created);
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function onRenameAlbum(album: Album) {
  const newName = editingAlbumName.value.trim();
  if (!newName || newName === album.name) {
    editingAlbumId.value = null;
    return;
  }
  try {
    await invoke("rename_album", { id: album.id, newName });
    editingAlbumId.value = null;
    await loadAlbums();
    emit("albumsChanged");
  } catch (e) {
    error.value = String(e);
  }
}

async function onDeleteAlbum(albumId: number) {
  if (!confirm("Are you sure you want to delete this album? Images will not be deleted from disk.")) {
    return;
  }
  try {
    await invoke("delete_album", { id: albumId });
    await loadAlbums();
    emit("albumsChanged");
  } catch (e) {
    error.value = String(e);
  }
}

async function onAddFiles(album: Album) {
  if (!props.fileIds || props.fileIds.length === 0) {
    emit("selectAlbum", album);
    emit("update:open", false);
    return;
  }
  try {
    await invoke("add_files_to_album", {
      albumId: album.id,
      fileIds: props.fileIds,
    });
    emit("addedToAlbum", album);
    emit("update:open", false);
  } catch (e) {
    error.value = String(e);
  }
}

function startEditing(album: Album) {
  editingAlbumId.value = album.id;
  editingAlbumName.value = album.name;
}
</script>

<template>
  <div v-if="open" class="modal-backdrop" @click="emit('update:open', false)" v-dialog="() => emit('update:open', false)">
    <div class="modal-dialog" role="dialog" aria-modal="true" @click.stop>
      <div class="modal-header">
        <h2>
          {{ fileIds && fileIds.length > 0 ? `${t.batch.album} (${fileIds.length})` : t.albumsModal.title }}
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
        <!-- New Album Form -->
        <div v-if="isCreating" class="create-form">
          <h3>{{ t.albumsModal.createBtn }}</h3>
          <div class="form-group">
            <label for="album-name">{{ t.albumsModal.createPlaceholder }}</label>
            <input
              id="album-name"
              v-model="newAlbumName"
              type="text"
              :placeholder="t.albumsModal.createPlaceholder"
              autofocus
              @keydown.enter.prevent="onCreateAlbum"
            />
          </div>
          <div class="form-group">
            <label for="album-desc">Description</label>
            <input
              id="album-desc"
              v-model="newAlbumDesc"
              type="text"
              placeholder="..."
              @keydown.enter.prevent="onCreateAlbum"
            />
          </div>
          <div class="form-actions">
            <button type="button" class="btn-cancel" @click="isCreating = false">
              {{ t.fileOpModal.cancel }}
            </button>
            <button
              type="button"
              class="btn-primary"
              :disabled="!newAlbumName.trim()"
              @click="onCreateAlbum"
            >
              {{ t.albumsModal.createBtn }}
            </button>
          </div>
        </div>

        <div v-else class="header-action">
          <button type="button" class="btn-create" @click="isCreating = true">
            + {{ t.nav.newAlbum }}
          </button>
        </div>

        <!-- Album list -->
        <div v-if="loading" class="album-loading">Loading...</div>
        <div v-else-if="albums.length === 0" class="album-empty">
          {{ t.albumsModal.noAlbums }}
        </div>
        <div v-else class="album-list">
          <div
            v-for="album in albums"
            :key="album.id"
            class="album-item"
            :class="{ 'selectable': fileIds && fileIds.length > 0 }"
            @click="onAddFiles(album)"
          >
            <div class="album-info">
              <div v-if="editingAlbumId === album.id" class="edit-row" @click.stop>
                <input
                  v-model="editingAlbumName"
                  type="text"
                  class="edit-input"
                  autofocus
                  @keydown.enter="onRenameAlbum(album)"
                  @keydown.esc="editingAlbumId = null"
                />
                <button type="button" class="btn-save" @click="onRenameAlbum(album)">
                  Save
                </button>
                <button type="button" class="btn-cancel-mini" @click="editingAlbumId = null">
                  ✕
                </button>
              </div>
              <template v-else>
                <div class="album-name-row">
                  <span class="album-name">{{ album.name }}</span>
                  <span class="album-count">
                    {{ albumCounts[album.id] ?? 0 }} {{ (albumCounts[album.id] === 1) ? 'item' : 'items' }}
                  </span>
                </div>
                <div v-if="album.description" class="album-desc">
                  {{ album.description }}
                </div>
              </template>
            </div>

            <div v-if="!fileIds || fileIds.length === 0" class="album-ops" @click.stop>
              <button
                type="button"
                class="op-btn"
                title="Rename Album"
                @click="startEditing(album)"
              >
                ✎
              </button>
              <button
                type="button"
                class="op-btn delete-btn"
                title="Delete Album"
                @click="onDeleteAlbum(album.id)"
              >
                🗑
              </button>
            </div>
            <div v-else class="album-select-arrow">
              →
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
  max-width: 480px;
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

.album-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.album-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.65rem 0.9rem;
  background: rgba(128, 128, 128, 0.08);
  border: 1px solid rgba(128, 128, 128, 0.15);
  border-radius: 8px;
  transition: all 0.15s ease;
}

.album-item.selectable {
  cursor: pointer;
}

.album-item.selectable:hover {
  border-color: #2f6fed;
  background: rgba(47, 111, 237, 0.1);
}

.album-info {
  flex: 1;
  min-width: 0;
}

.album-name-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.album-name {
  font-weight: 600;
  font-size: 0.92em;
}

.album-count {
  font-size: 0.76em;
  background: rgba(128, 128, 128, 0.2);
  padding: 0.1rem 0.45rem;
  border-radius: 999px;
  color: #aaa;
}

@media (prefers-color-scheme: light) {
  .album-count {
    color: #666;
  }
}

.album-desc {
  font-size: 0.8em;
  color: #888;
  margin-top: 0.15rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.album-ops {
  display: flex;
  align-items: center;
  gap: 0.3rem;
}

.op-btn {
  background: transparent;
  border: none;
  color: #888;
  font-size: 0.95em;
  cursor: pointer;
  padding: 0.25rem 0.45rem;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.op-btn:hover {
  color: #2f6fed;
  background: rgba(47, 111, 237, 0.15);
}

.op-btn.delete-btn:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.15);
}

.album-select-arrow {
  color: #2f6fed;
  font-weight: bold;
  font-size: 1.2em;
  padding-left: 0.5rem;
}

.edit-row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.edit-input {
  flex: 1;
  padding: 0.3rem 0.5rem;
  border-radius: 4px;
  border: 1px solid #2f6fed;
  background: rgba(0, 0, 0, 0.2);
  color: inherit;
  font: inherit;
  font-size: 0.88em;
}

.btn-save {
  background: #2f6fed;
  color: #fff;
  border: none;
  border-radius: 4px;
  padding: 0.3rem 0.6rem;
  font-size: 0.8em;
  cursor: pointer;
}

.btn-cancel-mini {
  background: transparent;
  border: none;
  color: #888;
  font-size: 0.85em;
  cursor: pointer;
}

.album-empty,
.album-loading {
  text-align: center;
  color: #888;
  font-size: 0.88em;
  padding: 1.5rem 0;
}
</style>
