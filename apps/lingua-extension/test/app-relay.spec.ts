import { describe, expect, it } from "vitest";
import type { CardOp, StatusOp } from "@/analyzer/port.ts";
import { DAILY_KEY, loadDailyStats } from "@/state/dailystats.ts";
import {
  accountDigest,
  APPLIED_APP_DAILY_KEY,
  APPLIED_APP_DAILY_TTL_MS,
  applyAppDaily,
  applyAppOps,
  appOpsSince,
  encodeAppOps,
  encodeSharedState,
  onlineAccessOf,
  parseAppOps,
  parseSharedState,
  RELAY_VERSION,
  reviewCountsBetween,
  sharedDailyStats,
  toSecond,
  withReviewCounts,
} from "@/state/app-relay.ts";
import { makeFakePort } from "./helpers.ts";

const card = (client_ts: number, lemma = "dog"): CardOp => ({
  client_id: lemma,
  language: "en",
  lemma,
  surface_form: lemma,
  source_sentence: "",
  source: "",
  gloss: "",
  fsrs_state: "{}",
  deleted: false,
  client_ts,
  device_id: "",
});
const status = (updated_at: number, lemma = "dog"): StatusOp => ({
  language: "en",
  lemma,
  status: "known",
  provenance: "srs",
  updated_at,
});

describe("the shared state (add-lingua-apple-review D3)", () => {
  it("reads back what it wrote", () => {
    const raw = encodeSharedState({ backup: "B", savedAt: 42, prefs: { a: 1 }, storeItems: { d: 2 } });
    expect(parseSharedState(raw)).toEqual({
      v: RELAY_VERSION,
      backup: "B",
      savedAt: 42,
      prefs: { a: 1 },
      storeItems: { d: 2 },
      onlineAccess: false,
      erasedAt: 0,
    });
    expect(
      parseSharedState(encodeSharedState({ backup: "B", savedAt: 1, prefs: {}, storeItems: {}, onlineAccess: true })),
    ).toMatchObject({ onlineAccess: true });
  });

  it("reads nothing, another version, an empty backup or garbage as no state", () => {
    for (const raw of [
      undefined,
      null,
      3,
      "",
      "not json",
      "[]",
      JSON.stringify({ v: 2, backup: "B" }),
      JSON.stringify({ v: 1, backup: "" }),
    ]) {
      expect(parseSharedState(raw)).toBeNull();
    }
  });

  it("defaults a missing date and malformed maps", () => {
    expect(parseSharedState(JSON.stringify({ v: 1, backup: "B", prefs: [1], storeItems: null }))).toEqual({
      v: 1,
      backup: "B",
      savedAt: 0,
      prefs: {},
      storeItems: {},
      onlineAccess: false,
      erasedAt: 0,
    });
  });

  // add-lingua-review-translation-fallback D9: an older payload has no access; only `true` grants it.
  it("reads an older payload, or anything but true, as no online access", () => {
    for (const onlineAccess of [undefined, "true", 1, null]) {
      expect(parseSharedState(JSON.stringify({ v: 1, backup: "B", onlineAccess }))?.onlineAccess).toBe(false);
    }
  });

  it("reads the one flag from the extension's snapshot, whatever else it holds", () => {
    const cases: [unknown, boolean][] = [
      [{ account: "a", values: { "lingua.online_translation.enabled": true, other: true } }, true],
      [{ account: "a", values: { "lingua.online_translation.enabled": false } }, false],
      [{ account: "a", values: {} }, false],
      [{ account: "a" }, false],
      [null, false],
      [undefined, false],
    ];
    for (const [snapshot, access] of cases) expect(onlineAccessOf(snapshot), JSON.stringify(snapshot)).toBe(access);
  });
});

describe("the account and the erasure mark the state carries (add-lingua-app-deck-sync D7, D8)", () => {
  it("encodes the account's digest and the mark, and reads them back", () => {
    const raw = encodeSharedState({ backup: "B", savedAt: 1, prefs: {}, storeItems: {}, account: "abc", erasedAt: 77 });
    expect(parseSharedState(raw)).toMatchObject({ account: "abc", erasedAt: 77 });
    const out = parseSharedState(
      encodeSharedState({ backup: "B", savedAt: 1, prefs: {}, storeItems: {}, account: null }),
    );
    expect(out?.account).toBeNull();
    expect(out?.erasedAt).toBe(0);
  });

  it("reads an older payload as no account at all — apart from a signed-out one — and as never erased", () => {
    const older = parseSharedState(JSON.stringify({ v: 1, backup: "B" }));
    expect(older).not.toHaveProperty("account");
    expect(older?.account).toBeUndefined();
    expect(older?.erasedAt).toBe(0);
    expect(parseSharedState(JSON.stringify({ v: 1, backup: "B", account: 3, erasedAt: "x" }))).toMatchObject({
      account: null,
      erasedAt: 0,
    });
    expect(parseSharedState(JSON.stringify({ v: 1, backup: "B", account: "" }))?.account).toBeNull();
  });

  it("digests the account id with SHA-256, in hex, never the id itself", async () => {
    const digest = await accountDigest("user-1");
    expect(digest).toMatch(/^[0-9a-f]{64}$/);
    expect(digest).not.toContain("user-1");
    expect(await accountDigest("user-1")).toBe(digest);
    expect(await accountDigest("user-2")).not.toBe(digest);
    // The prefix is part of what is hashed: not a bare SHA-256 of the id.
    const bare = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode("user-1")));
    expect(digest).not.toBe(Array.from(bare, (b) => b.toString(16).padStart(2, "0")).join(""));
  });
});

describe("the app's changes", () => {
  it("reads back what it wrote, and refuses what is not ops", () => {
    const ops = { since: 1000, cardOps: [card(2000)], statusOps: [status(3000)] };
    expect(parseAppOps(encodeAppOps(ops))).toEqual({ v: 1, ...ops });
    expect(parseAppOps(JSON.stringify({ v: 1, cardOps: [] }))).toBeNull();
    expect(parseAppOps(JSON.stringify({ v: 9, cardOps: [], statusOps: [] }))).toBeNull();
    expect(parseAppOps("{")).toBeNull();
    expect(parseAppOps(JSON.stringify({ v: 1, cardOps: [], statusOps: [] }))?.since).toBe(0);
  });

  it("dates to the second, as the deck does", () => {
    expect(toSecond(12_345)).toBe(12_000);
  });

  it("exports only what changed from `since` on", async () => {
    const { port } = makeFakePort();
    port.exportCardOps = async () => [card(1000, "old"), card(5000, "new")];
    port.exportStatusOps = async () => [status(999, "old"), status(5000, "new")];
    const ops = await appOpsSince(port, 5000);
    expect(ops.since).toBe(5000);
    expect(ops.cardOps.map((o) => o.lemma)).toEqual(["new"]);
    expect(ops.statusOps.map((o) => o.lemma)).toEqual(["new"]);
  });

  it("applies statuses and cards as a pull does, and counts what changed", async () => {
    const { port } = makeFakePort();
    const applied: string[] = [];
    port.applyStatusChanges = async (c) => (applied.push("statuses"), c.length);
    port.applyCardOps = async (c) => (applied.push("cards"), c.length);
    expect(await applyAppOps(port, { v: 1, since: 0, cardOps: [card(1)], statusOps: [status(1), status(2)] })).toBe(3);
    expect(applied).toEqual(["statuses", "cards"]);
    applied.length = 0;
    expect(await applyAppOps(port, { v: 1, since: 0, cardOps: [], statusOps: [] })).toBe(0);
    expect(applied).toEqual([]);
  });
});

describe("the app's daily counts (reviews and words learned)", () => {
  const counts = (reviews: number, wordsLearned = 0) => ({
    "20000": { en: { reviews, wordsLearned, native: "fr" as const } },
  });

  function store(seed: Record<string, unknown> = {}) {
    const items: Record<string, unknown> = { ...seed };
    return {
      items,
      get: async (keys: string | string[] | null) => {
        const list = keys == null ? Object.keys(items) : Array.isArray(keys) ? keys : [keys];
        return Object.fromEntries(list.filter((k) => k in items).map((k) => [k, items[k]]));
      },
      set: async (next: Record<string, unknown>) => void Object.assign(items, next),
    };
  }
  const reviewsOn = async (s: ReturnType<typeof store>) => (await loadDailyStats(s))[20000]?.en?.reviews ?? 0;

  it("carries each opening's counts, and reads only whole positive counts with a known native language", () => {
    const raw = encodeAppOps({
      since: 0,
      cardOps: [],
      statusOps: [],
      daily: {
        a: {
          "20000": {
            en: { reviews: 3, wordsLearned: 1.7, native: "fr" },
            es: { reviews: 0, wordsLearned: 0, native: "fr" },
          },
        },
        b: {
          x: { en: { reviews: 1, wordsLearned: 0, native: "fr" } },
          "20001": { en: { reviews: -2, wordsLearned: 1, native: "xx" } },
        },
      } as never,
    });
    expect(parseAppOps(raw)?.daily).toEqual({
      a: { "20000": { en: { reviews: 3, wordsLearned: 1, native: "fr" } } },
      b: {},
    });
    expect(parseAppOps(encodeAppOps({ since: 0, cardOps: [], statusOps: [] }))).not.toHaveProperty("daily");
  });

  it("measures what a review counted, and adds counts to the shared ones", () => {
    const before = {
      20000: { en: { exposures: 9, unknownSeen: 1, wordsLearned: 1, reviews: 2, native: "fr" as const } },
    };
    const after = withReviewCounts(before, counts(3, 1));
    expect(after[20000].en).toEqual({ exposures: 9, unknownSeen: 1, wordsLearned: 2, reviews: 5, native: "fr" });
    expect(withReviewCounts({}, counts(1))[20000].en.reviews).toBe(1);
    expect(reviewCountsBetween(before, after)).toEqual(counts(3, 1));
    expect(reviewCountsBetween(after, after)).toEqual({});
    expect(before[20000].en.reviews).toBe(2); // the shared counts are left as they were
  });

  it("adds an opening's counts once, however often the same payload comes back", async () => {
    const s = store();
    expect(await applyAppDaily(s, { a: counts(2) }, 1000)).toBe(2);
    expect(await applyAppDaily(s, { a: counts(2) }, 2000)).toBe(0);
    expect(await reviewsOn(s)).toBe(2);
    // The same opening went on: only what is new is added.
    expect(await applyAppDaily(s, { a: counts(5) }, 3000)).toBe(3);
    // Another opening counts on its own.
    expect(await applyAppDaily(s, { a: counts(5), b: counts(1, 1) }, 4000)).toBe(2);
    expect(await reviewsOn(s)).toBe(6);
    expect((await loadDailyStats(s))[20000].en.wordsLearned).toBe(1);
  });

  it("forgets openings not heard from for long, and reads garbage as nothing added", async () => {
    const s = store({ [APPLIED_APP_DAILY_KEY]: { old: { at: 0, counts: counts(1) }, broken: "x" } });
    await applyAppDaily(s, {}, APPLIED_APP_DAILY_TTL_MS + 1);
    expect(s.items[APPLIED_APP_DAILY_KEY]).toEqual({});
    const fresh = store({ [APPLIED_APP_DAILY_KEY]: { kept: { at: 10, counts: counts(1) } } });
    await applyAppDaily(fresh, {}, 20);
    expect(Object.keys(fresh.items[APPLIED_APP_DAILY_KEY] as object)).toEqual(["kept"]);
  });

  it("reads the shared daily counts as the statistics do", async () => {
    expect(await sharedDailyStats({ [DAILY_KEY]: { 20000: { en: { reviews: 1 } } } })).toEqual({
      20000: { en: { reviews: 1 } },
    });
    expect(await sharedDailyStats({})).toEqual({});
  });
});
