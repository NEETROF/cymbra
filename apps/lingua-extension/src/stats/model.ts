// Pure model for the stats screen (add-lingua-connected-clients §3.1, extended by
// add-lingua-cefr-levels). Turns a day-keyed count map — local aggregates when signed
// out, consolidated GetStats when signed in — into zero-filled per-metric series over a
// date window, plus totals; and derives the CEFR ladder's estimated position. No DOM,
// no Chrome: the screen renders this, and it is unit-tested directly.

import type { CefrLevel, LevelRow } from "../analyzer/types.ts";

/** The share of a level that must be known for it to count as "cleared". */
export const LEVEL_MASTERY = 0.9;

/**
 * The reader's estimated CEFR position from the ladder: the lowest level not yet
 * cleared (≥ {@link LEVEL_MASTERY} known = confirmed + presumed), i.e. their current
 * frontier. When every level with lemmas is cleared, the highest such level. `null`
 * when the pack carries no CEFR data (all bands empty).
 */
export function estimatedPosition(rows: LevelRow[]): CefrLevel | null {
  const withData = rows.filter((r) => r.total > 0);
  if (withData.length === 0) return null;
  for (const r of withData) {
    if ((r.confirmed + r.presumed) / r.total < LEVEL_MASTERY) return r.level;
  }
  return withData[withData.length - 1].level;
}

/** One day's counts (the three tracked metrics). */
export interface DayCounts {
  exposures: number;
  wordsLearned: number;
  reviews: number;
}

/** Counts keyed by UTC day number (days since the epoch). */
export type CountsByDay = Record<number, DayCounts>;

/** The supported windows, in days. */
export const RANGES = [7, 30, 90] as const;
export type Range = (typeof RANGES)[number];

/** Zero-filled per-metric arrays over `[fromDay, toDay]`, aligned to `days`, plus totals. */
export interface Series {
  days: number[];
  exposures: number[];
  wordsLearned: number[];
  reviews: number[];
  totals: DayCounts;
}

/** The inclusive day window ending today (`toDay`) that spans `window` days. */
export function dayWindow(toDay: number, window: Range): { fromDay: number; toDay: number } {
  return { fromDay: toDay - window + 1, toDay };
}

/** Build zero-filled metric series over the inclusive `[fromDay, toDay]` window. */
export function buildSeries(byDay: CountsByDay, fromDay: number, toDay: number): Series {
  const days: number[] = [];
  const exposures: number[] = [];
  const wordsLearned: number[] = [];
  const reviews: number[] = [];
  const totals: DayCounts = { exposures: 0, wordsLearned: 0, reviews: 0 };
  for (let day = fromDay; day <= toDay; day++) {
    const d = byDay[day] ?? { exposures: 0, wordsLearned: 0, reviews: 0 };
    days.push(day);
    exposures.push(d.exposures);
    wordsLearned.push(d.wordsLearned);
    reviews.push(d.reviews);
    totals.exposures += d.exposures;
    totals.wordsLearned += d.wordsLearned;
    totals.reviews += d.reviews;
  }
  return { days, exposures, wordsLearned, reviews, totals };
}

/** Normalise consolidated GetStats rows (summed per day across languages) to a map. */
export function consolidatedToMap(
  rows: { day: number; exposures: number; wordsLearned: number; reviews: number }[],
): CountsByDay {
  const out: CountsByDay = {};
  for (const r of rows) {
    const cur = out[r.day] ?? { exposures: 0, wordsLearned: 0, reviews: 0 };
    out[r.day] = {
      exposures: cur.exposures + r.exposures,
      wordsLearned: cur.wordsLearned + r.wordsLearned,
      reviews: cur.reviews + r.reviews,
    };
  }
  return out;
}
