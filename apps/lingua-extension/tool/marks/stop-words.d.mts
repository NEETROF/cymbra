/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */

/** A native language's stop words, authored by category: each value is its words, space-separated. */
export type StopWordSet = Readonly<Record<string, string>>;

/** Case and accents removed: the form glossMark compares. */
export declare const fold: (s: string) => string;

export declare const FRENCH: StopWordSet;
export declare const ENGLISH: StopWordSet;
export declare const SPANISH: StopWordSet;

/** The sets by native language, as the catalogue names one (a model's `to`). */
export declare const STOP_WORDS: Readonly<Record<string, StopWordSet>>;

/** Every word of a set, by category, as authored. */
export declare function wordsOf(set: StopWordSet): { category: string; word: string }[];

/** The stop words of `native`, folded for the lookup; a language without a set is refused. */
export declare function stopWords(native: string): Set<string>;
