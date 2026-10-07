import { DEFAULT_NATIVE, SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import type { LemmaStatus, NativeLanguage, StudiedLanguage } from "../analyzer/types.ts";
import type { SpeechSettings, VoicePreference } from "../reading/speech.ts";
import { nativeLanguageOf } from "./profile.ts";

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
 * Whether the reader allowed the browser's remote voices (Chrome's `Google …`) to stand in where
 * no voice of the studied language is on the device. Absent means not allowed. Per device, never
 * synchronised.
 */
export const REMOTE_VOICES_KEY = "cymbra-lingua-remote-voices";

/**
 * The last language the reader chose in the review (refine-lingua-review-language D3): the one it
 * opens in away from a page in a studied language. Per device, never synchronised, never in the
 * backup.
 */
export const REVIEW_LANGUAGE_KEY = "cymbra-lingua-review-language";

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

/**
 * How a page turns: `instant` (the default — one jump, which an e-ink screen shows once) or
 * `slide` (the page slides aside, and follows the finger on a swipe).
 */
export type ReaderTurn = "instant" | "slide";

export interface ReaderDisplay {
  /** The text size, in percent of the book's own. */
  textScale: number;
  theme: ReaderTheme;
  turn: ReaderTurn;
}

/** The text sizes offered, in percent: small steps, none so large a line holds three words. */
export const TEXT_SCALE_MIN = 80;
export const TEXT_SCALE_MAX = 200;
export const TEXT_SCALE_STEP = 10;

export const DEFAULT_READER_DISPLAY: ReaderDisplay = { textScale: 100, theme: "paper", turn: "instant" };

/** How unknown and learning words are marked, and the reader's page colours (add-lingua-colour-settings). */
export const COLOURS_KEY = "cymbra-lingua-colours";

/** The presets: Cymbra (the identity, the default) and two for e-ink screens. */
export const COLOUR_PRESETS = ["cymbra", "eink-mono", "eink-colour"] as const;
export type ColourPresetId = (typeof COLOUR_PRESETS)[number];

export const FILL_INTENSITIES = ["none", "light", "strong"] as const;
export type FillIntensity = (typeof FILL_INTENSITIES)[number];

export const UNDERLINE_STYLES = ["solid", "dotted", "dashed", "wavy", "double", "none"] as const;
export type UnderlineStyle = (typeof UNDERLINE_STYLES)[number];

export const UNDERLINE_THICKNESSES = ["thin", "thick"] as const;
export type UnderlineThickness = (typeof UNDERLINE_THICKNESSES)[number];

/** How one status is painted: a fill behind the word, an underline, and maybe its text's colour. */
export interface StatusColours {
  fill: { colour: string; intensity: FillIntensity };
  underline: { colour: string; style: UnderlineStyle; thickness: UnderlineThickness };
  /** The word's text colour; null keeps the page's own. */
  text: string | null;
}

/** A full set of colours, as the reader set them by hand. Every colour is `#rrggbb`. */
export interface Colours {
  unknown: StatusColours;
  learning: StatusColours;
  /** The paper page; a null text keeps the book's own colours. */
  paper: { background: string; text: string | null };
  /** The dark page; its text always replaces the book's (a book typeset for paper sets dark text). */
  dark: { background: string; text: string };
}

/** A preset, or the reader's own set — stored whole, so a later retune of a preset never moves it. */
export type ColourPreference = { preset: ColourPresetId } | { preset: "custom"; colours: Colours };

export const DEFAULT_COLOUR_PREFERENCE: ColourPreference = { preset: "cymbra" };

const HEX = /^#[0-9a-f]{6}$/i;

const oneOf = <T extends string>(values: readonly T[], v: unknown): v is T =>
  typeof v === "string" && (values as readonly string[]).includes(v);

const hexOf = (v: unknown): string | undefined => (typeof v === "string" && HEX.test(v) ? v.toLowerCase() : undefined);

function statusColoursOf(value: unknown): StatusColours | undefined {
  const v = (value ?? {}) as Record<string, Record<string, unknown> | unknown>;
  const fill = (v.fill ?? {}) as Record<string, unknown>;
  const underline = (v.underline ?? {}) as Record<string, unknown>;
  const fillColour = hexOf(fill.colour);
  const lineColour = hexOf(underline.colour);
  const text = v.text === null ? null : hexOf(v.text);
  if (!fillColour || !lineColour || text === undefined) return undefined;
  if (!oneOf(FILL_INTENSITIES, fill.intensity) || !oneOf(UNDERLINE_STYLES, underline.style)) return undefined;
  if (!oneOf(UNDERLINE_THICKNESSES, underline.thickness)) return undefined;
  return {
    fill: { colour: fillColour, intensity: fill.intensity },
    underline: { colour: lineColour, style: underline.style, thickness: underline.thickness },
    text,
  };
}

/** A stored set of colours, made safe; undefined when anything in it is malformed. */
export function coloursOf(value: unknown): Colours | undefined {
  const v = (value ?? {}) as Record<string, unknown>;
  const unknown = statusColoursOf(v.unknown);
  const learning = statusColoursOf(v.learning);
  const paper = (v.paper ?? {}) as Record<string, unknown>;
  const dark = (v.dark ?? {}) as Record<string, unknown>;
  const paperBackground = hexOf(paper.background);
  const paperText = paper.text === null ? null : hexOf(paper.text);
  const darkBackground = hexOf(dark.background);
  const darkText = hexOf(dark.text);
  if (!unknown || !learning || !paperBackground || paperText === undefined || !darkBackground || !darkText) {
    return undefined;
  }
  return {
    unknown,
    learning,
    paper: { background: paperBackground, text: paperText },
    dark: { background: darkBackground, text: darkText },
  };
}

/** A stored colour preference, made safe: an unknown preset or a malformed set is the default. */
export function colourPreferenceOf(value: unknown): ColourPreference {
  const v = (value ?? {}) as Record<string, unknown>;
  if (oneOf(COLOUR_PRESETS, v.preset)) return { preset: v.preset };
  if (v.preset === "custom") {
    const colours = coloursOf(v.colours);
    if (colours) return { preset: "custom", colours };
  }
  return DEFAULT_COLOUR_PREFERENCE;
}

export async function loadColourPreference(area: AsyncStorageArea): Promise<ColourPreference> {
  return colourPreferenceOf((await area.get(COLOURS_KEY))[COLOURS_KEY]);
}

export async function saveColourPreference(area: AsyncStorageArea, preference: ColourPreference): Promise<void> {
  await area.set({ [COLOURS_KEY]: colourPreferenceOf(preference) });
}

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

/**
 * The reader's native language, as a raw root value names it (`nativeLanguageOf` on a v2 backup):
 * French with no backup yet, or a reading-only (v1) store, which predates the profile. The one
 * reading of it, whether the value was just read (`storedNativeLanguage`) or is about to be written
 * (the owner's mirror of the interface language, state/store.ts).
 */
export function nativeLanguageOfStored(raw: unknown, pairs: readonly string[] = SHIPPED_PAIRS): NativeLanguage {
  const stored = classifyStored(raw);
  return stored.kind === "v2" ? nativeLanguageOf(stored.backup, pairs) : DEFAULT_NATIVE;
}

/**
 * The reader's native language, as their stored backup names it. Read before an engine loads its
 * first pack, which must be glossed in it (generalise-lingua-native-language D7).
 */
export async function storedNativeLanguage(
  area: AsyncStorageArea,
  pairs: readonly string[] = SHIPPED_PAIRS,
): Promise<NativeLanguage> {
  const got = await area.get(ROOT_KEY);
  return nativeLanguageOfStored(got[ROOT_KEY], pairs);
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

/** The last language chosen in the review, or null when none was (or what is kept is not one). */
export async function loadReviewLanguage(area: AsyncStorageArea): Promise<string | null> {
  const value = (await area.get(REVIEW_LANGUAGE_KEY))[REVIEW_LANGUAGE_KEY];
  return typeof value === "string" && value !== "" ? value : null;
}

/** Keep `language` as the last one chosen in the review. */
export async function saveReviewLanguage(area: AsyncStorageArea, language: StudiedLanguage): Promise<void> {
  await area.set({ [REVIEW_LANGUAGE_KEY]: language });
}

/**
 * The voice chosen for each studied language, by `voiceURI` (add-lingua-language-choice D4). A
 * language without one gets the automatic choice. A single `voiceURI` kept before voices were per
 * language is the English one.
 */
export type VoiceChoices = Readonly<Record<string, string>>;

/** A stored value read as voice choices: the old single string is English's. */
export function voiceChoicesOf(value: unknown): VoiceChoices {
  if (typeof value === "string") return value === "" ? {} : { en: value };
  if (value === null || typeof value !== "object") return {};
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).filter(
      (entry): entry is [string, string] => typeof entry[1] === "string" && entry[1] !== "",
    ),
  );
}

/** The chosen voices, per studied language. */
export async function loadVoices(area: AsyncStorageArea): Promise<VoiceChoices> {
  return voiceChoicesOf((await area.get(VOICE_KEY))[VOICE_KEY]);
}

/** Keep a voice for `language`, or null to go back to the automatic choice there. */
export async function saveVoice(area: AsyncStorageArea, language: string, voiceURI: string | null): Promise<void> {
  const next: Record<string, string> = { ...(await loadVoices(area)) };
  if (voiceURI) next[language] = voiceURI;
  else delete next[language];
  await area.set({ [VOICE_KEY]: next });
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

/** Whether remote voices may stand in; absent means not allowed. */
export async function loadRemoteVoices(area: AsyncStorageArea): Promise<boolean> {
  const got = await area.get(REMOTE_VOICES_KEY);
  return got[REMOTE_VOICES_KEY] === true;
}

/** Allow or refuse remote voices as a stand-in. */
export async function saveRemoteVoices(area: AsyncStorageArea, allowed: boolean): Promise<void> {
  await area.set({ [REMOTE_VOICES_KEY]: allowed });
}

/** The read-aloud settings in `area`, followed in every context through `storage.onChanged`. */
export function storedVoicePreference(area: AsyncStorageArea): VoicePreference {
  const load = async (): Promise<SpeechSettings> => ({
    voices: await loadVoices(area),
    androidVoices: await loadAndroidVoices(area),
    remoteVoices: await loadRemoteVoices(area),
  });
  return {
    load,
    watch(onChange) {
      chrome.storage.onChanged.addListener((changes, areaName) => {
        if (areaName !== "local" || !(changes[VOICE_KEY] || changes[ANDROID_VOICES_KEY] || changes[REMOTE_VOICES_KEY]))
          return;
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

/** A stored display, made safe: a size on the offered steps, a theme and a turn the reader knows. */
export function readerDisplayOf(value: unknown): ReaderDisplay {
  const v = (value ?? {}) as Partial<ReaderDisplay>;
  const raw = typeof v.textScale === "number" && Number.isFinite(v.textScale) ? v.textScale : 100;
  const stepped = Math.round(raw / TEXT_SCALE_STEP) * TEXT_SCALE_STEP;
  return {
    textScale: Math.min(TEXT_SCALE_MAX, Math.max(TEXT_SCALE_MIN, stepped)),
    theme: v.theme === "dark" ? "dark" : "paper",
    turn: v.turn === "slide" ? "slide" : "instant",
  };
}

/** How the book reader shows text; the book's own size on paper unless the reader chose otherwise. */
export async function loadReaderDisplay(area: AsyncStorageArea): Promise<ReaderDisplay> {
  return readerDisplayOf((await area.get(READER_DISPLAY_KEY))[READER_DISPLAY_KEY]);
}

export async function saveReaderDisplay(area: AsyncStorageArea, display: ReaderDisplay): Promise<void> {
  await area.set({ [READER_DISPLAY_KEY]: readerDisplayOf(display) });
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

/** The language of a reading-only (v1) store, which predates languages. */
const V1_LANGUAGE: StudiedLanguage = "en";

/**
 * Forward-migrate a v1 state into the engine (calibration, statuses, learning cards)
 * and return the resulting backup string. Learning forms become deck cards carrying
 * their captured sentence; known/ignored forms become plain statuses.
 */
export async function hydrateFromV1(port: LinguaPort, v1: V1State): Promise<string> {
  // A reading-only store predates languages: everything in it is English.
  const english = port.for(V1_LANGUAGE);
  await english.setCalibration(v1.calibration || DEFAULT_CALIBRATION);
  for (const [lemma, status] of Object.entries(v1.statuses)) {
    if (status === "learning") {
      const card = v1.cards[lemma];
      await english.addCard({
        lemma,
        surface: card?.surface ?? lemma,
        sentence: card?.sentence ?? "",
        url: "",
        gloss: null,
        capturedAt: card?.createdAt ?? 0,
      });
    } else {
      await english.setStatus(lemma, status);
    }
  }
  return port.backup();
}
