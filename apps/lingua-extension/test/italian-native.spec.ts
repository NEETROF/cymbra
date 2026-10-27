import { describe, expect, it, vi } from "vitest";
import { nativeLanguageName } from "@/analyzer/language-labels.ts";
import { SHIPPED_PAIRS, shippedNatives } from "@/analyzer/pairs.ts";
import type { NativeLanguage } from "@/analyzer/types.ts";
import {
  INTERFACE_LANGUAGE_KEY,
  INTERFACE_LANGUAGES,
  interfaceLanguage,
  isInterfaceLanguage,
  isNativeLanguage,
  NATIVE_LANGUAGES,
} from "@/i18n/language.ts";
import { encodeAppOps, parseAppOps } from "@/state/app-relay.ts";
import { PRIVACY_POLICY_URLS } from "@/translate/online.ts";
import { LEAVING_PAGES } from "@/usage/counts.ts";
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

// Italian is a native language the extension knew before it was offered
// (add-lingua-italian-native-language): read from a stored profile, named « Italiano » in every
// interface language, kept as a day's native language — and, while no shipped pair was glossed in
// Italian, nothing a reader saw moved. enable-lingua-italian-speakers lists en-it and shows the Italian
// interface (D1, D3): the « not yet » cases below keep the list before it as an explicit argument.

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

/** The shipped list, which glosses English in Italian since enable-lingua-italian-speakers. */
const WITH_EN_IT = SHIPPED_PAIRS;
/** The list before it: no pair glossed in Italian. */
const BEFORE_EN_IT = SHIPPED_PAIRS.filter((pair) => pair !== "en-it");

describe("the native list and the interface list (D2, D3)", () => {
  it("knows four native languages, the interface speaking all four (enable-lingua-italian-speakers D1)", () => {
    expect(NATIVE_LANGUAGES).toEqual(["fr", "en", "es", "it"]);
    expect(INTERFACE_LANGUAGES).toEqual(["fr", "en", "es", "it"]);
    expect(isNativeLanguage("it")).toBe(true);
    expect(isInterfaceLanguage("it")).toBe(true);
    for (const unknown of ["de", "pt", "IT", "", null, undefined, 3]) {
      expect(isNativeLanguage(unknown)).toBe(false);
      expect(isInterfaceLanguage(unknown)).toBe(false);
    }
  });

  it("reads a stored Italian interface as Italian, now it is shown (add-lingua-italian-interface D6)", async () => {
    expect(await interfaceLanguage(fakeArea({ [INTERFACE_LANGUAGE_KEY]: "it" }))).toBe("it");
    // en-it is the one shipped pair glossed in Italian: a profile naming Italian is read as Italian;
    // with the list before it, as French.
    expect(SHIPPED_PAIRS.filter((pair) => pair.endsWith("-it"))).toEqual(["en-it"]);
    expect(nativeLanguageOf(backupOf("Italian"))).toBe("it");
    expect(nativeLanguageOf(backupOf("Italian"), BEFORE_EN_IT)).toBe("fr");
  });
});

describe("The Italian interface, shown (add-lingua-italian-interface D6, enable-lingua-italian-speakers D1)", () => {
  it("lists every native language of the shipped pairs among the interface languages", () => {
    // A build listing a pair glossed in Italian without `it` in INTERFACE_LANGUAGES would show French:
    // enable-lingua-italian-speakers lists both.
    for (const native of shippedNatives()) expect(INTERFACE_LANGUAGES, native).toContain(native);
    expect(shippedNatives()).toContain("it");
  });

  it("sends the Italian links to the Italian pages (enable-lingua-italian-speakers D2)", () => {
    expect(PRIVACY_POLICY_URLS.it).toBe("https://cymbra.app/it/privacy/");
    expect(LEAVING_PAGES.it).toBe("https://cymbra.app/it/lingua/arrivederci/");
  });
});

describe("en-it listed", () => {
  it("offers Italian, and presets an Italian browser to Italian (enable-lingua-italian-speakers D3)", () => {
    expect(offeredNatives()).toContain("it");
    expect(presetNative("it-IT")).toBe("it");
    expect(presetNative("it")).toBe("it");
    expect(presetNative("it-CH")).toBe("it");
  });
});

describe("No pair glossed in Italian (the list before enable-lingua-italian-speakers)", () => {
  it("offers French, English and Spanish, and presets an Italian browser to English", () => {
    expect(offeredNatives(BEFORE_EN_IT)).not.toContain("it");
    expect(presetNative("it-IT", BEFORE_EN_IT)).toBe("en");
    expect(presetNative("it", BEFORE_EN_IT)).toBe("en");
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
      pairs: BEFORE_EN_IT,
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
    const owned = ownerArea(area, () => {}, fakeArea(), BEFORE_EN_IT);
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

  it("mirrors an Italian native as Italian, read as Italian now it is shown (enable-lingua-italian-speakers D1)", async () => {
    const prefs = fakeArea();
    await rememberInterfaceLanguage(fakeArea({ [ROOT_KEY]: { v: 2, backup: backupOf("Italian") } }), prefs, WITH_EN_IT);
    expect(prefs.store[INTERFACE_LANGUAGE_KEY]).toBe("it");
    expect(await interfaceLanguage(prefs)).toBe("it");

    const mirroredPrefs = fakeArea();
    const owned = ownerArea(fakeArea(), () => {}, mirroredPrefs, WITH_EN_IT);
    await owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("Italian") } });
    await owned.mirrored();
    expect(mirroredPrefs.store[INTERFACE_LANGUAGE_KEY]).toBe("it");
  });
});

describe("A backup naming an Italian native", () => {
  it("restores, is served in French while no pair is glossed in Italian, and keeps Italian", async () => {
    const italian = backupOf("Italian", ["English"]);
    const area = fakeArea({ [ROOT_KEY]: { v: 2, backup: backupOf("French") } });
    const prefs = fakeArea();
    const announced: { keys: string[]; reason?: StoreChangeReason }[] = [];
    const owned = ownerArea(area, (keys, reason) => announced.push({ keys, reason }), prefs, BEFORE_EN_IT);

    await owned.set({ [ROOT_KEY]: { v: 2, backup: italian } }, RESTORE_FROM_FILE);
    await owned.mirrored();

    expect(area.store[ROOT_KEY]).toEqual({ v: 2, backup: italian });
    expect(announced).toEqual([{ keys: [ROOT_KEY] }]);
    expect(prefs.store[INTERFACE_LANGUAGE_KEY]).toBe("fr");
    expect(nativeLanguageOf(italian, BEFORE_EN_IT)).toBe("fr");
    expect(nativeLanguageOf(italian, WITH_EN_IT)).toBe("it");
  });

  it("restores, and is served in Italian once en-it is listed (enable-lingua-italian-speakers)", async () => {
    const italian = backupOf("Italian", ["English"]);
    const area = fakeArea({ [ROOT_KEY]: { v: 2, backup: backupOf("French") } });
    const prefs = fakeArea();
    const owned = ownerArea(area, () => {}, prefs);

    await owned.set({ [ROOT_KEY]: { v: 2, backup: italian } }, RESTORE_FROM_FILE);
    await owned.mirrored();

    expect(area.store[ROOT_KEY]).toEqual({ v: 2, backup: italian });
    expect(prefs.store[INTERFACE_LANGUAGE_KEY]).toBe("it");
    expect(await interfaceLanguage(prefs)).toBe("it");
  });
});

describe("The own name (D3)", () => {
  it("is « Italiano » in every interface language, Italian's own among them", () => {
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
