import type { LinguaPort } from "../analyzer/port.ts";
import type { NativeLanguage } from "../analyzer/types.ts";
import type { AsyncStorageArea } from "./storage.ts";

// Local learning counters per UTC day (add-lingua-connected-clients §2.5 / §3), the
// source for both the stats screen (signed out) and the UpsertDailyStats push (signed
// in, summed across devices server-side), per studied language since
// add-lingua-language-stats-review, each day labelled with the native language it was
// counted under since add-lingua-native-language-sync-client. Best-effort: a lost increment
// under a rare concurrent write is acceptable — these are approximate activity counts, not
// state.
//
// Definitions (refine-lingua-reading-stats): `exposures` = words read — the counted
// occurrences of the blocks the reader actually saw (on screen past the reading dwell),
// each block once per document shown; `unknownSeen` = among them, the unknown or
// being-learned ones (pushed for the back office, not shown in the extension);
// `wordsLearned` = a word marked known (via "Je connais" or review "I know this");
// `reviews` = a card graded; `native` = the native language of the device when the day
// was counted — the last one, when it changed (add-lingua-native-language-sync-client D3).
// Agent-session activity (the Claude Code plugin) is never counted here — it is local to
// ~/.lingua and does not sync.

/**
 * `-v4`: day → studied language → counts and the native language of the day
 * (add-lingua-native-language-sync-client D3). Until the first write here, the counts kept under
 * {@link PER_LANGUAGE_DAILY_KEY} are read as French-native, and those under
 * {@link DAY_ONLY_DAILY_KEY} as English studied, French native.
 */
export const DAILY_KEY = "cymbra-lingua-daily-v4";

/**
 * `-v3`: day → studied language → counts, with no native language (add-lingua-language-stats-review
 * D3): every device was French-native then. Kept, like `-v2`, so a downgraded build still reads it;
 * never written again by this build.
 */
export const PER_LANGUAGE_DAILY_KEY = "cymbra-lingua-daily-v3";

/**
 * `-v2`: the counts kept per day only, all of them English (every reader read English then). The
 * counts kept before it were whole-document figures; the store owner drops that key once
 * ({@link RETIRED_DAILY_KEY}), so none of them is ever pushed.
 */
export const DAY_ONLY_DAILY_KEY = "cymbra-lingua-daily-v2";

/** Where the whole-document counts lived before refine-lingua-reading-stats. */
export const RETIRED_DAILY_KEY = "cymbra-lingua-daily";

/** The counts of one day in one studied language. */
export interface DailyCounts {
  exposures: number;
  unknownSeen: number;
  wordsLearned: number;
  reviews: number;
}

/** One day's record: its counts and the native language they were counted under. */
export interface DailyStat extends DailyCounts {
  native: NativeLanguage;
}

const ZERO: DailyCounts = { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 0 };

/**
 * The native language of every device before a day carried one (M22), and the gloss language of every
 * card before it was labelled: what the sync reads an absent label as, on a day or on a card.
 */
export const FRENCH: NativeLanguage = "fr";

/** UTC day number → studied language → record (the day number is the DailyStat.day wire key). */
export type DailyStats = Record<number, Record<string, DailyStat>>;

/** UTC day number → counts, for one language. */
export type DayCounts = Record<number, DailyCounts>;

/** The UTC day number (days since the Unix epoch) for an epoch-millis instant. */
export function utcDay(nowMs: number): number {
  return Math.floor(nowMs / 86_400_000);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * The records written before the native language was kept, every one of them French-native. A day that
 * is not a record (a `null` left by a bad write) is skipped rather than thrown on: this runs under every
 * record and every push, and one broken day must not stop them all.
 */
function frenchNative(perLanguage: Record<string, unknown>): DailyStats {
  return Object.fromEntries(
    Object.entries(perLanguage)
      .filter((entry): entry is [string, Record<string, DailyCounts>] => isRecord(entry[1]))
      .map(([day, byLanguage]) => [
        day,
        Object.fromEntries(
          Object.entries(byLanguage).map(([language, counts]) => [language, { ...counts, native: FRENCH }]),
        ),
      ]),
  ) as DailyStats;
}

/**
 * The daily records: v4, or — until the first write there — v3 read as French-native, or the
 * day-only v2 read as English studied and French native (add-lingua-native-language-sync-client D3).
 */
export async function loadDailyStats(area: AsyncStorageArea): Promise<DailyStats> {
  const got = await area.get([DAILY_KEY, PER_LANGUAGE_DAILY_KEY, DAY_ONLY_DAILY_KEY]);
  const labelled = got[DAILY_KEY];
  if (isRecord(labelled)) return labelled as DailyStats;
  const perLanguage = got[PER_LANGUAGE_DAILY_KEY];
  if (isRecord(perLanguage)) return frenchNative(perLanguage);
  const dayOnly = got[DAY_ONLY_DAILY_KEY];
  if (!isRecord(dayOnly)) return {};
  return frenchNative(
    Object.fromEntries(
      Object.entries(dayOnly as Record<string, DailyCounts>).map(([day, stat]) => [day, { en: stat }]),
    ),
  );
}

/** One language's counts, by day — the counts alone, not the native language they were counted under. */
export function countsOf(stats: DailyStats, language: string): DayCounts {
  const out: DayCounts = {};
  for (const [day, byLanguage] of Object.entries(stats)) {
    const stat = byLanguage?.[language];
    if (!stat) continue;
    const { exposures, unknownSeen, wordsLearned, reviews } = { ...ZERO, ...stat };
    out[Number(day)] = { exposures, unknownSeen, wordsLearned, reviews };
  }
  return out;
}

/** Add to a day's counts in `language`; the day takes `native`, the last native language it is counted under. */
async function bump(
  area: AsyncStorageArea,
  day: number,
  language: string,
  native: NativeLanguage,
  by: Partial<DailyCounts>,
): Promise<void> {
  const stats = await loadDailyStats(area);
  const byLanguage = { ...stats[day] };
  const current = { ...ZERO, ...byLanguage[language] };
  const next: DailyStat = { ...current, native };
  for (const [field, n] of Object.entries(by) as [keyof DailyCounts, number][]) next[field] = current[field] + n;
  byLanguage[language] = next;
  stats[day] = byLanguage;
  await area.set({ [DAILY_KEY]: stats });
}

/** Drop every local daily count, under every shape still read (a full wipe, e.g. after a Lingua-only erasure). */
export async function clearDailyStats(area: AsyncStorageArea): Promise<void> {
  await area.set({ [DAILY_KEY]: {}, [PER_LANGUAGE_DAILY_KEY]: {}, [DAY_ONLY_DAILY_KEY]: {} });
}

/** Record words read and new words seen for the day, in `language`, under `native` (no-op when nothing was read). */
export async function recordReading(
  area: AsyncStorageArea,
  day: number,
  read: number,
  unknown: number,
  language: string,
  native: NativeLanguage,
): Promise<void> {
  if (read > 0) await bump(area, day, language, native, { exposures: read, unknownSeen: Math.max(0, unknown) });
}

/** Record one word marked known for the day, in `language`, under `native`. */
export function recordWordLearned(
  area: AsyncStorageArea,
  day: number,
  language: string,
  native: NativeLanguage,
): Promise<void> {
  return bump(area, day, language, native, { wordsLearned: 1 });
}

/** Record one review graded for the day, in `language`, under `native`. */
export function recordReview(
  area: AsyncStorageArea,
  day: number,
  language: string,
  native: NativeLanguage,
): Promise<void> {
  return bump(area, day, language, native, { reviews: 1 });
}

/**
 * A recorder for the ReviewController: "review" on a grade, "learned" on mark-known, each counted
 * under the native language of `engine` — one per engine, fixed for the port's life, which the port
 * resolves asynchronously: the write follows that hop. A day is never labelled by guess: when the port
 * cannot answer, the event is logged and not recorded, rather than filed as French.
 */
export function dailyRecorder(
  area: AsyncStorageArea,
  engine: Pick<LinguaPort, "nativeLanguage">,
): (event: "review" | "learned", language?: string) => void {
  return (event, language = "en") => {
    const day = utcDay(Date.now());
    void engine
      .nativeLanguage()
      .then((native) =>
        event === "review" ? recordReview(area, day, language, native) : recordWordLearned(area, day, language, native),
      )
      .catch((e: unknown) => console.warn("[Cymbra Lingua] daily stat not recorded:", e));
  };
}
