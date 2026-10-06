import { afterEach, describe, expect, it, vi } from "vitest";
import type { AsyncStorageArea } from "@/state/storage.ts";
import {
  clearDailyStats,
  countsOf,
  DAILY_KEY,
  type DailyStats,
  DAY_ONLY_DAILY_KEY,
  dailyRecorder,
  loadDailyStats,
  recordReading,
  recordReview,
  recordWordLearned,
  RETIRED_DAILY_KEY,
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
    await recordReading(area, 10, 4, 1);
    await recordReading(area, 10, 6, 2);
    await recordWordLearned(area, 10);
    await recordReview(area, 10);
    await recordReview(area, 11); // a different day is separate

    const stats = await loadDailyStats(area);
    expect(stats[10]).toEqual({ en: { exposures: 10, unknownSeen: 3, wordsLearned: 1, reviews: 1 } });
    expect(stats[11]).toEqual({ en: { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 1 } });
  });

  it("keep each studied language apart on the same day", async () => {
    const area = fakeArea();
    await recordReading(area, 10, 4, 1, "en");
    await recordReading(area, 10, 9, 5, "es");
    await recordWordLearned(area, 10, "es");
    await recordReview(area, 10, "en");

    expect((await loadDailyStats(area))[10]).toEqual({
      en: { exposures: 4, unknownSeen: 1, wordsLearned: 0, reviews: 1 },
      es: { exposures: 9, unknownSeen: 5, wordsLearned: 1, reviews: 0 },
    });
  });

  it("ignores a reading with nothing read", async () => {
    const area = fakeArea();
    await recordReading(area, 5, 0, 0);
    await recordReading(area, 5, -3, 1);
    expect(await loadDailyStats(area)).toEqual({});
  });

  it("keeps its counts under the v3 key, never the retired whole-document one", async () => {
    const area = fakeArea({ [RETIRED_DAILY_KEY]: { 5: { exposures: 90_000, wordsLearned: 0, reviews: 0 } } });
    expect(await loadDailyStats(area)).toEqual({});
    await recordReading(area, 5, 12, 2);
    expect(area.store[DAILY_KEY]).toEqual({
      5: { en: { exposures: 12, unknownSeen: 2, wordsLearned: 0, reviews: 0 } },
    });
  });

  it("reads a day stored without new words seen as zero new words", async () => {
    const area = fakeArea({ [DAY_ONLY_DAILY_KEY]: { 5: { exposures: 3, wordsLearned: 0, reviews: 0 } } });
    await recordReview(area, 5);
    expect((await loadDailyStats(area))[5]).toEqual({
      en: { exposures: 3, unknownSeen: 0, wordsLearned: 0, reviews: 1 },
    });
  });

  it("returns an empty map for a fresh store", async () => {
    expect(await loadDailyStats(fakeArea())).toEqual({});
  });
});

describe("the counts kept per day only", () => {
  // Every reader read English before the counts were kept per language: v2's days are English.
  const DAY_ONLY = {
    5: { exposures: 12, unknownSeen: 2, wordsLearned: 1, reviews: 3 },
    6: { exposures: 4, unknownSeen: 0, wordsLearned: 0, reviews: 0 },
  };

  it("are read as English until the first per-language write", async () => {
    const area = fakeArea({ [DAY_ONLY_DAILY_KEY]: DAY_ONLY });

    expect(await loadDailyStats(area)).toEqual({ 5: { en: DAY_ONLY[5] }, 6: { en: DAY_ONLY[6] } });
  });

  it("are carried over by the first write, which adds to them", async () => {
    const area = fakeArea({ [DAY_ONLY_DAILY_KEY]: DAY_ONLY });

    await recordReview(area, 5, "es");

    expect(area.store[DAILY_KEY]).toEqual({
      5: { en: DAY_ONLY[5], es: { exposures: 0, unknownSeen: 0, wordsLearned: 0, reviews: 1 } },
      6: { en: DAY_ONLY[6] },
    });
    // From then on v3 is the truth: nothing is counted twice.
    await recordReview(area, 6);
    expect((await loadDailyStats(area))[6]).toEqual({ en: { ...DAY_ONLY[6], reviews: 1 } });
  });

  it("are not read once v3 exists, even empty", async () => {
    const area = fakeArea({ [DAY_ONLY_DAILY_KEY]: DAY_ONLY, [DAILY_KEY]: {} });

    expect(await loadDailyStats(area)).toEqual({});
  });
});

describe("countsOf", () => {
  it("picks one language's days, missing fields as zero", () => {
    // English on day 5 was stored before new words were counted.
    const stats = {
      5: {
        en: { exposures: 3, wordsLearned: 0, reviews: 0 },
        es: { exposures: 8, unknownSeen: 1, wordsLearned: 2, reviews: 0 },
      },
      6: { en: { exposures: 1, unknownSeen: 0, wordsLearned: 0, reviews: 4 } },
    } as unknown as DailyStats;

    expect(countsOf(stats, "es")).toEqual({ 5: { exposures: 8, unknownSeen: 1, wordsLearned: 2, reviews: 0 } });
    expect(countsOf(stats, "en")).toEqual({
      5: { exposures: 3, unknownSeen: 0, wordsLearned: 0, reviews: 0 },
      6: { exposures: 1, unknownSeen: 0, wordsLearned: 0, reviews: 4 },
    });
    expect(countsOf(stats, "de")).toEqual({});
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

    expect(await loadDailyStats(area)).toEqual({
      [day]: { en: { exposures: 0, unknownSeen: 0, wordsLearned: 1, reviews: 2 } },
    });
  });

  it("counts an event in the language it names", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-04T12:00:00Z"));
    const day = utcDay(Date.parse("2026-03-04T12:00:00Z"));

    const area = fakeArea();
    const record = dailyRecorder(area);
    record("review", "es");
    await settle();
    record("learned", "es");
    await settle();

    expect(await loadDailyStats(area)).toEqual({
      [day]: { es: { exposures: 0, unknownSeen: 0, wordsLearned: 1, reviews: 1 } },
    });
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
    await recordReading(area, 5, 3, 1);
    await recordReview(area, 6, "es");
    await clearDailyStats(area);
    expect(await loadDailyStats(area)).toEqual({});
  });

  it("drops the counts kept per day only too, so they never come back as English", async () => {
    const area = fakeArea({
      [DAY_ONLY_DAILY_KEY]: { 5: { exposures: 3, unknownSeen: 0, wordsLearned: 0, reviews: 0 } },
    });
    await clearDailyStats(area);
    expect(area.store[DAY_ONLY_DAILY_KEY]).toEqual({});
    expect(area.store[DAILY_KEY]).toEqual({});
    expect(await loadDailyStats(area)).toEqual({});
  });
});
