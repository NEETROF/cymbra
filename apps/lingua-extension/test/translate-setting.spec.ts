import { describe, expect, it, vi } from "vitest";
import { createTranslatorPort, translatorSource } from "@/translate/create-port.ts";
import type { TranslatorPort } from "@/translate/port.ts";
import {
  loadTranslationSetting,
  MODEL_STATE_KEY,
  type ModelState,
  modelReady,
  parseHost,
  parseModelState,
  saveTranslationSetting,
  type SettingArea,
  TRANSLATION_HOST_KEY,
} from "@/translate/setting.ts";

// « Traduction étendue » as stored on the device (add-lingua-translation-delivery D2), and the
// translator a surface gets from it: none unless the reader chose the device AND its model is there.

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
    await saveTranslationSetting(area, { host: "local", state: { phase: "ready" } });
    expect(area.store).toEqual({ [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } });
    expect(await loadTranslationSetting(area)).toEqual({ host: "local", state: { phase: "ready" } });
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
    expect(parseModelState({ phase: "ready" })).toEqual({ phase: "ready" });
    expect(parseModelState({ phase: "removed" })).toEqual({ phase: "removed" });
    expect(parseModelState({ phase: "failed", reason: "network" })).toEqual({ phase: "failed", reason: "network" });
    expect(parseModelState({ phase: "failed", reason: "EACCES" })).toEqual({ phase: "failed", reason: "unknown" });
    expect(parseModelState({ phase: "exploded" })).toEqual({ phase: "absent" });
    expect(parseModelState(null)).toEqual({ phase: "absent" });
  });

  it("is ready only when on and the model is there", () => {
    expect(modelReady("local", { phase: "ready" })).toBe(true);
    expect(modelReady("none", { phase: "ready" })).toBe(false);
    for (const state of [
      { phase: "absent" },
      { phase: "downloading", received: 1, total: 2 },
      { phase: "failed", reason: "network" },
      { phase: "interrupted", received: 1, total: 2 },
      { phase: "removed" },
    ] as ModelState[]) {
      expect(modelReady("local", state)).toBe(false);
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
    expect(source()).toBeNull();
  });

  it("gives the messaging port once the setting is on and the model ready", async () => {
    const { source, port, make } = setup({ [TRANSLATION_HOST_KEY]: "local", [MODEL_STATE_KEY]: { phase: "ready" } });
    await flush();
    expect(source()).toBe(port);
    expect(source()).toBe(port);
    expect(make).toHaveBeenCalledOnce(); // one port per page: its keep-warm starts once
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
      expect(source()).toBeNull();
    });
  }

  it("gives none when the setting is off, whatever was recorded", async () => {
    const { source } = setup({ [MODEL_STATE_KEY]: { phase: "ready" } });
    await flush();
    expect(source()).toBeNull();
  });

  it("follows the setting: a model that arrives, then a setting turned off", async () => {
    const { source, area, change, watched } = setup({
      [TRANSLATION_HOST_KEY]: "local",
      [MODEL_STATE_KEY]: { phase: "downloading", received: 0, total: 1 },
    });
    expect(watched).toEqual([[TRANSLATION_HOST_KEY, MODEL_STATE_KEY]]);
    await flush();
    expect(source()).toBeNull();

    area.store[MODEL_STATE_KEY] = { phase: "ready" };
    change();
    await flush();
    expect(source()).not.toBeNull();

    area.store[TRANSLATION_HOST_KEY] = "none";
    change();
    await flush();
    expect(source()).toBeNull();
  });

  it("gives none when the storage cannot be read (an orphaned page)", async () => {
    const source = translatorSource({
      area: { get: async () => Promise.reject(new Error("Extension context invalidated")), set: async () => {} },
      watch: () => {},
      port: () => ({ translate: vi.fn() }),
    });
    await flush();
    expect(source()).toBeNull();
  });

  it("is never anything in a variant without the engine", () => {
    // The test build defines __TRANSLATION_HOST__ as "none", as Safari's does.
    expect(createTranslatorPort()()).toBeNull();
  });
});
