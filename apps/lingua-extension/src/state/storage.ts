import type { LinguaPort } from "../analyzer/port.ts";
import type { LemmaStatus } from "../analyzer/types.ts";

// Versioned local state (designs D4 + the review change). The authoritative state is
// lingua-core's LinguaState, held by the WASM engine and persisted as its lossless
// backup string under one root key in the durable store the background owns (see
// `state/store.ts`). Every context restores from it and writes back through it, so the deck, FSRS
// schedule, statuses and calibration stay in lockstep and a backup file is a byte-for-
// byte export of the same thing.
//
// v2 is the backup-backed shape. v1 is the reading-only shape shipped by
// add-lingua-extension-reading (statuses + light cards + calibration); it is migrated
// forward through the engine without loss on first load.

export const STORAGE_VERSION = 2;
export const ROOT_KEY = "lingua";
export const DEFAULT_CALIBRATION = 3000;

/**
 * Whether the reader is enabled, kept under its own key (independent of the state
 * backup so toggling it never rewrites the deck/statuses). A global master switch:
 * every context reads it and reacts to its `storage.onChanged`. Default on.
 */
export const ENABLED_KEY = "cymbra-lingua-enabled";

/**
 * Whether the in-page HUD (the discreet percentage pill) is hidden by the reader. Its own
 * key, independent of the state backup, so toggling it never rewrites the deck/statuses;
 * every context reacts to its `storage.onChanged`. Absent means shown (the default).
 */
export const HUD_HIDDEN_KEY = "cymbra-lingua-hud-hidden";

/**
 * Where the reader dragged the in-page HUD pill. One position for every site (a preference,
 * not per-page state), kept beside the hidden flag for the same reasons; every tab follows
 * its `storage.onChanged`. Absent means the default, bottom-right.
 */
export const HUD_POSITION_KEY = "cymbra-lingua-hud-position";

/**
 * The pill rests against a side edge; `y` is its height as a fraction of the distance it can
 * travel (0 = top, 1 = bottom), so it keeps its place when the viewport is resized or rotated.
 */
export interface HudPosition {
  side: "left" | "right";
  y: number;
}

export const DEFAULT_HUD_POSITION: HudPosition = { side: "right", y: 1 };

/**
 * Set when a session this device held was refused by the server (expired or revoked), so
 * every surface can say so at a glance instead of silently not syncing. The Session clears
 * it on any successful sign-in or refresh, and when the reader signs out on purpose.
 */
export const SESSION_LOST_KEY = "cymbra-lingua-session-lost";

/** The minimal async storage surface we need; chrome.storage.local satisfies it. */
export interface AsyncStorageArea {
  get(keys: string | string[] | null): Promise<Record<string, unknown>>;
  set(items: Record<string, unknown>): Promise<void>;
}

/** A v1 (reading-only) card, as add-lingua-extension-reading stored it. */
interface V1Card {
  lemma: string;
  surface: string;
  sentence: string;
  createdAt: number;
}

/** The v1 (reading-only) stored shape. */
export interface V1State {
  statuses: Record<string, LemmaStatus>;
  cards: Record<string, V1Card>;
  calibration: number;
}

/** What the root key currently holds, classified. */
export type Stored = { kind: "v2"; backup: string } | { kind: "v1"; v1: V1State } | { kind: "empty" };

/** Classify a raw stored value into v2 / v1 / empty. */
export function classifyStored(raw: unknown): Stored {
  if (raw == null || typeof raw !== "object") return { kind: "empty" };
  const obj = raw as Record<string, unknown>;
  if (typeof obj.backup === "string") return { kind: "v2", backup: obj.backup };
  if (typeof obj.statuses === "object" && obj.statuses !== null) {
    return {
      kind: "v1",
      v1: {
        statuses: obj.statuses as Record<string, LemmaStatus>,
        cards: (typeof obj.cards === "object" && obj.cards !== null ? obj.cards : {}) as Record<string, V1Card>,
        calibration: typeof obj.calibration === "number" ? obj.calibration : DEFAULT_CALIBRATION,
      },
    };
  }
  return { kind: "empty" };
}

/** Read and classify the stored state. */
export async function loadStored(area: AsyncStorageArea): Promise<Stored> {
  const got = await area.get(ROOT_KEY);
  return classifyStored(got[ROOT_KEY]);
}

/** Persist a backup string under the root key. */
export async function saveBackup(area: AsyncStorageArea, backup: string): Promise<void> {
  await area.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup } });
}

/** Whether the reader is enabled; absent means on (the default for a fresh install). */
export async function loadEnabled(area: AsyncStorageArea): Promise<boolean> {
  const got = await area.get(ENABLED_KEY);
  return got[ENABLED_KEY] !== false;
}

/** Set the global enabled flag. */
export async function saveEnabled(area: AsyncStorageArea, enabled: boolean): Promise<void> {
  await area.set({ [ENABLED_KEY]: enabled });
}

/** Whether the in-page HUD is hidden; absent means shown (the default). */
export async function loadHudHidden(area: AsyncStorageArea): Promise<boolean> {
  const got = await area.get(HUD_HIDDEN_KEY);
  return got[HUD_HIDDEN_KEY] === true;
}

/** Set the HUD-hidden flag. */
export async function saveHudHidden(area: AsyncStorageArea, hidden: boolean): Promise<void> {
  await area.set({ [HUD_HIDDEN_KEY]: hidden });
}

/** Coerce a stored (untrusted) value to a HUD position; anything malformed is the default. */
export function parseHudPosition(raw: unknown): HudPosition {
  if (raw == null || typeof raw !== "object") return DEFAULT_HUD_POSITION;
  const { side, y } = raw as Record<string, unknown>;
  if ((side !== "left" && side !== "right") || typeof y !== "number" || !Number.isFinite(y)) {
    return DEFAULT_HUD_POSITION;
  }
  return { side, y: Math.min(1, Math.max(0, y)) };
}

/** Where the reader left the HUD pill; absent or malformed means bottom-right. */
export async function loadHudPosition(area: AsyncStorageArea): Promise<HudPosition> {
  const got = await area.get(HUD_POSITION_KEY);
  return parseHudPosition(got[HUD_POSITION_KEY]);
}

/** Remember where the reader left the HUD pill. */
export async function saveHudPosition(area: AsyncStorageArea, position: HudPosition): Promise<void> {
  await area.set({ [HUD_POSITION_KEY]: position });
}

/**
 * Load the authoritative state into a fresh engine (any context: content script or
 * side panel): restore a v2 backup, forward-migrate a v1 store, or seed the engine's
 * default backup on a fresh install. After this the engine holds the state and storage
 * carries its backup.
 */
export async function hydrateEngine(port: LinguaPort, area: AsyncStorageArea): Promise<void> {
  const stored = await loadStored(area);
  if (stored.kind === "v2") {
    await port.restore(stored.backup);
  } else if (stored.kind === "v1") {
    await saveBackup(area, await hydrateFromV1(port, stored.v1));
  } else {
    await saveBackup(area, await port.backup());
  }
}

/**
 * Forward-migrate a v1 state into the engine (calibration, statuses, learning cards)
 * and return the resulting backup string. Learning forms become deck cards carrying
 * their captured sentence; known/ignored forms become plain statuses.
 */
export async function hydrateFromV1(port: LinguaPort, v1: V1State): Promise<string> {
  await port.setCalibration(v1.calibration || DEFAULT_CALIBRATION);
  for (const [lemma, status] of Object.entries(v1.statuses)) {
    if (status === "learning") {
      const card = v1.cards[lemma];
      await port.addCard({
        lemma,
        surface: card?.surface ?? lemma,
        sentence: card?.sentence ?? "",
        url: "",
        gloss: null,
        capturedAt: card?.createdAt ?? 0,
      });
    } else {
      await port.setStatus(lemma, status);
    }
  }
  return port.backup();
}
