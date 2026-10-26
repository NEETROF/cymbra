import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

// The Italian legal pages are courtesy translations of the French, which prevails (change:
// add-site-italian-locale, D2; the Italian programme's I13): each carries its French page's date
// of last update and its sections, so a French edit not mirrored in Italian fails here, naming the
// Italian page. The privacy policy and the terms name the French page they translate
// (`frenchOriginal`, the notice under the title); the support page, which holds no terms, does not.

const pages = resolve(__dirname, "../src/pages");
const source = (file: string) => readFileSync(resolve(pages, file), "utf8");

/** A Markdown page's front matter, as flat `key: value` lines (its `alternates` block skipped). */
function frontMatter(text: string): Record<string, string> {
  const block = text.match(/^---\n(.*?)\n---\n/s)?.[1] ?? "";
  return Object.fromEntries(
    block
      .split("\n")
      .filter((line) => /^\w+:\s*\S/.test(line))
      .map((line) => {
        const at = line.indexOf(":");
        return [line.slice(0, at), line.slice(at + 1).trim()];
      }),
  );
}

/**
 * A page's sections, as their level and their number: « ## 3. … » is `2 3.`, « ### A.2 … » is
 * `3 A.2`, « ## Annexe A — … » / « ## Allegato A — … » is `2 A` — the shape of the text, whatever
 * its language.
 */
function sections(text: string): string[] {
  return [...text.matchAll(/^(#{2,4}) (.+)$/gm)].map(([, hashes, title]) => {
    const number = title.match(/^(\d+\.|[A-Z]\.\d+)\s/)?.[1] ?? title.match(/^\S+ ([A-Z]) —/)?.[1] ?? "";
    return `${hashes.length} ${number}`;
  });
}

const LEGAL: { italian: string; french: string; frenchPath: string }[] = [
  { italian: "it/privacy.md", french: "confidentialite.md", frenchPath: "/confidentialite" },
  { italian: "it/termini.md", french: "cgu.md", frenchPath: "/cgu" },
];

describe("the Italian legal pages follow their French page (I13)", () => {
  for (const { italian, french, frenchPath } of LEGAL) {
    it(`${italian}: the French page's date of last update`, () => {
      const updated = frontMatter(source(french)).updated;
      expect(updated).toMatch(/^\d{2}\/\d{2}\/\d{4}$/);
      expect(frontMatter(source(italian)).updated, `${italian} was not updated with ${french}`).toBe(updated);
    });

    it(`${italian}: the French page's sections, in its order`, () => {
      const expected = sections(source(french));
      expect(expected.length).toBeGreaterThan(5);
      expect(sections(source(italian)), `${italian} has drifted from ${french}'s sections`).toEqual(expected);
    });

    it(`${italian}: names ${frenchPath} as the French page that prevails`, () => {
      expect(frontMatter(source(italian)).frenchOriginal).toBe(frenchPath);
    });
  }

  it("it/assistenza.md, which holds no terms, carries no notice", () => {
    expect(frontMatter(source("it/assistenza.md")).frenchOriginal).toBeUndefined();
    expect(sections(source("it/assistenza.md"))).toEqual(sections(source("support.md")));
  });

  it("it/elimina-account.astro carries the notice and the French page's sections", () => {
    const italian = source("it/elimina-account.astro");
    expect(italian).toContain('<CourtesyNotice lang="it" href="/suppression-compte" />');
    const h2 = (text: string) => [...text.matchAll(/<h2>/g)].length;
    expect(h2(italian)).toBe(h2(source("suppression-compte.astro")));
  });

  it("reads sections the same way in any language", () => {
    expect(sections("## 3. Uso\n### A.2 Contenuti\n## Allegato B — Cymbra Lingua\n## Contatti")).toEqual(["2 3.", "3 A.2", "2 B", "2 "]);
  });
});
