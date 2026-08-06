<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { useToastsStore } from "@/stores/toasts";

// Global toast host: renders the toasts store's stack, mounted once in App.vue.
// Each toast auto-dismisses (store timer) and carries an explicit close button.
const toasts = useToastsStore();
const { t } = useI18n();
</script>

<template>
  <div class="toast-host" role="region" aria-live="polite" :aria-label="t('toast.region')">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts.items"
        :key="toast.id"
        class="toast"
        :class="toast.variant"
        :role="toast.variant === 'error' ? 'alert' : 'status'"
      >
        <span class="msg">{{ toast.message }}</span>
        <button type="button" class="close" :aria-label="t('toast.dismiss')" @click="toasts.dismiss(toast.id)">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-host {
  position: fixed;
  z-index: 1000;
  /* Bottom-right, out of the way of the header actions. */
  bottom: 1rem;
  right: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  max-width: min(92vw, 26rem);
  pointer-events: none;
}
.toast {
  pointer-events: auto;
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
  padding: 0.7rem 0.8rem 0.7rem 0.9rem;
  border-radius: var(--radius-lg, 12px);
  /* A SOLID variant-coloured fill (deliberately NOT the app's navy) so a toast pops
     against the dark blue canvas; dark text keeps it readable. */
  color: #10182b;
  font-weight: 600;
  box-shadow: 0 12px 32px rgb(0 0 0 / 45%);
}
.toast.success {
  background: var(--green);
}
.toast.error {
  background: var(--coral);
}
.toast.info {
  background: var(--amber);
}
.msg {
  flex: 1;
  font-size: 0.9rem;
  line-height: 1.35;
}
.close {
  flex: none;
  display: inline-grid;
  place-items: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: rgb(16 24 43 / 55%);
  cursor: pointer;
}
.close:hover {
  color: #10182b;
  background: rgb(16 24 43 / 14%);
}
.close svg {
  width: 15px;
  height: 15px;
}

/* Enter/leave transitions (slide up + fade from below). */
.toast-enter-active,
.toast-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(12px);
}
.toast-leave-active {
  position: absolute;
  right: 0;
}
</style>
