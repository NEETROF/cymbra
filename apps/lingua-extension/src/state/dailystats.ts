import type { AsyncStorageArea } from "./storage.ts";

// Local learning counters per UTC day (add-lingua-connected-clients §2.5 / §3), the
// source for both the stats screen (signed out) and the UpsertDailyStats push (signed
// in, summed across devices server-side), per studied language since
// add-lingua-language-stats-review. Best-effort: a lost increment under a rare
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
 * `-v3`: day → studied language → counts (add-lingua-language-stats-review D3). The counts kept
 * per day only under {@link DAY_ONLY_DAILY_KEY} are read as English until the first write here.
 */
export const DAILY_KEY = "cymbra-lingua-daily-v3";

/**
 * `-v2`: the counts kept per day only, all of them English (every reader read English then). The
 * counts kept before it were whole-document figures; the store owner drops that key once
 * ({@link RETIRED_DAILY_KEY}), so none of them is ever pushed.
 */
export const DAY_ONLY_DAILY_KEY = "cymbra-lingua-daily-v2";

/** Where the whole-document counts lived before refine-lingua-reading-stats. */
export const RETIRED_DAILY_KEY = "cymbra-lingua-daily";

export interface DailyStat {
  exposures: number;
  unknownSeen: number;
  wordsLearned: number;
  reviews: number;
}

const ZERO: DailyStat = { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 0 };

/** UTC day number → studied language → counts (the day number is the DailyStat.day wire key). */
export type DailyStats = Record<number, Record<string, DailyStat>>;

/** UTC day number → counts, for one language. */
export type DayCounts = Record<number, DailyStat>;

/** The UTC day number (days since the Unix epoch) for an epoch-millis instant. */
export function utcDay(nowMs: number): number {
  return Math.floor(nowMs / 86_400_000);
}

/** The daily counts per language: v3, or the day-only v2 read as English when v3 does not exist yet. */
export async function loadDailyStats(area: AsyncStorageArea): Promise<DailyStats> {
  const got = await area.get([DAILY_KEY, DAY_ONLY_DAILY_KEY]);
  const perLanguage = got[DAILY_KEY];
  if (perLanguage && typeof perLanguage === "object") return perLanguage as DailyStats;
  const dayOnly = got[DAY_ONLY_DAILY_KEY];
  if (!dayOnly || typeof dayOnly !== "object") return {};
  return Object.fromEntries(
    Object.entries(dayOnly as Record<string, DailyStat>).map(([day, stat]) => [day, { en: stat }]),
  ) as DailyStats;
}

/** One language's counts, by day. */
export function countsOf(stats: DailyStats, language: string): DayCounts {
  const out: DayCounts = {};
  for (const [day, byLanguage] of Object.entries(stats)) {
    const stat = byLanguage?.[language];
    if (stat) out[Number(day)] = { ...ZERO, ...stat };
  }
  return out;
}

async function bump(area: AsyncStorageArea, day: number, language: string, by: Partial<DailyStat>): Promise<void> {
  const stats = await loadDailyStats(area);
  const byLanguage = { ...stats[day] };
  const current = { ...ZERO, ...byLanguage[language] };
  const next = { ...current };
  for (const [field, n] of Object.entries(by) as [keyof DailyStat, number][]) next[field] = current[field] + n;
  byLanguage[language] = next;
  stats[day] = byLanguage;
  await area.set({ [DAILY_KEY]: stats });
}

/** Drop every local daily count (a full wipe, e.g. after a Lingua-only erasure). */
export async function clearDailyStats(area: AsyncStorageArea): Promise<void> {
  await area.set({ [DAILY_KEY]: {}, [DAY_ONLY_DAILY_KEY]: {} });
}

/** Record words read and new words seen for the day, in `language` (no-op when nothing was read). */
export async function recordReading(
  area: AsyncStorageArea,
  day: number,
  read: number,
  unknown: number,
  language = "en",
): Promise<void> {
  if (read > 0) await bump(area, day, language, { exposures: read, unknownSeen: Math.max(0, unknown) });
}

/** Record one word marked known for the day, in `language`. */
export function recordWordLearned(area: AsyncStorageArea, day: number, language = "en"): Promise<void> {
  return bump(area, day, language, { wordsLearned: 1 });
}

/** Record one review graded for the day, in `language`. */
export function recordReview(area: AsyncStorageArea, day: number, language = "en"): Promise<void> {
  return bump(area, day, language, { reviews: 1 });
}

/** A recorder for the ReviewController: "review" on a grade, "learned" on mark-known. */
export function dailyRecorder(area: AsyncStorageArea): (event: "review" | "learned", language?: string) => void {
  return (event, language = "en") => {
    const day = utcDay(Date.now());
    void (event === "review" ? recordReview(area, day, language) : recordWordLearned(area, day, language));
  };
}
