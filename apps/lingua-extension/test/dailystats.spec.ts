import { afterEach, describe, expect, it, vi } from "vitest";
import type { AsyncStorageArea } from "@/state/storage.ts";
import {
  clearDailyStats,
  dailyRecorder,
  loadDailyStats,
  recordExposures,
  recordReview,
  recordWordLearned,
  utcDay,
} from "@/state/dailystats.ts";

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

describe("dailyRecorder", () => {
  afterEach(() => vi.useRealTimers());

  /** The recorder is fire-and-forget, so let its write finish before reading it back.
   *  One event at a time: two overlapping bumps may lose an increment, which the module
   *  accepts on purpose — these are approximate activity counts, not state. */
  const settle = async () => {
    for (let i = 0; i < 5; i++) await Promise.resolve();
  };

  it("counts a grade as a review and a mark-known as a word learned, on today's UTC day", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-04T12:00:00Z"));
    const day = utcDay(Date.parse("2026-03-04T12:00:00Z"));

    const area = fakeArea();
    const record = dailyRecorder(area);
    record("review");
    await settle();
    record("learned");
    await settle();
    record("review");
    await settle();

    expect(await loadDailyStats(area)).toEqual({ [day]: { exposures: 0, wordsLearned: 1, reviews: 2 } });
  });

  it("files each event under the day it happened, across a UTC midnight", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-04T23:59:00Z"));
    const area = fakeArea();
    const record = dailyRecorder(area);
    record("review");
    await settle();
    vi.setSystemTime(new Date("2026-03-05T00:01:00Z"));
    record("review");
    await settle();

    const stats = await loadDailyStats(area);
    expect(Object.keys(stats)).toHaveLength(2);
  });
});

describe("clearDailyStats", () => {
  it("drops every local count, as a Lingua-only erasure must", async () => {
    const area = fakeArea();
    await recordExposures(area, 5, 3);
    await recordReview(area, 6);
    await clearDailyStats(area);
    expect(await loadDailyStats(area)).toEqual({});
  });
});
