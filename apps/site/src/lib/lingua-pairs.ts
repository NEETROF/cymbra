// The pairs Cymbra Lingua ships, as the site's Lingua page reads them (change:
// add-site-lingua-matrix-pages, D1 and D3).
//
// One source for the pairs: `src/data/lingua-coverage.json`, whose `glossed` keys are
// exactly the pairs of `apps/lingua-extension/packs.json` — `scripts/lingua-data/
// gloss_coverage.py --write` writes it from them, its `--check` holds the figures to the
// committed tables, `test_gloss_coverage.py` holds both, and `test/lingua-pairs.spec.ts`
// holds the keys to `packs.json` on the site's side. The extension's model catalogue
// (`apps/lingua-extension/model-manifest.json`) says, per pair, whether extended
// translation serves it: a route of one model is direct, a route of two goes through
// English, no route means not yet. Both files are read in Astro front matter and in tests
// only, never from a Vue island.
//
// The page inserts its text with `set:html` (D1: the French bytes do not move), language
// names and lists included, so what reaches it from these files is checked here and the
// build fails on anything else: a pair key is two language codes, a route is a list, every
// figure is a finite number and there is one per top.

import coverage from "../data/lingua-coverage.json";
import manifest from "../../../lingua-extension/model-manifest.json";
import { LANGS, type Lang } from "./i18n";

/** How extended translation serves a pair, from its route's length. */
export type Translation = "direct" | "through-english" | "none";

export interface LinguaPair {
  /** `<studied>-<native>`, as `packs.json` lists it. */
  pair: string;
  /** The language read, ISO 639 (two or three lowercase letters). */
  studied: string;
  /** The language the glosses are in, ISO 639 (two or three lowercase letters). */
  native: string;
  /** The glossed share of the commonest words, in percent, one per `ShippedPairs.tops`. */
  glossed: number[];
  translation: Translation;
}

/** `lingua-coverage.json`'s shape. */
export interface Coverage {
  tops: number[];
  glossed: Record<string, number[]>;
}

/** The catalogue's routes: pair → the model ids extended translation chains for it. */
export type Routes = Record<string, string[]>;

/** The shipped pairs and the sizes of the commonest-words lists their figures are measured on. */
export interface ShippedPairs {
  /** 5,000, 10,000, 20,000 today. */
  tops: number[];
  /** In `lingua-coverage.json`'s order, which is `packs.json`'s. */
  pairs: LinguaPair[];
}

/** A language code as `packs.json` writes one. */
const LANGUAGE_CODE = /^[a-z]{2,3}$/;

function refuse(message: string): never {
  throw new Error(`lingua-pairs.ts: ${message}`);
}

/**
 * The shipped pairs with their figures and their translation route, checked: the build
 * fails loudly on a key that is not `<studied>-<native>`, a figure that is not a finite
 * number, a pair with more or fewer figures than tops, or a route that is not a list.
 */
export function shippedPairs(data: Coverage = coverage, routes: Routes = manifest.routes): ShippedPairs {
  const { tops } = data;
  if (!Array.isArray(tops) || tops.length === 0 || !tops.every((top) => Number.isInteger(top) && top > 0)) {
    refuse(`lingua-coverage.json: "tops" must be positive whole numbers — got ${JSON.stringify(tops)}`);
  }
  const pairs = Object.entries(data.glossed).map(([pair, glossed]): LinguaPair => {
    const [studied, native, extra] = pair.split("-");
    if (!LANGUAGE_CODE.test(studied) || !LANGUAGE_CODE.test(native ?? "") || extra !== undefined) {
      refuse(`lingua-coverage.json: ${JSON.stringify(pair)} is not a <studied>-<native> pair of language codes`);
    }
    if (!Array.isArray(glossed) || glossed.length !== tops.length) {
      refuse(`lingua-coverage.json: "${pair}" needs one figure per top (${tops.length}) — got ${JSON.stringify(glossed)}`);
    }
    if (!glossed.every((share) => Number.isFinite(share))) {
      refuse(`lingua-coverage.json: "${pair}" has a figure that is not a finite number — got ${JSON.stringify(glossed)}`);
    }
    const route: unknown = Object.hasOwn(routes, pair) ? routes[pair] : [];
    if (!Array.isArray(route)) {
      refuse(`model-manifest.json: the route of "${pair}" is not a list of models — got ${JSON.stringify(route)}`);
    }
    const translation: Translation = route.length === 0 ? "none" : route.length === 1 ? "direct" : "through-english";
    return { pair, studied, native, glossed, translation };
  });
  return { tops, pairs };
}

/** The shipped pairs alone, in `packs.json`'s order (see `shippedPairs`). */
export function linguaPairs(data: Coverage = coverage, routes: Routes = manifest.routes): LinguaPair[] {
  return shippedPairs(data, routes).pairs;
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
 * Spanish nav, footer and not-found links, and the Spanish home's Lingua card
 * (`spanishHomeLinguaCard`, change: extend-site-spanish-locale).
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
