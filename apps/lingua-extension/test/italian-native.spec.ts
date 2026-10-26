import { describe, expect, it, vi } from "vitest";
import { nativeLanguageName } from "@/analyzer/language-labels.ts";
import { SHIPPED_PAIRS } from "@/analyzer/pairs.ts";
import type { NativeLanguage } from "@/analyzer/types.ts";
import {
  INTERFACE_LANGUAGE_KEY,
  INTERFACE_LANGUAGES,
  interfaceOf,
  isInterfaceLanguage,
  isNativeLanguage,
  NATIVE_LANGUAGES,
} from "@/i18n/language.ts";
import { encodeAppOps, parseAppOps } from "@/state/app-relay.ts";
import {
  changeNativeLanguage,
  NATIVE_LANGUAGE_MESSAGE,
  type NativeLanguageHost,
  offeredNatives,
  presetNative,
} from "@/state/native-language.ts";
import { nativeLanguageOf } from "@/state/profile.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";
import {
  LAST_NATIVE_KEY,
  ownerArea,
  RESTORE_FROM_FILE,
  rememberInterfaceLanguage,
  type StoreChangeReason,
  StaleNativeLanguageError,
  watchStore,
} from "@/state/store.ts";

// Italian is a native language the extension knows before it is offered
// (add-lingua-italian-native-language): read from a stored profile, named « Italiano » in every
// interface language, kept as a day's native language — while no shipped pair is glossed in Italian,
// nothing a reader sees moves.

function fakeArea(seed: Record<string, unknown> = {}): AsyncStorageArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
  return {
    store,
    async get(keys) {
      const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const k of list) if (k in store) out[k] = store[k];
      return out;
    },
    async set(items) {
      Object.assign(store, items);
    },
  };
}

/** A backup as the engine writes the profile: under its enums' names (lingua-core's decks/backup.rs). */
const backupOf = (native: string, studied: string[] = ["English"]) =>
  JSON.stringify({ schema_version: 2, profile: { native_language: native, studied_languages: studied } });

/** A pair glossed in Italian, which no shipped list holds yet: change 36's en-it. */
const WITH_EN_IT = [...SHIPPED_PAIRS, "en-it"];

describe("the native list and the interface list (D2, D3)", () => {
  it("knows four native languages and keeps the interface to three", () => {
    expect(NATIVE_LANGUAGES).toEqual(["fr", "en", "es", "it"]);
    expect(INTERFACE_LANGUAGES).toEqual(["fr", "en", "es"]);
    expect(isNativeLanguage("it")).toBe(true);
    expect(isInterfaceLanguage("it")).toBe(false);
    for (const unknown of ["de", "pt", "IT", "", null, undefined, 3]) expect(isNativeLanguage(unknown)).toBe(false);
  });

  it("serves Italian in English, the others in themselves — unreachable while no pair is glossed in Italian", () => {
    expect(interfaceOf("it")).toBe("en");
    for (const native of INTERFACE_LANGUAGES) expect(interfaceOf(native)).toBe(native);
    // No shipped pair is glossed in Italian: a profile naming it is read as French before the mapping.
    expect(SHIPPED_PAIRS.filter((pair) => pair.endsWith("-it"))).toEqual([]);
    expect(nativeLanguageOf(backupOf("Italian"))).toBe("fr");
  });
});

describe("No pair glossed in Italian", () => {
  it("offers French, English and Spanish, and presets an Italian browser to English", () => {
    expect(offeredNatives()).not.toContain("it");
    expect(offeredNatives()).toEqual(offeredNatives().filter((native) => native !== "it"));
    expect(presetNative("it-IT")).toBe("en");
    expect(presetNative("it")).toBe("en");
    // Once a pair is glossed in Italian, it is offered, and an Italian browser is preset to it.
    expect(offeredNatives(WITH_EN_IT)).toContain("it");
    expect(presetNative("it-CH", WITH_EN_IT)).toBe("it");
  });

  it("refuses the native-language message for Italian as having no pair, writing nothing", async () => {
    const backup = backupOf("French");
    const area = fakeArea({ [ROOT_KEY]: { v: 2, backup } });
    const prefs = fakeArea();
    const announced: unknown[] = [];
    const deps = {
      store: ownerArea(area, (keys, reason) => announced.push({ keys, reason }), prefs),
      preferences: prefs,
      ensureBackup: vi.fn(async () => {}),
      reprofile: vi.fn(async (b: string) => b),
      exclusive: <T>(task: () => Promise<T>): Promise<T> => task(),
      rehydrate: vi.fn(),
    } satisfies NativeLanguageHost;

    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "it" })).toEqual({
      ok: false,
      error: "no-pair",
    });
    expect(deps.reprofile).not.toHaveBeenCalled();
    expect(deps.rehydrate).not.toHaveBeenCalled();
    expect(announced).toEqual([]);
    expect(prefs.store).toEqual({});
    expect(area.store[ROOT_KEY]).toEqual({ v: 2, backup });
  });
});

describe("the store's record and reason (D3)", () => {
  it("hands a watcher a change of native language to Italian as a reason it knows", () => {
    const listeners: ((changes: Record<string, { newValue?: unknown }>, area: string) => void)[] = [];
    vi.stubGlobal("chrome", {
      storage: {
        onChanged: { addListener: (fn: (typeof listeners)[number]) => listeners.push(fn), removeListener() {} },
      },
    });
    const seen: (StoreChangeReason | undefined)[] = [];
    const stop = watchStore((_keys, reason) => seen.push(reason));
    listeners[0](
      {
        "cymbra-lingua-store-changed": {
          newValue: { rev: 1, keys: [ROOT_KEY], reason: { type: "native-language", native: "it" } },
        },
      },
      "local",
    );
    expect(seen).toEqual([{ type: "native-language", native: "it" }]);
    stop();
    vi.unstubAllGlobals();
  });

  it("reads a recorded Italian as French while no pair is glossed in it", async () => {
    const area = fakeArea({ [LAST_NATIVE_KEY]: "it" });
    const owned = ownerArea(area, () => {}, fakeArea());
    // The record reads French: a backup naming French is written, one naming English is refused.
    await owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("French") } });
    await expect(owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("English", ["Spanish"]) } })).rejects.toBeInstanceOf(
      StaleNativeLanguageError,
    );
  });

  it("reads a recorded Italian as Italian once a pair is glossed in it", async () => {
    const area = fakeArea({ [LAST_NATIVE_KEY]: "it" });
    const owned = ownerArea(area, () => {}, fakeArea(), WITH_EN_IT);
    await owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("Italian") } });
    await expect(owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("French") } })).rejects.toBeInstanceOf(
      StaleNativeLanguageError,
    );
  });

  it("mirrors an Italian native as English, the interface it is served in (D2)", async () => {
    const prefs = fakeArea();
    await rememberInterfaceLanguage(fakeArea({ [ROOT_KEY]: { v: 2, backup: backupOf("Italian") } }), prefs, WITH_EN_IT);
    expect(prefs.store[INTERFACE_LANGUAGE_KEY]).toBe("en");

    const mirroredPrefs = fakeArea();
    const owned = ownerArea(fakeArea(), () => {}, mirroredPrefs, WITH_EN_IT);
    await owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("Italian") } });
    await owned.mirrored();
    expect(mirroredPrefs.store[INTERFACE_LANGUAGE_KEY]).toBe("en");
  });
});

describe("A backup naming an Italian native", () => {
  it("restores, is served in French while no pair is glossed in Italian, and keeps Italian", async () => {
    const italian = backupOf("Italian", ["English"]);
    const area = fakeArea({ [ROOT_KEY]: { v: 2, backup: backupOf("French") } });
    const prefs = fakeArea();
    const announced: { keys: string[]; reason?: StoreChangeReason }[] = [];
    const owned = ownerArea(area, (keys, reason) => announced.push({ keys, reason }), prefs);

    await owned.set({ [ROOT_KEY]: { v: 2, backup: italian } }, RESTORE_FROM_FILE);
    await owned.mirrored();

    expect(area.store[ROOT_KEY]).toEqual({ v: 2, backup: italian });
    expect(announced).toEqual([{ keys: [ROOT_KEY] }]);
    expect(prefs.store[INTERFACE_LANGUAGE_KEY]).toBe("fr");
    expect(nativeLanguageOf(italian)).toBe("fr");
    expect(nativeLanguageOf(italian, WITH_EN_IT)).toBe("it");
  });
});

describe("The own name (D3)", () => {
  it("is « Italiano » in the French, English and Spanish interfaces", () => {
    for (const language of INTERFACE_LANGUAGES) expect(nativeLanguageName(language, "it")).toBe("Italiano");
  });
});

describe("An Italian label from the app (D3)", () => {
  it("keeps a day's counts under the native language it", () => {
    const daily = {
      a: {
        "20000": {
          en: { reviews: 2, wordsLearned: 1, native: "it" as NativeLanguage },
          es: { reviews: 1, wordsLearned: 0, native: "de" },
        },
      },
    };
    const raw = JSON.stringify({ ...JSON.parse(encodeAppOps({ since: 0, cardOps: [], statusOps: [] })), daily });
    expect(parseAppOps(raw)?.daily).toEqual({ a: { "20000": { en: { reviews: 2, wordsLearned: 1, native: "it" } } } });
  });
});
