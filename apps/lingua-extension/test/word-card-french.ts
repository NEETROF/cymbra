import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect } from "vitest";
import type { PhraseGloss, WordGrammar } from "@/analyzer/types.ts";
import type { card as frCard } from "@/i18n/fr/card.ts";
import type { GrammarRenderer } from "@/i18n/index.ts";
import { glossPages, pageText } from "@/reading/gloss-pages.ts";
import { lineText } from "@/reading/grammar-description.ts";
import { rowGloss, rowsFor, type SelectionCopy, wholeSelectionMatch } from "@/reading/selection-card.ts";

// add-lingua-french-word-card D9: the French card's lines, read from a French golden — fr-en's or
// fr-es's — and never from the tables, for `word-card-fr-en.spec.ts` and `word-card-fr-es.spec.ts`.
// Every `word-grammar` probe is rendered as the word card shows it with French studied: its grammar
// lines, each gloss page with its sense headings, and the selection card's row, as change 23's
// `word-card-es-en.spec.ts` renders es-en's; a word the pre-pass split, on its piece. Every
// `phrase-gloss` probe is rendered as the whole-selection card shows it: headed by the expression's
// name, with its gloss, when one covers the whole selection; else its word-by-word rows.

/** The interface a snapshot renders in: the grammar renderer, the card's and the selection card's copy. */
export interface Interface {
  grammar: GrammarRenderer;
  card: typeof frCard;
  selection: SelectionCopy;
}

interface GrammarProbe {
  kind: "word-grammar";
  name: string;
  written: string;
  lemma: string;
  grammar: WordGrammar;
}

interface PhraseProbe {
  kind: "phrase-gloss";
  name: string;
  answer: PhraseGloss;
}

export type Probe = GrammarProbe | PhraseProbe;

/** A golden of `crates/lingua-wasm/tests/baseline/`. */
export function golden(pair: string): string {
  return join(dirname(fileURLToPath(import.meta.url)), `../../../crates/lingua-wasm/tests/baseline/${pair}.golden`);
}

/** The golden's grammar and phrase probes, in its order: `### <probe>`, then one JSON line. */
export function probes(path: string): Probe[] {
  const lines = readFileSync(path, "utf8").split("\n");
  const out: Probe[] = [];
  lines.forEach((line, i) => {
    const grammar = /^### (word-grammar (\S+) (\S+))$/.exec(line);
    if (grammar) {
      const [, name, written, lemma] = grammar;
      out.push({
        kind: "word-grammar",
        name: name!,
        written: written!,
        lemma: lemma!,
        grammar: JSON.parse(lines[i + 1]!),
      });
    }
    const phrase = /^### (phrase-gloss .+)$/.exec(line);
    if (phrase) out.push({ kind: "phrase-gloss", name: phrase[1]!, answer: JSON.parse(lines[i + 1]!) });
  });
  return out;
}

/**
 * The form a card's readings are of: for a word the pre-pass split, the piece whose dictionary form
 * the card is about — `homme` of « l'homme », `à` of « au » — or the first, as the engine picks it;
 * otherwise the word as written. The card is opened on that token (`openForToken`: its `surface` is
 * the piece, its `written` the whole word).
 */
function surfaceOf({ written, lemma, grammar }: GrammarProbe): string {
  const { pieces } = grammar;
  if (pieces.length < 2) return written;
  return pieces.find((piece) => piece.toLowerCase() === lemma.toLowerCase()) ?? pieces[0]!;
}

/** A gloss, page by page, as the card shows it. */
function pagesOf(gloss: string, senses: WordGrammar["senses"], ui: Interface): string[] {
  const pages = glossPages(gloss, senses);
  // The pages are the pack's gloss, unaltered.
  expect(pages.map(pageText).join(ui.card.senseSeparator)).toBe(gloss);
  return pages.flatMap((page, i) => [
    `page ${i + 1}/${pages.length}:`,
    ...page.map((group) => {
      const heading = group.tagged ? ui.grammar.senseHeading(group.tag) : null;
      return `  [${heading ?? "—"}] ${group.senses.join(ui.card.senseSeparator)}`;
    }),
  ]);
}

/** What the card shows of one probe, as the reader reads it, one line each. */
export function render(probe: Probe, ui: Interface): string[] {
  const out = [`### ${probe.name}`];
  if (probe.kind === "phrase-gloss") {
    const whole = wholeSelectionMatch(probe.answer);
    if (whole) {
      out.push(`expression: ${whole.key}`);
      out.push(...(whole.gloss === null ? ["gloss: (none)"] : pagesOf(whole.gloss, [], ui)));
      return out;
    }
    const rows = rowsFor(probe.answer.tokens, probe.answer.expressions, ui.selection);
    if (rows.length === 0) out.push("rows: (none)");
    for (const row of rows) out.push(`row: ${row.form} — ${row.gloss}`);
    return out;
  }
  const { written, lemma, grammar: wordGrammar } = probe;
  for (const line of ui.grammar.grammarLines(wordGrammar, lemma, surfaceOf(probe), written, "fr")) {
    out.push(`grammar: ${lineText(line)}`);
  }
  const { gloss } = wordGrammar;
  if (gloss === null) {
    out.push("gloss: (none)");
    return out;
  }
  out.push(...pagesOf(gloss, wordGrammar.senses, ui));
  out.push(`row: ${rowGloss(gloss, ui.selection) ?? "(none)"}`);
  return out;
}

/** The text a snapshot pins: every probe's lines, in the golden's order. */
export function pinned(all: readonly Probe[], ui: Interface): string {
  return `${all.flatMap((probe) => render(probe, ui)).join("\n")}\n`;
}

/**
 * One probe's lines, and a check that its block, whole and once, is in the text the snapshot test
 * pins — whatever the pair's glosses are, the next reduction's included. Not read from the file,
 * which a re-bless run (`-u`) writes only once the spec's tests are done.
 */
export function pinnedBlock(all: readonly Probe[], name: string, ui: Interface): string[] {
  const probe = all.find((p) => p.name === name);
  expect(probe, name).toBeDefined();
  const lines = render(probe!, ui);
  const blocks = pinned(all, ui)
    .replace(/\n$/, "")
    .split(/\n(?=### )/);
  expect(blocks.filter((block) => block === lines.join("\n"))).toHaveLength(1);
  return lines;
}
