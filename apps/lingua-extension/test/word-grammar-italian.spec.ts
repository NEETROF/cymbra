import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { GrammarTag, WordGrammar } from "@/analyzer/types.ts";
import { grammar as en } from "@/i18n/en/grammar.ts";
import { grammar as es } from "@/i18n/es/grammar.ts";
import { grammar as fr } from "@/i18n/fr/grammar.ts";
import type { GrammarRenderer } from "@/i18n/index.ts";
import { lineText } from "@/reading/grammar-description.ts";

// add-lingua-italian-word-card: the card of an Italian word, in each interface — the French
// renderer in French school terms for Italian grammar, the English one in the English Wiktionary's
// Italian form-of wording, the Spanish one in the RAE's terms (D2) —, the indicative and the
// subjunctive of one tense said once where their persons are the same (D3), the gerund and the
// present participle named (D4). No card is opened on an Italian word before change 20 widens the
// extension's `StudiedLanguage`: the renderers are called with `it` studied, as the it-fr snapshot
// calls the French one (`word-card-it-fr.spec.ts`). The readings are change 13's, in the order
// `tables/it/grammar.tsv` writes them.

const grammar = (over: Partial<WordGrammar> = {}): WordGrammar => ({
  gloss: null,
  senses: [],
  readings: [],
  others: [],
  pieces: [],
  ...over,
});
const verb = (features: Record<string, string>): GrammarTag => ({ pos: "VERB", features });
const fin = (Mood: string, Person: string, Number: string, Tense?: string): GrammarTag =>
  verb({ Mood, Number, Person, ...(Tense ? { Tense } : {}), VerbForm: "Fin" });
const past = (Gender: string, Number: string): GrammarTag => verb({ Gender, Number, Tense: "Past", VerbForm: "Part" });
const noun = (Gender: string, Number: string): GrammarTag => ({ pos: "NOUN", features: { Gender, Number } });

/** Change 13's readings of the card's probes, as `tables/it/grammar.tsv` lists them. */
const PARLÒ = [fin("Ind", "3", "Sing", "Past")];
const PARLIAMO = [fin("Imp", "1", "Plur"), fin("Ind", "1", "Plur", "Pres"), fin("Sub", "1", "Plur", "Pres")];
const PARLI = [
  fin("Ind", "2", "Sing", "Pres"),
  fin("Sub", "1", "Sing", "Pres"),
  fin("Sub", "2", "Sing", "Pres"),
  fin("Sub", "3", "Sing", "Pres"),
];
const MANGIATE = [
  past("Fem", "Plur"),
  fin("Imp", "2", "Plur"),
  fin("Ind", "2", "Plur", "Pres"),
  fin("Sub", "2", "Plur", "Pres"),
];
const PARLANDO = [verb({ VerbForm: "Ger" })];
const CREDUTA = [past("Fem", "Sing")];
const PARLANTI = [verb({ Number: "Plur", Tense: "Pres", VerbForm: "Part" })];

/** A card's lines for an Italian form, as plain text. */
const lines = (
  renderer: GrammarRenderer,
  headword: string,
  surface: string,
  readings: GrammarTag[],
  others: WordGrammar["others"] = [],
) => renderer.grammarLines(grammar({ readings, others }), headword, surface, surface, "it").map(lineText);

describe("An Italian card names its forms in the interface language's grammar", () => {
  it("The passato remoto, in each interface", () => {
    expect(lines(fr, "parlare", "parlò", PARLÒ)).toEqual(["3e personne du singulier du passé simple de parlare"]);
    expect(lines(en, "parlare", "parlò", PARLÒ)).toEqual([
      "third-person singular past historic (passato remoto) of parlare",
    ]);
    expect(lines(es, "parlare", "parlò", PARLÒ)).toEqual([
      "tercera persona del singular del pretérito perfecto simple de indicativo de parlare",
    ]);
  });

  it("The indicative and the subjunctive said once (« parliamo »), the imperative after them", () => {
    expect(lines(fr, "parlare", "parliamo", PARLIAMO)).toEqual([
      "1re personne du pluriel du présent de l’indicatif ou du subjonctif et 1re personne du pluriel de l’impératif de parlare",
    ]);
    expect(lines(en, "parlare", "parliamo", PARLIAMO)).toEqual([
      "first-person plural present indicative or subjunctive and first-person plural imperative of parlare",
    ]);
    expect(lines(es, "parlare", "parliamo", PARLIAMO)).toEqual([
      "primera persona del plural del presente de indicativo o de subjuntivo y primera persona del plural del imperativo de parlare",
    ]);
  });

  it("A present of four readings (« parli »): the moods named apart, their persons differing", () => {
    expect(lines(en, "parlare", "parli", PARLI)).toEqual([
      "second-person singular present indicative and first-, second- and third-person singular present subjunctive of parlare",
    ]);
    expect(lines(fr, "parlare", "parli", PARLI)).toEqual([
      "2e personne du singulier du présent de l’indicatif et 1re, 2e et 3e personnes du singulier du présent du subjonctif de parlare",
    ]);
    expect(lines(es, "parlare", "parli", PARLI)).toEqual([
      "segunda persona del singular del presente de indicativo y primera, segunda y tercera persona del singular del presente de subjuntivo de parlare",
    ]);
  });

  it("« mangiate »: the second person plural said once, beside its participle and its imperative", () => {
    const others = [{ lemma: "mangiata", readings: [noun("Fem", "Plur")] }];
    expect(lines(fr, "mangiare", "mangiate", MANGIATE, others)).toEqual([
      "participe passé féminin pluriel, 2e personne du pluriel du présent de l’indicatif ou du subjonctif et 2e personne du pluriel de l’impératif de mangiare",
      "peut aussi être le féminin pluriel de mangiata",
    ]);
    expect(lines(en, "mangiare", "mangiate", MANGIATE, others)).toEqual([
      "feminine plural past participle, second-person plural present indicative or subjunctive and second-person plural imperative of mangiare",
      "may also be the feminine plural of mangiata",
    ]);
    expect(lines(es, "mangiare", "mangiate", MANGIATE, others)).toEqual([
      "participio pasado femenino plural, segunda persona del plural del presente de indicativo o de subjuntivo y segunda persona del plural del imperativo de mangiare",
      "también puede ser el femenino plural de mangiata",
    ]);
  });

  it("The gerund and the participles", () => {
    expect(lines(es, "parlare", "parlando", PARLANDO)).toEqual(["gerundio de parlare"]);
    expect(lines(es, "credere", "creduta", CREDUTA)).toEqual(["participio pasado femenino de credere"]);
    expect(lines(es, "parlare", "parlanti", PARLANTI)).toEqual(["participio presente de parlare"]);
    expect(lines(fr, "parlare", "parlando", PARLANDO)).toEqual(["gérondif de parlare"]);
    expect(lines(fr, "credere", "creduta", CREDUTA)).toEqual(["participe passé féminin de credere"]);
    expect(lines(fr, "parlare", "parlanti", PARLANTI)).toEqual(["participe présent de parlare"]);
    expect(lines(en, "parlare", "parlando", PARLANDO)).toEqual(["gerund of parlare"]);
    expect(lines(en, "credere", "creduta", CREDUTA)).toEqual(["feminine past participle of credere"]);
    expect(lines(en, "parlare", "parlanti", PARLANTI)).toEqual(["present participle of parlare"]);
    // The masculine singular participle is the participle itself.
    for (const [renderer, said] of [
      [fr, "participe passé de parlare"],
      [en, "past participle of parlare"],
      [es, "participio pasado de parlare"],
    ] as const) {
      expect(lines(renderer, "parlare", "parlato", [past("Masc", "Sing")])).toEqual([said]);
    }
    // A present participle with an entry of its own (`parlante`) names the verb on its « may also
    // be » line, its own readings the dictionary form's.
    const parlante = lines(
      fr,
      "parlante",
      "parlante",
      [{ pos: "ADJ", features: { Number: "Sing" } }, noun("Masc", "Sing")],
      [{ lemma: "parlare", readings: [verb({ Number: "Sing", Tense: "Pres", VerbForm: "Part" })] }],
    );
    expect(parlante).toEqual(["peut aussi être le participe présent de parlare"]);
  });

  it("The infinitive named where it is not the dictionary form (`pentire`, *pentirsi*'s)", () => {
    const inf = [verb({ VerbForm: "Inf" })];
    expect(lines(fr, "pentirsi", "pentire", inf)).toEqual(["infinitif de pentirsi"]);
    expect(lines(en, "pentirsi", "pentire", inf)).toEqual(["infinitive of pentirsi"]);
    expect(lines(es, "pentirsi", "pentire", inf)).toEqual(["infinitivo de pentirsi"]);
    expect(lines(fr, "parlare", "parlare", inf)).toEqual([]);
  });

  it("names every tense, in the order indicative (present, imperfect, past historic, future), conditional, subjunctive, imperative", () => {
    // In an order no table has, one person each, so the line lists every tense once.
    const all = [
      fin("Imp", "2", "Sing"),
      fin("Sub", "3", "Sing", "Imp"),
      fin("Sub", "3", "Sing", "Pres"),
      fin("Cnd", "3", "Sing"),
      fin("Ind", "3", "Sing", "Fut"),
      fin("Ind", "3", "Sing", "Past"),
      fin("Ind", "3", "Sing", "Imp"),
      fin("Ind", "1", "Sing", "Pres"),
    ];
    expect(lines(fr, "x", "y", all)).toEqual([
      "1re personne du singulier du présent de l’indicatif, 3e personne du singulier de l’imparfait de l’indicatif, 3e personne du singulier du passé simple, 3e personne du singulier du futur, 3e personne du singulier du conditionnel, 3e personne du singulier du présent du subjonctif, 3e personne du singulier de l’imparfait du subjonctif et 2e personne du singulier de l’impératif de x",
    ]);
    expect(lines(en, "x", "y", all)).toEqual([
      "first-person singular present indicative, third-person singular imperfect indicative, third-person singular past historic (passato remoto), third-person singular future, third-person singular conditional, third-person singular present subjunctive, third-person singular imperfect subjunctive and second-person singular imperative of x",
    ]);
    expect(lines(es, "x", "y", all)).toEqual([
      "primera persona del singular del presente de indicativo, tercera persona del singular del pretérito imperfecto de indicativo, tercera persona del singular del pretérito perfecto simple de indicativo, tercera persona del singular del futuro simple de indicativo, tercera persona del singular del condicional simple, tercera persona del singular del presente de subjuntivo, tercera persona del singular del pretérito imperfecto de subjuntivo y segunda persona del singular del imperativo de x",
    ]);
  });

  it("A plural spelled like its dictionary form, a numeral, and a plural determiner without a gender stay unnamed", () => {
    for (const renderer of [fr, en, es]) {
      expect(lines(renderer, "città", "città", [noun("Fem", "Plur"), noun("Fem", "Sing")])).toEqual([]);
      expect(lines(renderer, "mille", "mila", [{ pos: "NUM", features: { Gender: "Fem", Number: "Plur" } }])).toEqual(
        [],
      );
      expect(lines(renderer, "il", "i", [{ pos: "DET", features: { Number: "Plur" } }])).toEqual([]);
    }
    expect(lines(fr, "casa", "case", [noun("Fem", "Plur")])).toEqual(["féminin pluriel de casa"]);
  });
});

// The tag pool change 13 commits (`scripts/lingua-data/tables/it/tags.tsv`): every tag an Italian
// form can carry. Each is named in the three interfaces, or left unnamed by all three on purpose,
// and no name shows a tag.
describe("Every tag Italian can carry", () => {
  const TAGS = join(dirname(fileURLToPath(import.meta.url)), "../../../scripts/lingua-data/tables/it/tags.tsv");
  const pool: GrammarTag[] = readFileSync(TAGS, "utf8")
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => {
      const [pos, ...features] = line.split("|");
      return features.length === 0
        ? { pos: pos! }
        : { pos: pos!, features: Object.fromEntries(features.map((f) => f.split("=") as [string, string])) };
    });
  const RAW =
    /[A-Za-z]+=|\b(Ind|Sub|Cnd|Imp|Pres|Past|Fut|Fin|Part|Ger|Inf|Sing|Plur|Masc|Fem|VERB|NOUN|ADJ|DET|PRON|NUM)\b/;
  const key = (tag: GrammarTag) =>
    [tag.pos, ...Object.entries(tag.features ?? {}).map(([k, v]) => `${k}=${v}`)].join("|");

  it("is the 83 tags of the committed pool", () => {
    expect(pool).toHaveLength(83);
  });

  it("is named in every interface, or by none — a numeral's, a plural determiner's without a gender, a singular without a gender", () => {
    const unnamed: string[] = [];
    for (const tag of pool) {
      const said = [fr, en, es].map((renderer) => renderer.readingName(tag, "it"));
      expect(new Set(said.map((n) => n === null)).size, key(tag)).toBe(1);
      if (said[0] === null) unnamed.push(key(tag));
      for (const named of said) if (named) expect(named.name, key(tag)).not.toMatch(RAW);
    }
    expect(unnamed).toEqual([
      "ADJ|Number=Sing",
      "DET|Number=Plur",
      "NOUN|Number=Sing",
      "NUM|Gender=Fem|Number=Plur",
      "NUM|Gender=Fem|Number=Sing",
      "NUM|Gender=Masc|Number=Plur",
      "NUM|Number=Plur",
    ]);
  });

  it("never shows a tag on a card's line, in any interface", () => {
    for (const tag of pool) {
      for (const renderer of [fr, en, es]) {
        for (const line of lines(renderer, "lemma", "form", [tag], [{ lemma: "altro", readings: [tag] }])) {
          expect(line, key(tag)).not.toMatch(RAW);
        }
      }
    }
  });
});
