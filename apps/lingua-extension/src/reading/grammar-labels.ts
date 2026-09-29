import type { GrammarTag, WordGrammar } from "../analyzer/types.ts";

// The words a card uses for a form's grammar (add-lingua-word-grammar, design D7). The pack
// speaks Universal Dependencies codes; the reader never sees one. A code these tables cannot
// name gives no text at all, so a pack from a later vocabulary can only ever show less —
// never a code.
//
// The names of parts of speech, gender, number and person are the interface's own. A tense's
// name depends on the studied language — `Tense=Past` is the prétérit in English and the passé
// simple in Spanish — so the verb-form names are keyed by studied language, and only English's
// ship here: each language's pack brings its own.

/** The studied languages whose verb forms this interface can name. */
export type StudiedLanguageCode = "en";

/** A name, with the article French gives it ("le prétérit", "la forme en -ing"). */
export interface Named {
  article: "le" | "la";
  name: string;
}

/** One segment of a grammar line: plain text, or a word of the studied language (set apart). */
export type LineSegment = string | { word: string };

/** One line of a card's grammar block. */
export type GrammarLine = LineSegment[];

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
const NUMBERS: Record<string, string> = { Sing: "du singulier", Plur: "du pluriel" };

/** English finite tenses, named as French schools name them. */
const ENGLISH_TENSES: Record<string, string> = { Past: "prétérit", Pres: "présent" };

/** The heading of a group of senses: its part of speech, with the word's gender; null for none. */
export function senseHeading(tag: GrammarTag | undefined): string | null {
  const pos = tag ? PARTS_OF_SPEECH[tag.pos] : undefined;
  if (!pos) return null;
  const gender = tag?.features?.Gender;
  const named = gender ? GENDERS[gender] : undefined;
  return named ? `${pos} ${named}` : pos;
}

/** What a reading makes of a form, in words; null when this interface cannot say. */
export function readingName(tag: GrammarTag, studied: StudiedLanguageCode = "en"): Named | null {
  const f = tag.features ?? {};
  if (f.Degree === "Cmp") return { article: "le", name: "comparatif" };
  if (f.Degree === "Sup") return { article: "le", name: "superlatif" };
  if (tag.pos === "NOUN" || tag.pos === "PROPN") {
    return f.Number === "Plur" ? { article: "le", name: "pluriel" } : null;
  }
  if (tag.pos !== "VERB" && tag.pos !== "AUX") return null;
  switch (f.VerbForm) {
    case "Part":
      return f.Tense === "Past" ? { article: "le", name: "participe passé" } : null;
    case "Ger":
      return studied === "en" ? { article: "la", name: "forme en -ing" } : null;
    case "Fin": {
      if (f.Mood !== undefined && f.Mood !== "Ind") return null;
      const tense = f.Tense ? ENGLISH_TENSES[f.Tense] : undefined;
      if (studied !== "en" || !tense) return null;
      const person = f.Person ? PERSONS[f.Person] : undefined;
      const number = f.Number ? NUMBERS[f.Number] : undefined;
      if (person && number) return { article: "la", name: `${person} ${number} du ${tense}` };
      return { article: "le", name: tense };
    }
    default:
      return null;
  }
}

/** "a", "a et b", "a, b et c". */
export function joinFrench(items: readonly string[]): string {
  if (items.length <= 1) return items.join("");
  return `${items.slice(0, -1).join(", ")} et ${items.at(-1)}`;
}

/** The names of a form's readings, each once, in the pack's order. */
function namesOf(readings: readonly GrammarTag[], studied: StudiedLanguageCode): Named[] {
  const out: Named[] = [];
  for (const tag of readings) {
    const named = readingName(tag, studied);
    if (named && !out.some((n) => n.name === named.name)) out.push(named);
  }
  return out;
}

/** "de go", "d'eat": French elides before a vowel. */
function of(word: string): GrammarLine {
  return [/^[aeiou]/i.test(word) ? "d’" : "de ", { word }];
}

function withArticles(named: readonly Named[]): string {
  return joinFrench(named.map((n) => `${n.article} ${n.name}`));
}

/**
 * The lines a card shows about the form it was opened on, or none:
 * - the pieces of a word the pre-pass split: « doesn't » = does + not;
 * - what the form is: « prétérit et participe passé de walk » — or, for a form spelled like its
 *   dictionary form, what it may also be: « peut aussi être le prétérit de put »;
 * - what else it may be: « peut aussi être le pluriel de leaf », one line per dictionary form.
 *
 * `surface` is the form the readings are of (the piece, for a split word); `written` the word as
 * it stands on the page.
 */
export function grammarLines(
  grammar: WordGrammar,
  headword: string,
  surface: string,
  written: string,
  studied: StudiedLanguageCode = "en",
): GrammarLine[] {
  const lines: GrammarLine[] = [];
  if (grammar.pieces.length > 1) {
    const pieces: GrammarLine = [`« ${written} » = `];
    grammar.pieces.forEach((piece, i) => {
      if (i > 0) pieces.push(" + ");
      pieces.push({ word: piece });
    });
    lines.push(pieces);
  }
  const own = namesOf(grammar.readings, studied);
  if (own.length > 0) {
    const same = surface.trim().toLowerCase() === headword.toLowerCase();
    lines.push(
      same
        ? [`peut aussi être ${withArticles(own)} `, ...of(headword)]
        : [`${joinFrench(own.map((n) => n.name))} `, ...of(headword)],
    );
  }
  for (const other of grammar.others) {
    const named = namesOf(other.readings, studied);
    if (named.length > 0) lines.push([`peut aussi être ${withArticles(named)} `, ...of(other.lemma)]);
  }
  return lines;
}

/** A line as plain text — what a test or an accessibility label reads. */
export function lineText(line: GrammarLine): string {
  return line.map((s) => (typeof s === "string" ? s : s.word)).join("");
}
