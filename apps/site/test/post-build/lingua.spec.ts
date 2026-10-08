import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { linguaHref, linguaLocalePaths, linguaPairs, pairsByNative } from "../../src/lib/lingua-pairs";
import { LINGUA_TEXT } from "../../src/lib/lingua-text";
import { outputFileFor } from "../../src/lib/pinned-routes";

// Run after `yarn build` (see `vitest.build.config.ts`). The Lingua page is one component
// fed by the shipped pairs (change: add-site-lingua-matrix-pages); with today's pairs the
// French and English pages must read as the committed pages did (M23: the French bytes do
// not move). The fixtures are each page's `<main>` as origin/main built it, with no
// community invite; a build with PUBLIC_DISCORD_URL set adds the invite to the hero note
// and the community band, which the comparison removes first. The Spanish page exists only
// once a pair glossed in Spanish ships (D3), so the second block asserts whichever way the
// shipped pairs point.

const dist = resolve(__dirname, "../../dist");
const read = (file: string) => readFileSync(resolve(dist, file), "utf8");
const fixture = (lang: string) => readFileSync(resolve(__dirname, `fixtures/lingua-main.${lang}.html`), "utf8");

/** The page's `<main>` without what a community invite adds to it. */
function mainWithoutInvite(html: string): string {
  const main = html.match(/<main>(.*?)<\/main>/s)?.[1] ?? "";
  return main
    .replace(/(<p class="muted small hero-note">[^<]*) <a href="[^"]*" rel="noopener">[^<]*<\/a>[^<]*<\/p>/, "$1</p>")
    .replace(/<section class="container community".*?<\/section>/s, "");
}

describe("the Lingua page with today's pairs", () => {
  const today = linguaPairs();
  const onlyFrenchNative = today.every((p) => p.native === "fr");

  it.skipIf(!onlyFrenchNative)("French: <main> is the committed page's, byte for byte", () => {
    expect(mainWithoutInvite(read(outputFileFor("/lingua/")))).toBe(fixture("fr"));
  });

  it.skipIf(!onlyFrenchNative)("English: <main> reads as the committed page's", () => {
    expect(mainWithoutInvite(read(outputFileFor("/en/lingua/")))).toBe(fixture("en"));
  });

  it("titles and describes each page as before", () => {
    const fr = read(outputFileFor("/lingua/"));
    expect(fr).toContain("<title>Cymbra Lingua — enrichissez votre vocabulaire en lisant le web</title>");
    expect(fr).toContain('<html lang="fr"');
    const en = read(outputFileFor("/en/lingua/"));
    expect(en).toContain("<title>Cymbra Lingua — grow your vocabulary while you read the web</title>");
    expect(en).toContain('<html lang="en"');
  });
});

describe("a Spanish Lingua page only once a pair glossed in Spanish ships (D3)", () => {
  const extra = linguaLocalePaths();
  const esPage = resolve(dist, outputFileFor("/es/lingua/"));

  it("builds /es/lingua/ exactly when the shipped pairs call for it", () => {
    expect(existsSync(esPage), "dist/es/lingua/index.html").toBe(extra.some((p) => p.params.locale === "es"));
  });

  it("sends the Spanish not-found page's Lingua link where the page exists", () => {
    const notFound = read("es/404.html");
    expect(notFound).toContain(`<a class="btn btn-ghost" href="${linguaHref("es")}">Cymbra Lingua</a>`);
    const nav = notFound.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] ?? "";
    expect(nav).toContain(`<a href="${linguaHref("es")}">Lingua</a>`);
  });

  it.skipIf(!existsSync(esPage))("/es/lingua/ is Spanish, leads with the Spanish-glossed pairs and is linked from the Spanish nav", () => {
    const html = read(outputFileFor("/es/lingua/"));
    expect(html).toContain('<html lang="es"');
    expect(html).toContain(`<title>${LINGUA_TEXT.es.title}</title>`);
    expect(html).toContain('<link rel="alternate" hreflang="es" href="https://cymbra.app/es/lingua">');
    expect(html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1]).toContain('<a href="/es/lingua">Lingua</a>');
    // The first coverage column after the row header is the first Spanish-glossed pair.
    const first = pairsByNative("es", linguaPairs())[0];
    expect(first.native).toBe("es");
    const headers = [...html.matchAll(/<th>([^<]*)<\/th>/g)].map((m) => m[1]);
    expect(headers[1]).toBe(`${LINGUA_TEXT.es.names[first.pairs[0].studied].replace(/^./, (c) => c.toUpperCase())} → español`);
  });
});
