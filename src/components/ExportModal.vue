<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save, open as openDialog } from "@tauri-apps/plugin-dialog";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { t } from "../i18n";
import type {
  ExportFormat,
  ExportOptions,
  ExportProgressEvent,
  ExportSidecar,
  ExportSummary,
  ImageFile,
  MetadataPrivacyMode,
} from "../types";
import { formatBytes } from "../utils/image";

const props = defineProps<{
  show: boolean;
  files: ImageFile[];
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "exported", summary: ExportSummary): void;
}>();

const format = ref<ExportFormat>("webp");
const quality = ref<number>(85);
const maxEdgeMode = ref<"none" | "3840" | "2048" | "1080" | "custom">("none");
const customMaxEdge = ref<number>(1920);

const privacy = ref<MetadataPrivacyMode>("strip_all_ai_metadata");
const sidecar = ref<ExportSidecar>("none");
const filenameTemplate = ref<string>("{name}");

const asZip = ref<boolean>(true);
const destinationPath = ref<string>("");

const exportHtmlShowcase = ref<boolean>(false);
const htmlTitle = ref<string>("");

const exporting = ref<boolean>(false);
const progress = ref<ExportProgressEvent | null>(null);
const summary = ref<ExportSummary | null>(null);
const error = ref<string | null>(null);

let unlistenProgress: UnlistenFn | null = null;

watch(
  () => props.show,
  async (newVal) => {
    if (newVal) {
      summary.value = null;
      progress.value = null;
      error.value = null;
      exporting.value = false;
      destinationPath.value = "";

      if (!unlistenProgress) {
        unlistenProgress = await listen<ExportProgressEvent>(
          "berry://export-progress",
          (event) => {
            progress.value = event.payload;
          }
        );
      }
    } else {
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
    }
  },
  { immediate: true }
);

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
    unlistenProgress = null;
  }
});

const previewFilename = computed(() => {
  const sample = props.files[0];
  const originalStem = sample
    ? sample.path.split(/[/\\]/).pop()?.replace(/\.[^/.]+$/, "") || "hero"
    : "cyberpunk_hero";

  const ext =
    format.value === "original"
      ? sample?.container || "png"
      : format.value === "jpeg"
      ? "jpg"
      : format.value;

  let res = filenameTemplate.value.trim() || "{name}";
  res = res.replace("{name}", originalStem);
  res = res.replace("{filename}", originalStem);
  res = res.replace("{id}", sample?.id ? String(sample.id) : "42");
  res = res.replace("{index}", "1");
  res = res.replace("{date}", new Date().toISOString().slice(0, 10).replace(/-/g, ""));
  res = res.replace("{rating}", sample?.rating ? String(sample.rating) : "5");
  res = res.replace("{model}", sample?.metadata?.model_name || "dreamshaper");
  res = res.replace("{seed}", sample?.metadata?.seed || "123456789");

  const sanitized = res.replace(/[/\\:*?"<>|\0]/g, "_");
  return `${sanitized}.${ext}`;
});

async function handleBrowseDestination() {
  try {
    if (asZip.value) {
      const defaultName = `berry_export_${new Date().toISOString().slice(0, 10)}.zip`;
      const selected = await save({
        defaultPath: defaultName,
        filters: [{ name: "ZIP Archive", extensions: ["zip"] }],
        title: t.value.exportModal.destinationPath,
      });
      if (selected) {
        destinationPath.value = selected;
      }
    } else {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: t.value.exportModal.destinationPath,
      });
      if (selected && typeof selected === "string") {
        destinationPath.value = selected;
      }
    }
  } catch (err) {
    console.error("Browse export destination error:", err);
  }
}

async function handleStartExport() {
  if (!destinationPath.value.trim() || props.files.length === 0) return;

  exporting.value = true;
  summary.value = null;
  error.value = null;

  let max_edge: number | null = null;
  if (maxEdgeMode.value === "3840") max_edge = 3840;
  else if (maxEdgeMode.value === "2048") max_edge = 2048;
  else if (maxEdgeMode.value === "1080") max_edge = 1080;
  else if (maxEdgeMode.value === "custom" && customMaxEdge.value > 0) {
    max_edge = customMaxEdge.value;
  }

  const options: ExportOptions = {
    file_ids: props.files.map((f) => f.id).filter((id): id is number => typeof id === "number"),
    format: format.value,
    quality: quality.value,
    privacy: privacy.value,
    sidecar: sidecar.value,
    filename_template: filenameTemplate.value.trim() || "{name}",
    destination_path: destinationPath.value.trim(),
    as_zip: asZip.value,
    max_edge,
    export_html_showcase: exportHtmlShowcase.value,
    html_title: htmlTitle.value.trim() || undefined,
  };

  try {
    const res = await invoke<ExportSummary>("export_files_batch", { options });
    summary.value = res;
    emit("exported", res);
  } catch (err: any) {
    error.value = typeof err === "string" ? err : err?.message || String(err);
  } finally {
    exporting.value = false;
  }
}

async function handleOpenOutputFolder() {
  if (!summary.value?.output_path) return;
  try {
    await invoke("reveal_in_file_manager", { path: summary.value.output_path });
  } catch (err) {
    console.error("Reveal in file manager error:", err);
  }
}
</script>

<template>
  <div v-if="show" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-dialog">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-titles">
          <h3 class="modal-title">📤 {{ t.exportModal.title }}</h3>
          <p class="modal-subtitle">
            {{ t.exportModal.itemsCount.replace('{count}', String(files.length)) }}
          </p>
        </div>
        <button type="button" class="close-btn" @click="emit('close')">✕</button>
      </div>

      <!-- Error Alert -->
      <div v-if="error" class="alert-box error">
        <span>⚠️ {{ error }}</span>
      </div>

      <!-- Completed Summary Banner -->
      <div v-if="summary" class="summary-box" :class="{ success: summary.success, error: !summary.success }">
        <div class="summary-header">
          <span class="summary-icon">{{ summary.success ? '✅' : '⚠️' }}</span>
          <span class="summary-text">
            {{ t.exportModal.exportSuccess.replace('{count}', String(summary.total_exported)).replace('{duration}', String(summary.duration_ms)) }}
          </span>
        </div>
        <div class="summary-pills">
          <span class="summary-pill"><strong>{{ summary.total_exported }}</strong> exported</span>
          <span v-if="summary.total_failed > 0" class="summary-pill danger"><strong>{{ summary.total_failed }}</strong> failed</span>
          <span class="summary-pill"><strong>{{ formatBytes(summary.total_bytes_written) }}</strong> written</span>
        </div>
      </div>

      <!-- Modal Body -->
      <div class="modal-body">
        <!-- 1. Format & Quality -->
        <div class="section-card">
          <h4 class="section-title">🎨 {{ t.exportModal.formatSection }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.exportModal.format }}</label>
              <select v-model="format" class="form-select" :disabled="exporting">
                <option value="webp">{{ t.exportModal.formatWebp }}</option>
                <option value="jpeg">{{ t.exportModal.formatJpeg }}</option>
                <option value="png">{{ t.exportModal.formatPng }}</option>
                <option value="original">{{ t.exportModal.formatOriginal }}</option>
              </select>
            </div>

            <div v-if="format === 'webp' || format === 'jpeg'" class="form-group">
              <div class="label-with-value">
                <label class="form-label">{{ t.exportModal.quality }}</label>
                <span class="value-badge">{{ quality }}%</span>
              </div>
              <input
                v-model.number="quality"
                type="range"
                min="10"
                max="100"
                step="5"
                class="form-range"
                :disabled="exporting"
              />
            </div>
          </div>
        </div>

        <!-- 2. Resolution Constraint -->
        <div class="section-card">
          <h4 class="section-title">📐 {{ t.exportModal.dimensionSection }}</h4>
          <div class="grid-2-cols">
            <div class="form-group">
              <label class="form-label">{{ t.exportModal.maxEdge }}</label>
              <select v-model="maxEdgeMode" class="form-select" :disabled="exporting">
                <option value="none">{{ t.exportModal.maxEdgeNone }}</option>
                <option value="3840">{{ t.exportModal.maxEdge4k }}</option>
                <option value="2048">{{ t.exportModal.maxEdge2k }}</option>
                <option value="1080">{{ t.exportModal.maxEdge1080p }}</option>
                <option value="custom">Custom Constraint...</option>
              </select>
            </div>

            <div v-if="maxEdgeMode === 'custom'" class="form-group">
              <label class="form-label">Custom Max Edge (px)</label>
              <input
                v-model.number="customMaxEdge"
                type="number"
                min="64"
                max="16384"
                step="64"
                class="form-input"
                :disabled="exporting"
              />
            </div>
          </div>
        </div>

        <!-- 3. Privacy & Sanitization -->
        <div class="section-card">
          <h4 class="section-title">🛡️ {{ t.exportModal.privacySection }}</h4>
          <div class="form-group">
            <label class="form-label">{{ t.exportModal.privacyMode }}</label>
            <div class="radio-options">
              <label class="radio-card" :class="{ selected: privacy === 'keep_all' }">
                <input v-model="privacy" type="radio" value="keep_all" :disabled="exporting" />
                <div class="radio-content">
                  <span class="radio-title">{{ t.exportModal.privacyKeepAll }}</span>
                  <span class="radio-desc">{{ t.exportModal.privacyKeepAllDesc }}</span>
                </div>
              </label>

              <label class="radio-card" :class="{ selected: privacy === 'strip_prompt_only' }">
                <input v-model="privacy" type="radio" value="strip_prompt_only" :disabled="exporting" />
                <div class="radio-content">
                  <span class="radio-title">{{ t.exportModal.privacyStripPromptOnly }}</span>
                  <span class="radio-desc">{{ t.exportModal.privacyStripPromptOnlyDesc }}</span>
                </div>
              </label>

              <label class="radio-card" :class="{ selected: privacy === 'strip_all_ai_metadata' }">
                <input v-model="privacy" type="radio" value="strip_all_ai_metadata" :disabled="exporting" />
                <div class="radio-content">
                  <span class="radio-title">{{ t.exportModal.privacyStripAllAiMetadata }}</span>
                  <span class="radio-desc">{{ t.exportModal.privacyStripAllAiMetadataDesc }}</span>
                </div>
              </label>

              <label class="radio-card" :class="{ selected: privacy === 'strip_all' }">
                <input v-model="privacy" type="radio" value="strip_all" :disabled="exporting" />
                <div class="radio-content">
                  <span class="radio-title">{{ t.exportModal.privacyStripAll }}</span>
                  <span class="radio-desc">{{ t.exportModal.privacyStripAllDesc }}</span>
                </div>
              </label>
            </div>
          </div>

          <!-- Sidecar -->
          <div class="form-group">
            <label class="form-label">{{ t.exportModal.sidecarSection }}</label>
            <select v-model="sidecar" class="form-select" :disabled="exporting">
              <option value="none">{{ t.exportModal.sidecarNone }}</option>
              <option value="text_prompt">{{ t.exportModal.sidecarTextPrompt }}</option>
              <option value="json_metadata">{{ t.exportModal.sidecarJsonMetadata }}</option>
            </select>
          </div>
        </div>

        <!-- 4. Filename & Destination -->
        <div class="section-card">
          <h4 class="section-title">📦 {{ t.exportModal.packagingSection }}</h4>

          <div class="form-group">
            <label class="form-label">{{ t.exportModal.filenameTemplate }}</label>
            <input
              v-model="filenameTemplate"
              type="text"
              class="form-input mono"
              placeholder="{name}"
              :disabled="exporting"
            />
            <div class="template-meta">
              <span class="template-hint">{{ t.exportModal.templateHint }}</span>
              <span class="template-preview">
                {{ t.exportModal.templatePreview.replace('{preview}', previewFilename) }}
              </span>
            </div>
          </div>

          <div class="form-group">
            <label class="checkbox-label">
              <input v-model="asZip" type="checkbox" :disabled="exporting" />
              <span>{{ t.exportModal.packageAsZip }}</span>
            </label>
          </div>

          <div class="form-group">
            <label class="form-label">{{ t.exportModal.destinationPath }}</label>
            <div class="path-group">
              <input
                v-model="destinationPath"
                type="text"
                class="form-input path-input"
                :placeholder="t.exportModal.noDestination"
                readonly
              />
              <button
                type="button"
                class="btn-browse"
                :disabled="exporting"
                @click="handleBrowseDestination"
              >
                {{ t.exportModal.browseDestination }}
              </button>
            </div>
          </div>
        </div>

        <!-- 5. Standalone HTML Showcase Album -->
        <div class="section-card">
          <h4 class="section-title">🌐 {{ t.exportModal.htmlShowcaseSection }}</h4>
          <div class="form-group">
            <label class="checkbox-label">
              <input v-model="exportHtmlShowcase" type="checkbox" :disabled="exporting" />
              <span>{{ t.exportModal.generateHtmlShowcase }}</span>
            </label>
            <p class="section-desc" style="margin-top: 6px; font-size: 0.8rem; color: #94a3b8; line-height: 1.4;">
              {{ t.exportModal.htmlShowcaseDesc }}
            </p>
          </div>

          <div v-if="exportHtmlShowcase" class="form-group" style="margin-top: 10px;">
            <label class="form-label">{{ t.exportModal.htmlShowcaseTitle }}</label>
            <input
              v-model="htmlTitle"
              type="text"
              class="form-input"
              :placeholder="t.exportModal.defaultAlbumTitle"
              :disabled="exporting"
            />
          </div>
        </div>

        <!-- Live Export Progress Bar -->
        <div v-if="exporting && progress" class="progress-section">
          <div class="progress-info">
            <span class="progress-status">
              {{ t.exportModal.exporting.replace('{current}', String(progress.current)).replace('{total}', String(progress.total)) }}
            </span>
            <span class="progress-percent">
              {{ Math.round((progress.current / Math.max(1, progress.total)) * 100) }}%
            </span>
          </div>
          <div class="progress-track">
            <div
              class="progress-fill"
              :style="{ width: `${(progress.current / Math.max(1, progress.total)) * 100}%` }"
            ></div>
          </div>
          <span class="progress-filename">{{ progress.current_filename }}</span>
        </div>
      </div>

      <!-- Footer Actions -->
      <div class="modal-footer">
        <div class="footer-left">
          <button
            v-if="summary?.success"
            type="button"
            class="btn-action"
            @click="handleOpenOutputFolder"
          >
            📁 {{ t.exportModal.openOutputFolder }}
          </button>
        </div>

        <div class="footer-right">
          <button type="button" class="btn-secondary" :disabled="exporting" @click="emit('close')">
            {{ summary ? t.exportModal.close : t.exportModal.cancel }}
          </button>

          <button
            v-if="!summary"
            type="button"
            class="btn-primary"
            :disabled="!destinationPath.trim() || exporting || files.length === 0"
            @click="handleStartExport"
          >
            {{ exporting ? '⏳ ...' : '🚀 ' + t.exportModal.startExport }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
  padding: 1.5rem;
}

.modal-dialog {
  background: #181920;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  width: 100%;
  max-width: 660px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 48px rgba(0, 0, 0, 0.65);
  color: #e2e8f0;
  overflow: hidden;
}

.modal-header {
  padding: 1.15rem 1.5rem;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.modal-title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 600;
  color: #f8fafc;
}

.modal-subtitle {
  margin: 0.3rem 0 0 0;
  font-size: 0.8rem;
  color: #94a3b8;
}

.close-btn {
  background: none;
  border: none;
  color: #94a3b8;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
}
.close-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: #ffffff;
}

.alert-box {
  margin: 1rem 1.5rem 0 1.5rem;
  padding: 0.75rem 1rem;
  border-radius: 6px;
  font-size: 0.85rem;
}
.alert-box.error {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid rgba(239, 68, 68, 0.35);
  color: #fca5a5;
}

.summary-box {
  margin: 1rem 1.5rem 0 1.5rem;
  padding: 0.85rem 1rem;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}
.summary-box.success {
  background: rgba(52, 211, 153, 0.12);
  border: 1px solid rgba(52, 211, 153, 0.35);
}
.summary-box.error {
  background: rgba(239, 68, 68, 0.12);
  border: 1px solid rgba(239, 68, 68, 0.35);
}

.summary-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  font-size: 0.88rem;
  font-weight: 500;
  color: #6ee7b7;
}

.summary-pills {
  display: flex;
  gap: 0.5rem;
}

.summary-pill {
  background: rgba(0, 0, 0, 0.25);
  border: 1px solid rgba(255, 255, 255, 0.1);
  padding: 0.25rem 0.55rem;
  border-radius: 4px;
  font-size: 0.78rem;
  color: #cbd5e1;
}
.summary-pill.danger {
  color: #f87171;
  border-color: rgba(239, 68, 68, 0.3);
}

.modal-body {
  padding: 1.25rem 1.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 1.15rem;
}

.section-card {
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 8px;
  padding: 1rem 1.15rem;
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
}

.section-title {
  margin: 0;
  font-size: 0.92rem;
  font-weight: 600;
  color: #f1f5f9;
}

.grid-2-cols {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
}

.label-with-value {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.form-label {
  font-size: 0.82rem;
  font-weight: 500;
  color: #cbd5e1;
}

.value-badge {
  font-size: 0.78rem;
  color: #818cf8;
  font-weight: 600;
}

.form-select,
.form-input {
  background: #121317;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  padding: 0.5rem 0.7rem;
  color: #f8fafc;
  font-size: 0.85rem;
  outline: none;
  transition: border-color 0.2s ease;
}
.form-select:focus,
.form-input:focus {
  border-color: #6366f1;
}

.form-input.mono {
  font-family: monospace;
  font-size: 0.82rem;
}

.form-range {
  accent-color: #6366f1;
  height: 6px;
  cursor: pointer;
}

.radio-options {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}

.radio-card {
  display: flex;
  align-items: flex-start;
  gap: 0.65rem;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 6px;
  padding: 0.6rem 0.75rem;
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease;
}
.radio-card:hover {
  background: rgba(255, 255, 255, 0.04);
}
.radio-card.selected {
  border-color: #6366f1;
  background: rgba(99, 102, 241, 0.08);
}

.radio-content {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.radio-title {
  font-size: 0.82rem;
  font-weight: 500;
  color: #f1f5f9;
}

.radio-desc {
  font-size: 0.74rem;
  color: #94a3b8;
  line-height: 1.35;
}

.template-meta {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  margin-top: 0.2rem;
}

.template-hint {
  font-size: 0.72rem;
  color: #64748b;
}

.template-preview {
  font-size: 0.75rem;
  font-family: monospace;
  color: #a5b4fc;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.82rem;
  color: #e2e8f0;
  cursor: pointer;
}

.path-group {
  display: flex;
  gap: 0.5rem;
}

.path-input {
  flex: 1;
}

.btn-browse {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  color: #e2e8f0;
  font-size: 0.82rem;
  padding: 0 0.85rem;
  cursor: pointer;
  white-space: nowrap;
}
.btn-browse:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.14);
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  background: rgba(99, 102, 241, 0.06);
  border: 1px solid rgba(99, 102, 241, 0.2);
  border-radius: 6px;
  padding: 0.75rem 0.9rem;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 0.8rem;
  font-weight: 500;
  color: #c7d2fe;
}

.progress-track {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #6366f1;
  transition: width 0.15s ease;
}

.progress-filename {
  font-size: 0.72rem;
  color: #94a3b8;
  font-family: monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.modal-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.footer-right {
  display: flex;
  gap: 0.75rem;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  color: #cbd5e1;
  border-radius: 6px;
  padding: 0.5rem 1rem;
  font-size: 0.85rem;
  cursor: pointer;
}
.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

.btn-action {
  background: rgba(99, 102, 241, 0.15);
  border: 1px solid rgba(99, 102, 241, 0.3);
  color: #c7d2fe;
  border-radius: 6px;
  padding: 0.5rem 0.9rem;
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
}
.btn-action:hover {
  background: rgba(99, 102, 241, 0.28);
}

.btn-primary {
  background: #4f46e5;
  border: 1px solid #6366f1;
  color: #ffffff;
  border-radius: 6px;
  padding: 0.5rem 1.15rem;
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease;
}
.btn-primary:hover:not(:disabled) {
  background: #4338ca;
}
.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
