import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { beforeAll, describe, expect, it, vi } from "vitest";
import type { Coverage, Routes } from "../../src/lib/lingua-pairs";
import { NINE_PAIRS, ROUTES } from "../support/lingua";

// The Italian home, Music and not-found pages rendered as the build renders them, through Astro's
// Container API (change: add-site-italian-locale), as `spanish-pages.spec.ts` does for Spanish. The
// test stands in for `src/data/lingua-coverage.json` and the extension's model catalogue:
// - the nine pairs (no pair glossed in Italian): the home's Lingua card names every language read
//   and the readers it is made for, never that Lingua explains words in Italian, and opens the
//   English Lingua page (D4);
// - the nine pairs and en-it (change 45): the card names English, read with an Italian gloss, no
//   audience, and opens `/it/lingua`.
// Whatever the pairs, the Italian brand and « Music » open the Italian pages, and the account and
// code links the English ones.

const coverage = vi.hoisted(() => ({ tops: [] as number[], glossed: {} as Record<string, number[]> }));
const manifest = vi.hoisted(() => ({ routes: {} as Record<string, string[]> }));
vi.mock("../../src/data/lingua-coverage.json", () => ({ default: coverage }));
vi.mock("../../../lingua-extension/model-manifest.json", () => ({ default: manifest }));
// The community band is rendered only with an invite: none here, as in the Lingua fixtures.
vi.mock("../../src/lib/config", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../src/lib/config")>();
  return { ...original, config: { ...original.config, discordUrl: null } };
});

import ItalianHome from "../../src/pages/it/index.astro";
import ItalianMusic from "../../src/pages/it/music.astro";
import ItalianNotFound from "../../src/pages/it/404.astro";

function ship(data: Coverage, routes: Routes): void {
  coverage.tops = data.tops;
  coverage.glossed = data.glossed;
  manifest.routes = routes;
}
const shipToday = () => ship(NINE_PAIRS, ROUTES);
const shipEnIt = () => ship({ tops: NINE_PAIRS.tops, glossed: { ...NINE_PAIRS.glossed, "en-it": [91.4, 82.8, 70.3] } }, ROUTES);

/** The Lingua card of the home page: its text and its button. */
function linguaCard(html: string): { body: string; href: string } {
  const card = html.match(/<h2>📖 Lingua<\/h2>\s*<p>(.*?)<\/p>\s*<a class="btn btn-ghost" href="([^"]+)">Scopri Lingua<\/a>/s);
  if (!card) throw new Error("no Lingua card on the page");
  return { body: card[1], href: card[2] };
}
const brand = (html: string) => html.match(/<a class="brand" href="([^"]+)">Cymbra<\/a>/)?.[1];
const linkOf = (html: string, where: "nav" | "footer", label: string) => {
  const block = where === "nav" ? html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] : html.match(/<footer class="site-footer">(.*?)<\/footer>/s)?.[1];
  return block?.match(new RegExp(`<a href="([^"]+)">${label}</a>`))?.[1];
};
const hreflangs = (html: string) => [...html.matchAll(/<link rel="alternate" hreflang="([a-z]+)" href="([^"]+)">/g)].map((m) => [m[1], m[2]]);
const switchTo = (html: string) =>
  [...(html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] ?? "").matchAll(/<a href="([^"]+)">(FR|EN|ES|IT)<\/a>/g)].map((m) => [m[2], m[1]]);
const main = (html: string) => html.match(/<main>(.*?)<\/main>/s)?.[1] ?? "";

let container: AstroContainer;
beforeAll(async () => {
  container = await AstroContainer.create();
});
type Page = Parameters<AstroContainer["renderToString"]>[0];
const render = (page: Page) => container.renderToString(page, { partial: false });

describe("/it/ before any pair glossed in Italian", () => {
  it("names every language read and the readers it is made for, and opens the English Lingua page", async () => {
    shipToday();
    const html = await render(ItalianHome);
    const card = linguaCard(html);
    expect(card.body).toBe(
      "Leggi il web in inglese, in spagnolo, in francese o in italiano, con le parole che non conosci ancora evidenziate direttamente nella pagina. Una percentuale onesta per pagina, un clic per la traduzione e il tuo vocabolario, che si costruisce da solo. Pensata per francofoni, anglofoni e ispanofoni, con l'interfaccia e le traduzioni nella propria lingua.",
    );
    // Never that Lingua explains words in Italian before a pair glossed in Italian ships.
    expect(card.body).not.toMatch(/traduzioni in italiano|in italiano:/);
    expect(card.href).toBe("/en/lingua");
    expect(linkOf(html, "nav", "Lingua")).toBe("/en/lingua");
  });

  it("is Italian, its brand opens /it/, and it names its French, English and Spanish twins", async () => {
    shipToday();
    const html = await render(ItalianHome);
    expect(html).toContain('<html lang="it">');
    expect(html).toContain("<title>Cymbra — imparare facendo pratica, non ripassando</title>");
    expect(brand(html)).toBe("/it/");
    expect(hreflangs(html)).toEqual([
      ["fr", "https://cymbra.app/"],
      ["en", "https://cymbra.app/en/"],
      ["es", "https://cymbra.app/es/"],
      ["it", "https://cymbra.app/it/"],
    ]);
    expect(switchTo(html)).toEqual([
      ["FR", "/"],
      ["EN", "/en/"],
      ["ES", "/es/"],
    ]);
    expect(main(html)).toContain('<a class="btn btn-primary" href="/it/music">Scopri Music</a>');
    expect(main(html)).toContain('<p class="kicker">Cymbra Lingua · estensione per il browser <span class="badge">beta</span></p>');
    expect(html).not.toContain("<astro-island");
  });
});

describe("/it/ once en-it ships: Italian speakers study English", () => {
  it("names English, read with an Italian gloss, no audience, and opens /it/lingua", async () => {
    shipEnIt();
    const html = await render(ItalianHome);
    const card = linguaCard(html);
    expect(card.body).toBe(
      "Leggi il web in inglese, con le parole che non conosci ancora evidenziate direttamente nella pagina. Una percentuale onesta per pagina, un clic per la traduzione e il tuo vocabolario, che si costruisce da solo.",
    );
    expect(card.body).not.toContain("Pensata per");
    expect(card.href).toBe("/it/lingua");
    expect(linkOf(html, "nav", "Lingua")).toBe("/it/lingua");
  });
});

describe("/it/music/", () => {
  it("is Italian, names its twins, and carries the store buttons and the eight cards", async () => {
    shipToday();
    const html = await render(ItalianMusic);
    expect(html).toContain('<html lang="it">');
    expect(html).toContain("<title>Cymbra Music — pratica e corsi di musica collegati</title>");
    expect(hreflangs(html)).toEqual([
      ["fr", "https://cymbra.app/music/"],
      ["en", "https://cymbra.app/en/music/"],
      ["es", "https://cymbra.app/es/music/"],
      ["it", "https://cymbra.app/it/music/"],
    ]);
    expect(switchTo(html)).toEqual([
      ["FR", "/music"],
      ["EN", "/en/music"],
      ["ES", "/es/music"],
    ]);
    expect(main(html)).toContain('<a class="btn btn-primary" href="https://apps.apple.com/app/id6789557194" rel="noopener">App Store (iOS, iPadOS, macOS)</a>');
    expect(main(html)).toContain("Windows / Linux — in arrivo");
    expect([...main(html).matchAll(/<div class="card">/g)]).toHaveLength(8);
    expect(main(html)).toContain("<h3>⏸️ Modalità attesa</h3>");
    expect(html).not.toContain("<astro-island");
  });

  it("leaves out the French page's closing about plans and codes, as in Spanish", async () => {
    shipToday();
    const content = main(await render(ItalianMusic));
    expect(content).not.toContain('class="container closing"');
    expect(content).not.toContain("/account");
    expect(content).not.toContain("/redeem");
  });
});

describe("every Italian page opens the Italian home and Music pages, the English account and code pages", () => {
  for (const [name, page] of [
    ["/it/", ItalianHome],
    ["/it/music/", ItalianMusic],
    ["/it/404", ItalianNotFound],
  ] as const) {
    it(`${name}: brand /it/, « Music » /it/music, « Account » /en/account, « Codice di accesso » /en/redeem`, async () => {
      shipToday();
      const html = await render(page);
      expect(brand(html)).toBe("/it/");
      expect(linkOf(html, "nav", "Music")).toBe("/it/music");
      expect(linkOf(html, "footer", "Music")).toBe("/it/music");
      expect(linkOf(html, "nav", "Account")).toBe("/en/account");
      expect(linkOf(html, "footer", "Account")).toBe("/en/account");
      expect(linkOf(html, "footer", "Codice di accesso")).toBe("/en/redeem");
    });
  }

  it("the not-found page's « Home » and « Cymbra Music » open the Italian pages", async () => {
    shipToday();
    const html = await render(ItalianNotFound);
    expect(html).toContain('<a class="btn btn-primary" href="/it/">Home</a>');
    expect(html).toContain('<a class="btn btn-ghost" href="/it/music">Cymbra Music</a>');
  });
});
