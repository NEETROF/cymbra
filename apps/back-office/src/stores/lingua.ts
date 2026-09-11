import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import { LinguaSeriesMetric } from "@/gen/lingua_admin_pb";

// Lingua ops console (change: add-lingua-back-office, task 5). Reads aggregates over the
// existing sync data through the `api()` seam — never a direct API call from a component.
// Every resource is an `Async` union so the view matches on it exhaustively; a
// denied/failed read lands in the union as a localised error, not a throw.
//
// Privacy: these types carry AGGREGATES ONLY — counts by day and studied language, never
// a per-account row. "Active accounts" counts only devices that sync (the screen says so).

/** One studied language's aggregates over the window. */
export interface LanguageUsage {
  language: string;
  activeAccounts: number;
  wordsLearned: number;
  reviews: number;
}
/** The tiles + per-language breakdown for a window. */
export interface LinguaReport {
  activeAccounts: number;
  wordsLearned: number;
  reviews: number;
  byLanguage: LanguageUsage[];
}

/** One point of a per-day time series. */
export interface SeriesPoint {
  day: string; // ISO yyyy-mm-dd
  value: number;
}
/** The three per-day series that back the line charts. */
export interface LinguaSeries {
  wordsLearned: SeriesPoint[];
  reviews: SeriesPoint[];
  exposures: SeriesPoint[];
}

/** One published data pack in the read-only registry. */
export interface DataPack {
  studied: string;
  native: string;
  packVersion: string;
  analyzerVersion: string;
  builtAt: string;
  sizeBytes: number;
  notice: string;
}

/** The window + optional studied-language filter. Empty language = every language. */
export interface LinguaFilters {
  fromDay: string; // inclusive ISO yyyy-mm-dd
  toDay: string; // inclusive ISO yyyy-mm-dd
  language: string;
}

/** ISO `yyyy-mm-dd` for `date` (UTC). */
function isoDay(date: Date): string {
  return date.toISOString().slice(0, 10);
}

/** The default window: the trailing 30 days ending today (aligned with /usage). */
export function defaultFilters(): LinguaFilters {
  const to = new Date();
  const from = new Date(to.getTime() - 29 * 24 * 60 * 60 * 1000);
  return { fromDay: isoDay(from), toDay: isoDay(to), language: "" };
}

export const useLinguaStore = defineStore("lingua", () => {
  const report = ref<Async<LinguaReport>>(idle);
  const series = ref<Async<LinguaSeries>>(idle);
  const packs = ref<Async<DataPack[]>>(idle);
  const filters = reactive<LinguaFilters>(defaultFilters());

  function currentWindow() {
    return { fromDay: filters.fromDay, toDay: filters.toDay };
  }

  /** Load the tiles/breakdown + the three per-day series for the current filters. */
  async function load(next: Partial<LinguaFilters> = {}) {
    Object.assign(filters, next);
    const language = filters.language; // "" => every studied language
    await Promise.all([
      run(report, async () => {
        const r = await api().lingua.adminGetLinguaUsage({ window: currentWindow() });
        return {
          activeAccounts: Number(r.activeAccounts),
          wordsLearned: Number(r.wordsLearned),
          reviews: Number(r.reviews),
          byLanguage: r.byLanguage.map((l) => ({
            language: l.language,
            activeAccounts: Number(l.activeAccounts),
            wordsLearned: Number(l.wordsLearned),
            reviews: Number(l.reviews),
          })),
        } satisfies LinguaReport;
      }),
      run(series, async () => {
        const mapPoints = (pts: { day: string; value: bigint }[]): SeriesPoint[] =>
          pts.map((p) => ({ day: p.day, value: Number(p.value) }));
        const one = (metric: LinguaSeriesMetric) =>
          api().lingua.adminGetLinguaUsageSeries({ window: currentWindow(), metric, language });
        const [w, rv, ex] = await Promise.all([
          one(LinguaSeriesMetric.LINGUA_SERIES_WORDS_LEARNED),
          one(LinguaSeriesMetric.LINGUA_SERIES_REVIEWS),
          one(LinguaSeriesMetric.LINGUA_SERIES_EXPOSURES),
        ]);
        return {
          wordsLearned: mapPoints(w.points),
          reviews: mapPoints(rv.points),
          exposures: mapPoints(ex.points),
        } satisfies LinguaSeries;
      }),
    ]);
  }

  /** Load the read-only pack registry. */
  async function loadPacks() {
    await run(packs, async () =>
      (await api().lingua.adminListDataPacks({})).packs.map(
        (p) =>
          ({
            studied: p.studied,
            native: p.native,
            packVersion: p.packVersion,
            analyzerVersion: p.analyzerVersion,
            builtAt: p.builtAt,
            sizeBytes: Number(p.sizeBytes),
            notice: p.notice,
          }) satisfies DataPack,
      ),
    );
  }

  return { report, series, packs, filters, load, loadPacks };
});
