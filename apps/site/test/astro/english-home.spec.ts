import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { beforeAll, describe, expect, it, vi } from "vitest";
import type { Coverage, Routes } from "../../src/lib/lingua-pairs";
import { MATRIX, ROUTES, TODAY, WITH_FR_EN } from "../support/lingua";

// The English home page rendered as the build renders it, through Astro's Container API
// (change: add-lingua-french-listings, D9). As `spanish-pages.spec.ts` does, the test stands in
// for `src/data/lingua-coverage.json` and the extension's model catalogue, so the home's Lingua
// card reads the pair list each case ships:
// - today's pairs (en-fr, es-fr): the card is today's, byte for byte — written for French
//   speakers, « Read the English web… »;
// - es-en (change 34): Spanish, read with an English gloss;
// - es-en and fr-en (change 52): Spanish or French.
// Whatever the pairs, its button opens `/en/lingua`.

const coverage = vi.hoisted(() => ({ tops: [] as number[], glossed: {} as Record<string, number[]> }));
const manifest = vi.hoisted(() => ({ routes: {} as Record<string, string[]> }));
vi.mock("../../src/data/lingua-coverage.json", () => ({ default: coverage }));
vi.mock("../../../lingua-extension/model-manifest.json", () => ({ default: manifest }));
// The community band is rendered only with an invite: none here.
vi.mock("../../src/lib/config", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../src/lib/config")>();
  return { ...original, config: { ...original.config, discordUrl: null } };
});

import EnglishHome from "../../src/pages/en/index.astro";

function ship(data: Coverage, routes: Routes): void {
  coverage.tops = data.tops;
  coverage.glossed = data.glossed;
  manifest.routes = routes;
}

/** The Lingua card of the home page, as the build writes it. */
const linguaArticle = (html: string) =>
  html.match(/<article class="product"><p class="kicker">Cymbra Lingua · .*?<\/article>/s)?.[0];
const linguaCard = (html: string) => {
  const card = html.match(/<h2>📖 Lingua<\/h2><p>(.*?)<\/p><a class="btn btn-ghost" href="([^"]+)">Discover Lingua<\/a>/s);
  if (!card) throw new Error("no Lingua card on the page");
  return { body: card[1], href: card[2] };
};
const REST =
  "with the words you do not know yet highlighted in place. An honest per-page percentage, one click for the meaning, and a vocabulary that builds itself as you read.";

let container: AstroContainer;
beforeAll(async () => {
  container = await AstroContainer.create();
});
type Page = Parameters<AstroContainer["renderToString"]>[0];
const render = (page: Page) => container.renderToString(page, { partial: false });

describe("/en/ and the pairs it ships", () => {
  it("today's pairs: the Lingua card is today's, byte for byte", async () => {
    ship(TODAY, ROUTES);
    const html = await render(EnglishHome);
    // As origin/main built it before this change.
    expect(linguaArticle(html)).toBe(
      `<article class="product"><p class="kicker">Cymbra Lingua · browser extension <span class="badge">beta</span></p><h2>📖 Lingua</h2><p>Read the English web ${REST}</p><a class="btn btn-ghost" href="/en/lingua">Discover Lingua</a></article>`,
    );
  });

  it("es-en: Spanish, read with an English gloss, and the English Lingua page", async () => {
    ship({ tops: MATRIX.tops, glossed: { ...TODAY.glossed, "es-en": MATRIX.glossed["es-en"] } }, ROUTES);
    expect(linguaCard(await render(EnglishHome))).toEqual({ body: `Read the web in Spanish ${REST}`, href: "/en/lingua" });
    ship(MATRIX, ROUTES);
    expect(linguaCard(await render(EnglishHome))).toEqual({ body: `Read the web in Spanish ${REST}`, href: "/en/lingua" });
  });

  it("es-en and fr-en: Spanish or French, and the English Lingua page", async () => {
    ship(WITH_FR_EN, ROUTES);
    expect(linguaCard(await render(EnglishHome))).toEqual({
      body: `Read the web in Spanish or French ${REST}`,
      href: "/en/lingua",
    });
  });
});
