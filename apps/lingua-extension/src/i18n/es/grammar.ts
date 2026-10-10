import type { GrammarTag, WordGrammar } from "../../analyzer/types.ts";
import {
  type Agreement,
  composeLines,
  describeForm,
  type FormKind,
  formKind,
  type GenderedName,
  type LineWords,
  mergeGenders,
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
// (generalise-lingua-card-wording D2–D4), reviewed by the owner (M9). It names Spanish forms in the
// RAE/ASALE's terms (M10): « tercera persona del singular del pretérito perfecto simple de
// indicativo de venir », « participio de hablar »; English forms by the names Spanish-language
// teaching of English gives them, never a Spanish tense's — the Spanish Wiktionary's English form-of
// wording (« Pasado simple del verbo (to) have », « Tercera persona del singular (he, she, it) del
// presente simple del verbo (to) go », « Participio pasado del verbo (to) have ») and M10's « forma
// en -ing », read on the en-es golden's real forms (add-lingua-spanish-card-wording D3); French forms
// in the RAE's terms too, as the Spanish card names Spanish's (M10: « pretérito perfecto simple de
// indicativo », « futuro simple de indicativo », « condicional simple »), its participles « participio
// presente » and « participio pasado » (add-lingua-french-word-card D2, D4). Spanish has articles and
// no elision: « de » before a tense contracts with its « el » (« del presente simple »), never before
// a word of the page. Readings merge by tag; the tenses come in the order its table lists them; the
// genders of one number are named once, « el masculino y femenino plural » (D6), as the English card
// names them.

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
 * The studied languages' moods and tenses, keyed `Mood/Tense`: Spanish's in the RAE's terms, listed in
 * the RAE's order — indicative, conditional, subjunctive, imperative — which is the order the card
 * names them in (`tenseOrder`); English's two as Spanish-language teaching names them — « presente
 * simple », not the RAE's « presente » (add-lingua-spanish-card-wording D3) — in the pack's order;
 * French's in the RAE's terms, as Spanish's, without the future subjunctive French has not, its
 * indicative and subjunctive of one tense said once after the indicative of that tense
 * (`Ind|Sub/…`, add-lingua-french-word-card D2, D3).
 */
const TENSES: Record<StudiedLanguageCode, TenseTable> = {
  en: { "Ind/Past": "pasado simple", "Ind/Pres": "presente simple" },
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
  fr: {
    "Ind/Pres": "presente de indicativo",
    "Ind|Sub/Pres": "presente de indicativo o de subjuntivo",
    "Ind/Imp": "pretérito imperfecto de indicativo",
    "Ind|Sub/Imp": "pretérito imperfecto de indicativo o de subjuntivo",
    "Ind/Past": "pretérito perfecto simple de indicativo",
    "Ind/Fut": "futuro simple de indicativo",
    "Cnd/": "condicional simple",
    "Sub/Pres": "presente de subjuntivo",
    "Sub/Imp": "pretérito imperfecto de subjuntivo",
    "Imp/": "imperativo",
  },
};

/** The gerund's name: English's « forma en -ing », Spanish's « gerundio »; French has none (`CARD_NAMES`). */
const GERUNDS: Partial<Record<StudiedLanguageCode, Named>> = {
  en: { article: "la", name: "forma en -ing" },
  es: { article: "el", name: "gerundio" },
};

/**
 * The past participle's name: English's « participio pasado », as Spanish-language teaching of English
 * names it (add-lingua-spanish-card-wording D3); Spanish's « participio », the RAE's; French's
 * « participio pasado », beside its « participio presente » (add-lingua-french-word-card D4).
 */
const PARTICIPLES: Record<StudiedLanguageCode, string> = {
  en: "participio pasado",
  es: "participio",
  fr: "participio pasado",
};

/** « femenino plural », « masculino singular », « plural »: a nominal form's agreement. */
function agreement({ gender, number }: Agreement): string | undefined {
  if (gender) return number ? `${GENDERS[gender]} ${NUMBERS[number]}` : GENDERS[gender];
  return number === "Plur" ? "plural" : undefined;
}

/**
 * « y » becomes « e » before a word that starts with the sound /i/ — « i », « í », « hi »: « subjuntivo e
 * imperativo », « participio e infinitivo » — but not before a diphthong (« y hielo »).
 */
function and(next: string): string {
  return /^h?[ií](?![aeiouáéó])/i.test(next) ? "e" : "y";
}

/** "a", "a y b", "a, b y c". */
function join(items: readonly string[]): string {
  if (items.length <= 1) return items.join("");
  const last = items.at(-1)!;
  return `${items.slice(0, -1).join(", ")} ${and(last)} ${last}`;
}

const el = (name: string): Named => ({ article: "el", name });

/** Each gendered agreement's name — « femenino singular » — and the gender and number it names. */
const GENDERED: ReadonlyMap<string, GenderedName> = new Map(
  Object.keys(GENDERS).flatMap((gender) =>
    (["Sing", "Plur"] as const).map((number) => [agreement({ gender, number })!, { gender, number }] as const),
  ),
);

/**
 * « masculino y femenino plural »: the genders of one number, named once, in the order of `GENDERS`
 * (add-lingua-french-word-card D6) — French gives a noun of both genders a reading per gender
 * (`sommes`: *somme*). No English reading carries a gender, so the en-es card does not move.
 */
function genders(named: readonly Named[]): Named[] {
  return mergeGenders(
    named,
    (n) => GENDERED.get(n.name),
    (merged, number) => el(`${join(merged.map((g) => GENDERS[g]!))} ${NUMBERS[number]}`),
    Object.keys(GENDERS),
  );
}

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
        return el(agreed ? `${PARTICIPLES[studied]} ${agreed}` : PARTICIPLES[studied]);
      }
      case "presentParticiple":
        return el("participio presente");
      case "gerund":
        return GERUNDS[studied] ?? null;
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
  names: (readings, studied) => genders(nameReadings(readings, studied, readingWords)),
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
