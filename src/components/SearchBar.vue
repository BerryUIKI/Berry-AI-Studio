<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { t } from "../i18n";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    isSemantic?: boolean;
    placeholder?: string;
    loading?: boolean;
    resultCount?: number | null;
  }>(),
  {
    isSemantic: false,
    placeholder: "",
    loading: false,
    resultCount: null,
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "update:isSemantic", value: boolean): void;
  (e: "search", value: string): void;
  (e: "clear"): void;
  (e: "open-clip-manager"): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const localQuery = ref(props.modelValue);
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => props.modelValue,
  (val) => {
    if (val !== localQuery.value) {
      localQuery.value = val;
    }
  },
);

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  localQuery.value = target.value;
  emit("update:modelValue", target.value);

  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  debounceTimer = setTimeout(() => {
    emit("search", localQuery.value.trim());
  }, 250);
}

function onEnter() {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  emit("search", localQuery.value.trim());
}

function clear() {
  localQuery.value = "";
  emit("update:modelValue", "");
  emit("clear");
  inputRef.value?.focus();
}

function handleGlobalKey(e: KeyboardEvent) {
  const tag = (document.activeElement?.tagName ?? "").toLowerCase();
  if (tag === "input" || tag === "textarea") return;

  if (e.key === "/" || ((e.ctrlKey || e.metaKey) && (e.key === "f" || e.key === "F"))) {
    e.preventDefault();
    inputRef.value?.focus();
    inputRef.value?.select();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleGlobalKey);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKey);
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
});
function toggleMode() {
  const next = !props.isSemantic;
  emit("update:isSemantic", next);
  emit("search", localQuery.value.trim());
}
</script>

<template>
  <div class="search-bar-eagle">
    <div :class="['search-box', { 'semantic-mode': isSemantic }]">
      <!-- Semantic Mode Toggle Button -->
      <button
        type="button"
        class="mode-toggle-btn"
        :class="{ active: isSemantic }"
        :title="isSemantic ? t.search.semanticSearch : t.search.syntaxSearch"
        @click="toggleMode"
      >
        <span class="mode-icon">{{ isSemantic ? '🧠' : '🔍' }}</span>
      </button>

      <input
        ref="inputRef"
        :value="localQuery"
        type="text"
        class="search-input"
        :placeholder="isSemantic ? t.search.semanticPlaceholder : (placeholder || t.search.placeholder)"
        @input="onInput"
        @keydown.enter="onEnter"
      />

      <!-- CLIP Manager Trigger Button in Semantic Mode -->
      <button
        v-if="isSemantic"
        type="button"
        class="clip-index-btn"
        :title="t.search.clipManager"
        @click="emit('open-clip-manager')"
      >
        ⚙️
      </button>

      <span v-if="loading" class="spinner" :title="t.view.loading">⏳</span>

      <button
        v-if="localQuery"
        type="button"
        class="clear-btn"
        :title="t.search.clearSearch"
        @click="clear"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.search-bar-eagle {
  display: flex;
  align-items: center;
  width: 100%;
  min-width: 120px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 6px;
  background: #202024;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  padding: 0 8px;
  height: 30px;
  width: 100%;
  transition: all 0.15s ease;
}

.search-box:focus-within {
  border-color: rgba(168, 85, 247, 0.5);
  box-shadow: 0 0 0 2px rgba(168, 85, 247, 0.15);
  background: #242428;
}

.search-box.semantic-mode {
  border-color: rgba(139, 92, 246, 0.4);
  background: #211d2e;
}

.search-box.semantic-mode:focus-within {
  border-color: #8b5cf6;
  box-shadow: 0 0 0 2px rgba(139, 92, 246, 0.25);
  background: #262136;
}

.mode-toggle-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.82rem;
  transition: all 0.15s ease;
  line-height: 1;
}

.mode-toggle-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  transform: scale(1.05);
}

.mode-toggle-btn.active {
  background: rgba(139, 92, 246, 0.25);
}

.clip-index-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.75rem;
  color: #a855f7;
  transition: all 0.15s ease;
  line-height: 1;
}

.clip-index-btn:hover {
  background: rgba(168, 85, 247, 0.2);
  transform: scale(1.08);
}

.search-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #71717a;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  outline: none;
  font: inherit;
  font-size: 0.78rem;
  color: #f1f5f9;
}

.search-input::placeholder {
  color: #52525b;
}

.spinner {
  font-size: 0.75rem;
  flex-shrink: 0;
}

.clear-btn {
  background: transparent;
  border: none;
  color: #71717a;
  font-size: 0.75rem;
  cursor: pointer;
  padding: 2px 4px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  flex-shrink: 0;
  transition: all 0.12s ease;
}

.clear-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #f87171;
}
</style>
