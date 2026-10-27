import { describe, expect, it } from "vitest";
import type { GrammarTag, WordGrammar } from "@/analyzer/types.ts";
import type { StudiedLanguageCode } from "@/i18n/index.ts";
import { grammar as fr } from "@/i18n/fr/grammar.ts";
import { grammar as renderer } from "@/i18n/it/grammar.ts";
import { lineText } from "@/reading/grammar-description.ts";
import { createCard, type WordPopupContent } from "@/reading/wordpopup.ts";

// add-lingua-italian-card-wording D7: the Italian renderer over made-up readings, as the English and
// Spanish renderers' unit specs (test/word-grammar-en.spec.ts, test/word-grammar-es.spec.ts) hold
// theirs — every name of D1–D3, the future subjunctive and the gerunds the goldens' probes lack
// included; the persons merged; the agreements; the contractions and « può anche essere »; the sense
// headings; what stays unnamed. Shown since the Italian interface is (enable-lingua-italian-speakers).
// The real forms are pinned by test/word-card-grammar-it.spec.ts.

const verb = (features: Record<string, string>): GrammarTag => ({ pos: "VERB", features });
const fin = (Mood: string, Person: string, Number: string, Tense?: string): GrammarTag =>
  verb({ Mood, Number, Person, ...(Tense ? { Tense } : {}), VerbForm: "Fin" });
const noun = (Gender: string, Number: string): GrammarTag => ({ pos: "NOUN", features: { Gender, Number } });

const PAST: GrammarTag = verb({ Mood: "Ind", Tense: "Past", VerbForm: "Fin" });
const PARTICIPLE: GrammarTag = verb({ Tense: "Past", VerbForm: "Part" });
const THIRD: GrammarTag = fin("Ind", "3", "Sing", "Pres");
const PLURAL: GrammarTag = { pos: "NOUN", features: { Number: "Plur" } };
const GERUND: GrammarTag = verb({ VerbForm: "Ger" });
const INFINITIVE: GrammarTag = verb({ VerbForm: "Inf" });
const PRESENT_PARTICIPLE: GrammarTag = verb({ Tense: "Pres", VerbForm: "Part" });

const grammar = (over: Partial<WordGrammar> = {}): WordGrammar => ({
  gloss: null,
  senses: [],
  readings: [],
  others: [],
  pieces: [],
  ...over,
});

/** The card in the Italian interface. */
function shown(over: Partial<WordPopupContent>) {
  const card = createCard(undefined, "it");
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

/** The grammar lines of a form, `studied` studied. */
const lines = (
  studied: StudiedLanguageCode,
  headword: string,
  surface: string,
  readings: GrammarTag[],
  others: WordGrammar["others"] = [],
) => renderer.grammarLines(grammar({ readings, others }), headword, surface, surface, studied).map(lineText);

const name = (tag: GrammarTag, studied: StudiedLanguageCode) => renderer.readingName(tag, studied)?.name;

describe("the word card in Italian, of an English word", () => {
  it("An irregular form, its dictionary form marked as English: « passato semplice di go »", () => {
    const { lines: said, card } = shown({ grammar: grammar({ readings: [PAST] }) });
    expect(said).toEqual(["passato semplice di go"]);
    const word = card.el.querySelector(".grammar-line em")!;
    expect(word.textContent).toBe("go");
    expect(word.getAttribute("lang")).toBe("en");
  });

  it("The -ing form: « forma in -ing di run »", () => {
    expect(shown({ headword: "run", surface: "running", grammar: grammar({ readings: [GERUND] }) }).lines).toEqual([
      "forma in -ing di run",
    ]);
  });

  it("A form with two readings, in one statement", () => {
    expect(lines("en", "walk", "walked", [PAST, PARTICIPLE])).toEqual([
      "passato semplice e participio passato di walk",
    ]);
  });

  it("A form spelled like its dictionary form, as a possibility, each name with its article", () => {
    expect(lines("en", "put", "put", [PAST, PARTICIPLE])).toEqual([
      "può anche essere il passato semplice e il participio passato di put",
    ]);
    expect(lines("en", "running", "running", [GERUND])).toEqual(["può anche essere la forma in -ing di running"]);
  });

  it("A form of two dictionary forms: « del presente semplice », « il plurale »", () => {
    expect(lines("en", "leave", "leaves", [THIRD], [{ lemma: "leaf", readings: [PLURAL] }])).toEqual([
      "terza persona singolare del presente semplice di leave",
      "può anche essere il plurale di leaf",
    ]);
  });

  it("A contraction, its pieces in guillemets with no inner space", () => {
    expect(
      shown({ headword: "do", surface: "do", written: "don't", grammar: grammar({ pieces: ["do", "not"] }) }).lines,
    ).toEqual(["«don't» = do + not"]);
    expect(
      shown({
        headword: "do",
        surface: "does",
        written: "doesn't",
        grammar: grammar({ pieces: ["does", "not"], readings: [THIRD] }),
      }).lines,
    ).toEqual(["«doesn't» = does + not", "terza persona singolare del presente semplice di do"]);
  });

  it("names English verb forms, plurals and degrees", () => {
    expect(name(THIRD, "en")).toBe("terza persona singolare del presente semplice");
    expect(name(fin("Ind", "1", "Sing", "Past"), "en")).toBe("prima persona singolare del passato semplice");
    expect(name(verb({ Mood: "Ind", Tense: "Pres", VerbForm: "Fin" }), "en")).toBe("presente semplice");
    expect(renderer.readingName(PARTICIPLE)).toEqual({ article: "il", name: "participio passato" });
    expect(renderer.readingName(GERUND)).toEqual({ article: "la", name: "forma in -ing" });
    expect(name(PLURAL, "en")).toBe("plurale");
    expect(renderer.readingName({ pos: "ADJ", features: { Degree: "Cmp" } })).toEqual({
      article: "il",
      name: "comparativo",
    });
    expect(name({ pos: "ADV", features: { Degree: "Sup" } }, "en")).toBe("superlativo");
    // As every renderer: English's infinitive, a pronoun's case and an English subjunctive are unnamed.
    expect(renderer.readingName(INFINITIVE)).toBeNull();
    expect(renderer.readingName({ pos: "PRON", features: { Case: "Dat" } })).toBeNull();
    expect(renderer.readingName(verb({ Mood: "Sub", Tense: "Imp", VerbForm: "Fin" }))).toBeNull();
  });

  it("The order of tenses: English's come in the pack's order", () => {
    const pres = verb({ Mood: "Ind", Tense: "Pres", VerbForm: "Fin" });
    expect(lines("en", "read", "read2", [pres, PAST])).toEqual(["presente semplice e passato semplice di read"]);
    expect(lines("en", "read", "read2", [PAST, pres])).toEqual(["passato semplice e presente semplice di read"]);
  });

  it("No code reaches the reader", () => {
    const odd = verb({ Mood: "Sub", Tense: "Imp", VerbForm: "Fin" });
    const { lines: said, card } = shown({
      grammar: grammar({ readings: [PAST, odd, { pos: "WORD" }], others: [{ lemma: "gone", readings: [odd] }] }),
    });
    expect(said).toEqual(["passato semplice di go"]);
    expect(card.el.textContent).not.toMatch(/VERB|Tense=|Mood|VerbForm|gone/);
  });
});

describe("the word card in Italian, of a Spanish word", () => {
  it("The pretérito perfecto simple is the « passato remoto »: « habló »", () => {
    const { lines: said, card } = shown({
      language: "es",
      headword: "hablar",
      surface: "habló",
      grammar: grammar({ readings: [fin("Ind", "3", "Sing", "Past")] }),
    });
    expect(said).toEqual(["terza persona singolare del passato remoto di hablar"]);
    expect(card.el.querySelector(".grammar-line em")!.getAttribute("lang")).toBe("es");
  });

  it("Two persons named once: « hablaba »", () => {
    expect(lines("es", "hablar", "hablaba", [fin("Ind", "1", "Sing", "Imp"), fin("Ind", "3", "Sing", "Imp")])).toEqual([
      "prima e terza persona singolare dell'indicativo imperfetto di hablar",
    ]);
    expect(
      lines("es", "hablar", "x", [
        fin("Sub", "1", "Sing", "Pres"),
        fin("Sub", "2", "Sing", "Pres"),
        fin("Sub", "3", "Sing", "Pres"),
      ]),
    ).toEqual(["prima, seconda e terza persona singolare del congiuntivo presente di hablar"]);
  });

  it("names every Spanish mood and tense by its Italian name, the future subjunctive bracketed", () => {
    expect(name(fin("Ind", "1", "Plur", "Pres"), "es")).toBe("prima persona plurale dell'indicativo presente");
    expect(name(fin("Ind", "1", "Sing", "Imp"), "es")).toBe("prima persona singolare dell'indicativo imperfetto");
    expect(name(fin("Ind", "3", "Sing", "Past"), "es")).toBe("terza persona singolare del passato remoto");
    expect(name(fin("Ind", "2", "Sing", "Fut"), "es")).toBe("seconda persona singolare del futuro semplice");
    expect(name(fin("Cnd", "1", "Sing"), "es")).toBe("prima persona singolare del condizionale presente");
    expect(name(fin("Sub", "3", "Plur", "Pres"), "es")).toBe("terza persona plurale del congiuntivo presente");
    expect(name(fin("Sub", "3", "Plur", "Imp"), "es")).toBe("terza persona plurale del congiuntivo imperfetto");
    expect(name(fin("Sub", "1", "Sing", "Fut"), "es")).toBe(
      "prima persona singolare del congiuntivo futuro (futuro de subjuntivo)",
    );
    expect(name(fin("Imp", "2", "Sing"), "es")).toBe("seconda persona singolare dell'imperativo");
    expect(renderer.readingName(verb({ Mood: "Sub", Tense: "Fut", VerbForm: "Fin" }), "es")).toEqual({
      article: "il",
      name: "congiuntivo futuro (futuro de subjuntivo)",
    });
    expect(renderer.readingName(verb({ Mood: "Imp", VerbForm: "Fin" }), "es")).toEqual({
      article: "l'",
      name: "imperativo",
    });
  });

  it("The order of tenses: the subjunctive before the imperative; the singular before the plural", () => {
    expect(
      lines("es", "hablar", "hable", [
        fin("Imp", "3", "Sing"),
        fin("Sub", "1", "Sing", "Pres"),
        fin("Sub", "3", "Sing", "Pres"),
      ]),
    ).toEqual([
      "prima e terza persona singolare del congiuntivo presente e terza persona singolare dell'imperativo di hablar",
    ]);
    // One tense in both numbers is named once (review-lingua-italian-wording, after change 34's review).
    expect(lines("es", "hablar", "x", [fin("Ind", "1", "Plur", "Pres"), fin("Ind", "3", "Sing", "Pres")])).toEqual([
      "terza persona singolare e prima persona plurale dell'indicativo presente di hablar",
    ]);
  });

  it("One tense in both numbers, named once — each half with its article after « può anche essere »", () => {
    const sono = [fin("Ind", "3", "Plur", "Pres"), fin("Ind", "1", "Sing", "Pres")];
    expect(lines("it", "essere", "sono", sono)).toEqual([
      "prima persona singolare e terza persona plurale dell'indicativo presente di essere",
    ]);
    expect(lines("it", "x", "x", [], [{ lemma: "essere", readings: sono }])).toEqual([
      "può anche essere la prima persona singolare e la terza persona plurale dell'indicativo presente di essere",
    ]);
    // Two tenses: each still named, a comma before the name that joins two numbers itself.
    expect(lines("it", "essere", "x", [PARTICIPLE, ...sono])).toEqual([
      "participio passato, prima persona singolare e terza persona plurale dell'indicativo presente di essere",
    ]);
    // Different tenses are not merged: each number keeps its own.
    expect(lines("es", "hablar", "x", [fin("Ind", "3", "Sing", "Pres"), fin("Sub", "1", "Plur", "Pres")])).toEqual([
      "terza persona singolare dell'indicativo presente e prima persona plurale del congiuntivo presente di hablar",
    ]);
  });

  it("An adjective's agreement: « rápidas »", () => {
    expect(
      shown({
        language: "es",
        headword: "rápido",
        surface: "rápidas",
        grammar: grammar({ readings: [{ pos: "ADJ", features: { Gender: "Fem", Number: "Plur" } }] }),
      }).lines,
    ).toEqual(["femminile plurale di rápido"]);
    expect(lines("es", "grande", "grandes", [{ pos: "ADJ", features: { Number: "Plur" } }])).toEqual([
      "plurale di grande",
    ]);
    expect(lines("es", "él", "ella", [{ pos: "PRON", features: { Gender: "Fem", Number: "Sing" } }])).toEqual([
      "femminile singolare di él",
    ]);
  });

  it("The gerund, the participle's agreement and the infinitive", () => {
    expect(lines("es", "hablar", "hablando", [GERUND])).toEqual(["gerundio di hablar"]);
    const part = (features: Record<string, string>) =>
      name(verb({ Tense: "Past", VerbForm: "Part", ...features }), "es");
    expect(part({ Gender: "Masc", Number: "Sing" })).toBe("participio passato");
    expect(part({ Gender: "Fem", Number: "Sing" })).toBe("participio passato femminile");
    expect(part({ Gender: "Fem", Number: "Plur" })).toBe("participio passato femminile plurale");
    expect(part({ Gender: "Masc", Number: "Plur" })).toBe("participio passato maschile plurale");
    expect(renderer.readingName(INFINITIVE, "es")).toEqual({ article: "l'", name: "infinito" });
    expect(lines("es", "hablar", "hablado", [PARTICIPLE, INFINITIVE])).toEqual([
      "participio passato e infinito di hablar",
    ]);
  });

  it("The dictionary form itself: no line, and the other dictionary form named with « la »", () => {
    expect(lines("es", "hablar", "hablar", [INFINITIVE])).toEqual([]);
    expect(lines("es", "casa", "casa", [noun("Fem", "Sing")])).toEqual([]);
    expect(
      lines(
        "es",
        "vino",
        "vino",
        [noun("Masc", "Sing")],
        [{ lemma: "venir", readings: [fin("Ind", "3", "Sing", "Past")] }],
      ),
    ).toEqual(["può anche essere la terza persona singolare del passato remoto di venir"]);
  });

  it("A plural read in both numbers gives no line; a noun used only in the plural keeps its own", () => {
    expect(lines("es", "crisis", "crisis", [noun("Fem", "Plur"), noun("Fem", "Sing")])).toEqual([]);
    expect(
      lines("es", "gafas", "gafas", [noun("Fem", "Plur")], [{ lemma: "gafa", readings: [noun("Fem", "Plur")] }]),
    ).toEqual(["può anche essere il femminile plurale di gafas", "può anche essere il femminile plurale di gafa"]);
  });
});

describe("the word card in Italian, of a French word", () => {
  const PARLE: GrammarTag[] = [
    fin("Imp", "2", "Sing"),
    fin("Ind", "1", "Sing", "Pres"),
    fin("Ind", "3", "Sing", "Pres"),
    fin("Sub", "1", "Sing", "Pres"),
    fin("Sub", "3", "Sing", "Pres"),
  ];

  it("The passé simple is the « passato remoto »: « parla »", () => {
    expect(lines("fr", "parler", "parla", [fin("Ind", "3", "Sing", "Past")])).toEqual([
      "terza persona singolare del passato remoto di parler",
    ]);
  });

  it("A form of five readings, the moods said once, « parler » marked as French", () => {
    const { lines: said, card } = shown({
      language: "fr",
      headword: "parler",
      surface: "parle",
      grammar: grammar({ readings: PARLE }),
    });
    expect(said).toEqual([
      "prima e terza persona singolare dell'indicativo o congiuntivo presente e seconda persona singolare dell'imperativo di parler",
    ]);
    expect(card.el.querySelector(".grammar-line em")!.getAttribute("lang")).toBe("fr");
    expect(
      lines("fr", "finir", "finissions", [
        fin("Ind", "1", "Plur", "Imp"),
        fin("Sub", "1", "Plur", "Imp"),
        fin("Sub", "1", "Plur", "Pres"),
      ]),
    ).toEqual([
      "prima persona plurale dell'indicativo o congiuntivo imperfetto e prima persona plurale del congiuntivo presente di finir",
    ]);
  });

  it("names every tense of French by its Italian name", () => {
    expect(name(fin("Ind", "1", "Sing", "Pres"), "fr")).toBe("prima persona singolare dell'indicativo presente");
    expect(name(fin("Ind", "1", "Sing", "Imp"), "fr")).toBe("prima persona singolare dell'indicativo imperfetto");
    expect(name(fin("Ind", "1", "Sing", "Past"), "fr")).toBe("prima persona singolare del passato remoto");
    expect(name(fin("Ind", "1", "Sing", "Fut"), "fr")).toBe("prima persona singolare del futuro semplice");
    expect(name(fin("Cnd", "1", "Sing"), "fr")).toBe("prima persona singolare del condizionale presente");
    expect(name(fin("Sub", "1", "Sing", "Pres"), "fr")).toBe("prima persona singolare del congiuntivo presente");
    expect(name(fin("Sub", "1", "Sing", "Imp"), "fr")).toBe("prima persona singolare del congiuntivo imperfetto");
    expect(name(fin("Imp", "2", "Sing"), "fr")).toBe("seconda persona singolare dell'imperativo");
    expect(name(fin("Sub", "1", "Sing", "Fut"), "fr")).toBeUndefined(); // French has no future subjunctive
    expect(name(INFINITIVE, "fr")).toBe("infinito");
    expect(name(GERUND, "fr")).toBeUndefined(); // nor a gerund the card names
  });

  it("The participles: « participio presente » beside « participio passato »", () => {
    expect(lines("fr", "parler", "parlant", [PRESENT_PARTICIPLE])).toEqual(["participio presente di parler"]);
    const past = (Gender: string, Number: string) => verb({ Gender, Number, Tense: "Past", VerbForm: "Part" });
    expect(lines("fr", "parler", "parlé", [past("Masc", "Sing")])).toEqual(["participio passato di parler"]);
    expect(lines("fr", "diriger", "dirigée", [past("Fem", "Sing")])).toEqual([
      "participio passato femminile di diriger",
    ]);
  });

  it("Two genders of one number named once: « sommes »", () => {
    expect(
      lines(
        "fr",
        "être",
        "sommes",
        [fin("Ind", "1", "Plur", "Pres")],
        [
          { lemma: "somme", readings: [noun("Fem", "Plur"), noun("Masc", "Plur")] },
          { lemma: "sommer", readings: [fin("Ind", "2", "Sing", "Pres"), fin("Sub", "2", "Sing", "Pres")] },
        ],
      ),
    ).toEqual([
      "prima persona plurale dell'indicativo presente di être",
      "può anche essere il maschile e femminile plurale di somme",
      "può anche essere la seconda persona singolare dell'indicativo o congiuntivo presente di sommer",
    ]);
  });

  it("A numeral and a plural determiner without a gender are unnamed; a comparative is named", () => {
    expect(lines("fr", "million", "millions", [{ pos: "NUM", features: { Number: "Plur" } }])).toEqual([]);
    expect(
      lines("fr", "le", "les", [
        { pos: "DET", features: { Number: "Plur" } },
        { pos: "PRON", features: { Number: "Plur" } },
      ]),
    ).toEqual([]);
    expect(lines("fr", "petit", "moindre", [{ pos: "ADJ", features: { Degree: "Cmp", Number: "Sing" } }])).toEqual([
      "comparativo di petit",
    ]);
  });
});

describe("the word card in Italian, of an Italian word (the row M2 reserves)", () => {
  it("The passato remoto, the imperfetto, the future and the conditional: « parlò », « parlavo »", () => {
    expect(lines("it", "parlare", "parlò", [fin("Ind", "3", "Sing", "Past")])).toEqual([
      "terza persona singolare del passato remoto di parlare",
    ]);
    expect(lines("it", "parlare", "parlavo", [fin("Ind", "1", "Sing", "Imp")])).toEqual([
      "prima persona singolare dell'indicativo imperfetto di parlare",
    ]);
    expect(lines("it", "parlare", "parlerò", [fin("Ind", "1", "Sing", "Fut")])).toEqual([
      "prima persona singolare del futuro semplice di parlare",
    ]);
    expect(lines("it", "parlare", "parlerebbe", [fin("Cnd", "3", "Sing")])).toEqual([
      "terza persona singolare del condizionale presente di parlare",
    ]);
    expect(
      lines("it", "parlare", "parlassi", [fin("Sub", "1", "Sing", "Imp"), fin("Sub", "2", "Sing", "Imp")]),
    ).toEqual(["prima e seconda persona singolare del congiuntivo imperfetto di parlare"]);
  });

  it("The indicative and the subjunctive said once (« parliamo »), apart where their persons differ (« parli »)", () => {
    expect(
      lines("it", "parlare", "parliamo", [
        fin("Imp", "1", "Plur"),
        fin("Ind", "1", "Plur", "Pres"),
        fin("Sub", "1", "Plur", "Pres"),
      ]),
    ).toEqual([
      "prima persona plurale dell'indicativo o congiuntivo presente e prima persona plurale dell'imperativo di parlare",
    ]);
    expect(
      lines("it", "parlare", "parli", [
        fin("Ind", "2", "Sing", "Pres"),
        fin("Sub", "1", "Sing", "Pres"),
        fin("Sub", "2", "Sing", "Pres"),
        fin("Sub", "3", "Sing", "Pres"),
      ]),
    ).toEqual([
      "seconda persona singolare dell'indicativo presente e prima, seconda e terza persona singolare del congiuntivo presente di parlare",
    ]);
  });

  it("The gerund, the participles and the infinitive", () => {
    expect(lines("it", "parlare", "parlando", [GERUND])).toEqual(["gerundio di parlare"]);
    expect(lines("it", "parlare", "parlanti", [verb({ Number: "Plur", Tense: "Pres", VerbForm: "Part" })])).toEqual([
      "participio presente di parlare",
    ]);
    expect(
      lines("it", "credere", "creduta", [verb({ Gender: "Fem", Number: "Sing", Tense: "Past", VerbForm: "Part" })]),
    ).toEqual(["participio passato femminile di credere"]);
    expect(lines("it", "pentirsi", "pentire", [INFINITIVE])).toEqual(["infinito di pentirsi"]);
    expect(
      lines(
        "it",
        "parlante",
        "parlante",
        [noun("Masc", "Sing")],
        [{ lemma: "parlare", readings: [PRESENT_PARTICIPLE] }],
      ),
    ).toEqual(["può anche essere il participio presente di parlare"]);
  });

  it("Treccani's names, the mood before the tense", () => {
    const expected: [GrammarTag, string][] = [
      [verb({ Mood: "Ind", Tense: "Pres", VerbForm: "Fin" }), "indicativo presente"],
      [verb({ Mood: "Ind", Tense: "Imp", VerbForm: "Fin" }), "indicativo imperfetto"],
      [verb({ Mood: "Ind", Tense: "Past", VerbForm: "Fin" }), "passato remoto"],
      [verb({ Mood: "Ind", Tense: "Fut", VerbForm: "Fin" }), "futuro semplice"],
      [verb({ Mood: "Cnd", VerbForm: "Fin" }), "condizionale presente"],
      [verb({ Mood: "Sub", Tense: "Pres", VerbForm: "Fin" }), "congiuntivo presente"],
      [verb({ Mood: "Sub", Tense: "Imp", VerbForm: "Fin" }), "congiuntivo imperfetto"],
      [verb({ Mood: "Imp", VerbForm: "Fin" }), "imperativo"],
      [GERUND, "gerundio"],
      [PRESENT_PARTICIPLE, "participio presente"],
      [PARTICIPLE, "participio passato"],
      [INFINITIVE, "infinito"],
    ];
    for (const [tag, named] of expected) expect(name(tag, "it"), named).toBe(named);
    // Italian has no future subjunctive.
    expect(name(verb({ Mood: "Sub", Tense: "Fut", VerbForm: "Fin" }), "it")).toBeUndefined();
  });
});

describe("the line's grammar in Italian", () => {
  it("joins with « e », never « ed »", () => {
    expect(renderer.join([])).toBe("");
    expect(renderer.join(["a"])).toBe("a");
    expect(renderer.join(["a", "b"])).toBe("a e b");
    expect(renderer.join(["a", "b", "c"])).toBe("a, b e c");
  });

  it("contracts « di » with a tense's article, never before a word of the page", () => {
    // « del » before a consonant, « dell' » before a vowel, the apostrophe U+0027.
    expect(name(fin("Ind", "3", "Sing", "Past"), "es")).toContain("del passato remoto");
    expect(name(fin("Imp", "2", "Sing"), "es")).toContain("dell'imperativo");
    expect(name(fin("Ind", "3", "Sing", "Imp"), "es")).toContain("dell'indicativo imperfetto");
    // « di » before a word of the page, whatever it starts with.
    expect(lines("en", "eat", "ate", [PAST])).toEqual(["passato semplice di eat"]);
    expect(lines("it", "andare", "andò", [fin("Ind", "3", "Sing", "Past")])).toEqual([
      "terza persona singolare del passato remoto di andare",
    ]);
    const all = [
      ...lines("es", "hablar", "x", [fin("Imp", "2", "Sing"), fin("Ind", "1", "Sing", "Imp")]),
      ...lines("en", "eat", "eat", [PAST]),
    ].join(" ");
    expect(all).not.toMatch(/’|d'[a-z]|di l'|di il|de l/);
  });

  it("« può anche essere » takes each name's article: « il », « l' », « la »", () => {
    expect(
      lines("es", "x", "x", [
        PARTICIPLE,
        GERUND,
        fin("Ind", "3", "Sing", "Pres"),
        { pos: "ADJ", features: { Degree: "Sup" } },
      ]),
    ).toEqual([
      "può anche essere il participio passato, il gerundio, la terza persona singolare dell'indicativo presente e il superlativo di x",
    ]);
    // The infinitive of another dictionary form: « l' », with no space.
    expect(lines("es", "x", "x", [], [{ lemma: "hablar", readings: [INFINITIVE] }])).toEqual([
      "può anche essere l'infinito di hablar",
    ]);
  });

  it("names the agreements, the genders of one number once", () => {
    expect(name(noun("Masc", "Sing"), "es")).toBe("maschile singolare");
    expect(name(noun("Fem", "Plur"), "es")).toBe("femminile plurale");
    expect(name(noun("Neut", "Sing"), "es")).toBe("neutro singolare");
    expect(name(noun("Com", "Plur"), "es")).toBe("comune plurale");
    expect(name({ pos: "PRON", features: { Gender: "Fem" } }, "es")).toBe("femminile");
    expect(lines("es", "grande", "grandes", [noun("Masc", "Plur"), noun("Fem", "Plur")])).toEqual([
      "maschile e femminile plurale di grande",
    ]);
  });

  it("A comma, not a second « e », before the genders named once (review-lingua-italian-wording)", () => {
    const grandi = [PLURAL, noun("Masc", "Plur"), noun("Fem", "Plur")];
    expect(lines("it", "grande", "grandi", grandi)).toEqual(["plurale, maschile e femminile plurale di grande"]);
    expect(lines("it", "x", "x", [], [{ lemma: "grande", readings: grandi }])).toEqual([
      "può anche essere il plurale, il maschile e femminile plurale di grande",
    ]);
    // First, the merged name needs no comma after it: « plurale » closes it.
    expect(lines("es", "x", "y", [noun("Masc", "Plur"), noun("Fem", "Plur"), PLURAL])).toEqual([
      "maschile e femminile plurale e plurale di x",
    ]);
  });

  it("heads a group of senses with its part of speech, a noun before its gender", () => {
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
    expect(groups).toEqual(["sostantivo femminile", "verbo", null]);
    const heading = (pos: string, Gender?: string) =>
      renderer.senseHeading(Gender ? { pos, features: { Gender } } : { pos });
    expect(
      ["ADJ", "ADP", "ADV", "AUX", "CCONJ", "DET", "INTJ", "NOUN", "NUM", "PART", "PRON", "PROPN", "SCONJ", "VERB"].map(
        (pos) => heading(pos),
      ),
    ).toEqual([
      "aggettivo",
      "preposizione",
      "avverbio",
      "verbo ausiliare",
      "congiunzione",
      "determinante",
      "interiezione",
      "sostantivo",
      "numerale",
      "particella",
      "pronome",
      "nome proprio",
      "congiunzione",
      "verbo",
    ]);
    expect(heading("NOUN", "Masc")).toBe("sostantivo maschile");
    // « sostantivo comune » is a common noun: the common gender is said as such.
    expect(heading("NOUN", "Com")).toBe("sostantivo di genere comune");
    expect(heading("PRON", "Neut")).toBe("pronome neutro");
    for (const pos of ["PUNCT", "SYM", "X", "WORD"]) expect(heading(pos), pos).toBeNull();
    expect(renderer.senseHeading(undefined)).toBeNull();
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

  it("says what the French card says, no more, in Italian words", () => {
    const readings = [fin("Ind", "1", "Sing", "Imp"), fin("Ind", "3", "Sing", "Imp")];
    const italian = lines("es", "hablar", "hablaba", readings);
    const french = fr.grammarLines(grammar({ readings }), "hablar", "hablaba", "hablaba", "es").map(lineText);
    expect(italian).toHaveLength(french.length);
    expect(italian[0]).not.toBe(french[0]);
  });
});
