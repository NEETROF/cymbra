<script setup lang="ts">
import type { CatalogHit } from "@/gen/score_pb";
import type { ModerationStatus, SortKeyInit } from "@/stores/catalog";

// Sortable columns: only fields that are both displayable here AND in the server
// sort allow-list. Clicking a header rebuilds the single-key `sort` sent to the API
// (server-side sort across the whole set — no client-side sort) and toggles the
// direction. The active moderation status is the current filter, shown per row.
const props = defineProps<{
  hits: CatalogHit[];
  status: ModerationStatus;
  sort: SortKeyInit[];
}>();
const emit = defineEmits<{ sort: [field: string]; select: [id: string] }>();

const columns: { field: string; label: string }[] = [
  { field: "title", label: "Title" },
  { field: "composer", label: "Composer" },
  { field: "level", label: "Level" },
  { field: "note_count", label: "Notes" },
  { field: "tempo_bpm", label: "BPM" },
];

function arrow(field: string): string {
  const key = props.sort.find((k) => k.field === field);
  if (!key) return "";
  return key.descending ? " ↓" : " ↑";
}
</script>

<template>
  <table>
    <thead>
      <tr>
        <th
          v-for="c in columns"
          :key="c.field"
          class="sortable"
          role="button"
          :aria-label="`sort by ${c.label}`"
          @click="emit('sort', c.field)"
        >
          {{ c.label }}{{ arrow(c.field) }}
        </th>
        <th>Source</th>
        <th>Status</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="h in hits" :key="h.id" class="row" @click="emit('select', h.id)">
        <td>{{ h.title || "—" }}</td>
        <td>{{ h.composer || "—" }}</td>
        <td>{{ h.level || "—" }}</td>
        <td>{{ h.noteCount ?? "—" }}</td>
        <td>{{ h.tempoBpm ?? "—" }}</td>
        <td>{{ h.source }}</td>
        <td><span class="badge" :class="status">{{ status }}</span></td>
      </tr>
      <tr v-if="hits.length === 0">
        <td colspan="7" class="empty">No scores.</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.sortable {
  cursor: pointer;
  user-select: none;
}
.row {
  cursor: pointer;
}
.row:hover {
  background: var(--panel);
}
.empty {
  color: var(--muted);
  text-align: center;
  padding: 1.5rem;
}
</style>
