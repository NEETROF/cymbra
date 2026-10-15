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

  it("counts each card's showings: a missed card says its second and third time (add-lingua-review-repeat-hint)", async () => {
    const { port } = makeFakePort(cards(5));
    const c = new ReviewController(port, clock);
    let view = await c.start();
    expect([view.card?.headword, view.showing]).toEqual(["w0", 1]);
    // A reveal shows the same card: not another showing.
    view = await c.reveal();
    expect(view.showing).toBe(1);
    view = await c.grade("again"); // w0 comes back after three other cards
    for (const w of ["w1", "w2", "w3"]) {
      expect([view.card?.headword, view.showing]).toEqual([w, 1]);
      view = await c.grade("good");
    }
    expect([view.card?.headword, view.showing]).toEqual(["w0", 2]);
    view = await c.grade("again"); // back after w4, the one card left
    expect([view.card?.headword, view.showing]).toEqual(["w4", 1]);
    view = await c.grade("good");
    expect([view.card?.headword, view.showing]).toEqual(["w0", 3]);
    view = await c.grade("again");
    expect(view.phase).toBe("done");
    expect(view.showing).toBe(0);
  });

  it("never shows the session's last card again at once when it is missed (add-lingua-review-repeat-hint D4)", async () => {
    const { port } = makeFakePort(deck);
    const c = new ReviewController(port, clock);
    await c.start();
    await c.grade("good");
    // `conundrum`, the last card, missed: the session ends rather than show it again in a row.
    const view = await c.grade("again");
    expect(view.phase).toBe("done");
    expect(view.summary).toEqual({ reviewed: 2, recovered: 0, holding: 0, known: 0, hidden: 0 });
  });

  it("starts each session's count afresh", async () => {
    const { port } = makeFakePort(deck);
    const c = new ReviewController(port, clock);
    expect((await c.start()).showing).toBe(1);
    // Left unanswered, the same card opens the next session: its first showing there.
    const view = await c.start();
    expect([view.card?.headword, view.showing]).toEqual(["seldom", 1]);
  });

  it("keeps two languages' same headword apart", async () => {
    const { port } = makeFakePort([
      { headword: "real", surface: "real", sentence: "", gloss: null, language: "en" },
      { headword: "real", surface: "real", sentence: "", gloss: null, language: "es" },
    ]);
    const c = new ReviewController(port, clock);
    let view = await c.start();
    expect([view.card?.language, view.showing]).toEqual(["en", 1]);
    view = await c.grade("good");
    expect([view.card?.language, view.showing]).toEqual(["es", 1]);
  });

  it("offers practice once nothing is due, over the cards already reviewed (add-lingua-review-practice)", async () => {
    const recorded: string[] = [];
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, clock, (event) => void recorded.push(event));
    await c.start();
    await c.grade("good");
    let view = await c.grade("good");
    // The review is over and nothing is due: practice is offered, not « Encore 10 ».
    expect([view.phase, view.moreDue, view.practiceAvailable, view.practice]).toEqual(["done", false, true, false]);
    view = await c.start(undefined, true);
    expect([view.phase, view.practice, view.card?.headword]).toEqual(["reviewing", true, "seldom"]);
    expect(calls.reviewOptions.at(-1)).toEqual({ limit: SESSION_CARDS, practice: true });
    await c.grade("good");
    view = await c.grade("good");
    // Practice answers count in the day; its end offers practice again, never « Encore 10 ».
    expect(recorded).toEqual(["review", "review", "review", "review"]);
    expect([view.phase, view.moreDue, view.practiceAvailable]).toEqual(["done", false, true]);
  });

  it("offers no practice while cards are due, nor without a reviewed card", async () => {
    const { port } = makeFakePort(cards(12));
    const c = new ReviewController(port, clock);
    await c.start();
    let view = c.view();
    for (let i = 0; i < SESSION_CARDS; i++) view = await c.grade("good");
    expect([view.moreDue, view.practiceAvailable]).toEqual([true, false]);

    const empty = new ReviewController(makeFakePort(deck).port, clock);
    await empty.start();
    await empty.markKnown();
    view = await empty.markKnown();
    // Every card retired: nothing due, and nothing to practise.
    expect([view.phase, view.moreDue, view.practiceAvailable]).toEqual(["done", false, false]);
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

  describe("ending a session in progress (add-lingua-review-stop-session)", () => {
    it("keeps the answers given, leaves the cards not shown due and untouched, and offers more", async () => {
      const recorded: string[] = [];
      const { port, calls } = makeFakePort(cards(12));
      const c = new ReviewController(port, clock, (event) => void recorded.push(event));
      await c.start();
      await c.grade("good");
      await c.grade("again"); // w1 waits to come back after three other cards
      await c.grade("good");
      const view = await c.end();
      expect([view.phase, view.card, view.ended, view.showing]).toEqual(["done", null, true, 0]);
      expect(view.summary).toEqual({ reviewed: 3, recovered: 0, holding: 0, known: 0, hidden: 0 });
      // Ending answers nothing and counts nothing.
      expect(calls.grades).toEqual(["good", "again", "good"]);
      expect(recorded).toEqual(["review", "review", "review"]);
      expect(view.moreDue).toBe(true);
      // « Encore 10 »: the cards not shown in the ended session, a first showing each, w1 not among them.
      const next = await c.start();
      expect([next.card?.headword, next.showing, next.ended, next.card?.remaining]).toEqual(["w3", 1, false, 9]);
    });

    it("never asks again a missed card waiting to come back", async () => {
      const { port, calls } = makeFakePort(cards(5));
      const c = new ReviewController(port, clock);
      await c.start();
      await c.grade("again");
      const view = await c.end();
      expect(view.phase).toBe("done");
      expect(view.summary?.reviewed).toBe(1);
      expect(calls.grades).toEqual(["again"]);
      // Its miss stands; the next session does not hold it again.
      const next = await c.start();
      expect(next.card?.headword).toBe("w1");
    });

    it("leaves a revealed card unanswered", async () => {
      const { port, calls } = makeFakePort(deck);
      const c = new ReviewController(port, clock);
      await c.start();
      await c.reveal();
      const view = await c.end();
      expect([view.phase, view.ended, view.moreDue]).toEqual(["done", true, true]);
      expect(view.summary).toEqual({ reviewed: 0, recovered: 0, holding: 0, known: 0, hidden: 0 });
      expect(calls.grades).toEqual([]);
      // Still due: the next session opens on it, answer hidden again.
      const next = await c.start();
      expect([next.card?.headword, next.card?.revealed]).toEqual(["seldom", false]);
    });

    it("keeps a word marked known in the session", async () => {
      const { port } = makeFakePort(deck);
      const c = new ReviewController(port, clock);
      await c.start();
      await c.markKnown();
      const view = await c.end();
      expect(view.summary?.known).toBe(1);
      expect((await c.start()).card?.headword).toBe("conundrum");
    });

    it("ends a practice session the same way, offering practice again", async () => {
      const { port } = makeFakePort(deck);
      const c = new ReviewController(port, clock);
      await c.start();
      await c.grade("good");
      await c.grade("good");
      await c.start(undefined, true);
      await c.grade("good");
      const view = await c.end();
      expect([view.phase, view.practice, view.ended, view.moreDue, view.practiceAvailable]).toEqual([
        "done",
        true,
        true,
        false,
        true,
      ]);
      expect(view.summary?.reviewed).toBe(1);
    });

    it("does nothing outside a session", async () => {
      const { port, calls } = makeFakePort(deck);
      const c = new ReviewController(port, clock);
      expect((await c.end()).phase).toBe("idle");
      expect(calls.reviewOptions).toEqual([]);
      await c.start();
      await c.grade("good");
      const done = await c.grade("good");
      const starts = calls.reviewOptions.length;
      const again = await c.end();
      expect([again.phase, again.ended]).toEqual([done.phase, false]);
      expect(calls.reviewOptions).toHaveLength(starts);
    });
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
