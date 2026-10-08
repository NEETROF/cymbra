import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { beforeAll, describe, expect, it, vi } from "vitest";
import type { Coverage } from "../../src/lib/lingua-pairs";
import { MATRIX, TODAY } from "../support/lingua";

// The Lingua pages rendered as the build renders them, through Astro's Container API, on a
// committed pair list (change: add-site-lingua-matrix-pages, D1–D3). The test stands in for
// `src/data/lingua-coverage.json` and the extension's model catalogue, so the page, its
// `getStaticPaths()` and the layout's Spanish links all read the same list:
// - today's pairs: `/lingua/` and `/en/lingua/` render the `<main>` of the fixtures, byte
//   for byte, whatever the live data holds (`test/post-build/lingua.spec.ts` checks the
//   built pages against the same fixtures while the live pairs are the fixtures');
// - the matrix (es-en and en-es beside them): `/es/lingua/` exists, in Spanish, leads with
//   the Spanish-glossed pair, and the Spanish nav and footer link it — what the build will do
//   once change 35 ships en-es.

const coverage = vi.hoisted(() => ({ tops: [] as number[], glossed: {} as Record<string, number[]> }));
vi.mock("../../src/data/lingua-coverage.json", () => ({ default: coverage }));
vi.mock("../../../lingua-extension/model-manifest.json", async () => {
  const { ROUTES } = await import("../support/lingua");
  return { default: { routes: ROUTES } };
});
// The fixtures were taken with no community invite.
vi.mock("../../src/lib/config", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../src/lib/config")>();
  return { ...original, config: { ...original.config, discordUrl: null } };
});

import FrenchPage from "../../src/pages/lingua.astro";
import EnglishPage from "../../src/pages/en/lingua.astro";
import LocalePage, { getStaticPaths } from "../../src/pages/[locale]/lingua.astro";
import SpanishNotFound from "../../src/pages/es/404.astro";

/** The pairs every render below reads, as `lingua-coverage.json` would hold them. */
function ship(data: Coverage): void {
  coverage.tops = data.tops;
  coverage.glossed = data.glossed;
}

const fixture = (lang: string) => readFileSync(resolve(__dirname, `../fixtures/lingua/main.${lang}.html`), "utf8");
const main = (html: string) => html.match(/<main>(.*?)<\/main>/s)?.[1];
const navLingua = (html: string) => html.match(/<nav class="nav-links">.*?<a href="([^"]+)">Lingua<\/a>/s)?.[1];
const footerLingua = (html: string) => html.match(/<footer class="site-footer">.*?<a href="([^"]+)">Lingua<\/a>/s)?.[1];
const hreflangs = (html: string) => [...html.matchAll(/<link rel="alternate" hreflang="([a-z]+)" href="([^"]+)">/g)].map((m) => [m[1], m[2]]);
const headers = (html: string) => [...html.matchAll(/<th>([^<]*)<\/th>/g)].map((m) => m[1]);

let container: AstroContainer;
beforeAll(async () => {
  container = await AstroContainer.create();
});
type Page = Parameters<AstroContainer["renderToString"]>[0];
const render = (page: Page, params?: Record<string, string>) => container.renderToString(page, { partial: false, params });
// A page with `getStaticPaths()` and no `props` is typed as taking `never`: it takes its
// `params` alone.
const localePage = LocalePage as unknown as Page;

describe("today's pairs", () => {
  it("/lingua/ renders the fixture's <main>, byte for byte", async () => {
    ship(TODAY);
    expect(main(await render(FrenchPage))).toBe(fixture("fr"));
  });

  it("/en/lingua/ renders the fixture's <main>, byte for byte", async () => {
    ship(TODAY);
    expect(main(await render(EnglishPage))).toBe(fixture("en"));
  });

  it("builds no /es/lingua/, and the Spanish pages link the English one", async () => {
    ship(TODAY);
    expect(getStaticPaths()).toEqual([]);
    const notFound = await render(SpanishNotFound);
    expect(navLingua(notFound)).toBe("/en/lingua");
    expect(footerLingua(notFound)).toBe("/en/lingua");
    expect(notFound).toContain('<a class="btn btn-ghost" href="/en/lingua">Cymbra Lingua</a>');
    expect(hreflangs(await render(FrenchPage))).toEqual([
      ["fr", "https://cymbra.app/lingua"],
      ["en", "https://cymbra.app/en/lingua"],
    ]);
  });
});

describe("the matrix: a pair glossed in Spanish ships", () => {
  it("builds /es/lingua/", () => {
    ship(MATRIX);
    expect(getStaticPaths()).toEqual([{ params: { locale: "es" } }]);
  });

  it("/es/lingua/ is Spanish, leads with the Spanish-glossed pair and is linked from the Spanish nav and footer", async () => {
    ship(MATRIX);
    const html = await render(localePage, { locale: "es" });
    expect(html).toContain('<html lang="es">');
    expect(html).toContain("<title>Cymbra Lingua — amplía tu vocabulario leyendo la web</title>");
    expect(headers(html)).toEqual(["Palabras más frecuentes", "Inglés → español", "Inglés → francés", "Español → francés", "Español → inglés"]);
    expect(main(html)).toContain("Pensada para hispanohablantes que aprenden inglés.");
    expect(navLingua(html)).toBe("/es/lingua");
    expect(footerLingua(html)).toBe("/es/lingua");
    expect(hreflangs(html)).toEqual([
      ["fr", "https://cymbra.app/lingua"],
      ["en", "https://cymbra.app/en/lingua"],
      ["es", "https://cymbra.app/es/lingua"],
    ]);
  });

  it("the French and English pages name it, and the Spanish not-found page links it", async () => {
    ship(MATRIX);
    for (const page of [FrenchPage, EnglishPage]) {
      expect(hreflangs(await render(page)).map(([lang]) => lang)).toEqual(["fr", "en", "es"]);
    }
    expect(await render(EnglishPage)).toContain('<a href="/es/lingua">ES</a>');
    const notFound = await render(SpanishNotFound);
    expect(navLingua(notFound)).toBe("/es/lingua");
    expect(footerLingua(notFound)).toBe("/es/lingua");
    expect(notFound).toContain('<a class="btn btn-ghost" href="/es/lingua">Cymbra Lingua</a>');
  });

  it("/en/lingua/ leads with the English-glossed pair", async () => {
    ship(MATRIX);
    const html = await render(EnglishPage);
    expect(headers(html)[1]).toBe("Spanish → English");
    expect(main(html)).toContain("Made for English speakers learning Spanish.");
  });
});
