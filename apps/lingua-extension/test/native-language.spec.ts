import { afterEach, describe, expect, it, vi } from "vitest";
import { INTERFACE_LANGUAGE_KEY } from "@/i18n/language.ts";
import {
  changeNativeLanguage,
  chooseNativeLanguage,
  isNativeLanguageMessage,
  markNativeLanguageChosen,
  NATIVE_CHOSEN_KEY,
  NATIVE_LANGUAGE_MESSAGE,
  nativeChoiceOffered,
  nativeLanguageChosen,
  type NativeLanguageHost,
  offeredNatives,
  presetNative,
  reloadOnNativeLanguageChange,
  studiedForNative,
} from "@/state/native-language.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";
import { ownerArea, type StoreChangeReason, watchStore } from "@/state/store.ts";

// The reader chooses their native language (add-lingua-native-language-choice): the rule for the
// studied languages, the preset, and what the background does with the message (D1, D2, D4).

/** Today's package: French-native pairs alone (packs.json). */
const TODAY = ["en-fr", "es-fr"];
/** es-en shipping beside them (change 34): the first second native language. */
const MIXED = ["en-fr", "es-fr", "es-en"];

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

/** A backup as the engine writes the profile: under its enums' names (state/profile.ts). */
function backupOf(native: "French" | "English" | "Spanish", studied: ("English" | "Spanish")[]): string {
  return JSON.stringify({ schema_version: 2, profile: { native_language: native, studied_languages: studied } });
}

/** What `reprofileBackup` writes, as a fake: the profile replaced, the rest kept. */
const NAMES = { fr: "French", en: "English", es: "Spanish" } as const;
async function fakeReprofile(backup: string, native: "fr" | "en" | "es", studied: ("en" | "es")[]): Promise<string> {
  const parsed = JSON.parse(backup) as Record<string, unknown>;
  return JSON.stringify({
    ...parsed,
    profile: { native_language: NAMES[native], studied_languages: studied.map((language) => NAMES[language]) },
  });
}

function host(store: Record<string, unknown> = {}, preferences: Record<string, unknown> = {}) {
  const announced: { keys: string[]; reason?: StoreChangeReason; key: unknown; chosen: unknown }[] = [];
  const prefs = fakeArea(preferences);
  const owned = ownerArea(
    fakeArea(store),
    (keys, reason) =>
      announced.push({
        keys,
        reason,
        key: prefs.store[INTERFACE_LANGUAGE_KEY],
        chosen: prefs.store[NATIVE_CHOSEN_KEY],
      }),
    prefs,
    MIXED,
  );
  const deps = {
    store: owned,
    preferences: prefs,
    ensureBackup: vi.fn(async () => {
      await owned.set({ [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English"]) } });
    }),
    reprofile: vi.fn(fakeReprofile),
    dropEngines: vi.fn(),
    pairs: MIXED,
  } satisfies NativeLanguageHost;
  return { deps, announced, prefs, owned };
}

const stored = (deps: { store: AsyncStorageArea }) =>
  deps.store.get(ROOT_KEY).then((got) => JSON.parse((got[ROOT_KEY] as { backup: string }).backup) as object);

describe("the choice exists from two native languages (D1)", () => {
  it("Every reader today: one native language ships, no choice is offered", () => {
    expect(nativeChoiceOffered()).toBe(false);
    expect(nativeChoiceOffered(TODAY)).toBe(false);
    expect(offeredNatives(TODAY)).toEqual(["fr"]);
  });

  it("es-en shipping offers French and English, in listed order", () => {
    expect(nativeChoiceOffered(MIXED)).toBe(true);
    expect(offeredNatives(MIXED)).toEqual(["fr", "en"]);
    expect(offeredNatives(["es-en", "en-fr", "en-es"])).toEqual(["en", "fr", "es"]);
  });
});

describe("a new install's preset (D4, M3, M13)", () => {
  it("A new install when two native languages ship: an English browser is preset to English", () => {
    expect(presetNative("en-US", MIXED)).toBe("en");
    expect(presetNative("en", MIXED)).toBe("en");
  });

  it("A browser in another language: German is preset to English when English ships", () => {
    expect(presetNative("de-DE", MIXED)).toBe("en");
    expect(presetNative(undefined, MIXED)).toBe("en");
    expect(presetNative("", MIXED)).toBe("en");
  });

  it("a French browser is preset to French; Spanish only once a pair is glossed in it", () => {
    expect(presetNative("fr-CA", MIXED)).toBe("fr");
    expect(presetNative("es-MX", MIXED)).toBe("en");
    expect(presetNative("es_ES", [...MIXED, "en-es"])).toBe("es");
  });

  it("French when no pair is glossed in English", () => {
    expect(presetNative("de", TODAY)).toBe("fr");
    expect(presetNative("es", TODAY)).toBe("fr");
  });
});

describe("the studied languages of a reader who chooses (D2)", () => {
  it("Changing the native language: a French reader of English and Spanish who chooses English studies Spanish", () => {
    expect(studiedForNative(["en", "es"], "en", MIXED)).toEqual(["es"]);
  });

  it("A native language that was the only studied one: the native's first pair's language stands in", () => {
    expect(studiedForNative(["en"], "en", MIXED)).toEqual(["es"]);
  });

  it("drops a language no shipped pair glosses in the native chosen, keeping the reader's order", () => {
    expect(studiedForNative(["es", "en"], "fr", MIXED)).toEqual(["es", "en"]);
    expect(studiedForNative(["en", "es"], "es", [...MIXED, "en-es"])).toEqual(["en"]);
    expect(studiedForNative(["es"], "fr", ["en-fr"])).toEqual(["en"]);
  });

  it("A native language with no pair: refused", () => {
    expect(studiedForNative(["en"], "es", MIXED)).toBeNull();
    expect(studiedForNative(["en"], "de", MIXED)).toBeNull();
  });
});

describe("the background's change (D2)", () => {
  it("Changing the native language: the profile rewritten, the key written before the announcement, the engines dropped", async () => {
    const { deps, announced, prefs } = host({
      [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English", "Spanish"]) },
    });

    const reply = await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en" });

    expect(reply).toEqual({ ok: true, changed: true });
    expect(deps.reprofile).toHaveBeenCalledWith(backupOf("French", ["English", "Spanish"]), "en", ["es"]);
    expect(await stored(deps)).toEqual({
      schema_version: 2,
      profile: { native_language: "English", studied_languages: ["Spanish"] },
    });
    // One announcement, with its reason; the interface language key and the marker already written.
    expect(announced).toEqual([
      { keys: [ROOT_KEY], reason: { type: "native-language", native: "en" }, key: "en", chosen: true },
    ]);
    expect(prefs.store[INTERFACE_LANGUAGE_KEY]).toBe("en");
    expect(deps.dropEngines).toHaveBeenCalledOnce();
  });

  it("A native language with no pair: refused, nothing written, nothing dropped", async () => {
    const backup = backupOf("French", ["English"]);
    const { deps, announced, prefs } = host({ [ROOT_KEY]: { v: 2, backup } });

    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "es" })).toEqual({
      ok: false,
      error: "no-pair",
    });
    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "de" })).toEqual({
      ok: false,
      error: "no-pair",
    });

    expect(deps.reprofile).not.toHaveBeenCalled();
    expect(deps.dropEngines).not.toHaveBeenCalled();
    expect(announced).toEqual([]);
    expect(prefs.store).toEqual({});
    expect(await stored(deps)).toEqual(JSON.parse(backup));
  });

  it("choosing the native language the reader has marks the choice and changes nothing else", async () => {
    const { deps, announced, prefs } = host({ [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English"]) } });

    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "fr" })).toEqual({
      ok: true,
      changed: false,
    });

    expect(prefs.store[NATIVE_CHOSEN_KEY]).toBe(true);
    expect(deps.reprofile).not.toHaveBeenCalled();
    expect(deps.dropEngines).not.toHaveBeenCalled();
    expect(announced).toEqual([]);
  });

  it("a preset is applied once: never over a choice already made (An installed extension is not asked)", async () => {
    const { deps, announced } = host(
      { [ROOT_KEY]: { v: 2, backup: backupOf("French", ["English"]) } },
      { [NATIVE_CHOSEN_KEY]: true },
    );

    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en", preset: true })).toEqual({
      ok: true,
      changed: false,
    });

    expect(deps.reprofile).not.toHaveBeenCalled();
    expect(announced).toEqual([]);
    expect(await stored(deps)).toEqual(JSON.parse(backupOf("French", ["English"])));
  });

  it("A new install when two native languages ship: the preset writes the fresh backup first, then English", async () => {
    const { deps, announced, prefs } = host();

    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en", preset: true })).toEqual({
      ok: true,
      changed: true,
    });

    expect(deps.ensureBackup).toHaveBeenCalledOnce();
    expect(await stored(deps)).toEqual({
      schema_version: 2,
      profile: { native_language: "English", studied_languages: ["Spanish"] },
    });
    expect(prefs.store).toEqual({ [INTERFACE_LANGUAGE_KEY]: "en", [NATIVE_CHOSEN_KEY]: true });
    expect(announced.at(-1)?.reason).toEqual({ type: "native-language", native: "en" });
  });

  it("a backup that cannot be rewritten is kept, and the choice is not marked", async () => {
    vi.spyOn(console, "warn").mockImplementation(() => {});
    const backup = backupOf("French", ["English"]);
    const { deps, announced, prefs } = host({ [ROOT_KEY]: { v: 2, backup } });
    deps.reprofile.mockRejectedValueOnce(new Error("malformed backup"));

    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en" })).toEqual({
      ok: false,
      error: "failed",
    });

    expect(await stored(deps)).toEqual(JSON.parse(backup));
    expect(prefs.store[NATIVE_CHOSEN_KEY]).toBeUndefined();
    expect(announced).toEqual([]);
    expect(deps.dropEngines).not.toHaveBeenCalled();
    vi.restoreAllMocks();
  });

  it("a store that holds no backup even after hydration fails the change", async () => {
    const { deps } = host();
    deps.ensureBackup.mockResolvedValueOnce(undefined);
    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en" })).toEqual({
      ok: false,
      error: "failed",
    });
  });

  it("recognises its message", () => {
    expect(isNativeLanguageMessage({ type: "lingua-native-language", native: "en" })).toBe(true);
    expect(isNativeLanguageMessage({ type: "lingua-native-language" })).toBe(false);
    expect(isNativeLanguageMessage({ type: "store:get", native: "en" })).toBe(false);
    expect(isNativeLanguageMessage(null)).toBe(false);
  });
});

describe("a surface's side", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("sends the choice, a preset saying so, and reads a failure when the background is away", async () => {
    const send = vi.fn(async () => ({ ok: true, changed: true }));
    expect(await chooseNativeLanguage("en", false, send)).toEqual({ ok: true, changed: true });
    expect(await chooseNativeLanguage("en", true, send)).toEqual({ ok: true, changed: true });
    expect(send.mock.calls).toEqual([
      [{ type: "lingua-native-language", native: "en" }],
      [{ type: "lingua-native-language", native: "en", preset: true }],
    ]);
    expect(await chooseNativeLanguage("en", false, async () => undefined)).toEqual({ ok: false, error: "failed" });
    expect(
      await chooseNativeLanguage("en", false, async () => {
        throw new Error("Receiving end does not exist");
      }),
    ).toEqual({ ok: false, error: "failed" });
  });

  it("reads and sets the marker", async () => {
    const prefs = fakeArea();
    expect(await nativeLanguageChosen(prefs)).toBe(false);
    await markNativeLanguageChosen(prefs);
    expect(await nativeLanguageChosen(prefs)).toBe(true);
    expect(prefs.store).toEqual({ "cymbra-lingua-native-chosen": true });
  });

  it("a page reloads on the announced change of native language, unless it already shows it", async () => {
    let listener!: (keys: string[], reason?: StoreChangeReason) => void;
    const watch = (fn: typeof listener) => (listener = fn);
    const reload = vi.fn();
    reloadOnNativeLanguageChange("fr", watch, reload);

    listener([ROOT_KEY]); // a sync, a status: no reason
    listener([ROOT_KEY], { type: "native-language", native: "fr" }); // the page shows it already
    await Promise.resolve();
    expect(reload).not.toHaveBeenCalled();

    listener([ROOT_KEY], { type: "native-language", native: "en" });
    await Promise.resolve();
    expect(reload).toHaveBeenCalledOnce();
  });

  it("a page whose language is still being read compares once it is known", async () => {
    let listener!: (keys: string[], reason?: StoreChangeReason) => void;
    const reload = vi.fn();
    reloadOnNativeLanguageChange(Promise.resolve("en"), (fn) => (listener = fn), reload);
    listener([ROOT_KEY], { type: "native-language", native: "en" });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(reload).not.toHaveBeenCalled();
    listener([ROOT_KEY], { type: "native-language", native: "fr" });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(reload).toHaveBeenCalledOnce();
  });

  it("the store's marker hands each watcher the reason, and a known one only", () => {
    const listeners: ((changes: Record<string, { newValue?: unknown }>, area: string) => void)[] = [];
    const removed: unknown[] = [];
    vi.stubGlobal("chrome", {
      storage: {
        onChanged: {
          addListener: (fn: (typeof listeners)[number]) => listeners.push(fn),
          removeListener: (fn: unknown) => removed.push(fn),
        },
      },
    });
    const seen: [string[], StoreChangeReason | undefined][] = [];
    const stop = watchStore((keys, reason) => seen.push([keys, reason]));
    const fire = (value: unknown) => listeners[0]({ "cymbra-lingua-store-changed": { newValue: value } }, "local");

    fire({ rev: 1, keys: [ROOT_KEY] });
    fire({ rev: 2, keys: [ROOT_KEY], reason: { type: "native-language", native: "en" } });
    fire({ rev: 3, keys: [ROOT_KEY], reason: { type: "native-language", native: "de" } });
    fire({ rev: 4, keys: [ROOT_KEY], reason: { type: "something-else" } });
    listeners[0]({ "cymbra-lingua-store-changed": { newValue: { rev: 5, keys: [] } } }, "session");

    expect(seen).toEqual([
      [[ROOT_KEY], undefined],
      [[ROOT_KEY], { type: "native-language", native: "en" }],
      [[ROOT_KEY], undefined],
      [[ROOT_KEY], undefined],
    ]);
    stop();
    expect(removed).toEqual([listeners[0]]);
  });
});
