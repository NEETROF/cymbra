<script setup lang="ts">
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { CatalogHit } from "@/gen/score_pb";
import type { MetadataEdit } from "@/stores/catalog";

// Gated curatorial edit form (change: add-catalog-metadata-editing). Purely
// presentational: it edits ONLY the descriptive/attribution fields and emits the
// change set; the MusicXML-derived facets are shown read-only (editing them would
// desync from the actual score content). No API/engine access — the parent view
// owns the store action + submit state.
const props = defineProps<{
  hit: CatalogHit | null;
  submitting: boolean;
  error: string | null;
}>();
const emit = defineEmits<{ submit: [edit: MetadataEdit] }>();
const { t } = useI18n();

// The available difficulty levels (mirrors the server enum). "" = cleared/none.
const LEVELS = ["", "beginner", "intermediate", "advanced"] as const;

// Local editable copy, re-seeded whenever the fetched hit changes.
const form = reactive({ title: "", composer: "", arranger: "", level: "" });
watch(
  () => props.hit,
  (h) => {
    form.title = h?.title ?? "";
    form.composer = h?.composer ?? "";
    form.arranger = h?.arranger ?? "";
    form.level = h?.level ?? "";
  },
  { immediate: true },
);

// The derived facets, shown read-only (never editable — they come from the score).
function derived(): { label: string; value: string }[] {
  const h = props.hit;
  if (!h) return [];
  return [
    { label: t("preview.timeSig"), value: h.timeSig || "—" },
    { label: t("preview.notes"), value: h.noteCount != null ? String(h.noteCount) : "—" },
    { label: t("preview.tempo"), value: h.tempoBpm != null ? String(h.tempoBpm) : "—" },
    { label: t("preview.licence"), value: h.license || "—" },
    {
      label: t("preview.source"),
      // For a user upload, append the proposer's pseudo (privileged read) so the
      // metadata block matches the catalog list (change: add-score-catalog-proposal).
      value: h.proposerDisplayName ? `${h.source || "—"} · @${h.proposerDisplayName}` : h.source || "—",
    },
  ];
}

function onSubmit() {
  emit("submit", {
    title: form.title,
    composer: form.composer,
    arranger: form.arranger,
    level: form.level,
  });
}
</script>

<template>
  <form class="edit" @submit.prevent="onSubmit">
    <h2 class="edit-title">{{ t("edit.heading") }}</h2>
    <p class="hint">{{ t("edit.hint") }}</p>

    <div class="fields">
      <label>
        <span>{{ t("preview.title") }}</span>
        <input v-model="form.title" type="text" required :disabled="submitting" />
      </label>
      <label>
        <span>{{ t("preview.composer") }}</span>
        <input v-model="form.composer" type="text" :disabled="submitting" />
      </label>
      <label>
        <span>{{ t("preview.arranger") }}</span>
        <input v-model="form.arranger" type="text" :disabled="submitting" />
      </label>
      <label>
        <span>{{ t("preview.level") }}</span>
        <select v-model="form.level" :disabled="submitting">
          <option v-for="lvl in LEVELS" :key="lvl" :value="lvl">
            {{ lvl === "" ? t("edit.levelNone") : t(`level.${lvl}`) }}
          </option>
        </select>
      </label>
    </div>

    <dl class="derived" :aria-label="t('edit.derivedLabel')">
      <template v-for="d in derived()" :key="d.label">
        <dt>{{ d.label }}</dt>
        <dd>{{ d.value }}</dd>
      </template>
    </dl>

    <p v-if="error" class="error" role="alert">{{ error }}</p>
    <button type="submit" class="save" :disabled="submitting">
      {{ submitting ? t("edit.saving") : t("edit.save") }}
    </button>
  </form>
</template>

<style scoped>
.edit {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 1.25rem;
}
.edit-title {
  font-size: 1.15rem;
  margin: 0;
}
.hint {
  color: var(--muted);
  margin: 0;
  font-size: 0.85rem;
}
.fields {
  display: grid;
  gap: 0.6rem;
}
.fields label {
  display: grid;
  grid-template-columns: 8rem 1fr;
  align-items: center;
  gap: 0.6rem;
}
.fields label span {
  color: var(--muted);
}
.fields input,
.fields select {
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: inherit;
}
.derived {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 0.2rem 0.8rem;
  margin: 0;
  padding-top: 0.5rem;
  border-top: 1px dashed var(--border);
}
.derived dt {
  color: var(--muted);
}
.derived dd {
  margin: 0;
}
.error {
  color: var(--reject);
  margin: 0;
}
.save {
  align-self: flex-start;
}
</style>
