import { describe, expect, it } from "vitest";
import type { GrammarTag, WordGrammar } from "@/analyzer/types.ts";
import { createCard, type WordPopupContent } from "@/reading/wordpopup.ts";
import { grammarLines, joinFrench, lineText, readingName, senseHeading } from "@/reading/grammar-labels.ts";
import { glossPages, PAGE_CHARS, pageText } from "@/reading/gloss-pages.ts";

// The scenarios of the two card requirements add-lingua-word-grammar adds to
// lingua-browser-extension ("The word card says what the form is", "The word card lays its
// gloss out by part of speech"), rendered through the real card view, and the labels that turn
// the pack's Universal Dependencies codes into French words.

const PAST: GrammarTag = { pos: "VERB", features: { Mood: "Ind", Tense: "Past", VerbForm: "Fin" } };
const PARTICIPLE: GrammarTag = { pos: "VERB", features: { Tense: "Past", VerbForm: "Part" } };
const THIRD: GrammarTag = {
  pos: "VERB",
  features: { Mood: "Ind", Number: "Sing", Person: "3", Tense: "Pres", VerbForm: "Fin" },
};
const PLURAL: GrammarTag = { pos: "NOUN", features: { Number: "Plur" } };

const grammar = (over: Partial<WordGrammar> = {}): WordGrammar => ({
  gloss: null,
  senses: [],
  readings: [],
  others: [],
  pieces: [],
  ...over,
});

const content = (over: Partial<WordPopupContent>): WordPopupContent => ({
  headword: "go",
  surface: "went",
  gloss: "Aller",
  rarity: "Peu fréquent — au-delà de ton niveau.",
  sentence: "They went home.",
  rect: { left: 40, top: 60, bottom: 80 },
  ...over,
});

function shown(over: Partial<WordPopupContent>) {
  const card = createCard();
  document.body.append(card.el);
  card.show(content(over), () => {});
  const lines = [...card.el.querySelectorAll(".grammar-line")].map((l) => l.textContent);
  const groups = [...card.el.querySelectorAll(".sense-group")].map((g) => ({
    heading: g.querySelector(".pos")?.textContent ?? null,
    text: g.textContent,
  }));
  return { card, lines, groups, text: card.el.textContent ?? "" };
}

describe("the word card says what the form is", () => {
  it("An irregular form", () => {
    const { lines, card } = shown({ grammar: grammar({ readings: [PAST] }) });
    expect(lines).toEqual(["prétérit de go"]);
    expect(card.el.querySelector(".seen")!.textContent).toBe("forme vue : « went »");
    // The studied language's word is set apart from the French around it.
    expect(card.el.querySelector(".grammar-line em")!.textContent).toBe("go");
  });

  it("A form with two readings, in one statement", () => {
    const { lines } = shown({
      headword: "walk",
      surface: "walked",
      grammar: grammar({ readings: [PAST, PARTICIPLE] }),
    });
    expect(lines).toEqual(["prétérit et participe passé de walk"]);
  });

  it("A form spelled like its dictionary form, as a possibility", () => {
    const { lines, card } = shown({
      headword: "put",
      surface: "put",
      grammar: grammar({ readings: [PAST, PARTICIPLE] }),
    });
    expect(lines).toEqual(["peut aussi être le prétérit et le participe passé de put"]);
    expect((card.el.querySelector(".seen") as HTMLElement).hidden).toBe(true);
  });

  it("A form of two dictionary forms: the other is named, and the actions stay on this one", () => {
    const { lines, card } = shown({
      headword: "leave",
      surface: "leaves",
      grammar: grammar({ readings: [THIRD], others: [{ lemma: "leaf", readings: [PLURAL] }] }),
    });
    expect(lines).toEqual(["3e personne du singulier du présent de leave", "peut aussi être le pluriel de leaf"]);
    // Text only: nothing in the grammar block can be pressed.
    expect(card.el.querySelector(".grammar button, .grammar a")).toBeNull();
    const buttons = [...card.el.querySelectorAll(".actions button")].map((b) => b.textContent);
    expect(buttons).toContain("+ Deck");
  });

  it("A contraction", () => {
    const { lines } = shown({
      headword: "do",
      surface: "do",
      written: "don't",
      grammar: grammar({ pieces: ["do", "not"] }),
    });
    expect(lines).toEqual(["« don't » = do + not"]);
  });

  it("an inflected piece of a contraction says what it is too", () => {
    const { lines } = shown({
      headword: "do",
      surface: "does",
      written: "doesn't",
      grammar: grammar({ pieces: ["does", "not"], readings: [THIRD] }),
    });
    expect(lines).toEqual(["« doesn't » = does + not", "3e personne du singulier du présent de do"]);
  });

  it("A pack without grammar tables: the card of before", () => {
    const { lines, card } = shown({});
    expect(lines).toEqual([]);
    expect((card.el.querySelector(".grammar") as HTMLElement).hidden).toBe(true);
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Aller");
  });

  it("No code reaches the reader, and no 'lemma'", () => {
    const odd: GrammarTag = { pos: "VERB", features: { Mood: "Sub", Tense: "Imp", VerbForm: "Fin" } };
    const { text } = shown({
      grammar: grammar({
        readings: [PAST, odd, { pos: "WORD" }],
        others: [{ lemma: "gone", readings: [odd] }],
        senses: [{ tag: { pos: "SYM" }, text: "Aller" }],
      }),
    });
    expect(text).not.toMatch(/VERB|NOUN|SYM|Tense=|Mood|VerbForm|Past|Sub\b/);
    expect(text).not.toMatch(/lemm/i);
    expect(text).toContain("prétérit de go");
    expect(text).not.toContain("gone"); // an other word with nothing nameable is left out
  });

  it("a pending card shows no grammar, and the grammar sits below the listen row", () => {
    const { lines, card } = shown({ pending: true, grammar: grammar({ readings: [PAST] }) });
    expect(lines).toEqual([]);
    const order = [...card.el.children].map((el) => el.className);
    expect(order.indexOf("grammar")).toBeGreaterThan(order.indexOf("listen"));
    expect(order.indexOf("grammar")).toBeLessThan(order.indexOf("actions"));
  });

  it("an expression card has no grammar block", () => {
    const { lines } = shown({
      expression: true,
      headword: "give up",
      surface: "gave up",
      grammar: grammar({ readings: [PAST] }),
    });
    expect(lines).toEqual([]);
  });
});

describe("the word card lays its gloss out by part of speech", () => {
  it("A word with a noun sense and verb senses", () => {
    const { groups, card } = shown({
      headword: "can",
      surface: "can",
      gloss: "Boîte de conserve; Pouvoir, savoir",
      grammar: grammar({
        senses: [
          { tag: { pos: "NOUN" }, text: "Boîte de conserve" },
          { tag: { pos: "VERB" }, text: "Pouvoir, savoir" },
        ],
      }),
    });
    expect(groups).toEqual([
      { heading: "nom", text: "nom Boîte de conserve" },
      { heading: "verbe", text: "verbe Pouvoir, savoir" },
    ]);
    expect(card.el.querySelector(".gloss")!.classList.contains("empty")).toBe(false);
  });

  it("A word with one part of speech", () => {
    const { groups } = shown({
      headword: "put",
      surface: "put",
      gloss: "Mettre",
      grammar: grammar({ senses: [{ tag: { pos: "VERB" }, text: "Mettre" }] }),
    });
    expect(groups).toEqual([{ heading: "verbe", text: "verbe Mettre" }]);
  });

  it("A gendered noun", () => {
    const { groups } = shown({
      headword: "leche",
      surface: "leche",
      gloss: "Lait",
      grammar: grammar({ senses: [{ tag: { pos: "NOUN", features: { Gender: "Fem" } }, text: "Lait" }] }),
    });
    expect(groups[0]!.heading).toBe("nom féminin");
  });

  it("A sense with no part of speech to name shows without a heading", () => {
    const { groups } = shown({
      headword: "a",
      surface: "a",
      gloss: "Première lettre; La (note de musique)",
      grammar: grammar({
        senses: [
          { tag: { pos: "SYM" }, text: "Première lettre" },
          { tag: { pos: "NOUN" }, text: "La (note de musique)" },
        ],
      }),
    });
    expect(groups).toEqual([
      { heading: null, text: "Première lettre" },
      { heading: "nom", text: "nom La (note de musique)" },
    ]);
  });

  it("groups that do not make up the gloss the card holds are not used", () => {
    const { groups, card } = shown({
      gloss: "Aller; Tour",
      grammar: grammar({ senses: [{ tag: { pos: "VERB" }, text: "Aller" }] }),
    });
    expect(groups).toEqual([]);
    expect(card.el.querySelector(".gloss")!.textContent).toBe("Aller; Tour");
  });

  it("The stored gloss does not change shape: a gesture carries the flat gloss", () => {
    const card = createCard();
    const gestures: Array<{ gloss: string | null }> = [];
    card.show(
      content({
        headword: "can",
        surface: "can",
        gloss: "Boîte de conserve; Pouvoir, savoir",
        grammar: grammar({
          senses: [
            { tag: { pos: "NOUN" }, text: "Boîte de conserve" },
            { tag: { pos: "VERB" }, text: "Pouvoir, savoir" },
          ],
        }),
      }),
      (g) => gestures.push(g),
    );
    const deck = [...card.el.querySelectorAll("button")].find((b) => b.textContent === "+ Deck")!;
    deck.click();
    expect(gestures[0]!.gloss).toBe("Boîte de conserve; Pouvoir, savoir");
    expect(Object.keys(gestures[0]!)).not.toContain("grammar");
  });
});

describe("the word card pages a long gloss", () => {
  // `be` as the re-reduced pack glosses it (five whole senses, more than a page), with a noun
  // sense added so that a page holds two parts of speech.
  const BE = [
    "Être",
    "Être. Auxiliaire pour former le passif avec un participe passé",
    "Être en train de. Auxiliaire pour former le progressif avec un participe présent",
    "Aller. Auxiliaire pour former le futur proche avec un infinitif",
    "Avoir l’habitude de. Auxiliaire marquant l’aspect habituel",
    "Existence",
  ];
  const FIRST_PAGE = BE.slice(0, 3).join("; ");
  const beCard = () =>
    shown({
      headword: "be",
      surface: "be",
      gloss: BE.join("; "),
      grammar: grammar({
        senses: [
          { tag: { pos: "VERB" }, text: BE.slice(0, 5).join("; ") },
          { tag: { pos: "NOUN" }, text: BE[5]! },
        ],
      }),
    });
  const texts = (card: ReturnType<typeof shown>["card"]) => ({
    groups: [...card.el.querySelectorAll(".sense-group")].map((g) => g.textContent),
    count: card.el.querySelector(".page-count")?.textContent ?? null,
    previous: card.el.querySelector<HTMLButtonElement>('.gloss-nav button[aria-label="Sens précédents"]'),
    next: card.el.querySelector<HTMLButtonElement>('.gloss-nav button[aria-label="Sens suivants"]'),
  });

  it("A word with more senses than one page holds", () => {
    const { card } = beCard();
    const page = texts(card);
    expect(page.groups).toEqual([`verbe ${FIRST_PAGE}`]);
    expect(page.count).toBe("1/2");
    expect(page.previous!.disabled).toBe(true);
    expect(page.next!.disabled).toBe(false);
    expect(page.next!.textContent).toBe("Suivant ›");
  });

  it("Moving to the next page, and back", () => {
    const { card } = beCard();
    const actions = () => [...card.el.querySelectorAll(".actions button")].map((b) => b.textContent);
    const before = actions();
    texts(card).next!.click();
    let page = texts(card);
    // The part of speech is said again on the page where its senses continue.
    expect(page.groups).toEqual([`verbe ${BE[3]}; ${BE[4]}`, `nom ${BE[5]}`]);
    expect(page.count).toBe("2/2");
    expect(page.next!.disabled).toBe(true);
    expect(page.previous!.disabled).toBe(false);
    expect(actions()).toEqual(before);
    expect(card.visible()).toBe(true);
    page.previous!.click();
    page = texts(card);
    expect(page.count).toBe("1/2");
  });

  it("a paging press keeps the reader's selection", () => {
    const { card } = beCard();
    const down = new Event("mousedown", { cancelable: true });
    texts(card).next!.dispatchEvent(down);
    expect(down.defaultPrevented).toBe(true);
  });

  it("A card opened again starts on its first page", () => {
    const { card } = beCard();
    texts(card).next!.click();
    card.show(
      content({ headword: "be", surface: "be", gloss: BE.join("; "), grammar: grammar({ senses: [] }) }),
      () => {},
    );
    expect(texts(card).count).toBe("1/2");
    expect(card.el.querySelector(".gloss-page")!.textContent).toBe(FIRST_PAGE);
  });

  it("A gloss that fits shows no paging control", () => {
    const { card } = shown({ gloss: "Aller" });
    expect(card.el.querySelector(".gloss-nav")).toBeNull();
  });

  it("a card created from a paged gloss stores its first page, as one text", () => {
    const { card } = beCard();
    const gestures: Array<{ gloss: string | null }> = [];
    card.show(
      content({
        headword: "be",
        surface: "be",
        gloss: BE.join("; "),
        grammar: grammar({ senses: [{ tag: { pos: "AUX" }, text: BE.join("; ") }] }),
      }),
      (g) => gestures.push(g),
    );
    texts(card).next!.click();
    [...card.el.querySelectorAll("button")].find((b) => b.textContent === "+ Deck")!.click();
    expect(gestures[0]!.gloss).toBe(FIRST_PAGE);
  });
});

describe("glossPages", () => {
  it("fills a page with whole senses up to its size", () => {
    const pages = glossPages("aaaa; bbbb; cccc", [], 10);
    expect(pages.map(pageText)).toEqual(["aaaa; bbbb", "cccc"]);
    expect(pages[0]![0]!.tagged).toBe(false);
  });

  it("puts a sense longer than a page on a page of its own, whole", () => {
    const long = "x".repeat(PAGE_CHARS + 20);
    expect(glossPages(`Court; ${long}; Fin`).map(pageText)).toEqual(["Court", long, "Fin"]);
  });

  it("uses the grammar's groups only when they make up the gloss", () => {
    const tagged = glossPages("Boîte; Pouvoir", [
      { tag: { pos: "NOUN" }, text: "Boîte" },
      { tag: { pos: "VERB" }, text: "Pouvoir" },
    ]);
    expect(tagged).toEqual([
      [
        { tagged: true, tag: { pos: "NOUN" }, senses: ["Boîte"] },
        { tagged: true, tag: { pos: "VERB" }, senses: ["Pouvoir"] },
      ],
    ]);
    const stale = glossPages("Boîte; Pouvoir", [{ tag: { pos: "NOUN" }, text: "Boîte" }]);
    expect(stale).toEqual([[{ tagged: false, senses: ["Boîte", "Pouvoir"] }]]);
  });
});

describe("grammar labels", () => {
  it("names English verb forms as French schools do", () => {
    expect(readingName(PAST)?.name).toBe("prétérit");
    expect(readingName(PARTICIPLE)?.name).toBe("participe passé");
    expect(readingName({ pos: "VERB", features: { VerbForm: "Ger" } })?.name).toBe("forme en -ing");
    expect(readingName(THIRD)?.name).toBe("3e personne du singulier du présent");
    expect(
      readingName({
        pos: "VERB",
        features: { Mood: "Ind", Number: "Sing", Person: "1", Tense: "Pres", VerbForm: "Fin" },
      })?.name,
    ).toBe("1re personne du singulier du présent");
    expect(readingName({ pos: "VERB", features: { Mood: "Ind", Tense: "Pres", VerbForm: "Fin" } })?.name).toBe(
      "présent",
    );
    expect(readingName({ pos: "AUX", features: { Tense: "Past", VerbForm: "Fin" } })?.name).toBe("prétérit");
  });

  it("names plurals and degrees", () => {
    expect(readingName(PLURAL)?.name).toBe("pluriel");
    expect(readingName({ pos: "ADJ", features: { Degree: "Cmp" } })?.name).toBe("comparatif");
    expect(readingName({ pos: "ADV", features: { Degree: "Sup" } })?.name).toBe("superlatif");
  });

  it("names nothing it cannot say rather than show a code", () => {
    expect(readingName({ pos: "VERB", features: { Mood: "Sub", Tense: "Imp", VerbForm: "Fin" } })).toBeNull();
    expect(readingName({ pos: "VERB", features: { VerbForm: "Inf" } })).toBeNull();
    expect(readingName({ pos: "VERB", features: { Tense: "Pres", VerbForm: "Part" } })).toBeNull();
    expect(readingName({ pos: "VERB", features: { Tense: "Fut", VerbForm: "Fin" } })).toBeNull();
    expect(readingName({ pos: "NOUN", features: { Number: "Sing" } })).toBeNull();
    expect(readingName({ pos: "NOUN" })).toBeNull();
    expect(readingName({ pos: "PRON", features: { Case: "Dat" } })).toBeNull();
    expect(readingName({ pos: "VERB" })).toBeNull();
  });

  it("heads a group of senses with its part of speech, and a noun with its gender", () => {
    expect(senseHeading({ pos: "NOUN" })).toBe("nom");
    expect(senseHeading({ pos: "PROPN" })).toBe("nom propre");
    expect(senseHeading({ pos: "SCONJ" })).toBe("conjonction");
    expect(senseHeading({ pos: "NOUN", features: { Gender: "Masc" } })).toBe("nom masculin");
    expect(senseHeading({ pos: "NOUN", features: { Gender: "Other" } })).toBe("nom");
    expect(senseHeading({ pos: "X" })).toBeNull();
    expect(senseHeading(undefined)).toBeNull();
  });

  it("joins as French does", () => {
    expect(joinFrench([])).toBe("");
    expect(joinFrench(["a"])).toBe("a");
    expect(joinFrench(["a", "b"])).toBe("a et b");
    expect(joinFrench(["a", "b", "c"])).toBe("a, b et c");
  });

  it("elides before a vowel, and says each name once", () => {
    const lines = grammarLines(
      grammar({ readings: [PAST, PAST], others: [{ lemma: "eat", readings: [PARTICIPLE] }] }),
      "ate",
      "ate",
      "ate",
    ).map(lineText);
    expect(lines).toEqual(["peut aussi être le prétérit d’ate", "peut aussi être le participe passé d’eat"]);
  });
});
