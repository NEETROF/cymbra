import type { GrammarTag, WordGrammar } from "../../analyzer/types.ts";
import {
  type Agreement,
  composeLines,
  describeForm,
  type FormKind,
  formKind,
  type LineWords,
  lineText,
  nameKind,
  nameReadings,
  type ReadingWords,
  readingOf,
  type Reading,
  senseOf,
  tagsOf,
} from "../../reading/grammar-description.ts";
import type { GrammarLine, GrammarRenderer, Named, StudiedLanguageCode } from "../index.ts";

// The word card's grammar in French — the source renderer (generalise-lingua-card-wording D2): the
// card's wording before the catalogue, byte for byte (add-lingua-word-grammar,
// add-lingua-spanish-word-card). The pack speaks Universal Dependencies codes; the reader never sees
// one. The names of parts of speech, gender, number and person are French; a tense's name depends
// on the studied language — `Tense=Past` is the prétérit in English and the passé simple in
// Spanish — so the verb-form names are keyed by studied language, as French schools name them.

const PARTS_OF_SPEECH: Record<string, string> = {
  ADJ: "adjectif",
  ADP: "préposition",
  ADV: "adverbe",
  AUX: "auxiliaire",
  CCONJ: "conjonction",
  DET: "déterminant",
  INTJ: "interjection",
  NOUN: "nom",
  NUM: "nombre",
  PART: "particule",
  PRON: "pronom",
  PROPN: "nom propre",
  SCONJ: "conjonction",
  VERB: "verbe",
  // PUNCT, SYM and X have no name for a reader: their senses show without a heading.
};

const GENDERS: Record<string, string> = { Masc: "masculin", Fem: "féminin", Neut: "neutre", Com: "commun" };
const PERSONS: Record<string, string> = { "1": "1re personne", "2": "2e personne", "3": "3e personne" };
const ORDINALS: Record<string, string> = { "1": "1re", "2": "2e", "3": "3e" };
const NUMBERS: Record<string, string> = { Sing: "du singulier", Plur: "du pluriel" };

/**
 * The studied languages' moods and tenses, as French schools name them, keyed `Mood/Tense`
 * (`finiteKey`). English: the indicative's two. Spanish: the present and the imperfect name their
 * mood — the subjunctive has both too; the conditional and the imperative have no tense.
 */
const TENSES: Record<StudiedLanguageCode, Record<string, string>> = {
  en: { "Ind/Past": "prétérit", "Ind/Pres": "présent" },
  es: {
    "Ind/Pres": "présent de l’indicatif",
    "Ind/Imp": "imparfait de l’indicatif",
    "Ind/Past": "passé simple",
    "Ind/Fut": "futur",
    "Cnd/": "conditionnel",
    "Sub/Pres": "présent du subjonctif",
    "Sub/Imp": "imparfait du subjonctif",
    "Sub/Fut": "futur du subjonctif",
    "Imp/": "impératif",
  },
};

/**
 * The tenses in the order French grammars give them — indicative, conditional, subjunctive,
 * imperative; English's two come in the pack's order.
 */
const TENSE_ORDER: Record<StudiedLanguageCode, readonly string[]> = { en: [], es: Object.keys(TENSES.es) };

/** The gerund's name: English's « forme en -ing », Spanish's « gérondif ». */
const GERUNDS: Record<StudiedLanguageCode, Named> = {
  en: { article: "la", name: "forme en -ing" },
  es: { article: "le", name: "gérondif" },
};

/** Whether the infinitive is named: for Spanish only — an English card never named it. */
const NAMES_INFINITIVE: Record<StudiedLanguageCode, boolean> = { en: false, es: true };

/** French elides before a vowel, accented or not — never before « h » or « y ». */
function elides(word: string): boolean {
  return /^[aeiouàâäéèêëîïôöùûüáíóú]/i.test(word);
}

/** « du passé simple », « de l’imparfait de l’indicatif »: « de » before a masculine name. */
function ofName(name: string): string {
  return elides(name) ? `de l’${name}` : `du ${name}`;
}

/** A masculine name with its article: « le futur », « l’infinitif ». */
function masculine(name: string): Named {
  return { article: elides(name) ? "l’" : "le", name };
}

/** « féminin pluriel », « masculin singulier », « pluriel »: a nominal form's agreement. */
function agreement({ gender, number }: Agreement): string | undefined {
  const named = number === "Plur" ? "pluriel" : number === "Sing" ? "singulier" : undefined;
  if (gender) return named ? `${GENDERS[gender]} ${named}` : GENDERS[gender];
  return number === "Plur" ? "pluriel" : undefined;
}

/** "a", "a et b", "a, b et c". */
function join(items: readonly string[]): string {
  if (items.length <= 1) return items.join("");
  return `${items.slice(0, -1).join(", ")} et ${items.at(-1)}`;
}

const readingWords: ReadingWords = {
  tense: (key, studied) => TENSES[studied][key],
  name(kind: FormKind, studied, tense) {
    switch (kind.kind) {
      case "degree":
        return { article: "le", name: kind.degree === "Cmp" ? "comparatif" : "superlatif" };
      case "agreement": {
        const name = agreement(kind);
        return name ? { article: "le", name } : null;
      }
      case "infinitive":
        return NAMES_INFINITIVE[studied] ? masculine("infinitif") : null;
      case "participle": {
        const agreed = kind.gender === "Masc" && kind.number !== "Plur" ? undefined : agreement(kind);
        return {
          article: "le",
          name: agreed ? `participe passé ${agreed.replace(" singulier", "")}` : "participe passé",
        };
      }
      case "gerund":
        return GERUNDS[studied];
      case "finite":
        return tense === undefined ? null : masculine(tense);
    }
  },
  persons(persons, number, tense) {
    const who = persons.length === 1 ? PERSONS[persons[0]!] : `${join(persons.map((p) => ORDINALS[p]!))} personnes`;
    return { article: persons.length === 1 ? "la" : "les", name: `${who} ${NUMBERS[number]} ${ofName(tense)}` };
  },
  order: (studied) => TENSE_ORDER[studied],
  numbers: ["Plur", "Sing"],
  // Persons merge on the French tense's name, as the card always merged them.
  mergeBy: "name",
};

/**
 * The names of a list of readings, each once, in the pack's order. The French card names a form
 * once per French name: the description's readings, merged by tag, are taken back to their tags
 * and merged again by name — « 1re et 3e personnes du singulier de l’imparfait de l’indicatif ».
 */
function names(readings: readonly Reading[], studied: StudiedLanguageCode): Named[] {
  return nameReadings(readings.flatMap(tagsOf).map(readingOf), studied, readingWords);
}

function withArticles(named: readonly Named[]): string {
  return join(named.map((n) => (n.article === "l’" ? `l’${n.name}` : `${n.article} ${n.name}`)));
}

const lineWords: LineWords = {
  names,
  formOf: (named) => `${join(named.map((n) => n.name))} `,
  mayAlsoBe: (named) => `peut aussi être ${withArticles(named)} `,
  // "de go", "d’eat", "d’él": French elides before a vowel, accented or not.
  of: (word): GrammarLine => [elides(word) ? "d’" : "de ", { word }],
  pieces: (written) => `« ${written} » = `,
  piecesSeparator: " + ",
};

export const grammar: GrammarRenderer = {
  grammarLines(grammar: WordGrammar, headword, surface, written, studied: StudiedLanguageCode = "en") {
    return composeLines(describeForm(grammar, headword, surface, written), studied, lineWords);
  },
  readingName(tag: GrammarTag, studied: StudiedLanguageCode = "en") {
    const kind = formKind(readingOf(tag), studied);
    return kind ? nameKind(kind, studied, readingWords) : null;
  },
  senseHeading(tag) {
    const { pos, gender } = senseOf(tag);
    const name = pos ? PARTS_OF_SPEECH[pos] : undefined;
    if (!name) return null;
    const named = gender ? GENDERS[gender] : undefined;
    return named ? `${name} ${named}` : name;
  },
  lineText,
  join,
};
