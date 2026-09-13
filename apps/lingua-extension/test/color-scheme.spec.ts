import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";

// Native widgets (scrollbars, selects) must render dark on the surfaces the extension owns,
// WITHOUT touching the page being read. tokens.css is also adopted by that page, where :root
// is the site's own root, so color-scheme may only be set on :host (the injected shadow
// roots); the extension's own documents declare it with a <meta name="color-scheme">.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(root, "src");
const TOKENS = readFileSync(join(SRC, "styles/tokens.css"), "utf8");

function files(dir: string, ext: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) {
      if (name !== "pkg") out.push(...files(p, ext));
    } else if (p.endsWith(ext)) {
      out.push(p);
    }
  }
  return out;
}

/** Top-level `selectors { body }` rules of a stylesheet, comments stripped. */
function rules(css: string): { selectors: string[]; body: string }[] {
  const stripped = css.replace(/\/\*[\s\S]*?\*\//g, "");
  return [...stripped.matchAll(/([^{}]+)\{([^{}]*)\}/g)].map((m) => ({
    selectors: m[1].split(",").map((s) => s.trim()),
    body: m[2],
  }));
}

describe("dark native widgets without restyling the host page", () => {
  it("never sets color-scheme on :root in the token sheet (it is adopted by the page)", () => {
    const offenders = rules(TOKENS).filter((r) => r.selectors.includes(":root") && /color-scheme/.test(r.body));
    expect(offenders).toEqual([]);
  });

  it("sets a dark color-scheme on :host for the injected shadow roots", () => {
    const host = rules(TOKENS).filter((r) => r.selectors.includes(":host") && /color-scheme\s*:\s*dark/.test(r.body));
    expect(host).toHaveLength(1);
  });

  const pages = files(SRC, ".html").filter((f) => readFileSync(f, "utf8").includes('href="tokens.css"'));

  it("finds the extension's own pages", () => {
    expect(pages.length).toBeGreaterThanOrEqual(4);
  });

  for (const page of pages) {
    const rel = page.slice(SRC.length + 1);
    it(`declares a dark color-scheme: ${rel}`, () => {
      expect(readFileSync(page, "utf8")).toMatch(/<meta name="color-scheme" content="dark"\s*\/?>/);
    });
  }
});
