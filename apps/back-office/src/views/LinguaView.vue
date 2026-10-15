<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import {
  comprehension,
  formatUsd,
  type OnlineUsage,
  type SeriesPoint,
  type LinguaReport,
  studiedLanguageOptions,
  useLinguaStore,
} from "@/stores/lingua";
import { currentLocale } from "@/i18n";
import { languageLabel } from "@/i18n/language-label";
import UsageLineChart from "@/components/UsageLineChart.vue";
import {
  BROWSER_FAMILIES,
  FUNNEL_GRAINS,
  FUNNEL_STEPS,
  type BrowserFilter,
  type FunnelCell,
  type FunnelView,
  type InstallationFunnel,
  type LeaveReasons,
} from "@/lib/linguaFunnel";

// The back-office "Lingua" screen (change: add-lingua-back-office, task 5.3). OPS only:
// aggregates, never a per-account view. It NEVER calls the
// API directly — the Pinia store does, behind the injectable client seam — and each
// async resource is a single ts-pattern union, matched exhaustively. Scope-gated by the
// router (meta.adminScope = "lingua"); every RPC is re-gated server-side.

const store = useLinguaStore();
const { t, te } = useI18n();
/** A studied language's name, or its code when the console has none (design D1). */
const languageName = (code: string): string => languageLabel(code, t, te);

onMounted(() => {
  void store.load();
});

const empty: LinguaReport = {
  activeAccounts: 0,
  wordsLearned: 0,
  reviews: 0,
  wordsRead: 0,
  newWordsSeen: 0,
  byLanguage: [],
};

const vm = computed(() =>
  match(store.report)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, data: empty }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: empty }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: empty }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);

const num = (v: number) => v.toLocaleString(currentLocale());
const usd = (micro: number) => formatUsd(micro, currentLocale());

const emptyOnline: OnlineUsage = { answered: 0, failed: 0, limited: 0, costMicroUsd: 0, lines: [] };
/** « Traduction en ligne » (change: add-lingua-online-translation), its own union. */
const onlineVm = computed(() =>
  match(store.online)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, data: emptyOnline }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: emptyOnline }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: emptyOnline }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);
/** Comprehension as a whole percentage, or "—" when nothing was read. */
const pct = (read: number, fresh: number) => {
  const c = comprehension(read, fresh);
  return c === null
    ? t("lingua.unavailable")
    : c.toLocaleString(currentLocale(), { style: "percent", maximumFractionDigits: 0 });
};

/** Inclusive list of ISO days [from, to] (UTC) — the shared x-axis. */
function dayRange(from: string, to: string): string[] {
  const out: string[] = [];
  const d = new Date(`${from}T00:00:00Z`);
  const end = new Date(`${to}T00:00:00Z`);
  for (let i = 0; d <= end && i < 400; i++) {
    out.push(d.toISOString().slice(0, 10));
    d.setUTCDate(d.getUTCDate() + 1);
  }
  return out;
}

/** Fold a single-value per-day metric into a one-dataset line chart. */
function toChart(points: SeriesPoint[], label: string) {
  const days = dayRange(store.filters.fromDay, store.filters.toDay);
  const byDay = new Map(points.map((p) => [p.day, p.value]));
  return { days, datasets: [{ label, values: days.map((d) => byDay.get(d) ?? 0) }] };
}

const emptySeries = {
  wordsLearned: [] as SeriesPoint[],
  reviews: [] as SeriesPoint[],
  wordsRead: [] as SeriesPoint[],
  newWordsSeen: [] as SeriesPoint[],
};
const seriesData = computed(() =>
  match(store.series)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => emptySeries),
);
const wordsChart = computed(() => toChart(seriesData.value.wordsLearned, t("lingua.wordsLearned")));
const reviewsChart = computed(() => toChart(seriesData.value.reviews, t("lingua.reviews")));
const wordsReadChart = computed(() => toChart(seriesData.value.wordsRead, t("lingua.wordsRead")));
const newWordsChart = computed(() => toChart(seriesData.value.newWordsSeen, t("lingua.newWordsSeen")));

// The studied-language filter lists the languages the usage report holds, plus the selected
// one (change: remove-lingua-pack-registry — it used to list the registered packs). "" = every
// language is the select's own first option.
const languageOptions = computed(() => studiedLanguageOptions(vm.value.data.byLanguage, store.filters.language));

function apply() {
  void store.load();
}

// Installations (change: add-lingua-usage-counts, design D13): anonymous counts, by installation
// day and browser family — never finer — or summed by week. Two unions of their own, so a failed
// read leaves the rest of the console standing.
const emptyView: FunnelView = { rows: [], series: { labels: [], installed: [] } };
const emptyFunnel: InstallationFunnel = { day: emptyView, week: emptyView };
const funnelVm = computed(() =>
  match(store.funnel)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, data: emptyFunnel }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: emptyFunnel }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: emptyFunnel }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);
/** The view the toggle picked — by day or by week — of the same read. */
const funnelView = computed(() => funnelVm.value.data[store.funnelGrain]);
const emptyReasons: LeaveReasons = { reasons: [], capped: false };
const reasonsVm = computed(() =>
  match(store.reasons)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, data: emptyReasons }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: emptyReasons }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: emptyReasons }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);

/** The browser filter re-reads the section alone, funnel and leaving reasons alike. */
const browser = computed<BrowserFilter>({
  get: () => store.filters.browser,
  set: (b) => {
    void store.loadInstallations({ browser: b });
  },
});

/** A whole percent (63) in the console's locale: "63%", « 63 % ». */
const percent = (whole: number) =>
  (whole / 100).toLocaleString(currentLocale(), { style: "percent", maximumFractionDigits: 0 });

/** A step as "25 (63%)", or "not yet" while its return window is open. */
function cellText(cell: FunnelCell): string {
  return match(cell)
    .with({ kind: "notYet" }, () => t("lingua.installations.notYet"))
    .with({ kind: "count" }, ({ count, share }) => (share === null ? num(count) : `${num(count)} (${percent(share)})`))
    .exhaustive();
}

/** A leaving reason's label, or its code for an answer this console has no label for yet. */
function reasonLabel(code: string): string {
  const key = `lingua.installations.reasons.${code}`;
  return te(key) ? t(key) : code;
}

// A week is capped when one of its days is: the day rows tell for both views.
const anyCapped = computed(() => funnelVm.value.data.day.rows.some((r) => r.capped) || reasonsVm.value.data.capped);

const installationsChart = computed(() => ({
  labels: funnelView.value.series.labels,
  datasets: [{ label: t("lingua.installations.installed"), values: funnelView.value.series.installed }],
}));
</script>

<template>
  <section class="lingua">
    <header class="head">
      <h1>{{ t("lingua.title") }}</h1>
      <p class="caveat">{{ t("lingua.syncedNote") }}</p>
    </header>

    <!-- Window + studied-language filter. -->
    <form class="filters" @submit.prevent="apply">
      <label>
        <span>{{ t("lingua.from") }}</span>
        <input v-model="store.filters.fromDay" type="date" data-testid="from-day" />
      </label>
      <label>
        <span>{{ t("lingua.to") }}</span>
        <input v-model="store.filters.toDay" type="date" data-testid="to-day" />
      </label>
      <label>
        <span>{{ t("lingua.language") }}</span>
        <select v-model="store.filters.language" data-testid="language">
          <option value="">{{ t("lingua.anyLanguage") }}</option>
          <option v-for="l in languageOptions" :key="l" :value="l">{{ languageName(l) }}</option>
        </select>
      </label>
      <button type="submit" data-testid="apply">{{ t("lingua.apply") }}</button>
    </form>

    <p v-if="vm.loading" class="state" data-testid="loading">{{ t("lingua.loading") }}</p>
    <p v-else-if="vm.error" class="state error" data-testid="error">{{ vm.error }}</p>

    <template v-else>
      <!-- Tiles over the window: distinct synced accounts, words learned, reviews, and the
           reading figures (words read in blocks seen, new words among them, comprehension). -->
      <div class="kpis">
        <div class="kpi" data-testid="active-accounts">
          <span class="kpi-value">{{ num(vm.data.activeAccounts) }}</span>
          <span class="kpi-label">{{ t("lingua.activeAccounts") }}</span>
        </div>
        <div class="kpi" data-testid="words-learned">
          <span class="kpi-value">{{ num(vm.data.wordsLearned) }}</span>
          <span class="kpi-label">{{ t("lingua.wordsLearned") }}</span>
        </div>
        <div class="kpi" data-testid="reviews">
          <span class="kpi-value">{{ num(vm.data.reviews) }}</span>
          <span class="kpi-label">{{ t("lingua.reviews") }}</span>
        </div>
        <div class="kpi" data-testid="words-read">
          <span class="kpi-value">{{ num(vm.data.wordsRead) }}</span>
          <span class="kpi-label">{{ t("lingua.wordsRead") }}</span>
        </div>
        <div class="kpi" data-testid="new-words-seen">
          <span class="kpi-value">{{ num(vm.data.newWordsSeen) }}</span>
          <span class="kpi-label">{{ t("lingua.newWordsSeen") }}</span>
        </div>
        <div class="kpi" data-testid="comprehension">
          <span class="kpi-value">{{ pct(vm.data.wordsRead, vm.data.newWordsSeen) }}</span>
          <span class="kpi-label">{{ t("lingua.comprehension") }}</span>
        </div>
      </div>

      <div class="split">
        <div class="panel">
          <h2>{{ t("lingua.wordsLearned") }}</h2>
          <UsageLineChart
            :labels="wordsChart.days"
            :datasets="wordsChart.datasets"
            :y-label="t('lingua.wordsLearned')"
          />
        </div>
        <div class="panel">
          <h2>{{ t("lingua.reviews") }}</h2>
          <UsageLineChart
            :labels="reviewsChart.days"
            :datasets="reviewsChart.datasets"
            :y-label="t('lingua.reviews')"
          />
        </div>
        <div class="panel">
          <h2>{{ t("lingua.wordsRead") }}</h2>
          <UsageLineChart
            :labels="wordsReadChart.days"
            :datasets="wordsReadChart.datasets"
            :y-label="t('lingua.wordsRead')"
          />
        </div>
        <div class="panel">
          <h2>{{ t("lingua.newWordsSeen") }}</h2>
          <UsageLineChart
            :labels="newWordsChart.days"
            :datasets="newWordsChart.datasets"
            :y-label="t('lingua.newWordsSeen')"
          />
        </div>
      </div>

      <!-- Breakdown by studied language (counts only). -->
      <div class="panel">
        <h2>{{ t("lingua.byLanguage") }}</h2>
        <table>
          <thead>
            <tr>
              <th>{{ t("lingua.language") }}</th>
              <th class="n">{{ t("lingua.activeAccounts") }}</th>
              <th class="n">{{ t("lingua.wordsLearned") }}</th>
              <th class="n">{{ t("lingua.reviews") }}</th>
              <th class="n">{{ t("lingua.wordsRead") }}</th>
              <th class="n">{{ t("lingua.newWordsSeen") }}</th>
              <th class="n">{{ t("lingua.comprehension") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="l in vm.data.byLanguage" :key="l.language" data-testid="language-row">
              <td>{{ languageName(l.language) }}</td>
              <td class="n">{{ num(l.activeAccounts) }}</td>
              <td class="n">{{ num(l.wordsLearned) }}</td>
              <td class="n">{{ num(l.reviews) }}</td>
              <td class="n">{{ num(l.wordsRead) }}</td>
              <td class="n">{{ num(l.newWordsSeen) }}</td>
              <td class="n">{{ pct(l.wordsRead, l.newWordsSeen) }}</td>
            </tr>
            <tr v-if="vm.data.byLanguage.length === 0">
              <td colspan="7" class="muted">{{ t("lingua.noData") }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- « Traduction en ligne »: daily totals per provider and pair, and their estimated
           cost — no account, no text (change: add-lingua-online-translation). -->
      <div class="panel" data-testid="online-translation">
        <h2>{{ t("lingua.online.title") }}</h2>
        <p v-if="onlineVm.error" class="state error" data-testid="online-error">{{ onlineVm.error }}</p>
        <template v-else>
          <div class="kpis">
            <div class="kpi" data-testid="online-answered">
              <span class="kpi-value">{{ num(onlineVm.data.answered) }}</span>
              <span class="kpi-label">{{ t("lingua.online.answered") }}</span>
            </div>
            <div class="kpi" data-testid="online-failed">
              <span class="kpi-value">{{ num(onlineVm.data.failed) }}</span>
              <span class="kpi-label">{{ t("lingua.online.failed") }}</span>
            </div>
            <div class="kpi" data-testid="online-limited">
              <span class="kpi-value">{{ num(onlineVm.data.limited) }}</span>
              <span class="kpi-label">{{ t("lingua.online.limited") }}</span>
            </div>
            <div class="kpi" data-testid="online-cost">
              <span class="kpi-value">{{ usd(onlineVm.data.costMicroUsd) }}</span>
              <span class="kpi-label">{{ t("lingua.online.cost") }}</span>
            </div>
          </div>
          <table>
            <thead>
              <tr>
                <th>{{ t("lingua.online.provider") }}</th>
                <th>{{ t("lingua.online.pair") }}</th>
                <th class="n">{{ t("lingua.online.answered") }}</th>
                <th class="n">{{ t("lingua.online.failed") }}</th>
                <th class="n">{{ t("lingua.online.limited") }}</th>
                <th class="n">{{ t("lingua.online.tokens") }}</th>
                <th class="n">{{ t("lingua.online.cost") }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="l in onlineVm.data.lines" :key="`${l.provider}|${l.pair}`" data-testid="online-row">
                <td>{{ l.provider === "none" ? t("lingua.online.noProvider") : l.provider }}</td>
                <td class="mono">{{ l.pair }}</td>
                <td class="n">{{ num(l.answered) }}</td>
                <td class="n">{{ num(l.failed) }}</td>
                <td class="n">{{ num(l.limited) }}</td>
                <td class="n">{{ num(l.inputTokens) }} / {{ num(l.outputTokens) }}</td>
                <td class="n">{{ usd(l.costMicroUsd) }}</td>
              </tr>
              <tr v-if="onlineVm.data.lines.length === 0">
                <td colspan="7" class="muted">
                  {{ onlineVm.loading ? t("lingua.loading") : t("lingua.noData") }}
                </td>
              </tr>
            </tbody>
          </table>
          <p class="muted" data-testid="online-note">{{ t("lingua.online.note") }}</p>
        </template>
      </div>
    </template>

    <!-- Installations: anonymous counts by installation day (or summed by week) and browser family,
         and why readers left (change: add-lingua-usage-counts). Outside the report's gate: other
         counters. -->
    <div class="panel" data-testid="installations">
      <div class="panel-head">
        <h2>{{ t("lingua.installations.title") }}</h2>
        <div class="controls">
          <div class="toggle" role="group" :aria-label="t('lingua.installations.grainLabel')">
            <button
              v-for="g in FUNNEL_GRAINS"
              :key="g"
              type="button"
              :aria-pressed="store.funnelGrain === g"
              :data-testid="`grain-${g}`"
              @click="store.funnelGrain = g"
            >
              {{ t(`lingua.installations.grains.${g}`) }}
            </button>
          </div>
          <label class="inline">
            <span>{{ t("lingua.installations.browser") }}</span>
            <select v-model="browser" data-testid="browser">
              <option value="">{{ t("lingua.installations.anyBrowser") }}</option>
              <option v-for="b in BROWSER_FAMILIES" :key="b" :value="b">
                {{ t(`lingua.installations.browsers.${b}`) }}
              </option>
            </select>
          </label>
        </div>
      </div>
      <p class="muted" data-testid="installations-note">{{ t("lingua.installations.note") }}</p>

      <p v-if="funnelVm.error" class="state error" data-testid="funnel-error">{{ funnelVm.error }}</p>
      <template v-else>
        <h3>{{ t(`lingua.installations.chartTitle.${store.funnelGrain}`) }}</h3>
        <UsageLineChart
          :labels="installationsChart.labels"
          :datasets="installationsChart.datasets"
          :y-label="t('lingua.installations.installed')"
        />
        <table>
          <thead>
            <tr>
              <th>{{ t(`lingua.installations.period.${store.funnelGrain}`) }}</th>
              <th class="n">{{ t("lingua.installations.installed") }}</th>
              <th v-for="s in FUNNEL_STEPS" :key="s" class="n">{{ t(`lingua.installations.${s}`) }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="r in funnelView.rows" :key="r.period" data-testid="funnel-row" :data-period="r.period">
              <td class="mono">
                {{ r.period }}
                <span v-if="r.capped" class="badge" data-testid="capped" :title="t('lingua.installations.cappedHint')">
                  {{ t("lingua.installations.capped") }}
                </span>
              </td>
              <td class="n" data-testid="installed">{{ num(r.installed) }}</td>
              <td
                v-for="s in FUNNEL_STEPS"
                :key="s"
                class="n"
                :class="{ muted: r.cells[s].kind === 'notYet' }"
                :data-testid="`step-${s}`"
              >
                {{ cellText(r.cells[s]) }}
              </td>
            </tr>
            <tr v-if="funnelView.rows.length === 0">
              <td :colspan="2 + FUNNEL_STEPS.length" class="muted">
                {{ funnelVm.loading ? t("lingua.installations.loading") : t("lingua.noData") }}
              </td>
            </tr>
          </tbody>
        </table>
        <p class="muted" data-testid="not-yet-note">{{ t("lingua.installations.notYetNote") }}</p>
      </template>

      <h3>
        {{ t("lingua.installations.reasonsTitle") }}
        <span
          v-if="reasonsVm.data.capped"
          class="badge"
          data-testid="reasons-capped"
          :title="t('lingua.installations.cappedHint')"
        >
          {{ t("lingua.installations.capped") }}
        </span>
      </h3>
      <p v-if="reasonsVm.error" class="state error" data-testid="reasons-error">{{ reasonsVm.error }}</p>
      <p v-else-if="reasonsVm.loading" class="muted">{{ t("lingua.installations.loading") }}</p>
      <table v-else class="reasons">
        <tbody>
          <tr v-for="r in reasonsVm.data.reasons" :key="r.reason" data-testid="reason-row" :data-reason="r.reason">
            <th scope="row" class="reason">{{ reasonLabel(r.reason) }}</th>
            <td class="n">{{ num(r.count) }}</td>
          </tr>
        </tbody>
      </table>
      <p class="muted">{{ t("lingua.installations.reasonsNote") }}</p>
      <p v-if="anyCapped" class="muted" data-testid="capped-note">{{ t("lingua.installations.cappedHint") }}</p>
    </div>
  </section>
</template>

<style scoped>
.lingua {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}
.head h1 {
  margin: 0 0 0.25rem;
}
.caveat {
  margin: 0;
  font-size: 0.85rem;
  opacity: 0.7;
}
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: end;
}
.filters label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8rem;
}
.filters input,
.filters select {
  padding: 0.4rem 0.5rem;
}
.filters button {
  padding: 0.45rem 1rem;
}
.kpis {
  display: flex;
  gap: 2rem;
  flex-wrap: wrap;
}
.kpi {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}
.kpi-value {
  font-size: 2rem;
  font-weight: 700;
}
.kpi-label {
  font-size: 0.85rem;
  opacity: 0.7;
}
.split {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1rem;
}
.panel {
  border: 1px solid var(--border, #2a2a2a);
  border-radius: 8px;
  padding: 1rem;
  overflow-x: auto;
}
.panel h2 {
  margin: 0 0 0.75rem;
  font-size: 1rem;
}
.panel h3 {
  margin: 1rem 0 0.5rem;
  font-size: 0.9rem;
}
.panel-head {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: baseline;
  justify-content: space-between;
}
.inline {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  font-size: 0.8rem;
}
.inline select {
  padding: 0.3rem 0.5rem;
}
.controls {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  align-items: center;
}
/* The Day / Week toggle, drawn like the Jobs console's period buttons: the pressed one in the accent. */
.toggle {
  display: flex;
  gap: 0.35rem;
}
.toggle button {
  padding: 0.3rem 0.7rem;
  font-size: 0.8rem;
}
.toggle button[aria-pressed="true"] {
  border-color: var(--accent);
  color: var(--accent);
}
.badge {
  display: inline-block;
  margin-left: 0.4rem;
  padding: 0 0.4rem;
  border: 1px solid currentColor;
  border-radius: 4px;
  font-size: 0.7rem;
  font-family: inherit;
  opacity: 0.8;
}
table.reasons {
  width: auto;
  min-width: 50%;
}
table {
  width: 100%;
  border-collapse: collapse;
}
th,
td {
  text-align: left;
  padding: 0.35rem 0.5rem;
  border-bottom: 1px solid var(--border, #2a2a2a);
  vertical-align: top;
}
th.n,
td.n {
  text-align: right;
  font-variant-numeric: tabular-nums;
}
/* A leaving reason heads its row: it reads like the cells beside it. */
th.reason {
  font-weight: normal;
}
.muted {
  opacity: 0.6;
}
.state {
  opacity: 0.8;
}
.state.error {
  color: #e55;
}
</style>
