import type { AsyncStorageArea } from "./storage.ts";

// Local learning counters per UTC day (add-lingua-connected-clients §2.5 / §3), the
// source for both the stats screen (signed out) and the UpsertDailyStats push (signed
// in, summed across devices server-side). The MVP studies English only, so a day's
// entry is implicitly language "en". Best-effort: a lost increment under a rare
// concurrent write is acceptable — these are approximate activity counts, not state.
//
// Definitions (kept honest and documented, since the protocol names but does not define
// them): `exposures` = studied words the reader scanned, added once per page load (a
// re-scroll does not re-count; the reader guards it in memory); `wordsLearned` = a word
// marked known (via "Je connais" or review "I know this"); `reviews` = a card graded.
// Agent-session activity (the Claude Code plugin) is never counted here — it is local to
// ~/.lingua and does not sync.

const KEY = "cymbra-lingua-daily";

export interface DailyStat {
  exposures: number;
  wordsLearned: number;
  reviews: number;
}

/** UTC day number → counts (the day number is the DailyStat.day wire key). */
export type DailyStats = Record<number, DailyStat>;

/** The UTC day number (days since the Unix epoch) for an epoch-millis instant. */
export function utcDay(nowMs: number): number {
  return Math.floor(nowMs / 86_400_000);
}

export async function loadDailyStats(area: AsyncStorageArea): Promise<DailyStats> {
  const raw = (await area.get(KEY))[KEY];
  return raw && typeof raw === "object" ? (raw as DailyStats) : {};
}

async function bump(area: AsyncStorageArea, day: number, field: keyof DailyStat, by: number): Promise<void> {
  const stats = await loadDailyStats(area);
  const current = stats[day] ?? { exposures: 0, wordsLearned: 0, reviews: 0 };
  stats[day] = { ...current, [field]: current[field] + by };
  await area.set({ [KEY]: stats });
}

/** Record `n` studied-word exposures for the day (no-op for a non-positive count). */
export async function recordExposures(area: AsyncStorageArea, day: number, n: number): Promise<void> {
  if (n > 0) await bump(area, day, "exposures", n);
}

/** Record one word marked known for the day. */
export function recordWordLearned(area: AsyncStorageArea, day: number): Promise<void> {
  return bump(area, day, "wordsLearned", 1);
}

/** Record one review graded for the day. */
export function recordReview(area: AsyncStorageArea, day: number): Promise<void> {
  return bump(area, day, "reviews", 1);
}

/** A recorder for the ReviewController: "review" on a grade, "learned" on mark-known. */
export function dailyRecorder(area: AsyncStorageArea): (event: "review" | "learned") => void {
  return (event) => {
    const day = utcDay(Date.now());
    void (event === "review" ? recordReview(area, day) : recordWordLearned(area, day));
  };
}
