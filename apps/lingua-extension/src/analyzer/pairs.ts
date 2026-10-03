// The language pairs this bundle ships, and the pair whose pack serves a studied language
// (package-lingua-packs-per-pair). The list is packs.json, handed over by build.mjs as
// `__LINGUA_PACKS__`. tool/packs.mjs reads the same list for the build, but it reads files with
// node:fs, so the bundle cannot import it: test/pairs.spec.ts holds the two equal.

/** The pairs this bundle ships, the default language's first. */
export const SHIPPED_PAIRS: readonly string[] = __LINGUA_PACKS__.split(",");

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
  return pairs.find((pair) => pair.split("-")[0] === language) ?? null;
}
