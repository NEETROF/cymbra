import { describe, expect, it } from "vitest";
import { defaultState } from "@/state/storage.ts";
import { applyStatus, deck, knownCount, resetState } from "@/state/status.ts";

const ctx = (sentence: string, createdAt: number) => ({ surface: sentence.split(" ")[0]!, sentence, createdAt });

describe("applyStatus", () => {
  it("'+ Deck' (learning) adds the form and creates a card with its sentence", () => {
    const s = applyStatus(defaultState(), "seldom", "learning", ctx("They seldom ship.", 100));
    expect(s.statuses.seldom).toBe("learning");
    expect(s.cards.seldom).toEqual({ lemma: "seldom", surface: "They", sentence: "They seldom ship.", createdAt: 100 });
  });

  it("does not create a card when no context is supplied", () => {
    const s = applyStatus(defaultState(), "seldom", "learning");
    expect(s.statuses.seldom).toBe("learning");
    expect(s.cards.seldom).toBeUndefined();
  });

  it("keeps the first card when learning is applied twice", () => {
    const s1 = applyStatus(defaultState(), "seldom", "learning", ctx("First sentence.", 1));
    const s2 = applyStatus(s1, "seldom", "learning", ctx("Second sentence.", 2));
    expect(s2.cards.seldom!.sentence).toBe("First sentence.");
  });

  it("'known' and 'ignored' drop any deck card", () => {
    const learning = applyStatus(defaultState(), "seldom", "learning", ctx("They seldom ship.", 1));
    const known = applyStatus(learning, "seldom", "known");
    expect(known.statuses.seldom).toBe("known");
    expect(known.cards.seldom).toBeUndefined();
    const ignored = applyStatus(learning, "seldom", "ignored");
    expect(ignored.cards.seldom).toBeUndefined();
  });

  it("null clears both the status and the card", () => {
    const learning = applyStatus(defaultState(), "seldom", "learning", ctx("They seldom ship.", 1));
    const cleared = applyStatus(learning, "seldom", null);
    expect(cleared.statuses.seldom).toBeUndefined();
    expect(cleared.cards.seldom).toBeUndefined();
  });

  it("does not mutate the input state", () => {
    const base = defaultState();
    applyStatus(base, "seldom", "learning", ctx("x y", 1));
    expect(base.statuses).toEqual({});
    expect(base.cards).toEqual({});
  });
});

describe("deck / knownCount / resetState", () => {
  it("deck lists learning cards oldest-first", () => {
    let s = applyStatus(defaultState(), "beta", "learning", ctx("b", 200));
    s = applyStatus(s, "alpha", "learning", ctx("a", 100));
    expect(deck(s).map((c) => c.lemma)).toEqual(["alpha", "beta"]);
  });

  it("knownCount counts only known statuses", () => {
    let s = applyStatus(defaultState(), "a", "known");
    s = applyStatus(s, "b", "ignored");
    s = applyStatus(s, "c", "learning", ctx("c", 1));
    expect(knownCount(s)).toBe(1);
  });

  it("resetState returns fresh defaults", () => {
    expect(resetState()).toEqual(defaultState());
  });
});
