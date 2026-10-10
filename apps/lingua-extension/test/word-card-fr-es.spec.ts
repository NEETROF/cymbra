import { describe, expect, it } from "vitest";
import { card } from "@/i18n/es/card.ts";
import { grammar } from "@/i18n/es/grammar.ts";
import { selection } from "@/i18n/es/selection.ts";
import { golden, pinned, pinnedBlock, probes } from "./word-card-french.ts";

// add-lingua-french-word-card D9: the word card of a Spanish-native reader of French, pinned from
// the fr-es golden and never from the tables. Every `word-grammar` probe of
// crates/lingua-wasm/tests/baseline/fr-es.golden — the French scenario glossed in Spanish
// (`fr_es_baseline.rs`): the French baseline's 31, the card's 21 and the 40 lemmas — is rendered
// with the interface in Spanish through `src/i18n/es/grammar.ts`, French studied, in the RAE's
// terms, and every `phrase-gloss` probe as the whole-selection card shows it
// (`word-card-french.ts`); the lines are held in a snapshot. No reader sees them before change 52
// lists a French pack. Re-blessed with `yarn vitest run test/word-card-fr-es.spec.ts -u` when the
// golden is (`lingua-pack-update` does, on its branch), and in the pull request that moves the
// wording, which says why (M9: the owner reviews every Spanish line).

const UI = { grammar, card, selection };
const ALL = probes(golden("fr-es"));

describe("the word card of a Spanish-native reader of French, over the fr-es golden", () => {
  it("renders every grammar and phrase probe as the snapshot pins it", async () => {
    // The reference's 31 grammar probes, the card's 21 and the 40 lemmas; 37 phrases and the reader's.
    expect(ALL.filter((p) => p.kind === "word-grammar")).toHaveLength(92);
    expect(ALL.filter((p) => p.kind === "phrase-gloss")).toHaveLength(38);
    await expect(pinned(ALL, UI)).toMatchFileSnapshot("./baseline/word-card-fr-es.txt");
  });

  it("A form of five readings: « parle » read as a form of « parler »", () => {
    expect(pinnedBlock(ALL, "word-grammar parle parler", UI).slice(0, 2)).toEqual([
      "### word-grammar parle parler",
      "grammar: primera y tercera persona del singular del presente de indicativo o de subjuntivo y segunda persona del singular del imperativo de parler",
    ]);
  });

  it("The passé simple in the RAE's terms, and the present participle", () => {
    expect(pinnedBlock(ALL, "word-grammar fut être", UI)[1]).toBe(
      "grammar: tercera persona del singular del pretérito perfecto simple de indicativo de être",
    );
    expect(pinnedBlock(ALL, "word-grammar parlant parler", UI)[1]).toBe("grammar: participio presente de parler");
    expect(pinnedBlock(ALL, "word-grammar parlé parler", UI)[1]).toBe("grammar: participio pasado de parler");
  });

  it("Two genders of one number, named once: « sommes » and « somme »", () => {
    expect(pinnedBlock(ALL, "word-grammar sommes être", UI)).toContain(
      "grammar: también puede ser el masculino y femenino plural de somme",
    );
  });

  it("No French form is named by a gerundio, nor by the French card's words", () => {
    const text = pinned(ALL, UI);
    expect(text).not.toMatch(/gerundio|subjonctif|indicatif|passé simple/);
  });

  it("The whole-selection card of « l’homme », and « Au revoir » answered by its expression", () => {
    expect(pinnedBlock(ALL, "phrase-gloss l’homme", UI)).toEqual([
      "### phrase-gloss l’homme",
      expect.stringMatching(/^row: homme — /),
    ]);
    expect(pinnedBlock(ALL, "phrase-gloss Au revoir", UI)[1]).toBe("expression: au revoir");
    // fr-es's table holds no `d'abord` — the Spanish Wiktionary's French section has no entry for
    // it — so « D’abord » shows its words' rows, where fr-en's shows the expression.
    expect(pinnedBlock(ALL, "phrase-gloss D’abord", UI)[1]).toMatch(/^row: abord — /);
  });
});
