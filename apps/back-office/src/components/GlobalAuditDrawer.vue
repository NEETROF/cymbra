<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import type { AuditRow } from "@/stores/flags";
import { appLabel } from "@/i18n/app-label";
import AuditTimeline from "./AuditTimeline.vue";

const props = defineProps<{
  open: boolean;
  apps: string[];
  audit: AuditRow[];
  loading?: boolean;
  resolve?: (uuid: string) => string;
}>();

const emit = defineEmits<{
  (e: "search", payload: { app: string; key: string }): void;
  (e: "close"): void;
}>();

const { t } = useI18n();
const selApp = ref("");
const keyQuery = ref("");

function apply() {
  emit("search", { app: selApp.value, key: keyQuery.value.trim() });
}
function pickApp(a: string) {
  selApp.value = a;
  apply();
}
</script>

<template>
  <div v-if="open" class="overlay" @click.self="emit('close')">
    <dialog class="drawer" open aria-modal="true">
      <header>
        <h3>{{ t("flags.globalAuditTitle") }}</h3>
        <button type="button" class="x" :aria-label="t('flags.cancel')" @click="emit('close')">✕</button>
      </header>

      <div class="filters">
        <div class="apps">
          <button type="button" :class="{ active: selApp === '' }" @click="pickApp('')">
            {{ t("flags.allApps") }}
          </button>
          <button v-for="a in props.apps" :key="a" type="button" :class="{ active: selApp === a }" @click="pickApp(a)">
            {{ appLabel(a, t) }}
          </button>
        </div>
        <div class="search">
          <input
            v-model="keyQuery"
            :placeholder="t('flags.searchKey')"
            :aria-label="t('flags.searchKey')"
            @keyup.enter="apply"
          />
          <button type="button" @click="apply">{{ t("flags.search") }}</button>
        </div>
      </div>

      <AuditTimeline :entries="audit" :loading="loading" :resolve="resolve" show-key />
    </dialog>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 90;
}
.drawer {
  position: fixed;
  top: 0;
  right: 0;
  /* Cancel the browser's UA `dialog { left: 0 }`, which otherwise wins over `right`
     when the width is fixed and pins the drawer to the LEFT. */
  left: auto;
  height: 100vh;
  max-height: 100vh;
  width: min(480px, 94vw);
  max-width: none;
  margin: 0;
  overflow-y: auto;
  background: var(--panel, #1a1a24);
  color: inherit;
  border: none;
  border-left: 1px solid var(--border);
  padding: 1.35rem 1.4rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  animation: slideIn 0.22s ease;
}
@keyframes slideIn {
  from {
    transform: translateX(100%);
  }
  to {
    transform: translateX(0);
  }
}
header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}
header h3 {
  margin: 0;
  flex: 1;
}
.x {
  background: transparent;
  color: var(--muted);
}
.filters {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.apps {
  display: inline-flex;
  gap: 0.35rem;
  flex-wrap: wrap;
}
.apps button.active {
  border-color: var(--accent);
  color: var(--accent);
}
.search {
  display: flex;
  gap: 0.4rem;
}
.search input {
  flex: 1;
}
</style>
