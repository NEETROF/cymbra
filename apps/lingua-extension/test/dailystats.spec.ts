import { describe, expect, it } from "vitest";
import type { AsyncStorageArea } from "@/state/storage.ts";
import { loadDailyStats, recordExposures, recordReview, recordWordLearned, utcDay } from "@/state/dailystats.ts";

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

describe("utcDay", () => {
  it("is the number of whole UTC days since the epoch", () => {
    expect(utcDay(0)).toBe(0);
    expect(utcDay(86_400_000)).toBe(1);
    expect(utcDay(86_400_000 * 3 + 5)).toBe(3);
  });
});

describe("daily counters", () => {
  it("accumulate per day and per field, defaulting missing fields to zero", async () => {
    const area = fakeArea();
    await recordExposures(area, 10, 4);
    await recordExposures(area, 10, 6);
    await recordWordLearned(area, 10);
    await recordReview(area, 10);
    await recordReview(area, 11); // a different day is separate

    const stats = await loadDailyStats(area);
    expect(stats[10]).toEqual({ exposures: 10, wordsLearned: 1, reviews: 1 });
    expect(stats[11]).toEqual({ exposures: 0, wordsLearned: 0, reviews: 1 });
  });

  it("ignores a non-positive exposure count", async () => {
    const area = fakeArea();
    await recordExposures(area, 5, 0);
    await recordExposures(area, 5, -3);
    expect(await loadDailyStats(area)).toEqual({});
  });

  it("returns an empty map for a fresh store", async () => {
    expect(await loadDailyStats(fakeArea())).toEqual({});
  });
});
