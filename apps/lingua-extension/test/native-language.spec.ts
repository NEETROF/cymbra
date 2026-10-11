import { afterEach, describe, expect, it, vi } from "vitest";
import { acceptedLanguages } from "@/analyzer/pairs.ts";
import type { NativeLanguage, StudiedLanguage } from "@/analyzer/types.ts";
import { INTERFACE_LANGUAGE_KEY } from "@/i18n/language.ts";
import {
  changeNativeLanguage,
  chooseNativeLanguage,
  holdsReaderData,
  isNativeLanguageMessage,
  markNativeLanguageChosen,
  NATIVE_CHOSEN_KEY,
  NATIVE_LANGUAGE_MESSAGE,
  NATIVE_PRESET_KEY,
  nativeChoiceOffered,
  nativeLanguageChosen,
  type NativeLanguageHost,
  offeredNatives,
  onExtensionInstalled,
  presetNative,
  reloadOnNativeLanguageChange,
  studiedForNative,
} from "@/state/native-language.ts";
import { type AsyncStorageArea, ROOT_KEY } from "@/state/storage.ts";
import { LAST_NATIVE_KEY, ownerArea, type StoreChangeReason, watchStore } from "@/state/store.ts";
import { SyncScheduler } from "@/sync/scheduler.ts";
import { SyncEngine, type SyncClients } from "@/sync/sync.ts";
import { makeFakePort } from "./helpers.ts";

// The reader chooses their native language (add-lingua-native-language-choice): the rule for the
// studied languages, the preset, and what the background does with the message (D1, D2, D4).

/** The package before change 34: French-native pairs alone. */
const TODAY = ["en-fr", "es-fr"];
/** es-en shipping beside them, the first second native language: packs.json in change 34. */
const MIXED = ["en-fr", "es-fr", "es-en"];
/** en-es shipping beside those, the third native language: packs.json since change 35. */
const ALL = [...MIXED, "en-es"];

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
    exclusive: <T>(task: () => Promise<T>): Promise<T> => task(),
    rehydrate: vi.fn(),
    pairs: MIXED,
  } satisfies NativeLanguageHost;
  return { deps, announced, prefs, owned };
}

const stored = (deps: { store: AsyncStorageArea }) =>
  deps.store.get(ROOT_KEY).then((got) => JSON.parse((got[ROOT_KEY] as { backup: string }).backup) as object);

describe("the choice exists from two native languages (D1)", () => {
  it("Every reader today: one native language ships, no choice is offered", () => {
    expect(nativeChoiceOffered(TODAY)).toBe(false);
    expect(offeredNatives(TODAY)).toEqual(["fr"]);
  });

  it("es-en shipping offers French and English, in listed order", () => {
    expect(nativeChoiceOffered(MIXED)).toBe(true);
    expect(offeredNatives(MIXED)).toEqual(["fr", "en"]);
    expect(offeredNatives(["es-en", "en-fr", "en-es"])).toEqual(["en", "fr", "es"]);
  });

  it("en-es shipping too offers French, English and Spanish, in listed order (enable-lingua-spanish-speakers)", () => {
    expect(nativeChoiceOffered(ALL)).toBe(true);
    expect(offeredNatives(ALL)).toEqual(["fr", "en", "es"]);
    // The bundle's list since change 35.
    expect(nativeChoiceOffered()).toBe(true);
    expect(offeredNatives()).toEqual(["fr", "en", "es"]);
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

  it("on change 34's list: English for an English, a German or a Spanish browser, French for a French one", () => {
    expect(presetNative("en-US", MIXED)).toBe("en");
    expect(presetNative("de-DE", MIXED)).toBe("en");
    expect(presetNative("es-ES", MIXED)).toBe("en");
    expect(presetNative("fr-FR", MIXED)).toBe("fr");
  });

  it("on the bundle's list since change 35: Spanish for a Spanish browser, English still for a German one (M13)", () => {
    expect(presetNative("es-ES")).toBe("es");
    expect(presetNative("es-MX")).toBe("es");
    expect(presetNative("en-US")).toBe("en");
    expect(presetNative("de-DE")).toBe("en");
    expect(presetNative("fr-FR")).toBe("fr");
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
  it("Changing the native language: the profile rewritten, the key written before the announcement, the reading engine restored again", async () => {
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
    // One announcement, with its reason; the interface language key already written, the choice
    // marked once the backup is.
    expect(announced).toEqual([
      { keys: [ROOT_KEY], reason: { type: "native-language", native: "en" }, key: "en", chosen: undefined },
    ]);
    expect(prefs.store[INTERFACE_LANGUAGE_KEY]).toBe("en");
    expect(prefs.store[NATIVE_CHOSEN_KEY]).toBe(true);
    expect(deps.rehydrate).toHaveBeenCalledOnce();
  });

  it("A native language with no pair: refused, nothing written, no engine restored", async () => {
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
    expect(deps.rehydrate).not.toHaveBeenCalled();
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
    expect(deps.rehydrate).not.toHaveBeenCalled();
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
    // A preset is no answer: the device is known new, the choice still to be made.
    expect(prefs.store).toEqual({ [INTERFACE_LANGUAGE_KEY]: "en", [NATIVE_PRESET_KEY]: true });
    expect(announced.at(-1)?.reason).toEqual({ type: "native-language", native: "en" });
  });

  it("a preset is applied again until the reader answers, and never once they have", async () => {
    const { deps, announced, prefs } = host();
    const preset = { type: NATIVE_LANGUAGE_MESSAGE, native: "en", preset: true } as const;

    await changeNativeLanguage(deps, preset);
    // The reader read pages and clicked words, and closed the popup without answering: their own data
    // since the preset never passes for an installed extension's.
    const stored = await deps.store.get(ROOT_KEY);
    const backup = JSON.parse((stored[ROOT_KEY] as { backup: string }).backup) as object;
    await deps.store.set({
      [ROOT_KEY]: {
        v: 2,
        backup: JSON.stringify({ ...backup, knowledge: { statuses: { Spanish: { faro: "Learning" } } } }),
      },
    });
    expect(await changeNativeLanguage(deps, preset)).toEqual({ ok: true, changed: false });
    expect(prefs.store[NATIVE_CHOSEN_KEY]).toBeUndefined();

    // The answer — the preset confirmed — marks it.
    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en" })).toEqual({
      ok: true,
      changed: false,
    });
    expect(prefs.store[NATIVE_CHOSEN_KEY]).toBe(true);
    announced.length = 0;
    expect(await changeNativeLanguage(deps, { ...preset, native: "fr" })).toEqual({ ok: true, changed: false });
    expect(announced).toEqual([]);
  });

  it.each([
    ["statuses", { knowledge: { statuses: { English: { lighthouse: "Learning" } } } }],
    ["cards", { deck: { cards: { English: { lighthouse: { lemma: "lighthouse" } } } } }],
    ["a profile of its own", { profile: { native_language: "French", studied_languages: ["English", "Spanish"] } }],
  ])(
    "An installed extension is not asked, its update unreported: a backup holding %s is never preset, and marked",
    async (_what, data) => {
      const backup = JSON.stringify({ schema_version: 2, exposure: { English: { the: 3 } }, ...data });
      const { deps, announced, prefs } = host({ [ROOT_KEY]: { v: 2, backup } });

      expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en", preset: true })).toEqual({
        ok: true,
        changed: false,
      });

      expect(deps.reprofile).not.toHaveBeenCalled();
      expect(announced).toEqual([]);
      expect(await stored(deps)).toEqual(JSON.parse(backup));
      expect(prefs.store).toEqual({ [NATIVE_CHOSEN_KEY]: true });
    },
  );

  it("a page read before the popup is no reader's data: its exposures alone do not stop the preset", async () => {
    const backup = JSON.stringify({
      schema_version: 2,
      knowledge: { statuses: {} },
      exposure: { English: { the: 3 } },
      deck: { cards: {} },
    });
    const { deps } = host({ [ROOT_KEY]: { v: 2, backup } });
    expect(await changeNativeLanguage(deps, { type: NATIVE_LANGUAGE_MESSAGE, native: "en", preset: true })).toEqual({
      ok: true,
      changed: true,
    });
    expect(holdsReaderData("{not json")).toBe(true); // nothing is preset over a backup that does not parse
  });

  it("onInstalled: an update marks the choice as made, an install or the browser's update nothing", async () => {
    for (const reason of ["install", "chrome_update", "shared_module_update"]) {
      const prefs = fakeArea();
      await onExtensionInstalled(reason, prefs);
      expect(prefs.store).toEqual({});
    }
    const prefs = fakeArea();
    await onExtensionInstalled("update", prefs);
    expect(prefs.store).toEqual({ [NATIVE_CHOSEN_KEY]: true });
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
    expect(deps.rehydrate).not.toHaveBeenCalled();
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

describe("the background's change, beside a sync (D2)", () => {
  const CODES: Record<string, string> = { French: "fr", English: "en", Spanish: "es" };
  type Held = { profile: { native_language: string; studied_languages: string[] }; words: string[] };

  /**
   * The sync's port, holding the reader's state as a backup does — its profile, the words it knows —
   * so that what a sync restores, applies and saves can be read back. Applying what was pulled waits
   * for `release`: the window between the sync's restore of the latest backup and its save.
   */
  function syncPort() {
    const { port } = makeFakePort();
    let held: Held = { profile: { native_language: "French", studied_languages: ["English"] }, words: [] };
    let release: () => void = () => {};
    let entered: () => void = () => {};
    const applying = new Promise<void>((resolve) => (entered = resolve));
    port.restore = async (json) => void (held = JSON.parse(json) as Held);
    port.backup = async () => JSON.stringify({ schema_version: 2, ...held });
    port.nativeLanguage = async () => CODES[held.profile.native_language] as NativeLanguage;
    port.studiedLanguages = async () => held.profile.studied_languages.map((name) => CODES[name] as StudiedLanguage);
    port.exportStatusOps = async () => [];
    port.exportCardOps = async () => [];
    port.exportDeclaredLevels = async () => [];
    port.applyStatusChanges = async (changes) => {
      entered();
      await new Promise<void>((resolve) => (release = resolve));
      held.words.push(...changes.map((change) => change.lemma));
      return changes.length;
    };
    port.applyCardOps = async () => 0;
    port.applyDeclaredLevelChanges = async () => 0;
    return { port, applying, release: () => release() };
  }

  /** A server whose pulls hand back one Spanish word each — Spanish is accepted before and after. */
  function server(lemmas: string[]) {
    const pullChanges = vi.fn(async () => ({
      changes: [
        { language: "es", lemma: lemmas.shift(), status: "known", provenance: "manual", updatedAt: 5n, sequence: 1n },
      ],
      cursor: 7n,
      declaredLevels: [],
    }));
    return {
      knownWords: { pushOps: vi.fn(async () => ({ applied: 0n, cursor: 0n })), pullChanges },
      deck: {
        pushCards: vi.fn(async () => ({ applied: 0n, cursor: 0n })),
        pullCards: vi.fn(async () => ({ cards: [], cursor: 9n })),
      },
      stats: { upsertDailyStats: vi.fn(async () => ({ upserted: 0n })) },
      data: { getDataState: vi.fn(async () => ({ erasedAt: 0n })), eraseMyData: vi.fn() },
    } as unknown as SyncClients;
  }

  it("a sync in flight when the change lands neither loses what it pulled nor reverts the choice", async () => {
    const { deps } = host({
      [ROOT_KEY]: {
        v: 2,
        backup: JSON.stringify({
          schema_version: 2,
          profile: { native_language: "French", studied_languages: ["English", "Spanish"] },
          words: ["mine"],
        }),
      },
    });
    const sync = syncPort();
    const clients = server(["pulled", "later"]);
    const engine = new SyncEngine({
      port: sync.port,
      storage: deps.store,
      clients: () => clients,
      deviceId: "d",
      acceptedLanguages: () => acceptedLanguages(sync.port, MIXED),
    });
    const scheduler = new SyncScheduler({
      sync: async () => void (await engine.sync()),
      signedIn: () => true,
      now: () => 0,
      lastSynced: async () => 0,
      onSynced: async () => {},
      onError: (e) => {
        throw e;
      },
    });

    // The sync restored the latest backup and is applying what it pulled when the reader chooses
    // English: had the change gone ahead, the sync would save the French profile back over it, or
    // the change would save over the word the sync pulled.
    const first = scheduler.syncNow();
    await sync.applying;
    const change = changeNativeLanguage(
      { ...deps, exclusive: (task) => scheduler.exclusive(task) },
      { type: NATIVE_LANGUAGE_MESSAGE, native: "en" },
    );
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(deps.reprofile).not.toHaveBeenCalled(); // the change waits for the sync

    sync.release();
    expect(await first).toBeNull();
    expect(await change).toEqual({ ok: true, changed: true });
    // The change rewrote what the sync saved: the reader's word and the one pulled, now English-native.
    expect(await stored(deps)).toEqual({
      schema_version: 2,
      profile: { native_language: "English", studied_languages: ["Spanish"] },
      words: ["mine", "pulled"],
    });

    // The next sync restores the new profile and saves it again with what it pulls: nothing reverts.
    const next = scheduler.syncNow();
    await new Promise((resolve) => setTimeout(resolve, 0));
    sync.release();
    expect(await next).toBeNull();
    expect(await stored(deps)).toEqual({
      schema_version: 2,
      profile: { native_language: "English", studied_languages: ["Spanish"] },
      words: ["mine", "pulled", "later"],
    });
  });
});

describe("an erasure after a change of native language (task 4.5)", () => {
  const CODES: Record<string, string> = { French: "fr", English: "en", Spanish: "es" };
  const FIRST_STUDIED: Record<string, string> = { French: "English", English: "Spanish" };
  type Held = { profile: { native_language: string; studied_languages: string[] }; words: string[] };

  /**
   * The sync's engine as its last run left it: the reader's French backup restored. Its full reset
   * starts a fresh state on its own native language — the one of the backup it last restored.
   */
  function syncPort() {
    const { port } = makeFakePort();
    let held: Held = { profile: { native_language: "French", studied_languages: ["English"] }, words: ["mine"] };
    port.restore = async (json) => void (held = JSON.parse(json) as Held);
    port.backup = async () => JSON.stringify({ schema_version: 2, ...held });
    port.nativeLanguage = async () => CODES[held.profile.native_language] as NativeLanguage;
    port.studiedLanguages = async () => held.profile.studied_languages.map((name) => CODES[name] as StudiedLanguage);
    port.reset = async () => {
      const native = held.profile.native_language;
      held = { profile: { native_language: native, studied_languages: [FIRST_STUDIED[native]] }, words: [] };
    };
    port.exportStatusOps = async () => [];
    port.exportCardOps = async () => [];
    port.exportDeclaredLevels = async () => [];
    port.applyStatusChanges = async () => 0;
    port.applyCardOps = async () => 0;
    port.applyDeclaredLevelChanges = async () => 0;
    return port;
  }

  /** A server that erased the account at 5, and has nothing to pull. */
  function server() {
    return {
      knownWords: {
        pushOps: vi.fn(async () => ({ applied: 0n, cursor: 0n })),
        pullChanges: vi.fn(async () => ({ changes: [], cursor: 7n, declaredLevels: [] })),
      },
      deck: {
        pushCards: vi.fn(async () => ({ applied: 0n, cursor: 0n })),
        pullCards: vi.fn(async () => ({ cards: [], cursor: 9n })),
      },
      stats: { upsertDailyStats: vi.fn(async () => ({ upserted: 0n })) },
      data: {
        getDataState: vi.fn(async () => ({ erasedAt: 5n })),
        eraseMyData: vi.fn(async () => ({ erasedAt: 5n })),
      },
    } as unknown as SyncClients;
  }

  /** What an erasure leaves an English reader: the fresh state of English, studying Spanish. */
  const FRESH_ENGLISH = {
    schema_version: 2,
    profile: { native_language: "English", studied_languages: ["Spanish"] },
    words: [],
  };

  function erasing(store: Record<string, unknown>) {
    const { deps } = host({
      [ROOT_KEY]: {
        v: 2,
        backup: JSON.stringify({
          schema_version: 2,
          profile: { native_language: "French", studied_languages: ["English", "Spanish"] },
          words: ["mine"],
        }),
      },
      ...store,
    });
    const port = syncPort();
    const engine = new SyncEngine({
      port,
      storage: deps.store,
      clients: server,
      deviceId: "d",
      acceptedLanguages: () => acceptedLanguages(port, MIXED),
    });
    const scheduler = new SyncScheduler({
      sync: async () => void (await engine.sync()),
      signedIn: () => true,
      now: () => 0,
      lastSynced: async () => 0,
      onSynced: async () => {},
      onError: (e) => {
        throw e;
      },
    });
    const change = () =>
      changeNativeLanguage(
        { ...deps, exclusive: (task) => scheduler.exclusive(task) },
        { type: NATIVE_LANGUAGE_MESSAGE, native: "en" },
      );
    return { deps, engine, scheduler, change };
  }

  it("« Effacer mes données Lingua » queued behind a change: the fresh state is the native language chosen", async () => {
    const { deps, engine, scheduler, change } = erasing({});
    // The reader chose English, then erased their data before the change was over: the erasure
    // waits behind it, on an engine that last restored the French backup.
    const changed = change();
    const erased = scheduler.exclusive(() => engine.eraseAll());
    expect(await changed).toEqual({ ok: true, changed: true });
    await erased; // reset on the French engine, the owner would refuse what it saves

    expect(await stored(deps)).toEqual(FRESH_ENGLISH);
    expect((await deps.store.get(LAST_NATIVE_KEY))[LAST_NATIVE_KEY]).toBe("en");
    expect((await deps.store.get("cymbra-lingua-erased-at"))["cymbra-lingua-erased-at"]).toBe(5);
  });

  it("an erasure found by the next sync after a change resets to the native language chosen too", async () => {
    // This device synced before the account was erased elsewhere; the reader then chose English.
    const { deps, scheduler, change } = erasing({ "cymbra-lingua-status-cursor": 3 });
    expect(await change()).toEqual({ ok: true, changed: true });

    expect(await scheduler.syncNow()).toBeNull();

    expect(await stored(deps)).toEqual(FRESH_ENGLISH);
    expect((await deps.store.get("cymbra-lingua-erased-at"))["cymbra-lingua-erased-at"]).toBe(5);
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
