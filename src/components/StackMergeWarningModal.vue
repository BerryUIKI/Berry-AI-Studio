<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { t } from "../i18n";

const props = defineProps<{
  show: boolean;
  stackCount: number;
  imageCount: number;
}>();

const emit = defineEmits<{
  (e: "cancel"): void;
  (e: "confirm", suppressFutureWarnings: boolean): void;
}>();

const suppressFutureWarnings = ref(false);
const cancelButton = ref<HTMLButtonElement | null>(null);

const summary = computed(() => t.value.stack.mergeWarningMessage
  .replace("{stacks}", String(props.stackCount))
  .replace("{images}", String(props.imageCount)));

watch(
  () => props.show,
  (show) => {
    if (!show) return;
    suppressFutureWarnings.value = false;
    void nextTick(() => cancelButton.value?.focus());
  },
  { immediate: true },
);
</script>

<template>
  <div
    v-if="show"
    class="modal-overlay"
    role="presentation"
    @click.self="emit('cancel')"
    @keydown.esc.stop="emit('cancel')"
  >
    <section
      class="warning-dialog"
      role="alertdialog"
      aria-modal="true"
      :aria-labelledby="'stack-merge-warning-title'"
      :aria-describedby="'stack-merge-warning-description'"
    >
      <header class="dialog-header">
        <span class="warning-symbol" aria-hidden="true">!</span>
        <h3 id="stack-merge-warning-title">{{ t.stack.mergeWarningTitle }}</h3>
      </header>

      <div id="stack-merge-warning-description" class="dialog-body">
        <p>{{ summary }}</p>
        <p>{{ t.stack.mergeWarningTarget }}</p>
        <p class="invariant-note">{{ t.stack.mergeWarningNoNesting }}</p>

        <label class="suppress-option">
          <input v-model="suppressFutureWarnings" type="checkbox" />
          <span>{{ t.stack.dontShowAgain }}</span>
        </label>
      </div>

      <footer class="dialog-footer">
        <button ref="cancelButton" type="button" class="btn secondary" @click="emit('cancel')">
          {{ t.stack.cancelMerge }}
        </button>
        <button
          type="button"
          class="btn warning"
          @click="emit('confirm', suppressFutureWarnings)"
        >
          {{ t.stack.confirmMerge }}
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 2600;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(0, 0, 0, 0.72);
  backdrop-filter: blur(6px);
}

.warning-dialog {
  width: min(480px, 100%);
  overflow: hidden;
  border: 1px solid rgba(245, 158, 11, 0.38);
  border-radius: 12px;
  background: #18181c;
  color: #e2e8f0;
  box-shadow: 0 24px 70px rgba(0, 0, 0, 0.5);
}

.dialog-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.dialog-header h3 {
  margin: 0;
  font-size: 0.95rem;
}

.warning-symbol {
  display: inline-flex;
  width: 24px;
  height: 24px;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgba(245, 158, 11, 0.16);
  color: #fbbf24;
  font-weight: 800;
}

.dialog-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 18px;
  font-size: 0.8rem;
  line-height: 1.55;
}

.dialog-body p {
  margin: 0;
}

.invariant-note {
  padding: 9px 10px;
  border-left: 3px solid #f59e0b;
  border-radius: 4px;
  background: rgba(245, 158, 11, 0.08);
  color: #fde68a;
}

.suppress-option {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  color: #cbd5e1;
  cursor: pointer;
}

.suppress-option input {
  accent-color: #f59e0b;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 18px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: #141417;
}

.btn {
  padding: 7px 13px;
  border: 0;
  border-radius: 6px;
  color: #fff;
  font: inherit;
  cursor: pointer;
}

.btn.secondary {
  background: rgba(255, 255, 255, 0.08);
}

.btn.warning {
  background: #d97706;
}

.btn:focus-visible {
  outline: 2px solid #fbbf24;
  outline-offset: 2px;
}
</style>
