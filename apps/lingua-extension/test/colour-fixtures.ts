import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { Colours } from "@/state/storage.ts";

// The reader's colours, for the tests of the colour sheet and of the Couleurs block.

/** The token sheet, as shipped. */
export const TOKENS = readFileSync(
  join(dirname(fileURLToPath(import.meta.url)), "..", "src", "styles", "tokens.css"),
  "utf8",
);

/** A hand-made set, every colour distinct, so a test sees which one went where. */
export function customColours(): Colours {
  return {
    unknown: {
      fill: { colour: "#ff0000", intensity: "strong" },
      underline: { colour: "#00ff00", style: "solid", thickness: "thick" },
      text: "#0000ff",
    },
    learning: {
      fill: { colour: "#111111", intensity: "none" },
      underline: { colour: "#222222", style: "dotted", thickness: "thin" },
      text: null,
    },
    paper: { background: "#fefefe", text: null },
    dark: { background: "#010101", text: "#eeeeee" },
  };
}

/** The tokens' declared values, the way `getComputedStyle` reads them. */
export function tokenValues(): (token: string) => string {
  const values = new Map([...TOKENS.matchAll(/(--cymbra-lingua-[\w-]+):\s*([^;]+);/g)].map((m) => [m[1], ` ${m[2]}`]));
  return (token) => values.get(token) ?? "";
}
