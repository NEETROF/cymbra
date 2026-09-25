import type { LinguaPort } from "../analyzer/port.ts";
import type { LemmaStatus } from "../analyzer/types.ts";
import type { SpeechSettings, VoicePreference } from "../reading/speech.ts";

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
 * Set when a session this device held was refused by the server (expired or revoked), so
 * every surface can say so at a glance instead of silently not syncing. The Session clears
 * it on any successful sign-in or refresh, and when the reader signs out on purpose.
 */
export const SESSION_LOST_KEY = "cymbra-lingua-session-lost";

/**
 * The voice the reader chose to hear read aloud, as its `voiceURI`; absent or null means the
 * automatic choice. A preference, not the reader's data: voice identifiers are per platform, so
 * it is never synchronised.
 */
export const VOICE_KEY = "cymbra-lingua-voice";

/**
 * Whether the reader allowed Android's own voices (Firefox for Android, which cannot tell whether
 * that engine synthesises on the device). Absent means not allowed. Per device, never synchronised.
 */
export const ANDROID_VOICES_KEY = "cymbra-lingua-android-voices";

/**
 * How the book reader lays a book out: `paginated` (the default — pages turned by tap, suited
 * to e-ink) or `scrolled` (one continuous column, for a laptop). A preference, set in the
 * Réglages view every host renders; the reader page follows its `storage.onChanged`.
 */
export const READER_FLOW_KEY = "cymbra-lingua-reader-flow";

export type ReaderFlow = "paginated" | "scrolled";

/** How the book reader shows a book's text: its size and the colour of its page. */
export const READER_DISPLAY_KEY = "cymbra-lingua-reader-display";

/** Paper (the book's own colours, on a light page) or dark (light text on the night page). */
export type ReaderTheme = "paper" | "dark";

export interface ReaderDisplay {
  /** The text size, in percent of the book's own. */
  textScale: number;
  theme: ReaderTheme;
}

/** The text sizes offered, in percent: small steps, none so large a line holds three words. */
export const TEXT_SCALE_MIN = 80;
export const TEXT_SCALE_MAX = 200;
export const TEXT_SCALE_STEP = 10;

export const DEFAULT_READER_DISPLAY: ReaderDisplay = { textScale: 100, theme: "paper" };

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

/** The chosen voice's `voiceURI`, or null for the automatic choice. */
export async function loadVoice(area: AsyncStorageArea): Promise<string | null> {
  const got = await area.get(VOICE_KEY);
  const uri = got[VOICE_KEY];
  return typeof uri === "string" && uri !== "" ? uri : null;
}

/** Keep a voice, or null to go back to the automatic choice. */
export async function saveVoice(area: AsyncStorageArea, voiceURI: string | null): Promise<void> {
  await area.set({ [VOICE_KEY]: voiceURI });
}

/** Whether Android's own voices may speak; absent means not allowed. */
export async function loadAndroidVoices(area: AsyncStorageArea): Promise<boolean> {
  const got = await area.get(ANDROID_VOICES_KEY);
  return got[ANDROID_VOICES_KEY] === true;
}

/** Allow or refuse Android's own voices. */
export async function saveAndroidVoices(area: AsyncStorageArea, allowed: boolean): Promise<void> {
  await area.set({ [ANDROID_VOICES_KEY]: allowed });
}

/** The read-aloud settings in `area`, followed in every context through `storage.onChanged`. */
export function storedVoicePreference(area: AsyncStorageArea): VoicePreference {
  const load = async (): Promise<SpeechSettings> => ({
    voice: await loadVoice(area),
    androidVoices: await loadAndroidVoices(area),
  });
  return {
    load,
    watch(onChange) {
      chrome.storage.onChanged.addListener((changes, areaName) => {
        if (areaName !== "local" || !(changes[VOICE_KEY] || changes[ANDROID_VOICES_KEY])) return;
        void load().then(onChange);
      });
    },
  };
}

/** A stored value read as a reader flow; anything but `scrolled` is the paginated default. */
export function readerFlowOf(value: unknown): ReaderFlow {
  return value === "scrolled" ? "scrolled" : "paginated";
}

/** How the book reader lays a book out; paginated unless the reader chose otherwise. */
export async function loadReaderFlow(area: AsyncStorageArea): Promise<ReaderFlow> {
  return readerFlowOf((await area.get(READER_FLOW_KEY))[READER_FLOW_KEY]);
}

export async function saveReaderFlow(area: AsyncStorageArea, flow: ReaderFlow): Promise<void> {
  await area.set({ [READER_FLOW_KEY]: flow });
}

/** A stored display, made safe: a size on the offered steps, a theme the reader knows. */
export function readerDisplayOf(value: unknown): ReaderDisplay {
  const v = (value ?? {}) as Partial<ReaderDisplay>;
  const raw = typeof v.textScale === "number" && Number.isFinite(v.textScale) ? v.textScale : 100;
  const stepped = Math.round(raw / TEXT_SCALE_STEP) * TEXT_SCALE_STEP;
  return {
    textScale: Math.min(TEXT_SCALE_MAX, Math.max(TEXT_SCALE_MIN, stepped)),
    theme: v.theme === "dark" ? "dark" : "paper",
  };
}

/** How the book reader shows text; the book's own size on paper unless the reader chose otherwise. */
export async function loadReaderDisplay(area: AsyncStorageArea): Promise<ReaderDisplay> {
  return readerDisplayOf((await area.get(READER_DISPLAY_KEY))[READER_DISPLAY_KEY]);
}

export async function saveReaderDisplay(area: AsyncStorageArea, display: ReaderDisplay): Promise<void> {
  await area.set({ [READER_DISPLAY_KEY]: readerDisplayOf(display) });
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
