/** Declared for the tests: `tool/` is plain ESM, outside the TypeScript project. */
import type { Locales } from "./locales.mjs";
export declare const DESCRIPTION_MESSAGE: "extensionDescription";
export declare const COMMAND_MESSAGES: Readonly<Record<string, string>>;
export declare const LOCALISED_MESSAGES: readonly string[];
export declare const LITERAL_NATIVE: "fr";
export declare function localised(natives: readonly string[]): boolean;
export declare function defaultLocale(natives: readonly string[]): string;
export declare function hostPattern(url: string): string;
export declare function firefoxManifest(base: Record<string, unknown>): Record<string, unknown>;
export declare function safariManifest(base: Record<string, unknown>): Record<string, unknown>;
export declare function buildManifest(options: {
  base: Record<string, unknown>;
  target: "chromium" | "firefox" | "safari";
  pairs: readonly string[];
  messages: Locales;
  version: string;
  grpcWebUrl: string;
  extKey: string;
  offscreen: boolean;
  allUrls?: boolean;
}): { manifest: Record<string, unknown>; locales: string[] };
