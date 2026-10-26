import type { GrammarTag, WordGrammar } from "../../analyzer/types.ts";
import {
  type Agreement,
  composeBlocks,
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

// The word card's grammar in Italian — a draft after the French renderer
// (add-lingua-italian-card-wording D1–D5), reviewed by the Italian reviewer agent (I2, D8) and shown
// to no reader before the Italian interface is (the Italian programme's change 45). It names
// Italian's forms by Treccani's names, the mood before the tense — « indicativo imperfetto »,
// « congiuntivo presente », « passato remoto » (D1); French's and Spanish's by the Italian name of the
// same tense — the passé simple and the pretérito perfecto simple are « passato remoto » — Spanish's
// future subjunctive, which Italian lacks, « congiuntivo futuro (futuro de subjuntivo) » (D2);
// English's as the Italian Wiktionary's English form-of wording names them — « passato semplice di
// eat », « presente semplice », « participio passato » — and the -ing form « forma in -ing », as M10
// gave Spanish its « forma en -ing » (D3). Italian has articles and elides: « di » before a tense
// contracts with its article (« del passato remoto », « dell'indicativo imperfetto »), never before a
// word of the page (« di eat », « di andare »); persons read « terza persona singolare », several of
// one tense and number named once, « prima e terza persona singolare » (D4). The apostrophe is
// U+0027, as the Italian catalogue writes « l'inglese ». Readings merge by tag; the tenses come in
// the order its table lists them; the genders of one number are named once, « il maschile e
// femminile plurale », as the English and Spanish cards name them.

const PARTS_OF_SPEECH: Record<string, string> = {
  ADJ: "aggettivo",
  ADP: "preposizione",
  ADV: "avverbio",
  AUX: "verbo ausiliare",
  CCONJ: "congiunzione",
  DET: "determinante",
  INTJ: "interiezione",
  NOUN: "sostantivo",
  NUM: "numerale",
  PART: "particella",
  PRON: "pronome",
  PROPN: "nome proprio",
  SCONJ: "congiunzione",
  VERB: "verbo",
};

const GENDERS: Record<string, string> = { Masc: "maschile", Fem: "femminile", Neut: "neutro", Com: "comune" };
const ORDINALS: Record<string, string> = { "1": "prima", "2": "seconda", "3": "terza" };
const NUMBERS: Record<string, string> = { Sing: "singolare", Plur: "plurale" };

/**
 * The studied languages' moods and tenses, keyed `Mood/Tense`: Italian's by Treccani's names, the
 * mood before the tense (D1), listed in Treccani's order of moods — indicative, conditional,
 * subjunctive, imperative (« Modi del verbo ») — which is the order the card names them in
 * (`tenseOrder`), its present read in both moods said once after the indicative present; French's
 * and Spanish's by the Italian name of the same tense (D2), French's merged moods after the
 * indicative of their tense, Spanish's future subjunctive bracketed as the Italian Wikipedia's
 * « Verbi spagnoli » gives it; English's two as the Italian Wiktionary names them (D3), in the pack's
 * order.
 */
const TENSES: Record<StudiedLanguageCode, TenseTable> = {
  en: { "Ind/Past": "passato semplice", "Ind/Pres": "presente semplice" },
  es: {
    "Ind/Pres": "indicativo presente",
    "Ind/Imp": "indicativo imperfetto",
    "Ind/Past": "passato remoto",
    "Ind/Fut": "futuro semplice",
    "Cnd/": "condizionale presente",
    "Sub/Pres": "congiuntivo presente",
    "Sub/Imp": "congiuntivo imperfetto",
    "Sub/Fut": "congiuntivo futuro (futuro de subjuntivo)",
    "Imp/": "imperativo",
  },
  fr: {
    "Ind/Pres": "indicativo presente",
    "Ind|Sub/Pres": "indicativo o congiuntivo presente",
    "Ind/Imp": "indicativo imperfetto",
    "Ind|Sub/Imp": "indicativo o congiuntivo imperfetto",
    "Ind/Past": "passato remoto",
    "Ind/Fut": "futuro semplice",
    "Cnd/": "condizionale presente",
    "Sub/Pres": "congiuntivo presente",
    "Sub/Imp": "congiuntivo imperfetto",
    "Imp/": "imperativo",
  },
  it: {
    "Ind/Pres": "indicativo presente",
    "Ind|Sub/Pres": "indicativo o congiuntivo presente",
    "Ind/Imp": "indicativo imperfetto",
    "Ind/Past": "passato remoto",
    "Ind/Fut": "futuro semplice",
    "Cnd/": "condizionale presente",
    "Sub/Pres": "congiuntivo presente",
    "Sub/Imp": "congiuntivo imperfetto",
    "Imp/": "imperativo",
  },
};

/** Italian elides the article before a vowel: « l'infinito », « dell'imperativo ». */
function elides(word: string): boolean {
  return /^[aeiouàèéìíòóùú]/i.test(word);
}

/** A masculine name with its article: « il passato remoto », « l'indicativo imperfetto ». */
function masculine(name: string): Named {
  return { article: elides(name) ? "l'" : "il", name };
}

/** « del passato remoto », « dell'indicativo imperfetto »: « di » contracted with a tense's article. */
function ofTense(tense: string): string {
  return elides(tense) ? `dell'${tense}` : `del ${tense}`;
}

/**
 * The gerund's name: English's « forma in -ing » (D3), Spanish's and Italian's « gerundio »; French
 * has none (`CARD_NAMES`).
 */
const GERUNDS: Partial<Record<StudiedLanguageCode, Named>> = {
  en: { article: "la", name: "forma in -ing" },
  es: masculine("gerundio"),
  it: masculine("gerundio"),
};

/** « femminile plurale », « maschile singolare », « plurale »: a nominal form's agreement. */
function agreement({ gender, number }: Agreement): string | undefined {
  if (gender) return number ? `${GENDERS[gender]} ${NUMBERS[number]}` : GENDERS[gender];
  return number === "Plur" ? "plurale" : undefined;
}

/** "a", "a e b", "a, b e c". No name opens with « e », so « ed » never arises (D4). */
function join(items: readonly string[]): string {
  if (items.length <= 1) return items.join("");
  return `${items.slice(0, -1).join(", ")} e ${items.at(-1)}`;
}

/** Each gendered agreement's name — « femminile singolare » — and the gender and number it names. */
const GENDERED: ReadonlyMap<string, GenderedName> = new Map(
  Object.keys(GENDERS).flatMap((gender) =>
    (["Sing", "Plur"] as const).map((number) => [agreement({ gender, number })!, { gender, number }] as const),
  ),
);

/** « maschile e femminile plurale »: the genders of one number, named once, in the order of `GENDERS`. */
function genders(named: readonly Named[]): Named[] {
  return mergeGenders(
    named,
    (n) => GENDERED.get(n.name),
    (merged, number) => masculine(`${join(merged.map((g) => GENDERS[g]!))} ${NUMBERS[number]}`),
    Object.keys(GENDERS),
  );
}

const readingWords: ReadingWords = {
  tenses: TENSES,
  name(kind: FormKind, studied, tense) {
    switch (kind.kind) {
      case "degree":
        return masculine(kind.degree === "Cmp" ? "comparativo" : "superlativo");
      case "agreement": {
        const name = agreement(kind);
        return name ? masculine(name) : null;
      }
      case "infinitive":
        return masculine("infinito");
      case "participle": {
        // A masculine singular participle is the participle itself; the singular goes unsaid.
        const agreed =
          kind.gender === "Masc" && kind.number !== "Plur"
            ? undefined
            : agreement(kind.number === "Sing" ? { gender: kind.gender } : kind);
        return masculine(agreed ? `participio passato ${agreed}` : "participio passato");
      }
      case "presentParticiple":
        return masculine("participio presente");
      case "gerund":
        return GERUNDS[studied] ?? null;
      case "finite":
        return tense === undefined ? null : masculine(tense);
    }
  },
  persons(persons, number, tense) {
    // « terza persona singolare », « prima e terza persona singolare »: no « del » before the number.
    const who = join(persons.map((p) => ORDINALS[p]!));
    return { article: "la", name: `${who} persona ${NUMBERS[number]} ${ofTense(tense)}` };
  },
  numbers: ["Sing", "Plur"],
  mergeBy: "key",
};

/** « il passato semplice », « l'infinito », « la forma in -ing »: each name with its article. */
function withArticles(named: readonly Named[]): string {
  return join(named.map((n) => (n.article === "l'" ? `l'${n.name}` : `${n.article} ${n.name}`)));
}

const lineWords: LineWords = {
  names: (readings, studied) => genders(nameReadings(readings, studied, readingWords)),
  formOf: (named) => `${join(named.map((n) => n.name))} `,
  mayAlsoBe: (named) => `può anche essere ${withArticles(named)} `,
  // « di eat », « di andare »: « di » never elides before a word of the page.
  of: (word): GrammarLine => ["di ", { word }],
  pieces: (written) => `«${written}» = `,
  piecesSeparator: " + ",
};

export const grammar: typeof fr = {
  grammarLines(grammar: WordGrammar, headword, surface, written, studied: StudiedLanguageCode = "en") {
    return composeLines(describeForm(grammar, headword, surface, written), studied, lineWords);
  },
  grammarBlocks(grammar: WordGrammar, headword, surface, written, studied: StudiedLanguageCode = "en") {
    return composeBlocks(describeForm(grammar, headword, surface, written), studied, lineWords);
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
