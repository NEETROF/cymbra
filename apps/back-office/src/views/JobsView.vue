<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import {
  PAGE_SIZE,
  type JobStats,
  type PeriodPreset,
  type QueuedJobRow,
  useJobsStore,
  windowFor,
  toLocalInput,
} from "@/stores/jobs";
import { JobState } from "@/gen/jobs_admin_pb";
import { currentLocale } from "@/i18n";
import AppTag from "@/components/AppTag.vue";
import ConfirmDialog from "@/components/ConfirmDialog.vue";
import StatCards, { type StatItem } from "@/components/StatCards.vue";
import TablePager from "@/components/TablePager.vue";

// The back-office "Jobs" screen (change: add-admin-jobs-console, task 6.3). It NEVER calls
// the API directly — the Pinia store does, behind the injectable client seam — and each
// async resource is a single ts-pattern union, matched exhaustively here. Global-scope
// gated by the router (meta.adminScope = "global"); every RPC is re-gated server-side.
// A cancellation is fired, not awaited: its outcome arrives as a toast from the store, and
// the dialog closes when the store's `cancelling` returns to null.

const store = useJobsStore();
const { t } = useI18n();

type TagVariant = "accent" | "neutral" | "pending" | "accepted" | "rejected" | "review";
const STATES: { value: JobState; key: string; tag: TagVariant }[] = [
  { value: JobState.RUNNING, key: "running", tag: "accent" },
  { value: JobState.READY, key: "ready", tag: "accepted" },
  { value: JobState.SCHEDULED, key: "scheduled", tag: "neutral" },
  { value: JobState.RETRY_WAIT, key: "retry_wait", tag: "pending" },
  { value: JobState.BLOCKED, key: "blocked", tag: "review" },
  { value: JobState.EXHAUSTED, key: "exhausted", tag: "rejected" },
];
const stateInfo = (s: JobState) => STATES.find((x) => x.value === s);

const PRESETS: { id: PeriodPreset; key: string }[] = [
  { id: "1h", key: "jobs.period.lastHour" },
  { id: "24h", key: "jobs.period.last24h" },
  { id: "7d", key: "jobs.period.last7d" },
  { id: "30d", key: "jobs.period.last30d" },
  { id: "custom", key: "jobs.period.custom" },
];

// 24×24 stroke icons (Lucide-style), like the nav.
const ICON = {
  layers: "M12 2 2 7l10 5 10-5-10-5ZM2 17l10 5 10-5M2 12l10 5 10-5",
  play: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20ZM10 8l6 4-6 4V8Z",
  check: "M20 6 9 17l-5-5",
  retry: "M3 12a9 9 0 0 1 15-6.7L21 8M21 3v5h-5M21 12a9 9 0 0 1-15 6.7L3 16M3 21v-5h5",
  clock: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20ZM12 6v6l4 2",
  alert: "M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20ZM12 8v4M12 16h.01",
  x: "M18 6 6 18M6 6l12 12",
  inbox:
    "M22 12h-6l-2 3h-4l-2-3H2M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11Z",
  timer: "M10 2h4M12 14l3-3M12 22a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z",
};

onMounted(() => {
  void store.loadKinds();
  void store.load();
});

const pageVm = computed(() =>
  match(store.page)
    .with({ status: "idle" }, () => ({
      loading: true,
      error: null as string | null,
      jobs: [] as QueuedJobRow[],
      total: 0,
    }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, jobs: [] as QueuedJobRow[], total: 0 }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, jobs: [] as QueuedJobRow[], total: 0 }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, jobs: data.jobs, total: data.total }))
    .exhaustive(),
);

const statsVm = computed(() =>
  match(store.stats)
    .with({ status: "idle" }, () => ({ error: null as string | null, data: null as JobStats | null }))
    .with({ status: "loading" }, () => ({ error: null, data: null }))
    .with({ status: "error" }, ({ error }) => ({ error, data: null }))
    .with({ status: "success" }, ({ data }) => ({ error: null, data }))
    .exhaustive(),
);

const kindOptions = computed(() =>
  match(store.kinds)
    .with({ status: "success" }, ({ data }) => data.map((k) => k.name))
    .otherwise(() => [] as string[]),
);

const num = (v: number | undefined) => (v === undefined ? "—" : v.toLocaleString(currentLocale()));

/** A compact human duration: ms under a second, seconds under a minute, then minutes. */
function fmtDuration(ms: number | null | undefined): string {
  if (ms === null || ms === undefined) return "—";
  const n = (v: number) => v.toLocaleString(currentLocale(), { maximumFractionDigits: 1 });
  if (ms < 1000) return t("jobs.duration.ms", { n: n(Math.round(ms)) });
  if (ms < 60_000) return t("jobs.duration.s", { n: n(ms / 1000) });
  return t("jobs.duration.min", { n: n(ms / 60_000) });
}

const queueCards = computed<StatItem[]>(() => {
  const q = statsVm.value.data?.queue;
  return [
    { id: "queue-total", label: t("jobs.queue.total"), value: num(q?.total), accent: "accent", icon: ICON.layers },
    { id: "queue-running", label: t("jobs.queue.running"), value: num(q?.running), accent: "accent", icon: ICON.play },
    { id: "queue-ready", label: t("jobs.queue.ready"), value: num(q?.ready), accent: "green", icon: ICON.check },
    { id: "queue-retry", label: t("jobs.queue.retry"), value: num(q?.retryWait), accent: "amber", icon: ICON.retry },
    {
      id: "queue-waiting",
      label: t("jobs.queue.waiting"),
      value: q ? num(q.scheduled + q.blocked) : "—",
      accent: "accent",
      icon: ICON.clock,
    },
    {
      id: "queue-exhausted",
      label: t("jobs.queue.exhausted"),
      value: num(q?.exhausted),
      accent: "red",
      icon: ICON.alert,
    },
  ];
});

const periodCards = computed<StatItem[]>(() => {
  const p = statsVm.value.data?.period;
  return [
    {
      id: "period-completed",
      label: t("jobs.periodCards.completed"),
      value: num(p?.completed),
      accent: "green",
      icon: ICON.check,
    },
    {
      id: "period-failed",
      label: t("jobs.periodCards.failed"),
      value: num(p?.failedAttempts),
      accent: "amber",
      icon: ICON.retry,
    },
    {
      id: "period-dead",
      label: t("jobs.periodCards.dead"),
      value: num(p?.deadLettered),
      accent: "red",
      icon: ICON.inbox,
    },
    {
      id: "period-cancelled",
      label: t("jobs.periodCards.cancelled"),
      value: num(p?.cancelled),
      accent: "accent",
      icon: ICON.x,
    },
    {
      id: "period-avg",
      label: t("jobs.periodCards.avg"),
      value: fmtDuration(p?.avgRunMs),
      accent: "accent",
      icon: ICON.timer,
    },
  ];
});

const absolute = (ms: number) => new Date(ms).toLocaleString(currentLocale());

/** The history starts after the window does: earlier figures are unknown, not zero. */
const historyNote = computed(() => {
  const since = statsVm.value.data?.historySince ?? null;
  if (since === null) return null;
  const { fromMs } = windowFor(store.period, Date.now());
  if (!(fromMs < since)) return null;
  const date = new Date(since).toLocaleString(currentLocale(), { dateStyle: "medium", timeStyle: "short" });
  return t("jobs.historyNote", { date });
});

/** "3 minutes ago" / "in 20 seconds" — the absolute time goes in the title. */
function relative(ms: number | null): string {
  if (ms === null) return "—";
  const diff = ms - Date.now();
  const abs = Math.abs(diff);
  const rtf = new Intl.RelativeTimeFormat(currentLocale(), { numeric: "auto" });
  if (abs < 60_000) return rtf.format(Math.round(diff / 1000), "second");
  if (abs < 3_600_000) return rtf.format(Math.round(diff / 60_000), "minute");
  if (abs < 86_400_000) return rtf.format(Math.round(diff / 3_600_000), "hour");
  return rtf.format(Math.round(diff / 86_400_000), "day");
}

// ---- filters ----
const stateFilter = ref<JobState>(store.params.state);
const kindFilter = ref(store.params.kind);
function applyFilters() {
  void store.setFilters({ state: stateFilter.value, kind: kindFilter.value });
}

// ---- period ----
const customFrom = ref(store.period.from);
const customTo = ref(store.period.to);
watch(
  () => [store.period.from, store.period.to] as const,
  ([from, to]) => {
    customFrom.value = from;
    customTo.value = to;
  },
);
const maxInput = computed(() => toLocalInput(Date.now()));
function choosePreset(preset: PeriodPreset) {
  void store.setPeriod({ preset });
}
function applyCustom() {
  void store.setPeriod({ preset: "custom", from: customFrom.value, to: customTo.value });
}

// ---- cancellation ----
const pending = ref<QueuedJobRow | null>(null);
const confirmMessage = computed(() =>
  pending.value ? t("jobs.confirmCancel", { kind: pending.value.kind, id: pending.value.id.slice(0, 8) }) : null,
);
function confirmCancel() {
  if (pending.value) void store.cancel(pending.value.id);
}
function dismissCancel() {
  if (store.cancelling === null) pending.value = null;
}
// The store clears `cancelling` once the outcome is toasted and the table re-read.
watch(
  () => store.cancelling,
  (now, before) => {
    if (before !== null && now === null) pending.value = null;
  },
);

// ---- refresh ----
const AUTO_REFRESH_MS = 15_000;
const autoRefresh = ref(false);
let timer: ReturnType<typeof setInterval> | null = null;
function stopTimer() {
  if (timer !== null) clearInterval(timer);
  timer = null;
}
/** A tick is skipped while the tab is hidden, the dialog is open or a cancel is in flight:
 *  a row must not move under the operator while they are confirming an action on it. */
function tick() {
  if (document.hidden || pending.value !== null || store.cancelling !== null) return;
  void store.refresh();
}
watch(autoRefresh, (on) => {
  stopTimer();
  if (on) timer = setInterval(tick, AUTO_REFRESH_MS);
});
onUnmounted(stopTimer);

function refresh() {
  void store.refresh();
}
function goTo(offset: number) {
  void store.goToPage(offset);
}
</script>

<template>
  <section class="jobs">
    <header class="page-head">
      <h1 class="page-title">{{ t("jobs.title") }}</h1>
      <div class="head-actions">
        <label class="auto">
          <input v-model="autoRefresh" type="checkbox" data-testid="jobs-autorefresh" />
          {{ t("jobs.autoRefresh") }}
        </label>
        <button type="button" data-testid="jobs-refresh" @click="refresh">{{ t("jobs.refresh") }}</button>
      </div>
    </header>

    <!-- The queue as it is now. -->
    <h2 class="section">{{ t("jobs.queueTitle") }}</h2>
    <StatCards :items="queueCards" />

    <!-- Figures over a selectable window. -->
    <h2 class="section">{{ t("jobs.periodTitle") }}</h2>
    <div class="period" role="group" :aria-label="t('jobs.period.label')">
      <button
        v-for="p in PRESETS"
        :key="p.id"
        type="button"
        :class="{ active: store.period.preset === p.id }"
        :aria-pressed="store.period.preset === p.id"
        :data-testid="`period-${p.id}`"
        @click="choosePreset(p.id)"
      >
        {{ t(p.key) }}
      </button>
    </div>
    <form v-if="store.period.preset === 'custom'" class="custom" @submit.prevent="applyCustom">
      <label>
        <span>{{ t("jobs.period.from") }}</span>
        <input v-model="customFrom" type="datetime-local" :max="maxInput" data-testid="period-from" />
      </label>
      <label>
        <span>{{ t("jobs.period.to") }}</span>
        <input v-model="customTo" type="datetime-local" :max="maxInput" data-testid="period-to" />
      </label>
      <button type="submit" data-testid="period-apply">{{ t("jobs.period.apply") }}</button>
    </form>
    <p v-if="statsVm.error" class="error" role="alert" data-testid="stats-error">{{ statsVm.error }}</p>
    <StatCards :items="periodCards" />
    <p v-if="historyNote" class="muted note" data-testid="history-note">{{ historyNote }}</p>

    <!-- Table filters. -->
    <div class="filters">
      <label>
        <span>{{ t("jobs.filters.state") }}</span>
        <select v-model="stateFilter" data-testid="filter-state" @change="applyFilters">
          <option :value="JobState.UNSPECIFIED">{{ t("jobs.filters.any") }}</option>
          <option v-for="s in STATES" :key="s.value" :value="s.value">{{ t(`jobs.states.${s.key}`) }}</option>
        </select>
      </label>
      <label>
        <span>{{ t("jobs.filters.kind") }}</span>
        <select v-model="kindFilter" data-testid="filter-kind" @change="applyFilters">
          <option value="">{{ t("jobs.filters.any") }}</option>
          <option v-for="k in kindOptions" :key="k" :value="k">{{ k }}</option>
        </select>
      </label>
      <span v-if="!pageVm.loading && !pageVm.error" class="muted matching" data-testid="jobs-total">
        {{ t("jobs.matching", { n: num(pageVm.total) }) }}
      </span>
    </div>

    <p v-if="pageVm.error" class="error" role="alert" data-testid="jobs-error">{{ pageVm.error }}</p>

    <div class="table-card">
      <table data-testid="jobs-table">
        <thead>
          <tr>
            <th>{{ t("jobs.columns.kind") }}</th>
            <th>{{ t("jobs.columns.state") }}</th>
            <th class="n">{{ t("jobs.columns.attempts") }}</th>
            <th>{{ t("jobs.columns.enqueued") }}</th>
            <th>{{ t("jobs.columns.nextAttempt") }}</th>
            <th>{{ t("jobs.columns.started") }}</th>
            <th>{{ t("jobs.columns.id") }}</th>
            <!-- The cell stays in the layout (the header row spans the whole table);
                 only its label is visually hidden. -->
            <th>
              <span class="sr-only">{{ t("jobs.columns.actions") }}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="job in pageVm.jobs" :key="job.id" data-testid="job-row">
            <td>
              <div class="kind">{{ job.kind }}</div>
              <div class="muted channel">{{ job.channel }}</div>
            </td>
            <td>
              <AppTag :variant="stateInfo(job.state)?.tag ?? 'neutral'" data-testid="job-state">
                {{ stateInfo(job.state) ? t(`jobs.states.${stateInfo(job.state)?.key}`) : "—" }}
              </AppTag>
            </td>
            <td class="n">{{ job.attemptsMade }} / {{ job.attemptsLeft }}</td>
            <td :title="absolute(job.enqueuedAt)">{{ relative(job.enqueuedAt) }}</td>
            <td :title="job.nextAttemptAt === null ? undefined : absolute(job.nextAttemptAt)">
              {{ relative(job.nextAttemptAt) }}
            </td>
            <td :title="job.startedAt === null ? undefined : absolute(job.startedAt)">{{ relative(job.startedAt) }}</td>
            <td>
              <code class="id" :title="job.id">{{ job.id.slice(0, 8) }}</code>
            </td>
            <td class="actions">
              <button
                v-if="job.cancellable"
                type="button"
                class="reject"
                data-testid="job-cancel"
                :disabled="store.cancelling !== null"
                @click="pending = job"
              >
                {{ t("jobs.cancel") }}
              </button>
            </td>
          </tr>
          <tr v-if="pageVm.loading">
            <td colspan="8" class="empty" data-testid="jobs-loading">{{ t("jobs.loading") }}</td>
          </tr>
          <tr v-else-if="pageVm.jobs.length === 0 && !pageVm.error">
            <td colspan="8" class="empty" data-testid="jobs-empty">{{ t("jobs.empty") }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <TablePager :offset="store.params.offset" :limit="PAGE_SIZE" :total="pageVm.total" @page="goTo" />

    <ConfirmDialog
      :message="confirmMessage"
      :busy="store.cancelling !== null"
      :confirm-label="t('jobs.cancel')"
      :cancel-label="t('jobs.keep')"
      @confirm="confirmCancel"
      @cancel="dismissCancel"
    />
  </section>
</template>

<style scoped>
.head-actions {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}
.auto {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.9rem;
  color: var(--muted);
}
.section {
  margin: 0.5rem 0 0.75rem;
  font-size: 1rem;
  color: var(--muted);
}
.period {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin-bottom: 0.75rem;
}
.period button {
  padding: 0.35rem 0.75rem;
  font-size: 0.85rem;
}
.period button.active {
  border-color: var(--accent);
  color: var(--accent);
}
.custom,
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: end;
  margin-bottom: 1rem;
}
.custom label,
.filters label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8rem;
  color: var(--muted);
}
.matching {
  font-size: 0.85rem;
  padding-bottom: 0.55rem;
}
.note {
  margin: -0.75rem 0 1.25rem;
  font-size: 0.85rem;
}
.kind {
  font-weight: 600;
}
.channel {
  font-family: var(--mono);
  font-size: 0.72rem;
}
.id {
  font-family: var(--mono);
  font-size: 0.8rem;
  color: var(--muted);
}
th.n,
td.n {
  text-align: right;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.actions {
  text-align: right;
}
.actions button {
  padding: 0.3rem 0.7rem;
  font-size: 0.85rem;
  white-space: nowrap;
}
.empty {
  color: var(--muted);
  text-align: center;
  padding: 2rem;
}
</style>
