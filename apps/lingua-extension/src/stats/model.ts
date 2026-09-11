// Pure model for the stats screen (add-lingua-connected-clients §3.1). Turns a
// day-keyed count map — local aggregates when signed out, consolidated GetStats when
// signed in — into zero-filled per-metric series over a date window, plus totals. No
// DOM, no Chrome: the screen (stats.ts) renders this, and it is unit-tested directly.

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
