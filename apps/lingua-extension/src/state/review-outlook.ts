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
  /**
   * The digest of what it was worked out from in the backup (`outlookFingerprint`), kept beside it
   * by the background: a write whose digest is the same — reading exposures, statuses — changes no
   * figure here, and works nothing out again.
   */
  inputs?: string;
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
  return {
    version: OUTLOOK_VERSION,
    computedAt: v.computedAt,
    dayStart: v.dayStart,
    languages: languages as LanguageOutlook[],
    ...(typeof v.inputs === "string" ? { inputs: v.inputs } : {}),
  };
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
 * Whether the outlook cannot count at `now`, `dayStart` being the reader's local midnight then: the
 * clock is before the time it was worked out, or on an earlier day (it went back). The cards it
 * counted due may not be due yet, and the new words it counted introduced may not be: it would
 * over-count, so a new one is worked out instead (`ReviewOutlook::stale_at` in the core).
 */
export function staleAt(
  outlook: Pick<ReviewOutlook, "computedAt" | "dayStart">,
  now: number,
  dayStart: number,
): boolean {
  return now < outlook.computedAt || dayStart < outlook.dayStart;
}

/**
 * What a review started at `now` would hold in one language, while the data is unchanged: its
 * reviewed cards due, those come due since, and the smaller of its new words waiting and what
 * is left of `perDay` today. `dayStart` is the reader's local midnight at `now`: later than the
 * outlook's, it is another day, on which no new word has been let in yet. Null when the outlook
 * is stale at `now` (`staleAt`): only a new one knows.
 */
export function languageCountAt(
  entry: LanguageOutlook,
  outlook: Pick<ReviewOutlook, "computedAt" | "dayStart">,
  now: number,
  dayStart: number,
  perDay: number,
): number | null {
  if (staleAt(outlook, now, dayStart)) return null;
  let comeDue = 0;
  for (const at of entry.upcoming) {
    if (at > now) break;
    comeDue++;
  }
  const introduced = dayStart > outlook.dayStart ? 0 : entry.introduced;
  return entry.due + comeDue + Math.min(entry.fresh, Math.max(0, perDay - introduced));
}

/** Each studied language's count at `now`, in the reader's order; null when the outlook is stale. */
export function countsAt(
  outlook: ReviewOutlook,
  now: number,
  dayStart: number,
  perDay: number,
): LanguageCount[] | null {
  if (staleAt(outlook, now, dayStart)) return null;
  return outlook.languages.map((entry) => ({
    language: entry.language,
    count: languageCountAt(entry, outlook, now, dayStart, perDay) ?? 0,
  }));
}

/**
 * What a review started at `now` would hold in `languages` — every language of the outlook when
 * empty —, or null when the outlook is stale at `now`.
 */
export function countAt(
  outlook: ReviewOutlook,
  languages: readonly string[],
  now: number,
  dayStart: number,
  perDay: number,
): number | null {
  const counts = countsAt(outlook, now, dayStart, perDay);
  if (!counts) return null;
  return counts
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

/**
 * The top-level keys of a backup as the engine writes it (`LinguaState::to_backup`: pretty JSON,
 * one top-level key per line, two spaces in). A string in JSON never holds a raw line break, and a
 * nested key sits deeper, so this finds the top level only.
 */
const TOP_LEVEL_KEY = /\n {2}"([a-z_]+)": /g;

/** What the outlook is worked out from: the deck, and the profile that lists the studied languages. */
const OUTLOOK_SECTIONS: ReadonlySet<string> = new Set(["deck", "profile"]);

/**
 * The part of a backup the outlook reads (`reviewOutlook` in lingua-wasm: the deck and the studied
 * languages) — not the statuses, the exposures nor the scheduler's settings, which a page's reading
 * rewrites every few seconds. A backup not laid out as the engine writes it is read whole: any
 * change of it then counts as a change of the outlook's inputs.
 */
export function outlookInputs(backup: string): string {
  const keys = [...backup.matchAll(TOP_LEVEL_KEY)].map((m) => ({ key: m[1]!, at: m.index }));
  if (!keys.some(({ key }) => key === "deck")) return backup;
  return keys
    .map(({ key, at }, i) => (OUTLOOK_SECTIONS.has(key) ? backup.slice(at, keys[i + 1]?.at ?? backup.length) : ""))
    .join("");
}

/**
 * A digest of `outlookInputs(backup)` (SHA-256, hex): equal for two backups that hold the same
 * outlook. Null where the platform has no digest — every write then works the outlook out again.
 */
export async function outlookFingerprint(backup: string): Promise<string | null> {
  const subtle = globalThis.crypto?.subtle;
  if (!subtle) return null;
  const digest = await subtle.digest("SHA-256", new TextEncoder().encode(outlookInputs(backup)));
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}
