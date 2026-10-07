import type { LinguaPort } from "./port.ts";
import type { NativeLanguage, StudiedLanguage } from "./types.ts";

// The language pairs this bundle ships, the pair whose pack serves a studied language
// (package-lingua-packs-per-pair), and the language a surface reads in
// (add-lingua-studied-language-profile). The list is packs.json, handed over by build.mjs as
// `__LINGUA_PACKS__`. tool/packs.mjs reads the same list for the build, but it reads files with
// node:fs, so the bundle cannot import it: test/pairs.spec.ts holds the two equal.
//
// A reader is served the pairs of their native language alone (generalise-lingua-native-language
// D7): a pair is chosen by its studied and its native language, never by its studied language
// alone, and a pair glossed in another native language is never loaded, accepted from the sync or
// offered in Réglages.

/** The pairs this bundle ships, the default language's first. */
export const SHIPPED_PAIRS: readonly string[] = __LINGUA_PACKS__.split(",");

/** The native language of a reader whose profile names none: every installed reader's (M22). */
export const DEFAULT_NATIVE: NativeLanguage = "fr";

/** The studied side of a pair: "en-fr" → "en". The side a page's gate reads (translate/setting.ts). */
export function studiedOf(pair: string): string {
  return pair.split("-")[0];
}

/** The native side of a pair, the language its glosses are written in: "en-fr" → "fr". */
export function nativeOf(pair: string): string {
  return pair.split("-")[1] ?? "";
}

/** The listed pairs glossed in `native`, in listed order. */
export function pairsOf(native: string, pairs: readonly string[] = SHIPPED_PAIRS): string[] {
  return pairs.filter((pair) => nativeOf(pair) === native);
}

/** The default pair of `native`: the first listed pair glossed in it, or null when none is. Its pack
 *  is the one an engine for that native language starts with. */
export function defaultPair(native: string, pairs: readonly string[] = SHIPPED_PAIRS): string | null {
  return pairsOf(native, pairs)[0] ?? null;
}

/** The default pair's studied language: what every reader starts in, before the engine says more. */
export const DEFAULT_LANGUAGE = studiedOf(defaultPair(DEFAULT_NATIVE) ?? SHIPPED_PAIRS[0]) as StudiedLanguage;

/** Where a pair's pack lives inside the package. */
export function packPath(pair: string): string {
  return `assets/packs/${pair}.lingua`;
}

/**
 * The pair whose pack serves `language` to a reader of `native`: the first listed pair that studies
 * the one and is glossed in the other, or null when none is. The engine holds one pack per studied
 * language, so the first is the only one it could load.
 */
export function pairFor(language: string, native: string, pairs: readonly string[] = SHIPPED_PAIRS): string | null {
  return pairsOf(native, pairs).find((pair) => studiedOf(pair) === language) ?? null;
}

/** The studied language of `native`'s default pair, or the bundle's when no pair is glossed in it. */
function defaultLanguageOf(native: string, pairs: readonly string[]): StudiedLanguage {
  return studiedOf(defaultPair(native, pairs) ?? pairs[0]) as StudiedLanguage;
}

/**
 * The language a surface reads in: the first of the reader's studied languages that a pair of their
 * native language studies, or that native language's default pair's when none is. A build that does
 * not ship a language the reader studies keeps reading in what it ships, and leaves their profile as
 * it is.
 */
export async function readingLanguage(
  port: Pick<LinguaPort, "studiedLanguages" | "nativeLanguage">,
  pairs: readonly string[] = SHIPPED_PAIRS,
): Promise<StudiedLanguage> {
  const [native, studied] = await Promise.all([port.nativeLanguage(), port.studiedLanguages()]);
  const shipped = studied.find((language) => pairFor(language, native, pairs) !== null);
  return shipped ?? defaultLanguageOf(native, pairs);
}

/**
 * The languages a device accepts from the sync (add-lingua-language-sync-client): the reader's
 * studied languages that a pair of their native language studies, in the reader's order, or that
 * native language's default pair's alone when none is. `readingLanguage` is its first.
 */
export async function acceptedLanguages(
  port: Pick<LinguaPort, "studiedLanguages" | "nativeLanguage">,
  pairs: readonly string[] = SHIPPED_PAIRS,
): Promise<StudiedLanguage[]> {
  const [native, studied] = await Promise.all([port.nativeLanguage(), port.studiedLanguages()]);
  const shipped = studied.filter((language) => pairFor(language, native, pairs) !== null);
  return shipped.length > 0 ? shipped : [defaultLanguageOf(native, pairs)];
}
