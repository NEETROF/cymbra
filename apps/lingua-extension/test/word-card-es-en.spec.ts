import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { WordGrammar } from "@/analyzer/types.ts";
import { card } from "@/i18n/en/card.ts";
import { grammar } from "@/i18n/en/grammar.ts";
import { selection } from "@/i18n/en/selection.ts";
import { glossPages, pageText } from "@/reading/gloss-pages.ts";
import { lineText } from "@/reading/grammar-description.ts";
import { rowGloss } from "@/reading/selection-card.ts";

// add-lingua-english-card-wording D2: the word card of an English-native reader of Spanish,
// pinned from the es-en golden and never from the tables. Every `word-grammar` probe of
// crates/lingua-wasm/tests/baseline/es-en.golden — the Spanish baseline's and the 40 lemmas —
// is rendered with the interface in English through `src/i18n/en/grammar.ts` and the card's
// layout functions, and the lines are held in a snapshot: the grammar line, each gloss page with
// its sense headings, and the selection card's row. Re-blessed with
// `yarn vitest run test/word-card-es-en.spec.ts -u` when the golden is (`lingua-pack-update` does,
// on its branch), and in the pull request that moves the wording, which says why (M9: the owner
// reviews every English line).

const GOLDEN = join(dirname(fileURLToPath(import.meta.url)), "../../../crates/lingua-wasm/tests/baseline/es-en.golden");

interface Probe {
  written: string;
  lemma: string;
  grammar: WordGrammar;
}

/** The golden's grammar probes, in order: `### word-grammar <written> <lemma>`, then one JSON line. */
function probes(): Probe[] {
  const lines = readFileSync(GOLDEN, "utf8").split("\n");
  const out: Probe[] = [];
  lines.forEach((line, i) => {
    const probe = /^### word-grammar (\S+) (\S+)$/.exec(line);
    if (probe) out.push({ written: probe[1]!, lemma: probe[2]!, grammar: JSON.parse(lines[i + 1]!) as WordGrammar });
  });
  return out;
}

/**
 * The form a card's readings are of: for a word the pre-pass split, the piece whose dictionary form
 * the card is about — here the one spelled like it, « a » of « al » — or the first, as the engine
 * picks it; otherwise the word as written. The card is opened on that token (`openForToken`: its
 * `surface` is the piece, its `written` the whole word).
 */
function surfaceOf({ written, lemma, grammar: wordGrammar }: Probe): string {
  const { pieces } = wordGrammar;
  if (pieces.length < 2) return written;
  return pieces.find((piece) => piece.toLowerCase() === lemma.toLowerCase()) ?? pieces[0]!;
}

/** What the card shows of one probe, as the reader reads it, one line each. */
function render(probe: Probe): string[] {
  const { written, lemma, grammar: wordGrammar } = probe;
  const out = [`### word-grammar ${written} ${lemma}`];
  for (const line of grammar.grammarLines(wordGrammar, lemma, surfaceOf(probe), written, "es")) {
    out.push(`grammar: ${lineText(line)}`);
  }
  const { gloss } = wordGrammar;
  if (gloss === null) {
    out.push("gloss: (none)");
    return out;
  }
  const pages = glossPages(gloss, wordGrammar.senses);
  // The pages are the pack's gloss, unaltered.
  expect(pages.map(pageText).join(card.senseSeparator)).toBe(gloss);
  pages.forEach((page, i) => {
    out.push(`page ${i + 1}/${pages.length}:`);
    for (const group of page) {
      const heading = group.tagged ? grammar.senseHeading(group.tag) : null;
      out.push(`  [${heading ?? "—"}] ${group.senses.join(card.senseSeparator)}`);
    }
  });
  out.push(`row: ${rowGloss(gloss, selection) ?? "(none)"}`);
  return out;
}

/** The text the snapshot pins: every probe's lines, in the golden's order. */
function pinned(): string {
  return `${probes().flatMap(render).join("\n")}\n`;
}

describe("the word card of an English-native reader of Spanish, over the es-en golden", () => {
  it("renders every grammar probe as the snapshot pins it", async () => {
    // The reference's 28 probes and the 40 lemmas.
    expect(probes()).toHaveLength(68);
    await expect(pinned()).toMatchFileSnapshot("./baseline/word-card-es-en.txt");
  });

  it("A probe of the reference golden: « vino » read as a form of « venir »", () => {
    const probe = probes().find((p) => p.written === "vino" && p.lemma === "venir")!;
    // `render` checks that the pages are the pack's gloss, unaltered.
    const lines = render(probe);
    expect(lines.slice(0, 2)).toEqual([
      "### word-grammar vino venir",
      "grammar: third-person singular preterite indicative of venir",
    ]);
    // Its gloss headings, pages and row are the snapshot's, whatever es-en's gloss of « venir »
    // is — the next reduction's included: the probe's block, whole and once, is in the text the
    // test above holds to the committed snapshot. Not read from the file, which a re-bless run
    // (`-u`) writes only once this file's tests are done: this test would fail on the old one.
    const blocks = pinned()
      .replace(/\n$/, "")
      .split(/\n(?=### word-grammar )/);
    expect(blocks.filter((block) => block === lines.join("\n"))).toHaveLength(1);
    expect(lines.some((line) => line.startsWith("page 1/"))).toBe(true);
    expect(lines.at(-1)).toBe(`row: ${rowGloss(probe.grammar.gloss!, selection)}`);
  });
});
