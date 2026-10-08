import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { rowGloss } from "@/reading/selection-card.ts";

// add-lingua-english-card-wording D4: the selection card's row over every committed gloss of the
// French pairs, pinned before the cut's rules move (M23: the French byte for byte). A row is the
// gloss's first sense as the table writes it, unless the cut or the empty-sense skip touched it:
// those rows — and only those — are written in the snapshot, so the snapshot stays small and every
// row stays pinned: a rule that touched a row it did not touch before adds a line, one that leaves
// a row alone removes one. Re-blessed with `yarn vitest run test/row-gloss-tables.spec.ts -u` when
// the tables move (`lingua-pack-update` does, on its branch), and in the pull request that moves
// the cut, which says why.

const TABLES = join(dirname(fileURLToPath(import.meta.url)), "../../../scripts/lingua-data/tables");

/** The committed glosses of a pair, `(lemma, gloss)`, in the table's order. */
function glosses(pair: string): [string, string][] {
  return readFileSync(join(TABLES, pair, "gloss.tsv"), "utf8")
    .split("\n")
    .filter((line) => line.length > 0)
    .map((line) => {
      const tab = line.indexOf("\t");
      return [line.slice(0, tab), line.slice(tab + 1)];
    });
}

/** The gloss's first sense as the table writes it: what a row is when nothing touched it. */
function firstSense(gloss: string): string {
  return gloss.split(";")[0]!.trim();
}

/** The rows the cut or the skip touched, one `lemma<TAB>row` line each, under a line counting them. */
function touchedRows(pair: string): string {
  const all = glosses(pair);
  const touched = all
    .map(([lemma, gloss]) => ({ lemma, row: rowGloss(gloss), first: firstSense(gloss) }))
    .filter(({ row, first }) => row !== first);
  const lines = touched.map(({ lemma, row }) => `${lemma}\t${row ?? "(none)"}`);
  return [`# ${pair}: ${touched.length} of ${all.length} rows are not the gloss's first sense`, ...lines].join("\n");
}

describe("the selection card's rows over the committed French glosses", () => {
  it("are byte for byte what they were", async () => {
    const text = `${["en-fr", "es-fr"].map(touchedRows).join("\n")}\n`;
    await expect(text).toMatchFileSnapshot("./baseline/selection-rows-fr.txt");
  });
});

// The cut's rules, owned by add-lingua-english-card-wording (D4) and measured on every committed
// gloss of every pair, the English- and Spanish-glossed ones included: the trailing set gains an
// opening mark only when a committed row would end on one — none does — and a closing mark is
// never stripped.
const PAIRS = ["en-fr", "es-fr", "es-en", "en-es"];

/** The marks a row must not end on, before its ellipsis: the openers of every edition the packs read. */
const OPENING = /[“‘«([]…$/u;

describe("the row cut over every committed gloss", () => {
  it("skips no sense: the empty-sense pattern matches no gloss of any edition the packs read", () => {
    for (const pair of PAIRS) {
      for (const [lemma, gloss] of glosses(pair)) {
        expect(gloss, `${pair} ${lemma}`).not.toMatch(/définition manquante/i);
        expect(rowGloss(gloss), `${pair} ${lemma}`).not.toBeNull();
      }
    }
  });

  it("ends no row on an opening mark", () => {
    for (const pair of PAIRS) {
      for (const [lemma, gloss] of glosses(pair)) {
        expect(rowGloss(gloss), `${pair} ${lemma}`).not.toMatch(OPENING);
      }
    }
  });
});
