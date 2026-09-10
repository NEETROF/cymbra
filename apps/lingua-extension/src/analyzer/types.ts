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

/** The user's status for a dictionary form. Mirrors the WASM `setStatus` vocabulary. */
export type LemmaStatus = "known" | "learning" | "ignored";

/** A token that should be painted (Learning or Unknown); Known/Ignored/ProperNoun are not. */
export function isPaintedClass(cls: TokenClass): cls is "Learning" | "Unknown" {
  return cls === "Learning" || cls === "Unknown";
}
