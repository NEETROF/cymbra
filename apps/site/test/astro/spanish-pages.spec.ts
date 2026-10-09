import { experimental_AstroContainer as AstroContainer } from "astro/container";
import { beforeAll, describe, expect, it, vi } from "vitest";
import type { Coverage, Routes } from "../../src/lib/lingua-pairs";
import { MATRIX, ROUTES, TODAY } from "../support/lingua";

// The Spanish home and Music pages rendered as the build renders them, through Astro's
// Container API (change: extend-site-spanish-locale). As `lingua-page.spec.ts` does, the
// test stands in for `src/data/lingua-coverage.json` and the extension's model catalogue,
// so the home's Lingua card and the layout's Spanish links read the same pair list:
// - today's pairs (en-fr, es-fr): the card names every language read and its readers, and
//   opens the English Lingua page (D3);
// - the matrix (es-en and en-es beside them): the card names English, read with a Spanish
//   gloss, and opens `/es/lingua` — what the build will do once change 35 ships en-es.
// Whatever the pairs, the Spanish brand and « Music » open the Spanish pages (D5), and the
// account and code links the English ones (D4).

const coverage = vi.hoisted(() => ({ tops: [] as number[], glossed: {} as Record<string, number[]> }));
const manifest = vi.hoisted(() => ({ routes: {} as Record<string, string[]> }));
vi.mock("../../src/data/lingua-coverage.json", () => ({ default: coverage }));
vi.mock("../../../lingua-extension/model-manifest.json", () => ({ default: manifest }));
// The community band is rendered only with an invite: none here, as in the Lingua fixtures.
vi.mock("../../src/lib/config", async (importOriginal) => {
  const original = await importOriginal<typeof import("../../src/lib/config")>();
  return { ...original, config: { ...original.config, discordUrl: null } };
});

import SpanishHome from "../../src/pages/es/index.astro";
import SpanishMusic from "../../src/pages/es/music.astro";
import SpanishNotFound from "../../src/pages/es/404.astro";

function ship(data: Coverage, routes: Routes): void {
  coverage.tops = data.tops;
  coverage.glossed = data.glossed;
  manifest.routes = routes;
}
const shipToday = () => ship(TODAY, ROUTES);
const shipTheMatrix = () => ship(MATRIX, ROUTES);

/** The Lingua card of the home page: its text and its button. */
function linguaCard(html: string): { body: string; href: string } {
  const card = html.match(/<h2>📖 Lingua<\/h2>\s*<p>(.*?)<\/p>\s*<a class="btn btn-ghost" href="([^"]+)">Descubrir Lingua<\/a>/s);
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
  [...(html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] ?? "").matchAll(/<a href="([^"]+)">(FR|EN|ES)<\/a>/g)].map((m) => [m[2], m[1]]);
const main = (html: string) => html.match(/<main>(.*?)<\/main>/s)?.[1] ?? "";

let container: AstroContainer;
beforeAll(async () => {
  container = await AstroContainer.create();
});
type Page = Parameters<AstroContainer["renderToString"]>[0];
const render = (page: Page) => container.renderToString(page, { partial: false });

describe("/es/ with today's pairs", () => {
  it("names every language read and the readers it is made for, and opens the English Lingua page", async () => {
    shipToday();
    const html = await render(SpanishHome);
    const card = linguaCard(html);
    expect(card.body).toBe(
      "Lee la web en inglés o en español con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por página, un clic para la traducción y tu vocabulario, que se construye solo. Pensada para francohablantes, con la interfaz y las traducciones en su idioma.",
    );
    expect(card.href).toBe("/en/lingua");
    expect(linkOf(html, "nav", "Lingua")).toBe("/en/lingua");
  });

  it("is Spanish, its brand opens /es/, and it names its French and English twins", async () => {
    shipToday();
    const html = await render(SpanishHome);
    expect(html).toContain('<html lang="es">');
    expect(html).toContain("<title>Cymbra — aprender practicando, no repasando</title>");
    expect(brand(html)).toBe("/es/");
    expect(hreflangs(html)).toEqual([
      ["fr", "https://cymbra.app/"],
      ["en", "https://cymbra.app/en/"],
      ["es", "https://cymbra.app/es/"],
    ]);
    expect(switchTo(html)).toEqual([
      ["FR", "/"],
      ["EN", "/en/"],
    ]);
    expect(main(html)).toContain('<section id="productos" class="products-grid container">');
    expect(main(html)).toContain('<section id="funciones" class="features container">');
    expect(main(html)).toContain('<a class="btn btn-primary" href="/es/music">Descubrir Music</a>');
    // The badge, as on the French and English homes and the Lingua pages (open question 2).
    expect(main(html)).toContain('<p class="kicker">Cymbra Lingua · extensión para el navegador <span class="badge">beta</span></p>');
    // « reproducción » in the offline card (open question 4).
    expect(main(html)).toContain("El análisis y la reproducción se hacen en tu dispositivo");
    expect(html).not.toContain("<astro-island");
  });
});

describe("/es/ with the matrix: a pair glossed in Spanish ships", () => {
  it("names English, read with a Spanish gloss, no audience, and opens /es/lingua", async () => {
    shipTheMatrix();
    const html = await render(SpanishHome);
    const card = linguaCard(html);
    expect(card.body).toBe(
      "Lee la web en inglés con las palabras que aún no conoces resaltadas en la propia página. Un porcentaje honesto por página, un clic para la traducción y tu vocabulario, que se construye solo.",
    );
    expect(card.href).toBe("/es/lingua");
    expect(linkOf(html, "nav", "Lingua")).toBe("/es/lingua");
  });
});

describe("/es/music/", () => {
  it("is Spanish, names its twins, and carries the store buttons and the eight cards", async () => {
    shipToday();
    const html = await render(SpanishMusic);
    expect(html).toContain('<html lang="es">');
    expect(html).toContain("<title>Cymbra Music — práctica y cursos de música conectados</title>");
    expect(hreflangs(html)).toEqual([
      ["fr", "https://cymbra.app/music"],
      ["en", "https://cymbra.app/en/music"],
      ["es", "https://cymbra.app/es/music"],
    ]);
    expect(switchTo(html)).toEqual([
      ["FR", "/music"],
      ["EN", "/en/music"],
    ]);
    expect(main(html)).toContain('<a class="btn btn-primary" href="https://apps.apple.com/app/id6789557194" rel="noopener">App Store (iOS, iPadOS, macOS)</a>');
    expect(main(html)).toContain("Windows / Linux — próximamente");
    expect([...main(html).matchAll(/<div class="card">/g)]).toHaveLength(8);
    expect(main(html)).toContain("<h3>⏸️ Modo Espera</h3>");
    expect(html).not.toContain("<astro-island");
  });

  it("leaves out the French page's closing about plans and codes: no account or code link in the page", async () => {
    shipToday();
    const content = main(await render(SpanishMusic));
    expect(content).not.toContain('class="container closing"');
    expect(content).not.toContain("/account");
    expect(content).not.toContain("/redeem");
  });
});

describe("every Spanish page opens the Spanish home and Music pages (D5), the English account and code pages (D4)", () => {
  for (const [name, page] of [
    ["/es/", SpanishHome],
    ["/es/music/", SpanishMusic],
    ["/es/404", SpanishNotFound],
  ] as const) {
    it(`${name}: brand /es/, « Music » /es/music, « Cuenta » /en/account, « Código de acceso » /en/redeem`, async () => {
      shipToday();
      const html = await render(page);
      expect(brand(html)).toBe("/es/");
      expect(linkOf(html, "nav", "Music")).toBe("/es/music");
      expect(linkOf(html, "footer", "Music")).toBe("/es/music");
      expect(linkOf(html, "nav", "Cuenta")).toBe("/en/account");
      expect(linkOf(html, "footer", "Cuenta")).toBe("/en/account");
      expect(linkOf(html, "footer", "Código de acceso")).toBe("/en/redeem");
    });
  }

  it("the not-found page's « Inicio » and « Cymbra Music » open the Spanish pages", async () => {
    shipToday();
    const html = await render(SpanishNotFound);
    expect(html).toContain('<a class="btn btn-primary" href="/es/">Inicio</a>');
    expect(html).toContain('<a class="btn btn-ghost" href="/es/music">Cymbra Music</a>');
  });
});
