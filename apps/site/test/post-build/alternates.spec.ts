import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import coverage from "../../src/data/lingua-coverage.json";
import { outputFileFor } from "../../src/lib/pinned-routes";

// Run after `yarn build` (see `vitest.build.config.ts`): a page names its translations
// through `alternates` (change: add-site-spanish-locale), and the layout turns them into
// the header's language switch (every other language the page exists in) and one
// `hreflang` per alternate. Asserted on the built HTML: the layouts are `.astro`, which
// only the build renders. The home and Music pages exist in Spanish too, the account,
// code and checkout pages do not (change: extend-site-spanish-locale, D4, D5).

const dist = resolve(__dirname, "../../dist");
const site = "https://cymbra.app";

const builtPage = (path: string) => readFileSync(resolve(dist, outputFileFor(path)), "utf8");

/** The `hreflang` links and the switch entries of a built page, in document order. */
function alternatesOf(path: string): { hreflang: [string, string][]; switchTo: [string, string][] } {
  const html = builtPage(path);
  const hreflang = [...html.matchAll(/<link rel="alternate" hreflang="([a-z]+)" href="([^"]+)">/g)].map(
    (m) => [m[1], m[2]] as [string, string],
  );
  const nav = html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] ?? "";
  const switchTo = [...nav.matchAll(/<a href="([^"]+)">(FR|EN|ES)<\/a>/g)].map((m) => [m[2], m[1]] as [string, string]);
  return { hreflang, switchTo };
}

describe("each page names its translations", () => {
  // Each legal page in each language — and the home and Music pages (change:
  // extend-site-spanish-locale) — its own `hreflang` and its twins', and a switch to every
  // twin — not to the home page, as before the fix.
  const twins: Record<string, Record<string, string>> = {
    privacy: { fr: "/confidentialite", en: "/en/privacy", es: "/es/privacidad" },
    terms: { fr: "/cgu", en: "/en/terms", es: "/es/terminos" },
    support: { fr: "/support", en: "/en/support", es: "/es/soporte" },
    deletion: { fr: "/suppression-compte", en: "/en/delete-account", es: "/es/eliminar-cuenta" },
    home: { fr: "/", en: "/en/", es: "/es/" },
    music: { fr: "/music", en: "/en/music", es: "/es/music" },
  };

  for (const [page, byLang] of Object.entries(twins)) {
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
    // French and English always; Spanish once a key of the file the build reads,
    // `<studied>-<native>`, is glossed in Spanish — read here, not asked of the site's helpers.
    const lingua: Record<string, string> = { fr: "/lingua", en: "/en/lingua" };
    if (Object.keys(coverage.glossed).some((pair) => pair.split("-")[1] === "es")) lingua.es = "/es/lingua";
    const expected = Object.entries(lingua).map(([l, href]) => [l, `${site}${href}`]);
    for (const [lang, path] of Object.entries(lingua)) {
      const { hreflang, switchTo } = alternatesOf(path);
      expect(hreflang, `${path} hreflang`).toEqual(expected);
      const others = Object.entries(lingua)
        .filter(([l]) => l !== lang)
        .map(([l, href]) => [l.toUpperCase(), href]);
      expect(switchTo, `${path} switch`).toEqual(others);
    }
  });

  it("a page without a Spanish twin offers French and English alone", () => {
    // The account, code redemption and checkout pages (D4).
    expect(alternatesOf("/account").hreflang).toEqual([
      ["fr", `${site}/account`],
      ["en", `${site}/en/account`],
    ]);
    expect(alternatesOf("/account").switchTo).toEqual([["EN", "/en/account"]]);
    expect(alternatesOf("/en/redeem").switchTo).toEqual([["FR", "/redeem"]]);
    expect(alternatesOf("/checkout").hreflang.map(([lang]) => lang)).toEqual(["fr", "en"]);
    expect(alternatesOf("/checkout").switchTo).toEqual([["EN", "/en/checkout"]]);
  });
});

describe("a Spanish page's links (change: extend-site-spanish-locale, D4, D5)", () => {
  /** The `href` of the link labelled `label` in the header nav or the footer of a built page. */
  function linkOf(path: string, where: "nav" | "footer", label: string): string | undefined {
    const html = builtPage(path);
    const block =
      where === "nav" ? html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] : html.match(/<footer class="site-footer">(.*?)<\/footer>/s)?.[1];
    return block?.match(new RegExp(`<a href="([^"]+)">${label}</a>`))?.[1];
  }
  const SPANISH = ["/es/", "/es/music", "/es/privacidad", "/es/terminos", "/es/soporte", "/es/eliminar-cuenta"];

  for (const path of SPANISH) {
    it(`${path}: the brand and « Music » open the Spanish pages, « Cuenta » and « Código de acceso » the English ones`, () => {
      expect(builtPage(path)).toContain('<a class="brand" href="/es/">Cymbra</a>');
      expect(linkOf(path, "nav", "Music")).toBe("/es/music");
      expect(linkOf(path, "footer", "Music")).toBe("/es/music");
      expect(linkOf(path, "nav", "Cuenta")).toBe("/en/account");
      expect(linkOf(path, "footer", "Cuenta")).toBe("/en/account");
      expect(linkOf(path, "footer", "Código de acceso")).toBe("/en/redeem");
    });
  }

  it("the Spanish not-found page's « Inicio » and « Cymbra Music » open the Spanish pages", () => {
    const html = readFileSync(resolve(dist, "es/404.html"), "utf8");
    expect(html).toContain('<a class="brand" href="/es/">Cymbra</a>');
    expect(html).toContain('<a class="btn btn-primary" href="/es/">Inicio</a>');
    expect(html).toContain('<a class="btn btn-ghost" href="/es/music">Cymbra Music</a>');
  });
});
