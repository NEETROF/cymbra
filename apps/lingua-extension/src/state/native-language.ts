import {
  DEFAULT_NATIVE,
  defaultPair,
  pairFor,
  pairsOf,
  SHIPPED_PAIRS,
  shippedNatives,
  studiedOf,
} from "../analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "../analyzer/types.ts";
import { type InterfaceLanguage, isInterfaceLanguage } from "../i18n/language.ts";
import { nativeLanguageOf, studiedLanguagesOf } from "./profile.ts";
import { type AsyncStorageArea, loadStored, ROOT_KEY, STORAGE_VERSION } from "./storage.ts";
import { type StoreChangeReason, watchStore } from "./store.ts";

// The reader's choice of native language (add-lingua-native-language-choice): a runtime message to
// the background, which owns the store, and what it does with it (D2) — kept here, apart from the
// background's wiring, so it is tested. The choice is offered only while two native languages or
// more have a shipped pair (D1): French and English since change 34 shipped es-en; French alone
// before it, when no surface showed it (M22).

/** The runtime message a surface sends when the reader chooses (D2). */
export const NATIVE_LANGUAGE_MESSAGE = "lingua-native-language";

export interface NativeLanguageMessage {
  type: typeof NATIVE_LANGUAGE_MESSAGE;
  /** The native language chosen, as an ISO 639-1 tag. */
  native: string;
  /**
   * A new install's preset (D4): applied only while the choice was never made, and only on a device
   * found new — an updated extension, marked as chosen or holding its reader's data, is never preset
   * (M22). A preset is not an answer: the choice stays to be made.
   */
  preset?: boolean;
}

/**
 * What the background answers: `changed` when the backup now names another native language (every
 * open page then reloads, the reading sessions are built anew); a refusal names its cause, never an
 * error's text.
 */
export type NativeLanguageReply = { ok: true; changed: boolean } | { ok: false; error: "no-pair" | "failed" };

export function isNativeLanguageMessage(message: unknown): message is NativeLanguageMessage {
  const m = message as Partial<NativeLanguageMessage> | null;
  return m?.type === NATIVE_LANGUAGE_MESSAGE && typeof m.native === "string";
}

/**
 * Set, in chrome.storage.local, once the native language is chosen — by the reader's answer in one
 * of the three places, by an update (`onInstalled`'s reason), or when a preset finds the reader's
 * data already there, so an installed extension is never asked (D4, M22). Never by a preset: while
 * unset, the popup's first run asks, and a popover closed before its answer asks again. A preference
 * of this device, never synced, never in the backup.
 */
export const NATIVE_CHOSEN_KEY = "cymbra-lingua-native-chosen";

/**
 * Set, in chrome.storage.local, once a preset found this device new — no reader's data in its
 * backup — and applied the browser's language (D4): from then on the reader's first statuses and
 * cards, or the preset's own profile, never pass for an installed extension's data, and the choice
 * is asked until it is answered.
 */
export const NATIVE_PRESET_KEY = "cymbra-lingua-native-preset";

/** Whether the choice of native language was made on this device (`NATIVE_CHOSEN_KEY`). */
export async function nativeLanguageChosen(preferences: Pick<AsyncStorageArea, "get">): Promise<boolean> {
  return (await preferences.get(NATIVE_CHOSEN_KEY))[NATIVE_CHOSEN_KEY] === true;
}

/** Mark the choice as made (`NATIVE_CHOSEN_KEY`). */
export async function markNativeLanguageChosen(preferences: Pick<AsyncStorageArea, "set">): Promise<void> {
  await preferences.set({ [NATIVE_CHOSEN_KEY]: true });
}

/**
 * What `onInstalled` says about the native language (D4, M22): an update marks the choice as made,
 * whatever the package ships, so an installed extension is never asked; a new install chooses at the
 * onboarding, or in the popup's first run when the onboarding tab did not open. Anything else — the
 * browser updated, a shared module — says nothing.
 */
export async function onExtensionInstalled(reason: string, preferences: Pick<AsyncStorageArea, "set">): Promise<void> {
  if (reason === "update") await markNativeLanguageChosen(preferences);
}

/**
 * Whether `value` names a native language this build knows. The interface languages are the native
 * languages — `InterfaceLanguage` is `NativeLanguage` (add-lingua-interface-language): the reader's
 * interface speaks their native language — so the one list serves both.
 */
function isNativeLanguage(value: unknown): value is NativeLanguage {
  return isInterfaceLanguage(value);
}

/** Whether the choice is shown anywhere: two native languages or more have a shipped pair (D1). */
export function nativeChoiceOffered(pairs: readonly string[] = SHIPPED_PAIRS): boolean {
  return shippedNatives(pairs).length >= 2;
}

/** The native languages the choice offers, in listed order: those a shipped pair is glossed in (D1). */
export function offeredNatives(pairs: readonly string[] = SHIPPED_PAIRS): NativeLanguage[] {
  return shippedNatives(pairs).filter(isNativeLanguage);
}

/**
 * Whether `backup` holds what a reader puts there (D4, M22): a status, a card, or a profile other
 * than the one every engine starts with — French, studying the default pair's language. Exposures do
 * not count: reading one page writes them, and a new install's first page may come before its popup.
 * A backup that does not parse counts: nothing is preset over it.
 */
export function holdsReaderData(backup: string, pairs: readonly string[] = SHIPPED_PAIRS): boolean {
  let parsed: { knowledge?: { statuses?: unknown }; deck?: { cards?: unknown } } | null;
  try {
    parsed = JSON.parse(backup) as typeof parsed;
  } catch {
    return true;
  }
  // Both are kept by studied language, then by lemma: one entry anywhere is the reader's.
  const anyEntry = (byLanguage: unknown): boolean =>
    typeof byLanguage === "object" &&
    byLanguage !== null &&
    Object.values(byLanguage).some(
      (entries) => typeof entries === "object" && entries !== null && Object.keys(entries).length > 0,
    );
  if (anyEntry(parsed?.knowledge?.statuses) || anyEntry(parsed?.deck?.cards)) return true;
  const fresh = defaultProfile(pairs);
  return nativeLanguageOf(backup, pairs) !== fresh.native || studiedLanguagesOf(backup).join() !== fresh.studied.join();
}

/** The profile every engine starts with: French, studying the default pair's language. */
function defaultProfile(pairs: readonly string[]): { native: NativeLanguage; studied: StudiedLanguage[] } {
  return {
    native: DEFAULT_NATIVE,
    studied: [studiedOf(defaultPair(DEFAULT_NATIVE, pairs) ?? pairs[0]) as StudiedLanguage],
  };
}

/**
 * A new install's preset (D4, M3, M13): the browser language's primary subtag when a shipped pair is
 * glossed in it, English when one is glossed in English, French otherwise — a reader whose browser is
 * in German is preset to English, and may pick another in the same step.
 */
export function presetNative(
  browserLanguage: string | undefined,
  pairs: readonly string[] = SHIPPED_PAIRS,
): NativeLanguage {
  const offered = offeredNatives(pairs);
  const primary = (browserLanguage ?? "").split(/[-_]/)[0].toLowerCase();
  const own = offered.find((native) => native === primary);
  if (own) return own;
  return offered.includes("en") ? "en" : DEFAULT_NATIVE;
}

/**
 * The studied languages of a reader who chooses `native` (D2): the current ones without `native` —
 * a reader never studies their native language — and without those no shipped pair glosses in it,
 * in the reader's order; or, when none is left, the studied language of `native`'s first shipped
 * pair. Null when no shipped pair is glossed in `native`: the choice is refused.
 */
export function studiedForNative(
  current: readonly StudiedLanguage[],
  native: string,
  pairs: readonly string[] = SHIPPED_PAIRS,
): StudiedLanguage[] | null {
  const first = defaultPair(native, pairs);
  if (first === null) return null;
  const kept = current.filter((language) => language !== native && pairFor(language, native, pairs) !== null);
  return kept.length > 0 ? [...new Set(kept)] : [studiedOf(first) as StudiedLanguage];
}

/** What the background lends the change (D2): its store, its preferences and its engines. */
export interface NativeLanguageHost {
  /** The owner's handle on the reader's data: a write with a reason mirrors the key, then announces it. */
  store: Pick<AsyncStorageArea, "get"> & {
    set(items: Record<string, unknown>, reason?: StoreChangeReason): Promise<void>;
  };
  /** chrome.storage.local: the markers. */
  preferences: AsyncStorageArea;
  /** See to it that the store holds a backup: a new install's onboarding may come before any page. */
  ensureBackup(): Promise<void>;
  /** lingua-wasm's `reprofileBackup` (analyzer/engine.ts). */
  reprofile(backup: string, native: NativeLanguage, studied: StudiedLanguage[]): Promise<string>;
  /**
   * Run `task` with every sync held — the sync scheduler's `exclusive`. A sync restores the backup,
   * applies what it pulled and saves the result: a change landing in between would be saved over
   * (the native language reverted) or save over the pulls, and an engine forgotten in between would
   * have the pulls applied to an empty state, saved over the reader's data.
   */
  exclusive<T>(task: () => Promise<T>): Promise<T>;
  /**
   * After the change, the background's reading engine restores the stored backup before its next
   * answer, and the restore rebuilds it for the new native language (D3). The sync engine restores
   * the backup at the start of every run, and follows by itself. No engine is ever forgotten: a port
   * whose engine is gone would serve a fresh state until something restored it.
   */
  rehydrate(): void;
  /** The pairs the package ships: the bundle's, unless a spec offers others. */
  pairs?: readonly string[];
}

/**
 * The reader chooses `message.native` (D2): refused when no shipped pair is glossed in it; otherwise,
 * with every sync held, the stored backup's profile is rewritten — that native language, studying
 * `studiedForNative` — and saved with its reason, so the owner writes the interface language's key
 * before it announces the change; then the background's reading engine restores it, and the choice
 * is marked as made. Choosing the native language the reader already has only marks it.
 *
 * A preset (D4) is no answer, and marks nothing: it is left alone once the choice was made, and on a
 * device whose backup holds its reader's data — an extension installed before this build, whose
 * update the browser did not report — which it marks as chosen instead (M22).
 */
export async function changeNativeLanguage(
  host: NativeLanguageHost,
  message: NativeLanguageMessage,
): Promise<NativeLanguageReply> {
  const pairs = host.pairs ?? SHIPPED_PAIRS;
  const native = message.native;
  if (!isNativeLanguage(native) || pairsOf(native, pairs).length === 0) {
    return { ok: false, error: "no-pair" };
  }
  return host.exclusive(() => rewriteProfile(host, native, message.preset === true, pairs));
}

/**
 * `changeNativeLanguage`'s reads, rewrite and write, run with every sync held — and after any change
 * asked before it, so a preset that arrives after an answer finds it marked.
 */
async function rewriteProfile(
  host: NativeLanguageHost,
  native: NativeLanguage,
  preset: boolean,
  pairs: readonly string[],
): Promise<NativeLanguageReply> {
  try {
    if (preset && (await nativeLanguageChosen(host.preferences))) return { ok: true, changed: false };
    let stored = await loadStored(host.store);
    if (stored.kind !== "v2") {
      await host.ensureBackup();
      stored = await loadStored(host.store);
    }
    if (stored.kind !== "v2") return { ok: false, error: "failed" };
    if (preset && !(await presetBefore(host.preferences))) {
      if (holdsReaderData(stored.backup, pairs)) {
        await markNativeLanguageChosen(host.preferences);
        return { ok: true, changed: false };
      }
      await host.preferences.set({ [NATIVE_PRESET_KEY]: true });
    }
    if (nativeLanguageOf(stored.backup, pairs) === native) {
      if (!preset) await markNativeLanguageChosen(host.preferences);
      return { ok: true, changed: false };
    }
    const studied = studiedForNative(studiedLanguagesOf(stored.backup), native, pairs) as StudiedLanguage[];
    const backup = await host.reprofile(stored.backup, native, studied);
    await host.store.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup } }, { type: "native-language", native });
    host.rehydrate();
    // Marked once the backup is written — a write that failed answers nothing. A page that reloads on
    // the announcement reads the marker long after this write was asked.
    if (!preset) await markNativeLanguageChosen(host.preferences);
    return { ok: true, changed: true };
  } catch (e) {
    console.warn("[Cymbra Lingua] could not change the native language:", e);
    return { ok: false, error: "failed" };
  }
}

/** Whether a preset already found this device new (`NATIVE_PRESET_KEY`). */
async function presetBefore(preferences: Pick<AsyncStorageArea, "get">): Promise<boolean> {
  return (await preferences.get(NATIVE_PRESET_KEY))[NATIVE_PRESET_KEY] === true;
}

/**
 * The reader's native language and studied languages as their stored backup holds them, without an
 * engine — what the onboarding and the popup's first run show the choice from (D4): French studying
 * the default pair's language before any backup, as every engine starts.
 */
export async function storedProfile(
  store: AsyncStorageArea,
  pairs: readonly string[] = SHIPPED_PAIRS,
): Promise<{ native: NativeLanguage; studied: StudiedLanguage[] }> {
  const stored = await loadStored(store);
  if (stored.kind === "v2") {
    return { native: nativeLanguageOf(stored.backup, pairs), studied: studiedLanguagesOf(stored.backup) };
  }
  return defaultProfile(pairs);
}

/** What a surface's preset needs: its preferences, the browser's language, and the background. */
export interface PresetDeps {
  /** chrome.storage.local: the marker. */
  preferences: Pick<AsyncStorageArea, "get">;
  /** `navigator.language`. */
  browserLanguage: string | undefined;
  /** The background, by default. */
  choose?: (native: NativeLanguage, preset: boolean) => Promise<NativeLanguageReply>;
  pairs?: readonly string[];
}

/**
 * A new install's preset, applied before the surface paints (D4): only while two native languages
 * ship and the choice was never made on this device. The background writes it — the backup the first
 * page wrote, or a fresh one — and the surface then reads the interface language it names. Resolves
 * true while the choice is still to be made, so the popup's first run asks the question — again at
 * each opening until the reader answers; false once it was made, the background having found an
 * installed extension's data included, and at once, reading nothing, while one native language
 * ships. Never rejects: the surface paints whatever happened.
 */
export async function presetNativeLanguage(deps: PresetDeps): Promise<boolean> {
  const pairs = deps.pairs ?? SHIPPED_PAIRS;
  if (!nativeChoiceOffered(pairs)) return false;
  try {
    if (await nativeLanguageChosen(deps.preferences)) return false;
    const choose = deps.choose ?? ((native, preset) => chooseNativeLanguage(native, preset));
    await choose(presetNative(deps.browserLanguage, pairs), true);
    return !(await nativeLanguageChosen(deps.preferences));
  } catch (e) {
    console.warn("[Cymbra Lingua] could not preset the native language:", e);
    return false;
  }
}

/** Ask the background to make `native` the reader's native language (D2); a refusal when it cannot be reached. */
export async function chooseNativeLanguage(
  native: NativeLanguage,
  preset = false,
  send: (message: NativeLanguageMessage) => Promise<unknown> = (message) => chrome.runtime.sendMessage(message),
): Promise<NativeLanguageReply> {
  try {
    const message: NativeLanguageMessage = preset
      ? { type: NATIVE_LANGUAGE_MESSAGE, native, preset }
      : { type: NATIVE_LANGUAGE_MESSAGE, native };
    const reply = (await send(message)) as NativeLanguageReply | undefined;
    return reply ?? { ok: false, error: "failed" };
  } catch {
    return { ok: false, error: "failed" };
  }
}

/**
 * An extension page's rule when the native language changes (D3): it reloads, so it reads the
 * interface language again, fills its copy in it and builds its engine for the new native language,
 * as it does when it opens. `language` is the interface language the page showed in — the native
 * language it was opened under (M2): a page that already shows the one announced opened after the
 * change, its own preset's included, and has nothing to reload.
 */
export function reloadOnNativeLanguageChange(
  language: InterfaceLanguage | Promise<InterfaceLanguage>,
  watch: (onChanged: (keys: string[], reason?: StoreChangeReason) => void) => unknown = watchStore,
  reload: () => void = () => location.reload(),
): void {
  watch((_keys, reason) => {
    if (reason?.type !== "native-language") return;
    void Promise.resolve(language).then((shown) => {
      if (shown !== reason.native) reload();
    });
  });
}
