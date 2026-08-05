<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { match } from "ts-pattern";
import { type SeriesPoint, type UsageReport, useUsageStore } from "@/stores/usage";
import { currentLocale } from "@/i18n";
import UsageLineChart from "@/components/UsageLineChart.vue";

// The back-office "Usage" screen (change: add-feature-usage-analytics, task 7.3).
// It NEVER calls the API directly — the Pinia store does, behind the injectable
// client seam — and its async state is a single ts-pattern union, matched
// exhaustively. Admin-scope gated by the router (meta.admin).

const store = useUsageStore();
const { t } = useI18n();

const PLATFORMS = ["", "ios", "android", "macos", "windows", "linux", "web"];
const DEVICE_CLASSES = ["", "phone", "tablet", "desktop"];

onMounted(() => {
  void store.loadActions();
  void store.load();
});

const empty: UsageReport = { summary: { totalUsers: 0, byPlatform: [], byDeviceClass: [] }, rows: [] };

const vm = computed(() =>
  match(store.report)
    .with({ status: "idle" }, () => ({ loading: true, error: null as string | null, data: empty }))
    .with({ status: "loading" }, () => ({ loading: true, error: null, data: empty }))
    .with({ status: "error" }, ({ error }) => ({ loading: false, error, data: empty }))
    .with({ status: "success" }, ({ data }) => ({ loading: false, error: null, data }))
    .exhaustive(),
);

// The action filter list is data-driven (distinct actions in the aggregates) —
// never a hard-coded taxonomy, so a newly-emitted action appears automatically.
const actionOptions = computed(() =>
  match(store.actions)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => [] as string[]),
);

const num = (v: number) => v.toLocaleString(currentLocale());

// View mode: time-series line charts (default) or the raw tables.
const mode = ref<"graph" | "table">("graph");

// Max curves per chart before the tail folds into "Other" (never generate a hue
// beyond the validated 8-slot palette — data-viz non-negotiable).
const MAX_SERIES = 8;

/** Inclusive list of ISO days [from, to] (UTC) — the shared x-axis. */
function dayRange(from: string, to: string): string[] {
  const out: string[] = [];
  const d = new Date(`${from}T00:00:00Z`);
  const end = new Date(`${to}T00:00:00Z`);
  // Guard against an inverted/huge range.
  for (let i = 0; d <= end && i < 400; i++) {
    out.push(d.toISOString().slice(0, 10));
    d.setUTCDate(d.getUTCDate() + 1);
  }
  return out;
}

/** Fold per-day points into a chart: a day axis + one dataset per series key,
 *  ordered by total (busiest first), the tail beyond MAX_SERIES summed as "Other". */
function toChart(points: SeriesPoint[]) {
  const days = dayRange(store.filters.fromDay, store.filters.toDay);
  const bySeries = new Map<string, Map<string, number>>();
  const totals = new Map<string, number>();
  for (const p of points) {
    (bySeries.get(p.series) ?? bySeries.set(p.series, new Map()).get(p.series)!).set(p.day, p.value);
    totals.set(p.series, (totals.get(p.series) ?? 0) + p.value);
  }
  const ordered = [...totals.entries()].sort((a, b) => b[1] - a[1]).map(([k]) => k);
  const fold = ordered.length > MAX_SERIES;
  const keep = fold ? ordered.slice(0, MAX_SERIES - 1) : ordered;
  const rest = fold ? ordered.slice(MAX_SERIES - 1) : [];
  const datasets = keep.map((s) => ({ label: s, values: days.map((d) => bySeries.get(s)?.get(d) ?? 0) }));
  if (rest.length) {
    datasets.push({
      label: t("usage.other"),
      values: days.map((d) => rest.reduce((sum, s) => sum + (bySeries.get(s)?.get(d) ?? 0), 0)),
    });
  }
  return { days, datasets };
}

const emptySeries = { platform: [] as SeriesPoint[], deviceClass: [] as SeriesPoint[], action: [] as SeriesPoint[] };
const seriesData = computed(() =>
  match(store.series)
    .with({ status: "success" }, ({ data }) => data)
    .otherwise(() => emptySeries),
);
const platformChart = computed(() => toChart(seriesData.value.platform));
const deviceChart = computed(() => toChart(seriesData.value.deviceClass));
const actionChart = computed(() => toChart(seriesData.value.action));

function apply() {
  void store.load();
}
</script>

<template>
  <section class="usage">
    <header class="head">
      <div class="head-row">
        <h1>{{ t("usage.title") }}</h1>
        <!-- Graph (default) / Table view toggle. -->
        <div class="viewtoggle" role="tablist" :aria-label="t('usage.viewMode')">
          <button
            type="button"
            role="tab"
            :aria-selected="mode === 'graph'"
            :class="{ active: mode === 'graph' }"
            data-testid="view-graph"
            @click="mode = 'graph'"
          >
            {{ t("usage.viewGraph") }}
          </button>
          <button
            type="button"
            role="tab"
            :aria-selected="mode === 'table'"
            :class="{ active: mode === 'table' }"
            data-testid="view-table"
            @click="mode = 'table'"
          >
            {{ t("usage.viewTable") }}
          </button>
        </div>
      </div>
      <p class="caveat">{{ t("usage.periodCaveat") }}</p>
    </header>

    <!-- Freely-combinable filters: date range + platform + device class + action. -->
    <form class="filters" @submit.prevent="apply">
      <label>
        <span>{{ t("usage.from") }}</span>
        <input v-model="store.filters.fromDay" type="date" data-testid="from-day" />
      </label>
      <label>
        <span>{{ t("usage.to") }}</span>
        <input v-model="store.filters.toDay" type="date" data-testid="to-day" />
      </label>
      <label>
        <span>{{ t("usage.platform") }}</span>
        <select v-model="store.filters.platform" data-testid="platform">
          <option v-for="p in PLATFORMS" :key="p" :value="p">{{ p || t("usage.any") }}</option>
        </select>
      </label>
      <label>
        <span>{{ t("usage.deviceClass") }}</span>
        <select v-model="store.filters.deviceClass" data-testid="device-class">
          <option v-for="d in DEVICE_CLASSES" :key="d" :value="d">{{ d || t("usage.any") }}</option>
        </select>
      </label>
      <label>
        <span>{{ t("usage.action") }}</span>
        <select v-model="store.filters.action" data-testid="action">
          <option value="">{{ t("usage.any") }}</option>
          <option v-for="a in actionOptions" :key="a" :value="a">{{ a }}</option>
        </select>
      </label>
      <button type="submit" data-testid="apply">{{ t("usage.apply") }}</button>
    </form>

    <p v-if="vm.loading" class="state" data-testid="loading">{{ t("usage.loading") }}</p>
    <p v-else-if="vm.error" class="state error" data-testid="error">{{ vm.error }}</p>

    <template v-else>
      <!-- Unique users over the window (exact within the period). -->
      <div class="kpi" data-testid="total-users">
        <span class="kpi-value">{{ num(vm.data.summary.totalUsers) }}</span>
        <span class="kpi-label">{{ t("usage.distinctUsers") }}</span>
      </div>

      <div class="split">
        <div class="panel">
          <h2>{{ t("usage.byPlatform") }}</h2>
          <UsageLineChart
            v-if="mode === 'graph'"
            :labels="platformChart.days"
            :datasets="platformChart.datasets"
            :y-label="t('usage.users')"
          />
          <table v-else>
            <thead>
              <tr>
                <th>{{ t("usage.platform") }}</th>
                <th class="n">{{ t("usage.users") }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="p in vm.data.summary.byPlatform" :key="p.platform" data-testid="platform-row">
                <td>{{ p.platform }}</td>
                <td class="n">{{ num(p.users) }}</td>
              </tr>
              <tr v-if="vm.data.summary.byPlatform.length === 0">
                <td colspan="2" class="muted">{{ t("usage.noData") }}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="panel">
          <h2>{{ t("usage.byDeviceClass") }}</h2>
          <UsageLineChart
            v-if="mode === 'graph'"
            :labels="deviceChart.days"
            :datasets="deviceChart.datasets"
            :y-label="t('usage.users')"
          />
          <table v-else>
            <thead>
              <tr>
                <th>{{ t("usage.deviceClass") }}</th>
                <th class="n">{{ t("usage.users") }}</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="d in vm.data.summary.byDeviceClass" :key="d.deviceClass" data-testid="device-row">
                <td>{{ d.deviceClass }}</td>
                <td class="n">{{ num(d.users) }}</td>
              </tr>
              <tr v-if="vm.data.summary.byDeviceClass.length === 0">
                <td colspan="2" class="muted">{{ t("usage.noData") }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Action breakdown under the applied filters. -->
      <div class="panel">
        <h2>{{ t("usage.actionBreakdown") }}</h2>
        <UsageLineChart
          v-if="mode === 'graph'"
          :labels="actionChart.days"
          :datasets="actionChart.datasets"
          :y-label="t('usage.events')"
        />
        <table v-else>
          <thead>
            <tr>
              <th>{{ t("usage.action") }}</th>
              <th>{{ t("usage.variant") }}</th>
              <th class="n">{{ t("usage.events") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(r, i) in vm.data.rows" :key="`${r.action}-${r.variant}-${i}`" data-testid="action-row">
              <td>{{ r.action }}</td>
              <td class="muted">{{ r.variant || "—" }}</td>
              <td class="n">{{ num(r.events) }}</td>
            </tr>
            <tr v-if="vm.data.rows.length === 0">
              <td colspan="3" class="muted">{{ t("usage.noData") }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>
  </section>
</template>

<style scoped>
.usage {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}
.head h1 {
  margin: 0 0 0.25rem;
}
.head-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  flex-wrap: wrap;
}
.viewtoggle {
  display: inline-flex;
  border: 1px solid var(--border, #212a40);
  border-radius: 8px;
  overflow: hidden;
}
.viewtoggle button {
  appearance: none;
  border: 0;
  background: transparent;
  color: var(--muted, #9aa1ba);
  padding: 0.4rem 0.9rem;
  font: inherit;
  cursor: pointer;
}
.viewtoggle button.active {
  background: var(--accent-strong, #7c3aed);
  color: #fff;
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
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
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
