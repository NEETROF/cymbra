<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import {
  comprehension,
  type SeriesPoint,
  type LinguaReport,
  studiedLanguageOptions,
  useLinguaStore,
} from "@/stores/lingua";
import { currentLocale } from "@/i18n";
import UsageLineChart from "@/components/UsageLineChart.vue";

// The back-office "Lingua" screen (change: add-lingua-back-office, task 5.3). OPS only:
// aggregates, never a per-account view. It NEVER calls the
// API directly — the Pinia store does, behind the injectable client seam — and each
// async resource is a single ts-pattern union, matched exhaustively. Scope-gated by the
// router (meta.adminScope = "lingua"); every RPC is re-gated server-side.

const store = useLinguaStore();
const { t } = useI18n();

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
          <option v-for="l in languageOptions" :key="l" :value="l">{{ l }}</option>
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
              <td>{{ l.language }}</td>
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
    </template>
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
