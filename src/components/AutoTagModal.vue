<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type {
  ImageFile,
  TaggerConfig,
  TaggerModelSummary,
  TagPrediction,
  BatchTagResult,
} from "../types";

const props = defineProps<{
  show: boolean;
  selectedFile: ImageFile | null;
  selectedFileCount: number;
  selectedFileIds: number[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "tags-applied"): void;
}>();

const models = ref<TaggerModelSummary[]>([]);
const selectedModelPath = ref<string>("");
const loadedModel = ref<{ name: string; model_path: string } | null>(null);

const generalThreshold = ref<number>(35);
const characterThreshold = ref<number>(85);
const includeRating = ref<boolean>(false);
const maxTags = ref<number>(50);

const isDetecting = ref(false);
const isApplying = ref(false);
const predictions = ref<TagPrediction[]>([]);
const message = ref<{ type: "success" | "error"; text: string } | null>(null);

const taggerConfig = computed<TaggerConfig>(() => ({
  general_threshold: generalThreshold.value / 100,
  character_threshold: characterThreshold.value / 100,
  include_rating: includeRating.value,
  max_tags: maxTags.value,
}));

async function loadModelList() {
  try {
    const list = await invoke<TaggerModelSummary[]>("list_tagger_models");
    models.value = list;
    const currentlyLoaded = await invoke<{ name: string; model_path: string } | null>(
      "get_loaded_tagger_model",
    );
    loadedModel.value = currentlyLoaded;

    if (currentlyLoaded) {
      selectedModelPath.value = currentlyLoaded.model_path;
    } else if (list.length > 0) {
      selectedModelPath.value = list[0].model_path;
      // Auto-load first model if none active
      await onSelectModel(list[0]);
    }
  } catch (err: any) {
    console.error("Failed to list tagger models:", err);
  }
}

async function onSelectModel(modelSummary: TaggerModelSummary) {
  try {
    message.value = null;
    isDetecting.value = true;
    const info = await invoke<{ name: string; model_path: string }>("load_tagger_model", {
      modelPath: modelSummary.model_path,
      tagsPath: modelSummary.tags_path,
    });
    loadedModel.value = info;
    selectedModelPath.value = info.model_path;
    // Refresh models list to update is_loaded flags
    models.value = await invoke<TaggerModelSummary[]>("list_tagger_models");
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isDetecting.value = false;
  }
}

async function browseModelFolder() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: t.value.autoTagModal.browseFolder,
    });

    if (typeof selected === "string") {
      // Find model.onnx and selected_tags.csv inside selected directory
      const modelPath = `${selected}/model.onnx`.replace(/\\/g, "/");
      const tagsPath = `${selected}/selected_tags.csv`.replace(/\\/g, "/");

      message.value = null;
      isDetecting.value = true;
      const info = await invoke<{ name: string; model_path: string }>("load_tagger_model", {
        modelPath,
        tagsPath,
      });
      loadedModel.value = info;
      selectedModelPath.value = info.model_path;
      models.value = await invoke<TaggerModelSummary[]>("list_tagger_models");
      message.value = { type: "success", text: `${t.value.autoTagModal.modelLoaded}: ${info.name}` };
    }
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isDetecting.value = false;
  }
}

async function detectTags() {
  if (!props.selectedFile?.id) return;
  if (!loadedModel.value) {
    message.value = { type: "error", text: t.value.autoTagModal.noModels };
    return;
  }

  isDetecting.value = true;
  message.value = null;
  try {
    const results = await invoke<TagPrediction[]>("auto_tag_file", {
      fileId: props.selectedFile.id,
      config: taggerConfig.value,
      applyTags: false,
    });
    predictions.value = results;
    if (results.length === 0) {
      message.value = { type: "success", text: t.value.autoTagModal.noTagsDetected };
    }
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isDetecting.value = false;
  }
}

async function applyToCurrent() {
  if (!props.selectedFile?.id) return;
  isApplying.value = true;
  message.value = null;
  try {
    const results = await invoke<TagPrediction[]>("auto_tag_file", {
      fileId: props.selectedFile.id,
      config: taggerConfig.value,
      applyTags: true,
    });
    predictions.value = results;
    message.value = {
      type: "success",
      text: `${t.value.autoTagModal.success} (${results.length} tags)`,
    };
    emit("tags-applied");
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isApplying.value = false;
  }
}

async function applyToBatch() {
  if (props.selectedFileIds.length === 0) return;
  isApplying.value = true;
  message.value = null;
  try {
    const result = await invoke<BatchTagResult>("batch_auto_tag_files", {
      fileIds: props.selectedFileIds,
      config: taggerConfig.value,
    });
    message.value = {
      type: "success",
      text: `${t.value.autoTagModal.success} (${result.processed_files} files, ${result.tags_added} tags)`,
    };
    emit("tags-applied");
  } catch (err: any) {
    message.value = { type: "error", text: String(err) };
  } finally {
    isApplying.value = false;
  }
}

function getFileName(path: string) {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
}

function getTagCategoryClass(cat: string | number) {
  if (cat === "Character" || cat === 4) return "tag-badge-character";
  if (cat === "Rating" || cat === 9) return "tag-badge-rating";
  return "tag-badge-general";
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.show) {
    emit("close");
  }
}

watch(
  () => props.show,
  (val) => {
    if (val) {
      message.value = null;
      predictions.value = [];
      void loadModelList();
    }
  },
);

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div v-if="show" class="modal-backdrop" @click.self="emit('close')" v-dialog="() => emit('close')">
    <div class="modal-dialog">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-left">
          <span class="header-icon">🏷️</span>
          <h2 class="modal-title">{{ t.autoTagModal.title }}</h2>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Body -->
      <div class="modal-body">
        <!-- Model Selector & Status -->
        <div class="section-box">
          <div class="section-title-row">
            <span class="section-title">{{ t.autoTagModal.modelLabel }}</span>
            <span v-if="loadedModel" class="model-status-pill online">
              ● {{ t.autoTagModal.modelLoaded }}: {{ loadedModel.name }}
            </span>
            <span v-else class="model-status-pill offline">
              ○ Not Loaded
            </span>
          </div>

          <div class="model-select-row">
            <select
              v-if="models.length > 0"
              v-model="selectedModelPath"
              class="model-dropdown"
              @change="() => {
                const target = models.find(m => m.model_path === selectedModelPath);
                if (target) onSelectModel(target);
              }"
            >
              <option v-for="m in models" :key="m.model_path" :value="m.model_path">
                {{ m.name }} {{ m.is_loaded ? `(★ ${t.autoTagModal.modelLoaded})` : '' }}
              </option>
            </select>
            <button
              type="button"
              class="browse-btn"
              :title="t.autoTagModal.browseFolder"
              @click="browseModelFolder"
            >
              📁 {{ t.autoTagModal.browseFolder }}
            </button>
          </div>

          <p v-if="models.length === 0" class="empty-help-text">
            ℹ️ {{ t.autoTagModal.noModels }}
          </p>
        </div>

        <!-- Threshold & Configuration Sliders -->
        <div class="section-box">
          <div class="controls-grid">
            <!-- General Tag Threshold -->
            <div class="control-item">
              <div class="control-header">
                <label for="general-slider">{{ t.autoTagModal.generalThreshold }}</label>
                <span class="val-badge">≥ {{ generalThreshold }}%</span>
              </div>
              <input
                id="general-slider"
                type="range"
                min="10"
                max="90"
                step="5"
                v-model.number="generalThreshold"
                class="range-slider"
              />
            </div>

            <!-- Character Tag Threshold -->
            <div class="control-item">
              <div class="control-header">
                <label for="character-slider">{{ t.autoTagModal.characterThreshold }}</label>
                <span class="val-badge green">≥ {{ characterThreshold }}%</span>
              </div>
              <input
                id="character-slider"
                type="range"
                min="50"
                max="95"
                step="5"
                v-model.number="characterThreshold"
                class="range-slider"
              />
            </div>

            <!-- Content Rating Checkbox -->
            <div class="control-item-row">
              <label class="checkbox-label">
                <input type="checkbox" v-model="includeRating" />
                <span>{{ t.autoTagModal.includeRating }}</span>
              </label>

              <!-- Max Tags -->
              <div class="max-tags-group">
                <label>{{ t.autoTagModal.maxTags }}:</label>
                <select v-model.number="maxTags" class="small-select">
                  <option :value="30">30</option>
                  <option :value="50">50</option>
                  <option :value="80">80</option>
                  <option :value="150">150</option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- Feedback Messages -->
        <div v-if="message" :class="['message-banner', message.type]">
          <span>{{ message.text }}</span>
        </div>

        <!-- Preview of Detected Tags -->
        <div v-if="selectedFile" class="section-box preview-box">
          <div class="preview-header">
            <div class="preview-file-info">
              <span class="file-name">{{ getFileName(selectedFile.path) }}</span>
              <span v-if="predictions.length > 0" class="count-badge">
                {{ predictions.length }} {{ t.tagsModal.existingTags }}
              </span>
            </div>
            <button
              type="button"
              class="detect-btn"
              :disabled="isDetecting || !loadedModel"
              @click="detectTags"
            >
              {{ isDetecting ? t.autoTagModal.tagging : `🔍 ${t.autoTagModal.detectTags}` }}
            </button>
          </div>

          <div v-if="predictions.length > 0" class="tags-container">
            <div
              v-for="tag in predictions"
              :key="tag.name"
              :class="['tag-pill', getTagCategoryClass(tag.category)]"
            >
              <span class="tag-name">{{ tag.name }}</span>
              <span class="tag-score">{{ Math.round(tag.confidence * 100) }}%</span>
            </div>
          </div>
          <div v-else-if="!isDetecting" class="no-tags-prompt">
            {{ t.autoTagModal.noTagsDetected }}
          </div>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="modal-footer">
        <button type="button" class="btn-cancel" @click="emit('close')">
          {{ t.preview.close }}
        </button>

        <div class="footer-actions-right">
          <!-- Batch Apply Button -->
          <button
            v-if="selectedFileCount > 1"
            type="button"
            class="btn-primary batch-btn"
            :disabled="isApplying || isDetecting || !loadedModel"
            @click="applyToBatch"
          >
            {{
              isApplying
                ? t.autoTagModal.tagging
                : t.autoTagModal.applyToBatch.replace('{count}', String(selectedFileCount))
            }}
          </button>

          <!-- Current File Apply Button -->
          <button
            v-if="selectedFile"
            type="button"
            class="btn-primary"
            :disabled="isApplying || isDetecting || !loadedModel"
            @click="applyToCurrent"
          >
            {{ isApplying ? t.autoTagModal.tagging : t.autoTagModal.applyToCurrent }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1050;
  animation: fadeIn 0.15s ease;
}

.modal-dialog {
  background: #181926;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 90%;
  max-width: 620px;
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.02);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-icon {
  font-size: 1.25rem;
}

.modal-title {
  font-size: 1.05rem;
  font-weight: 700;
  color: #f1f5f9;
  margin: 0;
}

.close-btn {
  background: transparent;
  border: none;
  color: #94a3b8;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.15s;
}

.close-btn:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-box {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 8px;
  padding: 12px 14px;
}

.section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.section-title {
  font-size: 0.8rem;
  font-weight: 600;
  color: #cbd5e1;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.model-status-pill {
  font-size: 0.72rem;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
}

.model-status-pill.online {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
  border: 1px solid rgba(16, 185, 129, 0.3);
}

.model-status-pill.offline {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.model-select-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.model-dropdown {
  flex: 1;
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #f8fafc;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 0.82rem;
  outline: none;
}

.browse-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.18);
  color: #e2e8f0;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.browse-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #fff;
}

.empty-help-text {
  font-size: 0.75rem;
  color: #94a3b8;
  margin-top: 8px;
  line-height: 1.4;
}

.controls-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.control-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.control-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.78rem;
  color: #cbd5e1;
}

.val-badge {
  font-size: 0.72rem;
  font-weight: 700;
  color: #60a5fa;
  background: rgba(96, 165, 250, 0.15);
  padding: 1px 6px;
  border-radius: 4px;
}

.val-badge.green {
  color: #34d399;
  background: rgba(52, 211, 153, 0.15);
}

.range-slider {
  width: 100%;
  accent-color: #6366f1;
  height: 4px;
  cursor: pointer;
}

.control-item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 4px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.78rem;
  color: #cbd5e1;
  cursor: pointer;
}

.max-tags-group {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.78rem;
  color: #cbd5e1;
}

.small-select {
  background: #0f172a;
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #f8fafc;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.76rem;
}

.preview-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.preview-file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  overflow: hidden;
}

.file-name {
  font-size: 0.8rem;
  font-weight: 600;
  color: #f8fafc;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 250px;
}

.count-badge {
  font-size: 0.7rem;
  color: #a5b4fc;
  background: rgba(99, 102, 241, 0.15);
  padding: 1px 6px;
  border-radius: 4px;
  white-space: nowrap;
}

.detect-btn {
  background: rgba(99, 102, 241, 0.2);
  border: 1px solid rgba(99, 102, 241, 0.4);
  color: #c7d2fe;
  padding: 4px 10px;
  border-radius: 5px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.detect-btn:hover:not(:disabled) {
  background: rgba(99, 102, 241, 0.4);
  color: #fff;
}

.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  max-height: 160px;
  overflow-y: auto;
  padding-right: 4px;
}

.tag-pill {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 0.72rem;
  border: 1px solid transparent;
}

.tag-badge-general {
  background: rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
  color: #e0e7ff;
}

.tag-badge-character {
  background: rgba(16, 185, 129, 0.15);
  border-color: rgba(16, 185, 129, 0.3);
  color: #a7f3d0;
}

.tag-badge-rating {
  background: rgba(168, 85, 247, 0.15);
  border-color: rgba(168, 85, 247, 0.3);
  color: #f3e8ff;
}

.tag-score {
  font-weight: 700;
  opacity: 0.8;
}

.no-tags-prompt {
  font-size: 0.75rem;
  color: #64748b;
  text-align: center;
  padding: 12px 0;
}

.message-banner {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 600;
}

.message-banner.success {
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #34d399;
}

.message-banner.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.02);
}

.footer-actions-right {
  display: flex;
  gap: 8px;
}

.btn-cancel {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #94a3b8;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-cancel:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.05);
}

.btn-primary {
  background: #6366f1;
  border: none;
  color: #fff;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary:hover:not(:disabled) {
  background: #4f46e5;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.batch-btn {
  background: #8b5cf6;
}

.batch-btn:hover:not(:disabled) {
  background: #7c3aed;
}
</style>
