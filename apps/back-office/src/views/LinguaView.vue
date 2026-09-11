<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { type SeriesPoint, type LinguaReport, useLinguaStore } from "@/stores/lingua";
import { currentLocale } from "@/i18n";
import UsageLineChart from "@/components/UsageLineChart.vue";

// The back-office "Lingua" screen (change: add-lingua-back-office, task 5.3). OPS only:
// aggregates + the read-only pack registry, never a per-account view. It NEVER calls the
// API directly — the Pinia store does, behind the injectable client seam — and each
// async resource is a single ts-pattern union, matched exhaustively. Scope-gated by the
// router (meta.adminScope = "lingua"); every RPC is re-gated server-side.

const store = useLinguaStore();
const { t } = useI18n();

onMounted(() => {
  void store.loadPacks();
  void store.load();
});

const empty: LinguaReport = { activeAccounts: 0, wordsLearned: 0, reviews: 0, byLanguage: [] };

const vm = computed(() =>
  match(store.report)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, data: empty }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: empty }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: empty }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);

const num = (v: number) => v.toLocaleString(currentLocale());

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

const emptySeries = { wordsLearned: [] as SeriesPoint[], reviews: [] as SeriesPoint[], exposures: [] as SeriesPoint[] };
const seriesData = computed(() =>
  match(store.series)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => emptySeries),
);
const wordsChart = computed(() => toChart(seriesData.value.wordsLearned, t("lingua.wordsLearned")));
const reviewsChart = computed(() => toChart(seriesData.value.reviews, t("lingua.reviews")));
const exposuresChart = computed(() => toChart(seriesData.value.exposures, t("lingua.exposures")));

// The studied-language filter list is data-driven from the published packs (stable
// regardless of the usage window), plus "" = every language.
const languageOptions = computed(() =>
  match(store.packs)
    .with({ status: "success" }, ({ data }) => [...new Set(data.map((p) => p.studied))].sort())
    .otherwise(() => [] as string[]),
);

const packs = computed(() =>
  match(store.packs)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => []),
);

/** A compact human byte size (the pack budget is a few MB at most). */
function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

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
      <!-- Tiles: distinct synced accounts, words learned, reviews (over the window). -->
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
          <h2>{{ t("lingua.exposures") }}</h2>
          <UsageLineChart
            :labels="exposuresChart.days"
            :datasets="exposuresChart.datasets"
            :y-label="t('lingua.exposures')"
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
            </tr>
          </thead>
          <tbody>
            <tr v-for="l in vm.data.byLanguage" :key="l.language" data-testid="language-row">
              <td>{{ l.language }}</td>
              <td class="n">{{ num(l.activeAccounts) }}</td>
              <td class="n">{{ num(l.wordsLearned) }}</td>
              <td class="n">{{ num(l.reviews) }}</td>
            </tr>
            <tr v-if="vm.data.byLanguage.length === 0">
              <td colspan="4" class="muted">{{ t("lingua.noData") }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Read-only registry of published data packs (informational; no publish action). -->
      <div class="panel">
        <h2>{{ t("lingua.packsTitle") }}</h2>
        <table>
          <thead>
            <tr>
              <th>{{ t("lingua.pair") }}</th>
              <th>{{ t("lingua.packVersion") }}</th>
              <th>{{ t("lingua.analyzerVersion") }}</th>
              <th>{{ t("lingua.builtAt") }}</th>
              <th class="n">{{ t("lingua.size") }}</th>
              <th>{{ t("lingua.notice") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="p in packs" :key="`${p.studied}-${p.native}-${p.packVersion}`" data-testid="pack-row">
              <td>{{ p.studied }}→{{ p.native }}</td>
              <td>{{ p.packVersion }}</td>
              <td>{{ p.analyzerVersion }}</td>
              <td>{{ p.builtAt }}</td>
              <td class="n">{{ fmtSize(p.sizeBytes) }}</td>
              <td>
                <details>
                  <summary>{{ t("lingua.viewNotice") }}</summary>
                  <pre class="notice">{{ p.notice }}</pre>
                </details>
              </td>
            </tr>
            <tr v-if="packs.length === 0">
              <td colspan="6" class="muted">{{ t("lingua.noData") }}</td>
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
.notice {
  white-space: pre-wrap;
  font-size: 0.8rem;
  margin: 0.5rem 0 0;
  opacity: 0.85;
}
.state {
  opacity: 0.8;
}
.state.error {
  color: #e55;
}
</style>
