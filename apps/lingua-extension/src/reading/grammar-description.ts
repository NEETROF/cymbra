import type { GrammarTag, WordGrammar } from "../analyzer/types.ts";
import type { GrammarLine, Named, StudiedLanguageCode } from "../i18n/index.ts";

// What a word card knows about a form, in no language (generalise-lingua-card-wording D1): the
// readings as the engine's Universal Dependencies tags, merged and deduplicated by tag, the
// dictionary form left out on its own card — and with it its plural, where the card also reads the
// form in the singular, for every studied language (refine-lingua-card-invariable-plurals D2) —, the
// other dictionary forms and the pieces. A renderer per interface language
// (`src/i18n/{fr,en,es}/grammar.ts`) says it in that language's words; this module holds the
// decisions every renderer shares — what is named at all, and how a line is composed from its parts
// — so a renderer can only word what the French card names, never more.

/** The engine's parts of speech (lingua-core `PARTS_OF_SPEECH`), in UD's order. */
export const PARTS_OF_SPEECH: readonly string[] = [
  "ADJ",
  "ADP",
  "ADV",
  "AUX",
  "CCONJ",
  "DET",
  "INTJ",
  "NOUN",
  "NUM",
  "PART",
  "PRON",
  "PROPN",
  "PUNCT",
  "SCONJ",
  "SYM",
  "VERB",
  "X",
];

/** The engine's features and their values (lingua-core `FEATURES`): a closed vocabulary. */
export const FEATURES: Readonly<Record<string, readonly string[]>> = {
  Case: ["Acc", "Com", "Dat", "Nom"],
  Definite: ["Def", "Ind"],
  Degree: ["Cmp", "Pos", "Sup"],
  Gender: ["Com", "Fem", "Masc", "Neut"],
  Mood: ["Cnd", "Imp", "Ind", "Sub"],
  Number: ["Plur", "Sing"],
  Person: ["1", "2", "3"],
  PronType: ["Art", "Dem", "Ind", "Int", "Prs", "Rel"],
  Reflex: ["Yes"],
  Tense: ["Fut", "Imp", "Past", "Pqp", "Pres"],
  VerbForm: ["Fin", "Ger", "Inf", "Part"],
};

/** One reading of a form: its part of speech and every feature of the vocabulary, as tags. */
export interface Reading {
  pos: string;
  case?: string;
  definite?: string;
  degree?: string;
  gender?: string;
  number?: string;
  /** The persons this reading holds, merged from readings equal but for their person. */
  persons: string[];
  mood?: string;
  tense?: string;
  verbForm?: string;
  pronType?: string;
  reflex?: string;
}

/** The feature of a reading each UD feature name fills (`Person` is `persons`). */
const FIELDS = {
  Case: "case",
  Definite: "definite",
  Degree: "degree",
  Gender: "gender",
  Mood: "mood",
  Number: "number",
  PronType: "pronType",
  Reflex: "reflex",
  Tense: "tense",
  VerbForm: "verbForm",
} as const satisfies Record<string, keyof Reading>;

type FeatureName = keyof typeof FIELDS;

/** A sense group's heading, as tags: its part of speech and the word's gender. */
export interface SenseDescription {
  pos?: string;
  gender?: string;
}

/** A form, described once: what the card says about it, in no language. */
export interface FormDescription {
  /** The dictionary form the card is about, as the document writes it. */
  headword: string;
  /** The word as it stands on the page (the whole word, for a split one). */
  written: string;
  /**
   * The readings of the headword. On its own card, the dictionary form is left out
   * (`isDictionaryForm`), and so is its plural where the card also reads it in the singular
   * (`isInvariablePlural`); a plural alone is kept.
   */
  own: Reading[];
  /** What else the form may be, one dictionary form each. */
  others: { lemma: string; readings: Reading[] }[];
  /** The pieces the pre-pass split the written word into; empty when it did not split it. */
  pieces: string[];
  /** Whether the form is spelled like its dictionary form, case aside. */
  sameAsHeadword: boolean;
  /** The headings of the gloss's groups of senses. */
  senses: SenseDescription[];
}

const NOMINAL = new Set(["NOUN", "PROPN", "ADJ", "DET", "PRON"]);

/**
 * Whether a tag only says what the dictionary form is — its infinitive, a noun's singular, an
 * adjective's masculine singular — and so gives no line on that form's own card.
 */
export function isDictionaryForm(tag: GrammarTag): boolean {
  const f = tag.features ?? {};
  if (f.VerbForm === "Inf") return true;
  if (!NOMINAL.has(tag.pos) || f.Number !== "Sing" || f.Degree) return false;
  return tag.pos === "NOUN" || tag.pos === "PROPN" || f.Gender !== "Fem";
}

/**
 * Whether a tag is the plural of a word the card also reads in the singular — a noun's, proper
 * noun's, adjective's, determiner's or pronoun's plural without a degree, whatever its gender, beside
 * a singular of that part of speech among `tags`, the card's own readings — and so, like
 * `isDictionaryForm`, gives no line on that form's own card: the second omission of the dictionary
 * form's own card (refine-lingua-card-invariable-plurals D1). `crisis`, `lunes` (Spanish), `temps`
 * (French) are read in both numbers and their plural goes; a plural with no such singular — a noun
 * used only in the plural, `gafas`, `gens`, every English one (`police`, `fish`), whose table writes
 * no noun's singular — keeps its line, the card's one sign of the word's number. Keyed by nothing:
 * the same for every studied language and every renderer.
 */
export function isInvariablePlural(tag: GrammarTag, tags: readonly GrammarTag[]): boolean {
  const f = tag.features ?? {};
  if (!NOMINAL.has(tag.pos) || f.Number !== "Plur" || f.Degree) return false;
  return tags.some((other) => other.pos === tag.pos && other.features?.Number === "Sing");
}

/** A tag as a reading: every feature of the vocabulary; one it does not know, or an empty value, left out. */
export function readingOf(tag: GrammarTag): Reading {
  const f = tag.features ?? {};
  const reading: Reading = { pos: tag.pos, persons: f.Person ? [f.Person] : [] };
  for (const name of Object.keys(FIELDS) as FeatureName[]) {
    const value = f[name];
    if (value) reading[FIELDS[name]] = value;
  }
  return reading;
}

/** A reading as the tags it stands for: one per person, or one without a person. */
export function tagsOf(reading: Reading): GrammarTag[] {
  const features: Record<string, string> = {};
  for (const name of Object.keys(FIELDS) as FeatureName[]) {
    const value = reading[FIELDS[name]];
    if (typeof value === "string") features[name] = value;
  }
  const people = reading.persons.length > 0 ? reading.persons : [undefined];
  return people.map((person) => {
    const all = person === undefined ? { ...features } : { ...features, Person: person };
    return Object.keys(all).length > 0 ? { pos: reading.pos, features: all } : { pos: reading.pos };
  });
}

const PERSONS = new Set(FEATURES.Person);

/**
 * The readings of a list of tags, in the order of their first occurrence: a tag seen twice gives
 * one reading, and tags equal but for a person of the vocabulary give one reading holding each
 * person once. On the dictionary form's own card (`same`), a tag that only says what that form is
 * is left out (`isDictionaryForm`), and so is the plural of a word read there in the singular too
 * (`isInvariablePlural`), in every studied language; a plural alone is kept.
 *
 * This merge by tag is the one the spec asks of the description (it names no language, so it is the
 * one merge it can make). `nameReadings` merges the persons again, by tense and number — on the
 * tense's name for the French card, on its key for the others — which subsumes it: a renderer handed
 * the raw tags, one per person, says the same bytes (the French renderer did, for a while). The merge
 * stays here because the description, not a renderer, is what the spec says is merged.
 */
function describeReadings(tags: readonly GrammarTag[], same: boolean): Reading[] {
  const out: Reading[] = [];
  const at = new Map<string, Reading>();
  for (const tag of tags) {
    if (same && (isDictionaryForm(tag) || isInvariablePlural(tag, tags))) continue;
    const reading = readingOf(tag);
    const person = reading.persons[0];
    const merges = person !== undefined && PERSONS.has(person);
    const key = JSON.stringify([
      ...tagsOf({ ...reading, persons: [] }).map((t) => [t.pos, Object.entries(t.features ?? {})]),
      merges ? "person" : (person ?? ""),
    ]);
    const seen = at.get(key);
    if (!seen) {
      at.set(key, reading);
      out.push(reading);
    } else if (merges && !seen.persons.includes(person)) {
      seen.persons.push(person);
    }
  }
  return out;
}

/** A sense group's tag as its heading's tags. */
export function senseOf(tag: GrammarTag | undefined): SenseDescription {
  if (!tag) return {};
  const gender = tag.features?.Gender;
  return gender === undefined ? { pos: tag.pos } : { pos: tag.pos, gender };
}

/**
 * The description of a form a card was opened on. `surface` is the form the readings are of (the
 * piece, for a split word); `written` the word as it stands on the page.
 */
export function describeForm(
  grammar: WordGrammar,
  headword: string,
  surface: string,
  written: string,
): FormDescription {
  const same = surface.trim().toLowerCase() === headword.toLowerCase();
  return {
    headword,
    written,
    own: describeReadings(grammar.readings, same),
    others: grammar.others.map((other) => ({ lemma: other.lemma, readings: describeReadings(other.readings, false) })),
    pieces: grammar.pieces.length > 1 ? [...grammar.pieces] : [],
    sameAsHeadword: same,
    senses: grammar.senses.map((group) => senseOf(group.tag)),
  };
}

// — What a card names: the same for every renderer —

/** The gender a renderer can name; a value outside the vocabulary is none. */
const GENDERS = new Set(FEATURES.Gender);

/** A nominal form's or a participle's agreement: a gender of the vocabulary, and its number. */
export interface Agreement {
  gender?: string;
  number?: "Sing" | "Plur";
}

/** What a reading makes of a form, as a renderer words it; null when no card names it. */
export type FormKind =
  | { kind: "degree"; degree: "Cmp" | "Sup" }
  | ({ kind: "agreement" } & Agreement)
  | { kind: "infinitive" }
  /** The past participle, with its agreement. */
  | ({ kind: "participle" } & Agreement)
  /** The present participle, where the studied language's is named (`CARD_NAMES`: French's). */
  | { kind: "presentParticiple" }
  | { kind: "gerund" }
  /** A finite form: its mood and tense as `finiteKey` gives them, its persons and number when known. */
  | { kind: "finite"; key: string; persons: string[]; number?: "Sing" | "Plur" };

function agreementOf(reading: Reading): Agreement {
  const out: Agreement = {};
  if (reading.gender !== undefined && GENDERS.has(reading.gender)) out.gender = reading.gender;
  if (reading.number === "Sing" || reading.number === "Plur") out.number = reading.number;
  return out;
}

/** What a card names of one studied language's forms (`CARD_NAMES`). */
export interface CardNames {
  /** The infinitive is named (« infinitif de hablar »). */
  infinitive: boolean;
  /** The present participle (`VerbForm=Part|Tense=Pres`) is named, apart from the past participle. */
  presentParticiple: boolean;
  /** The gerund (`VerbForm=Ger`) is named. */
  gerund: boolean;
  /** The indicative and the subjunctive of one tense, persons and number, are named once (M21). */
  moods: boolean;
}

/**
 * What a card names of each studied language's forms, decided here once for every renderer
 * (add-lingua-french-word-card D1) — keyed by the renderers' `StudiedLanguageCode`, not by
 * `profile.ts`'s `NAMES` (the backup's language names, which change 52 widens with
 * `StudiedLanguage`). A renderer only words what is named:
 * - the infinitive: Spanish's and French's are named (« infinitif de hablar »; a French infinitive
 *   spelled otherwise than its dictionary form is mostly a 1990 spelling, `abimer`), English's never
 *   was — an English card already shows its verb as the infinitive;
 * - the present participle: French's (`parlant`, D4); English's and Spanish's tables write none;
 * - the gerund: English's -ing form and Spanish's gerundio; French has none — « en parlant » is the
 *   participle after « en » (D4);
 * - the moods' merge (D3): French's « parle » is said once in the present indicative or
 *   subjunctive; no English or Spanish form meets the rule, and theirs stay unmerged.
 * A plural spelled like the dictionary form is not keyed: `isInvariablePlural` leaves it out for
 * every studied language (refine-lingua-card-invariable-plurals D3).
 */
export const CARD_NAMES: Readonly<Record<StudiedLanguageCode, Readonly<CardNames>>> = {
  en: { infinitive: false, presentParticiple: false, gerund: true, moods: false },
  es: { infinitive: true, presentParticiple: false, gerund: true, moods: false },
  fr: { infinitive: true, presentParticiple: true, gerund: false, moods: true },
};

/**
 * A finite form's mood and tense, keyed `Mood/Tense` as the studied language has them: Spanish and
 * French name the conditional and the imperative without a tense (add-lingua-french-word-card D2:
 * French keyed as Spanish, as change 45 stores it); an English form without a mood is in the
 * indicative, and English has no other mood a card names. Undefined when there is nothing to key.
 * A renderer's tense tables are keyed by it, so they name the same tenses.
 */
export function finiteKey(reading: Reading, studied: StudiedLanguageCode): string | undefined {
  if (studied !== "en") {
    const bare = reading.mood === "Cnd" || reading.mood === "Imp";
    return `${reading.mood ?? ""}/${bare ? "" : (reading.tense ?? "")}`;
  }
  if (reading.mood !== undefined && reading.mood !== "Ind") return undefined;
  return reading.tense ? `Ind/${reading.tense}` : undefined;
}

/**
 * The key a tense's indicative and subjunctive are named under once (add-lingua-french-word-card
 * D3, M21): `Ind|Sub/Pres` for `Ind/Pres` and `Sub/Pres`; undefined for a key of another mood.
 */
export function mergedMoodsKey(key: string): string | undefined {
  const tense = key.startsWith("Ind/") ? key.slice(4) : undefined;
  return tense === undefined || tense === "" ? undefined : `Ind|Sub/${tense}`;
}

/**
 * What a reading makes of a form, or null when no card names it: a degree, a nominal form's
 * agreement (a noun's plural; a gendered pronoun's or determiner's), the infinitive, the present
 * participle and the gerund where the studied language's are named (`CARD_NAMES`), the past
 * participle, a finite form. A renderer may still leave a tense unnamed — one its table lacks — and
 * the French renderer's choices are every renderer's, which a test enumerates.
 */
export function formKind(reading: Reading, studied: StudiedLanguageCode): FormKind | null {
  if (reading.degree === "Cmp" || reading.degree === "Sup") return { kind: "degree", degree: reading.degree };
  if (NOMINAL.has(reading.pos)) {
    const agreement = agreementOf(reading);
    if (!agreement.gender && agreement.number !== "Plur") return null;
    // Without a gender, only the plural of a noun, a proper noun or an adjective is named, as it always was.
    const named = reading.pos === "NOUN" || reading.pos === "PROPN" || reading.pos === "ADJ";
    if (reading.gender === undefined && !named) return null;
    return { kind: "agreement", ...agreement };
  }
  if (reading.pos !== "VERB" && reading.pos !== "AUX") return null;
  switch (reading.verbForm) {
    case "Inf":
      return CARD_NAMES[studied].infinitive ? { kind: "infinitive" } : null;
    case "Part":
      if (reading.tense === "Past") return { kind: "participle", ...agreementOf(reading) };
      return reading.tense === "Pres" && CARD_NAMES[studied].presentParticiple ? { kind: "presentParticiple" } : null;
    case "Ger":
      return CARD_NAMES[studied].gerund ? { kind: "gerund" } : null;
    case "Fin": {
      const key = finiteKey(reading, studied);
      if (key === undefined) return null;
      const { number } = agreementOf(reading);
      const persons = number ? reading.persons.filter((p) => PERSONS.has(p)) : [];
      return persons.length > 0 ? { kind: "finite", key, persons, number } : { kind: "finite", key, persons: [] };
    }
    default:
      return null;
  }
}

/** A studied language's tense names by `finiteKey`, listed in the order its tenses come in. */
export type TenseTable = Readonly<Record<string, string>>;

/** How a renderer words what `nameReadings` hands it. */
export interface ReadingWords {
  /** Each studied language's tenses, as the renderer names them; a key the table lacks is unnamed. */
  tenses: Readonly<Record<StudiedLanguageCode, TenseTable>>;
  /** A kind other than a finite form of known persons; null when the renderer leaves it unnamed. */
  name(kind: FormKind, studied: StudiedLanguageCode, tense: string | undefined): Named | null;
  /** A finite form's persons (sorted) and number, in its tense. */
  persons(persons: readonly string[], number: "Sing" | "Plur", tense: string): Named;
  /** The order of the numbers within one tense. */
  numbers: readonly ("Sing" | "Plur")[];
  /** What merges the persons of two readings: the tense's name (French), or its key (by tag). */
  mergeBy: "name" | "key";
}

/** A finite form's tense as the renderer names it for the studied language; undefined: unnamed. */
function tenseName(words: ReadingWords, key: string, studied: StudiedLanguageCode): string | undefined {
  return words.tenses[studied][key];
}

/**
 * The order a studied language's named tenses come in, derived once from the renderer's table:
 * every studied language's but English's as the table lists them — the grammars' order, indicative,
 * conditional, subjunctive, imperative, French's merged moods after the indicative of their tense
 * (add-lingua-french-word-card D2) — and English's two in the pack's order, as the French card
 * always gave them, which an empty order means.
 */
export function tenseOrder(tenses: ReadingWords["tenses"], studied: StudiedLanguageCode): readonly string[] {
  return studied === "en" ? [] : Object.keys(tenses[studied]);
}

/** A kind as a renderer words it: a finite form through its tense table, the others as they are. */
export function nameKind(kind: FormKind, studied: StudiedLanguageCode, words: ReadingWords): Named | null {
  if (kind.kind !== "finite") return words.name(kind, studied, undefined);
  const tense = tenseName(words, kind.key, studied);
  if (tense === undefined) return null;
  return kind.number && kind.persons.length > 0
    ? words.persons([...kind.persons].sort(), kind.number, tense)
    : words.name(kind, studied, tense);
}

type Group = { at: number; key: string; number: "Sing" | "Plur"; tense: string; persons: string[] };

/** Whether two groups hold the same persons, in any order. */
function samePersons(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((p) => b.includes(p));
}

/**
 * The indicative and the subjunctive of one tense said once (add-lingua-french-word-card D3, M21):
 * where the studied language merges its moods (`CARD_NAMES`) and the renderer names the merged key,
 * a group keyed `Ind/T` and one keyed `Sub/T`, of one number and the same persons, become one group
 * keyed `Ind|Sub/T`, at the place of the first. Answers the places the merge emptied.
 */
function mergeMoods(groups: Group[], studied: StudiedLanguageCode, words: ReadingWords): Set<number> {
  const emptied = new Set<number>();
  if (!CARD_NAMES[studied].moods) return emptied;
  for (const indicative of [...groups]) {
    const both = mergedMoodsKey(indicative.key);
    const tense = both === undefined ? undefined : tenseName(words, both, studied);
    if (both === undefined || tense === undefined) continue;
    const subjunctive = groups.find(
      (g) =>
        g.key === `Sub/${indicative.key.slice(4)}` &&
        g.number === indicative.number &&
        samePersons(g.persons, indicative.persons),
    );
    if (!subjunctive) continue;
    emptied.add(Math.max(indicative.at, subjunctive.at));
    Object.assign(indicative, { at: Math.min(indicative.at, subjunctive.at), key: both, tense });
    groups.splice(groups.indexOf(subjunctive), 1);
  }
  return emptied;
}

/**
 * The names of a list of readings, each once, in the pack's order: the persons of one tense and one
 * number are named once — and, where the studied language merges them, the indicative and the
 * subjunctive of one tense, persons and number (`mergeMoods`) —, and the tenses come in the studied
 * language's order (`tenseOrder`) at the places the pack's order gave the first of them.
 */
export function nameReadings(readings: readonly Reading[], studied: StudiedLanguageCode, words: ReadingWords): Named[] {
  const out: Named[] = [];
  const merged = new Map<string, Group>();
  for (const reading of readings) {
    // A finite form of known persons and number is grouped whatever else it carries, as the
    // French card always grouped it.
    const key = reading.verbForm === "Fin" ? finiteKey(reading, studied) : undefined;
    const tense = key === undefined ? undefined : tenseName(words, key, studied);
    const { number } = agreementOf(reading);
    const persons = reading.persons.filter((p) => PERSONS.has(p));
    if (key !== undefined && tense !== undefined && number && persons.length > 0) {
      const id = `${number}|${words.mergeBy === "name" ? tense : key}`;
      const group = merged.get(id);
      if (!group) {
        merged.set(id, { at: out.length, key, number, tense, persons });
        out.push({ article: "", name: "" });
      } else {
        for (const p of persons) if (!group.persons.includes(p)) group.persons.push(p);
      }
      continue;
    }
    const kind = formKind(reading, studied);
    const named = kind ? nameKind(kind, studied, words) : null;
    if (named && !out.some((n) => n.name === named.name)) out.push(named);
  }
  const groups = [...merged.values()];
  const emptied = mergeMoods(groups, studied, words);
  const order = tenseOrder(words.tenses, studied);
  // A key the order lacks ranks with its first — which never happens while the order lists every
  // named tense (Spanish, French) or none at all (English: the pack's order), as the tables do.
  const rank = (key: string): number => Math.max(0, order.indexOf(key));
  // The places the groups fill, in the pack's order: a merge of moods may have moved a group to an
  // earlier one (`mergeMoods`).
  const places = groups.map((g) => g.at).sort((a, b) => a - b);
  groups.sort((a, b) => rank(a.key) - rank(b.key) || words.numbers.indexOf(a.number) - words.numbers.indexOf(b.number));
  groups.forEach(({ number, tense, persons }, i) => {
    out[places[i]!] = words.persons([...persons].sort(), number, tense);
  });
  return emptied.size === 0 ? out : out.filter((_, i) => !emptied.has(i));
}

/** A gendered agreement a renderer names, read back as the gender and the number it names. */
export type GenderedName = { gender: string; number: "Sing" | "Plur" };

/**
 * The names of a line, the genders of one number named once — "masculine and feminine singular",
 * not "feminine singular and masculine singular" — in the renderer's `order` of genders, where the
 * first of them stood (add-lingua-english-card-wording D3; the Spanish renderer's too since
 * add-lingua-french-word-card D6: « el masculino y femenino plural de somme »). Every gender and
 * number the French names is still named: only the wording merges. `agreed` reads a name back as
 * the gendered agreement it names, if it is one; `named` words the genders of one number.
 */
export function mergeGenders(
  names: readonly Named[],
  agreed: (name: Named) => GenderedName | undefined,
  named: (genders: readonly string[], number: "Sing" | "Plur") => Named,
  order: readonly string[],
): Named[] {
  const out: Named[] = [];
  const byNumber = new Map<"Sing" | "Plur", { at: number; genders: string[] }>();
  for (const n of names) {
    const agreement = agreed(n);
    const group = agreement && byNumber.get(agreement.number);
    if (!agreement) {
      out.push(n);
    } else if (group) {
      if (!group.genders.includes(agreement.gender)) group.genders.push(agreement.gender);
    } else {
      byNumber.set(agreement.number, { at: out.length, genders: [agreement.gender] });
      out.push(n);
    }
  }
  for (const [number, { at, genders }] of byNumber) {
    if (genders.length < 2) continue;
    out[at] = named(
      [...genders].sort((a, b) => order.indexOf(a) - order.indexOf(b)),
      number,
    );
  }
  return out;
}

/** How a renderer composes the lines of a card. */
export interface LineWords {
  /** The names of a list of readings. */
  names(readings: readonly Reading[], studied: StudiedLanguageCode): Named[];
  /** The text before the dictionary form, on a line saying what the form is: « prétérit et participe passé ». */
  formOf(names: readonly Named[]): string;
  /** The text before the dictionary form, on a line saying what the form may also be. */
  mayAlsoBe(names: readonly Named[]): string;
  /** The dictionary form a reading is of, set apart: « de go », « d’eat ». */
  of(word: string): GrammarLine;
  /** The text before the pieces of a split word: « « don't » = ». */
  pieces(written: string): string;
  /** Between two pieces. */
  piecesSeparator: string;
}

/**
 * The lines a card shows about a form, or none: the pieces of a split word; what the form is — or,
 * for a form spelled like its dictionary form, what it may also be; what else it may be, one line
 * per dictionary form.
 */
export function composeLines(form: FormDescription, studied: StudiedLanguageCode, words: LineWords): GrammarLine[] {
  const lines: GrammarLine[] = [];
  if (form.pieces.length > 0) {
    const pieces: GrammarLine = [words.pieces(form.written)];
    form.pieces.forEach((piece, i) => {
      if (i > 0) pieces.push(words.piecesSeparator);
      pieces.push({ word: piece });
    });
    lines.push(pieces);
  }
  const own = words.names(form.own, studied);
  if (own.length > 0) {
    lines.push([form.sameAsHeadword ? words.mayAlsoBe(own) : words.formOf(own), ...words.of(form.headword)]);
  }
  for (const other of form.others) {
    const named = words.names(other.readings, studied);
    if (named.length > 0) lines.push([words.mayAlsoBe(named), ...words.of(other.lemma)]);
  }
  return lines;
}

/** A line as plain text — what a test or an accessibility label reads. */
export function lineText(line: GrammarLine): string {
  return line.map((s) => (typeof s === "string" ? s : s.word)).join("");
}
