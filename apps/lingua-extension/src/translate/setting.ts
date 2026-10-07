// « Traduction étendue » — the reader's choice, per device (add-lingua-translation-delivery D2),
// and where its model stands. Both live in chrome.storage.local: on this device, never synced.
// The reader's synchronised state is the IndexedDB store the background owns, and neither key is
// part of it — so turning the setting on on the Mac starts no 25.8 MB download on the tablet.
//
// Every context reads these two keys and follows them through storage.onChanged; only the
// background writes them (translate/host/model-controller.ts). Surfaces may import this module:
// it holds no engine and constructs nothing.

import { DEFAULT_NATIVE, pairOf, studiedOf } from "../analyzer/pairs.ts";

/**
 * Where the engine runs. Stored as a HOST, shown as one checkbox: add-lingua-remote-translation
 * widens this union with its third place, and no stored value has to migrate.
 */
export type TranslationHost = "none" | "local";

export const TRANSLATION_HOST_KEY = "cymbra-lingua-translation-host";
export const MODEL_STATE_KEY = "cymbra-lingua-model-state";

/** Why a download stopped, as the setting explains it — never the raw error, which is logged. */
export type ModelFailure = "network" | "unavailable" | "not-the-model" | "storage" | "unknown";

const FAILURES: readonly ModelFailure[] = ["network", "unavailable", "not-the-model", "storage", "unknown"];

/**
 * Where the models the reader's pairs need stand on this device
 * (generalise-lingua-translation-model-state D3, generalise-lingua-translation-routes-by-pair D3).
 * - absent: nothing asked (the setting is off).
 * - downloading: in progress, `received` of `total` bytes over every needed model.
 * - ready: every needed model on the device and verified; `models` names them, and `pairs` the
 *   reader's pairs whose whole route they make.
 * - missing: the setting is on and a needed model was never downloaded — a language the reader
 *   added; `total` bytes would fetch it, and `models` and `pairs` name what is already complete
 *   and translatable. Nothing is fetched until the reader asks.
 * - failed: the download stopped for `reason`; the setting offers to try again.
 * - interrupted: its host was torn down mid-way; the setting offers to resume.
 * - removed: the setting is on and the browser has since removed a model; nothing is
 *   fetched again until the reader asks.
 */
export type ModelState =
  | { phase: "absent" }
  | { phase: "downloading"; received: number; total: number }
  | { phase: "ready"; models: string[]; pairs: string[] }
  | { phase: "missing"; models: string[]; pairs: string[]; total: number }
  | { phase: "failed"; reason: ModelFailure }
  | { phase: "interrupted"; received: number; total: number }
  | { phase: "removed" };

export const ABSENT: ModelState = { phase: "absent" };

/** Absent — or anything this version does not know — is "none": the engine runs nowhere. */
export function parseHost(raw: unknown): TranslationHost {
  return raw === "local" ? "local" : "none";
}

const bytes = (n: unknown): number => (typeof n === "number" && Number.isFinite(n) && n >= 0 ? n : 0);

/**
 * The pairs a stored `ready` stood for before anything was recorded with it: the release before
 * generalise-lingua-translation-model-state wrote `ready` only once the English model — en-fr's
 * one model — was complete.
 */
const READY_BEFORE_PAIRS = ["en-fr"];

/** Model ids or pairs, as stored; a state written before they were named names none. */
const names = (raw: unknown): string[] =>
  Array.isArray(raw) ? raw.filter((name): name is string => typeof name === "string") : [];

/**
 * The pairs a stored state names. `pairs` since generalise-lingua-translation-routes-by-pair; before
 * it, `languages`, each read as the pair of the default native language, because every pair shipped
 * so far is glossed in French (M22, routes-by-pair D3). Neither: `before`. The next reconciliation
 * rewrites the state with pairs.
 */
function storedPairs(s: { pairs?: unknown; languages?: unknown }, before: string[]): string[] {
  if (s.pairs !== undefined) return names(s.pairs);
  if (s.languages !== undefined) return names(s.languages).map((language) => pairOf(language, DEFAULT_NATIVE));
  return before;
}

/** A stored state, or `absent` when there is none or it cannot be read. */
export function parseModelState(raw: unknown): ModelState {
  const s = raw as
    | {
        phase?: unknown;
        received?: unknown;
        total?: unknown;
        reason?: unknown;
        models?: unknown;
        pairs?: unknown;
        languages?: unknown;
      }
    | null
    | undefined;
  switch (s?.phase) {
    case "downloading":
    case "interrupted":
      return { phase: s.phase, received: bytes(s.received), total: bytes(s.total) };
    case "ready":
      return { phase: "ready", models: names(s.models), pairs: storedPairs(s, [...READY_BEFORE_PAIRS]) };
    case "missing":
      return { phase: "missing", models: names(s.models), pairs: storedPairs(s, []), total: bytes(s.total) };
    case "removed":
      return { phase: "removed" };
    case "failed":
      return {
        phase: "failed",
        reason: FAILURES.includes(s.reason as ModelFailure) ? (s.reason as ModelFailure) : "unknown",
      };
    default:
      return ABSENT;
  }
}

/** On, and with a state that names what is translatable. */
function recorded(host: TranslationHost, state: ModelState): state is ModelState & { pairs: string[] } {
  return host === "local" && (state.phase === "ready" || state.phase === "missing");
}

/**
 * Whether a translation through `pair` can be asked: the reader chose the device, and every model of
 * the pair's route is on it, as the background recorded (generalise-lingua-translation-model-state
 * D5). The background's gate, on the exact pair (generalise-lingua-translation-routes-by-pair D3).
 */
export function pairReady(host: TranslationHost, state: ModelState, pair: string): boolean {
  return recorded(host, state) && state.pairs.includes(pair);
}

/**
 * Whether a sentence in `language` can be asked from a page, which asks in the document's language
 * and never names a pair: true when a ready pair studies it (routes-by-pair D3). The device holds one
 * native language, so this gate and `pairReady` agree.
 */
export function languageReady(host: TranslationHost, state: ModelState, language: string): boolean {
  return recorded(host, state) && state.pairs.some((pair) => studiedOf(pair) === language);
}

/** The slice of chrome.storage.local this needs. */
export interface SettingArea {
  get(keys: string | string[]): Promise<Record<string, unknown>>;
  set(items: Record<string, unknown>): Promise<void>;
}

export interface TranslationSetting {
  host: TranslationHost;
  state: ModelState;
}

export async function loadTranslationSetting(area: SettingArea): Promise<TranslationSetting> {
  const got = await area.get([TRANSLATION_HOST_KEY, MODEL_STATE_KEY]);
  return { host: parseHost(got[TRANSLATION_HOST_KEY]), state: parseModelState(got[MODEL_STATE_KEY]) };
}

/** Write either or both, in one call: a surface never sees the host without its state. */
export async function saveTranslationSetting(area: SettingArea, setting: Partial<TranslationSetting>): Promise<void> {
  const items: Record<string, unknown> = {};
  if (setting.host) items[TRANSLATION_HOST_KEY] = setting.host;
  if (setting.state) items[MODEL_STATE_KEY] = setting.state;
  await area.set(items);
}
