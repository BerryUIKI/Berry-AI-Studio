<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { currentLocaleSetting, setLocale, SUPPORTED_LOCALES, t, type LocaleSetting } from "../i18n";

const emit = defineEmits<{
  addFolder: [];
  scanActive: [];
  rescanAll: [];
  openDbManager: [];
  openSettings: [];
  selectAll: [];
  clearSelection: [];
  batchAlbum: [];
  batchTag: [];
  batchTrash: [];
  batchMove: [];
  batchCopy: [];
  batchRate: [rating: number | null];
  setViewMode: [mode: "grid" | "table"];
  toggleSidebar: [];
  toggleInspector: [];
  openLightbox: [];
  zoomIn: [];
  zoomOut: [];
  resetZoom: [];
  openPromptStats: [];
  openModelManager: [];
  openClipManager: [];
  openShortcutsHelp: [];
  openUpdater: [];
  openAbout: [];
}>();

const activeMenu = ref<string | null>(null);
const isLanguageSubmenuOpen = ref(false);

function toggleMenu(menu: string) {
  if (activeMenu.value === menu) {
    activeMenu.value = null;
    isLanguageSubmenuOpen.value = false;
  } else {
    activeMenu.value = menu;
    isLanguageSubmenuOpen.value = false;
  }
}

function onMenuHover(menu: string) {
  if (activeMenu.value !== null) {
    activeMenu.value = menu;
    isLanguageSubmenuOpen.value = false;
  }
}

function closeAll() {
  activeMenu.value = null;
  isLanguageSubmenuOpen.value = false;
}

function handleAction(action: () => void) {
  action();
  closeAll();
}

function onSelectLocale(setting: LocaleSetting) {
  setLocale(setting);
  closeAll();
}

async function quitApp() {
  closeAll();
  try {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  } catch (err) {
    console.error("Quit app failed:", err);
  }
}

function onClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (!target.closest(".menu-bar-container")) {
    closeAll();
  }
}

onMounted(() => {
  window.addEventListener("click", onClickOutside);
});

onUnmounted(() => {
  window.removeEventListener("click", onClickOutside);
});
</script>

<template>
  <nav class="menu-bar-container" data-tauri-drag-region="false">
    <!-- File Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'file' }">
      <button
        type="button"
        class="menu-trigger"
        @click="toggleMenu('file')"
        @mouseenter="onMenuHover('file')"
      >
        {{ t.menu.file }}
      </button>
      <div v-if="activeMenu === 'file'" class="dropdown-menu">
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('addFolder'))">
          <span class="item-icon">📁</span>
          <span class="item-title">{{ t.menu.addFolder }}</span>
          <span class="item-key">Ctrl+O</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('scanActive'))">
          <span class="item-icon">🔄</span>
          <span class="item-title">{{ t.menu.scanActive }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('rescanAll'))">
          <span class="item-icon">⚡</span>
          <span class="item-title">{{ t.menu.rescanAll }}</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openDbManager'))">
          <span class="item-icon">🗄️</span>
          <span class="item-title">{{ t.menu.dbManager }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openSettings'))">
          <span class="item-icon">⚙️</span>
          <span class="item-title">{{ t.menu.preferences }}</span>
          <span class="item-key">Ctrl+,</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="dropdown-item danger" @click="quitApp">
          <span class="item-icon">❌</span>
          <span class="item-title">{{ t.menu.exit }}</span>
          <span class="item-key">Alt+F4</span>
        </button>
      </div>
    </div>

    <!-- Edit Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'edit' }">
      <button
        type="button"
        class="menu-trigger"
        @click="toggleMenu('edit')"
        @mouseenter="onMenuHover('edit')"
      >
        {{ t.menu.edit }}
      </button>
      <div v-if="activeMenu === 'edit'" class="dropdown-menu">
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('selectAll'))">
          <span class="item-icon">✓</span>
          <span class="item-title">{{ t.menu.selectAll }}</span>
          <span class="item-key">Ctrl+A</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('clearSelection'))">
          <span class="item-icon">✕</span>
          <span class="item-title">{{ t.menu.clearSelection }}</span>
          <span class="item-key">Esc</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('batchTag'))">
          <span class="item-icon">🏷️</span>
          <span class="item-title">{{ t.menu.batchTag }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('batchAlbum'))">
          <span class="item-icon">📚</span>
          <span class="item-title">{{ t.menu.batchAlbum }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('batchMove'))">
          <span class="item-icon">↗</span>
          <span class="item-title">{{ t.menu.batchMove }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('batchCopy'))">
          <span class="item-icon">📋</span>
          <span class="item-title">{{ t.menu.batchCopy }}</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="dropdown-item danger" @click="handleAction(() => emit('batchTrash'))">
          <span class="item-icon">🗑️</span>
          <span class="item-title">{{ t.menu.batchTrash }}</span>
          <span class="item-key">Del</span>
        </button>
      </div>
    </div>

    <!-- View Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'view' }">
      <button
        type="button"
        class="menu-trigger"
        @click="toggleMenu('view')"
        @mouseenter="onMenuHover('view')"
      >
        {{ t.menu.view }}
      </button>
      <div v-if="activeMenu === 'view'" class="dropdown-menu">
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('setViewMode', 'grid'))">
          <span class="item-icon">⊞</span>
          <span class="item-title">{{ t.menu.grid }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('setViewMode', 'table'))">
          <span class="item-icon">☰</span>
          <span class="item-title">{{ t.menu.table }}</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('toggleSidebar'))">
          <span class="item-icon">📁</span>
          <span class="item-title">{{ t.menu.toggleSidebar }}</span>
          <span class="item-key">B</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('toggleInspector'))">
          <span class="item-icon">👁️</span>
          <span class="item-title">{{ t.menu.toggleInspector }}</span>
          <span class="item-key">I</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openLightbox'))">
          <span class="item-icon">🔍</span>
          <span class="item-title">{{ t.menu.lightbox }}</span>
          <span class="item-key">Space</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('zoomIn'))">
          <span class="item-icon">➕</span>
          <span class="item-title">{{ t.menu.zoomIn }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('zoomOut'))">
          <span class="item-icon">➖</span>
          <span class="item-title">{{ t.menu.zoomOut }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('resetZoom'))">
          <span class="item-icon">↺</span>
          <span class="item-title">{{ t.menu.resetZoom }}</span>
        </button>
      </div>
    </div>

    <!-- Tools Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'tools' }">
      <button
        type="button"
        class="menu-trigger"
        @click="toggleMenu('tools')"
        @mouseenter="onMenuHover('tools')"
      >
        {{ t.menu.tools }}
      </button>
      <div v-if="activeMenu === 'tools'" class="dropdown-menu">
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openPromptStats'))">
          <span class="item-icon">📊</span>
          <span class="item-title">{{ t.menu.promptStats }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openModelManager'))">
          <span class="item-icon">🧠</span>
          <span class="item-title">{{ t.menu.modelManager }}</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openClipManager'))">
          <span class="item-icon">🔎</span>
          <span class="item-title">{{ t.clipModal.title }}</span>
        </button>
      </div>
    </div>

    <!-- Help Menu -->
    <div class="menu-item" :class="{ open: activeMenu === 'help' }">
      <button
        type="button"
        class="menu-trigger"
        @click="toggleMenu('help')"
        @mouseenter="onMenuHover('help')"
      >
        {{ t.menu.help }}
      </button>
      <div v-if="activeMenu === 'help'" class="dropdown-menu">
        <!-- Language Submenu -->
        <div
          class="dropdown-submenu-wrapper"
          @mouseenter="isLanguageSubmenuOpen = true"
          @mouseleave="isLanguageSubmenuOpen = false"
        >
          <button type="button" class="dropdown-item has-submenu">
            <span class="item-icon">🌐</span>
            <span class="item-title">{{ t.menu.language }}</span>
            <span class="submenu-arrow">▶</span>
          </button>
          <div v-if="isLanguageSubmenuOpen" class="dropdown-submenu">
            <button
              v-for="loc in SUPPORTED_LOCALES"
              :key="loc.key"
              type="button"
              class="dropdown-item"
              :class="{ selected: currentLocaleSetting === loc.key }"
              @click="onSelectLocale(loc.key)"
            >
              <span class="check-icon">{{ currentLocaleSetting === loc.key ? '✓' : '' }}</span>
              <span class="item-title">{{ loc.label }}</span>
            </button>
          </div>
        </div>

        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openShortcutsHelp'))">
          <span class="item-icon">⌨️</span>
          <span class="item-title">{{ t.menu.shortcuts }}</span>
          <span class="item-key">?</span>
        </button>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openUpdater'))">
          <span class="item-icon">🚀</span>
          <span class="item-title">{{ t.menu.checkUpdates }}</span>
        </button>
        <div class="menu-divider"></div>
        <button type="button" class="dropdown-item" @click="handleAction(() => emit('openAbout'))">
          <span class="item-icon">
            <img src="../assets/logo.png" alt="Berry" width="14" height="14" style="display:block; object-fit:contain;" />
          </span>
          <span class="item-title">{{ t.menu.about }}</span>
        </button>
      </div>
    </div>
  </nav>
</template>

<style scoped>
.menu-bar-container {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 100%;
  user-select: none;
  font-size: 0.76rem;
  z-index: 1000;
  position: relative;
}

.menu-item {
  position: relative;
  height: 100%;
  display: flex;
  align-items: center;
}

.menu-trigger {
  background: transparent;
  border: none;
  color: #94a3b8;
  padding: 3px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.74rem;
  font-family: inherit;
  transition: all 0.12s ease;
  white-space: nowrap;
  outline: none;
}

.menu-trigger:hover,
.menu-item.open .menu-trigger {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  background: #1c1c20;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.55);
  min-width: 190px;
  padding: 4px 0;
  z-index: 1001;
  display: flex;
  flex-direction: column;
  backdrop-filter: blur(8px);
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: transparent;
  border: none;
  color: #cbd5e1;
  font-size: 0.74rem;
  font-family: inherit;
  cursor: pointer;
  text-align: left;
  transition: background-color 0.1s, color 0.1s;
  width: 100%;
  outline: none;
}

.dropdown-item:hover {
  background: rgba(18, 181, 203, 0.2);
  color: #ffffff;
}

.dropdown-item.danger:hover {
  background: rgba(239, 68, 68, 0.2);
  color: #fca5a5;
}

.item-icon {
  width: 16px;
  font-size: 0.8rem;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.item-title {
  flex: 1;
  white-space: nowrap;
}

.item-key {
  font-size: 0.68rem;
  color: #64748b;
  margin-left: 12px;
  flex-shrink: 0;
}

.menu-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.08);
  margin: 4px 0;
}

.dropdown-submenu-wrapper {
  position: relative;
  width: 100%;
}

.dropdown-item.has-submenu {
  display: flex;
  justify-content: space-between;
}

.submenu-arrow {
  font-size: 0.6rem;
  color: #64748b;
}

.dropdown-submenu {
  position: absolute;
  top: 0;
  left: 100%;
  background: #1c1c20;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.55);
  min-width: 175px;
  padding: 4px 0;
  z-index: 1002;
  display: flex;
  flex-direction: column;
}

.check-icon {
  width: 14px;
  font-size: 0.72rem;
  color: #12b5cb;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dropdown-item.selected {
  color: #67e8f9;
  font-weight: 600;
}
</style>
