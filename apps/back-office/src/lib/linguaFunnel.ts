// The Lingua installation funnel and leaving reasons (change: add-lingua-usage-counts, design
// D13): pure functions over the anonymous counters `LinguaAdminService` returns, filed by
// installation day. The store folds the RPC answers through them; the view only formats what they
// produce, by day or by week — the week view sums the same days, without another read.
//
// Privacy: the finest grain is a day and a browser family — no account, no identifier, no
// per-installation row — and nothing here derives a finer one.

/** The browser families the counts are filed under, in the filter's order. */
export const BROWSER_FAMILIES = ["chromium", "firefox", "safari"] as const;
export type BrowserFamily = (typeof BROWSER_FAMILIES)[number];
/** The console's browser filter: one family, or "" for every family. */
export type BrowserFilter = BrowserFamily | "";

/** The funnel's two views of the same read, in the toggle's order: by installation day (the
 *  grain the counts are filed under), or summed by week. */
export const FUNNEL_GRAINS = ["day", "week"] as const;
export type FunnelGrain = (typeof FUNNEL_GRAINS)[number];

/** The steps after "installed", in the table's column order. */
export const FUNNEL_STEPS = [
  "setUp",
  "firstAnalysis",
  "returnedDay1",
  "returnedDays7To13",
  "returnedDays30To59",
] as const;
export type FunnelStep = (typeof FUNNEL_STEPS)[number];

/**
 * The last day of each return's window, counted from the installation day (day 0). A device sends
 * a step as soon as it happens, at the extension's next wake (owner decision 2026-10-16), so a
 * return's figure is final the day after its window's last day. The other steps have no window:
 * their figure only grows, and is shown as it stands.
 */
const RETURN_WINDOW_END: Partial<Record<FunnelStep, number>> = {
  returnedDay1: 1,
  returnedDays7To13: 13,
  returnedDays30To59: 59,
};

/** One funnel cell: a count with its share of the row's installations, or "not yet". */
export type FunnelCell =
  | {
      readonly kind: "count";
      readonly count: number;
      /** A rounded whole percent of "installed"; null when the row has no installation. */
      readonly share: number | null;
    }
  | { readonly kind: "notYet" };

/** The counters of an installation day, or of a week's days summed. */
export interface StepCounts {
  installed: number;
  setUp: number;
  firstAnalysis: number;
  returnedDay1: number;
  returnedDays7To13: number;
  returnedDays30To59: number;
  /** One of the counters reached the ceiling: its figures are lower bounds. */
  capped: boolean;
}

/** One installation day as the RPC returns it, bigint already folded to number. */
export interface FunnelCounts extends StepCounts {
  day: string; // yyyy-mm-dd, the installation's local calendar day (day 0)
}

/** A week's days summed: `week` is the Monday that starts it. */
export interface WeekCounts extends StepCounts {
  week: string;
}

/** One row of the console's table: an installation day, or a week in the week view. */
export interface FunnelRow {
  /** yyyy-mm-dd: the installation day, or the Monday that starts the week. */
  period: string;
  installed: number;
  cells: Record<FunnelStep, FunnelCell>;
  /** One of the row's counters reached the ceiling: its figures are lower bounds. */
  capped: boolean;
}

/** One view of the funnel: the table's rows, latest first, and the chart's one series. */
export interface FunnelView {
  rows: FunnelRow[];
  series: { labels: string[]; installed: number[] };
}

/** The section's funnel, by day and by week — the toggle switches between them, no re-read. */
export type InstallationFunnel = Record<FunnelGrain, FunnelView>;

const DAY_MS = 24 * 60 * 60 * 1000;

function parseDay(day: string): Date | null {
  const d = new Date(`${day}T00:00:00Z`);
  return Number.isNaN(d.getTime()) ? null : d;
}

function isoDay(d: Date): string {
  return d.toISOString().slice(0, 10);
}

/** Today's UTC day, yyyy-mm-dd. */
export function todayUtc(now: Date = new Date()): string {
  return isoDay(now);
}

/** `day` moved by `n` days (UTC). */
export function addDays(day: string, n: number): string {
  const d = parseDay(day);
  return d ? isoDay(new Date(d.getTime() + n * DAY_MS)) : day;
}

/** The Monday that starts `day`'s week (UTC), or null for a malformed day. */
export function mondayOf(day: string): string | null {
  const d = parseDay(day);
  if (!d) return null;
  const sinceMonday = (d.getUTCDay() + 6) % 7;
  return isoDay(new Date(d.getTime() - sinceMonday * DAY_MS));
}

/** `count` as a rounded whole percent of `installed` (25 of 40 → 63), or null when nothing was installed. */
export function shareOf(count: number, installed: number): number | null {
  if (installed <= 0) return null;
  return Math.round((count * 100) / installed);
}

/** The first UTC day on which `step`'s figure is final for installations made up to `lastDay`,
 *  or null for a step that has no return window: the day after the window's last day — `lastDay`
 *  + 2, + 14 and + 60. Until then the return reads "not yet". */
export function finalFrom(lastDay: string, step: FunnelStep): string | null {
  const end = RETURN_WINDOW_END[step];
  return end === undefined ? null : addDays(lastDay, end + 1);
}

/** One row: each step as a count and a share of "installed", a return whose window is still open
 *  on `today` for `lastDay`'s installations as "not yet" — so a young row never reads as a poor one. */
function funnelRow(period: string, lastDay: string, counts: StepCounts, today: string): FunnelRow {
  const cell = (step: FunnelStep): FunnelCell => {
    const final = finalFrom(lastDay, step);
    if (final !== null && today < final) return { kind: "notYet" };
    return { kind: "count", count: counts[step], share: shareOf(counts[step], counts.installed) };
  };
  return {
    period,
    installed: counts.installed,
    cells: {
      setUp: cell("setUp"),
      firstAnalysis: cell("firstAnalysis"),
      returnedDay1: cell("returnedDay1"),
      returnedDays7To13: cell("returnedDays7To13"),
      returnedDays30To59: cell("returnedDays30To59"),
    },
    capped: counts.capped,
  };
}

/** An installation day's row: a return is final from the day after its window closed for that day. */
export function funnelDay(counts: FunnelCounts, today: string): FunnelRow {
  return funnelRow(counts.day, counts.day, counts, today);
}

/** A week's row: a return is final once its window has closed for the week's last day, its Sunday. */
export function funnelWeek(counts: WeekCounts, today: string): FunnelRow {
  return funnelRow(counts.week, addDays(counts.week, 6), counts, today);
}

/** The days summed by the Monday of each (their dates read as UTC calendar dates), ascending; a
 *  week is capped when any of its days is. A malformed day keeps a row of its own. */
export function sumByWeek(days: readonly FunnelCounts[]): WeekCounts[] {
  const byWeek = new Map<string, WeekCounts>();
  for (const d of days) {
    const week = mondayOf(d.day) ?? d.day;
    const cur = byWeek.get(week);
    byWeek.set(week, {
      week,
      installed: (cur?.installed ?? 0) + d.installed,
      setUp: (cur?.setUp ?? 0) + d.setUp,
      firstAnalysis: (cur?.firstAnalysis ?? 0) + d.firstAnalysis,
      returnedDay1: (cur?.returnedDay1 ?? 0) + d.returnedDay1,
      returnedDays7To13: (cur?.returnedDays7To13 ?? 0) + d.returnedDays7To13,
      returnedDays30To59: (cur?.returnedDays30To59 ?? 0) + d.returnedDays30To59,
      capped: (cur?.capped ?? false) || d.capped,
    });
  }
  return [...byWeek.values()].sort((a, b) => a.week.localeCompare(b.week));
}

/** At most this many days on the chart's day axis, like the console's other day series. */
const MAX_DAYS = 400;
/** At most this many weeks on the chart's week axis. */
const MAX_WEEKS = 60;

/** The axis from `first` to `last` by `step` days, at most `max` points, with the points the server
 *  returned outside it kept; each point's installations, 0 for a quiet one — so the axis does not
 *  skip a quiet day or week. */
function seriesOver(
  points: readonly { at: string; installed: number }[],
  first: string | null,
  last: string | null,
  step: number,
  max: number,
): { labels: string[]; installed: number[] } {
  const byPoint = new Map(points.map((p) => [p.at, p.installed]));
  const axis = new Set<string>();
  if (first && last) {
    for (let p = first, i = 0; p <= last && i < max; p = addDays(p, step), i++) axis.add(p);
  }
  for (const p of byPoint.keys()) axis.add(p);
  const sorted = [...axis].sort((a, b) => a.localeCompare(b));
  return { labels: sorted, installed: sorted.map((p) => byPoint.get(p) ?? 0) };
}

/** The day view's series: installations per day over every day of the window. */
export function dailyInstallations(
  days: readonly { day: string; installed: number }[],
  fromDay: string,
  toDay: string,
): { labels: string[]; installed: number[] } {
  const points = days.map((d) => ({ at: d.day, installed: d.installed }));
  const valid = (day: string) => (parseDay(day) ? day : null);
  return seriesOver(points, valid(fromDay), valid(toDay), 1, MAX_DAYS);
}

/** The week view's series: installations per week over every week the window touches (from the
 *  Monday of `fromDay` to the Monday of `toDay`). */
export function weeklyInstallations(
  weeks: readonly { week: string; installed: number }[],
  fromDay: string,
  toDay: string,
): { labels: string[]; installed: number[] } {
  const points = weeks.map((w) => ({ at: w.week, installed: w.installed }));
  return seriesOver(points, mondayOf(fromDay), mondayOf(toDay), 7, MAX_WEEKS);
}

/** Fold the funnel RPC's days into the section's two views: rows latest first, series ascending. */
export function installationFunnel(
  days: readonly FunnelCounts[],
  range: { fromDay: string; toDay: string },
  today: string,
): InstallationFunnel {
  const latestFirst = (rows: FunnelRow[]) => rows.sort((a, b) => b.period.localeCompare(a.period));
  const weeks = sumByWeek(days);
  return {
    day: {
      rows: latestFirst(days.map((d) => funnelDay(d, today))),
      series: dailyInstallations(days, range.fromDay, range.toDay),
    },
    week: {
      rows: latestFirst(weeks.map((w) => funnelWeek(w, today))),
      series: weeklyInstallations(weeks, range.fromDay, range.toDay),
    },
  };
}

/** The leaving page's six answers, in the page's fixed order (design D12). */
export const LEAVE_REASONS = ["not_using", "highlighting", "missing_language", "slow", "unclear", "other"] as const;

export interface ReasonTotal {
  reason: string;
  count: number;
}
export interface LeaveReasons {
  reasons: ReasonTotal[];
  /** One of the counters summed reached the ceiling. */
  capped: boolean;
}

/** The six reasons in their fixed order — 0 for one nobody gave — then any reason this console
 *  does not know yet, so a seventh answer added to the page never vanishes from the list. */
export function orderReasons(rows: readonly ReasonTotal[]): ReasonTotal[] {
  const totals = new Map<string, number>();
  for (const r of rows) totals.set(r.reason, (totals.get(r.reason) ?? 0) + r.count);
  const known: readonly string[] = LEAVE_REASONS;
  const unknown = [...totals.keys()].filter((r) => !known.includes(r)).sort((a, b) => a.localeCompare(b));
  return [...known, ...unknown].map((reason) => ({ reason, count: totals.get(reason) ?? 0 }));
}
