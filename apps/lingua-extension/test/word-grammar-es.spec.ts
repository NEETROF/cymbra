import { describe, expect, it } from "vitest";
import type { GrammarTag, WordGrammar } from "@/analyzer/types.ts";
import { grammar as renderer } from "@/i18n/es/grammar.ts";
import { grammar as fr } from "@/i18n/fr/grammar.ts";
import { createCard, type WordPopupContent } from "@/reading/wordpopup.ts";

// generalise-lingua-card-wording: the Spanish renderer over the French spec's inputs
// (test/word-grammar.spec.ts), drafted for the owner's review (M9, D4) and shown to no reader until
// the interface speaks Spanish (change 20). It names what the French card names, in its own words,
// joining and order of tenses.

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

function shown(over: Partial<WordPopupContent>) {
  const card = createCard(undefined, "es");
  document.body.append(card.el);
  card.show(
    {
      headword: "go",
      surface: "went",
      gloss: "x",
      rarity: "",
      sentence: "",
      rect: { left: 0, top: 0, bottom: 0 },
      ...over,
    },
    () => {},
  );
  const lines = [...card.el.querySelectorAll(".grammar-line")].map((l) => l.textContent);
  const groups = [...card.el.querySelectorAll(".sense-group")].map((g) => g.querySelector(".pos")?.textContent ?? null);
  return { card, lines, groups };
}

describe("the word card in Spanish, of an English word", () => {
  it("An irregular form, its dictionary form marked as English", () => {
    const { lines, card } = shown({ grammar: grammar({ readings: [PAST] }) });
    expect(lines).toEqual(["pasado simple de go"]);
    const word = card.el.querySelector(".grammar-line em")!;
    expect(word.textContent).toBe("go");
    expect(word.getAttribute("lang")).toBe("en");
  });

  it("A form with two readings, in one statement", () => {
    const { lines } = shown({
      headword: "walk",
      surface: "walked",
      grammar: grammar({ readings: [PAST, PARTICIPLE] }),
    });
    expect(lines).toEqual(["pasado simple y participio de walk"]);
  });

  it("A form spelled like its dictionary form, as a possibility", () => {
    const { lines } = shown({ headword: "put", surface: "put", grammar: grammar({ readings: [PAST, PARTICIPLE] }) });
    expect(lines).toEqual(["tambi\u00e9n puede ser el pasado simple y el participio de put"]);
  });

  it("A form of two dictionary forms", () => {
    const { lines } = shown({
      headword: "leave",
      surface: "leaves",
      grammar: grammar({ readings: [THIRD], others: [{ lemma: "leaf", readings: [PLURAL] }] }),
    });
    expect(lines).toEqual([
      "tercera persona del singular del presente de leave",
      "tambi\u00e9n puede ser el plural de leaf",
    ]);
  });

  it("A contraction, and an inflected piece of one", () => {
    expect(
      shown({ headword: "do", surface: "do", written: "don't", grammar: grammar({ pieces: ["do", "not"] }) }).lines,
    ).toEqual(["\u00abdon't\u00bb = do + not"]);
    expect(
      shown({
        headword: "do",
        surface: "does",
        written: "doesn't",
        grammar: grammar({ pieces: ["does", "not"], readings: [THIRD] }),
      }).lines,
    ).toEqual(["\u00abdoesn't\u00bb = does + not", "tercera persona del singular del presente de do"]);
  });

  it("No code reaches the reader", () => {
    const odd: GrammarTag = { pos: "VERB", features: { Mood: "Sub", Tense: "Imp", VerbForm: "Fin" } };
    const { lines, card } = shown({
      grammar: grammar({ readings: [PAST, odd, { pos: "WORD" }], others: [{ lemma: "gone", readings: [odd] }] }),
    });
    expect(lines).toEqual(["pasado simple de go"]);
    expect(card.el.textContent).not.toMatch(/VERB|Tense=|Mood|VerbForm|gone/);
  });

  it("heads a group of senses with its part of speech, and a noun with its gender", () => {
    const { groups } = shown({
      headword: "can",
      surface: "can",
      gloss: "a; b; c",
      grammar: grammar({
        senses: [
          { tag: { pos: "NOUN", features: { Gender: "Fem" } }, text: "a" },
          { tag: { pos: "VERB" }, text: "b" },
          { tag: { pos: "SYM" }, text: "c" },
        ],
      }),
    });
    expect(groups).toEqual(["sustantivo femenino", "verbo", null]);
    expect(renderer.senseHeading({ pos: "NOUN" })).toBe("sustantivo");
    expect(renderer.senseHeading({ pos: "NOUN", features: { Gender: "Masc" } })).toBe("sustantivo masculino");
    expect(renderer.senseHeading({ pos: "PROPN" })).toBe("nombre propio");
    expect(renderer.senseHeading({ pos: "SCONJ" })).toBe("conjunci\u00f3n");
    expect(renderer.senseHeading({ pos: "X" })).toBeNull();
  });

  it("names English verb forms, plurals and degrees", () => {
    expect(renderer.readingName({ pos: "VERB", features: { VerbForm: "Ger" } })?.name).toBe("forma en -ing");
    expect(renderer.readingName(THIRD)?.name).toBe("tercera persona del singular del presente");
    expect(
      renderer.readingName({
        pos: "VERB",
        features: { Mood: "Ind", Number: "Sing", Person: "1", Tense: "Pres", VerbForm: "Fin" },
      })?.name,
    ).toBe("primera persona del singular del presente");
    expect(renderer.readingName({ pos: "VERB", features: { Mood: "Ind", Tense: "Pres", VerbForm: "Fin" } })?.name).toBe(
      "presente",
    );
    expect(renderer.readingName(PLURAL)?.name).toBe("plural");
    expect(renderer.readingName({ pos: "ADJ", features: { Degree: "Cmp" } })?.name).toBe("comparativo");
    expect(renderer.readingName({ pos: "ADV", features: { Degree: "Sup" } })?.name).toBe("superlativo");
    expect(renderer.readingName({ pos: "VERB", features: { VerbForm: "Inf" } })).toBeNull();
    expect(renderer.readingName({ pos: "PRON", features: { Case: "Dat" } })).toBeNull();
  });

  it("joins as the language does, and says each name once", () => {
    expect(renderer.join([])).toBe("");
    expect(renderer.join(["a"])).toBe("a");
    expect(renderer.join(["a", "b"])).toBe("a y b");
    expect(renderer.join(["a", "b", "c"])).toBe("a, b y c");
    const lines = renderer
      .grammarLines(
        grammar({ readings: [PAST, PAST], others: [{ lemma: "eat", readings: [PARTICIPLE] }] }),
        "ate",
        "ate",
        "ate",
      )
      .map(renderer.lineText);
    expect(lines).toEqual([
      "tambi\u00e9n puede ser el pasado simple de ate",
      "tambi\u00e9n puede ser el participio de eat",
    ]);
  });

  it("The order of tenses: English's come in the pack's order", () => {
    const pres: GrammarTag = { pos: "VERB", features: { Mood: "Ind", Tense: "Pres", VerbForm: "Fin" } };
    const of = (readings: GrammarTag[]) =>
      renderer.grammarLines(grammar({ readings }), "read", "read2", "read2").map(renderer.lineText);
    const [a, b] = [renderer.readingName(pres)!.name, renderer.readingName(PAST)!.name];
    expect(of([pres, PAST])[0]).toBe(`${renderer.join([a, b])} de read`);
    expect(of([PAST, pres])[0]).toBe(`${renderer.join([b, a])} de read`);
  });
});

const verb = (features: Record<string, string>): GrammarTag => ({ pos: "VERB", features });
const fin = (Mood: string, Person: string, Number: string, Tense?: string): GrammarTag =>
  verb({ Mood, Number, Person, ...(Tense ? { Tense } : {}), VerbForm: "Fin" });

describe("the word card in Spanish, of a Spanish word", () => {
  const spanish = (headword: string, surface: string, readings: GrammarTag[], others: WordGrammar["others"] = []) =>
    shown({ language: "es", headword, surface, grammar: grammar({ readings, others }) }).lines;

  it("A form of two persons, named once", () => {
    expect(spanish("hablar", "hablaba", [fin("Ind", "1", "Sing", "Imp"), fin("Ind", "3", "Sing", "Imp")])).toEqual([
      "primera y tercera persona del singular del pret\u00e9rito imperfecto de indicativo de hablar",
    ]);
  });

  it("The order of tenses: the subjunctive before the imperative, as the French", () => {
    const readings = [fin("Imp", "3", "Sing"), fin("Sub", "1", "Sing", "Pres"), fin("Sub", "3", "Sing", "Pres")];
    expect(spanish("hablar", "hable", readings)).toEqual([
      "primera y tercera persona del singular del presente de subjuntivo y tercera persona del singular del imperativo de hablar",
    ]);
    expect(fr.grammarLines(grammar({ readings }), "hablar", "hable", "hable", "es").map(fr.lineText)).toEqual([
      "1re et 3e personnes du singulier du présent du subjonctif et 3e personne du singulier de l’impératif de hablar",
    ]);
  });

  it("The order of numbers within a tense: the singular first", () => {
    const readings = [fin("Ind", "1", "Plur", "Pres"), fin("Ind", "3", "Sing", "Pres")];
    expect(spanish("hablar", "x", readings)).toEqual([
      "tercera persona del singular del presente de indicativo y primera persona del plural del presente de indicativo de hablar",
    ]);
    // The French puts the plural first, as it always did.
    expect(fr.grammarLines(grammar({ readings }), "hablar", "x", "x", "es").map(fr.lineText)).toEqual([
      "1re personne du pluriel du présent de l’indicatif et 3e personne du singulier du présent de l’indicatif de hablar",
    ]);
  });

  it("An adjective's agreement", () => {
    expect(spanish("rápido", "rápidas", [{ pos: "ADJ", features: { Gender: "Fem", Number: "Plur" } }])).toEqual([
      "femenino plural de r\u00e1pido",
    ]);
    expect(spanish("grande", "grandes", [{ pos: "ADJ", features: { Number: "Plur" } }])).toEqual(["plural de grande"]);
  });

  it("The dictionary form itself: no line, and the other dictionary form named", () => {
    expect(spanish("hablar", "hablar", [verb({ VerbForm: "Inf" })])).toEqual([]);
    expect(spanish("casa", "casa", [{ pos: "NOUN", features: { Gender: "Fem", Number: "Sing" } }])).toEqual([]);
    expect(
      spanish(
        "vino",
        "vino",
        [{ pos: "NOUN", features: { Gender: "Masc", Number: "Sing" } }],
        [{ lemma: "venir", readings: [fin("Ind", "3", "Sing", "Past")] }],
      ),
    ).toEqual([
      "tambi\u00e9n puede ser la tercera persona del singular del pret\u00e9rito perfecto simple de indicativo de venir",
    ]);
  });

  it("The Spanish interface on a Spanish word: « vino » of « venir », « venir » marked as Spanish", () => {
    const { lines, card } = shown({
      language: "es",
      headword: "venir",
      surface: "vino",
      grammar: grammar({ readings: [fin("Ind", "3", "Sing", "Past")] }),
    });
    expect(lines).toEqual(["tercera persona del singular del pret\u00e9rito perfecto simple de indicativo de venir"]);
    expect(card.el.querySelector(".grammar-line em")!.getAttribute("lang")).toBe("es");
    expect(card.el.querySelector(".headword")!.getAttribute("lang")).toBe("es");
  });

  it("A pronoun's form and a gerund: the language's own grammar, no elision", () => {
    expect(spanish("él", "ella", [{ pos: "PRON", features: { Gender: "Fem", Number: "Sing" } }])).toEqual([
      "femenino singular de \u00e9l",
    ]);
    expect(spanish("hablar", "hablando", [verb({ VerbForm: "Ger" })])).toEqual(["gerundio de hablar"]);
  });

  it("names every mood and tense, and a participle's agreement", () => {
    const name = (tag: GrammarTag) => renderer.readingName(tag, "es");
    expect(name(fin("Ind", "1", "Plur", "Pres"))?.name).toBe("primera persona del plural del presente de indicativo");
    expect(name(fin("Ind", "3", "Sing", "Past"))?.name).toBe(
      "tercera persona del singular del pret\u00e9rito perfecto simple de indicativo",
    );
    expect(name(fin("Ind", "2", "Sing", "Fut"))?.name).toBe(
      "segunda persona del singular del futuro simple de indicativo",
    );
    expect(name(fin("Cnd", "1", "Sing"))?.name).toBe("primera persona del singular del condicional simple");
    expect(name(fin("Sub", "3", "Plur", "Imp"))?.name).toBe(
      "tercera persona del plural del pret\u00e9rito imperfecto de subjuntivo",
    );
    expect(name(fin("Sub", "1", "Sing", "Fut"))?.name).toBe("primera persona del singular del futuro de subjuntivo");
    expect(name(verb({ VerbForm: "Inf" }))).toEqual({ article: "el", name: "infinitivo" });
    const part = (features: Record<string, string>) =>
      name(verb({ Tense: "Past", VerbForm: "Part", ...features }))?.name;
    expect(part({ Gender: "Masc", Number: "Sing" })).toBe("participio");
    expect(part({ Gender: "Fem", Number: "Sing" })).toBe("participio femenino");
    expect(part({ Gender: "Fem", Number: "Plur" })).toBe("participio femenino plural");
  });

  it("A card with no language names English forms", () => {
    expect(
      shown({
        headword: "hablar",
        surface: "hablaba",
        grammar: grammar({ readings: [fin("Ind", "1", "Sing", "Imp")] }),
      }).lines,
    ).toEqual([]);
  });
});
