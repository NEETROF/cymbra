<script setup lang="ts">
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { Filters, StatusFilter } from "@/stores/catalog";

// Reuses the app-hub filters (text/author/level/piano) plus the back-office-only
// moderation-status + source selectors. Emits the current filter set whenever it changes.
const emit = defineEmits<{ change: [Filters] }>();
const { t } = useI18n();

// Default the status filter to "" (Tous) — the BO catalog shows every status
// (change: add-score-catalog-proposal).
const props = withDefaults(defineProps<{ status?: StatusFilter; source?: string }>(), {
  status: "",
  source: "",
});

const f = reactive<Filters>({
  query: "",
  author: "",
  level: "",
  isPiano: undefined,
  moderationStatus: props.status,
  source: props.source,
});

watch(f, () => emit("change", { ...f }), { deep: true });
</script>

<template>
  <div class="filters">
    <input v-model="f.query" type="search" :placeholder="t('filters.query')" :aria-label="t('filters.searchLabel')" />
    <input
      v-model="f.author"
      type="text"
      :placeholder="t('filters.composer')"
      :aria-label="t('filters.composerLabel')"
    />
    <select v-model="f.level" :aria-label="t('filters.levelLabel')">
      <option value="">{{ t("level.any") }}</option>
      <option value="beginner">{{ t("level.beginner") }}</option>
      <option value="intermediate">{{ t("level.intermediate") }}</option>
      <option value="advanced">{{ t("level.advanced") }}</option>
    </select>
    <label class="piano">
      <input type="checkbox" :checked="f.isPiano === true" @change="f.isPiano = f.isPiano ? undefined : true" />
      {{ t("filters.pianoOnly") }}
    </label>
    <select v-model="f.moderationStatus" :aria-label="t('filters.statusLabel')" class="status">
      <option value="">{{ t("status.all") }}</option>
      <option value="pending">{{ t("status.pending") }}</option>
      <option value="accepted">{{ t("status.accepted") }}</option>
      <option value="rejected">{{ t("status.rejected") }}</option>
    </select>
    <select v-model="f.source" :aria-label="t('filters.sourceLabel')" class="source">
      <option value="">{{ t("filters.sourceAny") }}</option>
      <option value="user-proposal">{{ t("filters.sourceUser") }}</option>
    </select>
  </div>
</template>

<style scoped>
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
  justify-content: flex-end;
  padding: 0.6rem 0.7rem;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 14px;
}
.filters input,
.filters select {
  background: var(--panel-2);
  border-radius: 999px;
}
.filters input[type="search"] {
  min-width: 12rem;
}
.piano {
  display: flex;
  gap: 0.35rem;
  align-items: center;
  color: var(--muted);
  font-size: 0.9rem;
  padding: 0 0.3rem;
  white-space: nowrap;
}
.piano input {
  accent-color: var(--accent-strong);
}
</style>
