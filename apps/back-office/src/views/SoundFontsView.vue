<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { SOUNDFONTS_PAGE_SIZE, useSoundFontsStore } from "@/stores/soundfonts";
import { useToastsStore } from "@/stores/toasts";
import type { AdminSoundFont } from "@/gen/score_pb";
import AppTag from "@/components/AppTag.vue";
import IdBadge from "@/components/IdBadge.vue";
import SoundFontDrawer from "@/components/SoundFontDrawer.vue";
import StatCards, { type StatItem } from "@/components/StatCards.vue";
import { currentLocale } from "@/i18n";

const store = useSoundFontsStore();
const toasts = useToastsStore();
const { t } = useI18n();

onMounted(() => {
  void store.list({ status: "", offset: 0 });
});

const vm = computed(() =>
  match(store.catalog)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, rows: [] as AdminSoundFont[] }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, rows: [] as AdminSoundFont[] }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, rows: [] as AdminSoundFont[] }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, rows: data }))
    .exhaustive(),
);

// Catalog-wide KPI cards (same look/labels as the score catalog), independent of
// the current filter/page.
const num = (v: number) => v.toLocaleString(currentLocale());
const statCards = computed<StatItem[]>(() => [
  {
    id: "total",
    label: t("stats.total"),
    value: num(store.counts.total),
    accent: "accent",
    icon: "M4 7h16M4 12h16M4 17h10",
  },
  {
    id: "accepted",
    label: t("stats.approved"),
    value: num(store.counts.accepted),
    accent: "green",
    icon: "M20 6 9 17l-5-5",
  },
  {
    id: "pending",
    label: t("stats.pending"),
    value: num(store.counts.pending),
    accent: "amber",
    icon: "M12 7v5l3 2M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Z",
  },
]);

const acting = computed(() => store.op.status === "loading");

// "Generate sample" (change: add-soundfont-entitlement-previews): (re)render a font's
// server-side preview clip. Its state is the store's `preview` Async union, matched
// exhaustively so a forgotten branch is a compile error.
const previewVm = computed(() =>
  match(store.preview)
    .with({ status: "idle" }, () => ({ busy: false, error: null as string | null, done: false }))
    .with({ status: "loading" }, () => ({ busy: true, error: null, done: false }))
    .with({ status: "error" }, ({ error }) => ({ busy: false, error, done: false }))
    .with({ status: "success" }, () => ({ busy: false, error: null, done: true }))
    .exhaustive(),
);
/** Whether the given row is the one currently (re)generating its sample. */
function generatingSample(id: string): boolean {
  return store.previewTarget === id && previewVm.value.busy;
}
async function regenerateSample(row: AdminSoundFont) {
  const outcome = await store.regeneratePreview(row.id);
  if (outcome.status === "error") toasts.error(outcome.error);
  else toasts.success(t("soundfonts.sampleGenerated"));
}

// Moderation review (change: add-soundfont-moderation): a back-office-only status
// filter and accept/reject actions. Filtering + paging are server-side.
type StatusFilter = "all" | "pending" | "accepted" | "rejected";
const statusFilter = ref<StatusFilter>("all");
function selectStatus(f: StatusFilter) {
  statusFilter.value = f;
  void store.list({ status: f === "all" ? "" : f, offset: 0 });
}

// Pagination (server-side): a window over `total` rows.
const pageStart = computed(() => (store.total === 0 ? 0 : store.offset + 1));
const pageEnd = computed(() => store.offset + vm.value.rows.length);
const canPrev = computed(() => store.offset > 0);
const canNext = computed(() => store.offset + SOUNDFONTS_PAGE_SIZE < store.total);
function prevPage() {
  void store.list({ offset: Math.max(0, store.offset - SOUNDFONTS_PAGE_SIZE) });
}
function nextPage() {
  void store.list({ offset: store.offset + SOUNDFONTS_PAGE_SIZE });
}

async function setStatus(id: string, status: string) {
  const outcome = await store.setModerationStatus(id, status);
  if (outcome.status === "error") toasts.error(outcome.error);
}

// Reject with a motive (change: add-soundfont-uploader-attribution): clicking
// "Reject" opens an inline reason input on that row; confirming sends the reason
// with the evaluate call (it is surfaced back to the uploader). Mirrors the score
// review's reject-reason input.
const rejectTargetId = ref<string | null>(null);
const rejectReason = ref("");
function startReject(id: string) {
  rejectTargetId.value = id;
  rejectReason.value = "";
}
function cancelReject() {
  rejectTargetId.value = null;
  rejectReason.value = "";
}
async function confirmReject(id: string) {
  const reason = rejectReason.value.trim() || undefined;
  const outcome = await store.setModerationStatus(id, "rejected", reason);
  if (outcome.status === "error") toasts.error(outcome.error);
  else cancelReject();
}

// Normalise a row's (possibly empty) moderation status to a known badge variant.
function statusOf(row: AdminSoundFont): "pending" | "accepted" | "rejected" {
  const s = row.moderationStatus || "pending";
  if (s === "accepted") return "accepted";
  if (s === "rejected") return "rejected";
  return "pending";
}

// Per-row audition (change: add-soundfont-entitlement-previews): play the SAME
// server-rendered preview clip the app plays — no font download, no wasm synth. The
// play control and "Generate sample" are merged: a font with no preview shows
// "Generate sample"; once a preview exists the slot becomes a play button. One row
// plays at a time.
const previewingId = ref<string | null>(null);
let previewAudio: HTMLAudioElement | null = null;
let previewUrl: string | null = null;

function stopPreview() {
  previewAudio?.pause();
  if (previewUrl) {
    URL.revokeObjectURL(previewUrl);
    previewUrl = null;
  }
  previewAudio = null;
  previewingId.value = null;
}

async function togglePlay(row: AdminSoundFont) {
  if (previewingId.value === row.id) {
    stopPreview();
    return;
  }
  stopPreview();
  let bytes: Uint8Array;
  try {
    bytes = await store.previewClip(row.id);
  } catch {
    // The preview vanished (race with a delete/regenerate) — nothing to play.
    return;
  }
  previewUrl = URL.createObjectURL(new Blob([bytes as BlobPart], { type: "audio/wav" }));
  previewAudio = new Audio(previewUrl);
  previewAudio.addEventListener("ended", stopPreview);
  previewingId.value = row.id;
  void previewAudio.play();
}

onBeforeUnmount(stopPreview);

const drawerMode = ref<"create" | "edit" | null>(null);
const drawerEntry = ref<AdminSoundFont | null>(null);
function openCreate() {
  drawerEntry.value = null;
  drawerMode.value = "create";
}
function openEdit(row: AdminSoundFont) {
  drawerEntry.value = row;
  drawerMode.value = "edit";
}
function closeDrawer() {
  drawerMode.value = null;
}

async function remove(id: string) {
  if (!window.confirm(t("soundfonts.confirmRemove"))) return;
  const outcome = await store.remove(id);
  if (outcome.status === "error") toasts.error(outcome.error);
}

// Plain-language gloss of a licence acronym, shown as a hover tooltip on the licence
// cell (same mapping as the drawer's help text). Empty for an unrecognised licence.
function licenseDesc(license: string): string {
  if (license.startsWith("CC0")) return t("soundfonts.licenseDesc.cc0");
  if (license.startsWith("CC-BY-SA")) return t("soundfonts.licenseDesc.ccbysa");
  if (license.startsWith("CC-BY")) return t("soundfonts.licenseDesc.ccby");
  return "";
}
</script>

<template>
  <section class="soundfonts">
    <header class="head">
      <div>
        <h1>{{ t("soundfonts.title") }}</h1>
        <p class="intro">{{ t("soundfonts.intro") }}</p>
      </div>
      <button type="button" class="primary" @click="openCreate">{{ t("soundfonts.add") }}</button>
    </header>

    <!-- List-level action results (accept/reject/delete/generate) surface as global
         toasts (ToastHost); the add/edit drawer keeps its own inline error. -->
    <StatCards :items="statCards" />

    <div class="filters" role="tablist" :aria-label="t('soundfonts.filter.label')">
      <button
        v-for="f in ['all', 'pending', 'accepted', 'rejected'] as const"
        :key="f"
        type="button"
        role="tab"
        :aria-selected="statusFilter === f"
        :class="{ chip: true, active: statusFilter === f }"
        @click="selectStatus(f)"
      >
        {{ t(`soundfonts.filter.${f}`) }}
      </button>
    </div>

    <p v-if="vm.loading" class="muted">…</p>
    <p v-else-if="vm.error" class="error" role="alert">{{ vm.error }}</p>
    <p v-else-if="vm.rows.length === 0" class="muted">{{ t("soundfonts.empty") }}</p>

    <div v-else class="table-card">
      <table>
        <thead>
          <tr>
            <th>{{ t("soundfonts.label") }}</th>
            <th>{{ t("soundfonts.statusCol") }}</th>
            <th>{{ t("soundfonts.instrument") }}</th>
            <th>{{ t("soundfonts.license") }}</th>
            <th>{{ t("soundfonts.attribution") }}</th>
            <th class="actions-col">{{ t("soundfonts.actionsCol") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in vm.rows" :key="row.id">
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
                    {{ row.label }}
                    <!-- Privileged uploader pseudo (change:
                         add-soundfont-uploader-attribution); empty for a seeded font. -->
                    <span v-if="row.uploaderDisplayName" class="uploader"
                      >· {{ t("soundfonts.proposedBy", { name: row.uploaderDisplayName }) }}</span
                    >
                  </span>
                  <IdBadge :id="row.id" />
                  <!-- The uploader's justification when a rejected font was re-proposed. -->
                  <span v-if="row.resubmissionNote" class="resub">{{
                    t("soundfonts.resubmission", { note: row.resubmissionNote })
                  }}</span>
                </span>
              </div>
            </td>
            <td>
              <AppTag :variant="statusOf(row)">{{ t(`soundfonts.status.${statusOf(row)}`) }}</AppTag>
            </td>
            <td>
              <AppTag variant="neutral">{{ t(`soundfonts.instr.${row.instrument || "piano"}`) }}</AppTag>
            </td>
            <td>
              <span v-if="licenseDesc(row.license)" class="license-help" :title="licenseDesc(row.license)">{{
                row.license
              }}</span>
              <template v-else>{{ row.license }}</template>
            </td>
            <td>{{ row.attribution }}</td>
            <td class="actions-col">
              <div class="row-actions">
                <!-- Merged play / "Generate sample": a font with a preview plays the clip;
                   without one, the same slot generates it (change:
                   add-soundfont-entitlement-previews). -->
                <button
                  v-if="row.hasPreview"
                  type="button"
                  class="icon-btn"
                  :aria-label="t('soundfonts.play')"
                  :title="t('soundfonts.play')"
                  @click="togglePlay(row)"
                >
                  {{ previewingId === row.id ? "⏸" : "▶" }}
                </button>
                <button
                  v-else
                  type="button"
                  class="icon-btn"
                  :disabled="previewVm.busy"
                  :aria-label="t('soundfonts.generateSample')"
                  :title="t('soundfonts.generateSampleHint')"
                  @click="regenerateSample(row)"
                >
                  <span v-if="generatingSample(row.id)" aria-hidden="true">…</span>
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
                  v-if="(row.moderationStatus || 'pending') !== 'accepted'"
                  type="button"
                  class="btn-sm accept"
                  :disabled="acting"
                  @click="setStatus(row.id, 'accepted')"
                >
                  {{ t("soundfonts.accept") }}
                </button>
                <!-- Reject opens an inline reason input (change:
                     add-soundfont-uploader-attribution): the motive is optional but
                     surfaced back to the uploader when given. -->
                <template v-if="(row.moderationStatus || 'pending') !== 'rejected'">
                  <template v-if="rejectTargetId === row.id">
                    <input
                      :id="`soundfont-reject-reason-${row.id}`"
                      v-model="rejectReason"
                      class="reason"
                      type="text"
                      :aria-label="t('soundfonts.rejectReasonPlaceholder')"
                      :placeholder="t('soundfonts.rejectReasonPlaceholder')"
                      :disabled="acting"
                      @keyup.enter="confirmReject(row.id)"
                      @keyup.escape="cancelReject"
                    />
                    <button
                      type="button"
                      class="btn-sm reject"
                      :disabled="acting"
                      @click="confirmReject(row.id)"
                    >
                      {{ t("soundfonts.reject") }}
                    </button>
                    <button type="button" class="btn-sm" :disabled="acting" @click="cancelReject">
                      {{ t("soundfonts.cancel") }}
                    </button>
                  </template>
                  <button
                    v-else
                    type="button"
                    class="btn-sm reject"
                    :disabled="acting"
                    @click="startReject(row.id)"
                  >
                    {{ t("soundfonts.reject") }}
                  </button>
                </template>
                <button
                  type="button"
                  class="icon-btn"
                  :aria-label="t('soundfonts.edit')"
                  :title="t('soundfonts.edit')"
                  @click="openEdit(row)"
                >
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M12 20h9" />
                    <path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" />
                  </svg>
                </button>
                <button
                  type="button"
                  class="icon-btn danger"
                  :disabled="acting"
                  :aria-label="t('soundfonts.remove')"
                  :title="t('soundfonts.remove')"
                  @click="remove(row.id)"
                >
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    aria-hidden="true"
                  >
                    <polyline points="3 6 5 6 21 6" />
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  </svg>
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <nav v-if="!vm.loading && !vm.error && store.total > 0" class="pager" :aria-label="t('soundfonts.pager.label')">
      <button type="button" :disabled="!canPrev" @click="prevPage">{{ t("soundfonts.pager.prev") }}</button>
      <span class="range">{{
        t("soundfonts.pager.range", { start: pageStart, end: pageEnd, total: store.total })
      }}</span>
      <button type="button" :disabled="!canNext" @click="nextPage">{{ t("soundfonts.pager.next") }}</button>
    </nav>

    <SoundFontDrawer :mode="drawerMode" :entry="drawerEntry" @close="closeDrawer" />
  </section>
</template>

<style scoped>
.soundfonts {
  padding: 1.5rem;
}
.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}
.intro {
  color: var(--muted);
  margin: 0.25rem 0 1rem;
  max-width: 46rem;
}
.primary {
  background: var(--accent-strong);
  color: #fff;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.4rem;
  cursor: pointer;
  white-space: nowrap;
}

/* Title cell: a thumbnail + label + id badge, matching the catalog table. */
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
  gap: 0.15rem;
  line-height: 1.25;
}
.t-name {
  font-weight: 600;
  color: var(--text);
}
/* Privileged uploader pseudo next to the label (change:
   add-soundfont-uploader-attribution). */
.uploader {
  font-weight: 400;
  color: var(--muted);
  font-size: 0.85em;
}
/* The uploader's re-submission justification, shown to the reviewer. */
.resub {
  color: var(--muted);
  font-size: 0.8rem;
  font-style: italic;
}
/* Inline rejection-reason input (mirrors the score review's .reason). */
.reason {
  flex: 1 1 12rem;
  min-width: 8rem;
  padding: 0.4rem 0.6rem;
  border: 1px solid var(--border);
  border-radius: var(--radius-md, 6px);
  background: var(--panel);
  color: var(--text);
  font-size: 0.8rem;
}

/* Cue that the licence has a hover explanation (native title tooltip). */
.license-help {
  cursor: help;
  text-decoration: underline dotted;
  text-underline-offset: 2px;
}

/* Actions: a compact, single-line row (the card scrolls if it overflows). */
.actions-col {
  width: 1%;
}
.row-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.35rem;
}
.btn-sm {
  padding: 0.32rem 0.6rem;
  font-size: 0.8rem;
  white-space: nowrap;
}
/* accept / reject colours come from the global .accept / .reject classes. */
.icon-btn {
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
.icon-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: color-mix(in srgb, var(--accent) 45%, transparent);
  background: color-mix(in srgb, var(--accent-strong) 12%, transparent);
}
.icon-btn.danger:hover:not(:disabled) {
  color: var(--coral);
  border-color: color-mix(in srgb, var(--coral) 45%, transparent);
  background: color-mix(in srgb, var(--coral) 12%, transparent);
}
.icon-btn svg {
  width: 16px;
  height: 16px;
}
.icon-btn:disabled {
  cursor: default;
  opacity: 0.6;
}

.filters {
  display: flex;
  gap: 0.4rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}
.chip {
  border: 1px solid var(--border-2);
  background: transparent;
  color: inherit;
  padding: 0.3rem 0.7rem;
  border-radius: 999px;
  cursor: pointer;
}
.chip.active {
  /* A concrete darker accent (not color-mix, which the analyzer can't resolve) so
     white label text meets the WCAG AA contrast ratio (~5:1). */
  background: #4a37a8;
  border-color: var(--accent);
  color: #fff;
}
.chip .count {
  display: inline-block;
  margin-left: 0.35rem;
  min-width: 1.2em;
  padding: 0 0.35em;
  border-radius: 999px;
  background: var(--coral);
  color: #fff;
  font-size: 0.78em;
}
.error {
  color: var(--coral);
}
.muted {
  color: var(--muted);
}
.pager {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-top: 1rem;
}
.pager .range {
  color: var(--muted);
  font-variant-numeric: tabular-nums;
}
</style>
