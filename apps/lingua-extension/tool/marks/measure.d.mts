/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */

import type { ModelCatalogue } from "../../src/translate/host/model-manifest.ts";
import type { Span } from "../../src/translate/markup.ts";

/** A selection of the corpus with its sentence: what measureSelections takes. */
export interface Selection {
  k: number;
  id: string;
  word: string;
  upos: string;
  sentence: string;
  selection: Span;
}

/** The pack tables the experiment reads: the studied language's form → lemma, the pair's lemma → gloss. */
export interface GlossTables {
  forms: Map<string, string>;
  gloss: Map<string, string>;
}

/** The experiment's mark: the pack's lemma and gloss of the word, and where the gloss was found. */
export interface GlossMark {
  lemma: string;
  gloss: string | null;
  marks: Span[];
}

/** One line of results-<pair>.jsonl. */
export interface ResultLine {
  k: number;
  id: string;
  word: string;
  upos: string;
  sentence: string;
  /**
   * The sentence trapped the engine twice: no translation, no mark. Optional because
   * results-en-fr.jsonl and results-es-fr.jsonl predate it (measured 2026-10-05, before the loop
   * answered a trap): absent reads as false.
   */
  trapped?: boolean;
  translation: string | null;
  marks: string[];
  shown: string | null;
  engineMarks: string[];
  /**
   * The fragment's own translation; null when it trapped twice, "" for an empty fragment, which is
   * not asked. Either way the engine's marks stand unreconciled, as relay.ts shows them.
   */
  alone: string | null;
  gloss: { lemma: string; text: string | null; marks: string[]; shown: string } | null;
}

/** The engine as tool/marks/engine.mjs builds it: one markup string in, its translation out. */
export type Translate = (markup: string) => string | Promise<string>;

export interface MeasureOptions {
  /** A factory: each call builds a fresh engine. */
  engine: () => Translate | Promise<Translate>;
  gloss?: ((word: string, translation: string) => GlossMark) | null;
  trap?: (error: unknown) => boolean;
  log?: (message: string) => void;
}

/**
 * The native language `pair`'s route translates into — its last model's `to`; a route that does not
 * end in the pair's native language is refused as a catalogue error.
 */
export declare function nativeOfRoute(catalogue: Pick<ModelCatalogue, "models" | "routes">, pair: string): string;

/**
 * The experiment's tables for `pair` under `dir`: the studied language's forms and the pair's glosses,
 * or null when the pair's gloss table is missing or holds no gloss.
 */
export declare function readGlossTables(dir: string, pair: string, studied: string): GlossTables | null;

/** The experiment's mark for `word` in `translation`, from the pack's gloss. */
export declare function glossMark(word: string, translation: string, tables: GlossTables, stop: Set<string>): GlossMark;

/** `sentence` with `marks` bracketed ⟦…⟧. */
export declare function bracketed(sentence: string, marks: Span[]): string;

/** Every selection measured through the engine factory: one result line per selection, in order. */
export declare function measureSelections(selections: Selection[], options: MeasureOptions): Promise<ResultLine[]>;
