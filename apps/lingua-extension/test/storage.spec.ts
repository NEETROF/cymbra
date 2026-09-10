import { describe, expect, it } from "vitest";
import {
  type AsyncStorageArea,
  DEFAULT_CALIBRATION,
  defaultState,
  loadState,
  migrate,
  ROOT_KEY,
  SCHEMA_VERSION,
  saveState,
} from "@/state/storage.ts";

/** An in-memory chrome.storage.local stand-in. */
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

describe("migrate", () => {
  it("returns defaults for undefined/garbage", () => {
    expect(migrate(undefined)).toEqual(defaultState());
    expect(migrate(null)).toEqual(defaultState());
    expect(migrate(42)).toEqual(defaultState());
    expect(defaultState().calibration).toBe(DEFAULT_CALIBRATION);
  });

  it("folds the pre-schema flat shape (no version) forward without loss", () => {
    const legacy = { statuses: { run: "known", seldom: "learning" }, calib: 1500 };
    const migrated = migrate(legacy);
    expect(migrated.version).toBe(SCHEMA_VERSION);
    expect(migrated.calibration).toBe(1500);
    expect(migrated.statuses).toEqual({ run: "known", seldom: "learning" });
    expect(migrated.cards).toEqual({});
    expect(migrated.prefs.autoHighlight).toBe(false);
  });

  it("preserves a current-version object", () => {
    const state = { ...defaultState(), calibration: 2000, statuses: { city: "ignored" as const } };
    expect(migrate(state)).toEqual(state);
  });

  it("never downgrades a newer-than-known version", () => {
    const future = { version: 99, statuses: {}, cards: {}, calibration: 4000, prefs: { autoHighlight: true } };
    expect(migrate(future).version).toBe(99);
  });

  it("drops a malformed statuses/cards map instead of trusting it", () => {
    const bad = { version: 1, statuses: { x: "bogus" }, cards: { y: { nope: true } }, calibration: 3000, prefs: {} };
    const migrated = migrate(bad);
    expect(migrated.statuses).toEqual({});
    expect(migrated.cards).toEqual({});
  });
});

describe("loadState / saveState", () => {
  it("migrates a legacy store on read and persists the upgraded shape", async () => {
    const area = fakeArea({ [ROOT_KEY]: { statuses: { run: "known" }, calib: 1200 } });
    const state = await loadState(area);
    expect(state.version).toBe(SCHEMA_VERSION);
    expect(state.calibration).toBe(1200);
    // Persisted: a second read sees the migrated shape directly.
    expect((area.store[ROOT_KEY] as { version: number }).version).toBe(SCHEMA_VERSION);
  });

  it("round-trips a saved state", async () => {
    const area = fakeArea();
    const state = { ...defaultState(), calibration: 5000 };
    await saveState(area, state);
    expect(await loadState(area)).toEqual(state);
  });

  it("returns defaults for an empty store", async () => {
    expect(await loadState(fakeArea())).toEqual(defaultState());
  });
});
