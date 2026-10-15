import type { CardOp, LinguaPort, StatusOp } from "../analyzer/port.ts";
import type { NativeLanguage } from "../analyzer/types.ts";
import { FLAGS_KEY, flagOn, type FlagValues } from "../flags/store.ts";
import { INTERFACE_LANGUAGE_KEY, isInterfaceLanguage } from "../i18n/language.ts";
import { ONLINE_TRANSLATION_FLAG } from "../translate/online.ts";
import { TRANSLATION_HOST_KEY } from "../translate/setting.ts";
import {
  addDailyCounts,
  DAILY_KEY,
  type DailyStats,
  DAY_ONLY_DAILY_KEY,
  loadDailyStats,
  PER_LANGUAGE_DAILY_KEY,
} from "./dailystats.ts";
import {
  type AsyncStorageArea,
  ANDROID_VOICES_KEY,
  COLOURS_KEY,
  NEW_WORDS_PER_DAY_KEY,
  ONLINE_TRANSLATION_KEY,
  READER_DISPLAY_KEY,
  REMOTE_VOICES_KEY,
  REVIEW_LANGUAGE_KEY,
  VOICE_KEY,
} from "./storage.ts";

// The relay between the extension and the Cymbra Lingua app on the same Apple device
// (add-lingua-apple-review D3), through the container they share. Two payloads, both opaque to the
// native side, which only stores them:
// - the extension's state, one way: its backup and the preferences the review reads, written after
//   each save of the backup — the app reviews that snapshot;
// - the app's changes, the other way: the card and status ops its review produced since it loaded,
//   in the engine's sync shape — the extension applies them under last-write-wins, as it applies a
//   pull, so neither side's state ever replaces the other's.
// Nothing here leaves the device. Never a token: the extension's state names its account only by a
// digest of its id, so the app can tell whether it is signed in to the same one before it synchronises
// (add-lingua-app-deck-sync D7).

/** The payloads' version: a reader of another one ignores what it cannot read. */
export const RELAY_VERSION = 1;

/**
 * The preferences the app's review reads, carried with the backup (never the session; the account only
 * as a digest, `RELAYED_ACCOUNT_KEY`).
 * « Traduction étendue »'s setting — not its models' state, which is this device's extension's — and
 * « Traduction en ligne »'s: the app follows the extension's choices on the device
 * (add-lingua-review-translation-fallback D7, D9).
 */
export const RELAYED_PREFS = [
  INTERFACE_LANGUAGE_KEY,
  NEW_WORDS_PER_DAY_KEY,
  REVIEW_LANGUAGE_KEY,
  VOICE_KEY,
  ANDROID_VOICES_KEY,
  REMOTE_VOICES_KEY,
  COLOURS_KEY,
  READER_DISPLAY_KEY,
  TRANSLATION_HOST_KEY,
  ONLINE_TRANSLATION_KEY,
] as const;

/** The store's keys beside the backup the app shows: the daily counts its Statistiques draw. */
export const RELAYED_STORE_KEYS = [DAILY_KEY, PER_LANGUAGE_DAILY_KEY, DAY_ONLY_DAILY_KEY] as const;

/** The extension's state as the app reviews it. */
export interface SharedState {
  v: typeof RELAY_VERSION;
  /** The engine's backup, as the store holds it. */
  backup: string;
  /** When the extension saved it (epoch millis): the app says how old the deck it shows is. */
  savedAt: number;
  /** The relayed preferences the extension holds, by key. */
  prefs: Record<string, unknown>;
  /** The relayed store keys the extension holds, by key. */
  storeItems: Record<string, unknown>;
  /**
   * The extension's last snapshot of the account's online-translation access — the one flag, never
   * the session or the account (add-lingua-review-translation-fallback D9). It decides whether the
   * app shows « Traduction en ligne » before the app has a session of its own; the server stays the
   * authority. Absent from an older extension: no access.
   */
  onlineAccess: boolean;
  /**
   * The extension's account, as a digest of its id (`accountDigest`), never a token
   * (add-lingua-app-deck-sync D7): null when the extension is signed out; absent (undefined) from an
   * extension that predates the field — which the app tells apart, to ask for an update.
   */
  account?: string | null;
  /**
   * The account erasure mark the extension's store last acted on (epoch millis; 0 when none, and from
   * an older extension): a newer one on the server means the state predates an erasure (D8).
   */
  erasedAt: number;
}

/** Where the extension keeps its account's digest: chrome.storage.local, beside the relayed preferences. */
export const RELAYED_ACCOUNT_KEY = "cymbra-lingua-relay-account";

/**
 * The digest of a Cymbra account id the relay carries (add-lingua-app-deck-sync D7): SHA-256 of
 * « cymbra-lingua-relay: » and the id, in hex. Both sides compute it from `UserService.GetAccount`.
 */
export async function accountDigest(userId: string): Promise<string> {
  const bytes = new TextEncoder().encode(`cymbra-lingua-relay:${userId}`);
  const hash = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  return Array.from(hash, (b) => b.toString(16).padStart(2, "0")).join("");
}

/** The extension's snapshot of the online-translation flag, as its flag store keeps it (`FLAGS_KEY`). */
export function onlineAccessOf(snapshot: unknown): boolean {
  const values = (snapshot as { values?: unknown } | null | undefined)?.values;
  return values !== null && typeof values === "object" && flagOn(values as FlagValues, ONLINE_TRANSLATION_FLAG);
}

/** Where the extension keeps that snapshot: chrome.storage.local, beside the relayed preferences. */
export const RELAYED_FLAGS_KEY = FLAGS_KEY;

/** What a review counts in a day, in one studied language, and the native language it counted under. */
export interface ReviewCounts {
  reviews: number;
  wordsLearned: number;
  native: NativeLanguage;
}

/** A review's counts: UTC day → studied language → counts. */
export type DayReviewCounts = Record<string, Record<string, ReviewCounts>>;

/**
 * What each opening of the app counted, by its id: the whole of it each time, never an increment, so
 * a payload written again — or read again — adds nothing the extension has not added yet.
 */
export type AppDaily = Record<string, DayReviewCounts>;

/** What the app's review changed. */
export interface AppOps {
  v: typeof RELAY_VERSION;
  /** Every op dated from here on (epoch millis) is in the payload. */
  since: number;
  cardOps: CardOp[];
  statusOps: StatusOp[];
  /** The daily counts each opening of the app made; absent from an app that counted none. */
  daily?: AppDaily;
}

/**
 * Where the extension keeps what it already added of each opening's counts, and when it last did:
 * the store, beside the daily counts themselves.
 */
export const APPLIED_APP_DAILY_KEY = "cymbra-lingua-app-daily-applied";

/** How long the extension remembers an opening of the app: far longer than any wait for Safari. */
export const APPLIED_APP_DAILY_TTL_MS = 60 * 86_400_000;

function record(raw: unknown): Record<string, unknown> | null {
  if (typeof raw !== "string") return null;
  try {
    const parsed: unknown = JSON.parse(raw);
    return parsed !== null && typeof parsed === "object" && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : null;
  } catch {
    return null;
  }
}

function object(value: unknown): Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
}

export function encodeSharedState(
  state: Omit<SharedState, "v" | "onlineAccess" | "erasedAt"> & { onlineAccess?: boolean; erasedAt?: number },
): string {
  return JSON.stringify({
    v: RELAY_VERSION,
    ...state,
    onlineAccess: state.onlineAccess === true,
    erasedAt: state.erasedAt ?? 0,
  } satisfies SharedState);
}

/** The shared state in `raw`, or null for nothing, another version or anything unreadable. */
export function parseSharedState(raw: unknown): SharedState | null {
  const r = record(raw);
  if (r?.v !== RELAY_VERSION || typeof r.backup !== "string" || r.backup.length === 0) return null;
  return {
    v: RELAY_VERSION,
    backup: r.backup,
    savedAt: typeof r.savedAt === "number" ? r.savedAt : 0,
    prefs: object(r.prefs),
    storeItems: object(r.storeItems),
    onlineAccess: r.onlineAccess === true,
    // Absent stays absent: an extension that predates the field, apart from one signed out.
    ...("account" in r ? { account: typeof r.account === "string" && r.account ? r.account : null } : {}),
    erasedAt: typeof r.erasedAt === "number" && Number.isFinite(r.erasedAt) ? r.erasedAt : 0,
  };
}

export function encodeAppOps(ops: Omit<AppOps, "v">): string {
  return JSON.stringify({ v: RELAY_VERSION, ...ops } satisfies AppOps);
}

/** The app's ops in `raw`, or null for nothing, another version or anything unreadable. */
export function parseAppOps(raw: unknown): AppOps | null {
  const r = record(raw);
  if (r?.v !== RELAY_VERSION || !Array.isArray(r.cardOps) || !Array.isArray(r.statusOps)) return null;
  const daily = appDailyOf(r.daily);
  return {
    v: RELAY_VERSION,
    since: typeof r.since === "number" ? r.since : 0,
    cardOps: r.cardOps as CardOp[],
    statusOps: r.statusOps as StatusOp[],
    ...(daily ? { daily } : {}),
  };
}

const count = (n: unknown): number => (typeof n === "number" && Number.isFinite(n) && n > 0 ? Math.floor(n) : 0);

/** A review's counts as read from the outside: only whole, positive counts and a known native language. */
function dayCountsOf(raw: unknown): DayReviewCounts {
  const out: DayReviewCounts = {};
  for (const [day, byLanguage] of Object.entries(object(raw))) {
    if (!/^\d+$/.test(day)) continue;
    for (const [language, counts] of Object.entries(object(byLanguage))) {
      const c = object(counts);
      if (!isInterfaceLanguage(c.native)) continue;
      const reviews = count(c.reviews);
      const wordsLearned = count(c.wordsLearned);
      if (reviews === 0 && wordsLearned === 0) continue;
      (out[day] ??= {})[language] = { reviews, wordsLearned, native: c.native };
    }
  }
  return out;
}

function appDailyOf(raw: unknown): AppDaily | undefined {
  if (raw === undefined) return undefined;
  return Object.fromEntries(Object.entries(object(raw)).map(([id, counts]) => [id, dayCountsOf(counts)]));
}

/** `stats` with `added` counted in: the daily counts the app shows, its earlier openings' included. */
export function withReviewCounts(stats: DailyStats, ...added: DayReviewCounts[]): DailyStats {
  const out: DailyStats = structuredClone(stats);
  for (const counts of added) {
    for (const [day, byLanguage] of Object.entries(counts)) {
      for (const [language, c] of Object.entries(byLanguage)) {
        const today = (out[Number(day)] ??= {});
        const current = today[language] ?? {
          exposures: 0,
          unknownSeen: 0,
          wordsLearned: 0,
          reviews: 0,
          native: c.native,
        };
        today[language] = {
          ...current,
          reviews: current.reviews + c.reviews,
          wordsLearned: current.wordsLearned + c.wordsLearned,
          native: c.native,
        };
      }
    }
  }
  return out;
}

/** What a review counted between `before` and `after`, the daily counts it opened and now holds. */
export function reviewCountsBetween(before: DailyStats, after: DailyStats): DayReviewCounts {
  const out: DayReviewCounts = {};
  for (const [day, byLanguage] of Object.entries(after)) {
    for (const [language, stat] of Object.entries(byLanguage ?? {})) {
      const was = before[Number(day)]?.[language];
      const reviews = Math.max(0, stat.reviews - (was?.reviews ?? 0));
      const wordsLearned = Math.max(0, stat.wordsLearned - (was?.wordsLearned ?? 0));
      if (reviews === 0 && wordsLearned === 0) continue;
      (out[day] ??= {})[language] = { reviews, wordsLearned, native: stat.native };
    }
  }
  return out;
}

interface AppliedOpening {
  /** When the extension last added this opening's counts (epoch millis). */
  at: number;
  counts: DayReviewCounts;
}

/**
 * Add to the extension's daily counts what each opening of the app counted and the extension has not
 * added yet — the difference with what it added before for that opening, never the whole again — and
 * remember it. Openings not heard from in `APPLIED_APP_DAILY_TTL_MS` are forgotten. Resolves how many
 * counts were added.
 */
export async function applyAppDaily(store: AsyncStorageArea, daily: AppDaily, nowMs: number): Promise<number> {
  const raw = object((await store.get(APPLIED_APP_DAILY_KEY))[APPLIED_APP_DAILY_KEY]);
  const applied: Record<string, AppliedOpening> = {};
  for (const [id, entry] of Object.entries(raw)) {
    const e = object(entry);
    const at = typeof e.at === "number" ? e.at : 0;
    if (nowMs - at <= APPLIED_APP_DAILY_TTL_MS) applied[id] = { at, counts: dayCountsOf(e.counts) };
  }
  let added = 0;
  for (const [id, counts] of Object.entries(daily)) {
    const before = applied[id]?.counts ?? {};
    for (const [day, byLanguage] of Object.entries(counts)) {
      for (const [language, c] of Object.entries(byLanguage)) {
        const was = before[day]?.[language];
        const reviews = Math.max(0, c.reviews - (was?.reviews ?? 0));
        const wordsLearned = Math.max(0, c.wordsLearned - (was?.wordsLearned ?? 0));
        if (reviews === 0 && wordsLearned === 0) continue;
        await addDailyCounts(store, Number(day), language, c.native, { reviews, wordsLearned });
        added += reviews + wordsLearned;
      }
    }
    applied[id] = { at: nowMs, counts };
  }
  await store.set({ [APPLIED_APP_DAILY_KEY]: applied });
  return added;
}

/** The daily counts the shared state carries, as the app's statistics read them. */
export function sharedDailyStats(storeItems: Record<string, unknown>): Promise<DailyStats> {
  return loadDailyStats({ get: async () => storeItems, set: async () => {} });
}

/** `ms` down to its second: the deck dates its cards to the second, and exports them in millis. */
export function toSecond(ms: number): number {
  return Math.floor(ms / 1000) * 1000;
}

/** The ops `port` holds dated from `since` on: what a review begun then changed. */
export async function appOpsSince(port: LinguaPort, since: number): Promise<AppOps> {
  const [cards, statuses] = [await port.exportCardOps(), await port.exportStatusOps()];
  return {
    v: RELAY_VERSION,
    since,
    cardOps: cards.filter((op) => op.client_ts >= since),
    statusOps: statuses.filter((op) => op.updated_at >= since),
  };
}

/** Apply the app's ops to `port` under last-write-wins; resolves how many changed something. */
export async function applyAppOps(port: LinguaPort, ops: AppOps): Promise<number> {
  let changed = 0;
  if (ops.statusOps.length > 0) changed += await port.applyStatusChanges(ops.statusOps);
  if (ops.cardOps.length > 0) changed += await port.applyCardOps(ops.cardOps);
  return changed;
}
