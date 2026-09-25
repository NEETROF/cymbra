// « Traduction étendue » — the reader's choice, per device (add-lingua-translation-delivery D2),
// and where its model stands. Both live in chrome.storage.local: on this device, never synced.
// The reader's synchronised state is the IndexedDB store the background owns, and neither key is
// part of it — so turning the setting on on the Mac starts no 25.8 MB download on the tablet.
//
// Every context reads these two keys and follows them through storage.onChanged; only the
// background writes them (translate/host/model-controller.ts). Surfaces may import this module:
// it holds no engine and constructs nothing.

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
 * Where the model stands on this device.
 * - absent: nothing asked (the setting is off).
 * - downloading: in progress, `received` of `total` bytes.
 * - ready: every file on the device and verified.
 * - failed: the download stopped for `reason`; the setting offers to try again.
 * - interrupted: its host was torn down mid-way; the setting offers to resume.
 * - removed: the setting is on and the browser has since removed the model; nothing is
 *   fetched again until the reader asks.
 */
export type ModelState =
  | { phase: "absent" }
  | { phase: "downloading"; received: number; total: number }
  | { phase: "ready" }
  | { phase: "failed"; reason: ModelFailure }
  | { phase: "interrupted"; received: number; total: number }
  | { phase: "removed" };

export const ABSENT: ModelState = { phase: "absent" };

/** Absent — or anything this version does not know — is "none": the engine runs nowhere. */
export function parseHost(raw: unknown): TranslationHost {
  return raw === "local" ? "local" : "none";
}

const bytes = (n: unknown): number => (typeof n === "number" && Number.isFinite(n) && n >= 0 ? n : 0);

/** A stored state, or `absent` when there is none or it cannot be read. */
export function parseModelState(raw: unknown): ModelState {
  const s = raw as { phase?: unknown; received?: unknown; total?: unknown; reason?: unknown } | null | undefined;
  switch (s?.phase) {
    case "downloading":
    case "interrupted":
      return { phase: s.phase, received: bytes(s.received), total: bytes(s.total) };
    case "ready":
    case "removed":
      return { phase: s.phase };
    case "failed":
      return {
        phase: "failed",
        reason: FAILURES.includes(s.reason as ModelFailure) ? (s.reason as ModelFailure) : "unknown",
      };
    default:
      return ABSENT;
  }
}

/** Whether a translation can be asked: the reader chose the device, and its model is there. */
export function modelReady(host: TranslationHost, state: ModelState): boolean {
  return host === "local" && state.phase === "ready";
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
