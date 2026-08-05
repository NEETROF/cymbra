import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import { SeriesDimension } from "@/gen/usage_pb";

// Feature-usage analytics console (change: add-feature-usage-analytics, task 7).
// Reads the permanent daily aggregates through the `api()` seam — never a direct
// API call from a component. Both the report and the (data-driven) action filter
// list are `Async` unions so the view matches on them exhaustively; a denied/failed
// read lands in the union as an error, not a throw.

/** Distinct users on one platform over the window. */
export interface PlatformCount {
  platform: string;
  users: number;
}
/** Distinct users on one device class over the window. */
export interface DeviceClassCount {
  deviceClass: string;
  users: number;
}
/** Exact distinct users over the window, split by platform + device class. */
export interface UsersSummary {
  totalUsers: number;
  byPlatform: PlatformCount[];
  byDeviceClass: DeviceClassCount[];
}
/** One (action, variant) volume row. */
export interface ActionRow {
  action: string;
  variant: string;
  events: number;
}
/** The two aggregates the screen shows for a filtered window. */
export interface UsageReport {
  summary: UsersSummary;
  rows: ActionRow[];
}

/** One point of a per-day time series. */
export interface SeriesPoint {
  day: string; // ISO yyyy-mm-dd
  series: string;
  value: number;
}
/** The three per-day series that back the line charts. */
export interface UsageSeries {
  platform: SeriesPoint[];
  deviceClass: SeriesPoint[];
  action: SeriesPoint[];
}

/** The freely-combinable filters. Empty string = no filter on that dimension. */
export interface UsageFilters {
  fromDay: string; // inclusive ISO yyyy-mm-dd
  toDay: string; // inclusive ISO yyyy-mm-dd
  platform: string;
  deviceClass: string;
  action: string;
}

/** ISO `yyyy-mm-dd` for `date` (UTC). */
function isoDay(date: Date): string {
  return date.toISOString().slice(0, 10);
}

/** The default window: the trailing 30 days ending today. */
export function defaultFilters(): UsageFilters {
  const to = new Date();
  const from = new Date(to.getTime() - 29 * 24 * 60 * 60 * 1000);
  return { fromDay: isoDay(from), toDay: isoDay(to), platform: "", deviceClass: "", action: "" };
}

export const useUsageStore = defineStore("usage", () => {
  const report = ref<Async<UsageReport>>(idle);
  // Per-day series for the line charts (default graph view).
  const series = ref<Async<UsageSeries>>(idle);
  // The data-driven action filter list (distinct actions in the aggregates).
  const actions = ref<Async<string[]>>(idle);
  const filters = reactive<UsageFilters>(defaultFilters());

  function currentQuery() {
    return {
      fromDay: filters.fromDay,
      toDay: filters.toDay,
      platform: filters.platform || undefined,
      deviceClass: filters.deviceClass || undefined,
      action: filters.action || undefined,
    };
  }

  /** Load the report (totals) + the per-day series for the current filters. */
  async function load(next: Partial<UsageFilters> = {}) {
    Object.assign(filters, next);
    await Promise.all([
      run(report, async () => {
        const query = currentQuery();
        const [s, b] = await Promise.all([
          api().usage.getUsersSummary({ query }),
          api().usage.getActionBreakdown({ query }),
        ]);
        return {
          summary: {
            totalUsers: Number(s.totalUsers),
            byPlatform: s.byPlatform.map((p) => ({ platform: p.platform, users: Number(p.users) })),
            byDeviceClass: s.byDeviceClass.map((d) => ({ deviceClass: d.deviceClass, users: Number(d.users) })),
          },
          rows: b.rows.map((r) => ({ action: r.action, variant: r.variant, events: Number(r.events) })),
        } satisfies UsageReport;
      }),
      run(series, async () => {
        const query = currentQuery();
        const mapPoints = (pts: { day: string; series: string; value: bigint }[]): SeriesPoint[] =>
          pts.map((p) => ({ day: p.day, series: p.series, value: Number(p.value) }));
        const [pf, dv, ac] = await Promise.all([
          api().usage.getUsageSeries({ query, dimension: SeriesDimension.SERIES_PLATFORM }),
          api().usage.getUsageSeries({ query, dimension: SeriesDimension.SERIES_DEVICE_CLASS }),
          api().usage.getUsageSeries({ query, dimension: SeriesDimension.SERIES_ACTION }),
        ]);
        return {
          platform: mapPoints(pf.points),
          deviceClass: mapPoints(dv.points),
          action: mapPoints(ac.points),
        } satisfies UsageSeries;
      }),
    ]);
  }

  /** Load the distinct actions for the filter dropdown (data-driven, no hard-coded list). */
  async function loadActions() {
    await run(actions, async () => (await api().usage.listActions({})).actions);
  }

  return { report, series, actions, filters, load, loadActions };
});
