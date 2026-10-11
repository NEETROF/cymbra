import { describe, expect, it } from "vitest";
import type { AnalyzedToken, PhraseGloss, PhraseToken, StudiedLanguage, WordGrammar } from "@/analyzer/types.ts";
import { selection as enSelection } from "@/i18n/en/selection.ts";
import {
  ANSWER_TIMEOUT_MS,
  GRAMMAR_WAIT_MS,
  type CardSurface,
  type Clock,
  type PageHit,
  rarityText,
  SelectionCards,
  type SelectionCardPorts,
  type SelectionInput,
  unknownToDocument,
} from "@/reading/selection-card.ts";
import type { TranslationRequest, TranslationResult, TranslatorPort } from "@/translate/port.ts";
import type { Gesture, WordPopupContent } from "@/reading/wordpopup.ts";

// read-lingua-card-in-another-studied-language, the cards' side (tasks 1.1–1.4, 4.1): a word the
// document's language does not know at all is read in the reader's other studied languages, on every
// path that opens a word card, within the card's bound — and no other card moves. The engine of each
// language is a table the test writes; a language can be held back to answer late.

const RECT = { left: 10, top: 20, bottom: 40 };
const SENTENCE = "Le verbe hablaba se conjugue ainsi.";

const tok = (over: Partial<PhraseToken> & Pick<PhraseToken, "surface" | "lemma" | "class">): PhraseToken => ({
  gloss: null,
  function_word: false,
  ...over,
});
const grammarOf = (over: Partial<WordGrammar> = {}): WordGrammar => ({
  gloss: null,
  senses: [],
  readings: [],
  others: [],
  pieces: [],
  rank: null,
  ...over,
});
/** What a pack that does not know a word at all answers about it. */
const EMPTY = grammarOf();
const pageToken = (over: Partial<AnalyzedToken> = {}): AnalyzedToken => ({
  block: 0,
  start: 0,
  end: 0,
  surface: "hablaba",
  lemma: "hablaba",
  class: "Unknown",
  gloss: null,
  ...over,
});
const hitOf = (token: AnalyzedToken): PageHit => ({ token, rect: RECT, sentence: SENTENCE });
const sel = (text: string): SelectionInput => ({ text, sentence: SENTENCE, rect: RECT });

/** Spanish's answer for « hablaba »: one token, glossed, with its reading and its rank. */
const HABLABA = tok({ surface: "hablaba", lemma: "hablar", class: "Unknown", gloss: "to talk, to speak" });
const HABLAR_GRAMMAR = grammarOf({
  gloss: "to talk, to speak",
  readings: [{ person: "1s", tense: "imperfect", mood: "indicative" } as unknown as WordGrammar["readings"][number]],
  rank: 181,
});

interface LanguageTable {
  phrase?: Record<string, PhraseToken[]>;
  grammar?: Record<string, WordGrammar>;
  gloss?: Record<string, string>;
  /** Hold every phrase gloss back until the test releases it. */
  held?: boolean;
  /** Every call rejects. */
  failing?: boolean;
}

/** One language's engine, answering from its table (an unknown text: no token). */
function engine(table: LanguageTable) {
  const asked = { phrase: [] as string[], grammar: [] as string[], gloss: [] as string[] };
  const releases: Array<() => void> = [];
  const ports: SelectionCardPorts = {
    phraseGloss: (text) => {
      asked.phrase.push(text);
      if (table.failing) return Promise.reject(new Error("engine down"));
      const answer: PhraseGloss = { tokens: table.phrase?.[text] ?? [] };
      if (!table.held) return Promise.resolve(answer);
      return new Promise((resolve) => releases.push(() => resolve(answer)));
    },
    gloss: (lemma) => {
      asked.gloss.push(lemma);
      return Promise.resolve(table.gloss?.[lemma]);
    },
    wordGrammar: (written, lemma) => {
      asked.grammar.push(`${written}/${lemma}`);
      if (table.failing) return Promise.reject(new Error("engine down"));
      return Promise.resolve(table.grammar?.[lemma] ?? grammarOf({ rank: 5 }));
    },
  };
  return { ports, asked, release: () => releases.splice(0).forEach((r) => r()) };
}

function fakeSurface() {
  const shows: WordPopupContent[] = [];
  let generation = 0;
  const surface: CardSurface = {
    show: (content) => {
      shows.push(content);
      return ++generation;
    },
    generation: () => generation,
  };
  return { surface, shows, last: () => shows.at(-1)!, hide: () => ++generation };
}

function fakeClock() {
  const timers = new Map<number, { fn: () => void; ms: number }>();
  let next = 0;
  const clock: Clock = {
    setTimeout: (fn, ms) => {
      timers.set(++next, { fn, ms });
      return next;
    },
    clearTimeout: (handle) => void timers.delete(handle as number),
  };
  return {
    clock,
    armed: () => [...timers.values()].map((t) => t.ms),
    elapseOnly: (ms: number) => {
      for (const [h, t] of [...timers.entries()]) {
        if (t.ms !== ms) continue;
        timers.delete(h);
        t.fn();
      }
    },
  };
}

const settle = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

/** The French document's engine: « hablaba », and the typo « parlaait », are nothing to it. */
const FRENCH: LanguageTable = {
  phrase: { hablaba: [tok({ surface: "hablaba", lemma: "hablaba", class: "Unknown" })] },
  grammar: { hablaba: EMPTY, parlaait: EMPTY },
};
/** Spanish, which reads « hablaba ». */
const SPANISH: LanguageTable = { phrase: { hablaba: [HABLABA] }, grammar: { hablar: HABLAR_GRAMMAR } };

function harness(
  opts: {
    document?: LanguageTable;
    others?: Partial<Record<StudiedLanguage, LanguageTable>>;
    order?: StudiedLanguage[];
    translator?: TranslatorPort;
    noPorts?: boolean;
  } = {},
) {
  const doc = engine(opts.document ?? FRENCH);
  const engines = new Map<StudiedLanguage, ReturnType<typeof engine>>();
  for (const [language, table] of Object.entries(opts.others ?? { es: SPANISH }))
    engines.set(language as StudiedLanguage, engine(table));
  const view = fakeSurface();
  const clock = fakeClock();
  const remembered: Array<[string, StudiedLanguage, string]> = [];
  const cards = new SelectionCards(doc.ports, view.surface, {
    clock: clock.clock,
    language: () => "fr",
    otherLanguages: () => opts.order ?? [...engines.keys()],
    ...(opts.noPorts
      ? {}
      : { portsFor: (language: StudiedLanguage) => engines.get(language)?.ports as SelectionCardPorts }),
    onReadElsewhere: (form, language, lemma) => remembered.push([form, language, lemma]),
    ...(opts.translator ? { translator: () => opts.translator! } : {}),
  });
  return { cards, doc, engines, es: engines.get("es"), remembered, ...view, ...clock };
}

/** The Spanish card of « hablaba », as the spec's first scenario reads it (a French interface). */
const SPANISH_CARD = {
  headword: "hablar",
  surface: "hablaba",
  gloss: "to talk, to speak",
  rarity: rarityText("Unknown", 181),
  sentence: SENTENCE,
  status: null,
  rect: RECT,
  language: "es",
  readIn: "es",
  grammar: HABLAR_GRAMMAR,
};

/** A translator that records what it is asked, and never answers. */
function recordingTranslator() {
  const asked: TranslationRequest[] = [];
  const translator: TranslatorPort = {
    translate: (request) => {
      asked.push(request);
      return new Promise<TranslationResult>(() => {});
    },
  };
  return { translator, asked };
}

describe("unknownToDocument (D1)", () => {
  it("is an unknown word with no gloss and an empty grammar answer", () => {
    expect(unknownToDocument("Unknown", null, EMPTY)).toBe(true);
  });

  it("is not a word the reader marked, a name, or a word with a gloss", () => {
    for (const cls of ["Known", "Learning", "Ignored", "ProperNounOutOfLexicon"] as const) {
      expect(unknownToDocument(cls, null, EMPTY)).toBe(false);
    }
    expect(unknownToDocument("Unknown", "parlait", EMPTY)).toBe(false);
  });

  it("is not a word the pack ranks, reads, glosses or knows another form of", () => {
    expect(unknownToDocument("Unknown", null, grammarOf({ rank: 30_000 }))).toBe(false);
    expect(unknownToDocument("Unknown", null, grammarOf({ gloss: "x" }))).toBe(false);
    expect(unknownToDocument("Unknown", null, HABLAR_GRAMMAR)).toBe(false);
    expect(
      unknownToDocument("Unknown", null, grammarOf({ others: [{} as unknown as WordGrammar["others"][number]] })),
    ).toBe(false);
  });

  it("is not an absent answer, nor one without a rank", () => {
    expect(unknownToDocument("Unknown", null, null)).toBe(false);
    expect(unknownToDocument("Unknown", null, undefined)).toBe(false);
    expect(unknownToDocument("Unknown", null, grammarOf({ rank: undefined }))).toBe(false);
  });
});

describe("a word the document's language does not know is read in another studied language", () => {
  it("A Spanish word on a French page: a click", async () => {
    const h = harness();
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(h.shows).toEqual([SPANISH_CARD]); // the French card is never shown
    expect(h.es!.asked.phrase).toEqual(["hablaba"]);
    expect(h.es!.asked.grammar).toEqual(["hablaba/hablar"]);
    expect(h.remembered).toEqual([["hablaba", "es", "hablar"]]);
  });

  it("A one-word selection over a page word", async () => {
    const h = harness();
    h.cards.openForSelection(sel("hablaba"), hitOf(pageToken()));
    await settle();
    expect(h.shows).toEqual([SPANISH_CARD]);
  });

  it("A one-word selection no page word covers", async () => {
    const h = harness();
    h.cards.openForSelection(sel("hablaba"), null);
    expect(h.last()).toMatchObject({ pending: true, headword: "hablaba" });
    await settle();
    expect(h.shows).toHaveLength(2);
    expect(h.last()).toEqual({ ...SPANISH_CARD, selection: undefined });
  });

  it("keeps the word as written on the page beside the form seen", async () => {
    const h = harness({ others: { es: { ...SPANISH, phrase: { Hablaba: [{ ...HABLABA, surface: "Hablaba" }] } } } });
    h.cards.openForToken({ ...hitOf(pageToken()), written: "Hablaba", selection: { start: 9, end: 16 } });
    await settle();
    expect(h.es!.asked.phrase).toEqual(["Hablaba"]);
    expect(h.last()).toMatchObject({ readIn: "es", surface: "Hablaba", selection: { start: 9, end: 16 } });
  });

  it("asks no translation for a re-read card", async () => {
    const t = recordingTranslator();
    const h = harness({ translator: t.translator });
    h.cards.openForToken({ ...hitOf(pageToken()), selection: { start: 9, end: 16 } });
    await settle();
    expect(h.last()).toMatchObject({ readIn: "es" });
    expect(h.last().translating).toBeUndefined();
    expect(t.asked).toHaveLength(0);
  });

  it("names English by leaving the card's language out, as an English card always did", async () => {
    const h = harness({ others: { en: { phrase: { hablaba: [tok({ ...HABLABA, lemma: "hablab" })] } } } });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(h.last().language).toBeUndefined();
    expect(h.last().readIn).toBe("en");
  });

  it("reads the status the reader gave the word in that language", async () => {
    const h = harness({ others: { es: { ...SPANISH, phrase: { hablaba: [{ ...HABLABA, class: "Known" }] } } } });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(h.last()).toMatchObject({ readIn: "es", status: "known" });
  });
});

describe("which language answers (D2)", () => {
  it("the first glossing language wins, in the profile's order", async () => {
    const english: LanguageTable = {
      phrase: { hablaba: [tok({ surface: "hablaba", lemma: "hablaba", class: "Unknown", gloss: "x" })] },
    };
    const h = harness({ others: { es: SPANISH, en: english }, order: ["es", "en"] });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(h.last().readIn).toBe("es");
    expect(h.engines.get("en")!.asked.phrase).toEqual([]);

    const g = harness({ others: { es: SPANISH, en: english }, order: ["en", "es"] });
    g.cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(g.last().readIn).toBe("en");
  });

  it("passes over an answer that is not one glossed word, and a failing engine", async () => {
    const answers: Array<PhraseToken[]> = [
      [], // no token
      [HABLABA, HABLABA], // several
      [{ ...HABLABA, class: "ProperNounOutOfLexicon" }], // a name
      [{ ...HABLABA, gloss: null }], // no gloss
    ];
    for (const tokens of answers) {
      const h = harness({ others: { es: { phrase: { hablaba: tokens } } } });
      h.cards.openForToken(hitOf(pageToken()));
      await settle();
      expect(h.shows).toHaveLength(1);
      expect(h.last().readIn).toBeUndefined();
    }
    const h = harness({ others: { es: { failing: true }, en: { phrase: { hablaba: [HABLABA] } } } });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(h.last()).toMatchObject({ readIn: "en", headword: "hablar" });
  });

  it("shows a re-read card without its grammar when that language's grammar fails", async () => {
    const es = engine(SPANISH);
    es.ports.wordGrammar = () => Promise.reject(new Error("down"));
    const view = fakeSurface();
    const cards = new SelectionCards(engine(FRENCH).ports, view.surface, {
      clock: fakeClock().clock,
      otherLanguages: () => ["es"],
      portsFor: () => es.ports,
    });
    cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(view.last()).toMatchObject({ headword: "hablar", readIn: "es", rarity: "" });
    expect(view.last().grammar).toBeUndefined();
  });

  it("reads nothing as several words, and without a view of the other language", async () => {
    const h = harness();
    await expect(h.cards.readElsewhere("yo hablaba")).resolves.toBeNull();
    await expect(h.cards.readElsewhere("  ")).resolves.toBeNull();
    const n = harness({ noPorts: true });
    await expect(n.cards.readElsewhere("hablaba")).resolves.toBeNull();
  });
});

describe("the cards that keep their card", () => {
  /** The card each case opens without any other language, and the one it opens with Spanish. */
  async function both(open: (cards: SelectionCards) => void, document: LanguageTable = FRENCH) {
    const before = harness({ document, others: {} });
    open(before.cards);
    await settle();
    const after = harness({ document });
    open(after.cards);
    await settle();
    return { before: before.shows, after: after.shows, spanish: after.es!.asked };
  }

  const cases: Array<[string, AnalyzedToken, LanguageTable?]> = [
    ["a known word", pageToken({ class: "Known" })],
    ["a word being learnt", pageToken({ class: "Learning" })],
    ["an ignored word", pageToken({ class: "Ignored" })],
    ["a name", pageToken({ class: "ProperNounOutOfLexicon" })],
    ["a word the French pack glosses", pageToken({ gloss: "il parlait" })],
    [
      "a word the French pack ranks without a gloss",
      pageToken(),
      { grammar: { hablaba: grammarOf({ rank: 30_000 }) } },
    ],
    [
      "a word the French pack reads",
      pageToken(),
      { grammar: { hablaba: grammarOf({ readings: HABLAR_GRAMMAR.readings }) } },
    ],
  ];
  for (const [name, token, document] of cases) {
    it(`${name}: its card is the one it was, and Spanish is not asked`, async () => {
      const { before, after, spanish } = await both((cards) => cards.openForToken(hitOf(token)), document);
      expect(after).toEqual(before);
      expect(spanish.phrase).toEqual([]);
    });
  }

  it("a word no other language glosses (a French typo): the card it was", async () => {
    const before = harness({ others: {} });
    before.cards.openForToken(hitOf(pageToken({ surface: "parlaait", lemma: "parlaait" })));
    await settle();
    const after = harness();
    after.cards.openForToken(hitOf(pageToken({ surface: "parlaait", lemma: "parlaait" })));
    await settle();
    expect(after.shows).toEqual(before.shows);
    expect(after.last().rarity).toBe("Rare — au-delà des 20\u202f000 mots les plus fréquents.");
    expect(after.es!.asked.phrase).toEqual(["parlaait"]);
  });

  it("Several words: the whole-selection card, read in the document's language", async () => {
    const h = harness();
    h.cards.openForSelection(sel("yo hablaba"), null);
    await settle();
    expect(h.es!.asked.phrase).toEqual([]);
    expect(h.last().readIn).toBeUndefined();
  });

  it("One studied language: no other language is asked", async () => {
    const h = harness({ order: [] });
    h.cards.openForToken(hitOf(pageToken()));
    h.cards.openForSelection(sel("hablaba"), null);
    await settle();
    expect(h.es!.asked.phrase).toEqual([]);
    expect(h.shows.every((card) => card.readIn === undefined)).toBe(true);
  });

  it("a loose word the French pack ranks keeps its card", async () => {
    const h = harness({ document: { ...FRENCH, grammar: { hablaba: grammarOf({ rank: 9_000 }) } } });
    h.cards.openForSelection(sel("hablaba"), null);
    await settle();
    expect(h.last()).toMatchObject({ headword: "hablaba", rarity: rarityText("Unknown", 9_000) });
    expect(h.es!.asked.phrase).toEqual([]);
  });
});

describe("within the card's bound (D3)", () => {
  it("The Spanish pack not loaded yet: the card as before, then the Spanish card", async () => {
    const h = harness({ others: { es: { ...SPANISH, held: true } } });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    expect(h.shows).toHaveLength(0); // the re-read is waited for, the card never drawn pending
    expect(h.armed()).toEqual([GRAMMAR_WAIT_MS]);
    h.elapseOnly(GRAMMAR_WAIT_MS);
    await settle();
    expect(h.shows).toHaveLength(1);
    expect(h.last()).toMatchObject({ headword: "hablaba", rarity: rarityText("Unknown", null) });
    expect(h.last().readIn).toBeUndefined();
    h.es!.release();
    await settle();
    expect(h.shows).toHaveLength(2);
    expect(h.last()).toEqual(SPANISH_CARD);
    expect(h.remembered).toEqual([["hablaba", "es", "hablar"]]);
  });

  it("a late re-read lands nowhere once the reader closed the card or opened another", async () => {
    const h = harness({ others: { es: { ...SPANISH, held: true } } });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    h.elapseOnly(GRAMMAR_WAIT_MS);
    await settle();
    h.hide();
    h.es!.release();
    await settle();
    expect(h.shows).toHaveLength(1);
    expect(h.remembered).toEqual([]);
  });

  it("a late re-read that finds nothing leaves the card", async () => {
    const h = harness({ others: { es: { held: true } } });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    h.elapseOnly(GRAMMAR_WAIT_MS);
    await settle();
    h.es!.release();
    await settle();
    expect(h.shows).toHaveLength(1);
  });

  it("a card closed while the re-read was waited for is never shown", async () => {
    const h = harness({ others: { es: { ...SPANISH, held: true } } });
    h.cards.openForToken(hitOf(pageToken()));
    await settle();
    h.hide();
    h.es!.release();
    await settle();
    expect(h.shows).toHaveLength(0);
  });

  it("a late re-read keeps the translation the document's card asked, until it lands", async () => {
    const t = recordingTranslator();
    const h = harness({ others: { es: { ...SPANISH, held: true } }, translator: t.translator });
    h.cards.openForToken({ ...hitOf(pageToken()), selection: { start: 9, end: 16 } });
    await settle();
    h.elapseOnly(GRAMMAR_WAIT_MS);
    await settle();
    expect(h.last()).toMatchObject({ translating: true });
    expect(t.asked).toHaveLength(1);
    h.es!.release();
    await settle();
    expect(h.last()).toMatchObject({ readIn: "es" });
    expect(h.last().translating).toBeUndefined();
  });

  it("A loose word: pending, then the card of today past the bound, then the Spanish card", async () => {
    const h = harness({ others: { es: { ...SPANISH, held: true } } });
    h.cards.openForSelection(sel("hablaba"), null);
    await settle();
    expect(h.armed()).toEqual([ANSWER_TIMEOUT_MS, GRAMMAR_WAIT_MS]);
    h.elapseOnly(GRAMMAR_WAIT_MS);
    await settle();
    expect(h.last()).toMatchObject({ headword: "hablaba", rarity: rarityText("Unknown", null) });
    expect(h.last().readIn).toBeUndefined();
    h.es!.release();
    await settle();
    expect(h.last()).toEqual({ ...SPANISH_CARD, selection: undefined });
  });
});

describe("what a re-read card records (D5)", () => {
  const g = (over: Partial<Gesture>): Gesture => ({
    lemma: "hablar",
    surface: "hablaba",
    sentence: SENTENCE,
    status: "learning",
    expression: false,
    gloss: "to talk, to speak",
    ...over,
  });

  it("stores the gloss of the language the card was read in", async () => {
    const h = harness({ others: { es: { gloss: { hablar: "to talk, to speak" } } } });
    await expect(h.cards.cardGloss(g({ readIn: "es" }))).resolves.toBe("to talk, to speak");
    expect(h.es!.asked.gloss).toEqual(["hablar"]);
    expect(h.doc.asked.gloss).toEqual([]);
  });

  it("asks the document's language for every other card", async () => {
    const h = harness();
    await h.cards.cardGloss(g({}));
    expect(h.doc.asked.gloss).toEqual(["hablar"]);
    expect(h.es!.asked.gloss).toEqual([]);
  });
});

describe("the language line's copy (D4)", () => {
  it("takes the language's form after the noun", () => {
    expect(enSelection.readAs("Spanish")).toBe("Spanish word");
  });
});
