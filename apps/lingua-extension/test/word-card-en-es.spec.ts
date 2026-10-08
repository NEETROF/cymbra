import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { WordGrammar } from "@/analyzer/types.ts";
import { card } from "@/i18n/es/card.ts";
import { grammar } from "@/i18n/es/grammar.ts";
import { selection } from "@/i18n/es/selection.ts";
import { glossPages, pageText } from "@/reading/gloss-pages.ts";
import { lineText } from "@/reading/grammar-description.ts";
import { rowGloss } from "@/reading/selection-card.ts";

// add-lingua-spanish-card-wording D2: the word card of a Spanish-native reader of English, pinned
// from the en-es golden and never from the tables. Every `word-grammar` probe of
// crates/lingua-wasm/tests/baseline/en-es.golden — the English baseline's and the 40 lemmas — is
// rendered with the interface in Spanish through `src/i18n/es/grammar.ts` and the card's layout
// functions, and the lines are held in a snapshot: the grammar line, each gloss page with its
// sense headings, and the selection card's row. Re-blessed with
// `yarn vitest run test/word-card-en-es.spec.ts -u` when the golden is (`lingua-pack-update` does,
// on its branch), and in the pull request that moves the wording, which says why (M9: the owner
// reviews every Spanish line).

const GOLDEN = join(dirname(fileURLToPath(import.meta.url)), "../../../crates/lingua-wasm/tests/baseline/en-es.golden");

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
 * the card is about — « do » of « don't » — or the first, as the engine picks it; otherwise the word
 * as written. The card is opened on that token (`openForToken`: its `surface` is the piece, its
 * `written` the whole word).
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
  for (const line of grammar.grammarLines(wordGrammar, lemma, surfaceOf(probe), written, "en")) {
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

/** One probe's lines, and a check that its block, whole and once, is in the text the snapshot test pins. */
function pinnedBlock(written: string, lemma: string): string[] {
  const probe = probes().find((p) => p.written === written && p.lemma === lemma)!;
  // `render` checks that the pages are the pack's gloss, unaltered.
  const lines = render(probe);
  // Its gloss headings, pages and row are the snapshot's, whatever en-es's gloss is — the next
  // reduction's included: the probe's block, whole and once, is in the text the first test holds to
  // the committed snapshot. Not read from the file, which a re-bless run (`-u`) writes only once
  // this file's tests are done: this test would fail on the old one.
  const blocks = pinned()
    .replace(/\n$/, "")
    .split(/\n(?=### word-grammar )/);
  expect(blocks.filter((block) => block === lines.join("\n"))).toHaveLength(1);
  expect(lines.some((line) => line.startsWith("page 1/"))).toBe(true);
  expect(lines.at(-1)).toBe(`row: ${rowGloss(probe.grammar.gloss!, selection)}`);
  return lines;
}

describe("the word card of a Spanish-native reader of English, over the en-es golden", () => {
  it("renders every grammar probe as the snapshot pins it", async () => {
    // The reference's 32 probes and the 40 lemmas.
    expect(probes()).toHaveLength(72);
    await expect(pinned()).toMatchFileSnapshot("./baseline/word-card-en-es.txt");
  });

  it("A probe of the reference golden: « went » read as a form of « go »", () => {
    // The simple past by the name Spanish-language teaching of English gives it, never a Spanish
    // tense's (« pretérito perfecto simple », « pretérito indefinido »).
    expect(pinnedBlock("went", "go").slice(0, 2)).toEqual(["### word-grammar went go", "grammar: pasado simple de go"]);
  });

  it("The -ing form: « running » read as a form of « run »", () => {
    // The -ing form as Spanish-language teaching of English names it, never a Spanish gerundio.
    const lines = pinnedBlock("running", "run");
    expect(lines.slice(0, 2)).toEqual(["### word-grammar running run", "grammar: forma en -ing de run"]);
    expect(lines.join("\n")).not.toMatch(/gerundio/);
  });

  it("names no English form by a Spanish tense or mood", () => {
    const grammarLines = pinned()
      .split("\n")
      .filter((line) => line.startsWith("grammar: "));
    expect(grammarLines.length).toBeGreaterThan(0);
    for (const line of grammarLines) {
      expect(line).not.toMatch(/pretérito|indicativo|subjuntivo|gerundio|condicional|imperativo/);
    }
  });
});
