<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { DetectedLora, ImageFile, Tag } from "../types";
import { assetUrl, formatBytes, formatDuration, formatPlatformName, getFileName, normalizePath } from "../utils/image";
import { getThumbnailUrl } from "../utils/thumbnail";
import { t } from "../i18n";
import {
  extractWorkflowJson,
  hasComfyWorkflow,
  hasPromptData,
  sendPromptToWebUI,
  sendWorkflowToComfyUI,
} from "../utils/generation";
import { loadAppConfig } from "../utils/config";

const props = defineProps<{
  file: ImageFile | null;
  selectedCount?: number;
}>();

const emit = defineEmits<{
  close: [];
  openLightbox: [file: ImageFile];
  openTagModal: [fileId: number];
  openAlbumModal: [fileId: number];
  updateFile: [file: ImageFile];
  filterByModel: [model: string];
  filterByHash: [hash: string];
  findSimilar: [file: ImageFile];
  openAutoTagModal: [file: ImageFile];
  openLoraManager: [];
  registerLora: [name: string, weight: number];
}>();

defineExpose({
  loadTags,
  loadDetectedLoras,
});

const fileTags = ref<Tag[]>([]);
const detectedLoras = ref<DetectedLora[]>([]);
const triggerCopied = ref<string | null>(null);
const promptCopied = ref(false);
const negativePromptCopied = ref(false);
const seedCopied = ref(false);
const rawCopied = ref(false);
const showRaw = ref(false);
const revealedNsfw = ref(false);
const thumbUrl = ref("");
const sendingComfy = ref(false);
const sendingWebui = ref(false);
const comfyStatusMsg = ref<string | null>(null);
const webuiStatusMsg = ref<string | null>(null);
const workflowCopied = ref(false);

watch(
  () => props.file,
  async (newFile) => {
    revealedNsfw.value = false;
    if (newFile) {
      thumbUrl.value = assetUrl(newFile.path); // instant fallback
      thumbUrl.value = await getThumbnailUrl(newFile);
    } else {
      thumbUrl.value = "";
    }
    await Promise.all([loadTags(), loadDetectedLoras()]);
  },
  { immediate: true },
);

let triggerCopyTimeout: any = null;
async function copyTriggerText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    triggerCopied.value = text;
    if (triggerCopyTimeout) clearTimeout(triggerCopyTimeout);
    triggerCopyTimeout = setTimeout(() => {
      triggerCopied.value = null;
    }, 2000);
  } catch {
    //
  }
}

async function loadDetectedLoras() {
  if (!props.file?.id) {
    detectedLoras.value = [];
    return;
  }
  try {
    detectedLoras.value = await invoke<DetectedLora[]>("get_image_detected_loras", {
      fileId: props.file.id,
    });
  } catch (e) {
    console.error("Failed to load detected loras:", e);
    detectedLoras.value = [];
  }
}

async function loadTags() {
  if (!props.file?.id) {
    fileTags.value = [];
    return;
  }
  try {
    fileTags.value = await invoke<Tag[]>("get_file_tags", { fileId: props.file.id });
  } catch (e) {
    console.error("Failed to load file tags:", e);
    fileTags.value = [];
  }
}

async function removeTag(tagId: number) {
  if (!props.file?.id) return;
  try {
    await invoke("untag_file", { fileId: props.file.id, tagId });
    await loadTags();
  } catch (e) {
    console.error("Failed to remove tag:", e);
  }
}

async function setRating(rating: number) {
  if (!props.file) return;
  const newRating = props.file.rating === rating ? 0 : rating;
  try {
    await invoke("update_file_rating", { path: props.file.path, rating: newRating });
    emit("updateFile", { ...props.file, rating: newRating });
  } catch (e) {
    console.error("Failed to update rating:", e);
  }
}

async function toggleFavorite() {
  if (!props.file) return;
  const isFavorite = !props.file.is_favorite;
  try {
    await invoke("update_file_favorite", { path: props.file.path, isFavorite });
    emit("updateFile", { ...props.file, is_favorite: isFavorite });
  } catch (e) {
    console.error("Failed to toggle favorite:", e);
  }
}

async function toggleNsfw() {
  if (!props.file) return;
  const isNsfw = !props.file.is_nsfw;
  try {
    await invoke("update_file_nsfw", { path: props.file.path, isNsfw });
    emit("updateFile", { ...props.file, is_nsfw: isNsfw });
  } catch (e) {
    console.error("Failed to toggle nsfw:", e);
  }
}

async function copyText(text: string, type: "prompt" | "negative" | "seed" | "raw") {
  try {
    await navigator.clipboard.writeText(text);
    if (type === "prompt") {
      promptCopied.value = true;
      setTimeout(() => (promptCopied.value = false), 2000);
    } else if (type === "negative") {
      negativePromptCopied.value = true;
      setTimeout(() => (negativePromptCopied.value = false), 2000);
    } else if (type === "seed") {
      seedCopied.value = true;
      setTimeout(() => (seedCopied.value = false), 2000);
    } else if (type === "raw") {
      rawCopied.value = true;
      setTimeout(() => (rawCopied.value = false), 2000);
    }
  } catch (err) {
    console.error("Failed to copy text: ", err);
  }
}

const promptTokens = computed(() => {
  const p = props.file?.metadata?.prompt;
  if (!p) return [];
  return p
    .split(/[,|\n]/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
});

async function handleSendToComfyUI() {
  if (!props.file) return;
  const json = extractWorkflowJson(props.file);
  if (!json) {
    comfyStatusMsg.value = t.value.interop.noWorkflow;
    setTimeout(() => { comfyStatusMsg.value = null; }, 3000);
    return;
  }
  sendingComfy.value = true;
  comfyStatusMsg.value = null;
  try {
    const cfg = await loadAppConfig();
    const endpoint = cfg.comfyui_url || "http://127.0.0.1:8188";
    const res = await sendWorkflowToComfyUI(endpoint, json);
    if (res.prompt_id) {
      comfyStatusMsg.value = `${t.value.interop.queued} (#${res.prompt_id.slice(0, 8)})`;
    } else {
      comfyStatusMsg.value = t.value.interop.sentSuccess;
    }
  } catch (err) {
    comfyStatusMsg.value = `${t.value.interop.failed}: ${String(err)}`;
  } finally {
    sendingComfy.value = false;
    setTimeout(() => { comfyStatusMsg.value = null; }, 5000);
  }
}

async function handleSendToWebUI() {
  if (!props.file) return;
  if (!hasPromptData(props.file)) {
    webuiStatusMsg.value = t.value.interop.noPrompt;
    setTimeout(() => { webuiStatusMsg.value = null; }, 3000);
    return;
  }
  sendingWebui.value = true;
  webuiStatusMsg.value = null;
  try {
    const cfg = await loadAppConfig();
    const endpoint = cfg.webui_url || "http://127.0.0.1:7860";
    await sendPromptToWebUI(endpoint, props.file);
    webuiStatusMsg.value = t.value.interop.sentSuccess;
  } catch (err) {
    webuiStatusMsg.value = `${t.value.interop.failed}: ${String(err)}`;
  } finally {
    sendingWebui.value = false;
    setTimeout(() => { webuiStatusMsg.value = null; }, 5000);
  }
}

async function handleCopyWorkflowJson() {
  if (!props.file) return;
  const json = extractWorkflowJson(props.file);
  if (!json) return;
  try {
    await navigator.clipboard.writeText(json);
    workflowCopied.value = true;
    setTimeout(() => { workflowCopied.value = false; }, 2000);
  } catch (err) {
    console.warn("Failed to copy workflow JSON:", err);
  }
}

function onWorkflowDragStart(e: DragEvent) {
  if (!props.file || !e.dataTransfer) return;
  const json = extractWorkflowJson(props.file);
  if (json) {
    e.dataTransfer.setData("application/json", json);
    e.dataTransfer.setData("text/plain", json);
    e.dataTransfer.effectAllowed = "copy";
  }
}
</script>

<template>
  <aside class="inspector-eagle">
    <!-- Header -->
    <div class="inspector-header">
      <div class="header-left">
        <span class="inspector-title">{{ t.preview.inspector }}</span>
        <span v-if="selectedCount && selectedCount > 1" class="multi-badge">
          {{ selectedCount }} {{ t.batch.selectedCount }}
        </span>
      </div>
      <button
        type="button"
        class="close-btn"
        :title="`${t.preview.close} (I)`"
        @click="emit('close')"
      >
        <svg viewBox="0 0 16 16" width="12" height="12" fill="currentColor">
          <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z"/>
        </svg>
      </button>
    </div>

    <!-- Empty State -->
    <div v-if="!file" class="empty-inspector">
      <span class="empty-icon">🖼️</span>
      <p class="empty-text">{{ t.preview.noSelection }}</p>
      <span class="empty-sub">{{ t.preview.noSelectionSub }}</span>
    </div>

    <!-- Content when file selected -->
    <div v-else class="inspector-scrollable">
      <!-- Thumbnail & Quick Lightbox Trigger (Eagle Style) -->
      <div class="preview-card" @click="emit('openLightbox', file)">
        <div class="img-wrapper">
          <img
            :src="thumbUrl || assetUrl(file.path)"
            :alt="getFileName(file.path)"
            :class="{ blurred: file.is_nsfw && !revealedNsfw }"
            loading="lazy"
          />
          <div
            v-if="file.is_nsfw && !revealedNsfw"
            class="nsfw-overlay"
            @click.stop="revealedNsfw = true"
          >
            <span>{{ t.preview.clickToReveal }}</span>
          </div>
          <div class="expand-badge" :title="`${t.preview.preview} (Space / Enter)`">
            {{ t.preview.preview }}
          </div>
        </div>

        <div class="file-main-info">
          <h4 class="file-name" :title="getFileName(file.path)">
            {{ getFileName(file.path) }}
          </h4>
          <div class="file-meta-tags">
            <span class="meta-tag uppercase">{{ formatPlatformName(file.metadata?.format) || file.container }}</span>
            <span v-if="file.metadata?.width && file.metadata?.height" class="meta-tag">
              {{ file.metadata.width }} × {{ file.metadata.height }}
            </span>
            <span v-if="file.metadata?.duration_seconds" class="meta-tag duration-tag">
              ⏱ {{ formatDuration(file.metadata.duration_seconds) }}
            </span>
            <span v-if="file.metadata?.fps" class="meta-tag">
              {{ Math.round(file.metadata.fps) }} fps
            </span>
            <span v-if="file.metadata?.video_codec" class="meta-tag uppercase">
              {{ file.metadata.video_codec }}
            </span>
            <span v-if="file.size_bytes" class="meta-tag">
              {{ formatBytes(file.size_bytes) }}
            </span>
          </div>
        </div>
      </div>

      <!-- Quick Actions (Rating, Fav, NSFW, Album, Tag) -->
      <div class="quick-actions-bar">
        <!-- Stars -->
        <div class="rating-stars" :title="t.preview.ratingShortcut">
          <button
            v-for="star in 5"
            :key="star"
            type="button"
            class="star-btn"
            :class="{ active: (file.rating ?? 0) >= star }"
            @click="setRating(star)"
          >
            ★
          </button>
        </div>

        <div class="action-btn-group">
          <!-- Favorite -->
          <button
            type="button"
            class="action-btn"
            :class="{ active: file.is_favorite }"
            :title="file.is_favorite ? t.preview.removeFavorite : t.preview.addFavorite"
            @click="toggleFavorite"
          >
            {{ file.is_favorite ? '❤️' : '🤍' }}
          </button>

          <!-- NSFW -->
          <button
            type="button"
            class="action-btn"
            :class="{ active: file.is_nsfw }"
            :title="file.is_nsfw ? t.preview.markSfw : t.preview.markNsfw"
            @click="toggleNsfw"
          >
            🔞
          </button>

          <!-- Album -->
          <button
            type="button"
            class="action-btn"
            :title="t.preview.addToAlbum"
            @click="file.id && emit('openAlbumModal', file.id)"
          >
            🗂️
          </button>

          <!-- Tag -->
          <button
            type="button"
            class="action-btn"
            :title="t.preview.addTag"
            @click="file.id && emit('openTagModal', file.id)"
          >
            🏷️
          </button>

          <!-- Find Similar -->
          <button
            type="button"
            class="action-btn"
            :title="t.preview.findSimilar"
            @click="file && emit('findSimilar', file)"
          >
            🔍
          </button>

          <!-- Auto Tag (WD14) -->
          <button
            type="button"
            class="action-btn"
            :title="t.preview.autoTag"
            @click="file && emit('openAutoTagModal', file)"
          >
            🤖
          </button>
        </div>
      </div>

      <!-- Tags Section -->
      <div v-if="fileTags.length > 0" class="section tags-section">
        <div class="section-header">
          <span class="section-title">{{ t.nav.tags }}</span>
        </div>
        <div class="tag-chips">
          <span
            v-for="tag in fileTags"
            :key="tag.id"
            class="tag-pill"
            :style="{ borderColor: tag.color || '#8b5cf6' }"
          >
            <span class="tag-dot" :style="{ backgroundColor: tag.color || '#8b5cf6' }"></span>
            {{ tag.name }}
            <button
              type="button"
              class="tag-del-btn"
              @click="removeTag(tag.id)"
            >
              ×
            </button>
          </span>
        </div>
      </div>

      <!-- Generation Interop Section -->
      <div v-if="hasComfyWorkflow(file) || hasPromptData(file)" class="section interop-section">
        <div class="section-header">
          <span class="section-title">🔌 {{ t.interop.title }}</span>
        </div>

        <div class="interop-button-group">
          <!-- Send to ComfyUI -->
          <button
            v-if="hasComfyWorkflow(file)"
            type="button"
            class="interop-action-btn comfy-style"
            :disabled="sendingComfy"
            :title="t.interop.sendToComfyUI"
            @click="handleSendToComfyUI"
          >
            <span>⚡ {{ sendingComfy ? t.interop.checking : t.interop.sendToComfyUI }}</span>
          </button>

          <!-- Send to SD WebUI -->
          <button
            v-if="hasPromptData(file)"
            type="button"
            class="interop-action-btn webui-style"
            :disabled="sendingWebui"
            :title="t.interop.sendToWebUI"
            @click="handleSendToWebUI"
          >
            <span>🎨 {{ sendingWebui ? t.interop.checking : t.interop.sendToWebUI }}</span>
          </button>
        </div>

        <!-- Draggable Workflow Chip -->
        <div
          v-if="hasComfyWorkflow(file)"
          class="workflow-drag-pill"
          draggable="true"
          :title="t.interop.dragWorkflow"
          @dragstart="onWorkflowDragStart"
          @click="handleCopyWorkflowJson"
        >
          <span>🧩 {{ workflowCopied ? t.preview.copied : t.interop.dragWorkflow }}</span>
          <span class="drag-hint">⋮⋮</span>
        </div>

        <!-- Feedback message -->
        <div v-if="comfyStatusMsg" class="interop-feedback" :class="{ error: comfyStatusMsg.includes('Failed') }">
          {{ comfyStatusMsg }}
        </div>
        <div v-if="webuiStatusMsg" class="interop-feedback" :class="{ error: webuiStatusMsg.includes('Failed') }">
          {{ webuiStatusMsg }}
        </div>
      </div>

      <!-- Prompts Section -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">{{ t.preview.prompt }}</span>
          <button
            v-if="file.metadata?.prompt"
            type="button"
            class="copy-btn"
            @click="copyText(file.metadata.prompt, 'prompt')"
          >
            {{ promptCopied ? t.preview.copied : t.preview.copyPrompt }}
          </button>
        </div>
        <div v-if="file.metadata?.prompt" class="prompt-box">
          <p class="prompt-text">{{ file.metadata.prompt }}</p>
          <div v-if="promptTokens.length > 0" class="token-chips">
            <span v-for="(token, i) in promptTokens.slice(0, 16)" :key="i" class="token-chip">
              {{ token }}
            </span>
            <span v-if="promptTokens.length > 16" class="token-more">
              +{{ promptTokens.length - 16 }}
            </span>
          </div>
        </div>
        <p v-else class="empty-field">—</p>
      </div>

      <!-- Negative Prompt Section -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">{{ t.preview.negativePrompt }}</span>
          <button
            v-if="file.metadata?.negative_prompt"
            type="button"
            class="copy-btn"
            @click="copyText(file.metadata.negative_prompt, 'negative')"
          >
            {{ negativePromptCopied ? t.preview.copied : t.preview.copyNegative }}
          </button>
        </div>
        <div v-if="file.metadata?.negative_prompt" class="prompt-box negative">
          <p class="prompt-text">{{ file.metadata.negative_prompt }}</p>
        </div>
        <p v-else class="empty-field">—</p>
      </div>

      <!-- LoRA Models & Trigger Words Section -->
      <div v-if="detectedLoras.length > 0" class="section lora-section">
        <div class="section-header">
          <div class="section-title-wrap">
            <span class="lora-header-icon">🎨</span>
            <span class="section-title">{{ t.loraModal.sectionTitle }}</span>
            <span class="badge-count-sm">{{ detectedLoras.length }}</span>
          </div>
          <button
            type="button"
            class="text-action-btn"
            @click="emit('openLoraManager')"
          >
            {{ t.loraModal.manageLibrary }}
          </button>
        </div>

        <div class="detected-lora-list">
          <div v-for="(lora, idx) in detectedLoras" :key="idx" class="detected-lora-item">
            <div class="detected-lora-header">
              <span class="detected-name" :title="lora.name">{{ lora.name }}</span>
              <div class="detected-badges">
                <span v-if="lora.hash" class="detected-hash" :title="lora.hash">#{{ lora.hash }}</span>
                <span class="detected-weight">×{{ lora.weight }}</span>
              </div>
            </div>

            <!-- Trigger words if known in model catalog -->
            <div v-if="lora.model?.trigger_words && lora.model.trigger_words.length > 0" class="detected-triggers">
              <span
                v-for="(tw, twIdx) in lora.model.trigger_words"
                :key="twIdx"
                class="detected-trigger-chip"
                :class="{ active: triggerCopied === tw }"
                :title="t.loraModal.clickToCopy"
                @click="copyTriggerText(tw)"
              >
                {{ tw }}
                <span class="copy-hint">📋</span>
              </span>
            </div>
            <div v-else class="detected-unknown">
              <span class="unregistered-text">{{ t.loraModal.notInLibrary }}</span>
              <button
                type="button"
                class="quick-register-btn"
                @click="emit('registerLora', lora.name, lora.weight)"
              >
                + {{ t.loraModal.quickAdd }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Generation Parameters Section -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">{{ t.preview.generationParams }}</span>
        </div>

        <div class="params-grid">
          <!-- Model -->
          <div v-if="file.metadata?.model_name" class="param-row full-width">
            <span class="param-label">{{ t.preview.modelName }}</span>
            <div class="param-value-box">
              <span class="param-val truncate" :title="file.metadata.model_name">
                {{ file.metadata.model_name }}
              </span>
              <button
                type="button"
                class="filter-icon-btn"
                :title="t.modelsModal.filterModel"
                @click="emit('filterByModel', file.metadata!.model_name!)"
              >
                🔍
              </button>
            </div>
          </div>

          <!-- Model Hash -->
          <div v-if="file.metadata?.model_hash" class="param-row">
            <span class="param-label">{{ t.preview.modelHash }}</span>
            <div class="param-value-box">
              <span class="param-val">{{ file.metadata.model_hash }}</span>
              <button
                type="button"
                class="filter-icon-btn"
                :title="t.modelsModal.filterHash"
                @click="emit('filterByHash', file.metadata!.model_hash!)"
              >
                🔍
              </button>
            </div>
          </div>

          <!-- Sampler -->
          <div v-if="file.metadata?.sampler" class="param-row">
            <span class="param-label">{{ t.preview.sampler }}</span>
            <span class="param-val">{{ file.metadata.sampler }}</span>
          </div>

          <!-- Steps -->
          <div v-if="file.metadata?.steps != null" class="param-row">
            <span class="param-label">{{ t.preview.steps }}</span>
            <span class="param-val">{{ file.metadata.steps }}</span>
          </div>

          <!-- CFG Scale -->
          <div v-if="file.metadata?.cfg_scale != null" class="param-row">
            <span class="param-label">{{ t.preview.cfgScale }}</span>
            <span class="param-val">{{ file.metadata.cfg_scale }}</span>
          </div>

          <!-- Seed -->
          <div v-if="file.metadata?.seed != null" class="param-row">
            <span class="param-label">{{ t.preview.seed }}</span>
            <div class="param-value-box">
              <span class="param-val">{{ file.metadata.seed }}</span>
              <button
                type="button"
                class="filter-icon-btn"
                @click="copyText(String(file.metadata!.seed), 'seed')"
              >
                {{ seedCopied ? '✓' : '📋' }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- File Path & Location -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">{{ t.view.files }}</span>
        </div>
        <p class="file-path-text" :title="file.path">{{ normalizePath(file.path) }}</p>
      </div>

      <!-- Raw Metadata / Workflow JSON -->
      <div v-if="file.metadata?.raw" class="section">
        <div class="section-header cursor-pointer" @click="showRaw = !showRaw">
          <span class="section-title">{{ t.preview.copyAll }}</span>
          <div class="header-right-actions">
            <button
              type="button"
              class="copy-btn"
              @click.stop="copyText(file.metadata!.raw!, 'raw')"
            >
              {{ rawCopied ? t.preview.copied : 'Raw' }}
            </button>
            <span class="collapse-icon">{{ showRaw ? '▲' : '▼' }}</span>
          </div>
        </div>
        <pre v-if="showRaw" class="raw-box">{{ file.metadata.raw }}</pre>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.inspector-eagle {
  width: 280px;
  min-width: 280px;
  max-width: 280px;
  flex-shrink: 0;
  height: 100%;
  background: var(--color-bg-primary);
  border-left: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  user-select: none;
  overflow: hidden;
}

.inspector-header {
  height: 42px;
  min-height: 42px;
  padding: 0 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.inspector-title {
  font-size: 0.8rem;
  font-weight: 700;
  color: #f1f5f9;
}

.multi-badge {
  font-size: 0.68rem;
  padding: 1px 6px;
  border-radius: 999px;
  background: rgba(139, 92, 246, 0.2);
  color: #c4b5fd;
  border: 1px solid rgba(139, 92, 246, 0.3);
}

.close-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #f8fafc;
}

.empty-inspector {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 30px 20px;
  text-align: center;
  gap: 8px;
}

.empty-icon {
  font-size: 2.2rem;
  opacity: 0.3;
}

.empty-text {
  font-size: 0.88rem;
  font-weight: 600;
  color: #94a3b8;
  margin: 0;
}

.empty-sub {
  font-size: 0.74rem;
  color: #475569;
  line-height: 1.4;
}

.inspector-scrollable {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.inspector-scrollable::-webkit-scrollbar {
  width: 4px;
}
.inspector-scrollable::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
}

.preview-card {
  border-radius: 8px;
  background: #202024;
  border: 1px solid rgba(255, 255, 255, 0.06);
  overflow: hidden;
  cursor: pointer;
  transition: all 0.15s;
}

.preview-card:hover {
  border-color: rgba(139, 92, 246, 0.4);
}

.img-wrapper {
  position: relative;
  width: 100%;
  height: 170px;
  background: #0f0f12;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.img-wrapper img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  transition: transform 0.2s ease;
}

.preview-card:hover .img-wrapper img {
  transform: scale(1.03);
}

.img-wrapper img.blurred {
  filter: blur(20px);
}

.nsfw-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #f87171;
  font-size: 0.74rem;
  font-weight: 600;
}

.expand-badge {
  position: absolute;
  bottom: 6px;
  right: 6px;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  padding: 2px 7px;
  border-radius: 4px;
  font-size: 0.68rem;
  color: #e2e8f0;
  border: 1px solid rgba(255, 255, 255, 0.12);
  opacity: 0;
  transition: opacity 0.15s;
}

.preview-card:hover .expand-badge {
  opacity: 1;
}

.file-main-info {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-name {
  margin: 0;
  font-size: 0.8rem;
  font-weight: 600;
  color: #f1f5f9;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.meta-tag {
  font-size: 0.66rem;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.04);
  color: #94a3b8;
}

.meta-tag.uppercase {
  text-transform: uppercase;
}

.quick-actions-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #202024;
  padding: 5px 8px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.rating-stars {
  display: flex;
  gap: 1px;
}

.star-btn {
  background: transparent;
  border: none;
  color: #475569;
  font-size: 1rem;
  cursor: pointer;
  padding: 0 1px;
  transition: all 0.1s;
}

.star-btn:hover {
  transform: scale(1.15);
  color: #fbbf24;
}

.star-btn.active {
  color: #f59e0b;
}

.action-btn-group {
  display: flex;
  gap: 3px;
}

.action-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 5px;
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.12s;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.action-btn.active {
  background: rgba(236, 72, 153, 0.2);
  border-color: rgba(236, 72, 153, 0.4);
}

.section {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-header.cursor-pointer {
  cursor: pointer;
}

.section-title {
  font-size: 0.72rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #64748b;
}

.copy-btn {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 4px;
  font-size: 0.66rem;
  color: #cbd5e1;
  padding: 2px 6px;
  cursor: pointer;
  transition: all 0.12s;
}

.copy-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #ffffff;
}

.prompt-box {
  background: #202024;
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 6px;
  padding: 8px;
  font-size: 0.76rem;
  color: #e2e8f0;
  line-height: 1.45;
  user-select: text;
}

.prompt-box.negative {
  color: #fca5a5;
  border-color: rgba(239, 68, 68, 0.2);
}

.prompt-text {
  margin: 0;
  word-break: break-word;
}

.token-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.token-chip {
  font-size: 0.66rem;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(139, 92, 246, 0.12);
  color: #c4b5fd;
}

.token-more {
  font-size: 0.66rem;
  color: #64748b;
}

.empty-field {
  font-size: 0.72rem;
  color: #475569;
  font-style: italic;
  margin: 0;
}

.params-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 5px;
}

.param-row {
  display: flex;
  flex-direction: column;
  gap: 1px;
  background: #202024;
  padding: 5px 7px;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.param-row.full-width {
  grid-column: span 2;
}

.param-label {
  font-size: 0.64rem;
  color: #64748b;
  text-transform: uppercase;
  font-weight: 600;
}

.param-value-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 4px;
}

.param-val {
  font-size: 0.76rem;
  color: #e2e8f0;
  font-weight: 500;
}

.param-val.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.filter-icon-btn {
  background: transparent;
  border: none;
  font-size: 0.7rem;
  cursor: pointer;
  padding: 0 2px;
  opacity: 0.6;
  transition: opacity 0.1s;
}

.filter-icon-btn:hover {
  opacity: 1;
}

.tag-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid;
  color: #f1f5f9;
}

.tag-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
}

.tag-del-btn {
  background: transparent;
  border: none;
  color: #64748b;
  cursor: pointer;
  font-size: 0.75rem;
  padding: 0;
  display: flex;
  align-items: center;
}

.tag-del-btn:hover {
  color: #ef4444;
}

.file-path-text {
  font-size: 0.7rem;
  font-family: monospace;
  color: #64748b;
  word-break: break-all;
  margin: 0;
  background: #202024;
  padding: 5px 7px;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.header-right-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.collapse-icon {
  font-size: 0.64rem;
  color: #64748b;
}

.raw-box {
  margin: 0;
  padding: 8px;
  background: #111114;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.05);
  color: #94a3b8;
  font-family: monospace;
  font-size: 0.66rem;
  max-height: 160px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
  user-select: text;
}

/* LoRA Section in Inspector */
.lora-section {
  background: rgba(236, 72, 153, 0.03);
  border: 1px solid rgba(236, 72, 153, 0.15);
  border-radius: 8px;
  padding: 10px;
}

.section-title-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}

.lora-header-icon {
  font-size: 0.85rem;
}

.badge-count-sm {
  background: rgba(236, 72, 153, 0.2);
  color: #f472b6;
  font-size: 0.65rem;
  padding: 1px 5px;
  border-radius: 10px;
  font-weight: 600;
}

.text-action-btn {
  background: none;
  border: none;
  color: #ec4899;
  font-size: 0.72rem;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
}

.text-action-btn:hover {
  background: rgba(236, 72, 153, 0.1);
  text-decoration: underline;
}

.detected-lora-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 6px;
}

.detected-lora-item {
  background: #1e1e26;
  border: 1px solid #333342;
  border-radius: 6px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detected-lora-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.detected-name {
  font-weight: 600;
  font-size: 0.8rem;
  color: #f1f5f9;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 170px;
}

.detected-badges {
  display: flex;
  align-items: center;
  gap: 6px;
}

.detected-hash {
  font-size: 0.7rem;
  color: #94a3b8;
  background: rgba(148, 163, 184, 0.12);
  padding: 1px 4px;
  border-radius: 3px;
  font-family: monospace;
  max-width: 90px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detected-weight {
  font-size: 0.72rem;
  color: #fbbf24;
  background: rgba(251, 191, 36, 0.12);
  padding: 1px 5px;
  border-radius: 3px;
  font-family: monospace;
}

.detected-triggers {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.detected-trigger-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  background: #16161e;
  border: 1px solid #3a3a4c;
  color: #cbd5e1;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.72rem;
  cursor: pointer;
  transition: all 0.12s ease;
}

.detected-trigger-chip:hover {
  background: #272738;
  border-color: #ec4899;
  color: #f472b6;
}

.detected-trigger-chip.active {
  background: #ec4899;
  color: #fff;
  border-color: #ec4899;
}

.copy-hint {
  font-size: 0.65rem;
  opacity: 0.5;
}

.detected-trigger-chip:hover .copy-hint {
  opacity: 1;
}

.detected-unknown {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.7rem;
}

.unregistered-text {
  color: #71717a;
  font-style: italic;
}

.quick-register-btn {
  background: rgba(236, 72, 153, 0.12);
  border: 1px solid rgba(236, 72, 153, 0.3);
  color: #f472b6;
  font-size: 0.68rem;
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
}

.quick-register-btn:hover {
  background: rgba(236, 72, 153, 0.25);
}

.interop-section {
  background: rgba(255, 255, 255, 0.02);
  border-radius: 6px;
  padding: 8px 10px;
  margin-bottom: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.interop-button-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 6px;
}

.interop-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 6px 10px;
  font-size: 0.74rem;
  font-weight: 500;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
  user-select: none;
}

.interop-action-btn:active:not(:disabled) {
  transform: scale(0.98);
}

.interop-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.comfy-style {
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.35);
  color: #93c5fd;
}

.comfy-style:hover:not(:disabled) {
  background: rgba(59, 130, 246, 0.25);
  border-color: rgba(59, 130, 246, 0.5);
}

.webui-style {
  background: rgba(168, 85, 247, 0.15);
  border: 1px solid rgba(168, 85, 247, 0.35);
  color: #d8b4fe;
}

.webui-style:hover:not(:disabled) {
  background: rgba(168, 85, 247, 0.25);
  border-color: rgba(168, 85, 247, 0.5);
}

.workflow-drag-pill {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 6px;
  padding: 5px 8px;
  font-size: 0.7rem;
  border-radius: 5px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px dashed rgba(255, 255, 255, 0.2);
  color: #cbd5e1;
  cursor: grab;
  user-select: none;
  transition: background 0.15s ease, border-color 0.15s ease;
}

.workflow-drag-pill:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.35);
}

.workflow-drag-pill:active {
  cursor: grabbing;
}

.drag-hint {
  font-size: 0.8rem;
  opacity: 0.5;
  letter-spacing: -2px;
}

.interop-feedback {
  margin-top: 6px;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.7rem;
  background: rgba(16, 185, 129, 0.15);
  border: 1px solid rgba(16, 185, 129, 0.3);
  color: #6ee7b7;
}

.interop-feedback.error {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.3);
  color: #fca5a5;
}
</style>
