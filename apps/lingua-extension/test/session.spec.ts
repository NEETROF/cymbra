import { describe, expect, it } from "vitest";
import { localDayStart, ReviewController, SESSION_CARDS } from "@/review/session.ts";
import { type FakeCard, makeFakePort } from "./helpers.ts";

const deck: FakeCard[] = [
  { headword: "seldom", surface: "seldom", sentence: "They seldom ship.", gloss: "rarement" },
  { headword: "conundrum", surface: "conundrum", sentence: "A tricky conundrum.", gloss: "casse-tête" },
];

const clock = () => 1000;

/** `n` cards named w0, w1… */
function cards(n: number): FakeCard[] {
  return Array.from({ length: n }, (_, i) => ({ headword: `w${i}`, surface: `w${i}`, sentence: "", gloss: null }));
}

describe("ReviewController", () => {
  it("is idle before starting", () => {
    const { port } = makeFakePort(deck);
    const view = new ReviewController(port, clock).view();
    expect(view.phase).toBe("idle");
    expect(view.summary).toBeNull();
    expect(view.moreDue).toBe(false);
  });

  it("starts on the first due card, answer hidden", async () => {
    const { port } = makeFakePort(deck);
    const view = await new ReviewController(port, clock).start();
    expect(view.phase).toBe("reviewing");
    expect(view.card?.headword).toBe("seldom");
    expect(view.card?.revealed).toBe(false);
    expect(view.card?.remaining).toBe(2);
  });

  it("starts a session of ten cards with the day's allowance from the local midnight", async () => {
    const { port, calls } = makeFakePort(deck);
    await new ReviewController(port, clock).start();
    await new ReviewController(port, clock, undefined, async () => 20).start();
    expect(calls.reviewOptions).toEqual([
      { limit: SESSION_CARDS, newPerDay: 10, dayStart: localDayStart(1000) },
      { limit: SESSION_CARDS, newPerDay: 20, dayStart: localDayStart(1000) },
    ]);
  });

  it("reveals the current answer", async () => {
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, clock);
    await c.start();
    const view = await c.reveal();
    expect(view.card?.revealed).toBe(true);
    expect(calls.reveals).toBe(1);
  });

  it("brings a missed card back, then finishes done", async () => {
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, clock);
    await c.start();
    let view = await c.grade("again");
    expect(view.card?.headword).toBe("conundrum");
    view = await c.grade("good");
    expect(view.card?.headword).toBe("seldom"); // back, after the other card
    view = await c.grade("good");
    expect(view.phase).toBe("done");
    expect(view.card).toBeNull();
    expect(calls.grades).toEqual(["again", "good", "good"]);
    expect(view.summary).toEqual({ reviewed: 2, recovered: 1, holding: 0, known: 0, hidden: 0 });
  });

  it("records one review per card: a missed card's second answer is not one", async () => {
    const recorded: string[] = [];
    const { port } = makeFakePort(deck);
    const c = new ReviewController(port, clock, (event) => void recorded.push(event));
    await c.start();
    await c.grade("again");
    await c.grade("good");
    await c.grade("good");
    expect(recorded).toEqual(["review", "review"]);
  });

  it("passes the injected clock to the port and records mark-known", async () => {
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, () => 4242);
    await c.start();
    await c.markKnown();
    expect(calls.markKnown).toBe(1);
  });

  it("hides a word and moves on, recording nothing", async () => {
    const recorded: string[] = [];
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, clock, (event) => void recorded.push(event));
    await c.start();
    const view = await c.ignore();
    expect(calls.ignored).toBe(1);
    expect(view.card?.headword).toBe("conundrum");
    expect(recorded).toEqual([]);
  });

  it("starts over the languages it is given, or every language", async () => {
    const { port, calls } = makeFakePort([...deck, { ...deck[0], headword: "faro", language: "es" }]);
    const c = new ReviewController(port, clock);
    expect((await c.start(["es"])).card?.headword).toBe("faro");
    await c.start();
    expect(calls.reviewLanguages).toEqual([["es"], undefined]);
  });

  it("records each grade and each word learned in the card's language, English by default", async () => {
    const recorded: [string, string | undefined][] = [];
    const { port } = makeFakePort([{ ...deck[0], language: "es" }, deck[1]]);
    const c = new ReviewController(port, clock, (event, language) => void recorded.push([event, language]));
    await c.start();
    await c.grade("good");
    await c.markKnown();
    expect(recorded).toEqual([
      ["review", "es"],
      ["learned", "en"], // a card that does not say its language is English
    ]);
  });

  it("offers more only while another session would hold cards", async () => {
    const { port } = makeFakePort(cards(12));
    const c = new ReviewController(port, clock);
    let view = await c.start();
    expect(view.card?.remaining).toBe(SESSION_CARDS);
    for (let i = 0; i < SESSION_CARDS; i++) view = await c.grade("good");
    expect(view.phase).toBe("done");
    expect(view.summary?.reviewed).toBe(SESSION_CARDS);
    expect(view.moreDue).toBe(true);

    view = await c.start(); // « Encore 10 »
    expect(view.card?.remaining).toBe(2);
    await c.grade("good");
    view = await c.grade("good");
    expect(view.phase).toBe("done");
    expect(view.moreDue).toBe(false);
  });

  it("is done immediately when the deck is empty", async () => {
    const { port } = makeFakePort([]);
    const view = await new ReviewController(port, clock).start();
    expect(view.phase).toBe("done");
    expect(view.card).toBeNull();
    expect(view.moreDue).toBe(false);
  });
});

describe("localDayStart", () => {
  it("is the reader's local midnight, whatever the hour", () => {
    const afternoon = new Date(2026, 9, 4, 15, 30, 12).getTime() / 1000;
    const midnight = new Date(2026, 9, 4).getTime() / 1000;
    expect(localDayStart(afternoon)).toBe(midnight);
    expect(localDayStart(midnight)).toBe(midnight);
  });
});
