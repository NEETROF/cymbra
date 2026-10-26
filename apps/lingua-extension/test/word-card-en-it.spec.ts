import { describe, expect, it } from "vitest";
import { card } from "@/i18n/it/card.ts";
import { grammar } from "@/i18n/it/grammar.ts";
import { selection } from "@/i18n/it/selection.ts";
import { golden, pinned, pinnedBlock, probes } from "./word-card-french.ts";

// add-lingua-pack-en-it task 4.3, done by refine-lingua-en-it-glosses D15: the word card of an
// Italian-native reader of English, pinned from the en-it golden and never from the tables. Every
// `word-grammar` probe of crates/lingua-wasm/tests/baseline/en-it.golden — the English baseline's 32
// and the 40 lemmas — is rendered with the interface in Italian (change 33's copy, change 34's
// renderer, `src/i18n/it/grammar.ts`), English studied, and every `phrase-gloss` probe as the
// whole-selection card shows it (`word-card-french.ts`); the lines are held in a snapshot. Blessed
// first over change 36's tables, so that a reduction's re-bless shows its own moved lines alone. No
// reader sees them before change 45 lists en-it. Re-blessed with
// `yarn vitest run test/word-card-en-it.spec.ts -u` when the golden is (`lingua-pack-update` does, on
// its branch), and in the pull request that moves the wording, which says why.

const UI = { grammar, card, selection };
const ENGLISH = "en";
const ALL = probes(golden("en-it"));

/** A probe's block, whole and once in the pinned text. */
const block = (name: string) => pinnedBlock(ALL, name, UI, ENGLISH);

describe("the word card of an Italian-native reader of English, over the en-it golden", () => {
  it("renders every grammar and phrase probe as the snapshot pins it", async () => {
    // The reference's 32 grammar probes and the 40 lemmas; 28 phrases.
    expect(ALL.filter((p) => p.kind === "word-grammar")).toHaveLength(72);
    expect(ALL.filter((p) => p.kind === "phrase-gloss")).toHaveLength(28);
    await expect(pinned(ALL, UI, ENGLISH)).toMatchFileSnapshot("./baseline/word-card-en-it.txt");
  });

  it("English forms, as the Italian Wiktionary names them", () => {
    expect(block("word-grammar went go")[1]).toBe("grammar: passato semplice di go");
    expect(block("word-grammar running run")[1]).toBe("grammar: forma in -ing di run");
    expect(block("word-grammar gone go")[1]).toBe("grammar: participio passato di go");
  });

  it("Every gloss page and row in Italian, the pack's gloss unaltered", () => {
    // `render` holds each page to the pack's gloss; a lemma the pack glosses has a first page and a row.
    const lines = block("word-grammar time time");
    expect(lines.some((line) => line.startsWith("page 1/"))).toBe(true);
    expect(lines.at(-1)).toMatch(/^row: /);
  });

  it("No line shows a tag", () => {
    const grammarLines = pinned(ALL, UI, ENGLISH)
      .split("\n")
      .filter((l) => l.startsWith("grammar: "));
    expect(grammarLines.length).toBeGreaterThan(25);
    for (const l of grammarLines) {
      expect(l).not.toMatch(/[A-Za-z]+=|\b(Ind|Sub|Cnd|Imp|Pres|Past|Fut|Fin|Part|Ger|Inf|Sing|Plur|Masc|Fem)\b/);
    }
  });

  it("« give up » answered by its expression, the whole selection covered", () => {
    expect(block("phrase-gloss give up")[1]).toBe("expression: give up");
  });
});
