import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";
import { HL_LEARNING, HL_UNKNOWN } from "@/reading/highlight.ts";

// The two highlight statuses must stay apart without colour (lingua-browser-extension, "Cymbra
// visual identity"): on a monochrome e-ink screen the amber and coral tints are the same grey,
// so the underline's STYLE is what tells a learning word from an unknown one.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const TOKENS = readFileSync(join(root, "src/styles/tokens.css"), "utf8").replace(/\/\*[\s\S]*?\*\//g, "");

/** The `text-decoration` of a `::highlight(name)` rule. */
function decoration(name: string): string {
  const body = TOKENS.match(new RegExp(`::highlight\\(${name}\\)\\s*\\{([^}]*)\\}`))?.[1] ?? "";
  return body.match(/text-decoration:\s*([^;]+);/)?.[1]?.trim() ?? "";
}

/** The line style among a `text-decoration` value's keywords. */
function lineStyle(value: string): string | undefined {
  return value.split(/\s+/).find((w) => ["solid", "double", "dotted", "dashed", "wavy"].includes(w));
}

describe("highlight statuses without colour", () => {
  it("underlines a learning word dotted and an unknown word solid", () => {
    expect(lineStyle(decoration(HL_LEARNING))).toBe("dotted");
    expect(lineStyle(decoration(HL_UNKNOWN))).toBe("solid");
  });

  it("keeps both tints from the token sheet", () => {
    expect(decoration(HL_LEARNING)).toContain("var(--cymbra-lingua-learning-underline)");
    expect(decoration(HL_UNKNOWN)).toContain("var(--cymbra-lingua-unknown-underline)");
  });
});

/** A token's value in the sheet. */
function token(name: string): string {
  return TOKENS.match(new RegExp(`--cymbra-lingua-${name}:\\s*([^;]+);`))?.[1]?.trim() ?? "";
}

// The reader's dark page (add-lingua-reader D10) sits under the word popup and the reader's
// panels: were it their colour, a card would vanish into the page — as it did at first.
describe("the dark page under the reading surfaces", () => {
  it("is not the colour of any surface laid over it", () => {
    const night = token("night").toLowerCase();
    expect(night).toMatch(/^#[0-9a-f]{6}$/);
    for (const surface of ["panel", "panel-2", "panel-3"]) expect(token(surface).toLowerCase()).not.toBe(night);
  });
});
