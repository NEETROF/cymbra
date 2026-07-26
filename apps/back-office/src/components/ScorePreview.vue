<script setup lang="ts">
import type { CatalogHit } from "@/gen/score_pb";

// Isolated preview module (design D5). Today it shows metadata + confirms the
// score bytes fetched; the notation itself is rendered by a deferred module that
// compiles the app's Rust `layout_systems` to wasm and paints with a JS/SVG SMuFL
// painter, so the moderator eventually sees exactly the app's engraving. Keeping
// this component the single seam means that renderer drops in without touching the
// rest of the console — or is swapped for a JS fallback if the wasm cost is too high.
const props = defineProps<{ hit: CatalogHit | null; bytes: Uint8Array | null; loading: boolean }>();

function meta(): { label: string; value: string }[] {
  const h = props.hit;
  if (!h) return [];
  return [
    { label: "Title", value: h.title || "—" },
    { label: "Composer", value: h.composer || "—" },
    { label: "Arranger", value: h.arranger || "—" },
    { label: "Level", value: h.level || "—" },
    { label: "Licence", value: h.license },
    { label: "Source", value: h.source },
    { label: "Time signature", value: h.timeSig || "—" },
    { label: "Notes", value: h.noteCount != null ? String(h.noteCount) : "—" },
    { label: "Tempo (BPM)", value: h.tempoBpm != null ? String(h.tempoBpm) : "—" },
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
      <p v-if="loading" class="muted">Loading score…</p>
      <p v-else-if="bytes" class="muted">
        Score loaded ({{ bytes.length.toLocaleString() }} bytes). Notation rendering
        (Rust <code>layout_systems</code> → wasm) is not wired yet in this slice.
      </p>
      <p v-else class="muted">No score bytes.</p>
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
