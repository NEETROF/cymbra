import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// A studied language is named in one place, src/analyzer/language-labels.ts
// (add-lingua-language-choice D1): every other source and page asks it, so a surface always names
// the language it speaks of. « anglais » written anywhere else is a label that will say English
// to a reader of Spanish.

const SRC = join(dirname(fileURLToPath(import.meta.url)), "../src");
const LABELS = join(SRC, "analyzer", "language-labels.ts");

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const p = join(dir, name);
    // The catalogue (src/i18n, add-lingua-interface-language) names the languages in every
    // interface language: the labels module's own copy lives there.
    if (statSync(p).isDirectory()) return name === "pkg" || name === "gen" || name === "i18n" ? [] : files(p);
    return /\.(ts|html)$/.test(name) && p !== LABELS ? [p] : [];
  });
}

describe("languages are named in one place", () => {
  const sources = files(SRC);

  it("finds the sources and pages", () => {
    expect(sources.length).toBeGreaterThan(50);
  });

  it("names English nowhere but the labels module", () => {
    const offenders = sources.flatMap((p) =>
      readFileSync(p, "utf8")
        .split("\n")
        .map((line, i) => ({ line, at: `${p.slice(SRC.length + 1)}:${i + 1}` }))
        .filter(({ line }) => /\banglais(e|es)?\b/i.test(line))
        .map(({ at, line }) => `${at}: ${line.trim()}`),
    );
    expect(offenders).toEqual([]);
  });
});
