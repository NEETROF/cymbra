import { describe, expect, it } from "vitest";
import { card } from "@/i18n/en/card.ts";
import { grammar } from "@/i18n/en/grammar.ts";
import { selection } from "@/i18n/en/selection.ts";
import { golden, pinned, pinnedBlock, probes } from "./word-card-french.ts";

// add-lingua-french-word-card D9: the word card of an English-native reader of French, pinned from
// the fr-en golden and never from the tables. Every `word-grammar` probe of
// crates/lingua-wasm/tests/baseline/fr-en.golden — the French baseline's 31, the card's 21 and the
// 40 lemmas — is rendered with the interface in English through `src/i18n/en/grammar.ts`, French
// studied, and every `phrase-gloss` probe as the whole-selection card shows it
// (`word-card-french.ts`); the lines are held in a snapshot. No reader sees them before change 52
// lists a French pack. Re-blessed with `yarn vitest run test/word-card-fr-en.spec.ts -u` when the
// golden is (`lingua-pack-update` does, on its branch), and in the pull request that moves the
// wording, which says why (M9: the owner reviews every English line).

const UI = { grammar, card, selection };
const ALL = probes(golden("fr-en"));

describe("the word card of an English-native reader of French, over the fr-en golden", () => {
  it("renders every grammar and phrase probe as the snapshot pins it", async () => {
    // The reference's 31 grammar probes, the card's 21 and the 40 lemmas; 33 phrases and the reader's.
    expect(ALL.filter((p) => p.kind === "word-grammar")).toHaveLength(92);
    expect(ALL.filter((p) => p.kind === "phrase-gloss")).toHaveLength(34);
    await expect(pinned(ALL, UI)).toMatchFileSnapshot("./baseline/word-card-fr-en.txt");
  });

  it("A form of five readings: « parle » read as a form of « parler »", () => {
    expect(pinnedBlock(ALL, "word-grammar parle parler", UI).slice(0, 2)).toEqual([
      "### word-grammar parle parler",
      "grammar: first- and third-person singular present indicative or subjunctive and second-person singular imperative of parler",
    ]);
  });

  it("The passé simple, the present participle and the indicative before the subjunctive", () => {
    expect(pinnedBlock(ALL, "word-grammar fut être", UI)[1]).toBe(
      "grammar: third-person singular past historic (passé simple) of être",
    );
    expect(pinnedBlock(ALL, "word-grammar parlant parler", UI)[1]).toBe("grammar: present participle of parler");
    expect(pinnedBlock(ALL, "word-grammar finissions finir", UI)[1]).toBe(
      "grammar: first-person plural imperfect indicative or subjunctive and first-person plural present subjunctive of finir",
    );
  });

  it("A plural spelled like its dictionary form gives no line; a numeral and a genderless plural none", () => {
    for (const name of [
      "word-grammar temps temps",
      "word-grammar heureux heureux",
      "word-grammar millions million",
      "word-grammar les le",
    ]) {
      expect(
        pinnedBlock(ALL, name, UI).some((line) => line.startsWith("grammar: ")),
        name,
      ).toBe(false);
    }
  });

  it("« été été » and « porte porte » read neither être's nor porter's readings (fix-lingua-lemma-lookup)", () => {
    for (const name of ["word-grammar été été", "word-grammar porte porte"]) {
      const lines = pinnedBlock(ALL, name, UI).join("\n");
      expect(lines, name).not.toMatch(/of (être|porter)$|to be|to carry/m);
    }
  });

  it("The whole-selection card of « l’homme », and « D’abord » answered by its expression", () => {
    expect(pinnedBlock(ALL, "phrase-gloss l’homme", UI)).toEqual([
      "### phrase-gloss l’homme",
      expect.stringMatching(/^row: homme — /),
    ]);
    expect(pinnedBlock(ALL, "phrase-gloss D’abord", UI)[1]).toBe("expression: d'abord");
    // The expression `jusqu'à` takes the place of its words, the article of « au » included (D12).
    expect(pinnedBlock(ALL, "phrase-gloss jusqu'au soir", UI).map((line) => line.split(" — ")[0])).toEqual([
      "### phrase-gloss jusqu'au soir",
      "row: jusqu'à",
      "row: soir",
    ]);
  });
});
