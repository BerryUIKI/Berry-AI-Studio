<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { UnlistenFn } from "@tauri-apps/api/event";

defineProps<{
  title?: string;
  subtitle?: string;
}>();

const appWindow = getCurrentWindow();
const isMaximized = ref(false);
let unlistenResize: UnlistenFn | null = null;

async function checkMaximized() {
  try {
    isMaximized.value = await appWindow.isMaximized();
  } catch {
    // Non-tauri or mock environment
  }
}

onMounted(async () => {
  await checkMaximized();
  try {
    unlistenResize = await appWindow.onResized(async () => {
      await checkMaximized();
    });
  } catch {
    // Ignore in browser dev preview
  }
});

onUnmounted(() => {
  unlistenResize?.();
});

async function onMinimize() {
  try {
    await appWindow.minimize();
  } catch (err) {
    console.error("Failed to minimize window:", err);
  }
}

async function onToggleMaximize() {
  try {
    await appWindow.toggleMaximize();
    await checkMaximized();
  } catch (err) {
    console.error("Failed to toggle maximize:", err);
  }
}

async function onClose() {
  try {
    await appWindow.close();
  } catch (err) {
    console.error("Failed to close window:", err);
  }
}
</script>

<template>
  <header class="titlebar" data-tauri-drag-region @dblclick="onToggleMaximize">
    <div class="titlebar-left">
      <!-- Leading Slot (e.g. Sidebar toggle button before logo) -->
      <slot name="leading"></slot>

      <div class="titlebar-brand" data-tauri-drag-region>
        <div class="brand-icon">
          <img src="../assets/logo.png" alt="Berry Logo" width="18" height="18" class="brand-logo-img" />
        </div>
        <span class="brand-title">{{ title || "Berry AI Studio" }}</span>
        <span v-if="subtitle" class="brand-subtitle">{{ subtitle }}</span>
      </div>

      <!-- Menu Slot -->
      <slot name="menu"></slot>
    </div>

    <!-- Draggable Center Space -->
    <div class="titlebar-drag-spacer" data-tauri-drag-region>
      <slot name="center"></slot>
    </div>

    <!-- Window Controls -->
    <div class="titlebar-controls">
      <slot name="actions"></slot>

      <button
        type="button"
        class="control-btn minimize"
        title="Minimize"
        aria-label="Minimize"
        @click="onMinimize"
      >
        <svg viewBox="0 0 12 12" width="11" height="11">
          <path d="M 1 6.5 L 11 6.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>

      <button
        type="button"
        class="control-btn maximize"
        :title="isMaximized ? 'Restore Down' : 'Maximize'"
        :aria-label="isMaximized ? 'Restore Down' : 'Maximize'"
        @click="onToggleMaximize"
      >
        <!-- Maximize icon -->
        <svg v-if="!isMaximized" viewBox="0 0 12 12" width="11" height="11">
          <rect x="1.5" y="1.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.2" rx="1" />
        </svg>
        <!-- Restore down icon -->
        <svg v-else viewBox="0 0 12 12" width="11" height="11">
          <rect x="3.5" y="1.5" width="7" height="7" fill="none" stroke="currentColor" stroke-width="1" rx="0.5" />
          <path d="M 1.5 4.5 L 1.5 10.5 L 7.5 10.5 L 7.5 4.5 Z" fill="#18181b" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>

      <button
        type="button"
        class="control-btn close"
        title="Close"
        aria-label="Close"
        @click="onClose"
      >
        <svg viewBox="0 0 12 12" width="11" height="11">
          <path d="M 2 2 L 10 10 M 10 2 L 2 10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  height: 38px;
  min-height: 38px;
  background: var(--color-bg-app);
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  display: flex;
  align-items: center;
  justify-content: space-between;
  user-select: none;
  z-index: 999;
  position: relative;
}

.titlebar-left {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 100%;
  flex-shrink: 0;
  min-width: 0;
}

.titlebar-brand {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  height: 100%;
  flex-shrink: 0;
  min-width: 0;
}

.brand-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.brand-logo-img {
  display: block;
  object-fit: contain;
}

.brand-title {
  font-size: 0.8rem;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.brand-subtitle {
  font-size: 0.7rem;
  padding: 1px 6px;
  border-radius: 999px;
  background: rgba(18, 181, 203, 0.15);
  color: #67e8f9;
  border: 1px solid rgba(18, 181, 203, 0.3);
  flex-shrink: 0;
}

.titlebar-drag-spacer {
  flex: 1;
  min-width: 0;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.titlebar-controls {
  display: flex;
  align-items: center;
  height: 100%;
  flex-shrink: 0;
}

.control-btn {
  width: 44px;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #94a3b8;
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s;
  outline: none;
}

.control-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.control-btn.close:hover {
  background: #ef4444;
  color: #ffffff;
}

.control-btn:active {
  background: rgba(255, 255, 255, 0.12);
}

.control-btn.close:active {
  background: #dc2626;
}
</style>
