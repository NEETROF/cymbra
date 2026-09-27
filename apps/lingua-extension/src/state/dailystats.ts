import type { AsyncStorageArea } from "./storage.ts";

// Local learning counters per UTC day (add-lingua-connected-clients §2.5 / §3), the
// source for both the stats screen (signed out) and the UpsertDailyStats push (signed
// in, summed across devices server-side). The MVP studies English only, so a day's
// entry is implicitly language "en". Best-effort: a lost increment under a rare
// concurrent write is acceptable — these are approximate activity counts, not state.
//
// Definitions (refine-lingua-reading-stats): `exposures` = words read — the counted
// occurrences of the blocks the reader actually saw (on screen past the reading dwell),
// each block once per document shown; `unknownSeen` = among them, the unknown or
// being-learned ones (pushed for the back office, not shown in the extension);
// `wordsLearned` = a word marked known (via "Je connais" or review "I know this");
// `reviews` = a card graded.
// Agent-session activity (the Claude Code plugin) is never counted here — it is local to
// ~/.lingua and does not sync.

/**
 * `-v2`: the counts kept under the previous key were whole-document figures; the store owner
 * drops that key once ({@link RETIRED_DAILY_KEY}), so none of them is ever pushed.
 */
export const DAILY_KEY = "cymbra-lingua-daily-v2";

/** Where the whole-document counts lived before refine-lingua-reading-stats. */
export const RETIRED_DAILY_KEY = "cymbra-lingua-daily";

export interface DailyStat {
  exposures: number;
  unknownSeen: number;
  wordsLearned: number;
  reviews: number;
}

const ZERO: DailyStat = { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 0 };

/** UTC day number → counts (the day number is the DailyStat.day wire key). */
export type DailyStats = Record<number, DailyStat>;

/** The UTC day number (days since the Unix epoch) for an epoch-millis instant. */
export function utcDay(nowMs: number): number {
  return Math.floor(nowMs / 86_400_000);
}

export async function loadDailyStats(area: AsyncStorageArea): Promise<DailyStats> {
  const raw = (await area.get(DAILY_KEY))[DAILY_KEY];
  return raw && typeof raw === "object" ? (raw as DailyStats) : {};
}

async function bump(area: AsyncStorageArea, day: number, by: Partial<DailyStat>): Promise<void> {
  const stats = await loadDailyStats(area);
  const current = { ...ZERO, ...stats[day] };
  const next = { ...current };
  for (const [field, n] of Object.entries(by) as [keyof DailyStat, number][]) next[field] = current[field] + n;
  stats[day] = next;
  await area.set({ [DAILY_KEY]: stats });
}

/** Drop every local daily count (a full wipe, e.g. after a Lingua-only erasure). */
export async function clearDailyStats(area: AsyncStorageArea): Promise<void> {
  await area.set({ [DAILY_KEY]: {} });
}

/** Record words read and new words seen for the day (no-op when nothing was read). */
export async function recordReading(area: AsyncStorageArea, day: number, read: number, unknown: number): Promise<void> {
  if (read > 0) await bump(area, day, { exposures: read, unknownSeen: Math.max(0, unknown) });
}

/** Record one word marked known for the day. */
export function recordWordLearned(area: AsyncStorageArea, day: number): Promise<void> {
  return bump(area, day, { wordsLearned: 1 });
}

/** Record one review graded for the day. */
export function recordReview(area: AsyncStorageArea, day: number): Promise<void> {
  return bump(area, day, { reviews: 1 });
}

/** A recorder for the ReviewController: "review" on a grade, "learned" on mark-known. */
export function dailyRecorder(area: AsyncStorageArea): (event: "review" | "learned") => void {
  return (event) => {
    const day = utcDay(Date.now());
    void (event === "review" ? recordReview(area, day) : recordWordLearned(area, day));
  };
}
