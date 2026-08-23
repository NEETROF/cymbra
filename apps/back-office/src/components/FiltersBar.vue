<script setup lang="ts">
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { Filters, StatusFilter } from "@/stores/catalog";

// Reuses the app-hub filters (text/author/level/instrument) plus the back-office-only
// moderation-status + source selectors. Emits the current filter set whenever it changes.
const emit = defineEmits<{ change: [Filters] }>();
const { t } = useI18n();

// `initial` seeds the bar from the caller's persisted browse state (so filters —
// including the source and the "" = Tous status — survive a detail-page round-trip);
// `status`/`source` are the back-compat fallbacks when no full filter set is supplied.
// The BO catalog defaults the status to "" (Tous): every moderation status (change:
// add-score-catalog-proposal).
const props = withDefaults(defineProps<{ status?: StatusFilter; source?: string; initial?: Filters }>(), {
  status: "",
  source: "",
  initial: undefined,
});

const f = reactive<Filters>(
  props.initial
    ? { ...props.initial }
    : {
        query: "",
        author: "",
        level: "",
        instrument: "",
        moderationStatus: props.status,
        source: props.source,
        hasPreview: "",
      },
);

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
    <!-- Instrument family (change: add-drums-access) — replaces the piano checkbox;
         "" = every instrument, mapped to an unset request field. -->
    <select v-model="f.instrument" :aria-label="t('filters.instrumentLabel')" class="instrument">
      <option value="">{{ t("filters.instrumentAny") }}</option>
      <option value="keyboard">{{ t("filters.instrumentKeyboard") }}</option>
      <option value="percussion">{{ t("filters.instrumentPercussion") }}</option>
    </select>
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
    <!-- Audio-teaser filter (change: add-score-daily-access-rewards): "no sample" is
         the backfill view — pieces still missing their server-rendered clip. -->
    <select v-model="f.hasPreview" :aria-label="t('filters.previewLabel')" class="preview">
      <option value="">{{ t("filters.previewAny") }}</option>
      <option value="yes">{{ t("filters.previewYes") }}</option>
      <option value="no">{{ t("filters.previewNo") }}</option>
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
</style>
