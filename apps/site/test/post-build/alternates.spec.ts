import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { linguaAlternates } from "../../src/lib/lingua-pairs";
import { outputFileFor } from "../../src/lib/pinned-routes";

// Run after `yarn build` (see `vitest.build.config.ts`): a page names its translations
// through `alternates` (change: add-site-spanish-locale), and the layout turns them into
// the header's language switch (every other language the page exists in) and one
// `hreflang` per alternate. Asserted on the built HTML: the layouts are `.astro`, which
// only the build renders.

const dist = resolve(__dirname, "../../dist");
const site = "https://cymbra.app";

/** The `hreflang` links and the switch entries of a built page, in document order. */
function alternatesOf(path: string): { hreflang: [string, string][]; switchTo: [string, string][] } {
  const html = readFileSync(resolve(dist, outputFileFor(path)), "utf8");
  const hreflang = [...html.matchAll(/<link rel="alternate" hreflang="([a-z]+)" href="([^"]+)">/g)].map(
    (m) => [m[1], m[2]] as [string, string],
  );
  const nav = html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] ?? "";
  const switchTo = [...nav.matchAll(/<a href="([^"]+)">(FR|EN|ES)<\/a>/g)].map((m) => [m[2], m[1]] as [string, string]);
  return { hreflang, switchTo };
}

describe("each page names its translations", () => {
  // Each legal page in each language: its own `hreflang` and its twins', and a switch
  // to every twin — not to the home page, as before the fix.
  const legal: Record<string, Record<string, string>> = {
    privacy: { fr: "/confidentialite", en: "/en/privacy", es: "/es/privacidad" },
    terms: { fr: "/cgu", en: "/en/terms", es: "/es/terminos" },
    support: { fr: "/support", en: "/en/support", es: "/es/soporte" },
    deletion: { fr: "/suppression-compte", en: "/en/delete-account", es: "/es/eliminar-cuenta" },
  };

  for (const [page, byLang] of Object.entries(legal)) {
    it(`${page}: the switch and hreflang of each language open the same page`, () => {
      const expectedHreflang = Object.entries(byLang).map(([l, href]) => [l, `${site}${href}`]);
      for (const [lang, path] of Object.entries(byLang)) {
        const { hreflang, switchTo } = alternatesOf(path);
        expect(hreflang, `${path} hreflang`).toEqual(expectedHreflang);
        const others = Object.entries(byLang)
          .filter(([l]) => l !== lang)
          .map(([l, href]) => [l.toUpperCase(), href]);
        expect(switchTo, `${path} switch`).toEqual(others);
      }
    });
  }

  it("the Lingua page names the languages a shipped pair is glossed in (change: add-site-lingua-matrix-pages)", () => {
    const expected = Object.entries(linguaAlternates()).map(([l, href]) => [l, `${site}${href}`]);
    for (const [lang, path] of Object.entries(linguaAlternates())) {
      const { hreflang, switchTo } = alternatesOf(path);
      expect(hreflang, `${path} hreflang`).toEqual(expected);
      const others = Object.entries(linguaAlternates())
        .filter(([l]) => l !== lang)
        .map(([l, href]) => [l.toUpperCase(), href]);
      expect(switchTo, `${path} switch`).toEqual(others);
    }
  });

  it("a page without a Spanish twin offers French and English alone", () => {
    expect(alternatesOf("/").hreflang).toEqual([
      ["fr", `${site}/`],
      ["en", `${site}/en/`],
    ]);
    expect(alternatesOf("/").switchTo).toEqual([["EN", "/en/"]]);
    expect(alternatesOf("/en/music").switchTo).toEqual([["FR", "/music"]]);
  });
});
