/** Declared for the test: `tool/` is plain ESM, outside the TypeScript project. */
import type { Locales } from "./locales.mjs";
export declare function versionProblem(version: string): string | null;
export declare function manifestProblems(manifest: Record<string, unknown>): string[];
export declare function localeProblems(manifest: Record<string, unknown>, locales: Locales): string[];
