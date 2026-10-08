import type { GrammarTag, WordGrammar } from "../../analyzer/types.ts";
import {
  type Agreement,
  composeLines,
  describeForm,
  type FormKind,
  formKind,
  type LineWords,
  nameKind,
  nameReadings,
  type ReadingWords,
  readingOf,
  senseOf,
  type TenseTable,
} from "../../reading/grammar-description.ts";
import type { grammar as fr } from "../fr/grammar.ts";
import type { GrammarLine, Named, StudiedLanguageCode } from "../index.ts";

// The word card's grammar in Spanish — a draft after the French renderer
// (generalise-lingua-card-wording D2–D4), reviewed by the owner (M9). It names the forms in the
// RAE/ASALE's terms (M10): « tercera persona del singular del pretérito perfecto simple de
// indicativo de venir », « participio de hablar »; English's tenses by the names Spanish teaching
// gives them (« pasado simple »). Spanish has articles and no elision: « de » before a tense
// contracts with its « el » (« del presente »), never before a word of the page. Readings merge by
// tag; the tenses come in the order its table lists them.

const PARTS_OF_SPEECH: Record<string, string> = {
  ADJ: "adjetivo",
  ADP: "preposición",
  ADV: "adverbio",
  AUX: "verbo auxiliar",
  CCONJ: "conjunción",
  DET: "determinante",
  INTJ: "interjección",
  NOUN: "sustantivo",
  NUM: "numeral",
  PART: "partícula",
  PRON: "pronombre",
  PROPN: "nombre propio",
  SCONJ: "conjunción",
  VERB: "verbo",
};

const GENDERS: Record<string, string> = { Masc: "masculino", Fem: "femenino", Neut: "neutro", Com: "común" };
const ORDINALS: Record<string, string> = { "1": "primera", "2": "segunda", "3": "tercera" };
const NUMBERS: Record<string, string> = { Sing: "singular", Plur: "plural" };

/**
 * The studied languages' moods and tenses, in the RAE's terms, keyed `Mood/Tense` and listed in the
 * RAE's order — indicative, conditional, subjunctive, imperative — which is the order the card names
 * them in (`tenseOrder`; English's two come in the pack's order).
 */
const TENSES: Record<StudiedLanguageCode, TenseTable> = {
  en: { "Ind/Past": "pasado simple", "Ind/Pres": "presente" },
  es: {
    "Ind/Pres": "presente de indicativo",
    "Ind/Imp": "pretérito imperfecto de indicativo",
    "Ind/Past": "pretérito perfecto simple de indicativo",
    "Ind/Fut": "futuro simple de indicativo",
    "Cnd/": "condicional simple",
    "Sub/Pres": "presente de subjuntivo",
    "Sub/Imp": "pretérito imperfecto de subjuntivo",
    "Sub/Fut": "futuro de subjuntivo",
    "Imp/": "imperativo",
  },
};

/** The gerund's name: English's « forma en -ing », Spanish's « gerundio ». */
const GERUNDS: Record<StudiedLanguageCode, Named> = {
  en: { article: "la", name: "forma en -ing" },
  es: { article: "el", name: "gerundio" },
};

/** « femenino plural », « masculino singular », « plural »: a nominal form's agreement. */
function agreement({ gender, number }: Agreement): string | undefined {
  if (gender) return number ? `${GENDERS[gender]} ${NUMBERS[number]}` : GENDERS[gender];
  return number === "Plur" ? "plural" : undefined;
}

/** « y » becomes « e » before a word that starts with the sound /i/ (« subjuntivo e imperativo »). */
function and(next: string): string {
  return /^h?i(?![aeiouáéó])/i.test(next) ? "e" : "y";
}

/** "a", "a y b", "a, b y c". */
function join(items: readonly string[]): string {
  if (items.length <= 1) return items.join("");
  const last = items.at(-1)!;
  return `${items.slice(0, -1).join(", ")} ${and(last)} ${last}`;
}

const el = (name: string): Named => ({ article: "el", name });

const readingWords: ReadingWords = {
  tenses: TENSES,
  name(kind: FormKind, studied, tense) {
    switch (kind.kind) {
      case "degree":
        return el(kind.degree === "Cmp" ? "comparativo" : "superlativo");
      case "agreement": {
        const name = agreement(kind);
        return name ? el(name) : null;
      }
      case "infinitive":
        return el("infinitivo");
      case "participle": {
        // A masculine singular participle is the participle itself; the singular goes unsaid.
        const agreed =
          kind.gender === "Masc" && kind.number !== "Plur"
            ? undefined
            : agreement(kind.number === "Sing" ? { gender: kind.gender } : kind);
        return el(agreed ? `participio ${agreed}` : "participio");
      }
      case "gerund":
        return GERUNDS[studied];
      case "finite":
        return tense === undefined ? null : el(tense);
    }
  },
  persons(persons, number, tense) {
    // « tercera persona del singular », « primera y tercera persona del singular ».
    const who = join(persons.map((p) => ORDINALS[p]!));
    return { article: "la", name: `${who} persona del ${NUMBERS[number]} del ${tense}` };
  },
  numbers: ["Sing", "Plur"],
  mergeBy: "key",
};

const lineWords: LineWords = {
  names: (readings, studied) => nameReadings(readings, studied, readingWords),
  formOf: (named) => `${join(named.map((n) => n.name))} `,
  mayAlsoBe: (named) => `también puede ser ${join(named.map((n) => `${n.article} ${n.name}`))} `,
  of: (word): GrammarLine => ["de ", { word }],
  pieces: (written) => `«${written}» = `,
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
    return named ? `${name} ${named}` : name;
  },
  join,
};
