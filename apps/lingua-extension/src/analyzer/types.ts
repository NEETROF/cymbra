// The analysis result shape, mirroring lingua-core's `PageAnalysis`
// (crates/lingua-core/src/engine.rs). These are the exact JSON keys the WASM
// `analyse()` emits — plain serde, no renaming — so the field names and the
// `class` enum strings are verbatim. Keep this in lockstep with the Rust source.

/** How a token counts and highlights. Five variants — `Ignored` counts as known. */
export type TokenClass = "Known" | "Ignored" | "Learning" | "Unknown" | "ProperNounOutOfLexicon";

/** One analysed token, positioned by byte offsets into its source block. */
export interface AnalyzedToken {
  /** Index into the blocks array passed to `analyse()`. */
  block: number;
  /** Byte start of the word in that block (UTF-8, not UTF-16). */
  start: number;
  /** Byte end (exclusive) of the word in that block. */
  end: number;
  /** Surface text, case preserved. */
  surface: string;
  /** The dictionary form. */
  lemma: string;
  /** Classification. */
  class: TokenClass;
  /** Native-language gloss, present only for not-yet-known words that the pack glosses. */
  gloss: string | null;
}

/** The analysis of one page (a batch of blocks). */
export interface PageAnalysis {
  /** Analyser generation that produced this (parity/comparability key). */
  analyzer_version: string;
  /** Whether the page had enough studied-language content to analyse. */
  analysable: boolean;
  /** Tokens in document order (empty when not analysable). */
  tokens: AnalyzedToken[];
  /** Occurrences that entered the percentage (proper nouns excluded). */
  counted: number;
  /** Occurrences counting as known (Known + Ignored). */
  known: number;
  /** Known-token percentage 0–100, or null when nothing was counted. */
  percent: number | null;
}

/** One part of a hyphenated compound the lexicon does not list, described like a token. */
export interface PhrasePart {
  /** The part's dictionary form. */
  lemma: string;
  /** Classification, on its own. */
  class: TokenClass;
  /** Native-language gloss whatever the class, or null when the pack has none. */
  gloss: string | null;
  /** Whether the dictionary form is a closed-class word of the studied language. */
  function_word: boolean;
}

/** One token of a glossed selection — every class carries its gloss, unlike `AnalyzedToken`. */
export interface PhraseToken {
  /** Surface text, case preserved. */
  surface: string;
  /** The dictionary form. */
  lemma: string;
  /** Classification. */
  class: TokenClass;
  /** Native-language gloss whatever the class, or null when the pack has none. */
  gloss: string | null;
  /** Whether the dictionary form is a closed-class word of the studied language. */
  function_word: boolean;
  /** The parts of a hyphenated compound the lexicon does not list; absent otherwise. */
  parts?: PhrasePart[];
}

/** The gloss of a short text — a selection — read without the page gates (`phraseGloss`). */
/** An expression of the pack found in a selection: the tokens it covers, and its own entry. */
export interface PhraseMatch {
  /** Index of the first token it covers. */
  start: number;
  /** Index after the last token it covers. */
  end: number;
  /** Its key: the covered tokens' dictionary forms, joined by single spaces. */
  key: string;
  /** Classification of the key itself — an expression is a dictionary form of its own. */
  class: TokenClass;
  /** Its native-language gloss. */
  gloss: string;
}

export interface PhraseGloss {
  /** Tokens in reading order. */
  tokens: PhraseToken[];
  /** The pack's expressions found in the text, in token order; absent when none. */
  expressions?: PhraseMatch[];
}

/** The user's status for a dictionary form. Mirrors the WASM `setStatus` vocabulary. */
export type LemmaStatus = "known" | "learning" | "ignored";

/** A token that should be painted (Learning or Unknown); Known/Ignored/ProperNoun are not. */
export function isPaintedClass(cls: TokenClass): cls is "Learning" | "Unknown" {
  return cls === "Learning" || cls === "Unknown";
}

/** A CEFR level, ordered A1 < … < C2 (add-lingua-cefr-levels). */
export type CefrLevel = "A1" | "A2" | "B1" | "B2" | "C1" | "C2";

/** The six levels in ascending order — the ladder's rows / the picker's options. */
export const CEFR_LEVELS: readonly CefrLevel[] = ["A1", "A2", "B1", "B2", "C1", "C2"];

/** One row of the CEFR progression ladder, from `levelLadder()`. */
export interface LevelRow {
  level: CefrLevel;
  /** Explicitly known (proven; any provenance but the implicit calibration). */
  confirmed: number;
  /** Presumed known below the declared level (implicit, unproven). */
  presumed: number;
  /** Learning or new — not yet known. */
  toLearn: number;
  /** Band size (confirmed + presumed + toLearn). */
  total: number;
}

/** The reader's estimated vocabulary size, from `vocabularyEstimate()`. */
export interface VocabularyEstimate {
  /** Estimated known words among `universe` (each frequency band's known share, extrapolated). */
  estimated: number;
  /** Words explicitly known: marked, validated in review or confirmed by reading. */
  confirmed: number;
  /** The pack's dictionary words the estimate is taken over. */
  universe: number;
  /**
   * What it rests on: a declared level or the frequency slider (extrapolated), or only
   * the words marked known (then `estimated` is their exact count).
   */
  basis: "level" | "frequency" | "marked";
}

/** Order in which level-targeted seeding takes a level's lemmas. */
export type SeedOrder = "common" | "rare";
