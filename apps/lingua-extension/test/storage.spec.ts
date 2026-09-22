import { describe, expect, it } from "vitest";
import {
  type AsyncStorageArea,
  classifyStored,
  ENABLED_KEY,
  hydrateEngine,
  hydrateFromV1,
  HUD_HIDDEN_KEY,
  loadEnabled,
  loadHudHidden,
  loadStored,
  ROOT_KEY,
  saveBackup,
  saveEnabled,
  saveHudHidden,
  STORAGE_VERSION,
  type V1State,
} from "@/state/storage.ts";
import { makeFakePort } from "./helpers.ts";

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

describe("classifyStored", () => {
  it("recognises a v2 backup-backed shape", () => {
    expect(classifyStored({ v: 2, backup: '{"schema_version":1}' })).toEqual({
      kind: "v2",
      backup: '{"schema_version":1}',
    });
  });

  it("recognises the v1 reading-only shape", () => {
    const raw = { version: 1, statuses: { run: "known" }, cards: {}, calibration: 1500 };
    const c = classifyStored(raw);
    expect(c.kind).toBe("v1");
    if (c.kind === "v1") expect(c.v1.calibration).toBe(1500);
  });

  it("treats null / garbage / unknown shapes as empty", () => {
    expect(classifyStored(null).kind).toBe("empty");
    expect(classifyStored(42).kind).toBe("empty");
    expect(classifyStored({ nothing: true }).kind).toBe("empty");
  });
});

describe("loadStored / saveBackup", () => {
  it("round-trips a saved backup as v2", async () => {
    const area = fakeArea();
    await saveBackup(area, "BACKUP-STRING");
    expect(await loadStored(area)).toEqual({ kind: "v2", backup: "BACKUP-STRING" });
    expect((area.store[ROOT_KEY] as { v: number }).v).toBe(STORAGE_VERSION);
  });

  it("reports empty for an empty store", async () => {
    expect((await loadStored(fakeArea())).kind).toBe("empty");
  });
});

describe("loadEnabled / saveEnabled", () => {
  it("defaults to enabled when the flag was never set", async () => {
    expect(await loadEnabled(fakeArea())).toBe(true);
  });

  it("round-trips the flag and never touches the state backup", async () => {
    const area = fakeArea({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: "KEEP" } });
    await saveEnabled(area, false);
    expect(await loadEnabled(area)).toBe(false);
    expect(area.store[ENABLED_KEY]).toBe(false);
    // Toggling the reader must not disturb the deck/status backup.
    expect(area.store[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "KEEP" });
    await saveEnabled(area, true);
    expect(await loadEnabled(area)).toBe(true);
  });
});

describe("hydrateFromV1", () => {
  it("migrates calibration, statuses and learning cards into the engine", async () => {
    const { port, calls } = makeFakePort();
    const v1: V1State = {
      calibration: 1500,
      statuses: { run: "known", city: "ignored", seldom: "learning" },
      cards: { seldom: { lemma: "seldom", surface: "Seldom", sentence: "They seldom ship.", createdAt: 111 } },
    };
    const backup = await hydrateFromV1(port, v1);

    expect(calls.setCalibration).toEqual([1500]);
    // known/ignored → plain statuses; learning → a deck card.
    expect(calls.setStatus).toEqual(
      expect.arrayContaining([
        ["run", "known"],
        ["city", "ignored"],
      ]),
    );
    expect(calls.setStatus).not.toContainEqual(["seldom", "learning"]);
    expect(calls.addCard).toEqual([
      { lemma: "seldom", surface: "Seldom", sentence: "They seldom ship.", url: "", gloss: null, capturedAt: 111 },
    ]);
    expect(typeof backup).toBe("string");
  });

  it("falls back to the default calibration when v1 has none", async () => {
    const { port, calls } = makeFakePort();
    await hydrateFromV1(port, { calibration: 0, statuses: {}, cards: {} });
    expect(calls.setCalibration).toEqual([3000]);
  });
});

describe("hydrateEngine", () => {
  it("restores the stored backup, and leaves it exactly as it was", async () => {
    const area = fakeArea({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: "BACKUP-V2" } });
    const { port, calls } = makeFakePort();
    await hydrateEngine(port, area);
    expect(calls.restored).toEqual(["BACKUP-V2"]);
    expect(area.store[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "BACKUP-V2" });
  });

  it("forward-migrates a v1 store, and writes v2 back so it happens once", async () => {
    const area = fakeArea({
      [ROOT_KEY]: {
        statuses: { seldom: "learning", ship: "known" },
        cards: { seldom: { lemma: "seldom", surface: "seldom", sentence: "They seldom ship.", createdAt: 7 } },
        calibration: 2500,
      } satisfies V1State,
    });
    const { port, calls } = makeFakePort();
    await hydrateEngine(port, area);

    expect(calls.setCalibration).toEqual([2500]);
    expect(calls.addCard.map((c) => [c.lemma, c.sentence, c.capturedAt])).toEqual([["seldom", "They seldom ship.", 7]]);
    expect(calls.setStatus).toEqual([["ship", "known"]]);
    // A second load must be a plain restore: migrating twice would re-add every card.
    expect(classifyStored(area.store[ROOT_KEY]).kind).toBe("v2");
  });

  it("seeds a fresh install from the engine's own default backup", async () => {
    const area = fakeArea();
    const { port, calls } = makeFakePort();
    await hydrateEngine(port, area);
    expect(calls.restored).toEqual([]); // there was nothing to restore
    expect(area.store[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "{}" });
  });
});

describe("the HUD-hidden flag", () => {
  it("shows the pill until it is hidden, and shows it again when unhidden", async () => {
    // Absent means shown: a fresh install must not start with the pill missing.
    const area = fakeArea();
    expect(await loadHudHidden(area)).toBe(false);
    await saveHudHidden(area, true);
    expect(area.store[HUD_HIDDEN_KEY]).toBe(true);
    expect(await loadHudHidden(area)).toBe(true);
    await saveHudHidden(area, false);
    expect(await loadHudHidden(area)).toBe(false);
  });

  it("stays out of the state backup, so toggling it never rewrites the deck", async () => {
    const area = fakeArea({ [ROOT_KEY]: { v: STORAGE_VERSION, backup: "BACKUP" } });
    await saveHudHidden(area, true);
    expect(area.store[ROOT_KEY]).toEqual({ v: STORAGE_VERSION, backup: "BACKUP" });
  });
});
