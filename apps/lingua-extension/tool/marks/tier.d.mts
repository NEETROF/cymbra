/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */

/** A judged file's counts: correct and withheld, the marks shown, and all selections. */
export interface JudgedCounts {
  correct: number;
  shown: number;
  withheld: number;
  total: number;
}

/** The engine's verdicts of a `judged-<pair>.tsv` text, counted as the README reads D2. */
export declare function judgedCounts(tsv: string): JudgedCounts;

/** D2's first tier: at least 90 % of the shown marks correct, and at most 25 % of all selections withheld. */
export declare function firstTier(counts: JudgedCounts): boolean;
