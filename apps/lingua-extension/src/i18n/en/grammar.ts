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
  senseOf,
} from "../../reading/grammar-description.ts";
import type { grammar as fr } from "../fr/grammar.ts";
import type { GrammarLine, Named, StudiedLanguageCode } from "../index.ts";

// The word card's grammar in English — a draft after the French renderer
// (generalise-lingua-card-wording D2–D4), reviewed by the owner (M9). It follows the English
// Wiktionary's form-of wording: "third-person singular preterite indicative of venir", "past
// participle of walk". English has no gendered article and elides nothing: a line says "the" once,
// before its list. Readings merge by tag; the tenses keep the order its tables pin.

const PARTS_OF_SPEECH: Record<string, string> = {
  ADJ: "adjective",
  ADP: "preposition",
  ADV: "adverb",
  AUX: "auxiliary verb",
  CCONJ: "conjunction",
  DET: "determiner",
  INTJ: "interjection",
  NOUN: "noun",
  NUM: "numeral",
  PART: "particle",
  PRON: "pronoun",
  PROPN: "proper noun",
  SCONJ: "conjunction",
  VERB: "verb",
};

const GENDERS: Record<string, string> = { Masc: "masculine", Fem: "feminine", Neut: "neuter", Com: "common" };
const ORDINALS: Record<string, string> = { "1": "first", "2": "second", "3": "third" };
const NUMBERS: Record<string, string> = { Sing: "singular", Plur: "plural" };

/** The studied languages' moods and tenses, as the English Wiktionary names them, keyed `Mood/Tense`. */
const TENSES: Record<StudiedLanguageCode, Record<string, string>> = {
  en: { "Ind/Past": "simple past", "Ind/Pres": "simple present" },
  es: {
    "Ind/Pres": "present indicative",
    "Ind/Imp": "imperfect indicative",
    "Ind/Past": "preterite indicative",
    "Ind/Fut": "future indicative",
    "Cnd/": "conditional",
    "Sub/Pres": "present subjunctive",
    "Sub/Imp": "imperfect subjunctive",
    "Sub/Fut": "future subjunctive",
    "Imp/": "imperative",
  },
};

/** The tenses in the order of the grammars: Spanish's indicative, conditional, subjunctive, imperative. */
const TENSE_ORDER: Record<StudiedLanguageCode, readonly string[]> = { en: [], es: Object.keys(TENSES.es) };

/** The gerund's name: English's "-ing form", Spanish's "gerund". */
const GERUNDS: Record<StudiedLanguageCode, string> = { en: "-ing form", es: "gerund" };

/** Whether the infinitive is named: for Spanish only, as the French card does. */
const NAMES_INFINITIVE: Record<StudiedLanguageCode, boolean> = { en: false, es: true };

/** "feminine plural", "masculine singular", "plural": a nominal form's agreement. */
function agreement({ gender, number }: Agreement): string | undefined {
  if (gender) return number ? `${GENDERS[gender]} ${NUMBERS[number]}` : GENDERS[gender];
  return number === "Plur" ? "plural" : undefined;
}

/** "a", "a and b", "a, b and c". */
function join(items: readonly string[]): string {
  if (items.length <= 1) return items.join("");
  return `${items.slice(0, -1).join(", ")} and ${items.at(-1)}`;
}

const the = (name: string): Named => ({ article: "the", name });

const readingWords: ReadingWords = {
  tense: (key, studied) => TENSES[studied][key],
  name(kind: FormKind, studied, tense) {
    switch (kind.kind) {
      case "degree":
        return the(kind.degree === "Cmp" ? "comparative" : "superlative");
      case "agreement": {
        const name = agreement(kind);
        return name ? the(name) : null;
      }
      case "infinitive":
        return NAMES_INFINITIVE[studied] ? the("infinitive") : null;
      case "participle": {
        // A masculine singular participle is the participle itself; the singular goes unsaid.
        const agreed =
          kind.gender === "Masc" && kind.number !== "Plur"
            ? undefined
            : agreement(kind.number === "Sing" ? { gender: kind.gender } : kind);
        return the(agreed ? `${agreed} past participle` : "past participle");
      }
      case "gerund":
        return the(GERUNDS[studied]);
      case "finite":
        return tense === undefined ? null : the(tense);
    }
  },
  persons(persons, number, tense) {
    // "third-person singular", "first- and third-person singular".
    const ordinals = persons.map((p) => ORDINALS[p]!);
    const who = join([...ordinals.slice(0, -1).map((o) => `${o}-`), `${ordinals.at(-1)}-person`]);
    return the(`${who} ${NUMBERS[number]} ${tense}`);
  },
  order: (studied) => TENSE_ORDER[studied],
  numbers: ["Sing", "Plur"],
  mergeBy: "key",
};

const lineWords: LineWords = {
  names: (readings, studied) => nameReadings(readings, studied, readingWords),
  formOf: (named) => `${join(named.map((n) => n.name))} `,
  mayAlsoBe: (named) => `may also be the ${join(named.map((n) => n.name))} `,
  of: (word): GrammarLine => ["of ", { word }],
  pieces: (written) => `“${written}” = `,
  piecesSeparator: " + ",
};

export const grammar: typeof fr = {
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
    // "feminine noun": English puts the gender first.
    return named ? `${named} ${name}` : name;
  },
  lineText,
  join,
};
