import { beforeEach, describe, expect, it, vi } from "vitest";
import { createCard, type Gesture, WordPopup, type WordPopupContent } from "@/reading/wordpopup.ts";
import { createSpeaker, type VoiceInfo } from "@/reading/speech.ts";
import { makeFakeSpeech } from "./helpers.ts";

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

  it("hides the frequency line while the card has none (add-lingua-card-frequency)", () => {
    const card = createCard();
    const rarity = () => card.el.querySelector(".rarity") as HTMLElement;
    card.show(content({ rarity: "" }), () => {});
    expect(rarity().hidden).toBe(true);
    card.show(content({ rarity: "Très courant — parmi les 100 mots les plus fréquents." }), () => {});
    expect(rarity().hidden).toBe(false);
    expect(rarity().textContent).toBe("Très courant — parmi les 100 mots les plus fréquents.");
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
      expression: false,
      gloss: "rarement",
    });
    expect(card.visible()).toBe(false); // hides after a gesture
  });

  it("marks the gesture of an expression card as an expression, and a word card's as not", () => {
    const spy = vi.fn<(g: Gesture) => void>();
    const phrase = createCard();
    phrase.show(content({ headword: "ship on friday", surface: "ship on Friday", expression: true }), spy);
    button(phrase.el, "+ Deck").click();
    expect(spy).toHaveBeenLastCalledWith(expect.objectContaining({ lemma: "ship on friday", expression: true }));

    const word = createCard();
    word.show(content(), spy);
    button(word.el, "Ignorer").click();
    expect(spy).toHaveBeenLastCalledWith(expect.objectContaining({ lemma: "seldom", expression: false }));
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

  it("carries the answer on the gesture, so an expression's gloss reaches the card it creates", () => {
    const onGesture = vi.fn();
    const view = createCard();
    // A card keyed by an expression: the single-lemma pack port could not answer `give up`,
    // so the gloss travels with the gesture.
    view.show(content({ headword: "give up", surface: "gave up", gloss: "Abandonner" }), onGesture);
    const deck = [...view.el.querySelectorAll<HTMLButtonElement>(".actions button")].find(
      (b) => b.textContent === "+ Deck",
    )!;
    deck.click();
    expect(onGesture).toHaveBeenCalledWith(expect.objectContaining({ lemma: "give up", gloss: "Abandonner" }));
  });

  it("says on the gesture that the expression table answered, whatever the name's spelling (add-lingua-french-word-card D8)", () => {
    const onGesture = vi.fn();
    const view = createCard();
    view.show(
      content({ headword: "d'abord", surface: "D\u2019abord", gloss: "first, at first", expressionAnswer: true }),
      onGesture,
    );
    button(view.el, "+ Deck").click();
    expect(onGesture).toHaveBeenCalledWith(
      expect.objectContaining({ lemma: "d'abord", gloss: "first, at first", expressionAnswer: true }),
    );
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
      expression: false,
      gloss: "rarement",
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
    card.show(content({ rows: [{ form: "compelling", gloss: "convaincant" }], expression: true }), () => {});
    expect(card.el.textContent ?? "").not.toMatch(/lemm/i);
  });
});

describe("a card that waits for the engine", () => {
  it("shows a pending card with the headword, the kind line and a waiting line, but no action", () => {
    const card = createCard();
    card.show(
      content({ headword: "endeavors", surface: "endeavors", gloss: null, rarity: "Sélection.", pending: true }),
      () => {},
    );
    expect(card.visible()).toBe(true);
    expect(card.el.querySelector(".headword")!.textContent).toBe("endeavors");
    expect(card.el.querySelector(".rarity")!.textContent).toBe("Sélection.");
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Recherche dans le pack…");
    expect(card.el.querySelectorAll(".actions button")).toHaveLength(0);
    expect((card.el.querySelector(".actions") as HTMLElement).hidden).toBe(true);
  });

  it("lets the reader close a pending card", () => {
    const card = createCard();
    card.show(content({ pending: true }), () => {});
    (card.el.querySelector(".close") as HTMLButtonElement).click();
    expect(card.visible()).toBe(false);
  });

  it("bumps the generation on show, on hide, on the close button and after an action", () => {
    const card = createCard();
    const g0 = card.generation();
    const g1 = card.show(content({ pending: true }), () => {});
    expect(g1).not.toBe(g0);
    expect(card.generation()).toBe(g1);

    card.hide();
    const g2 = card.generation();
    expect(g2).not.toBe(g1);

    card.show(content(), () => {});
    const g3 = card.generation();
    (card.el.querySelector(".close") as HTMLButtonElement).click();
    expect(card.generation()).not.toBe(g3);

    card.show(content(), () => {});
    const g4 = card.generation();
    button(card.el, "+ Deck").click();
    expect(card.generation()).not.toBe(g4);
  });

  it("completes a card by showing it again, its actions arriving with the answer", () => {
    const card = createCard();
    card.show(content({ headword: "endeavors", surface: "endeavors", gloss: null, pending: true }), () => {});
    card.show(content({ headword: "endeavor", surface: "endeavors", gloss: "effort" }), () => {});
    expect(card.el.querySelector(".headword")!.textContent).toBe("endeavor");
    expect(card.el.querySelector(".seen")!.textContent).toContain("endeavors");
    expect(card.el.querySelector(".gloss")!.textContent).toBe("effort");
    expect(card.el.querySelectorAll(".actions button")).toHaveLength(3);
    expect((card.el.querySelector(".actions") as HTMLElement).hidden).toBe(false);
  });

  it("offers nothing to press on a complete card whose key never arrived", () => {
    const card = createCard();
    card.show(content({ headword: "endeavors", surface: "endeavors", gloss: null, noActions: true }), () => {});
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Pas de traduction dans le pack.");
    expect(card.el.querySelectorAll("button:not(.close)")).toHaveLength(0);
    expect((card.el.querySelector(".actions") as HTMLElement).hidden).toBe(true);
  });
});

describe("word by word", () => {
  it("renders the rows under their label, instead of the gloss line", () => {
    const card = createCard();
    card.show(
      content({
        headword: "a compelling argument",
        surface: "a compelling argument",
        gloss: null,
        expression: true,
        rows: [
          { form: "compelling", gloss: "convaincant" },
          { form: "argument", gloss: "argument" },
        ],
      }),
      () => {},
    );
    const answer = card.el.querySelector(".gloss")!;
    expect(answer.querySelector(".rows-label")!.textContent).toBe(
      "Mot à mot — ce n'est pas une traduction de l'expression.",
    );
    expect([...answer.querySelectorAll(".row")].map((r) => r.textContent)).toEqual([
      "compelling → convaincant",
      "argument → argument",
    ]);
    expect(answer.classList.contains("empty")).toBe(false);
    expect(answer.textContent).not.toContain("Pas de traduction");
  });

  it("says a translation is still coming, under the pack's answer", () => {
    // The card no longer makes the reader wait for a cold engine (4.8 s, measured on a Galaxy
    // Tab S6 Lite): the pack answers first, and says its rows are not the last word.
    const card = createCard();
    card.show(
      content({
        headword: "a compelling argument",
        surface: "a compelling argument",
        gloss: null,
        expression: true,
        rows: [{ form: "compelling", gloss: "convaincant" }],
        translating: true,
      }),
      () => {},
    );
    const answer = card.el.querySelector(".gloss")!;
    expect(answer.querySelector(".rows-label")).not.toBeNull();
    expect([...answer.querySelectorAll(".row")].map((r) => r.textContent)).toEqual(["compelling → convaincant"]);
    expect(answer.querySelector(".translating-note")!.textContent).toBe("Traduction en cours…");
  });

  it("stops saying it once the translation is there, and once it is known there is none", () => {
    const card = createCard();
    const base = {
      headword: "gave up",
      surface: "gave up",
      gloss: null,
      expression: true,
      rows: [{ form: "give", gloss: "Donner" }],
    };
    card.show(content({ ...base, translating: false }), () => {});
    expect(card.el.querySelector(".translating-note")).toBeNull();

    card.show(
      content({
        ...base,
        rows: undefined,
        translating: false,
        translation: { sentence: "Elle a abandonné.", marks: [] },
      }),
      () => {},
    );
    expect(card.el.querySelector(".translating-note")).toBeNull();
  });

  it("states that the pack has no translation for the expression when no row qualifies", () => {
    const card = createCard();
    card.show(
      content({ headword: "put up with", surface: "put up with", gloss: null, expression: true, rows: [] }),
      () => {},
    );
    const answer = card.el.querySelector(".gloss")!;
    expect(answer.querySelectorAll(".row")).toHaveLength(0);
    expect(answer.querySelector(".rows-label")).toBeNull();
    expect(answer.textContent).toBe("Pas de traduction dans le pack pour cette expression.");
    expect(answer.classList.contains("empty")).toBe(true);
    // The expression card keeps its two actions: its key is the text, known without an answer.
    expect([...card.el.querySelectorAll(".actions button")].map((b) => b.textContent)).toEqual(["+ Deck", "Ignorer"]);
  });

  it("keeps the plain no-translation note for a word", () => {
    const card = createCard();
    card.show(content({ gloss: null, rows: [] }), () => {});
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Pas de traduction dans le pack.");
  });
});

describe("WordPopup", () => {
  it("returns the card's generation from show and exposes it", () => {
    const popup = new WordPopup({ css: "", onGesture: () => {} });
    const g1 = popup.show(content());
    expect(popup.generation()).toBe(g1);
    expect(popup.visible()).toBe(true);
    popup.hide();
    expect(popup.generation()).not.toBe(g1);
    expect(popup.show(content())).not.toBe(g1);
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

describe("createCard — the translated sentence", () => {
  const translation = {
    sentence: "Elle a abandonné après la troisième tentative.",
    marks: [{ start: 5, end: 16 }],
  };
  const expression = (over: Partial<WordPopupContent> = {}) =>
    content({ headword: "gave up", surface: "gave up", expression: true, gloss: null, translation, ...over });

  it("labels it as a machine translation and marks where the selection landed", () => {
    const card = createCard();
    card.show(expression(), () => {});
    const block = card.el.querySelector(".translation") as HTMLElement;
    expect(block.hidden).toBe(false);
    expect(block.querySelector(".translation-label")!.textContent).toBe("Dans votre phrase — traduction automatique");
    expect(block.querySelector(".translation-sentence")!.textContent).toBe(translation.sentence);
    expect([...block.querySelectorAll("mark")].map((m) => m.textContent)).toEqual(["a abandonné"]);
  });

  it("marks every span when the engine split the selection across a reordering", () => {
    const card = createCard();
    card.show(
      expression({
        translation: {
          sentence: "Il a dû supporter le bruit.",
          marks: [
            { start: 8, end: 17 },
            { start: 0, end: 2 },
          ],
        },
      }),
      () => {},
    );
    expect([...card.el.querySelectorAll("mark")].map((m) => m.textContent)).toEqual(["Il", "supporter"]);
    expect(card.el.querySelector(".translation-sentence")!.textContent).toBe("Il a dû supporter le bruit.");
  });

  it("keeps the page's markup characters as text — never as live markup", () => {
    // The sentence came from the page, through the engine: as HTML it could carry anything.
    const card = createCard();
    const sentence = 'Utilisez <img src=x onerror="alert(1)"> et <b>ceci</b>.';
    card.show(expression({ translation: { sentence, marks: [] } }), () => {});
    const shown = card.el.querySelector(".translation-sentence")!;
    expect(shown.textContent).toBe(sentence);
    expect(shown.querySelector("img, b")).toBeNull();
  });

  it("does not claim the pack has no translation right above one", () => {
    const card = createCard();
    card.show(expression(), () => {});
    expect((card.el.querySelector(".gloss") as HTMLElement).hidden).toBe(true);
    expect(card.el.textContent).not.toContain("Pas de traduction dans le pack");
  });

  it("shows no word-by-word rows beside a translation", () => {
    const card = createCard();
    card.show(expression({ rows: [{ form: "give", gloss: "Donner" }] }), () => {});
    expect(card.el.textContent).not.toContain("Mot à mot");
  });

  it("keeps an expression's dictionary gloss above the translation", () => {
    const card = createCard();
    card.show(expression({ headword: "give up", gloss: "Abandonner, renoncer", expression: false }), () => {});
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Abandonner, renoncer");
    expect((card.el.querySelector(".translation") as HTMLElement).hidden).toBe(false);
  });

  it("shows no translation while the card is still waiting", () => {
    const card = createCard();
    card.show(expression({ pending: true }), () => {});
    expect((card.el.querySelector(".translation") as HTMLElement).hidden).toBe(true);
  });

  it("clears the translation when the next card has none", () => {
    const card = createCard();
    card.show(expression(), () => {});
    card.show(content(), () => {});
    expect((card.el.querySelector(".translation") as HTMLElement).hidden).toBe(true);
    expect(card.el.querySelectorAll("mark")).toHaveLength(0);
  });

  it("skips a mark it cannot place instead of breaking the card", () => {
    const card = createCard();
    card.show(expression({ translation: { sentence: "Court.", marks: [{ start: 2, end: 40 }] } }), () => {});
    expect(card.el.querySelector(".translation-sentence")!.textContent).toBe("Court.");
  });

  it("gives + Deck no machine translation to store", () => {
    const card = createCard();
    const seen: Gesture[] = [];
    card.show(expression(), (g) => seen.push(g));
    button(card.el, "+ Deck").click();
    expect(seen).toHaveLength(1);
    expect(JSON.stringify(seen[0])).not.toContain("abandonné");
    expect(seen[0]!.gloss).toBeNull();
  });
});

describe("createCard — what a pending card says it waits for", () => {
  it("says it is translating when the card asked the translation engine", () => {
    const card = createCard();
    card.show(content({ pending: true, translating: true }), () => {});
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Traduction en cours…");
  });

  it("keeps saying it searches the pack when there is no engine to wait for", () => {
    // Every shipped build: the pending line is exactly what it was.
    const card = createCard();
    card.show(content({ pending: true }), () => {});
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Recherche dans le pack…");
  });
});

describe("read-aloud on the card", () => {
  const samantha: VoiceInfo = {
    name: "Samantha",
    lang: "en-US",
    localService: true,
    default: false,
    voiceURI: "Samantha",
  };

  function speaking(voices: VoiceInfo[] = [samantha]) {
    const fake = makeFakeSpeech(voices);
    const speaker = createSpeaker(fake.engine, "en", fake.preference);
    const card = createCard(speaker);
    document.body.append(card.el);
    return { fake, speaker, card };
  }

  const listenRow = (card: { el: HTMLElement }): HTMLElement => card.el.querySelector(".listen") as HTMLElement;
  const labels = (card: { el: HTMLElement }): string[] =>
    [...listenRow(card).querySelectorAll("button")].map((b) => b.textContent ?? "");

  it("has no listen row without a speaker, or without a voice", () => {
    const plain = createCard();
    plain.show(content(), () => {});
    expect(listenRow(plain).hidden).toBe(true);
    const { card } = speaking([]);
    card.show(content(), () => {});
    expect(listenRow(card).hidden).toBe(true);
  });

  it("adds the row to the open card when the voices are announced late", () => {
    const { fake, card } = speaking([]);
    card.show(content(), () => {});
    fake.list([samantha]);
    expect(listenRow(card).hidden).toBe(false);
    expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
  });

  it("speaks a word as seen on the page, and its dictionary form beside it", () => {
    const { fake, card } = speaking();
    card.show(content({ headword: "run", surface: "ran", sentence: "She ran home." }), () => {});
    expect(labels(card)).toEqual(["▶ ran", "▶ run", "▶ Phrase"]);
    button(card.el, "▶ ran").click();
    expect(fake.spoken.map((u) => u.text)).toEqual(["ran"]);
    expect(button(card.el, "■ Arrêter").getAttribute("aria-label")).toBe("Arrêter la lecture");
  });

  describe("the dictionary form (add-lingua-dictionary-form-voice)", () => {
    const es = () => content({ headword: "ser", surface: "Es", sentence: "Es una casa." });

    it("offers the form seen, then the dictionary form, each labelled with what it reads", () => {
      const { fake, card } = speaking();
      card.show(es(), () => {});
      expect(labels(card)).toEqual(["▶ Es", "▶ ser", "▶ Phrase"]);
      expect(button(card.el, "▶ Es").getAttribute("aria-label")).toBe("Écouter la forme vue « Es »");
      expect(button(card.el, "▶ ser").getAttribute("aria-label")).toBe("Écouter la forme du dictionnaire « ser »");
      button(card.el, "▶ ser").click();
      expect(fake.spoken.map((u) => u.text)).toEqual(["ser"]);
      expect(labels(card)).toEqual(["▶ Es", "■ Arrêter", "▶ Phrase"]);
    });

    it("keeps one word button for a word that is its own dictionary form, whatever its case", () => {
      const { card } = speaking();
      card.show(content({ headword: "casa", surface: "Casa", sentence: "Casa grande." }), () => {});
      expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
      expect(card.el.querySelector<HTMLElement>(".seen")?.hidden).toBe(true);
      card.show(content({ headword: "casa", surface: "casa", sentence: "Una casa." }), () => {});
      expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
    });

    it("switches from the form seen to the dictionary form, and stops it", () => {
      const { fake, speaker, card } = speaking();
      card.show(es(), () => {});
      button(card.el, "▶ Es").click();
      button(card.el, "▶ ser").click();
      expect(fake.spoken.map((u) => u.text)).toEqual(["Es", "ser"]);
      expect(speaker.speaking()).toEqual({ key: "headword", text: "ser" });
      button(card.el, "■ Arrêter").click();
      expect(speaker.speaking()).toBeNull();
      expect(labels(card)).toEqual(["▶ Es", "▶ ser", "▶ Phrase"]);
    });

    it("falls silent when the card closes or another word opens", () => {
      const { speaker, card } = speaking();
      card.show(es(), () => {});
      button(card.el, "▶ ser").click();
      button(card.el, "✕").click();
      expect(speaker.speaking()).toBeNull();
      card.show(es(), () => {});
      button(card.el, "▶ ser").click();
      card.show(content({ headword: "ship", surface: "ship", sentence: "Ships sail." }), () => {});
      expect(speaker.speaking()).toBeNull();
    });

    it("keeps reading the form seen when a pending card completes with its dictionary form", () => {
      const { speaker, card } = speaking();
      card.show(
        content({ headword: "Es", surface: "Es", sentence: "Es una casa.", pending: true, gloss: null }),
        () => {},
      );
      expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
      button(card.el, "▶ Mot").click();
      card.show(es(), () => {});
      expect(speaker.speaking()).toEqual({ key: "selection", text: "Es" });
      expect(labels(card)).toEqual(["■ Arrêter", "▶ ser", "▶ Phrase"]);
    });
  });

  it("speaks the selected words of an expression card as one utterance", () => {
    const { fake, card } = speaking();
    card.show(content({ headword: "ship on friday", surface: "ship on Friday", expression: true }), () => {});
    expect(labels(card)).toEqual(["▶ Sélection", "▶ Phrase"]);
    expect(button(card.el, "▶ Sélection").getAttribute("aria-label")).toBe("Écouter la sélection");
    button(card.el, "▶ Sélection").click();
    expect(fake.spoken.map((u) => u.text)).toEqual(["ship on Friday"]);
  });

  it("labels the words of an expression the pack knows a selection, too", () => {
    // « animal doméstico » has a gloss of its own, so the card is not an unknown expression's.
    const { card } = speaking();
    card.show(
      content({
        headword: "animal doméstico",
        surface: "animal doméstico",
        gloss: "Animal domestique",
        sentence: "El gato es el animal doméstico más popular.",
      }),
      () => {},
    );
    expect(labels(card)).toEqual(["▶ Sélection", "▶ Phrase"]);
  });

  it("speaks the whole sentence the selection was taken from", () => {
    const { fake, card } = speaking();
    card.show(content(), () => {});
    expect(button(card.el, "▶ Phrase").getAttribute("aria-label")).toBe("Écouter la phrase");
    button(card.el, "▶ Phrase").click();
    expect(fake.spoken.map((u) => u.text)).toEqual(["They seldom ship on Friday."]);
  });

  it("offers no sentence button when the selection is its whole sentence, or there is none", () => {
    const { card } = speaking();
    card.show(
      content({ headword: "they seldom ship on friday", surface: "They seldom ship on Friday", expression: true }),
      () => {},
    );
    expect(labels(card)).toEqual(["▶ Sélection"]);
    card.show(content({ sentence: "" }), () => {});
    expect(labels(card)).toEqual(["▶ Mot"]);
  });

  it("stops when the speaking button is pressed, and offers to listen again", () => {
    const { fake, card } = speaking();
    card.show(content(), () => {});
    button(card.el, "▶ Mot").click();
    expect(button(card.el, "■ Arrêter").classList.contains("speaking")).toBe(true);
    button(card.el, "■ Arrêter").click();
    expect(fake.cancels()).toBe(2);
    expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
  });

  it("switches from the sentence to the selection", () => {
    const { fake, speaker, card } = speaking();
    card.show(content(), () => {});
    button(card.el, "▶ Phrase").click();
    button(card.el, "▶ Mot").click();
    expect(speaker.speaking()).toEqual({ key: "selection", text: "Seldom" });
    expect(labels(card)).toEqual(["■ Arrêter", "▶ Phrase"]);
    fake.spoken[1].done(null);
    expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
  });

  it("offers the row on a pending card, and keeps speaking when the answer completes it", () => {
    const { speaker, card } = speaking();
    card.show(content({ pending: true, gloss: null }), () => {});
    expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
    button(card.el, "▶ Phrase").click();
    card.show(content(), () => {});
    expect(speaker.speaking()?.key).toBe("sentence");
    expect(labels(card)).toEqual(["▶ Mot", "■ Arrêter"]);
  });

  it("falls silent when the card closes, by its close button or by a gesture", () => {
    const { speaker, card } = speaking();
    card.show(content(), () => {});
    button(card.el, "▶ Phrase").click();
    button(card.el, "✕").click();
    expect(speaker.speaking()).toBeNull();
    card.show(content(), () => {});
    button(card.el, "▶ Mot").click();
    button(card.el, "+ Deck").click();
    expect(speaker.speaking()).toBeNull();
  });

  it("falls silent when another word's card opens", () => {
    const { speaker, card } = speaking();
    card.show(content(), () => {});
    button(card.el, "▶ Mot").click();
    card.show(content({ headword: "ship", surface: "ship", sentence: "Ships sail." }), () => {});
    expect(speaker.speaking()).toBeNull();
  });

  it("leaves a settings preview alone: it is not the card's", () => {
    const { speaker, card } = speaking();
    card.show(content(), () => {});
    speaker.speak("preview", "This is how your pages will sound.");
    card.hide();
    card.show(content({ headword: "ship", surface: "ship" }), () => {});
    expect(speaker.speaking()?.key).toBe("preview");
  });

  it("keeps the page selection: pressing a listen button prevents the press's default", () => {
    const { card } = speaking();
    card.show(content(), () => {});
    const b = button(card.el, "▶ Mot");
    const down = new Event("pointerdown", { cancelable: true });
    const mouse = new MouseEvent("mousedown", { cancelable: true });
    b.dispatchEvent(down);
    b.dispatchEvent(mouse);
    expect(down.defaultPrevented).toBe(true);
    expect(mouse.defaultPrevented).toBe(true);
  });

  it("adds the row to the open card once the reader allows Android's voices", async () => {
    const android: VoiceInfo = {
      name: "anglais (USA,DEFAULT)",
      lang: "eng-USA-default",
      localService: false,
      default: false,
      voiceURI: "moz-tts:android:eng_USA_default",
    };
    const { fake, card } = speaking([android]);
    card.show(content(), () => {});
    expect(listenRow(card).hidden).toBe(true);
    fake.prefer({ androidVoices: true });
    expect(labels(card)).toEqual(["▶ Mot", "▶ Phrase"]);
  });

  it("hands the speaker to the page's popup, whose hide silences it", () => {
    const fake = makeFakeSpeech([samantha]);
    const speaker = createSpeaker(fake.engine, "en", fake.preference);
    const popup = new WordPopup({ css: "", onGesture: () => {}, speaker });
    popup.show(content());
    speaker.speak("selection", "Seldom");
    popup.hide();
    expect(speaker.speaking()).toBeNull();
  });
});

describe("an elided French word, heard with the word it leans on (add-lingua-french-read-aloud D5)", () => {
  const thomas: VoiceInfo = { name: "Thomas", lang: "fr-FR", localService: true, default: false, voiceURI: "Thomas" };
  const samantha: VoiceInfo = { name: "Samantha", lang: "en-US", localService: true, default: false, voiceURI: "S" };
  const monica: VoiceInfo = { name: "Mónica", lang: "es-ES", localService: true, default: false, voiceURI: "M" };

  /** Where `piece` sits in `sentence` — its `nth` occurrence — as the card's place in the sentence. */
  const placeOf = (sentence: string, piece: string, nth = 0): { start: number; end: number } => {
    let start = -1;
    for (let i = 0; i <= nth; i++) start = sentence.indexOf(piece, start + 1);
    if (start < 0) throw new Error(`no « ${piece} » in « ${sentence} »`);
    return { start, end: start + piece.length };
  };

  /** A card whose speaker reads `lang`, in the interface language `language` (English by default). */
  function speaking(lang = "fr", language: "fr" | "en" | "es" = "en") {
    const fake = makeFakeSpeech([thomas, samantha, monica]);
    const speaker = createSpeaker(fake.engine, lang, fake.preference);
    const card = createCard(speaker, language);
    document.body.append(card.el);
    return { fake, speaker, card };
  }

  /** What each listen button says and what it speaks, in order. */
  function row(card: { el: HTMLElement }, fake: ReturnType<typeof makeFakeSpeech>): [string, string][] {
    const buttons = [...card.el.querySelectorAll<HTMLButtonElement>(".listen button")];
    return buttons.map((b) => {
      const label = b.textContent ?? "";
      b.click();
      const text = fake.spoken.at(-1)!.text;
      // Pressing it again stops it, so the next button reads under its own label.
      [...card.el.querySelectorAll<HTMLButtonElement>(".listen button")]
        .find((x) => x.textContent === "■ Stop")
        ?.click();
      return [label, text];
    });
  }

  /** The card of a piece of « L'homme est venu. »: `l'` with its apostrophe, or `l` without it. */
  const article = (surface: "L'" | "L", over: Partial<WordPopupContent> = {}): WordPopupContent =>
    content({
      headword: "le",
      surface,
      gloss: "the",
      sentence: "L'homme est venu.",
      selection: { start: 0, end: surface.length },
      ...over,
    });

  it("An elided article, its span with the apostrophe: « ▶ L'homme », then « ▶ le », then the sentence", () => {
    const { fake, card } = speaking();
    card.show(article("L'"), () => {});
    expect(row(card, fake)).toEqual([
      ["▶ L'homme", "L'homme"],
      ["▶ le", "le"],
      ["▶ Sentence", "L'homme est venu."],
    ]);
    card.show(article("L'"), () => {});
    expect(button(card.el, "▶ L'homme").getAttribute("aria-label")).toBe("Listen to the seen form “L'homme”");
  });

  it("An elided article, its apostrophe left to the next span: the same buttons", () => {
    const { fake, card } = speaking();
    card.show(article("L"), () => {});
    expect(row(card, fake)).toEqual([
      ["▶ L'homme", "L'homme"],
      ["▶ le", "le"],
      ["▶ Sentence", "L'homme est venu."],
    ]);
  });

  it("Before a hyphen: « Qu'est » of « Qu'est-ce que c'est ? »", () => {
    const { fake, card } = speaking();
    const qu = content({
      headword: "que",
      surface: "Qu'",
      sentence: "Qu'est-ce que c'est ?",
      selection: placeOf("Qu'est-ce que c'est ?", "Qu'"),
    });
    card.show(qu, () => {});
    expect(row(card, fake)[0]).toEqual(["▶ Qu'est", "Qu'est"]);
  });

  it("A typographic apostrophe: « jusqu’à » of « jusqu’à demain »", () => {
    const { fake, card } = speaking();
    card.show(
      content({
        headword: "jusque",
        surface: "jusqu’",
        sentence: "jusqu’à demain",
        selection: placeOf("jusqu’à demain", "jusqu’"),
      }),
      () => {},
    );
    expect(row(card, fake)[0]).toEqual(["▶ jusqu’à", "jusqu’à"]);
  });

  it("carries on across an apostrophe between two letters: « jusqu'aujourd'hui » whole", () => {
    const { fake, card } = speaking();
    card.show(
      content({
        headword: "jusque",
        surface: "jusqu'",
        sentence: "Il reste jusqu'aujourd'hui, 3 fois.",
        selection: placeOf("Il reste jusqu'aujourd'hui, 3 fois.", "jusqu'"),
      }),
      () => {},
    );
    expect(row(card, fake)[0]).toEqual(["▶ jusqu'aujourd'hui", "jusqu'aujourd'hui"]);
  });

  it("stops at a digit or a punctuation mark", () => {
    const { fake, card } = speaking();
    card.show(
      content({
        headword: "le",
        surface: "l'",
        sentence: "Voir l'an2000 et l'été.",
        selection: placeOf("Voir l'an2000 et l'été.", "l'"),
      }),
      () => {},
    );
    expect(row(card, fake)[0]).toEqual(["▶ l'an", "l'an"]);
    card.show(
      content({
        headword: "le",
        surface: "l'",
        sentence: "Voir l'an2000 et l'été.",
        selection: placeOf("Voir l'an2000 et l'été.", "l'", 1),
      }),
      () => {},
    );
    expect(row(card, fake)[0]).toEqual(["▶ l'été", "l'été"]);
  });

  it("A contraction: the piece of « au » reads « au », under one word button", () => {
    const au = (): WordPopupContent =>
      content({
        headword: "à",
        surface: "à",
        written: "au",
        sentence: "Il va au marché.",
        selection: placeOf("Il va au marché.", "au"),
      });
    const en = speaking();
    en.card.show(au(), () => {});
    expect(row(en.card, en.fake)).toEqual([
      ["▶ Word", "au"],
      ["▶ Sentence", "Il va au marché."],
    ]);
    const es = speaking("fr", "es");
    es.card.show(au(), () => {});
    expect(row(es.card, es.fake)[0]).toEqual(["▶ Palabra", "au"]);
  });

  it("A whole « l’homme », its apostrophe aside: reads its surface, as today", () => {
    const { fake, card } = speaking();
    card.show(
      content({
        headword: "homme",
        surface: "l'homme",
        written: "l’homme",
        sentence: "Voici l’homme.",
        selection: placeOf("Voici l’homme.", "l’homme"),
      }),
      () => {},
    );
    expect(row(card, fake)).toEqual([
      ["▶ l'homme", "l'homme"],
      ["▶ homme", "homme"],
      ["▶ Sentence", "Voici l’homme."],
    ]);
    // Case and Unicode normalisation aside too: « É » composed or decomposed is the same word.
    card.show(
      content({
        headword: "été",
        surface: "Été",
        written: "Été",
        sentence: "Été chaud.",
        selection: placeOf("E\u0301te\u0301 chaud.", "E\u0301te\u0301"),
      }),
      () => {},
    );
    expect(row(card, fake)[0]).toEqual(["▶ Word", "Été"]);
  });

  it("leaves out the sentence button when the heard text is the whole sentence", () => {
    const { card } = speaking();
    card.show(article("L'", { sentence: "L'homme." }), () => {});
    expect([...card.el.querySelectorAll(".listen button")].map((b) => b.textContent)).toEqual(["▶ L'homme", "▶ le"]);
  });

  it("reads the piece as today without its place in the sentence", () => {
    const { fake, card } = speaking();
    card.show(article("L'", { selection: undefined }), () => {});
    expect(row(card, fake)).toEqual([
      ["▶ L'", "L'"],
      ["▶ le", "le"],
      ["▶ Sentence", "L'homme est venu."],
    ]);
    card.show(article("L'", { selection: null }), () => {});
    expect(row(card, fake)[0]).toEqual(["▶ L'", "L'"]);
    // A place that does not fit its sentence is no place.
    card.show(article("L'", { selection: { start: 3, end: 40 } }), () => {});
    expect(row(card, fake)[0]).toEqual(["▶ L'", "L'"]);
  });

  describe("whatever text the analysis gives the piece's token (add-lingua-french-tokenisation D3, D6)", () => {
    // Change 40 writes an elided piece's token as the word it stands for — `Le` [0, 2) for `L'` —
    // and the card's `written` is the range's text: `L'`.
    it("`Le` written `L'`: « ▶ L'homme », then « ▶ le », then the sentence", () => {
      const { fake, card } = speaking();
      const sentence = "L'homme est venu.";
      card.show(
        content({ headword: "le", surface: "Le", written: "L'", sentence, selection: placeOf(sentence, "L'") }),
        () => {},
      );
      expect(row(card, fake)).toEqual([
        ["▶ L'homme", "L'homme"],
        ["▶ le", "le"],
        ["▶ Sentence", "L'homme est venu."],
      ]);
    });

    it("`Ce` written `C’` inside guillemets and a narrow no-break space: « ▶ C’est », then « ▶ ce »", () => {
      const { fake, card } = speaking();
      const sentence = "«\u202FC’est la vie\u202F»";
      card.show(
        content({ headword: "ce", surface: "Ce", written: "C’", sentence, selection: placeOf(sentence, "C’") }),
        () => {},
      );
      expect(row(card, fake).slice(0, 2)).toEqual([
        ["▶ C’est", "C’est"],
        ["▶ ce", "ce"],
      ]);
    });

    it("`jusqu'au`: the elided piece reads « jusqu'au », the contraction's piece « au »", () => {
      const { fake, card } = speaking();
      const sentence = "Il dort jusqu'au soir.";
      card.show(
        content({
          headword: "jusque",
          surface: "jusque",
          written: "jusqu'",
          sentence,
          selection: placeOf(sentence, "jusqu'"),
        }),
        () => {},
      );
      expect(row(card, fake).slice(0, 2)).toEqual([
        ["▶ jusqu'au", "jusqu'au"],
        ["▶ jusque", "jusque"],
      ]);
      card.show(
        content({ headword: "à", surface: "à", written: "au", sentence, selection: placeOf(sentence, "au") }),
        () => {},
      );
      expect(row(card, fake)[0]).toEqual(["▶ Word", "au"]);
    });

    it("an elided piece alone, or without its place, reads the word it stands for — never its letter", () => {
      const { fake, card } = speaking();
      // `l’ homme`, typed with a space: nothing to lean on.
      const sentence = "Voici l’ homme.";
      card.show(
        content({ headword: "le", surface: "le", written: "l’", sentence, selection: placeOf(sentence, "l’") }),
        () => {},
      );
      expect(row(card, fake)[0]).toEqual(["▶ Word", "le"]);
      card.show(content({ headword: "le", surface: "Le", written: "L'", sentence: "L'homme est venu." }), () => {});
      expect(row(card, fake)[0]).toEqual(["▶ Word", "Le"]);
    });
  });

  it("keeps « ▶ Selection » for a selection of several words", () => {
    const { fake, card } = speaking();
    card.show(
      content({
        headword: "l'homme est",
        surface: "L'homme est",
        expression: true,
        sentence: "L'homme est venu.",
        selection: placeOf("L'homme est venu.", "L'homme est"),
      }),
      () => {},
    );
    expect(row(card, fake)).toEqual([
      ["▶ Selection", "L'homme est"],
      ["▶ Sentence", "L'homme est venu."],
    ]);
  });

  it("keeps reading the heard text when a pending card completes", () => {
    const { speaker, card } = speaking();
    card.show(article("L'", { pending: true, gloss: null }), () => {});
    button(card.el, "▶ L'homme").click();
    card.show(article("L'"), () => {});
    expect(speaker.speaking()).toEqual({ key: "selection", text: "L'homme" });
    card.show(content({ headword: "venir", surface: "venu", sentence: "L'homme est venu." }), () => {});
    expect(speaker.speaking()).toBeNull();
  });

  describe("English and Spanish, as before", () => {
    /** The same cards, their place in the sentence given — which only a French speaker reads. */
    const cards = (): WordPopupContent[] => [
      article("L'"),
      article("L"),
      article("L'", { surface: "Le", written: "L'" }),
      content({
        headword: "do",
        surface: "do",
        written: "don't",
        sentence: "I don't know.",
        selection: placeOf("I don't know.", "don't"),
      }),
      content({
        headword: "de",
        surface: "de",
        written: "del",
        sentence: "Salgo del cine.",
        selection: placeOf("Salgo del cine.", "del"),
      }),
      content({
        headword: "run",
        surface: "ran",
        sentence: "She ran home.",
        selection: placeOf("She ran home.", "ran"),
      }),
    ];

    for (const lang of ["en", "es"]) {
      it(`a speaker reading ${lang}: the buttons, labels and texts of the card without its place`, () => {
        const withPlace = speaking(lang);
        const without = speaking(lang);
        for (const card of cards()) {
          withPlace.card.show(card, () => {});
          without.card.show({ ...card, selection: undefined }, () => {});
          expect(row(withPlace.card, withPlace.fake)).toEqual(row(without.card, without.fake));
        }
        withPlace.card.show(cards()[0], () => {});
        expect(row(withPlace.card, withPlace.fake)[0]).toEqual(["▶ L'", "L'"]);
        withPlace.card.show(cards()[2], () => {});
        expect(row(withPlace.card, withPlace.fake)[0]).toEqual(["▶ Word", "Le"]);
        withPlace.card.show(cards()[3], () => {});
        expect(row(withPlace.card, withPlace.fake)[0]).toEqual(["▶ Word", "do"]);
        withPlace.card.show(cards()[4], () => {});
        expect(row(withPlace.card, withPlace.fake)[0]).toEqual(["▶ Word", "de"]);
      });
    }
  });
});
