import type { LemmaStatus } from "../analyzer/types.ts";

// Versioned local state (design D4). Everything the extension owns — statuses, the
// deck of captured cards, calibration and preferences — lives under ONE root key in
// chrome.storage.local, with forward migration. The pack is an extension asset, not
// storage. Reconciliation with the server / the agent plugin belongs to the sync
// changes, not here: this is a self-contained local store.

/** Current schema version. Bump + add a migration step when the shape changes. */
export const SCHEMA_VERSION = 1;

/** The root key under which the whole state object is stored. */
export const ROOT_KEY = "lingua";

/** Default calibration: "I know the 3000 most common words." */
export const DEFAULT_CALIBRATION = 3000;

/** A captured deck card: a form plus the sentence it was seen in. */
export interface Card {
  /** Dictionary form (the deck key). */
  lemma: string;
  /** The surface text as first captured. */
  surface: string;
  /** The source sentence the word/phrase was seen in (context for review). */
  sentence: string;
  /** Capture time (epoch ms); the caller supplies the clock. */
  createdAt: number;
}

/** The whole extension state. */
export interface LinguaState {
  version: number;
  /** Explicit per-form statuses. Absent = classified by calibration only. */
  statuses: Record<string, LemmaStatus>;
  /** Deck cards, keyed by dictionary form. */
  cards: Record<string, Card>;
  /** Calibration threshold (frequency rank). */
  calibration: number;
  prefs: {
    /** Whether the user granted "always highlight" (<all_urls>). */
    autoHighlight: boolean;
  };
}

/** The minimal async storage surface we need; chrome.storage.local satisfies it. */
export interface AsyncStorageArea {
  get(keys: string | string[] | null): Promise<Record<string, unknown>>;
  set(items: Record<string, unknown>): Promise<void>;
}

/** A fresh default state. */
export function defaultState(): LinguaState {
  return {
    version: SCHEMA_VERSION,
    statuses: {},
    cards: {},
    calibration: DEFAULT_CALIBRATION,
    prefs: { autoHighlight: false },
  };
}

/**
 * Bring any stored shape up to the current schema without loss.
 *
 * `undefined` → defaults. A legacy shape from the proof-of-concept — a flat
 * `{ statuses, calib }` with no `version` — is treated as version 0 and folded
 * forward (calibration renamed, cards/prefs introduced). Unknown newer versions
 * are returned as-is (a forward-compatible read should not clobber them).
 */
export function migrate(raw: unknown): LinguaState {
  if (raw == null || typeof raw !== "object") return defaultState();
  const obj = raw as Record<string, unknown>;

  // Version 0: the pre-schema flat shape ({ statuses, calib }), no `version` field.
  let state: LinguaState;
  if (typeof obj.version !== "number") {
    state = {
      version: 1,
      statuses: isStatusMap(obj.statuses) ? obj.statuses : {},
      cards: {},
      calibration: typeof obj.calib === "number" ? obj.calib : DEFAULT_CALIBRATION,
      prefs: { autoHighlight: false },
    };
  } else {
    state = {
      version: obj.version,
      statuses: isStatusMap(obj.statuses) ? obj.statuses : {},
      cards: isCardMap(obj.cards) ? obj.cards : {},
      calibration: typeof obj.calibration === "number" ? obj.calibration : DEFAULT_CALIBRATION,
      prefs: {
        autoHighlight:
          typeof obj.prefs === "object" &&
          obj.prefs !== null &&
          (obj.prefs as Record<string, unknown>).autoHighlight === true,
      },
    };
  }

  // Future steps: `while (state.version < SCHEMA_VERSION) { ...; state.version++ }`.
  // A newer-than-known version is left untouched (never downgraded).
  if (state.version < SCHEMA_VERSION) state.version = SCHEMA_VERSION;
  return state;
}

function isStatusMap(v: unknown): v is Record<string, LemmaStatus> {
  if (typeof v !== "object" || v === null) return false;
  return Object.values(v).every((s) => s === "known" || s === "learning" || s === "ignored");
}

function isCardMap(v: unknown): v is Record<string, Card> {
  if (typeof v !== "object" || v === null) return false;
  return Object.values(v).every(
    (c) =>
      typeof c === "object" &&
      c !== null &&
      typeof (c as Card).lemma === "string" &&
      typeof (c as Card).createdAt === "number",
  );
}

/** Read and migrate the state. Persists the migrated shape when it changed on read. */
export async function loadState(area: AsyncStorageArea): Promise<LinguaState> {
  const got = await area.get(ROOT_KEY);
  const raw = got[ROOT_KEY];
  const state = migrate(raw);
  // Persist the upgrade so the next reader sees the current shape.
  if (raw == null || (raw as Record<string, unknown>).version !== state.version) {
    await saveState(area, state);
  }
  return state;
}

/** Write the whole state under the root key. */
export async function saveState(area: AsyncStorageArea, state: LinguaState): Promise<void> {
  await area.set({ [ROOT_KEY]: state });
}
