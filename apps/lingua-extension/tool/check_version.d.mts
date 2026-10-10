/** Declared for the test: `tool/` is plain ESM, outside the TypeScript project. */
import type { Locales } from "./locales.mjs";
export declare function versionProblem(version: string): string | null;
export declare function manifestProblems(manifest: Record<string, unknown>): string[];
export declare function localeProblems(manifest: Record<string, unknown>, locales: Locales): string[];
/** Each native language's names for the languages a pair may study, as the catalogue writes them. */
export declare const LANGUAGE_NAMES: Readonly<Record<string, Readonly<Record<string, string>>>>;
export declare function namedLanguages(native: string, text: string): string[];
export declare function summaryProblems(
  manifest: Record<string, unknown>,
  locales: Locales,
  pairs: readonly string[],
): string[];
