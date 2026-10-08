import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { card as enCard } from "@/i18n/en/card.ts";
import { grammar as enGrammar } from "@/i18n/en/grammar.ts";
import { hud as enHud } from "@/i18n/en/hud.ts";
import { selection as enSelection } from "@/i18n/en/selection.ts";
import { selection as esSelection } from "@/i18n/es/selection.ts";
import { grammar as esGrammar } from "@/i18n/es/grammar.ts";
import { grammar as frGrammar } from "@/i18n/fr/grammar.ts";
import { hud as frHud } from "@/i18n/fr/hud.ts";
import { createHud, LinguaHud } from "@/reading/hud.ts";
import { readingCopy } from "@/reading/reading-copy.ts";
import { rarityText, rowGloss, SelectionCards } from "@/reading/selection-card.ts";
import { ReadingSession } from "@/reading/session.ts";
import { createSpeaker, type VoiceInfo } from "@/reading/speech.ts";
import { createCard, WordPopup, type WordPopupContent, type WordPopupOptions } from "@/reading/wordpopup.ts";
import type { InterfaceLanguage } from "@/i18n/index.ts";
import type { ReviewView } from "@/review/session.ts";
import { renderReview } from "@/review/view.ts";
import { makeFakePort, makeFakeSpeech } from "./helpers.ts";

// The reading surfaces speak the interface language (localise-lingua-reading-surfaces): the HUD,
// the word card, the selection card's lines and the session that builds them, handed the copy and
// the language — what `hud`, `wordpopup`, `selection-card` and `rarity-text` assert in French,
// unchanged, asserted here in English and Spanish, with the hosts' `lang` — and the words of the
// document inside them, which say the studied language in theirs (D3).

/**
 * The language a text of `container` is read in — by a voice, a hyphenator, a spell-checker: the
 * `lang` of the innermost element holding exactly that text, or of its nearest ancestor that says
 * one, up through a shadow root to its host.
 */
function languageOfText(container: ParentNode, text: string): string | null {
  const holders = [...container.querySelectorAll("*")].filter((el) => el.textContent === text);
  let at: Element | null = holders.at(-1) ?? null;
  if (!at) throw new Error(`no element holds « ${text} »`);
  while (at) {
    const tagged: Element | null = at.closest("[lang]");
    if (tagged) return tagged.getAttribute("lang");
    const root = at.getRootNode();
    at = root instanceof ShadowRoot ? root.host : null;
  }
  return null;
}

/** A word card in its page host, its closed shadow root opened so the test can look inside. */
function popupInPage(opts: Partial<WordPopupOptions> = {}): { popup: WordPopup; root: ShadowRoot } {
  const attach = HTMLElement.prototype.attachShadow;
  let root: ShadowRoot | null = null;
  const spy = vi.spyOn(HTMLElement.prototype, "attachShadow").mockImplementation(function (this: HTMLElement) {
    root = attach.call(this, { mode: "open" });
    return root;
  });
  const popup = new WordPopup({ css: "", onGesture: () => {}, ...opts });
  spy.mockRestore();
  return { popup, root: root! };
}

/** A Spanish voice, so the card offers its listen buttons on a Spanish page. */
const monica: VoiceInfo = { name: "Mónica", lang: "es-ES", localService: true, default: false, voiceURI: "Mónica" };

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
  it("An English-native reader: on a Spanish page, the listen buttons and their names are the English catalogue's", () => {
    const fake = makeFakeSpeech([monica]);
    const speaker = createSpeaker(fake.engine, "es", fake.preference);
    const { popup, root } = popupInPage({ speaker, language: "en" });
    const listens = () =>
      [...root.querySelectorAll<HTMLButtonElement>(".listen button")].map((b) => [
        b.textContent,
        b.getAttribute("aria-label"),
      ]);
    const es = { language: "es", gloss: "être", rarity: "" } as const;

    popup.show(content({ ...es, headword: "ser", surface: "Es", sentence: "Es una casa." }));
    expect(listens()).toEqual([
      ["▶ Es", "Listen to the seen form “Es”"],
      ["▶ ser", "Listen to the dictionary form “ser”"],
      ["▶ Sentence", "Listen to the sentence"],
    ]);
    root.querySelector<HTMLButtonElement>(".listen button")!.click();
    expect(listens()[0]).toEqual(["■ Stop", "Stop reading"]);

    popup.show(content({ ...es, headword: "casa", surface: "casa", sentence: "Una casa grande." }));
    expect(listens()).toEqual([
      ["▶ Word", "Listen to the word"],
      ["▶ Sentence", "Listen to the sentence"],
    ]);
    popup.show(content({ ...es, headword: "casa grande", surface: "casa grande", expression: true }));
    expect(listens()[0]).toEqual(["▶ Selection", "Listen to the selection"]);
    expect(popup.host.getAttribute("lang")).toBe("en");
    popup.hide();
  });

  it("An English-native reader: the labels, the actions and the close control are the English catalogue's", () => {
    const card = createCard(undefined, "en");
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
    expect(new WordPopup({ css: "", onGesture: () => {}, language: "en" }).host.getAttribute("lang")).toBe("en");
    expect(new WordPopup({ css: "", onGesture: () => {} }).host.getAttribute("lang")).toBe("fr");
    expect(new WordPopup({ css: "", onGesture: () => {} }).host.id).toBe("cymbra-lingua-host");
  });

  it("On an English page with a French interface: the page's words say English, the card's labels French", () => {
    const { popup, root } = popupInPage({ speaker: undefined });
    popup.show(content({ status: null }));
    expect(languageOfText(root, "run")).toBe("en"); // the headword
    expect(languageOfText(root, "running")).toBe("en"); // the form seen…
    expect(languageOfText(root, "forme vue : « running »")).toBe("fr"); // …inside the French label
    expect(languageOfText(root, "Je connais")).toBe("fr");
    expect(languageOfText(root, "Ignorer")).toBe("fr");
    expect(languageOfText(root, "courir")).toBe("fr"); // the gloss is the reader's language

    popup.show(content({ gloss: null, rows: [{ form: "late", gloss: "tard" }] }));
    expect(languageOfText(root, "late → tard")).toBe("fr");
    expect(languageOfText(root, "late")).toBe("en");
    popup.hide();
  });

  it("names a Spanish page's words Spanish, whatever the interface language", () => {
    const { popup, root } = popupInPage({ language: "en" });
    popup.show(content({ headword: "ser", surface: "Es", gloss: "to be", language: "es" }));
    expect(languageOfText(root, "ser")).toBe("es");
    expect(languageOfText(root, "Es")).toBe("es");
    expect(languageOfText(root, "form seen: “Es”")).toBe("en");
    expect(languageOfText(root, "I know it")).toBe("en");
    popup.hide();
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

  it("cuts a long sense after a whole word with the catalogue's ellipsis, whatever the language", () => {
    const long = `${"word ".repeat(20)}end`; // 103 characters, cut within 80
    const kept = Array.from({ length: 16 }, () => "word").join(" ");
    expect(rowGloss(long)).toBe(`${kept}…`);
    expect(rowGloss(long, enSelection)).toBe(`${kept}…`);
    // The ellipsis is the module's, not the card's: a language writing it otherwise is followed.
    expect(rowGloss(long, { ...enSelection, truncated: (text) => `${text} [...]` })).toBe(`${kept} [...]`);
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

  it("gives an expression card the document's language, as a word's card has it", () => {
    const shown: WordPopupContent[] = [];
    const cards = new SelectionCards(
      {
        phraseGloss: () => new Promise(() => {}),
        gloss: async () => undefined,
        wordGrammar: () => new Promise(() => {}),
      },
      { show: (c) => shown.push(c), generation: () => shown.length },
      { clock: { setTimeout: () => 0, clearTimeout: () => {} }, language: () => "es" },
    );
    cards.openForSelection({ text: "casa grande", sentence: "Una casa grande.", rect: content().rect }, null);
    expect(shown.at(-1)).toMatchObject({ expression: true, headword: "casa grande", language: "es" });
  });
});

describe("the review card", () => {
  const actions = { start() {}, reveal() {}, grade() {}, markKnown() {} };
  const card = (over: Partial<NonNullable<ReviewView["card"]>> = {}): NonNullable<ReviewView["card"]> => ({
    headword: "faro",
    surface: "faro",
    sentence: "El faro brilla.",
    gloss: "phare",
    revealed: true,
    remaining: 3,
    language: "es",
    ...over,
  });

  /** The review as the drawer hosts it: inside a host that says the interface language. */
  function inHost(language: string): HTMLElement {
    const host = document.createElement("div");
    host.lang = language;
    const root = document.createElement("div");
    host.append(root);
    document.body.append(host);
    return root;
  }

  it("says the card's language on its headword and sentence, the interface's around them", () => {
    const root = inHost("fr");
    renderReview(root, { phase: "reviewing", card: card() }, actions);
    expect(languageOfText(root, "faro")).toBe("es");
    expect(languageOfText(root, "El faro brilla.")).toBe("es");
    expect(root.querySelector(".review-sentence")?.textContent).toBe("« El faro brilla. »");
    expect(languageOfText(root, "« El faro brilla. »")).toBe("fr");
    expect(languageOfText(root, "phare")).toBe("fr");
    expect(languageOfText(root, "Correct")).toBe("fr");
  });

  it("names an older card's words English, as the review counts them", () => {
    const root = inHost("fr");
    renderReview(root, { phase: "reviewing", card: card({ headword: "seldom", language: undefined }) }, actions);
    expect(languageOfText(root, "seldom")).toBe("en");
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

  it("picks the five modules by language, the French being each surface's default", () => {
    expect(readingCopy("fr").hud).toBe(frHud);
    expect(readingCopy("en").card).toBe(enCard);
    expect(readingCopy("es").selection).toBe(esSelection);
    expect(readingCopy("fr").grammar).toBe(frGrammar);
    expect(readingCopy("en").grammar).toBe(enGrammar);
    expect(readingCopy("es").grammar).toBe(esGrammar);
  });

  it("reads French for a language the catalogue lacks, never throwing", () => {
    const bogus = "xx" as InterfaceLanguage;
    expect(readingCopy(bogus)).toBe(readingCopy("fr"));
    // The card built on it: French labels and the French grammar line, as a French reader's.
    const card = createCard(undefined, bogus);
    card.show(
      content({
        headword: "go",
        surface: "went",
        grammar: {
          gloss: null,
          senses: [],
          readings: [{ pos: "VERB", features: { Mood: "Ind", Tense: "Past", VerbForm: "Fin" } }],
          others: [],
          pieces: [],
        },
      }),
      () => {},
    );
    expect(card.el.querySelector(".grammar-line")?.textContent).toBe("prétérit de go");
    expect(card.el.querySelector(".seen")?.textContent).toBe("forme vue : « went »");
  });
});
