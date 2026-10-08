import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import manifest from "../../../lingua-extension/model-manifest.json";
import coverage from "../../src/data/lingua-coverage.json";
import takenWith from "../fixtures/lingua/taken-with.json";

// Run after `yarn build` (see `vitest.build.config.ts`). The Lingua page is one component
// fed by the shipped pairs (change: add-site-lingua-matrix-pages); with the pairs the
// fixtures were taken with, the French and English pages must read as origin/main built
// them (D1: the French bytes do not move). Nothing here asks `src/lib/lingua-pairs.ts` what
// to expect: the expectations come from `lingua-coverage.json`'s keys and from `dist/`.

const dist = resolve(__dirname, "../../dist");
const read = (file: string) => readFileSync(resolve(dist, file), "utf8");
const fixture = (lang: string) => readFileSync(resolve(__dirname, `../fixtures/lingua/main.${lang}.html`), "utf8");

/** Every built page, relative to `dist/`. */
function builtPages(dir = dist): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return builtPages(path);
    return entry.name.endsWith(".html") ? [relative(dist, path)] : [];
  });
}

/** The page's `<main>` without what a community invite adds to it. */
function mainWithoutInvite(html: string): string {
  const main = html.match(/<main>(.*?)<\/main>/s)?.[1] ?? "";
  return main
    .replace(/(<p class="muted small hero-note">[^<]*) <a href="[^"]*" rel="noopener">[^<]*<\/a>[^<]*<\/p>/, "$1</p>")
    .replace(/<section class="container community".*?<\/section>/s, "");
}

/** The coverage table's cells blanked: what is left is every byte but the figures. */
const withoutFigures = (main: string) => main.replace(/<td>[^<]*<\/td>/g, "<td></td>");

describe("the Lingua page with the pairs the fixtures were taken with", () => {
  // The fixtures (`test/fixtures/lingua/`) hold the `<main>` of `/lingua/` and `/en/lingua/`
  // as origin/main built them, with the pairs, figures and routes of `taken-with.json`. A
  // figure refreshed since (lingua-pack-update) blanks the table's cells on both sides; a pair
  // added or removed, or a route changed, moves the page's words: the comparison is skipped,
  // saying so, and the fixtures are refreshed as `apps/site/README.md` says.
  // `test/astro/lingua-page.spec.ts` keeps the byte pin on the fixtures' own pairs either way.
  const livePairs = Object.keys(coverage.glossed);
  const liveRoutes = Object.fromEntries(
    livePairs.map((pair) => [pair, Object.hasOwn(manifest.routes, pair) ? manifest.routes[pair as keyof typeof manifest.routes] : []]),
  );
  const sameWords =
    JSON.stringify(livePairs) === JSON.stringify(Object.keys(takenWith.coverage.glossed)) &&
    JSON.stringify(coverage.tops) === JSON.stringify(takenWith.coverage.tops) &&
    JSON.stringify(liveRoutes) === JSON.stringify(takenWith.routes);
  const sameFigures = JSON.stringify(coverage.glossed) === JSON.stringify(takenWith.coverage.glossed);
  const why =
    `the fixtures were taken with ${Object.keys(takenWith.coverage.glossed).join(", ")} (routes ${JSON.stringify(takenWith.routes)}), ` +
    `the build ships ${livePairs.join(", ")} (routes ${JSON.stringify(liveRoutes)}): the pages' words moved with them — ` +
    "refresh test/fixtures/lingua/ as apps/site/README.md says";
  const compare = (built: string, lang: string) => {
    const [actual, expected] = [mainWithoutInvite(built), fixture(lang)];
    if (sameFigures) expect(actual).toBe(expected);
    else expect(withoutFigures(actual)).toBe(withoutFigures(expected));
  };

  it("French: <main> is the fixture's, byte for byte (but the figures, once refreshed)", (ctx) => {
    ctx.skip(!sameWords, why);
    compare(read("lingua/index.html"), "fr");
  });

  it("English: <main> is the fixture's, byte for byte (but the figures, once refreshed)", (ctx) => {
    ctx.skip(!sameWords, why);
    compare(read("en/lingua/index.html"), "en");
  });

  it("titles and describes each page as before", () => {
    const fr = read("lingua/index.html");
    expect(fr).toContain("<title>Cymbra Lingua — enrichissez votre vocabulaire en lisant le web</title>");
    expect(fr).toContain('<html lang="fr"');
    const en = read("en/lingua/index.html");
    expect(en).toContain("<title>Cymbra Lingua — grow your vocabulary while you read the web</title>");
    expect(en).toContain('<html lang="en"');
  });
});

describe("every link to a Lingua page opens one", () => {
  it("each href to …/lingua, hreflang included, is a page of dist/", () => {
    const missing: string[] = [];
    let links = 0;
    for (const page of builtPages()) {
      for (const [, path] of read(page).matchAll(/href="(?:https:\/\/cymbra\.app)?(\/(?:[a-z]{2}\/)?lingua)\/?"/g)) {
        links += 1;
        if (!existsSync(resolve(dist, `.${path}`, "index.html"))) missing.push(`${page} → ${path}`);
      }
    }
    expect(links).toBeGreaterThan(0);
    expect(missing).toEqual([]);
  });
});

describe("a Spanish Lingua page only once a pair glossed in Spanish ships (D3)", () => {
  // From the keys of the file the build reads, `<studied>-<native>`: no helper of the site's.
  const glossedInSpanish = Object.keys(coverage.glossed).filter((pair) => pair.split("-")[1] === "es");
  const expected = glossedInSpanish.length ? "/es/lingua" : "/en/lingua";
  const esPage = resolve(dist, "es/lingua/index.html");
  const spanishPages = builtPages().filter((page) => page.startsWith("es/") && !page.startsWith("es/lingua/"));

  it("builds /es/lingua/ exactly when a shipped pair is glossed in Spanish", () => {
    expect(existsSync(esPage), "dist/es/lingua/index.html").toBe(glossedInSpanish.length > 0);
  });

  it(`links Lingua from every Spanish page's nav and footer to ${expected}`, () => {
    expect(spanishPages.length).toBeGreaterThan(0);
    for (const page of spanishPages) {
      const html = read(page);
      const nav = html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1] ?? "";
      const footer = html.match(/<footer class="site-footer">(.*?)<\/footer>/s)?.[1] ?? "";
      expect(nav, `${page} nav`).toContain(`<a href="${expected}">Lingua</a>`);
      expect(footer, `${page} footer`).toContain(`<a href="${expected}">Lingua</a>`);
    }
    expect(read("es/404.html")).toContain(`<a class="btn btn-ghost" href="${expected}">Cymbra Lingua</a>`);
  });

  it.skipIf(glossedInSpanish.length === 0)(
    "/es/lingua/ is Spanish, leads with a Spanish-glossed pair and is linked from its own nav and footer",
    () => {
      const html = read("es/lingua/index.html");
      expect(html).toContain('<html lang="es"');
      expect(html).toContain('<link rel="alternate" hreflang="es" href="https://cymbra.app/es/lingua">');
      expect(html.match(/<nav class="nav-links">(.*?)<\/nav>/s)?.[1]).toContain('<a href="/es/lingua">Lingua</a>');
      expect(html.match(/<footer class="site-footer">(.*?)<\/footer>/s)?.[1]).toContain('<a href="/es/lingua">Lingua</a>');
      // The first coverage column after the row header is a pair glossed in Spanish: its
      // header names the studied language, then « → español » when other readers' pairs
      // follow.
      const headers = [...html.matchAll(/<th>([^<]*)<\/th>/g)].map((m) => m[1]);
      const spanishOnly = Object.keys(coverage.glossed).every((pair) => pair.endsWith("-es"));
      expect(headers[1]).toMatch(spanishOnly ? /^\p{Lu}\p{Ll}+$/u : /^\p{Lu}\p{Ll}+ → español$/u);
    },
  );
});
