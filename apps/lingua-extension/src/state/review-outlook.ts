import type { AsyncStorageArea } from "./storage.ts";

// The review outlook (add-lingua-comeback-hints D2): what the toolbar icon and the popup read to
// count the cards a review would hold, without an engine. The background works it out after each
// change of the reader's data — lingua-wasm's `reviewOutlook` over the backup, which needs no pack —
// and keeps it in chrome.storage.local; every event only reads it. Counts and timestamps only: no
// word, no page. Reading it repeats one line of the core's allowance rule (`ReviewOutlook::count_at`
// in crates/lingua-core/src/decks/outlook.rs), held to the engine's `reviewableCount` by a test
// (test/review-outlook.spec.ts), so the two cannot drift apart unnoticed.

/** The format the core writes; another version is worked out again. */
export const OUTLOOK_VERSION = 1;

/** Where the background keeps it: a device key, derived from the backup, never synced. */
export const OUTLOOK_KEY = "cymbra-lingua-review-outlook";

/** The most upcoming due times the core keeps per language (`UPCOMING_MAX`). */
export const UPCOMING_MAX = 100;

/** One studied language's outlook, as the core writes it. */
export interface LanguageOutlook {
  /** The language's ISO 639-1 tag. */
  language: string;
  /** Its cards, retired ones included: the deck the popup shows away from a page (D8). */
  cards: number;
  /** The reviewed cards due when it was worked out. */
  due: number;
  /** The next reviewed cards' due times (epoch seconds), ascending, at most UPCOMING_MAX. */
  upcoming: number[];
  /** The never-reviewed cards waiting. */
  fresh: number;
  /** The cards first answered since the outlook's `dayStart`. */
  introduced: number;
}

/** Every studied language's outlook, the primary first. */
export interface ReviewOutlook {
  version: number;
  /** When it was worked out, epoch seconds. */
  computedAt: number;
  /** The reader's local midnight then, epoch seconds: the day `introduced` counts. */
  dayStart: number;
  languages: LanguageOutlook[];
}

/** One language's count at a time. */
export interface LanguageCount {
  language: string;
  count: number;
}

const isCount = (v: unknown): v is number => typeof v === "number" && Number.isInteger(v) && v >= 0;
const isTime = (v: unknown): v is number => typeof v === "number" && Number.isFinite(v);

function languageOutlookOf(value: unknown): LanguageOutlook | null {
  const v = (value ?? {}) as Record<string, unknown>;
  if (typeof v.language !== "string" || !Array.isArray(v.upcoming) || !v.upcoming.every(isTime)) return null;
  if (!isCount(v.cards) || !isCount(v.due) || !isCount(v.fresh) || !isCount(v.introduced)) return null;
  return {
    language: v.language,
    cards: v.cards,
    due: v.due,
    upcoming: [...(v.upcoming as number[])],
    fresh: v.fresh,
    introduced: v.introduced,
  };
}

/** A stored outlook, or null for one of another version, malformed, or absent. */
export function outlookOf(value: unknown): ReviewOutlook | null {
  const v = (value ?? {}) as Record<string, unknown>;
  if (v.version !== OUTLOOK_VERSION || !isTime(v.computedAt) || !isTime(v.dayStart)) return null;
  if (!Array.isArray(v.languages)) return null;
  const languages = v.languages.map(languageOutlookOf);
  if (languages.some((l) => l === null)) return null;
  return { version: OUTLOOK_VERSION, computedAt: v.computedAt, dayStart: v.dayStart, languages: languages as LanguageOutlook[] };
}

/** The core's JSON, read: null when it is not an outlook of this version. */
export function parseOutlook(json: string): ReviewOutlook | null {
  try {
    return outlookOf(JSON.parse(json));
  } catch {
    return null;
  }
}

export async function loadOutlook(area: AsyncStorageArea): Promise<ReviewOutlook | null> {
  return outlookOf((await area.get(OUTLOOK_KEY))[OUTLOOK_KEY]);
}

/**
 * What a review started at `now` would hold in one language, while the data is unchanged: its
 * reviewed cards due, those come due since, and the smaller of its new words waiting and what
 * is left of `perDay` today. `dayStart` is the reader's local midnight at `now`: later than the
 * outlook's, it is another day, on which no new word has been let in yet.
 */
export function languageCountAt(
  entry: LanguageOutlook,
  outlookDayStart: number,
  now: number,
  dayStart: number,
  perDay: number,
): number {
  let comeDue = 0;
  for (const at of entry.upcoming) {
    if (at > now) break;
    comeDue++;
  }
  const introduced = dayStart > outlookDayStart ? 0 : entry.introduced;
  return entry.due + comeDue + Math.min(entry.fresh, Math.max(0, perDay - introduced));
}

/** Each studied language's count at `now`, in the reader's order. */
export function countsAt(outlook: ReviewOutlook, now: number, dayStart: number, perDay: number): LanguageCount[] {
  return outlook.languages.map((entry) => ({
    language: entry.language,
    count: languageCountAt(entry, outlook.dayStart, now, dayStart, perDay),
  }));
}

/** What a review started at `now` would hold in `languages` — every language of the outlook when empty. */
export function countAt(
  outlook: ReviewOutlook,
  languages: readonly string[],
  now: number,
  dayStart: number,
  perDay: number,
): number {
  return countsAt(outlook, now, dayStart, perDay)
    .filter(({ language }) => languages.length === 0 || languages.includes(language))
    .reduce((sum, { count }) => sum + count, 0);
}

/**
 * Whether a language's upcoming times are used up at `now`: the list was cut at UPCOMING_MAX and
 * its last time has passed, so a card beyond it may be due — only a new outlook knows.
 */
export function usedUp(outlook: ReviewOutlook, now: number): boolean {
  return outlook.languages.some((entry) => {
    const last = entry.upcoming.at(-1);
    return entry.upcoming.length >= UPCOMING_MAX && last !== undefined && last <= now;
  });
}

/** A language's cards in the outlook, 0 for a language it does not hold. */
export function cardsOf(outlook: ReviewOutlook, language: string): number {
  return outlook.languages.find((entry) => entry.language === language)?.cards ?? 0;
}
