// The Lingua installation funnel and leaving reasons (change: add-lingua-usage-counts, design
// D13): pure functions over the anonymous weekly counters `LinguaAdminService` returns. The
// store folds the RPC answers through them; the view only formats what they produce.
//
// Privacy: the finest grain is a week and a browser family — no account, no identifier, no
// per-installation row — and nothing here derives a finer one.

/** The browser families the counts are filed under, in the filter's order. */
export const BROWSER_FAMILIES = ["chromium", "firefox", "safari"] as const;
export type BrowserFamily = (typeof BROWSER_FAMILIES)[number];
/** The console's browser filter: one family, or "" for every family. */
export type BrowserFilter = BrowserFamily | "";

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
 * The last day of each return's window, counted from day 0. A device sends a step only from the
 * Monday after the week it happened in (owner decision 2026-10-15), so a return's figure is final
 * once that Monday has come for the last day any installation of the week could return on — a
 * Sunday installation's: day 1 is the next Monday, reported from the Monday after it. The other
 * steps have no window: their figure only grows, and is shown as it stands.
 */
const RETURN_WINDOW_END: Partial<Record<FunnelStep, number>> = {
  returnedDay1: 1,
  returnedDays7To13: 13,
  returnedDays30To59: 59,
};

/** One funnel cell: a count with its share of the week's installations, or "not yet". */
export type FunnelCell =
  | {
      readonly kind: "count";
      readonly count: number;
      /** A rounded whole percent of "installed"; null when the week has no installation. */
      readonly share: number | null;
    }
  | { readonly kind: "notYet" };

/** One installation week as the RPC returns it, bigint already folded to number. */
export interface FunnelCounts {
  week: string; // yyyy-mm-dd, the Monday that starts the installation week
  installed: number;
  setUp: number;
  firstAnalysis: number;
  returnedDay1: number;
  returnedDays7To13: number;
  returnedDays30To59: number;
  capped: boolean;
}

/** One row of the console's table. */
export interface FunnelWeek {
  week: string;
  installed: number;
  cells: Record<FunnelStep, FunnelCell>;
  /** One of this week's counters reached the ceiling: its figures are lower bounds. */
  capped: boolean;
}

/** The section's funnel: the table's rows and the chart's one series. */
export interface InstallationFunnel {
  weeks: FunnelWeek[];
  series: { weeks: string[]; installed: number[] };
}

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

/** The first UTC day on which `step`'s figure for the week starting `week` is final, or null
 *  for a step that has no return window: the Monday after the week holding the window's last
 *  day for the week's last installation day (its Sunday) — week + 14, + 21 and + 70 days. */
export function finalOn(week: string, step: FunnelStep): string | null {
  const end = RETURN_WINDOW_END[step];
  if (end === undefined) return null;
  const lastReturn = mondayOf(addDays(week, 6 + end));
  return lastReturn === null ? null : addDays(lastReturn, 7);
}

/** One week's row: each step as a count and a share of "installed", a return whose window is
 *  still open on `today` as "not yet" — so a young week never reads as a poor one. */
export function funnelWeek(counts: FunnelCounts, today: string): FunnelWeek {
  const cell = (step: FunnelStep): FunnelCell => {
    const final = finalOn(counts.week, step);
    if (final !== null && today < final) return { kind: "notYet" };
    return { kind: "count", count: counts[step], share: shareOf(counts[step], counts.installed) };
  };
  return {
    week: counts.week,
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

/** At most this many weeks on the chart's axis (the console's day axis stops at 400 days). */
const MAX_WEEKS = 60;

/**
 * The chart's one series: installations per week, over every week the window touches (from the
 * Monday of `fromDay` to the Monday of `toDay`), a week with no installation at 0 — so the axis
 * does not skip a quiet week. A week the server returned outside that range is kept.
 */
export function installationsSeries(
  weeks: readonly { week: string; installed: number }[],
  fromDay: string,
  toDay: string,
): { weeks: string[]; installed: number[] } {
  const byWeek = new Map(weeks.map((w) => [w.week, w.installed]));
  const axis = new Set<string>();
  const first = mondayOf(fromDay);
  const last = mondayOf(toDay);
  if (first && last) {
    for (let w = first, i = 0; w <= last && i < MAX_WEEKS; w = addDays(w, 7), i++) axis.add(w);
  }
  for (const w of byWeek.keys()) axis.add(w);
  const sorted = [...axis].sort((a, b) => a.localeCompare(b));
  return { weeks: sorted, installed: sorted.map((w) => byWeek.get(w) ?? 0) };
}

/** Fold the funnel RPC's weeks into the section's table and chart. */
export function installationFunnel(
  weeks: readonly FunnelCounts[],
  range: { fromDay: string; toDay: string },
  today: string,
): InstallationFunnel {
  const sorted = [...weeks].sort((a, b) => a.week.localeCompare(b.week));
  return {
    weeks: sorted.map((w) => funnelWeek(w, today)),
    series: installationsSeries(sorted, range.fromDay, range.toDay),
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
