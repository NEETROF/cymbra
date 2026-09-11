import { beforeEach, describe, expect, it, vi } from "vitest";
import { createCard, type Gesture, type WordPopupContent } from "@/reading/wordpopup.ts";

beforeEach(() => {
  document.body.innerHTML = "";
});

const content = (over: Partial<WordPopupContent> = {}): WordPopupContent => ({
  headword: "seldom",
  surface: "Seldom",
  gloss: "rarement",
  rarity: "Peu fréquent — au-delà de tes 3 000 mots les plus courants.",
  sentence: "They seldom ship on Friday.",
  rect: { left: 40, bottom: 80 },
  ...over,
});

function button(card: HTMLElement, label: string): HTMLButtonElement {
  const b = [...card.querySelectorAll("button")].find((el) => el.textContent === label);
  if (!b) throw new Error(`no button "${label}"`);
  return b as HTMLButtonElement;
}

describe("word popup card", () => {
  it("shows the dictionary form and the gloss, and the 'forme vue' line when the surface differs", () => {
    const card = createCard();
    document.body.append(card.el);
    card.show(content({ headword: "run", surface: "running", gloss: "courir" }), () => {});
    expect(card.visible()).toBe(true);
    expect(card.el.querySelector(".headword")!.textContent).toBe("run");
    expect(card.el.querySelector(".gloss")!.textContent).toBe("courir");
    expect(card.el.querySelector(".seen")!.textContent).toContain("forme vue");
    expect(card.el.querySelector(".seen")!.textContent).toContain("running");
  });

  it("hides the 'forme vue' line when the surface differs only by case", () => {
    const card = createCard();
    card.show(content({ headword: "seldom", surface: "Seldom" }), () => {});
    expect((card.el.querySelector(".seen") as HTMLElement).hidden).toBe(true);
  });

  it("emits a learning gesture with the source sentence on '+ Deck'", () => {
    const card = createCard();
    const spy = vi.fn<(g: Gesture) => void>();
    card.show(content(), spy);
    button(card.el, "+ Deck").click();
    expect(spy).toHaveBeenCalledWith({
      lemma: "seldom",
      surface: "Seldom",
      sentence: "They seldom ship on Friday.",
      status: "learning",
    });
    expect(card.visible()).toBe(false); // hides after a gesture
  });

  it("offers Je connais / + Deck / Ignorer for a word, and hides Je connais for an expression", () => {
    const word = createCard();
    word.show(content(), () => {});
    expect([...word.el.querySelectorAll(".actions button")].map((b) => b.textContent)).toEqual([
      "Je connais",
      "+ Deck",
      "Ignorer",
    ]);

    const phrase = createCard();
    phrase.show(content({ headword: "ship on friday", expression: true }), () => {});
    expect([...phrase.el.querySelectorAll(".actions button")].map((b) => b.textContent)).toEqual(["+ Deck", "Ignorer"]);
  });

  it("dismisses on the ✕ close button", () => {
    const card = createCard();
    card.show(content(), () => {});
    expect(card.visible()).toBe(true);
    (card.el.querySelector(".close") as HTMLButtonElement).click();
    expect(card.visible()).toBe(false);
  });

  it("falls back to a 'no translation' note when the pack has no gloss", () => {
    const card = createCard();
    card.show(content({ gloss: null }), () => {});
    const gloss = card.el.querySelector(".gloss")!;
    expect(gloss.classList.contains("empty")).toBe(true);
    expect(gloss.textContent).toContain("Pas de traduction");
  });

  it("never renders the word 'lemma'/'lemme' anywhere in the card", () => {
    const card = createCard();
    card.show(content(), () => {});
    expect(card.el.textContent ?? "").not.toMatch(/lemm/i);
  });
});
