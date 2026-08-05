<script setup lang="ts">
import { computed, onMounted, ref, shallowRef } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { useScorePlayer } from "@/composables/useScorePlayer";
import { sampleScoreBytes } from "@/lib/sampleScore";
import { SOUNDFONTS_PAGE_SIZE, useSoundFontsStore } from "@/stores/soundfonts";
import type { AdminSoundFont } from "@/gen/score_pb";
import AppTag from "@/components/AppTag.vue";
import SoundFontDrawer from "@/components/SoundFontDrawer.vue";
import StatCards, { type StatItem } from "@/components/StatCards.vue";
import { currentLocale } from "@/i18n";

const store = useSoundFontsStore();
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
const opError = computed(() => (store.op.status === "error" ? store.op.error : null));

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
  await store.regeneratePreview(row.id);
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
  await store.setModerationStatus(id, status);
}

// Normalise a row's (possibly empty) moderation status to a known badge variant.
function statusOf(row: AdminSoundFont): "pending" | "accepted" | "rejected" {
  const s = row.moderationStatus || "pending";
  if (s === "accepted") return "accepted";
  if (s === "rejected") return "rejected";
  return "pending";
}

// Drawer for create/edit (right-to-left).
// Per-row audition: play the shared sample (Ode to Joy — same as the app) with a
// row's font. One row plays at a time.
const previewingId = ref<string | null>(null);
const previewScore = shallowRef<Uint8Array | null>(sampleScoreBytes);
const previewSf2 = shallowRef<Uint8Array | null>(null);
const rowPlayer = useScorePlayer(previewScore, previewSf2);

async function togglePlay(row: AdminSoundFont) {
  if (previewingId.value === row.id) {
    rowPlayer.stop();
    previewingId.value = null;
    return;
  }
  rowPlayer.stop();
  previewingId.value = row.id;
  try {
    previewSf2.value = await store.fontBytes(row.id);
  } catch {
    previewingId.value = null;
    return;
  }
  rowPlayer.playFrom(0);
}

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
  await store.remove(id);
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

    <!-- The op error is shared with the add/edit drawer; while it's open the drawer
         owns the message, so only surface it here (above the grid) for list-level
         actions (accept/reject/delete) when the drawer is closed. -->
    <p v-if="opError && drawerMode === null" class="error" role="alert">{{ opError }}</p>
    <!-- "Generate sample" outcome (change: add-soundfont-entitlement-previews). -->
    <p v-if="previewVm.error" class="error" role="alert">{{ previewVm.error }}</p>
    <p v-else-if="previewVm.done" class="ok" role="status">{{ t("soundfonts.sampleGenerated") }}</p>

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

    <table v-else class="grid">
      <thead>
        <tr>
          <th>{{ t("soundfonts.label") }}</th>
          <th>{{ t("soundfonts.statusCol") }}</th>
          <th>{{ t("soundfonts.instrument") }}</th>
          <th>{{ t("soundfonts.license") }}</th>
          <th>{{ t("soundfonts.attribution") }}</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="row in vm.rows" :key="row.id">
          <td>{{ row.label }}</td>
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
          <td class="actions">
            <button
              type="button"
              class="play-row"
              :aria-label="t('soundfonts.play')"
              :title="t('soundfonts.play')"
              @click="togglePlay(row)"
            >
              {{ previewingId === row.id && rowPlayer.playing.value ? "⏸" : "▶" }}
            </button>
            <button
              v-if="(row.moderationStatus || 'pending') !== 'accepted'"
              type="button"
              class="accept"
              :disabled="acting"
              @click="setStatus(row.id, 'accepted')"
            >
              {{ t("soundfonts.accept") }}
            </button>
            <button
              v-if="(row.moderationStatus || 'pending') !== 'rejected'"
              type="button"
              class="reject"
              :disabled="acting"
              @click="setStatus(row.id, 'rejected')"
            >
              {{ t("soundfonts.reject") }}
            </button>
            <button
              type="button"
              class="sample"
              :disabled="previewVm.busy"
              :title="t('soundfonts.generateSampleHint')"
              @click="regenerateSample(row)"
            >
              {{ generatingSample(row.id) ? t("soundfonts.generatingSample") : t("soundfonts.generateSample") }}
            </button>
            <button type="button" @click="openEdit(row)">{{ t("soundfonts.edit") }}</button>
            <button type="button" :disabled="acting" @click="remove(row.id)">{{ t("soundfonts.remove") }}</button>
          </td>
        </tr>
      </tbody>
    </table>

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
  max-width: 60rem;
}
.head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}
.intro {
  color: var(--muted, #888);
  margin: 0.25rem 0 1rem;
  max-width: 46rem;
}
.primary {
  background: var(--accent, #7c5cff);
  color: #fff;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.4rem;
  cursor: pointer;
  white-space: nowrap;
}
.grid {
  width: 100%;
  border-collapse: collapse;
}
.grid th,
.grid td {
  text-align: left;
  padding: 0.5rem;
  border-bottom: 1px solid var(--outline, #333);
  vertical-align: middle;
}
.mono {
  font-family: ui-monospace, monospace;
  font-size: 0.85em;
}
/* Cue that the licence has a hover explanation (native title tooltip). */
.license-help {
  cursor: help;
  text-decoration: underline dotted;
  text-underline-offset: 2px;
}
.actions {
  display: flex;
  gap: 0.4rem;
  flex-wrap: wrap;
}
.actions .accept {
  border-color: var(--ok, #2e7d32);
  color: var(--ok, #2e7d32);
}
.actions .reject {
  border-color: var(--danger, #c0392b);
  color: var(--danger, #c0392b);
}
.filters {
  display: flex;
  gap: 0.4rem;
  margin-bottom: 1rem;
  flex-wrap: wrap;
}
.chip {
  border: 1px solid var(--outline, #333);
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
  border-color: var(--accent, #7c5cff);
  color: #fff;
}
.chip .count {
  display: inline-block;
  margin-left: 0.35rem;
  min-width: 1.2em;
  padding: 0 0.35em;
  border-radius: 999px;
  background: var(--danger, #c0392b);
  color: #fff;
  font-size: 0.78em;
}
.error {
  color: var(--danger, #c0392b);
}
.ok {
  color: var(--ok, #2e7d32);
}
.muted {
  color: var(--muted, #888);
}
.pager {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  margin-top: 1rem;
}
.pager .range {
  color: var(--muted, #888);
  font-variant-numeric: tabular-nums;
}
</style>
