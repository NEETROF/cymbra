import { describe, expect, it } from "vitest";
import { createCard, type WordPopupContent } from "@/reading/wordpopup.ts";
import { PAGE_CHARS } from "@/reading/gloss-pages.ts";
import { ROW_SENSE_CHARS, rowGloss } from "@/reading/selection-card.ts";

// keep-lingua-expression-glosses-whole, lingua-browser-extension *An expression's card shows its whole
// gloss, paged*: an expression's gloss is no longer cut by the pack, so its card pages it as a word's
// long gloss is paged (`renderPackAnswer` → `renderGloss`), and a row still shows its first sense
// within 80 characters. Nothing of the extension changed: these pin what it already did.

const content = (over: Partial<WordPopupContent>): WordPopupContent => ({
  headword: "sin embargo",
  surface: "sin embargo",
  gloss: null,
  rarity: "Peu fréquent — au-delà de ton niveau.",
  sentence: "Sin embargo, no vino.",
  rect: { left: 40, top: 60, bottom: 80 },
  expression: true,
  expressionAnswer: true,
  ...over,
});

function shown(over: Partial<WordPopupContent>, gestures: Array<{ gloss: string | null }> = []) {
  const card = createCard();
  document.body.append(card.el);
  card.show(content(over), (g) => gestures.push(g));
  return card;
}

const pageOf = (card: ReturnType<typeof createCard>) => ({
  text: card.el.querySelector(".gloss-page")?.textContent ?? null,
  count: card.el.querySelector(".page-count")?.textContent ?? null,
  next: card.el.querySelector<HTMLButtonElement>('.gloss-nav button[aria-label="Sens suivants"]'),
});

// Three senses of 100 characters: no two fit one 160-character page, so each has its own.
const SENSES = ["a", "b", "c"].map((letter) => `${letter.repeat(9)} `.repeat(10).trim() + "!".repeat(1));
const LONG = SENSES.join("; ");

describe("an expression's card shows its whole gloss, paged", () => {
  it("the senses are 100 characters each, 300 together", () => {
    expect(SENSES.map((s) => s.length)).toEqual([100, 100, 100]);
    expect(SENSES.join("").length).toBe(300);
    expect(100 + "; ".length + 100).toBeGreaterThan(PAGE_CHARS);
  });

  it("An expression longer than the old cut: one page, no paging control", () => {
    const whole = "however, notwithstanding, nevertheless, in spite of, nonetheless";
    const card = shown({ gloss: whole });
    expect(pageOf(card).text).toBe(whole);
    expect(card.el.querySelector(".gloss-nav")).toBeNull();
  });

  it("An expression longer than a page: page 1 of its pages, whole senses each", () => {
    const card = shown({ headword: "a bit of", surface: "a bit of", gloss: LONG });
    let page = pageOf(card);
    expect(page.count).toBe("1/3");
    expect(page.text).toBe(SENSES[0]);
    expect(page.next!.disabled).toBe(false);
    page.next!.click();
    page = pageOf(card);
    expect(page.count).toBe("2/3");
    expect(page.text).toBe(SENSES[1]);
    page.next!.click();
    expect(pageOf(card).text).toBe(SENSES[2]);
    expect(pageOf(card).next!.disabled).toBe(true);
  });

  it("Adding a paged expression to the deck stores its first page", () => {
    const gestures: Array<{ gloss: string | null }> = [];
    const card = shown({ headword: "a bit of", surface: "a bit of", gloss: LONG }, gestures);
    pageOf(card).next!.click();
    [...card.el.querySelectorAll("button")].find((b) => b.textContent === "+ Deck")!.click();
    expect(gestures[0]!.gloss).toBe(SENSES[0]);
  });
});

describe("an expression's row", () => {
  it("shows a 95-character first sense cut after a whole word within 80 characters, with « … »", () => {
    const first = "to keep in mind, to bear in mind, to take into account, to consider carefully and in good faith";
    expect(first.length).toBe(95);
    const row = rowGloss(`${first}; to heed`)!;
    expect(row.length).toBeLessThanOrEqual(ROW_SENSE_CHARS);
    expect(row.endsWith("…")).toBe(true);
    expect(first.startsWith(row.slice(0, -1).trimEnd())).toBe(true);
    expect(first[row.length - 1]).toBe(" ");
  });
});
