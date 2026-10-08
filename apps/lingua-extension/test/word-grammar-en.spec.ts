import { describe, expect, it } from "vitest";
import type { GrammarTag, WordGrammar } from "@/analyzer/types.ts";
import { grammar as renderer } from "@/i18n/en/grammar.ts";
import { grammar as fr } from "@/i18n/fr/grammar.ts";
import { lineText } from "@/reading/grammar-description.ts";
import { createCard, type WordPopupContent } from "@/reading/wordpopup.ts";

// generalise-lingua-card-wording: the English renderer over the French spec's inputs
// (test/word-grammar.spec.ts), drafted for the owner's review (M9, D4) and shown to no reader until
// the interface speaks English (change 20). It names what the French card names, in its own words,
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
  const card = createCard(undefined, "en");
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

describe("the word card in English, of an English word", () => {
  it("An irregular form, its dictionary form marked as English", () => {
    const { lines, card } = shown({ grammar: grammar({ readings: [PAST] }) });
    expect(lines).toEqual(["simple past of go"]);
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
    expect(lines).toEqual(["simple past and past participle of walk"]);
  });

  it("A form spelled like its dictionary form, as a possibility", () => {
    const { lines } = shown({ headword: "put", surface: "put", grammar: grammar({ readings: [PAST, PARTICIPLE] }) });
    expect(lines).toEqual(["may also be the simple past and past participle of put"]);
  });

  it("A form of two dictionary forms", () => {
    const { lines } = shown({
      headword: "leave",
      surface: "leaves",
      grammar: grammar({ readings: [THIRD], others: [{ lemma: "leaf", readings: [PLURAL] }] }),
    });
    expect(lines).toEqual(["third-person singular simple present of leave", "may also be the plural of leaf"]);
  });

  it("A contraction, and an inflected piece of one", () => {
    expect(
      shown({ headword: "do", surface: "do", written: "don't", grammar: grammar({ pieces: ["do", "not"] }) }).lines,
    ).toEqual(["\u201cdon't\u201d = do + not"]);
    expect(
      shown({
        headword: "do",
        surface: "does",
        written: "doesn't",
        grammar: grammar({ pieces: ["does", "not"], readings: [THIRD] }),
      }).lines,
    ).toEqual(["\u201cdoesn't\u201d = does + not", "third-person singular simple present of do"]);
  });

  it("No code reaches the reader", () => {
    const odd: GrammarTag = { pos: "VERB", features: { Mood: "Sub", Tense: "Imp", VerbForm: "Fin" } };
    const { lines, card } = shown({
      grammar: grammar({ readings: [PAST, odd, { pos: "WORD" }], others: [{ lemma: "gone", readings: [odd] }] }),
    });
    expect(lines).toEqual(["simple past of go"]);
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
    expect(groups).toEqual(["feminine noun", "verb", null]);
    expect(renderer.senseHeading({ pos: "NOUN" })).toBe("noun");
    expect(renderer.senseHeading({ pos: "NOUN", features: { Gender: "Masc" } })).toBe("masculine noun");
    expect(renderer.senseHeading({ pos: "PROPN" })).toBe("proper noun");
    expect(renderer.senseHeading({ pos: "SCONJ" })).toBe("conjunction");
    expect(renderer.senseHeading({ pos: "X" })).toBeNull();
  });

  it("names English verb forms, plurals and degrees", () => {
    expect(renderer.readingName({ pos: "VERB", features: { VerbForm: "Ger" } })?.name).toBe("-ing form");
    expect(renderer.readingName(THIRD)?.name).toBe("third-person singular simple present");
    expect(
      renderer.readingName({
        pos: "VERB",
        features: { Mood: "Ind", Number: "Sing", Person: "1", Tense: "Pres", VerbForm: "Fin" },
      })?.name,
    ).toBe("first-person singular simple present");
    expect(renderer.readingName({ pos: "VERB", features: { Mood: "Ind", Tense: "Pres", VerbForm: "Fin" } })?.name).toBe(
      "simple present",
    );
    expect(renderer.readingName(PLURAL)?.name).toBe("plural");
    expect(renderer.readingName({ pos: "ADJ", features: { Degree: "Cmp" } })?.name).toBe("comparative");
    expect(renderer.readingName({ pos: "ADV", features: { Degree: "Sup" } })?.name).toBe("superlative");
    expect(renderer.readingName({ pos: "VERB", features: { VerbForm: "Inf" } })).toBeNull();
    expect(renderer.readingName({ pos: "PRON", features: { Case: "Dat" } })).toBeNull();
  });

  it("joins as the language does, and says each name once", () => {
    expect(renderer.join([])).toBe("");
    expect(renderer.join(["a"])).toBe("a");
    expect(renderer.join(["a", "b"])).toBe("a and b");
    expect(renderer.join(["a", "b", "c"])).toBe("a, b and c");
    const lines = renderer
      .grammarLines(
        grammar({ readings: [PAST, PAST], others: [{ lemma: "eat", readings: [PARTICIPLE] }] }),
        "ate",
        "ate",
        "ate",
      )
      .map(lineText);
    expect(lines).toEqual(["may also be the simple past of ate", "may also be the past participle of eat"]);
  });

  it("The order of tenses: English's come in the pack's order", () => {
    const pres: GrammarTag = { pos: "VERB", features: { Mood: "Ind", Tense: "Pres", VerbForm: "Fin" } };
    const of = (readings: GrammarTag[]) =>
      renderer.grammarLines(grammar({ readings }), "read", "read2", "read2").map(lineText);
    const [a, b] = [renderer.readingName(pres)!.name, renderer.readingName(PAST)!.name];
    expect(of([pres, PAST])[0]).toBe(`${renderer.join([a, b])} of read`);
    expect(of([PAST, pres])[0]).toBe(`${renderer.join([b, a])} of read`);
  });
});

const verb = (features: Record<string, string>): GrammarTag => ({ pos: "VERB", features });
const fin = (Mood: string, Person: string, Number: string, Tense?: string): GrammarTag =>
  verb({ Mood, Number, Person, ...(Tense ? { Tense } : {}), VerbForm: "Fin" });

describe("the word card in English, of a Spanish word", () => {
  const spanish = (headword: string, surface: string, readings: GrammarTag[], others: WordGrammar["others"] = []) =>
    shown({ language: "es", headword, surface, grammar: grammar({ readings, others }) }).lines;

  it("A form of two persons, named once", () => {
    expect(spanish("hablar", "hablaba", [fin("Ind", "1", "Sing", "Imp"), fin("Ind", "3", "Sing", "Imp")])).toEqual([
      "first- and third-person singular imperfect indicative of hablar",
    ]);
  });

  it("The order of tenses: the subjunctive before the imperative, as the French", () => {
    const readings = [fin("Imp", "3", "Sing"), fin("Sub", "1", "Sing", "Pres"), fin("Sub", "3", "Sing", "Pres")];
    expect(spanish("hablar", "hable", readings)).toEqual([
      "first- and third-person singular present subjunctive and third-person singular imperative of hablar",
    ]);
    expect(fr.grammarLines(grammar({ readings }), "hablar", "hable", "hable", "es").map(lineText)).toEqual([
      "1re et 3e personnes du singulier du présent du subjonctif et 3e personne du singulier de l’impératif de hablar",
    ]);
  });

  it("The order of numbers within a tense: the singular first", () => {
    const readings = [fin("Ind", "1", "Plur", "Pres"), fin("Ind", "3", "Sing", "Pres")];
    expect(spanish("hablar", "x", readings)).toEqual([
      "third-person singular present indicative and first-person plural present indicative of hablar",
    ]);
    // The French puts the plural first, as it always did.
    expect(fr.grammarLines(grammar({ readings }), "hablar", "x", "x", "es").map(lineText)).toEqual([
      "1re personne du pluriel du présent de l’indicatif et 3e personne du singulier du présent de l’indicatif de hablar",
    ]);
  });

  it("An adjective's agreement", () => {
    expect(spanish("rápido", "rápidas", [{ pos: "ADJ", features: { Gender: "Fem", Number: "Plur" } }])).toEqual([
      "feminine plural of r\u00e1pido",
    ]);
    expect(spanish("grande", "grandes", [{ pos: "ADJ", features: { Number: "Plur" } }])).toEqual(["plural of grande"]);
  });

  it("Two genders of one number, named once (add-lingua-english-card-wording D3: « gran » of « grande »)", () => {
    const readings: GrammarTag[] = [
      { pos: "ADJ", features: { Number: "Sing" } },
      { pos: "NOUN", features: { Gender: "Fem", Number: "Sing" } },
      { pos: "NOUN", features: { Gender: "Masc", Number: "Sing" } },
    ];
    expect(spanish("grande", "gran", readings)).toEqual(["masculine and feminine singular of grande"]);
    // The French names the same genders and number, in its words, as before.
    expect(fr.grammarLines(grammar({ readings }), "grande", "gran", "gran", "es").map(lineText)).toEqual([
      "féminin singulier et masculin singulier de grande",
    ]);
    // Each number on its own, where the first of its genders stood.
    const agreed = (Gender: string, Number: string): GrammarTag => ({ pos: "ADJ", features: { Gender, Number } });
    expect(spanish("x", "y", [agreed("Fem", "Plur"), agreed("Masc", "Sing"), agreed("Masc", "Plur")])).toEqual([
      "masculine and feminine plural and masculine singular of x",
    ]);
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
    ).toEqual(["may also be the third-person singular preterite indicative of venir"]);
  });

  it("An English-native reader of Spanish: « vino » of « venir », « venir » marked as Spanish", () => {
    const { lines, card } = shown({
      language: "es",
      headword: "venir",
      surface: "vino",
      grammar: grammar({ readings: [fin("Ind", "3", "Sing", "Past")] }),
    });
    expect(lines).toEqual(["third-person singular preterite indicative of venir"]);
    expect(card.el.querySelector(".grammar-line em")!.getAttribute("lang")).toBe("es");
    expect(card.el.querySelector(".headword")!.getAttribute("lang")).toBe("es");
  });

  it("A pronoun's form and a gerund: the language's own grammar, no elision", () => {
    expect(spanish("él", "ella", [{ pos: "PRON", features: { Gender: "Fem", Number: "Sing" } }])).toEqual([
      "feminine singular of \u00e9l",
    ]);
    expect(spanish("hablar", "hablando", [verb({ VerbForm: "Ger" })])).toEqual(["gerund of hablar"]);
  });

  it("names every mood and tense, and a participle's agreement", () => {
    const name = (tag: GrammarTag) => renderer.readingName(tag, "es");
    expect(name(fin("Ind", "1", "Plur", "Pres"))?.name).toBe("first-person plural present indicative");
    expect(name(fin("Ind", "3", "Sing", "Past"))?.name).toBe("third-person singular preterite indicative");
    expect(name(fin("Ind", "2", "Sing", "Fut"))?.name).toBe("second-person singular future indicative");
    expect(name(fin("Cnd", "1", "Sing"))?.name).toBe("first-person singular conditional");
    expect(name(fin("Sub", "3", "Plur", "Imp"))?.name).toBe("third-person plural imperfect subjunctive");
    expect(name(fin("Sub", "1", "Sing", "Fut"))?.name).toBe("first-person singular future subjunctive");
    expect(name(verb({ VerbForm: "Inf" }))).toEqual({ article: "the", name: "infinitive" });
    const part = (features: Record<string, string>) =>
      name(verb({ Tense: "Past", VerbForm: "Part", ...features }))?.name;
    expect(part({ Gender: "Masc", Number: "Sing" })).toBe("past participle");
    expect(part({ Gender: "Fem", Number: "Sing" })).toBe("feminine past participle");
    expect(part({ Gender: "Fem", Number: "Plur" })).toBe("feminine plural past participle");
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
