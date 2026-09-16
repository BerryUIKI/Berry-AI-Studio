<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { t } from "../i18n";
import type { Folder, PipelineDetectedPath } from "../types";

const emit = defineEmits<{
  (e: "close"): void;
}>();

const step = ref<1 | 2 | 3>(1);
const detectedPaths = ref<PipelineDetectedPath[]>([]);
const selectedDetected = ref<PipelineDetectedPath | null>(null);
const vaultPath = ref("");
const detecting = ref(false);
const submitting = ref(false);
const error = ref("");

onMounted(() => {
  void scanLocalAi();
});

async function scanLocalAi() {
  detecting.value = true;
  try {
    detectedPaths.value = await invoke<PipelineDetectedPath[]>("autodetect_local_ai_paths");
    if (detectedPaths.value.length > 0) {
      selectedDetected.value = detectedPaths.value[0];
    }
  } catch (e) {
    console.warn("Failed to detect AI paths:", e);
  } finally {
    detecting.value = false;
  }
}

async function pickVaultPath() {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string" && selected) {
      vaultPath.value = selected;
    }
  } catch (e) {
    error.value = String(e);
  }
}

async function finishSetup() {
  error.value = "";
  submitting.value = true;
  try {
    // If user configured a pipeline
    if (selectedDetected.value && vaultPath.value) {
      await invoke<Folder>("add_folder_with_options", {
        path: vaultPath.value,
        folderType: "pipeline",
        sourcePath: selectedDetected.value.path,
        ingestAction: "copy",
        gracePeriodHours: 24,
        autoHarvest: true,
      });
    }
  } catch (e) {
    console.warn("Error during onboarding pipeline creation:", e);
  } finally {
    submitting.value = false;
    emit("close");
  }
}

function skip() {
  emit("close");
}
</script>

<template>
  <div class="modal-backdrop" @click.self="skip">
    <div class="modal-dialog">
      <!-- Close button -->
      <button type="button" class="btn-modal-close" aria-label="Close" @click="skip">✕</button>

      <!-- Header -->
      <div class="modal-header">
        <div class="header-badge">🚀 {{ t.onboarding.badge }}</div>
        <h2>{{ t.onboarding.title }}</h2>
        <p class="header-subtitle">{{ t.onboarding.subtitle }}</p>
      </div>

      <div v-if="error" class="modal-error">
        ⚠️ {{ error }}
      </div>

      <div class="modal-body">
        <!-- Step 1: Welcome & Value Prop -->
        <div v-if="step === 1" class="step-pane">
          <div class="feature-highlights">
            <div class="feature-item">
              <div class="feature-icon">⚡</div>
              <div class="feature-info">
                <strong>{{ t.onboarding.feat1Title }}</strong>
                <p>{{ t.onboarding.feat1Desc }}</p>
              </div>
            </div>
            <div class="feature-item">
              <div class="feature-icon">📚</div>
              <div class="feature-info">
                <strong>{{ t.onboarding.feat2Title }}</strong>
                <p>{{ t.onboarding.feat2Desc }}</p>
              </div>
            </div>
            <div class="feature-item">
              <div class="feature-icon">🔍</div>
              <div class="feature-info">
                <strong>{{ t.onboarding.feat3Title }}</strong>
                <p>{{ t.onboarding.feat3Desc }}</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 2: Auto-detect AI Pipeline Setup -->
        <div v-else-if="step === 2" class="step-pane">
          <div class="step-intro">
            <h4>{{ t.onboarding.step2Title }}</h4>
            <p>{{ t.onboarding.step2Desc }}</p>
          </div>

          <div v-if="detecting" class="loading-state">
            <span class="spinner">⏳</span>
            <span>{{ t.addFolder.detecting }}</span>
          </div>

          <div v-else-if="detectedPaths.length > 0" class="detected-selection">
            <label class="section-label">{{ t.onboarding.selectSource }}</label>
            <div class="source-list">
              <div
                v-for="(dp, idx) in detectedPaths"
                :key="idx"
                class="source-option"
                :class="{ active: selectedDetected?.path === dp.path }"
                @click="selectedDetected = dp"
              >
                <div class="radio-indicator">
                  <span class="dot" :class="{ on: selectedDetected?.path === dp.path }"></span>
                </div>
                <div class="source-text">
                  <div class="source-title">
                    <strong>{{ dp.tool_name }}</strong>
                    <span class="badge">{{ dp.category }}</span>
                  </div>
                  <span class="source-path">{{ dp.path }}</span>
                </div>
              </div>
            </div>

            <div class="vault-target">
              <label class="section-label">{{ t.onboarding.vaultLabel }}</label>
              <div class="path-group">
                <input
                  v-model="vaultPath"
                  type="text"
                  :placeholder="t.onboarding.vaultPlaceholder"
                />
                <button type="button" class="btn-browse" @click="pickVaultPath">
                  {{ t.addFolder.browse }}
                </button>
              </div>
              <span class="hint">{{ t.onboarding.vaultHint }}</span>
            </div>
          </div>

          <div v-else class="empty-detected">
            <span class="empty-icon">📁</span>
            <p>{{ t.onboarding.noAiDetected }}</p>
          </div>
        </div>

        <!-- Step 3: All Ready -->
        <div v-else-if="step === 3" class="step-pane ready-pane">
          <div class="ready-hero">🎉</div>
          <h3>{{ t.onboarding.readyTitle }}</h3>
          <p>{{ t.onboarding.readyDesc }}</p>
          <div class="ready-tips">
            <div class="tip-item">
              <kbd>Ctrl+O</kbd> <span>{{ t.onboarding.tipAddFolder }}</span>
            </div>
            <div class="tip-item">
              <kbd>Ctrl+G</kbd> <span>{{ t.onboarding.tipStack }}</span>
            </div>
            <div class="tip-item">
              <kbd>C</kbd> <span>{{ t.onboarding.tipCompare }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="modal-footer">
        <button type="button" class="btn-skip" @click="skip">
          {{ t.onboarding.skip }}
        </button>

        <div class="footer-actions">
          <button
            v-if="step > 1"
            type="button"
            class="btn-back"
            @click="step--"
          >
            {{ t.onboarding.back }}
          </button>
          <button
            v-if="step < 3"
            type="button"
            class="btn-primary"
            @click="step++"
          >
            {{ t.onboarding.next }}
          </button>
          <button
            v-else
            type="button"
            class="btn-primary"
            :disabled="submitting"
            @click="finishSetup"
          >
            {{ submitting ? t.nav.scanning : t.onboarding.startUsing }}
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
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(6px);
  z-index: 400;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-dialog {
  background: #1e1e1e;
  color: #fff;
  width: 90%;
  max-width: 580px;
  border-radius: 14px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  animation: zoomIn 0.2s ease-out;
}

.btn-modal-close {
  position: absolute;
  top: 1rem;
  right: 1.2rem;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  font-size: 1.2rem;
  cursor: pointer;
  z-index: 10;
  border-radius: 4px;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.btn-modal-close:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

@media (prefers-color-scheme: light) {
  .btn-modal-close {
    color: rgba(0, 0, 0, 0.5);
  }
  .btn-modal-close:hover {
    background: rgba(0, 0, 0, 0.08);
    color: #000;
  }
}

@media (prefers-color-scheme: light) {
  .modal-dialog {
    background: #ffffff;
    color: #1a1a1a;
    border: 1px solid rgba(0, 0, 0, 0.15);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.25);
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
  padding: 1.5rem 1.8rem 1rem;
  border-bottom: 1px solid rgba(128, 128, 128, 0.15);
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.header-badge {
  font-size: 0.75em;
  font-weight: 600;
  background: rgba(47, 111, 237, 0.15);
  color: #2f6fed;
  padding: 0.2rem 0.6rem;
  border-radius: 20px;
  margin-bottom: 0.5rem;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.35em;
  font-weight: 700;
}

.header-subtitle {
  margin: 0.4rem 0 0;
  font-size: 0.88em;
  color: #888;
}

.modal-error {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
  padding: 0.6rem 1.2rem;
  font-size: 0.85em;
  border-bottom: 1px solid rgba(239, 68, 68, 0.3);
}

.modal-body {
  padding: 1.5rem 1.8rem;
  min-height: 260px;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.feature-highlights {
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
}

.feature-item {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
}

.feature-icon {
  font-size: 1.5rem;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: rgba(47, 111, 237, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.feature-info strong {
  font-size: 0.95em;
  display: block;
}

.feature-info p {
  margin: 0.2rem 0 0;
  font-size: 0.82em;
  color: #888;
  line-height: 1.35;
}

.step-intro h4 {
  margin: 0 0 0.2rem;
  font-size: 1.05em;
}

.step-intro p {
  margin: 0 0 1rem;
  font-size: 0.82em;
  color: #888;
}

.section-label {
  font-size: 0.78em;
  text-transform: uppercase;
  color: #888;
  letter-spacing: 0.04em;
  font-weight: 600;
  display: block;
  margin-bottom: 0.4rem;
}

.source-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.source-option {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 0.6rem 0.8rem;
  border-radius: 8px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: rgba(128, 128, 128, 0.05);
  cursor: pointer;
  transition: all 0.15s ease;
}

.source-option.active {
  border-color: #2f6fed;
  background: rgba(47, 111, 237, 0.1);
}

.radio-indicator .dot {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid rgba(128, 128, 128, 0.6);
}

.radio-indicator .dot.on {
  border-color: #2f6fed;
  background: #2f6fed;
  box-shadow: inset 0 0 0 2px #fff;
}

.source-text {
  flex: 1;
  overflow: hidden;
}

.source-title {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.88em;
}

.source-title .badge {
  font-size: 0.7em;
  background: rgba(47, 111, 237, 0.2);
  color: #2f6fed;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
}

.source-path {
  font-size: 0.75em;
  color: #777;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: block;
}

.path-group {
  display: flex;
  gap: 0.5rem;
}

.path-group input {
  flex: 1;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: rgba(0, 0, 0, 0.2);
  color: inherit;
  font: inherit;
  font-size: 0.85em;
}

@media (prefers-color-scheme: light) {
  .path-group input {
    background: #fff;
  }
}

.btn-browse {
  padding: 0.5rem 0.8rem;
  border-radius: 6px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  background: transparent;
  color: inherit;
  font: inherit;
  font-size: 0.85em;
  cursor: pointer;
}

.hint {
  font-size: 0.75em;
  color: #888;
  margin-top: 0.25rem;
  display: block;
}

.empty-detected {
  text-align: center;
  padding: 1.5rem 0;
  color: #888;
}

.empty-icon {
  font-size: 2.2rem;
  display: block;
  margin-bottom: 0.4rem;
}

.ready-pane {
  text-align: center;
}

.ready-hero {
  font-size: 3rem;
  margin-bottom: 0.5rem;
}

.ready-pane h3 {
  margin: 0;
  font-size: 1.25em;
}

.ready-pane p {
  margin: 0.4rem 0 1.2rem;
  font-size: 0.85em;
  color: #888;
}

.ready-tips {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-width: 320px;
  margin: 0 auto;
  text-align: left;
}

.tip-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  font-size: 0.85em;
}

.tip-item kbd {
  background: rgba(128, 128, 128, 0.2);
  border: 1px solid rgba(128, 128, 128, 0.3);
  border-radius: 4px;
  padding: 0.2rem 0.5rem;
  font-family: inherit;
  font-size: 0.85em;
  font-weight: 600;
  min-width: 60px;
  text-align: center;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.8rem;
  border-top: 1px solid rgba(128, 128, 128, 0.15);
}

.btn-skip {
  background: transparent;
  border: none;
  color: #888;
  font: inherit;
  font-size: 0.85em;
  cursor: pointer;
}

.btn-skip:hover {
  text-decoration: underline;
}

.footer-actions {
  display: flex;
  gap: 0.6rem;
}

.btn-back,
.btn-primary {
  padding: 0.5rem 1.1rem;
  border-radius: 6px;
  font: inherit;
  font-size: 0.85em;
  cursor: pointer;
}

.btn-back {
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
</style>
