<script setup lang="ts">
import { useI18n } from "vue-i18n";
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
const { t } = useI18n();

const columns: { field: string; labelKey: string }[] = [
  { field: "title", labelKey: "table.title" },
  { field: "composer", labelKey: "table.composer" },
  { field: "level", labelKey: "table.level" },
  { field: "note_count", labelKey: "table.notes" },
  { field: "tempo_bpm", labelKey: "table.bpm" },
];

const KNOWN_LEVELS = new Set(["beginner", "intermediate", "advanced"]);

function arrow(field: string): string {
  const key = props.sort.find((k) => k.field === field);
  if (!key) return "";
  return key.descending ? " ↓" : " ↑";
}

function shortId(id: string): string {
  return id.replaceAll("-", "").slice(0, 8).toUpperCase();
}

function levelLabel(level: string): string {
  return KNOWN_LEVELS.has(level) ? t(`level.${level}`) : level;
}

// Each row shows its OWN moderation status (the review queue mixes pending +
// flagged accepted); fall back to the active filter when the hit carries none.
function rowStatus(h: CatalogHit): ModerationStatus {
  return (h.moderationStatus as ModerationStatus) || props.status;
}
</script>

<template>
  <table>
    <thead>
      <tr>
        <th v-for="c in columns" :key="c.field">
          <button
            type="button"
            class="sortable"
            :aria-label="t('table.sortBy', { field: t(c.labelKey) })"
            @click="emit('sort', c.field)"
          >
            {{ t(c.labelKey) }}{{ arrow(c.field) }}
          </button>
        </th>
        <th>{{ t("table.source") }}</th>
        <th>{{ t("table.status") }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="h in hits" :key="h.id" class="row" @click="emit('select', h.id)">
        <td>
          <div class="title-cell">
            <span class="thumb" aria-hidden="true">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <path d="M9 18V5l12-2v13" />
                <circle cx="6" cy="18" r="3" />
                <circle cx="18" cy="16" r="3" />
              </svg>
            </span>
            <span class="title-text">
              <span class="t-name">{{ h.title || "—" }}</span>
              <span class="t-id">ID: {{ shortId(h.id) }}</span>
            </span>
          </div>
        </td>
        <td>{{ h.composer || "—" }}</td>
        <td>
          <span v-if="h.level" class="pill" :class="`lvl-${h.level}`">{{ levelLabel(h.level) }}</span>
          <span v-else>—</span>
        </td>
        <td class="num">{{ h.noteCount ?? "—" }}</td>
        <td class="num">{{ h.tempoBpm ?? "—" }}</td>
        <td>
          <span class="src">{{ h.source }}</span>
        </td>
        <td>
          <span class="badge" :class="rowStatus(h)">{{ t(`status.${rowStatus(h)}`) }}</span>
          <span v-if="h.needsReview" class="badge review" :title="t('table.needsReviewHint')">{{
            t("table.needsReview")
          }}</span>
        </td>
      </tr>
      <tr v-if="hits.length === 0">
        <td colspan="7" class="empty">{{ t("table.empty") }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
/* The sortable header is a real <button> (keyboard-accessible) reset to look like a
   plain <th> — it inherits the uppercase/color from the global thead th. */
.sortable {
  background: none;
  border: 0;
  margin: 0;
  padding: 0;
  font: inherit;
  color: inherit;
  letter-spacing: inherit;
  text-transform: inherit;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
}
.sortable:hover {
  background: none;
  color: var(--text);
}
.row {
  cursor: pointer;
  transition: background 0.12s;
}
.row:hover {
  background: var(--panel-2);
}

.title-cell {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}
.thumb {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  flex: none;
  border-radius: 10px;
  background: color-mix(in srgb, var(--accent-strong) 20%, transparent);
  color: var(--accent);
}
.thumb svg {
  width: 18px;
  height: 18px;
}
.title-text {
  display: flex;
  flex-direction: column;
  line-height: 1.25;
}
.t-name {
  font-weight: 600;
  color: var(--text);
}
.t-id {
  font-family: var(--mono);
  font-size: 0.68rem;
  color: var(--faint);
  letter-spacing: 0.05em;
}

.num {
  font-variant-numeric: tabular-nums;
  color: var(--text);
}

.pill {
  display: inline-block;
  padding: 0.18rem 0.55rem;
  border-radius: 999px;
  font-size: 0.74rem;
  font-weight: 600;
  border: 1px solid var(--border-2);
  color: var(--muted);
  text-transform: capitalize;
}
.pill.lvl-beginner {
  color: var(--green);
  border-color: color-mix(in srgb, var(--green) 35%, transparent);
}
.pill.lvl-intermediate {
  color: var(--teal);
  border-color: color-mix(in srgb, var(--teal) 35%, transparent);
}
.pill.lvl-advanced {
  color: var(--amber);
  border-color: color-mix(in srgb, var(--amber) 35%, transparent);
}

.src {
  color: var(--green);
  font-weight: 500;
}

/* Community-flagged re-review marker, shown beside the row's status in the queue. */
.badge.review {
  margin-left: 0.4rem;
  color: var(--amber);
  border: 1px solid color-mix(in srgb, var(--amber) 45%, transparent);
  background: color-mix(in srgb, var(--amber) 14%, transparent);
}

.empty {
  color: var(--muted);
  text-align: center;
  padding: 2rem;
}
</style>
