import { describe, expect, it, vi } from "vitest";
import { createTranslatorPort, translatorSource } from "@/translate/create-port.ts";
import type { TranslatorPort } from "@/translate/port.ts";
import {
  loadTranslationSetting,
  MODEL_STATE_KEY,
  languageReady,
  type ModelState,
  pairReady,
  parseHost,
  parseModelState,
  saveTranslationSetting,
  type SettingArea,
  TRANSLATION_HOST_KEY,
} from "@/translate/setting.ts";

// « Traduction étendue » as stored on the device (add-lingua-translation-delivery D2), and the
// translator a surface gets from it: none unless the reader chose the device AND the models of a
// ready pair that studies the sentence's language are there (generalise-lingua-translation-model-state
// D3, D5; generalise-lingua-translation-routes-by-pair D3).

const EN_FR = "en-fr/base-memory/2.0";
const ES_EN = "es-en/base-memory/2.0";
const READY: ModelState = { phase: "ready", models: [EN_FR], pairs: ["en-fr"] };

function memoryArea(seed: Record<string, unknown> = {}): SettingArea & { store: Record<string, unknown> } {
  const store: Record<string, unknown> = { ...seed };
  return {
    store,
    get: async (keys) => {
      const out: Record<string, unknown> = {};
      for (const k of Array.isArray(keys) ? keys : [keys]) if (k in store) out[k] = store[k];
      return out;
    },
    set: async (items) => void Object.assign(store, items),
  };
}

describe("the stored setting", () => {
  it("is off on a fresh install: an absent host is none", async () => {
    expect(await loadTranslationSetting(memoryArea())).toEqual({ host: "none", state: { phase: "absent" } });
  });

  it("stores a host, not a boolean, so a third place needs no migration", async () => {
    const area = memoryArea();
    await saveTranslationSetting(area, { host: "local", state: READY });
    expect(area.store).toEqual({ [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: READY });
    expect(await loadTranslationSetting(area)).toEqual({ host: "local", state: READY });
  });

  it("writes only what it is given", async () => {
    const area = memoryArea({ [TRANSLATION_HOST_KEY]: "local" });
    await saveTranslationSetting(area, { state: { phase: "removed" } });
    expect(area.store[TRANSLATION_HOST_KEY]).toBe("local");
  });

  it("reads anything it does not know as off", () => {
    expect(parseHost("local")).toBe("local");
    expect(parseHost("remote")).toBe("none"); // a later version's value, read by this one
    expect(parseHost(true)).toBe("none");
    expect(parseHost(undefined)).toBe("none");
  });

  it("reads each state, and anything malformed as absent", () => {
    expect(parseModelState({ phase: "downloading", received: 10, total: 20 })).toEqual({
      phase: "downloading",
      received: 10,
      total: 20,
    });
    expect(parseModelState({ phase: "interrupted", received: -1, total: "x" })).toEqual({
      phase: "interrupted",
      received: 0,
      total: 0,
    });
    expect(parseModelState(READY)).toEqual(READY);
    expect(parseModelState({ phase: "missing", models: [EN_FR], pairs: ["en-fr"], total: 26 })).toEqual({
      phase: "missing",
      models: [EN_FR],
      pairs: ["en-fr"],
      total: 26,
    });
    expect(parseModelState({ phase: "missing", models: "x", pairs: [3], total: -1 })).toEqual({
      phase: "missing",
      models: [],
      pairs: [],
      total: 0,
    });
    expect(parseModelState({ phase: "removed" })).toEqual({ phase: "removed" });
    expect(parseModelState({ phase: "failed", reason: "network" })).toEqual({ phase: "failed", reason: "network" });
    expect(parseModelState({ phase: "failed", reason: "EACCES" })).toEqual({ phase: "failed", reason: "unknown" });
    expect(parseModelState({ phase: "exploded" })).toEqual({ phase: "absent" });
    expect(parseModelState(null)).toEqual({ phase: "absent" });
  });

  describe("the three stored shapes (generalise-lingua-translation-routes-by-pair D3)", () => {
    it("reads a state that names pairs as written", () => {
      expect(parseModelState({ phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] })).toEqual({
        phase: "ready",
        models: [EN_FR, ES_EN],
        pairs: ["en-fr", "es-fr"],
      });
      // `pairs` wins over a `languages` that would have been left beside it.
      expect(parseModelState({ phase: "ready", models: [EN_FR], pairs: ["en-fr"], languages: ["es"] })).toEqual(READY);
    });

    it("A state recorded before pairs: a stored `languages` is read as the pairs of the default native language", () => {
      // Every release since generalise-lingua-translation-model-state wrote `languages`; every pair
      // shipped so far is glossed in French (M22), so `en` and `es` are en-fr and es-fr.
      const ready = parseModelState({ phase: "ready", models: [EN_FR, ES_EN], languages: ["en", "es"] });
      expect(ready).toEqual({ phase: "ready", models: [EN_FR, ES_EN], pairs: ["en-fr", "es-fr"] });
      expect(parseModelState({ phase: "missing", models: [EN_FR], languages: ["en"], total: 26 })).toEqual({
        phase: "missing",
        models: [EN_FR],
        pairs: ["en-fr"],
        total: 26,
      });
      // English and Spanish pages are translated as before, through the pairs the background gates on.
      expect(languageReady("local", ready, "en")).toBe(true);
      expect(languageReady("local", ready, "es")).toBe(true);
      expect(pairReady("local", ready, "en-fr")).toBe(true);
      expect(pairReady("local", ready, "es-fr")).toBe(true);
      expect(parseModelState({ phase: "missing", models: "x", languages: [3], total: -1 })).toEqual({
        phase: "missing",
        models: [],
        pairs: [],
        total: 0,
      });
    });

    it("reads a bare `ready`, from the release before anything was recorded with it, as en-fr", () => {
      // That release recorded `ready` only once the English model — en-fr's one model — was there.
      expect(parseModelState({ phase: "ready" })).toEqual({ phase: "ready", models: [], pairs: ["en-fr"] });
      expect(parseModelState({ phase: "missing", models: [EN_FR], total: 26 })).toEqual({
        phase: "missing",
        models: [EN_FR],
        pairs: [],
        total: 26,
      });
    });
  });

  it("is ready for a pair only when on and its models are there — the background's gate, exact", () => {
    expect(pairReady("local", READY, "en-fr")).toBe(true);
    expect(pairReady("local", READY, "es-fr")).toBe(false);
    expect(pairReady("local", READY, "en-es")).toBe(false); // the same page, another native language
    expect(pairReady("local", READY, "en")).toBe(false); // a studied language is no pair
    expect(pairReady("none", READY, "en-fr")).toBe(false);
    // A language added since: its pair's model is missing, en-fr is still translated.
    const missing: ModelState = { phase: "missing", models: [EN_FR], pairs: ["en-fr"], total: 26 };
    expect(pairReady("local", missing, "en-fr")).toBe(true);
    expect(pairReady("local", missing, "es-fr")).toBe(false);
    for (const state of [
      { phase: "absent" },
      { phase: "downloading", received: 1, total: 2 },
      { phase: "failed", reason: "network" },
      { phase: "interrupted", received: 1, total: 2 },
      { phase: "removed" },
    ] as ModelState[]) {
      expect(pairReady("local", state, "en-fr")).toBe(false);
    }
  });

  it("is ready for a language, as a page asks, when a ready pair studies it", () => {
    expect(languageReady("local", READY, "en")).toBe(true);
    expect(languageReady("local", READY, "es")).toBe(false);
    expect(languageReady("none", READY, "en")).toBe(false);
    // Only es-fr ready: an English page gets nothing, a Spanish one does.
    const spanish: ModelState = { phase: "ready", models: [ES_EN, EN_FR], pairs: ["es-fr"] };
    expect(languageReady("local", spanish, "en")).toBe(false);
    expect(languageReady("local", spanish, "es")).toBe(true);
    // A language added since: its pair's model is missing, English is still translated.
    const missing: ModelState = { phase: "missing", models: [EN_FR], pairs: ["en-fr"], total: 26 };
    expect(languageReady("local", missing, "en")).toBe(true);
    expect(languageReady("local", missing, "es")).toBe(false);
    for (const state of [
      { phase: "absent" },
      { phase: "downloading", received: 1, total: 2 },
      { phase: "failed", reason: "network" },
      { phase: "interrupted", received: 1, total: 2 },
      { phase: "removed" },
    ] as ModelState[]) {
      expect(languageReady("local", state, "en")).toBe(false);
    }
  });
});

describe("translatorSource — which translator a surface gets", () => {
  const flush = async () => {
    for (let i = 0; i < 5; i++) await Promise.resolve();
  };

  function setup(seed: Record<string, unknown>) {
    const area = memoryArea(seed);
    let changed: (() => void) | null = null;
    const watched: string[][] = [];
    const port: TranslatorPort = { translate: vi.fn() };
    const make = vi.fn(() => port);
    const source = translatorSource({
      area,
      watch: (keys, onChange) => {
        watched.push([...keys]);
        changed = onChange;
      },
      port: make,
    });
    return { source, area, port, make, watched, change: () => changed?.() };
  }

  it("gives none before the setting has been read — the safe answer", () => {
    const { source } = setup({ [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } });
    expect(source("en")).toBeNull();
  });

  it("gives the messaging port once the setting is on and the model ready", async () => {
    const { source, port, make } = setup({ [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } });
    await flush();
    expect(source("en")).toBe(port);
    expect(source("en")).toBe(port);
    expect(make).toHaveBeenCalledOnce(); // one port per page: its keep-warm starts once
  });

  it("gives a translator per language: English while es-fr's model is missing (model-state D5)", async () => {
    const { source, port } = setup({
      [TRANSLATION_HOST_KEY]: "local",
      [MODEL_STATE_KEY]: { phase: "missing", models: [EN_FR], pairs: ["en-fr"], total: 26 },
    });
    await flush();
    expect(source("en")).toBe(port);
    expect(source("es")).toBeNull();
  });

  it("gates a page on the document's language through the ready pairs: en with en-fr ready gets a port, with only es-fr ready none (routes-by-pair D3)", async () => {
    const english = setup({
      [TRANSLATION_HOST_KEY]: "local",
      [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR], pairs: ["en-fr"] },
    });
    await flush();
    expect(english.source("en")).toBe(english.port);

    const spanish = setup({
      [TRANSLATION_HOST_KEY]: "local",
      [MODEL_STATE_KEY]: { phase: "ready", models: [ES_EN, EN_FR], pairs: ["es-fr"] },
    });
    await flush();
    expect(spanish.source("en")).toBeNull();
    expect(spanish.source("es")).toBe(spanish.port);
  });

  it("follows a state written before pairs as it was meant: English and Spanish pages still get a port", async () => {
    const { source, port } = setup({
      [TRANSLATION_HOST_KEY]: "local",
      [MODEL_STATE_KEY]: { phase: "ready", models: [EN_FR, ES_EN], languages: ["en", "es"] },
    });
    await flush();
    expect(source("en")).toBe(port);
    expect(source("es")).toBe(port);
  });

  for (const state of [
    { phase: "absent" },
    { phase: "downloading", received: 1, total: 2 },
    { phase: "failed", reason: "network" },
    { phase: "interrupted", received: 1, total: 2 },
    { phase: "removed" },
  ]) {
    it(`gives none while the model is ${state.phase}`, async () => {
      const { source } = setup({ [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: state });
      await flush();
      expect(source("en")).toBeNull();
    });
  }

  it("gives none when the setting is off, whatever was recorded", async () => {
    const { source } = setup({ [MODEL_STATE_KEY]: { phase: "ready" } });
    await flush();
    expect(source("en")).toBeNull();
  });

  it("follows the setting: a model that arrives, then a setting turned off", async () => {
    const { source, area, change, watched } = setup({
      [TRANSLATION_HOST_KEY]: "local",
      [MODEL_STATE_KEY]: { phase: "downloading", received: 0, total: 1 },
    });
    expect(watched).toEqual([[TRANSLATION_HOST_KEY, MODEL_STATE_KEY]]);
    await flush();
    expect(source("en")).toBeNull();

    area.store[MODEL_STATE_KEY] = { phase: "ready" };
    change();
    await flush();
    expect(source("en")).not.toBeNull();

    area.store[TRANSLATION_HOST_KEY] = "none";
    change();
    await flush();
    expect(source("en")).toBeNull();
  });

  it("gives none when the storage cannot be read (an orphaned page)", async () => {
    const source = translatorSource({
      area: { get: async () => Promise.reject(new Error("Extension context invalidated")), set: async () => {} },
      watch: () => {},
      port: () => ({ translate: vi.fn() }),
    });
    await flush();
    expect(source("en")).toBeNull();
  });

  it("is never anything in a variant without the engine", () => {
    // The test build defines __TRANSLATION_HOST__ as "none", as Safari's does.
    expect(createTranslatorPort()("en")).toBeNull();
  });
});
