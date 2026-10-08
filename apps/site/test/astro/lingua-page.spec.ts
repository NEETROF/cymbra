import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { beforeAll, describe, expect, it, vi } from "vitest";
import type { Coverage, Routes } from "../../src/lib/lingua-pairs";
import takenWith from "../fixtures/lingua/taken-with.json";
import { MATRIX, ROUTES, SIX_PAIRS, TODAY } from "../support/lingua";

// The Lingua pages rendered as the build renders them, through Astro's Container API, on a
// committed pair list (change: add-site-lingua-matrix-pages, D1–D3). The test stands in for
// `src/data/lingua-coverage.json` and the extension's model catalogue, so the page, its
// `getStaticPaths()` and the layout's Spanish links all read the same list:
// - the fixtures' pairs (`test/fixtures/lingua/taken-with.json`, the shipped en-fr, es-fr and es-en):
//   `/lingua/` and `/en/lingua/` render the fixtures' `<main>`, byte for byte, whatever the
//   live data holds (`test/post-build/lingua.spec.ts` checks the built pages against the
//   same fixtures while the live pairs are the fixtures');
// - the French-native pairs alone (`test/support/lingua.ts`'s `TODAY`): no `/es/lingua/`, the
//   Spanish pages link the English one;
// - the matrix (es-en and en-es beside them): `/es/lingua/` exists, in Spanish, leads with
//   the Spanish-glossed pair, and the Spanish nav and footer link it — what the build will do
//   once change 35 ships en-es;
// - the six pairs (fr-en and fr-es beside the matrix, change 52): the coverage table has a row
//   per pair inside a box that scrolls on its own (change: add-lingua-french-listings, D8).

const coverage = vi.hoisted(() => ({ tops: [] as number[], glossed: {} as Record<string, number[]> }));
const manifest = vi.hoisted(() => ({ routes: {} as Record<string, string[]> }));
vi.mock("../../src/data/lingua-coverage.json", () => ({ default: coverage }));
vi.mock("../../../lingua-extension/model-manifest.json", () => ({ default: manifest }));
// The fixtures were taken with no community invite.
vi.mock("../../src/lib/config", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../src/lib/config")>();
  return { ...original, config: { ...original.config, discordUrl: null } };
});

import FrenchPage from "../../src/pages/lingua.astro";
import EnglishPage from "../../src/pages/en/lingua.astro";
import LocalePage, { getStaticPaths } from "../../src/pages/[locale]/lingua.astro";
import SpanishNotFound from "../../src/pages/es/404.astro";

/** The pairs every render below reads, as `lingua-coverage.json` and the catalogue would hold them. */
function ship(data: Coverage, routes: Routes): void {
  coverage.tops = data.tops;
  coverage.glossed = data.glossed;
  manifest.routes = routes;
}
const shipTheFixtures = () => ship(takenWith.coverage, takenWith.routes);
const shipToday = () => ship(TODAY, ROUTES);
const shipTheMatrix = () => ship(MATRIX, ROUTES);
const shipSixPairs = () => ship(SIX_PAIRS, ROUTES);

const fixture = (lang: string) => readFileSync(resolve(__dirname, `../fixtures/lingua/main.${lang}.html`), "utf8");
const main = (html: string) => html.match(/<main>(.*?)<\/main>/s)?.[1];
const navLingua = (html: string) => html.match(/<nav class="nav-links">.*?<a href="([^"]+)">Lingua<\/a>/s)?.[1];
const footerLingua = (html: string) => html.match(/<footer class="site-footer">.*?<a href="([^"]+)">Lingua<\/a>/s)?.[1];
const hreflangs = (html: string) => [...html.matchAll(/<link rel="alternate" hreflang="([a-z]+)" href="([^"]+)">/g)].map((m) => [m[1], m[2]]);
const headers = (html: string) => [...html.matchAll(/<th>([^<]*)<\/th>/g)].map((m) => m[1]);
const rowHeaders = (html: string) => [...html.matchAll(/<th scope="row">([^<]*)<\/th>/g)].map((m) => m[1]);

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
    shipTheFixtures();
    expect(main(await render(FrenchPage))).toBe(fixture("fr"));
  });

  it("/en/lingua/ renders the fixture's <main>, byte for byte", async () => {
    shipTheFixtures();
    expect(main(await render(EnglishPage))).toBe(fixture("en"));
  });

  it("builds no /es/lingua/, and the Spanish pages link the English one", async () => {
    shipToday();
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
    shipTheMatrix();
    expect(getStaticPaths()).toEqual([{ params: { locale: "es" } }]);
  });

  it("/es/lingua/ is Spanish, leads with the Spanish-glossed pair and is linked from the Spanish nav and footer", async () => {
    shipTheMatrix();
    const html = await render(localePage, { locale: "es" });
    expect(html).toContain('<html lang="es">');
    expect(html).toContain("<title>Cymbra Lingua — amplía tu vocabulario leyendo la web</title>");
    // Two native languages listed: one row per pair, its header the pair (D8).
    expect(headers(html)).toEqual(["Palabras más frecuentes", "5000", "10\u202F000", "20\u202F000"]);
    expect(rowHeaders(html)).toEqual(["Inglés → español", "Inglés → francés", "Español → francés", "Español → inglés"]);
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
    shipTheMatrix();
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
    shipTheMatrix();
    const html = await render(EnglishPage);
    expect(rowHeaders(html)[0]).toBe("Spanish → English");
    expect(main(html)).toContain("Made for English speakers learning Spanish.");
  });
});

describe("the six pairs: a row per pair, in a box that scrolls on its own (change: add-lingua-french-listings, D8)", () => {
  it("/en/lingua/ has six rows, Spanish → English and French → English first, inside the box", async () => {
    shipSixPairs();
    const html = await render(EnglishPage);
    expect(rowHeaders(html)).toEqual([
      "Spanish → English",
      "French → English",
      "English → French",
      "Spanish → French",
      "English → Spanish",
      "French → Spanish",
    ]);
    expect(headers(html)).toEqual(["Commonest words", "5,000", "10,000", "20,000"]);
    expect(main(html)).toContain(
      '<div class="table-scroll"><table><thead><tr><th>Commonest words</th><th>5,000</th><th>10,000</th><th>20,000</th></tr></thead><tbody><tr><th scope="row">Spanish → English</th><td>90%</td><td>80%</td><td>70%</td></tr>',
    );
    expect(main(html)).toContain("In Spanish and French, the card also names the tense and the gender.");
    expect(main(html)).toContain("For Spanish and French, the levels are estimated from word frequency");
  });

  it("/es/lingua/ leads with the Spanish speakers' pairs, French among them", async () => {
    shipSixPairs();
    const html = await render(localePage, { locale: "es" });
    expect(rowHeaders(html).slice(0, 2)).toEqual(["Inglés → español", "Francés → español"]);
    expect(main(html)).toContain("En francés y en español, la tarjeta también indica el tiempo verbal y el género.");
  });

  it("the French-native pairs alone: no box, no row header — the table before change 34", async () => {
    shipToday();
    for (const page of [FrenchPage, EnglishPage]) {
      const html = await render(page);
      expect(html).not.toContain("table-scroll");
      expect(rowHeaders(html)).toEqual([]);
    }
  });

  it("es-en shipping (change 34): two native languages listed, a row per pair in the box — the fixtures' table", async () => {
    shipTheFixtures();
    const french = await render(FrenchPage);
    expect(main(french)).toContain('<div class="table-scroll">');
    expect(rowHeaders(french)).toEqual(["Anglais → français", "Espagnol → français", "Espagnol → anglais"]);
    const english = await render(EnglishPage);
    expect(main(english)).toContain('<div class="table-scroll">');
    expect(rowHeaders(english)).toEqual(["Spanish → English", "English → French", "Spanish → French"]);
  });
});
