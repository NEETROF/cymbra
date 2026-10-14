import { describe, expect, it } from "vitest";
import {
  afterSession,
  answer,
  INITIAL_RATING,
  loadRating,
  localDay,
  markShown,
  RATING_KEY,
  ratingDue,
  ratingStateOf,
  type RatingState,
  saveRating,
} from "@/review/rating.ts";
import { memoryArea } from "./helpers.ts";

// When the end of a session asks for a rating (add-lingua-comeback-hints D10): from the fifth
// finished session, on the third day of review or later; « Plus tard » or no answer asks again ten
// sessions later; twice at most; « Noter » and « Ne plus demander » end it.

/** `n` sessions finished, each on the day given (the days cycle). */
function finish(state: RatingState, n: number, days: string[]): RatingState {
  let s = state;
  for (let i = 0; i < n; i++) s = afterSession(s, days[i % days.length]);
  return s;
}

/** Sessions finished one after another, the request shown whenever due: when it was. */
function showings(start: RatingState, sessions: number, day = "2026-10-20"): number[] {
  let s = start;
  const shown: number[] = [];
  for (let i = 0; i < sessions; i++) {
    s = afterSession(s, day);
    if (ratingDue(s)) {
      shown.push(s.sessions);
      s = markShown(s);
    }
  }
  return shown;
}

describe("the rating request", () => {
  it("Five sessions in one evening: no request, it waits for the third day", () => {
    const evening = finish(INITIAL_RATING, 5, ["2026-10-14"]);
    expect(evening.days).toBe(1);
    expect(ratingDue(evening)).toBe(false);
    const secondDay = afterSession(evening, "2026-10-15");
    expect(ratingDue(secondDay)).toBe(false);
    expect(ratingDue(afterSession(secondDay, "2026-10-17"))).toBe(true);
  });

  it("The fifth session, on the third day: shown — not before the fifth", () => {
    const four = finish(INITIAL_RATING, 4, ["2026-10-12", "2026-10-13", "2026-10-14", "2026-10-14"]);
    expect(four.days).toBe(3);
    expect(ratingDue(four)).toBe(false);
    expect(ratingDue(afterSession(four, "2026-10-14"))).toBe(true);
  });

  it("Later, or no answer: again after ten more sessions, and never a third time", () => {
    const ready = finish(INITIAL_RATING, 4, ["2026-10-12", "2026-10-13", "2026-10-14"]);
    expect(showings(ready, 40)).toEqual([5, 15]);
    // « Plus tard » leaves the state as the showing left it.
    const shown = markShown(afterSession(ready, "2026-10-14"));
    expect(answer(shown, "later")).toEqual(shown);
  });

  it("Rating ends it, and so does « Ne plus demander »", () => {
    const shown = markShown(finish(INITIAL_RATING, 5, ["2026-10-12", "2026-10-13", "2026-10-14"]));
    for (const choice of ["rate", "never"] as const) {
      const ended = answer(shown, choice);
      expect(ended.answer).toBe(choice === "rate" ? "rated" : "never");
      expect(showings(ended, 30)).toEqual([]);
    }
  });

  it("counts a distinct day once, whatever the number of sessions on it", () => {
    const s = finish(INITIAL_RATING, 3, ["2026-10-12", "2026-10-12", "2026-10-13"]);
    expect(s).toMatchObject({ sessions: 3, days: 2, lastDay: "2026-10-13" });
  });

  it("a local day is the reader's calendar date", () => {
    const noon = new Date(2026, 9, 14, 12, 0, 0).getTime() / 1000;
    expect(localDay(noon)).toBe("2026-10-14");
  });

  it("is kept on the device, and a malformed state reads as never asked", async () => {
    const area = memoryArea();
    expect(await loadRating(area)).toEqual(INITIAL_RATING);
    const state = markShown(finish(INITIAL_RATING, 5, ["2026-10-12", "2026-10-13", "2026-10-14"]));
    await saveRating(area, state);
    expect(area.raw[RATING_KEY]).toEqual(state);
    expect(await loadRating(area)).toEqual(state);
    expect(ratingStateOf({ sessions: -1, days: "3", answer: "maybe", lastDay: 4 })).toEqual(INITIAL_RATING);
  });
});
