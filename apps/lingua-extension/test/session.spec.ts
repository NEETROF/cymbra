import { describe, expect, it } from "vitest";
import { ReviewController } from "@/review/session.ts";
import { type FakeCard, makeFakePort } from "./helpers.ts";

const deck: FakeCard[] = [
  { headword: "seldom", surface: "seldom", sentence: "They seldom ship.", gloss: "rarement" },
  { headword: "conundrum", surface: "conundrum", sentence: "A tricky conundrum.", gloss: "casse-tête" },
];

const clock = () => 1000;

describe("ReviewController", () => {
  it("is idle before starting", () => {
    const { port } = makeFakePort(deck);
    expect(new ReviewController(port, clock).view().phase).toBe("idle");
  });

  it("starts on the first due card, answer hidden", async () => {
    const { port } = makeFakePort(deck);
    const view = await new ReviewController(port, clock).start();
    expect(view.phase).toBe("reviewing");
    expect(view.card?.headword).toBe("seldom");
    expect(view.card?.revealed).toBe(false);
    expect(view.card?.remaining).toBe(2);
  });

  it("reveals the current answer", async () => {
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, clock);
    await c.start();
    const view = await c.reveal();
    expect(view.card?.revealed).toBe(true);
    expect(calls.reveals).toBe(1);
  });

  it("advances through the queue on grade and finishes done", async () => {
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, clock);
    await c.start();
    let view = await c.grade("good");
    expect(view.card?.headword).toBe("conundrum");
    expect(view.card?.remaining).toBe(1);
    view = await c.grade("again");
    expect(view.phase).toBe("done");
    expect(view.card).toBeNull();
    expect(calls.grades).toEqual(["good", "again"]);
  });

  it("passes the injected clock to the port and records mark-known", async () => {
    const { port, calls } = makeFakePort(deck);
    const c = new ReviewController(port, () => 4242);
    await c.start();
    await c.markKnown();
    expect(calls.markKnown).toBe(1);
  });

  it("is done immediately when the deck is empty", async () => {
    const { port } = makeFakePort([]);
    const view = await new ReviewController(port, clock).start();
    expect(view.phase).toBe("done");
    expect(view.card).toBeNull();
  });
});
