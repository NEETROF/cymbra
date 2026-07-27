<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { CatalogHit } from "@/gen/score_pb";

// Isolated preview module (design D5). Today it shows metadata + confirms the
// score bytes fetched; the notation itself is rendered by a deferred module that
// compiles the app's Rust `layout_systems` to wasm and paints with a JS/SVG SMuFL
// painter, so the moderator eventually sees exactly the app's engraving. Keeping
// this component the single seam means that renderer drops in without touching the
// rest of the console — or is swapped for a JS fallback if the wasm cost is too high.
const props = defineProps<{ hit: CatalogHit | null; bytes: Uint8Array | null; loading: boolean }>();
const { t } = useI18n();

function meta(): { label: string; value: string }[] {
  const h = props.hit;
  if (!h) return [];
  return [
    { label: t("preview.title"), value: h.title || "—" },
    { label: t("preview.composer"), value: h.composer || "—" },
    { label: t("preview.arranger"), value: h.arranger || "—" },
    { label: t("preview.level"), value: h.level || "—" },
    { label: t("preview.licence"), value: h.license },
    { label: t("preview.source"), value: h.source },
    { label: t("preview.timeSig"), value: h.timeSig || "—" },
    { label: t("preview.notes"), value: h.noteCount != null ? String(h.noteCount) : "—" },
    { label: t("preview.tempo"), value: h.tempoBpm != null ? String(h.tempoBpm) : "—" },
  ];
}
</script>

<template>
  <div class="preview">
    <dl class="meta">
      <template v-for="m in meta()" :key="m.label">
        <dt>{{ m.label }}</dt>
        <dd>{{ m.value }}</dd>
      </template>
    </dl>

    <div class="notation" aria-label="score preview">
      <p v-if="loading" class="muted">{{ t("preview.loading") }}</p>
      <p v-else-if="bytes" class="muted">
        {{ t("preview.loaded", { bytes: bytes.length.toLocaleString() }) }}
      </p>
      <p v-else class="muted">{{ t("preview.noBytes") }}</p>
    </div>
  </div>
</template>

<style scoped>
.preview {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 1rem;
}
.meta {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 0.2rem 0.8rem;
  margin: 0;
  align-content: start;
}
.meta dt {
  color: var(--muted);
}
.meta dd {
  margin: 0;
}
.notation {
  min-height: 220px;
  border: 1px dashed var(--border);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
  text-align: center;
}
.muted {
  color: var(--muted);
}
</style>
