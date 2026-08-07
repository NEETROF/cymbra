<script setup lang="ts">
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import type { CatalogHit } from "@/gen/score_pb";
import type { MetadataEdit } from "@/stores/catalog";

// Gated curatorial edit, in a right-side drawer (matching the SoundFont/Flag/
// GlobalAudit convention) so the screen underneath keeps showing the read-only
// summary — change: add-catalog-metadata-editing. Purely presentational: it edits
// ONLY the descriptive/attribution fields and emits the change set; the
// MusicXML-derived facets are never editable (they'd desync from the score) and are
// read on the page behind, not duplicated here. No API/engine access — the parent
// view owns the store action, the submit state, and when the drawer closes.
const props = defineProps<{
  open: boolean;
  hit: CatalogHit | null;
  submitting: boolean;
  error: string | null;
}>();
const emit = defineEmits<{ submit: [edit: MetadataEdit]; close: [] }>();
const { t } = useI18n();

// The available difficulty levels (mirrors the server enum). "" = cleared/none.
const LEVELS = ["", "beginner", "intermediate", "advanced"] as const;

// Local editable copy, re-seeded whenever the drawer opens on a (new) score, so a
// cancelled edit never leaks into the next one.
const form = reactive({ title: "", composer: "", arranger: "", level: "" });
watch(
  () => [props.open, props.hit?.id] as const,
  () => {
    const h = props.hit;
    form.title = h?.title ?? "";
    form.composer = h?.composer ?? "";
    form.arranger = h?.arranger ?? "";
    form.level = h?.level ?? "";
  },
  { immediate: true },
);

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
  <Teleport to="body">
    <div v-if="open" class="overlay" @click.self="emit('close')">
      <dialog class="drawer" open aria-modal="true" @keydown.esc="emit('close')">
        <header>
          <div>
            <h2>{{ t("edit.heading") }}</h2>
            <p class="who">{{ hit?.title || t("detail.score") }}</p>
          </div>
          <button type="button" class="x" :aria-label="t('edit.cancel')" @click="emit('close')">✕</button>
        </header>

        <form class="edit body" @submit.prevent="onSubmit">
          <p class="hint">{{ t("edit.hint") }}</p>
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

          <p v-if="error" class="error" role="alert">{{ error }}</p>

          <footer>
            <button type="submit" class="save btn-primary" :disabled="submitting">
              {{ submitting ? t("edit.saving") : t("edit.save") }}
            </button>
            <button type="button" class="cancel" :disabled="submitting" @click="emit('close')">
              {{ t("edit.cancel") }}
            </button>
          </footer>
        </form>
      </dialog>
    </div>
  </Teleport>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 50;
}
.drawer {
  position: fixed;
  top: 0;
  right: 0;
  /* Cancel the browser's UA `dialog { left: 0 }`, which otherwise wins over `right`
     when the width is fixed and pins the drawer to the LEFT. */
  left: auto;
  width: min(30rem, 94vw);
  height: 100vh;
  max-height: 100vh;
  max-width: none;
  margin: 0;
  border: none;
  border-left: 1px solid var(--border);
  background: var(--panel);
  color: inherit;
  padding: 0;
  display: flex;
  flex-direction: column;
  animation: slide-in 0.18s ease-out;
}
@keyframes slide-in {
  from {
    transform: translateX(100%);
  }
}
header {
  display: flex;
  align-items: flex-start;
  gap: 0.6rem;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}
header > div {
  flex: 1;
}
header h2 {
  margin: 0;
  font-size: 1.1rem;
}
.who {
  margin: 0.15rem 0 0;
  color: var(--muted);
  font-size: 0.85rem;
}
.x {
  background: none;
  border: none;
  color: var(--muted);
  font-size: 1.1rem;
}
.body {
  padding: 1.25rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.85rem;
  flex: 1;
}
.hint {
  color: var(--muted);
  margin: 0;
  font-size: 0.8rem;
}
.body label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.85rem;
  min-width: 0;
}
.body label span {
  color: var(--muted);
}
.body input,
.body select {
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
}
.error {
  color: var(--coral);
  margin: 0;
}
footer {
  display: flex;
  gap: 0.5rem;
  margin-top: auto;
  padding-top: 0.5rem;
}
</style>
