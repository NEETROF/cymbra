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
  rect: { left: 40, top: 60, bottom: 80 },
  ...over,
});

/** Stub the card's measured box so positioning can be tested without real layout. */
function stubSize(card: { el: HTMLElement }, width: number, height: number): void {
  card.el.getBoundingClientRect = () =>
    ({ width, height, top: 0, left: 0, right: width, bottom: height, x: 0, y: 0, toJSON() {} }) as DOMRect;
}

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

  it("offers reclassify actions for a KNOWN word (no 'Je connais', no clear — a Known may be presumed)", () => {
    const card = createCard();
    card.show(content({ status: "known" }), () => {});
    expect([...card.el.querySelectorAll(".actions button")].map((b) => b.textContent)).toEqual(["+ Deck", "Ignorer"]);
  });

  it("offers reclassify actions for an IGNORED word (no 'Ignorer', adds 'Remettre à apprendre')", () => {
    const card = createCard();
    card.show(content({ status: "ignored" }), () => {});
    expect([...card.el.querySelectorAll(".actions button")].map((b) => b.textContent)).toEqual([
      "Je connais",
      "+ Deck",
      "Remettre à apprendre",
    ]);
  });

  it("hides '+ Deck' for a LEARNING word and offers no clear (already 'à apprendre')", () => {
    const card = createCard();
    card.show(content({ status: "learning" }), () => {});
    expect([...card.el.querySelectorAll(".actions button")].map((b) => b.textContent)).toEqual([
      "Je connais",
      "Ignorer",
    ]);
  });

  it("emits a null status (clear → 'à apprendre') on 'Remettre à apprendre'", () => {
    const card = createCard();
    const spy = vi.fn<(g: Gesture) => void>();
    card.show(content({ status: "ignored" }), spy);
    button(card.el, "Remettre à apprendre").click();
    expect(spy).toHaveBeenCalledWith({
      lemma: "seldom",
      surface: "Seldom",
      sentence: "They seldom ship on Friday.",
      status: null,
    });
    expect(card.visible()).toBe(false);
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

describe("word popup positioning", () => {
  const H = 200; // pretend viewport height
  beforeEach(() => {
    Object.defineProperty(window, "innerHeight", { value: H, configurable: true });
    Object.defineProperty(window, "innerWidth", { value: 1000, configurable: true });
  });

  it("places the card just below the word when there is room", () => {
    const card = createCard();
    document.body.append(card.el);
    stubSize(card, 260, 100);
    card.show(content({ rect: { left: 40, top: 20, bottom: 40 } }), () => {});
    expect(card.el.style.top).toBe("48px"); // bottom(40) + 8
    expect(card.el.style.left).toBe("40px");
  });

  it("flips the card above the word when there is no room below (near the page bottom)", () => {
    const card = createCard();
    document.body.append(card.el);
    stubSize(card, 260, 120);
    // Word near the bottom: below (170+8=178)+120=298 > 192 → flip above: top(150)-8-120=22.
    card.show(content({ rect: { left: 40, top: 150, bottom: 170 } }), () => {});
    expect(card.el.style.top).toBe("22px");
  });

  it("clamps into the viewport so the card is never partially off-screen", () => {
    const card = createCard();
    document.body.append(card.el);
    stubSize(card, 260, 120);
    // Even flipping above would overflow the top (top 20 → 20-8-120 = -108) → clamp to 8.
    card.show(content({ rect: { left: 40, top: 20, bottom: 190 } }), () => {});
    expect(card.el.style.top).toBe("8px");
  });

  it("leaves room for the platform's selection callout when it flips above on touch", () => {
    // No room below (150+8+60 = 218 > 192) → flip above. On a mouse: 130-8-60 = 62.
    // On a touch device the platform draws its Copier/Rechercher bar just above the
    // selection, so the card clears it: 130-8-44-60 = 18.
    const place = (coarse: boolean): string => {
      vi.stubGlobal("matchMedia", () => ({ matches: coarse }) as MediaQueryList);
      const card = createCard();
      document.body.append(card.el);
      stubSize(card, 260, 60);
      card.show(content({ rect: { left: 40, top: 130, bottom: 150 } }), () => {});
      return card.el.style.top;
    };
    expect(place(false)).toBe("62px");
    expect(place(true)).toBe("18px");
    vi.unstubAllGlobals();
  });

  it("clamps the card's left edge within the viewport width", () => {
    const card = createCard();
    document.body.append(card.el);
    stubSize(card, 260, 100);
    card.show(content({ rect: { left: 5000, top: 20, bottom: 40 } }), () => {});
    expect(card.el.style.left).toBe(`${1000 - 260 - 8}px`); // vw - width - 8
  });
});
