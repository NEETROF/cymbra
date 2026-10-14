import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { api } from "@/lib/api";
import { type Async, idle, run } from "@/lib/async";
import { LinguaSeriesMetric } from "@/gen/lingua_admin_pb";
import {
  type BrowserFilter,
  type InstallationFunnel,
  type LeaveReasons,
  installationFunnel,
  orderReasons,
  todayUtc,
} from "@/lib/linguaFunnel";

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
  wordsRead: number;
  newWordsSeen: number;
}
/** The tiles + per-language breakdown for a window. */
export interface LinguaReport {
  activeAccounts: number;
  wordsLearned: number;
  reviews: number;
  wordsRead: number;
  newWordsSeen: number;
  byLanguage: LanguageUsage[];
}

/**
 * Reading comprehension: the share of words read that were not new, 0–1 — or null when
 * nothing was read, shown as unavailable rather than 0 % or 100 % (change:
 * refine-lingua-reading-stats). Words read and new words seen come only from extensions that
 * count reading in blocks seen: the backend drops the others.
 */
export function comprehension(wordsRead: number, newWordsSeen: number): number | null {
  if (wordsRead <= 0) return null;
  return Math.min(1, Math.max(0, 1 - newWordsSeen / wordsRead));
}

/** « Traduction en ligne » over the window, per provider and pair (change:
 *  add-lingua-online-translation). Daily totals the service keeps — no account, no text. The cost
 *  is an estimate in micro-dollars, priced at request time. */
export interface OnlineUsageLine {
  provider: string;
  pair: string;
  answered: number;
  failed: number;
  limited: number;
  inputTokens: number;
  outputTokens: number;
  costMicroUsd: number;
}
export interface OnlineUsage {
  answered: number;
  failed: number;
  limited: number;
  costMicroUsd: number;
  lines: OnlineUsageLine[];
}

/** Fold the per-day rows into one line per provider × pair, busiest first. */
export function foldOnlineUsage(rows: readonly OnlineUsageLine[]): OnlineUsageLine[] {
  const by = new Map<string, OnlineUsageLine>();
  for (const r of rows) {
    const k = `${r.provider}|${r.pair}`;
    const cur = by.get(k) ?? {
      ...r,
      answered: 0,
      failed: 0,
      limited: 0,
      inputTokens: 0,
      outputTokens: 0,
      costMicroUsd: 0,
    };
    by.set(k, {
      ...cur,
      answered: cur.answered + r.answered,
      failed: cur.failed + r.failed,
      limited: cur.limited + r.limited,
      inputTokens: cur.inputTokens + r.inputTokens,
      outputTokens: cur.outputTokens + r.outputTokens,
      costMicroUsd: cur.costMicroUsd + r.costMicroUsd,
    });
  }
  return [...by.values()].sort(
    (a, b) => b.answered - a.answered || a.provider.localeCompare(b.provider) || a.pair.localeCompare(b.pair),
  );
}

/** An estimated cost in micro-dollars, as dollars in the console's locale. */
export function formatUsd(microUsd: number, locale: string): string {
  return (microUsd / 1_000_000).toLocaleString(locale, {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    // Under a dollar, cents alone would read as nothing: a sentence costs about $0.0001.
    maximumFractionDigits: microUsd < 1_000_000 ? 4 : 2,
  });
}

/** One point of a per-day time series. */
export interface SeriesPoint {
  day: string; // ISO yyyy-mm-dd
  value: number;
}
/** The per-day series that back the line charts. */
export interface LinguaSeries {
  wordsLearned: SeriesPoint[];
  reviews: SeriesPoint[];
  wordsRead: SeriesPoint[];
  newWordsSeen: SeriesPoint[];
}

/**
 * The studied-language filter's options: the languages of the usage report's per-language
 * breakdown, plus the one currently selected — so a window with no activity in it never leaves
 * the filter on a value it no longer offers. Not circular: the report takes only the window;
 * the language filter applies to the per-day series alone. "" (every language) is the view's
 * own first option and is not listed here.
 */
export function studiedLanguageOptions(byLanguage: readonly LanguageUsage[], selected: string): string[] {
  const languages = new Set(byLanguage.map((l) => l.language));
  if (selected) languages.add(selected);
  return [...languages].sort((a, b) => a.localeCompare(b));
}

/** The window + optional studied-language filter. Empty language = every language. The browser
 *  family filters the Installations section alone (change: add-lingua-usage-counts); empty =
 *  every family. */
export interface LinguaFilters {
  fromDay: string; // inclusive ISO yyyy-mm-dd
  toDay: string; // inclusive ISO yyyy-mm-dd
  language: string;
  browser: BrowserFilter;
}

/** ISO `yyyy-mm-dd` for `date` (UTC). */
function isoDay(date: Date): string {
  return date.toISOString().slice(0, 10);
}

/** The default window: the trailing 30 days ending today (aligned with /usage). */
export function defaultFilters(): LinguaFilters {
  const to = new Date();
  const from = new Date(to.getTime() - 29 * 24 * 60 * 60 * 1000);
  return { fromDay: isoDay(from), toDay: isoDay(to), language: "", browser: "" };
}

export const useLinguaStore = defineStore("lingua", () => {
  const report = ref<Async<LinguaReport>>(idle);
  const series = ref<Async<LinguaSeries>>(idle);
  /** Its own union: a failed online-usage read leaves the rest of the console standing. */
  const online = ref<Async<OnlineUsage>>(idle);
  /** The installation funnel and the leaving reasons (change: add-lingua-usage-counts): anonymous
   *  weekly counters, each its own union so one failed read leaves the other standing. */
  const funnel = ref<Async<InstallationFunnel>>(idle);
  const reasons = ref<Async<LeaveReasons>>(idle);
  const filters = reactive<LinguaFilters>(defaultFilters());

  function currentWindow() {
    return { fromDay: filters.fromDay, toDay: filters.toDay };
  }

  /** The funnel + the leaving reasons over the console's window, both for the same browser filter.
   *  "Not yet" is decided against today's UTC day, at load time. */
  function installationReads(): Promise<unknown>[] {
    const win = currentWindow();
    const browser = filters.browser; // "" => every family
    const today = todayUtc();
    return [
      run(funnel, async () => {
        const r = await api().lingua.adminGetUsageFunnel({ window: win, browser });
        const weeks = r.weeks.map((w) => ({
          week: w.week,
          installed: Number(w.installed),
          setUp: Number(w.setUp),
          firstAnalysis: Number(w.firstAnalysis),
          returnedDay1: Number(w.returnedDay1),
          returnedDays7To13: Number(w.returnedDays7To13),
          returnedDays30To59: Number(w.returnedDays30To59),
          capped: w.capped,
        }));
        return installationFunnel(weeks, win, today);
      }),
      run(reasons, async () => {
        const r = await api().lingua.adminGetLeaveReasons({ window: win, browser });
        return {
          reasons: orderReasons(r.reasons.map((x) => ({ reason: x.reason, count: Number(x.count) }))),
          capped: r.capped,
        } satisfies LeaveReasons;
      }),
    ];
  }

  /** Re-read the Installations section alone — on a change of its browser filter. */
  async function loadInstallations(next: { browser?: BrowserFilter } = {}) {
    if (next.browser !== undefined) filters.browser = next.browser;
    await Promise.all(installationReads());
  }

  /** Load the tiles/breakdown + the per-day series for the current filters. */
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
          wordsRead: Number(r.wordsRead),
          newWordsSeen: Number(r.newWordsSeen),
          byLanguage: r.byLanguage.map((l) => ({
            language: l.language,
            activeAccounts: Number(l.activeAccounts),
            wordsLearned: Number(l.wordsLearned),
            reviews: Number(l.reviews),
            wordsRead: Number(l.wordsRead),
            newWordsSeen: Number(l.newWordsSeen),
          })),
        } satisfies LinguaReport;
      }),
      run(series, async () => {
        const mapPoints = (pts: { day: string; value: bigint }[]): SeriesPoint[] =>
          pts.map((p) => ({ day: p.day, value: Number(p.value) }));
        const one = (metric: LinguaSeriesMetric) =>
          api().lingua.adminGetLinguaUsageSeries({ window: currentWindow(), metric, language });
        const [w, rv, read, fresh] = await Promise.all([
          one(LinguaSeriesMetric.LINGUA_SERIES_WORDS_LEARNED),
          one(LinguaSeriesMetric.LINGUA_SERIES_REVIEWS),
          one(LinguaSeriesMetric.LINGUA_SERIES_EXPOSURES), // words read (the wire name predates it)
          one(LinguaSeriesMetric.LINGUA_SERIES_UNKNOWN_SEEN),
        ]);
        return {
          wordsLearned: mapPoints(w.points),
          reviews: mapPoints(rv.points),
          wordsRead: mapPoints(read.points),
          newWordsSeen: mapPoints(fresh.points),
        } satisfies LinguaSeries;
      }),
      run(online, async () => {
        const r = await api().lingua.adminGetOnlineTranslationUsage({ window: currentWindow() });
        return {
          answered: Number(r.answered),
          failed: Number(r.failed),
          limited: Number(r.limited),
          costMicroUsd: Number(r.costMicroUsd),
          lines: foldOnlineUsage(
            r.rows.map((x) => ({
              provider: x.provider,
              pair: x.pair,
              answered: Number(x.answered),
              failed: Number(x.failed),
              limited: Number(x.limited),
              inputTokens: Number(x.inputTokens),
              outputTokens: Number(x.outputTokens),
              costMicroUsd: Number(x.costMicroUsd),
            })),
          ),
        } satisfies OnlineUsage;
      }),
      ...installationReads(),
    ]);
  }

  return { report, series, online, funnel, reasons, filters, load, loadInstallations };
});
