import type { LinguaPort } from "./port.ts";
import type { StudiedLanguage } from "./types.ts";

// The language pairs this bundle ships, the pair whose pack serves a studied language
// (package-lingua-packs-per-pair), and the language a surface reads in
// (add-lingua-studied-language-profile). The list is packs.json, handed over by build.mjs as
// `__LINGUA_PACKS__`. tool/packs.mjs reads the same list for the build, but it reads files with
// node:fs, so the bundle cannot import it: test/pairs.spec.ts holds the two equal.

/** The pairs this bundle ships, the default language's first. */
export const SHIPPED_PAIRS: readonly string[] = __LINGUA_PACKS__.split(",");

/** The studied side of a pair: "en-fr" → "en". */
function studiedOf(pair: string): string {
  return pair.split("-")[0];
}

/** The default pair's studied language: what every reader starts in. */
export const DEFAULT_LANGUAGE = studiedOf(SHIPPED_PAIRS[0]) as StudiedLanguage;

/** Where a pair's pack lives inside the package. */
export function packPath(pair: string): string {
  return `assets/packs/${pair}.lingua`;
}

/**
 * The pair whose pack serves `language`: the first listed pair that studies it, or null when
 * none does. The engine holds one pack per studied language, so the first is the only one it
 * could load.
 */
export function pairFor(language: string, pairs: readonly string[] = SHIPPED_PAIRS): string | null {
  return pairs.find((pair) => studiedOf(pair) === language) ?? null;
}

/**
 * The language a surface reads in: the first of the reader's studied languages that a shipped
 * pair studies, or the default pair's when none is. A build that does not ship a language the
 * reader studies keeps reading in what it ships, and leaves their profile as it is.
 */
export async function readingLanguage(
  port: Pick<LinguaPort, "studiedLanguages">,
  pairs: readonly string[] = SHIPPED_PAIRS,
): Promise<StudiedLanguage> {
  const shipped = (await port.studiedLanguages()).find((language) => pairFor(language, pairs) !== null);
  return shipped ?? (studiedOf(pairs[0]) as StudiedLanguage);
}
