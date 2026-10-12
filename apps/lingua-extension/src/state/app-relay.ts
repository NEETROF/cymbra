import type { CardOp, LinguaPort, StatusOp } from "../analyzer/port.ts";
import { INTERFACE_LANGUAGE_KEY } from "../i18n/language.ts";
import { DAILY_KEY, DAY_ONLY_DAILY_KEY, PER_LANGUAGE_DAILY_KEY } from "./dailystats.ts";
import {
  ANDROID_VOICES_KEY,
  COLOURS_KEY,
  NEW_WORDS_PER_DAY_KEY,
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
// Nothing here leaves the device.

/** The payloads' version: a reader of another one ignores what it cannot read. */
export const RELAY_VERSION = 1;

/** The preferences the app's review reads, carried with the backup (never the session or the account). */
export const RELAYED_PREFS = [
  INTERFACE_LANGUAGE_KEY,
  NEW_WORDS_PER_DAY_KEY,
  REVIEW_LANGUAGE_KEY,
  VOICE_KEY,
  ANDROID_VOICES_KEY,
  REMOTE_VOICES_KEY,
  COLOURS_KEY,
  READER_DISPLAY_KEY,
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
}

/** What the app's review changed. */
export interface AppOps {
  v: typeof RELAY_VERSION;
  /** Every op dated from here on (epoch millis) is in the payload. */
  since: number;
  cardOps: CardOp[];
  statusOps: StatusOp[];
}

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

export function encodeSharedState(state: Omit<SharedState, "v">): string {
  return JSON.stringify({ v: RELAY_VERSION, ...state } satisfies SharedState);
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
  };
}

export function encodeAppOps(ops: Omit<AppOps, "v">): string {
  return JSON.stringify({ v: RELAY_VERSION, ...ops } satisfies AppOps);
}

/** The app's ops in `raw`, or null for nothing, another version or anything unreadable. */
export function parseAppOps(raw: unknown): AppOps | null {
  const r = record(raw);
  if (r?.v !== RELAY_VERSION || !Array.isArray(r.cardOps) || !Array.isArray(r.statusOps)) return null;
  return {
    v: RELAY_VERSION,
    since: typeof r.since === "number" ? r.since : 0,
    cardOps: r.cardOps as CardOp[],
    statusOps: r.statusOps as StatusOp[],
  };
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
