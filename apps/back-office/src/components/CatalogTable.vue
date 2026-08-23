<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import type { CatalogHit } from "@/gen/score_pb";
import type { ModerationStatus, SortKeyInit, StatusFilter } from "@/stores/catalog";
import { type Async, idle } from "@/lib/async";
import AppTag from "@/components/AppTag.vue";
import IdBadge from "@/components/IdBadge.vue";

// Sortable columns: only fields that are both displayable here AND in the server
// sort allow-list. Clicking a header rebuilds the single-key `sort` sent to the API
// (server-side sort across the whole set — no client-side sort) and toggles the
// direction. The active moderation status is the current filter, shown per row.
// `canDownload` is the moderator/admin gate (provenance): the download control is
// rendered only for authorized operators; `downloads` carries each row's own
// download `Async` so a slow/failed one never blocks the table.
const props = withDefaults(
  defineProps<{
    hits: CatalogHit[];
    status: StatusFilter;
    sort: SortKeyInit[];
    canDownload?: boolean;
    downloads?: Record<string, Async<Uint8Array>>;
    // Audio teaser (change: add-score-daily-access-rewards): the row whose sample is
    // sounding, and whether a (re)generation is in flight (one at a time).
    playingId?: string | null;
    sampleBusy?: boolean;
    generatingId?: string | null;
  }>(),
  { canDownload: false, downloads: () => ({}), playingId: null, sampleBusy: false, generatingId: null },
);
const emit = defineEmits<{
  sort: [field: string];
  select: [id: string];
  download: [hit: CatalogHit];
  playSample: [hit: CatalogHit];
  generateSample: [hit: CatalogHit];
}>();
const { t } = useI18n();

// Total column count, so the empty-state row spans the full width (the download
// column only exists for authorized operators).
const colCount = computed(() => columns.length + 2 + (props.canDownload ? 1 : 0));

// This row's download state (defaults to idle when it has never been triggered).
function downloadState(id: string): Async<Uint8Array> {
  return props.downloads[id] ?? idle;
}
function isDownloading(id: string): boolean {
  return downloadState(id).status === "loading";
}
function downloadError(id: string): string | null {
  return match(downloadState(id))
    .with({ status: "error" }, ({ error }) => error)
    .otherwise(() => null);
}

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

function levelLabel(level: string): string {
  return KNOWN_LEVELS.has(level) ? t(`level.${level}`) : level;
}

// Each row shows its OWN moderation status (the review queue mixes pending +
// flagged accepted); fall back to the active filter when the hit carries none.
function rowStatus(h: CatalogHit): ModerationStatus {
  // In "Tous" mode (props.status === "") the row carries its own status; fall back to
  // "pending" only in the impossible case where neither is set.
  return (h.moderationStatus as ModerationStatus) || props.status || "pending";
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
        <th v-if="canDownload" class="dl-col">{{ t("table.actions") }}</th>
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
              <span class="t-name">
                {{ h.title || "—" }}
                <!-- Instrument badge (change: add-drums-access): a percussion row is
                     identifiable BEFORE it is opened — the moderator then knows why
                     Play is unavailable (the preview renders since
                     add-drum-notation-render). Keyboard/unknown rows show nothing. -->
                <AppTag
                  v-if="h.instrument === 'percussion'"
                  variant="accent"
                  class="perc-tag"
                  :title="t('table.percussionHint')"
                  >{{ t("table.percussion") }}</AppTag
                >
              </span>
              <IdBadge :id="h.id" @click.stop />
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
          <!-- For a user upload, show the proposer's pseudo (privileged read). -->
          <span v-if="h.proposerDisplayName" class="proposer">· @{{ h.proposerDisplayName }}</span>
        </td>
        <td>
          <AppTag :variant="rowStatus(h)">{{ t(`status.${rowStatus(h)}`) }}</AppTag>
          <AppTag v-if="h.needsReview" variant="review" class="review-gap" :title="t('table.needsReviewHint')">{{
            t("table.needsReview")
          }}</AppTag>
          <!-- A reopened proposal: the proposer's justification is the tooltip, so a
               reviewer spots re-submissions in the queue without opening each one
               (change: add-score-catalog-proposal). -->
          <AppTag
            v-if="h.resubmissionNote"
            variant="review"
            class="review-gap"
            :title="t('table.resubmittedHint', { note: h.resubmissionNote })"
            >{{ t("table.resubmitted") }}</AppTag
          >
        </td>
        <!-- Download the linked MusicXML to the operator's machine. Moderator/admin
             only (provenance gate). @click.stop so it never triggers the row's select. -->
        <td v-if="canDownload" class="dl-col" @click.stop>
          <!-- Merged play / "Generate sample" (change: add-score-daily-access-rewards):
               a piece with a teaser plays the clip the app auditions on a locked piece;
               without one, the same slot generates it. Same shape as the SoundFonts. -->
          <button
            v-if="h.hasPreview"
            type="button"
            class="dl-btn sample-btn"
            :data-testid="`play-sample-${h.id}`"
            :aria-label="playingId === h.id ? t('detail.stopSample') : t('detail.playSample')"
            :title="playingId === h.id ? t('detail.stopSample') : t('detail.playSample')"
            @click="emit('playSample', h)"
          >
            {{ playingId === h.id ? "⏸" : "▶" }}
          </button>
          <button
            v-else
            type="button"
            class="dl-btn sample-btn"
            :data-testid="`generate-sample-${h.id}`"
            :disabled="sampleBusy"
            :aria-label="t('detail.generateSample')"
            :title="t('detail.generateSampleHint')"
            @click="emit('generateSample', h)"
          >
            <span v-if="generatingId === h.id" aria-hidden="true">…</span>
            <svg
              v-else
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <line x1="4" y1="10" x2="4" y2="14" />
              <line x1="8" y1="7" x2="8" y2="17" />
              <line x1="12" y1="4" x2="12" y2="20" />
              <line x1="16" y1="7" x2="16" y2="17" />
              <line x1="20" y1="10" x2="20" y2="14" />
            </svg>
          </button>
          <button
            type="button"
            class="dl-btn download-btn"
            :disabled="isDownloading(h.id)"
            :aria-label="isDownloading(h.id) ? t('table.downloading') : t('table.download')"
            :title="downloadError(h.id) ?? t('table.download')"
            @click="emit('download', h)"
          >
            <span v-if="isDownloading(h.id)" aria-hidden="true">…</span>
            <svg
              v-else
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
          </button>
          <span v-if="downloadError(h.id)" class="dl-error" role="alert">{{ downloadError(h.id) }}</span>
        </td>
      </tr>
      <tr v-if="hits.length === 0">
        <td :colspan="colCount" class="empty">{{ t("table.empty") }}</td>
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
/* Drums badge beside the title (change: add-drums-access). */
.perc-tag {
  margin-left: 0.35rem;
  vertical-align: middle;
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
.proposer {
  margin-left: 0.35rem;
  color: var(--muted);
  font-size: 0.85rem;
}

/* Community-flagged re-review marker, shown beside the row's status in the queue. */
.review-gap {
  margin-left: 0.4rem;
}

/* Per-row MusicXML download (moderator/admin only). */
.dl-col {
  width: 1%;
  white-space: nowrap;
  text-align: center;
}
.dl-btn {
  display: inline-grid;
  place-items: center;
  width: 34px;
  height: 34px;
  padding: 0;
  border: 1px solid var(--border-2);
  border-radius: 8px;
  background: var(--panel);
  color: var(--muted);
  cursor: pointer;
  transition:
    color 0.12s,
    border-color 0.12s,
    background 0.12s;
}
.dl-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: color-mix(in srgb, var(--accent) 45%, transparent);
  background: color-mix(in srgb, var(--accent-strong) 12%, transparent);
}
.dl-btn:disabled {
  cursor: default;
  opacity: 0.7;
}
.dl-btn svg {
  width: 16px;
  height: 16px;
}
.dl-error {
  display: block;
  margin-top: 0.25rem;
  font-size: 0.7rem;
  color: var(--reject);
}

.empty {
  color: var(--muted);
  text-align: center;
  padding: 2rem;
}
</style>
