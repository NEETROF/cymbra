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

/**
 * Running totals of the ladder's band sizes: each level's words plus those of every
 * level below it. A band only counts the words introduced at its level, so this is
 * the figure comparable with overall vocabulary-size estimates per level.
 */
export function cumulativeTotals(rows: LevelRow[]): number[] {
  let sum = 0;
  return rows.map((r) => (sum += r.total));
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

/**
 * Where a marked word's decision came from, one section each in the list: the reader's
 * own "connu"/"ignoré" gesture (`decision`), a known confirmed automatically by reading
 * below the declared level (`reading`, provenance `exposure`), or "je connais" during a
 * review (`review`, provenance `srs`). An ignored word is always a decision.
 */
export type MarkedOrigin = "decision" | "reading" | "review";

/** The list's sections, in display order. */
export const MARKED_ORIGINS: readonly MarkedOrigin[] = ["decision", "reading", "review"];

/** A word marked "connu" or "ignoré" (both hide it from highlighting). Learning words
 *  are excluded — they live in the deck, not here. */
export interface MarkedWord {
  lemma: string;
  status: "known" | "ignored";
  origin: MarkedOrigin;
  updated_at: number;
}

/** The marked words split by origin, each newest decision first. */
export type MarkedGroups = Record<MarkedOrigin, MarkedWord[]>;

function markedOrigin(status: "known" | "ignored", provenance: string | undefined): MarkedOrigin {
  if (status === "known" && provenance === "exposure") return "reading";
  if (status === "known" && provenance === "srs") return "review";
  return "decision"; // manual, import, or an ignored word
}

/**
 * The explicitly known/ignored words from an `exportStatusOps` list, newest decision
 * first (ties alphabetical). Drives the "Mots marqués" management list, where each can be
 * put back "à apprendre". `exportStatusOps` returns explicit statuses plus withdrawn ones
 * (`cleared`, dropped here), never presumed-known. Clearing one withdraws the decision in
 * the engine, which resurfaces the word even below the declared level and stops reading
 * from re-confirming it.
 */
export function markedWords(
  ops: { lemma: string; status: string; provenance?: string; updated_at: number }[],
): MarkedWord[] {
  return ops
    .filter((o) => o.status === "known" || o.status === "ignored")
    .map((o) => {
      const status = o.status as "known" | "ignored";
      return { lemma: o.lemma, status, origin: markedOrigin(status, o.provenance), updated_at: o.updated_at };
    })
    .sort((a, b) => b.updated_at - a.updated_at || a.lemma.localeCompare(b.lemma));
}

/**
 * {@link markedWords} split into the list's sections, keeping the newest-first order
 * within each. Lets the reader's own decisions stay short and scannable while the
 * automatic confirmations (which grow with reading) sit apart.
 */
export function groupMarkedWords(
  ops: { lemma: string; status: string; provenance?: string; updated_at: number }[],
): MarkedGroups {
  const groups: MarkedGroups = { decision: [], reading: [], review: [] };
  for (const word of markedWords(ops)) groups[word.origin].push(word);
  return groups;
}
