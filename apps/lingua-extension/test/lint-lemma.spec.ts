import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, extname, join } from "node:path";
import { describe, expect, it } from "vitest";

// Product rule: the word "lemma"/"lemme" must NEVER reach the user (say "forme du
// dictionnaire" / "mots différents"). English "lemma" is a legitimate internal
// identifier (LemmaStatus, .lemma), so the guard is targeted:
//   - HTML and CSS (pure UI/style, no identifiers): no "lemm" at all.
//   - TS: no French word "lemme"/"lemmes" (word-boundary) — the actual user-facing
//     risk in a French UI — while English identifiers like "lemma" are allowed.
// The rendered word popup is separately asserted lemma-free in wordpopup.spec.ts.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(root, "src");

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const name of readdirSync(dir)) {
    if (name === "pkg") continue; // generated wasm glue
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...walk(p));
    else out.push(p);
  }
  return out;
}

const files = walk(SRC);
const FRENCH_LEMME = /\blemmes?\b/i; // matches "lemme"/"lemmes", not "lemma"/"LemmaStatus"
const ANY_LEMM = /lemm/i;

describe("no user-visible 'lemma'/'lemme'", () => {
  it("finds source files to scan", () => {
    expect(files.length).toBeGreaterThan(5);
  });

  for (const file of files) {
    const ext = extname(file);
    if (![".ts", ".css", ".html"].includes(ext)) continue;
    const rel = file.slice(root.length + 1);
    it(`clean: ${rel}`, () => {
      const text = readFileSync(file, "utf8");
      if (ext === ".css" || ext === ".html") {
        expect(text, `${rel} must not contain "lemm" (UI/style has no identifiers)`).not.toMatch(ANY_LEMM);
      } else {
        expect(text, `${rel} must not contain the French word "lemme"`).not.toMatch(FRENCH_LEMME);
      }
    });
  }
});
