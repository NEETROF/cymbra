<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useI18n } from "vue-i18n";

/** In-app confirmation for a destructive action.
 *
 *  The console never uses `window.confirm` / `window.prompt`: a native dialog
 *  blocks the renderer, which puts the action out of reach of the Playwright
 *  suite and of browser automation, and it cannot be localized or styled. Views
 *  that already own a modal stack (Plans) inline their own; this component
 *  serves the ones whose only dialog is a confirmation.
 *
 *  Renders nothing while `message` is null, so a view can bind it straight to a
 *  "pending action" ref. */
const props = defineProps<{
  /** The localized question; `null` closes the dialog. */
  message: string | null;
  /** An action in flight: the buttons stay visible but inert. */
  busy?: boolean;
}>();

const emit = defineEmits<{ confirm: []; cancel: [] }>();
const { t } = useI18n();

/** Focus moves INTO the dialog when it opens. Two reasons, both load-bearing:
 *  `aria-modal` requires it, and the Escape handler below lives on the dialog —
 *  with focus left on the trigger button the keydown never reaches it, so Escape
 *  silently did nothing. The container takes the focus rather than the confirm
 *  button, so Enter cannot fire a destructive action the operator never aimed at. */
const dialogEl = ref<HTMLDialogElement | null>(null);
watch(
  () => props.message,
  async (message) => {
    if (!message) return;
    await nextTick();
    dialogEl.value?.focus();
  },
);
</script>

<template>
  <div v-if="props.message" class="overlay" @click.self="emit('cancel')">
    <dialog ref="dialogEl" class="modal" open aria-modal="true" tabindex="-1" @keydown.esc="emit('cancel')">
      <h2>{{ t("common.confirmTitle") }}</h2>
      <p>{{ props.message }}</p>
      <div class="modal-actions">
        <button type="button" class="btn-primary" :disabled="props.busy" @click="emit('confirm')">
          {{ t("common.confirm") }}
        </button>
        <button type="button" :disabled="props.busy" @click="emit('cancel')">{{ t("common.cancel") }}</button>
      </div>
    </dialog>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 90;
  display: grid;
  place-items: center;
}
.modal {
  position: static;
  width: min(480px, 94vw);
  margin: 0;
  background: var(--panel, #1a1a24);
  color: inherit;
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.35rem 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}
.modal:focus {
  outline: none;
}
.modal h2 {
  font-size: 1.05rem;
  margin: 0;
}
.modal p {
  margin: 0;
  font-size: 0.9rem;
}
.modal-actions {
  display: flex;
  gap: 0.6rem;
  justify-content: flex-end;
}
</style>
