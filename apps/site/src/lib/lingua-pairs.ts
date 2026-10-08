// The pairs Cymbra Lingua ships, as the site's Lingua page reads them (change:
// add-site-lingua-matrix-pages, D1 and D3).
//
// One source for the pairs: `src/data/lingua-coverage.json`, whose `glossed` keys are
// exactly the pairs of `apps/lingua-extension/packs.json` — `scripts/lingua-data/
// gloss_coverage.py --write` writes it from them, its `--check` holds the figures to the
// committed tables, and `test_gloss_coverage.py` holds both. The extension's model
// catalogue (`apps/lingua-extension/model-manifest.json`) says, per pair, whether extended
// translation serves it: a route of one model is direct, a route of two goes through
// English, no route means not yet. Both files are read in Astro front matter and in tests
// only, never from a Vue island.

import coverage from "../data/lingua-coverage.json";
import manifest from "../../../lingua-extension/model-manifest.json";
import { LANGS, type Lang } from "./i18n";

/** How extended translation serves a pair, from its route's length. */
export type Translation = "direct" | "through-english" | "none";

export interface LinguaPair {
  /** `<studied>-<native>`, as `packs.json` lists it. */
  pair: string;
  /** The language read, ISO 639-1. */
  studied: string;
  /** The language the glosses are in, ISO 639-1. */
  native: string;
  /** The glossed share of the `LINGUA_TOPS` commonest words, in percent, one per top. */
  glossed: number[];
  translation: Translation;
}

export interface Coverage {
  tops: number[];
  glossed: Record<string, number[]>;
}

/** The catalogue's routes: pair → the model ids extended translation chains for it. */
export type Routes = Record<string, string[]>;

/** The sizes of the commonest-words lists the coverage is measured on (5,000, 10,000, 20,000). */
export const LINGUA_TOPS: number[] = coverage.tops;

/** The shipped pairs, in `packs.json`'s order, with their figures and their translation route. */
export function linguaPairs(data: Coverage = coverage, routes: Routes = manifest.routes): LinguaPair[] {
  return Object.entries(data.glossed).map(([pair, glossed]) => {
    const [studied, native, extra] = pair.split("-");
    if (!studied || !native || extra !== undefined) {
      throw new Error(`lingua-coverage.json: "${pair}" is not a <studied>-<native> pair`);
    }
    const route = routes[pair] ?? [];
    const translation: Translation = route.length === 0 ? "none" : route.length === 1 ? "direct" : "through-english";
    return { pair, studied, native, glossed, translation };
  });
}

export interface NativeGroup {
  native: string;
  pairs: LinguaPair[];
}

/**
 * The pairs grouped by the language they are glossed in, the readers' (`lang`) first (D2),
 * then the other site languages in `LANGS`' order, then any other language in order of
 * appearance. Within a group, `packs.json`'s order.
 */
export function pairsByNative(lang: Lang, pairs: LinguaPair[]): NativeGroup[] {
  const natives: string[] = [];
  for (const p of pairs) if (!natives.includes(p.native)) natives.push(p.native);
  const rank = (native: string): number => {
    if (native === lang) return -1;
    const i = (LANGS as readonly string[]).indexOf(native);
    return i === -1 ? LANGS.length + natives.indexOf(native) : i;
  };
  natives.sort((a, b) => rank(a) - rank(b));
  return natives.map((native) => ({ native, pairs: pairs.filter((p) => p.native === native) }));
}

/**
 * The site languages a Lingua page exists in: French and English always (`/lingua/`,
 * `/en/lingua/` are static pages), and every other site language some shipped pair is
 * glossed in — `es` once en-es ships, not before (D3). A native language the site does not
 * speak gets no page.
 */
export function linguaPageLangs(pairs: LinguaPair[] = linguaPairs()): Lang[] {
  return LANGS.filter((l) => l === "fr" || l === "en" || pairs.some((p) => p.native === l));
}

/** The Lingua page's address in `lang`: French at the root, the others under their prefix. */
export function linguaPath(lang: Lang): string {
  return lang === "fr" ? "/lingua" : `/${lang}/lingua`;
}

/**
 * The `getStaticPaths()` of `src/pages/[locale]/lingua.astro`: one path per language of
 * `linguaPageLangs` that has no static page, `[]` while none has — a fixed-path page would
 * always be built.
 */
export function linguaLocalePaths(pairs: LinguaPair[] = linguaPairs()): { params: { locale: Lang } }[] {
  return linguaPageLangs(pairs)
    .filter((l) => l !== "fr" && l !== "en")
    .map((locale) => ({ params: { locale } }));
}

/**
 * Where a page in `lang` links Lingua: the Lingua page in that language when it exists,
 * else the English one — `Base.astro`'s rule for a page with no Spanish twin. Drives the
 * Spanish nav, footer and not-found links.
 */
export function linguaHref(lang: Lang, pairs: LinguaPair[] = linguaPairs()): string {
  return linguaPageLangs(pairs).includes(lang) ? linguaPath(lang) : linguaPath("en");
}

/** The Lingua page's `alternates` for `Base.astro`: its address in each language it exists in. */
export function linguaAlternates(pairs: LinguaPair[] = linguaPairs()): Partial<Record<Lang, string>> {
  const alternates: Partial<Record<Lang, string>> = {};
  for (const l of linguaPageLangs(pairs)) alternates[l] = linguaPath(l);
  return alternates;
}
