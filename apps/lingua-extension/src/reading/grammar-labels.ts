import type { GrammarTag, WordGrammar } from "../analyzer/types.ts";

// The words a card uses for a form's grammar (add-lingua-word-grammar, design D7). The pack
// speaks Universal Dependencies codes; the reader never sees one. A code these tables cannot
// name gives no text at all, so a pack from a later vocabulary can only ever show less —
// never a code.
//
// The names of parts of speech, gender, number and person are the interface's own. A tense's
// name depends on the studied language — `Tense=Past` is the prétérit in English and the passé
// simple in Spanish — so the verb-form names are keyed by studied language: English's, and
// Spanish's as French schools name them (add-lingua-spanish-word-card).

/** The studied languages whose verb forms this interface can name. */
export type StudiedLanguageCode = "en" | "es";

/** A name, with the article French gives it ("le prétérit", "la forme en -ing", "l’infinitif"). */
export interface Named {
  article: "le" | "la" | "l’" | "les";
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
const ORDINALS: Record<string, string> = { "1": "1re", "2": "2e", "3": "3e" };
const NUMBERS: Record<string, string> = { Sing: "du singulier", Plur: "du pluriel" };

/** English finite tenses, named as French schools name them. */
const ENGLISH_TENSES: Record<string, string> = { Past: "prétérit", Pres: "présent" };

/**
 * Spanish moods and tenses, named as French schools name them, keyed `Mood/Tense`. The present
 * and the imperfect name their mood: the subjunctive has both too.
 */
const SPANISH_TENSES: Record<string, string> = {
  "Ind/Pres": "présent de l’indicatif",
  "Ind/Imp": "imparfait de l’indicatif",
  "Ind/Past": "passé simple",
  "Ind/Fut": "futur",
  "Cnd/": "conditionnel",
  "Sub/Pres": "présent du subjonctif",
  "Sub/Imp": "imparfait du subjonctif",
  "Sub/Fut": "futur du subjonctif",
  "Imp/": "impératif",
};

/** French elides before a vowel, accented or not — never before « h » or « y » (design D3). */
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

/** A finite form's tense, as `studied` names it; undefined when it cannot. */
function tenseName(f: Record<string, string>, studied: StudiedLanguageCode): string | undefined {
  if (studied === "es")
    return SPANISH_TENSES[`${f.Mood ?? ""}/${f.Mood === "Cnd" || f.Mood === "Imp" ? "" : (f.Tense ?? "")}`];
  if (f.Mood !== undefined && f.Mood !== "Ind") return undefined;
  return f.Tense ? ENGLISH_TENSES[f.Tense] : undefined;
}

/** « féminin pluriel », « masculin singulier », « pluriel »: a nominal form's agreement. */
function agreement(f: Record<string, string>): string | undefined {
  const gender = f.Gender ? GENDERS[f.Gender] : undefined;
  const number = f.Number === "Plur" ? "pluriel" : f.Number === "Sing" ? "singulier" : undefined;
  if (gender) return number ? `${gender} ${number}` : gender;
  return f.Number === "Plur" ? "pluriel" : undefined;
}

const NOMINAL = new Set(["NOUN", "PROPN", "ADJ", "DET", "PRON"]);

/**
 * Whether a reading only says what the dictionary form is — its infinitive, a noun's singular, an
 * adjective's masculine singular — and so gives no line on that form's own card (design D4).
 */
function isDictionaryForm(tag: GrammarTag): boolean {
  const f = tag.features ?? {};
  if (f.VerbForm === "Inf") return true;
  if (!NOMINAL.has(tag.pos) || f.Number !== "Sing" || f.Degree) return false;
  return tag.pos === "NOUN" || tag.pos === "PROPN" || f.Gender !== "Fem";
}

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
  if (NOMINAL.has(tag.pos)) {
    const name = agreement(f);
    // Without a gender, only a noun's plural is named, as it always was.
    if (!name || (!f.Gender && tag.pos !== "NOUN" && tag.pos !== "PROPN" && tag.pos !== "ADJ")) return null;
    return { article: "le", name };
  }
  if (tag.pos !== "VERB" && tag.pos !== "AUX") return null;
  switch (f.VerbForm) {
    case "Inf":
      return studied === "es" ? masculine("infinitif") : null;
    case "Part": {
      if (f.Tense !== "Past") return null;
      const agreed = f.Gender === "Masc" && f.Number !== "Plur" ? undefined : agreement(f);
      return {
        article: "le",
        name: agreed ? `participe passé ${agreed.replace(" singulier", "")}` : "participe passé",
      };
    }
    case "Ger":
      return studied === "en" ? { article: "la", name: "forme en -ing" } : masculine("gérondif");
    case "Fin": {
      const tense = tenseName(f, studied);
      if (!tense) return null;
      const person = f.Person ? PERSONS[f.Person] : undefined;
      const number = f.Number ? NUMBERS[f.Number] : undefined;
      if (person && number) return { article: "la", name: `${person} ${number} ${ofName(tense)}` };
      return masculine(tense);
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

/**
 * The names of a form's readings, each once, in the pack's order. The persons of one tense and
 * one number are named once (« 1re et 3e personnes du singulier de l’imparfait de l’indicatif »,
 * design D2). On the dictionary form's own card (`same`), a reading that only says what that
 * form is gives no name (design D4).
 */
function namesOf(readings: readonly GrammarTag[], studied: StudiedLanguageCode, same = false): Named[] {
  const out: Named[] = [];
  const merged = new Map<string, { at: number; number: string; tense: string; persons: string[] }>();
  for (const tag of readings) {
    if (same && isDictionaryForm(tag)) continue;
    const f = tag.features ?? {};
    const tense = f.VerbForm === "Fin" ? tenseName(f, studied) : undefined;
    if (tense && f.Person && f.Number && PERSONS[f.Person] && NUMBERS[f.Number]) {
      const key = `${f.Number}|${tense}`;
      const group = merged.get(key);
      if (!group) {
        merged.set(key, { at: out.length, number: f.Number, tense, persons: [f.Person] });
        out.push({ article: "la", name: "" });
      } else if (!group.persons.includes(f.Person)) {
        group.persons.push(f.Person);
      }
      continue;
    }
    const named = readingName(tag, studied);
    if (named && !out.some((n) => n.name === named.name)) out.push(named);
  }
  // The tenses in the order French grammars give them — indicative, conditional, subjunctive,
  // imperative — at the places the pack's order gave the first of them.
  const groups = [...merged.values()];
  const places = groups.map((g) => g.at);
  groups.sort((a, b) => tenseRank(a.tense) - tenseRank(b.tense) || a.number.localeCompare(b.number));
  groups.forEach(({ number, tense, persons }, i) => {
    const ordered = [...persons].sort();
    const who = ordered.length === 1 ? PERSONS[ordered[0]] : `${joinFrench(ordered.map((p) => ORDINALS[p]))} personnes`;
    out[places[i]] = {
      article: ordered.length === 1 ? "la" : "les",
      name: `${who} ${NUMBERS[number]} ${ofName(tense)}`,
    };
  });
  return out;
}

/** A tense's place in the order its table gives it; English's two come in the pack's order. */
function tenseRank(tense: string): number {
  const at = Object.values(SPANISH_TENSES).indexOf(tense);
  return at === -1 ? 0 : at;
}

/** "de go", "d’eat", "d’él": French elides before a vowel, accented or not (design D3). */
function of(word: string): GrammarLine {
  return [elides(word) ? "d’" : "de ", { word }];
}

function withArticles(named: readonly Named[]): string {
  return joinFrench(named.map((n) => (n.article === "l’" ? `l’${n.name}` : `${n.article} ${n.name}`)));
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
  const same = surface.trim().toLowerCase() === headword.toLowerCase();
  const own = namesOf(grammar.readings, studied, same);
  if (own.length > 0) {
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
