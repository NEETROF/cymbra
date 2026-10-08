import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { card as enCard } from "@/i18n/en/card.ts";
import { hud as enHud } from "@/i18n/en/hud.ts";
import { selection as enSelection } from "@/i18n/en/selection.ts";
import { selection as esSelection } from "@/i18n/es/selection.ts";
import { hud as frHud } from "@/i18n/fr/hud.ts";
import { createHud, LinguaHud } from "@/reading/hud.ts";
import { readingCopy } from "@/reading/reading-copy.ts";
import { rarityText, rowGloss, SelectionCards } from "@/reading/selection-card.ts";
import { ReadingSession } from "@/reading/session.ts";
import { createCard, WordPopup, type WordPopupContent } from "@/reading/wordpopup.ts";
import { makeFakePort } from "./helpers.ts";

// The reading surfaces speak the interface language (localise-lingua-reading-surfaces): the HUD,
// the word card, the selection card's lines and the session that builds them, handed the copy and
// the language — what `hud`, `wordpopup`, `selection-card` and `rarity-text` assert in French,
// unchanged, asserted here in English and Spanish, with the hosts' `lang`.

const NNBSP = " ";
const actions = () => ({ onReview() {}, onStats() {}, onSettings() {} });

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
  document.documentElement.querySelectorAll("[data-cymbra-lingua-skip]").forEach((el) => el.remove());
});

describe("the HUD", () => {
  it("An English-native reader: the pill, its actions and its figure are the English catalogue's", () => {
    const hud = createHud(actions(), undefined, enHud, "en");
    const pct = hud.el.querySelector<HTMLButtonElement>(".hud-pct")!;
    expect(pct.getAttribute("aria-label")).toBe("Known words on the page — open the actions");
    expect([...hud.el.querySelectorAll(".hud-act")].map((b) => b.textContent)).toEqual(["Review", "Stats"]);
    expect(hud.el.querySelector(".hud-gear")?.getAttribute("aria-label")).toBe("Settings");
    expect(hud.el.querySelector(".hud-level")?.textContent).toBe("Choose your level");
    hud.update({ analysable: true, percent: 45 });
    expect(pct.textContent).toBe("45%");
    hud.update({ analysable: true, percent: null });
    expect(pct.textContent).toBe("—");
  });

  it("writes the percentage as each language does: the French tight, the Spanish with its space", () => {
    const fr = createHud(actions());
    fr.update({ analysable: true, percent: 45 });
    expect(fr.el.querySelector(".hud-pct")?.textContent).toBe("45%");
    const es = createHud(actions(), undefined, readingCopy("es").hud, "es");
    es.update({ analysable: true, percent: 45 });
    expect(es.el.querySelector(".hud-pct")?.textContent).toBe(`45${NNBSP}%`);
  });

  it("says the interface language on its host, French without one", () => {
    const en = new LinguaHud({ css: "", actions: actions(), language: "en", copy: enHud });
    en.mount();
    expect(document.getElementById("cymbra-lingua-hud-host")?.getAttribute("lang")).toBe("en");
    en.destroy();
    const fr = new LinguaHud({ css: "", actions: actions() });
    fr.mount();
    expect(document.getElementById("cymbra-lingua-hud-host")?.getAttribute("lang")).toBe("fr");
    fr.destroy();
  });
});

const content = (over: Partial<WordPopupContent> = {}): WordPopupContent => ({
  headword: "run",
  surface: "running",
  gloss: "courir",
  rarity: "",
  sentence: "They were running late.",
  rect: { left: 40, top: 60, bottom: 80 },
  ...over,
});

describe("the word card", () => {
  it("An English-native reader: the labels, the actions and the close control are the English catalogue's", () => {
    const card = createCard(undefined, enCard);
    document.body.append(card.el);
    card.show(content({ status: "ignored" }), () => {});
    expect(card.el.querySelector(".seen")?.textContent).toBe("form seen: “running”");
    expect([...card.el.querySelectorAll(".actions button")].map((b) => b.textContent)).toEqual([
      "I know it",
      "+ Deck",
      "Learn again",
    ]);
    expect(card.el.querySelector(".close")?.getAttribute("aria-label")).toBe("Close");
    card.show(content({ gloss: null, pending: true }), () => {});
    expect(card.el.querySelector(".gloss")?.textContent).toBe("Looking in the pack…");
    card.show(content({ gloss: null, expression: true, rows: [] }), () => {});
    expect(card.el.querySelector(".gloss")?.textContent).toBe("No translation in the pack for this expression.");
    card.show(content({ gloss: null, rows: [{ form: "late", gloss: "tard" }] }), () => {});
    expect(card.el.querySelector(".rows-label")?.textContent).toBe(
      "Word by word — not a translation of the expression.",
    );
    expect(card.el.querySelector(".row")?.textContent).toBe("late → tard");
  });

  it("says the interface language on its host, French without one", () => {
    expect(
      new WordPopup({ css: "", onGesture: () => {}, language: "en", copy: enCard }).host.getAttribute("lang"),
    ).toBe("en");
    expect(new WordPopup({ css: "", onGesture: () => {} }).host.getAttribute("lang")).toBe("fr");
    expect(new WordPopup({ css: "", onGesture: () => {} }).host.id).toBe("cymbra-lingua-host");
  });
});

describe("the selection card's lines", () => {
  it("An English-native reader: the frequency bands are the English catalogue's, their counts English", () => {
    expect(rarityText("Unknown", 20_001, enSelection, "en")).toBe("Rare — beyond the 20,000 most frequent words.");
    expect(rarityText("Unknown", 101, enSelection, "en")).toBe("Common — among the 1,000 most frequent words.");
    expect(rarityText("Learning", 3, enSelection, "en")).toBe("In your deck — being learned.");
    expect(rarityText("Unknown", undefined, enSelection, "en")).toBe("");
  });

  it("writes a Spanish count as the RAE does: no grouping below ten thousand, a narrow space above", () => {
    expect(rarityText("Unknown", 5_001, esSelection, "es")).toBe(esSelection.uncommon("5000"));
    expect(rarityText("Unknown", null, esSelection, "es")).toBe(esSelection.rare(`20${NNBSP}000`));
  });

  it("cuts a long sense with the catalogue's ellipsis, whatever the language", () => {
    const long = `${"word ".repeat(20)}end`;
    expect(rowGloss(long)).toBe(enSelection.truncated(rowGloss(long)!.slice(0, -1)));
    expect(rowGloss(long, enSelection)).toBe(rowGloss(long));
  });

  it("opens an expression card with the English kind line", () => {
    const shown: WordPopupContent[] = [];
    const cards = new SelectionCards(
      {
        phraseGloss: () => new Promise(() => {}), // never answers: the pending card is what is checked
        gloss: async () => undefined,
        wordGrammar: () => new Promise(() => {}),
      },
      { show: (c) => shown.push(c), generation: () => shown.length },
      { clock: { setTimeout: () => 0, clearTimeout: () => {} }, copy: enSelection, interfaceLanguage: "en" },
    );
    cards.openForSelection({ text: "give up", sentence: "Never give up.", rect: content().rect }, null);
    expect(shown.at(-1)?.rarity).toBe("Expression — the card will keep its original sentence.");
    cards.openForSelection({ text: "never", sentence: "Never give up.", rect: content().rect }, null);
    expect(shown.at(-1)?.rarity).toBe("Selection.");
  });
});

describe("the session hands the surfaces the language", () => {
  type RuntimeListener = (msg: unknown, sender: { tab?: { id: number } }, send: (r: unknown) => void) => unknown;
  let runtimeListeners: RuntimeListener[];

  beforeEach(() => {
    runtimeListeners = [];
    vi.stubGlobal("chrome", {
      runtime: {
        sendMessage: vi.fn(async (msg: { type?: string }) => (msg?.type === "store:get" ? { items: {} } : undefined)),
        onMessage: { addListener: (fn: RuntimeListener) => void runtimeListeners.push(fn) },
      },
      storage: { local: { get: async () => ({}), set: async () => {} }, onChanged: { addListener: () => {} } },
    });
  });

  it("An English-native reader: the drawer the session opens carries lang=en", async () => {
    const { port } = makeFakePort();
    const css = { tokens: "", popup: "", drawer: "", hud: "" };
    const session = new ReadingSession(port, { css, surface: "book", language: "en", copy: readingCopy("en") });
    await session.start(null);
    for (const fn of runtimeListeners) fn({ type: "openDrawer", view: "review" }, {}, () => {});
    expect(document.getElementById("cymbra-lingua-drawer-host")?.getAttribute("lang")).toBe("en");
  });

  it("Every reader today: a session built without a language opens a French drawer", async () => {
    const { port } = makeFakePort();
    const css = { tokens: "", popup: "", drawer: "", hud: "" };
    const session = new ReadingSession(port, { css, surface: "book" });
    await session.start(null);
    for (const fn of runtimeListeners) fn({ type: "openDrawer", view: "review" }, {}, () => {});
    expect(document.getElementById("cymbra-lingua-drawer-host")?.getAttribute("lang")).toBe("fr");
  });

  it("picks the four modules by language, the French being each surface's default", () => {
    expect(readingCopy("fr").hud).toBe(frHud);
    expect(readingCopy("en").card).toBe(enCard);
    expect(readingCopy("es").selection).toBe(esSelection);
  });
});
