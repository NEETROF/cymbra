import { describe, expect, it } from "vitest";
import manifest from "../../lingua-extension/model-manifest.json";
import coverage from "../src/data/lingua-coverage.json";
import {
  LINGUA_TOPS,
  linguaAlternates,
  linguaHref,
  linguaLocalePaths,
  linguaPageLangs,
  linguaPairs,
  pairsByNative,
  type Coverage,
} from "../src/lib/lingua-pairs";

// The shipped pairs as the Lingua page reads them (change: add-site-lingua-matrix-pages,
// D1, D3): from the committed figures and the extension's model catalogue. The matrix
// below is the list changes 34 and 35 ship — es-en (Spanish for English speakers) and
// en-es (English for Spanish speakers) beside today's French-native pairs.

const matrix: Coverage = {
  tops: [5000, 10000, 20000],
  glossed: {
    "en-fr": [95.1, 90.1, 78.9],
    "es-fr": [87.6, 77.2, 63.7],
    "es-en": [90.0, 80.0, 70.0],
    "en-es": [96.0, 91.0, 80.0],
  },
};
const routes = manifest.routes;

describe("the shipped pairs", () => {
  it("come from lingua-coverage.json, with their figures and their route", () => {
    const pairs = linguaPairs();
    expect(pairs.map((p) => p.pair)).toEqual(Object.keys(coverage.glossed));
    expect(LINGUA_TOPS).toEqual([5000, 10000, 20000]);
    const enFr = pairs.find((p) => p.pair === "en-fr")!;
    expect(enFr).toMatchObject({ studied: "en", native: "fr", glossed: coverage.glossed["en-fr"] });
    // One model is a direct route; es-fr chains es-en then en-fr: through English.
    expect(enFr.translation).toBe("direct");
    expect(pairs.find((p) => p.pair === "es-fr")!.translation).toBe("through-english");
  });

  it("read the matrix's routes the same way, and a pair without a route as not served yet", () => {
    const served = linguaPairs(matrix, routes);
    expect(served.map((p) => [p.pair, p.translation])).toEqual([
      ["en-fr", "direct"],
      ["es-fr", "through-english"],
      ["es-en", "direct"],
      ["en-es", "direct"],
    ]);
    const { "en-es": _, ...without } = routes;
    expect(linguaPairs(matrix, without).find((p) => p.pair === "en-es")!.translation).toBe("none");
  });

  it("refuse a key that is not a <studied>-<native> pair", () => {
    expect(() => linguaPairs({ tops: [1], glossed: { en: [1] } }, {})).toThrow(/not a <studied>-<native> pair/);
    expect(() => linguaPairs({ tops: [1], glossed: { "en-fr-x": [1] } }, {})).toThrow(/not a <studied>-<native> pair/);
  });
});

describe("the readers' pairs first (D2)", () => {
  const pairs = linguaPairs(matrix, routes);
  const order = (lang: "fr" | "en" | "es") =>
    pairsByNative(lang, pairs).map((g) => [g.native, g.pairs.map((p) => p.pair)]);

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
    const today = linguaPairs();
    expect(pairsByNative("en", today)).toEqual([{ native: "fr", pairs: today }]);
    expect(pairsByNative("fr", today)).toEqual([{ native: "fr", pairs: today }]);
  });

  it("puts a native language the site does not speak after the site's languages", () => {
    const withItalian = linguaPairs({ tops: [1], glossed: { "en-it": [1], "en-fr": [1] } }, {});
    expect(pairsByNative("en", withItalian).map((g) => g.native)).toEqual(["fr", "it"]);
  });
});

describe("a page for a language a pair is glossed in (D3)", () => {
  const today = linguaPairs();
  const withEnEs = linguaPairs(matrix, routes);
  const withoutEnEs = linguaPairs({ tops: matrix.tops, glossed: { "en-fr": [1], "es-fr": [1], "es-en": [1] } }, routes);

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
