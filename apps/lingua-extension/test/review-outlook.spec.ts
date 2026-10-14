import { afterEach, describe, expect, it, vi } from "vitest";
import { reviewOutlookJson } from "@/analyzer/engine.ts";
import { localDayStart } from "@/review/session.ts";
import {
  cardsOf,
  countAt,
  countsAt,
  languageCountAt,
  loadOutlook,
  OUTLOOK_KEY,
  OUTLOOK_VERSION,
  outlookOf,
  parseOutlook,
  type ReviewOutlook,
  UPCOMING_MAX,
  usedUp,
} from "@/state/review-outlook.ts";
import { memoryArea } from "./helpers.ts";
import { realGlue, realPort } from "./real-wasm.ts";

// The outlook the toolbar icon and the popup count from (add-lingua-comeback-hints D2): read in
// TypeScript, held to the engine's own count of what a review would hold on the same deck.

const HOUR = 3_600;
const DAY = 86_400;

function outlook(over: Partial<ReviewOutlook> = {}): ReviewOutlook {
  return {
    version: OUTLOOK_VERSION,
    computedAt: 1_000 * DAY + HOUR,
    dayStart: 1_000 * DAY,
    languages: [
      { language: "en", cards: 60, due: 3, upcoming: [1_000 * DAY + 5 * HOUR, 1_001 * DAY + HOUR], fresh: 40, introduced: 4 },
      { language: "es", cards: 9, due: 1, upcoming: [], fresh: 2, introduced: 0 },
    ],
    ...over,
  };
}

describe("reading an outlook", () => {
  it("keeps one of this version and refuses anything else", () => {
    const good = outlook();
    expect(outlookOf(JSON.parse(JSON.stringify(good)))).toEqual(good);
    expect(parseOutlook(JSON.stringify(good))).toEqual(good);
    expect(outlookOf({ ...good, version: OUTLOOK_VERSION + 1 })).toBeNull();
    expect(outlookOf({ ...good, languages: [{ language: "en" }] })).toBeNull();
    expect(outlookOf({ ...good, languages: "en" })).toBeNull();
    expect(outlookOf({ ...good, computedAt: "noon" })).toBeNull();
    expect(outlookOf(undefined)).toBeNull();
    expect(parseOutlook("{ not json")).toBeNull();
  });

  it("is read from the preferences, null when none was worked out", async () => {
    expect(await loadOutlook(memoryArea())).toBeNull();
    expect(await loadOutlook(memoryArea({ [OUTLOOK_KEY]: outlook() }))).toEqual(outlook());
  });

  it("names each language's cards, 0 for one it does not hold", () => {
    expect(cardsOf(outlook(), "es")).toBe(9);
    expect(cardsOf(outlook(), "fr")).toBe(0);
  });
});

describe("counting from an outlook", () => {
  const o = outlook();
  const today = o.dayStart;

  it("counts the cards due, those come due since, and the new words left of today's allowance", () => {
    // 3 due + 0 come due + min(40, 10 − 4).
    expect(languageCountAt(o.languages[0], today, today + 2 * HOUR, today, 10)).toBe(9);
    // Past the first upcoming time.
    expect(languageCountAt(o.languages[0], today, today + 6 * HOUR, today, 10)).toBe(10);
    // An allowance already spent today: no new word.
    expect(languageCountAt(o.languages[0], today, today + 2 * HOUR, today, 4)).toBe(3);
  });

  it("on the next local day, the allowance is whole again", () => {
    const tomorrow = today + DAY;
    expect(languageCountAt(o.languages[0], today, tomorrow + 2 * HOUR, tomorrow, 10)).toBe(3 + 2 + 10);
  });

  it("sums the languages asked, every one when none is named", () => {
    const at = today + 2 * HOUR;
    expect(countsAt(o, at, today, 10)).toEqual([
      { language: "en", count: 9 },
      { language: "es", count: 3 },
    ]);
    expect(countAt(o, [], at, today, 10)).toBe(12);
    expect(countAt(o, ["es"], at, today, 10)).toBe(3);
  });

  it("says when a cut list of upcoming times is used up, never for a list that was not cut", () => {
    const many = Array.from({ length: UPCOMING_MAX }, (_, i) => today + (i + 1) * 60);
    const cut = outlook({ languages: [{ language: "en", cards: 130, due: 0, upcoming: many, fresh: 0, introduced: 0 }] });
    expect(usedUp(cut, today + 99 * 60)).toBe(false);
    expect(usedUp(cut, today + 100 * 60)).toBe(true);
    expect(usedUp(o, today + 10 * DAY)).toBe(false);
  });
});

describe("the outlook against the engine (add-lingua-comeback-hints D2)", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("counts what the engine's reviewableCount counts, now, later the same day and the next day", async () => {
    const port = realPort();
    const day = localDayStart(1_790_000_000);
    const morning = day + 8 * HOUR;
    const english = port.for("en");
    for (let i = 0; i < 24; i++) {
      await english.addCard({
        lemma: `word${i}`,
        surface: `word${i}`,
        sentence: "",
        url: "",
        gloss: null,
        capturedAt: day - DAY + i,
      });
    }
    // A morning session: new words answered every way, so their due times spread over the days.
    const allowance = { newPerDay: 10, dayStart: day };
    await port.startReview(morning, ["en"], { limit: 8, ...allowance });
    const ratings = ["again", "hard", "good", "easy"] as const;
    for (let i = 0; (await port.reviewCurrent()) !== null; i++) {
      await port.reviewReveal();
      await port.reviewGrade(ratings[i % ratings.length], morning);
    }

    const computed = morning + 60;
    const json = await reviewOutlookJson(await port.backup(), computed, day, async () => realGlue());
    const read = parseOutlook(json)!;
    expect(read.languages[0].introduced).toBeGreaterThan(0);

    for (const at of [computed, day + 20 * HOUR, day + DAY + 9 * HOUR, day + 4 * DAY]) {
      const dayStart = localDayStart(at);
      for (const perDay of [5, 10, 20]) {
        expect(countAt(read, ["en"], at, dayStart, perDay), `at ${at - day} s, ${perDay} a day`).toBe(
          await port.reviewableCount(at, ["en"], { newPerDay: perDay, dayStart }),
        );
      }
    }
  });
});
