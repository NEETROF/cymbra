/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
export declare const DAILY_KEY: "cymbra-lingua-daily-v4";
export declare const DAYS: number;
export declare function recentDays(options: {
  nowMs: number;
  languages: readonly [string, string];
  native: string;
}): Record<
  number,
  Record<string, { exposures: number; unknownSeen: number; wordsLearned: number; reviews: number; native: string }>
>;
