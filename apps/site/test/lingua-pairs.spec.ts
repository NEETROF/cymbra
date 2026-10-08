import { describe, expect, it } from "vitest";
import manifest from "../../lingua-extension/model-manifest.json";
import packs from "../../lingua-extension/packs.json";
import coverage from "../src/data/lingua-coverage.json";
import {
  linguaAlternates,
  linguaHref,
  linguaLocalePaths,
  linguaPageLangs,
  linguaPairs,
  pairsByNative,
  shippedPairs,
  type Routes,
} from "../src/lib/lingua-pairs";
import { MATRIX, ROUTES, TODAY } from "./support/lingua";

// The shipped pairs as the Lingua page reads them (change: add-site-lingua-matrix-pages,
// D1, D3), on committed lists (`test/support/lingua.ts`): today's two French-native pairs,
// and the matrix changes 34 and 35 ship — es-en and en-es beside them. Only the first block
// reads the live files, for their structure: a pair added or a figure refreshed moves none
// of these tests.

describe("the real files", () => {
  it("list exactly the pairs packs.json ships: the Spanish page is built from these keys (D3)", () => {
    expect(Object.keys(coverage.glossed).sort()).toEqual([...packs.pairs].sort());
  });

  it("hold <studied>-<native> keys, finite figures, one per top", () => {
    expect(coverage.tops.length).toBeGreaterThan(0);
    for (const top of coverage.tops) expect(Number.isInteger(top) && top > 0, `top ${top}`).toBe(true);
    for (const [pair, figures] of Object.entries(coverage.glossed)) {
      expect(pair).toMatch(/^[a-z]{2,3}-[a-z]{2,3}$/);
      expect(figures, pair).toHaveLength(coverage.tops.length);
      for (const figure of figures) expect(Number.isFinite(figure), `${pair}: ${figure}`).toBe(true);
    }
    const routes: Record<string, unknown> = manifest.routes;
    for (const [pair, route] of Object.entries(routes)) {
      expect(Array.isArray(route) && route.every((model) => typeof model === "string"), `route of ${pair}`).toBe(true);
    }
  });

  it("read without an error, in their order", () => {
    const { tops, pairs } = shippedPairs();
    expect(tops).toEqual(coverage.tops);
    expect(pairs.map((p) => p.pair)).toEqual(Object.keys(coverage.glossed));
  });
});

describe("the shipped pairs", () => {
  it("come with their figures, their tops and their route", () => {
    const { tops, pairs } = shippedPairs(TODAY, ROUTES);
    expect(tops).toEqual([5000, 10000, 20000]);
    expect(pairs.map((p) => p.pair)).toEqual(["en-fr", "es-fr"]);
    const enFr = pairs[0];
    expect(enFr).toMatchObject({ studied: "en", native: "fr", glossed: TODAY.glossed["en-fr"] });
    // One model is a direct route; es-fr chains es-en then en-fr: through English.
    expect(enFr.translation).toBe("direct");
    expect(pairs[1].translation).toBe("through-english");
  });

  it("read the matrix's routes the same way, and a pair without a route as not served yet", () => {
    expect(linguaPairs(MATRIX, ROUTES).map((p) => [p.pair, p.translation])).toEqual([
      ["en-fr", "direct"],
      ["es-fr", "through-english"],
      ["es-en", "direct"],
      ["en-es", "direct"],
    ]);
    const { "en-es": _, ...without } = ROUTES;
    expect(linguaPairs(MATRIX, without).find((p) => p.pair === "en-es")!.translation).toBe("none");
  });
});

describe("what reaches the page's set:html is checked at build time", () => {
  const tops = [5000, 10000, 20000];

  it("refuses a key that is not a <studied>-<native> pair of language codes", () => {
    for (const key of ["en", "en-fr-x", "EN-fr", "en-", "-fr", "en-fren", '<img src=x onerror="alert(1)">-fr', "en-<b>"]) {
      expect(() => linguaPairs({ tops, glossed: { [key]: [1, 1, 1] } }, {}), key).toThrow(/not a <studied>-<native> pair/);
    }
    // Three letters is an ISO 639 code too.
    expect(linguaPairs({ tops, glossed: { "ast-fr": [1, 1, 1] } }, {})[0]).toMatchObject({ studied: "ast", native: "fr" });
  });

  it("refuses a pair with more or fewer figures than tops", () => {
    expect(() => linguaPairs({ tops, glossed: { "en-fr": [95.1, 90.1] } }, {})).toThrow(/one figure per top \(3\)/);
    expect(() => linguaPairs({ tops, glossed: { "en-fr": [1, 2, 3, 4] } }, {})).toThrow(/one figure per top \(3\)/);
  });

  it("refuses a figure that is not a finite number", () => {
    for (const figure of [Number.NaN, Number.POSITIVE_INFINITY, "95" as unknown as number, null as unknown as number]) {
      expect(() => linguaPairs({ tops, glossed: { "en-fr": [figure, 1, 1] } }, {}), String(figure)).toThrow(/not a finite number/);
    }
  });

  it("refuses tops that are not positive whole numbers", () => {
    expect(() => linguaPairs({ tops: [], glossed: {} }, {})).toThrow(/"tops" must be positive whole numbers/);
    expect(() => linguaPairs({ tops: [5000, 1.5], glossed: {} }, {})).toThrow(/"tops" must be positive whole numbers/);
  });

  it("refuses a route that is not a list of models", () => {
    const routes = { "en-fr": "en-fr/base-memory/2.0" } as unknown as Routes;
    expect(() => linguaPairs({ tops, glossed: { "en-fr": [1, 1, 1] } }, routes)).toThrow(/route of "en-fr" is not a list/);
  });
});

describe("the readers' pairs first (D2)", () => {
  const pairs = linguaPairs(MATRIX, ROUTES);
  const order = (lang: "fr" | "en" | "es") => pairsByNative(lang, pairs).map((g) => [g.native, g.pairs.map((p) => p.pair)]);

  it("groups the pairs by the language they are glossed in, the readers' first, then the site's order", () => {
    expect(order("en")).toEqual([
      ["en", ["es-en"]],
      ["fr", ["en-fr", "es-fr"]],
      ["es", ["en-es"]],
    ]);
    expect(order("es")).toEqual([
      ["es", ["en-es"]],
      ["fr", ["en-fr", "es-fr"]],
      ["en", ["es-en"]],
    ]);
    expect(order("fr")).toEqual([
      ["fr", ["en-fr", "es-fr"]],
      ["en", ["es-en"]],
      ["es", ["en-es"]],
    ]);
  });

  it("presents today's French-native pairs as one group on every page", () => {
    const today = linguaPairs(TODAY, ROUTES);
    expect(pairsByNative("en", today)).toEqual([{ native: "fr", pairs: today }]);
    expect(pairsByNative("fr", today)).toEqual([{ native: "fr", pairs: today }]);
  });

  it("puts a native language the site does not speak after the site's languages", () => {
    const withItalian = linguaPairs({ tops: [1], glossed: { "en-it": [1], "en-fr": [1] } }, {});
    expect(pairsByNative("en", withItalian).map((g) => g.native)).toEqual(["fr", "it"]);
  });
});

describe("a page for a language a pair is glossed in (D3)", () => {
  const today = linguaPairs(TODAY, ROUTES);
  const withEnEs = linguaPairs(MATRIX, ROUTES);
  const withoutEnEs = linguaPairs({ tops: MATRIX.tops, glossed: { "en-fr": [1, 1, 1], "es-fr": [1, 1, 1], "es-en": [1, 1, 1] } }, ROUTES);

  it("builds no Spanish Lingua page while every pair is glossed in French or English", () => {
    expect(linguaPageLangs(today)).toEqual(["fr", "en"]);
    expect(linguaLocalePaths(today)).toEqual([]);
    expect(linguaPageLangs(withoutEnEs)).toEqual(["fr", "en"]);
    expect(linguaLocalePaths(withoutEnEs)).toEqual([]);
    // The Spanish nav, footer and not-found page send their readers to the English page.
    expect(linguaHref("es", today)).toBe("/en/lingua");
    expect(linguaAlternates(today)).toEqual({ fr: "/lingua", en: "/en/lingua" });
  });

  it("builds /es/lingua/ once en-es ships, links it from the Spanish nav and names it in hreflang", () => {
    expect(linguaPageLangs(withEnEs)).toEqual(["fr", "en", "es"]);
    expect(linguaLocalePaths(withEnEs)).toEqual([{ params: { locale: "es" } }]);
    expect(linguaHref("es", withEnEs)).toBe("/es/lingua");
    expect(linguaAlternates(withEnEs)).toEqual({ fr: "/lingua", en: "/en/lingua", es: "/es/lingua" });
  });

  it("keeps the French and English pages static whatever ships", () => {
    expect(linguaHref("fr", today)).toBe("/lingua");
    expect(linguaHref("en", today)).toBe("/en/lingua");
    expect(linguaHref("en", withEnEs)).toBe("/en/lingua");
  });

  it("gives no page to a native language the site does not speak", () => {
    const withItalian = linguaPairs({ tops: [1], glossed: { "en-fr": [1], "en-it": [1] } }, {});
    expect(linguaPageLangs(withItalian)).toEqual(["fr", "en"]);
    expect(linguaLocalePaths(withItalian)).toEqual([]);
  });
});
