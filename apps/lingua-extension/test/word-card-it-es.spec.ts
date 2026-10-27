import { describe, expect, it } from "vitest";
import { card } from "@/i18n/es/card.ts";
import { grammar } from "@/i18n/es/grammar.ts";
import { selection } from "@/i18n/es/selection.ts";
import { golden, pinned, pinnedBlock, probes } from "./word-card-french.ts";

// add-lingua-pack-it-es D11, add-lingua-italian-word-card D8: the word card of a Spanish-native reader
// of Italian, pinned from the it-es golden and never from the tables. Every `word-grammar` probe of
// crates/lingua-wasm/tests/baseline/it-es.golden — the Italian baseline's 32, the card's 24 and the 40
// lemmas — is rendered with the interface in Spanish through `src/i18n/es/grammar.ts`, Italian studied,
// and every `phrase-gloss` probe as the whole-selection card shows it (`word-card-french.ts`); the lines
// are held in a snapshot. No reader saw them before change 30 (enable-lingua-italian-for-spanish-speakers)
// listed it-es; the owner reads them first (task 6.1). Re-blessed with `yarn vitest run test/word-card-it-es.spec.ts -u` when the golden is
// (`lingua-pack-update` does, on its branch), and in the pull request that moves the wording, which says
// why (M9: the owner reviews every Spanish line).

const UI = { grammar, card, selection };
const ITALIAN = "it";
const ALL = probes(golden("it-es"));

/** The grammar line of a probe, its first line after the heading. */
const line = (name: string) => pinnedBlock(ALL, name, UI, ITALIAN)[1];

describe("the word card of a Spanish-native reader of Italian, over the it-es golden", () => {
  it("renders every grammar and phrase probe as the snapshot pins it", async () => {
    // The reference's 32 grammar probes, the card's 24 and the 40 lemmas; 48 phrases and the reader's.
    expect(ALL.filter((p) => p.kind === "word-grammar")).toHaveLength(96);
    expect(ALL.filter((p) => p.kind === "phrase-gloss")).toHaveLength(49);
    await expect(pinned(ALL, UI, ITALIAN)).toMatchFileSnapshot("./baseline/word-card-it-es.txt");
  });

  it("The passato remoto and the imperfect in the RAE's terms", () => {
    expect(line("word-grammar parlò parlare")).toBe(
      "grammar: tercera persona del singular del pretérito perfecto simple de indicativo de parlare",
    );
    expect(line("word-grammar parlavo parlare")).toBe(
      "grammar: primera persona del singular del pretérito imperfecto de indicativo de parlare",
    );
  });

  it("The indicative and the subjunctive said once (« parliamo »), and named apart where their persons differ (« parli »)", () => {
    expect(line("word-grammar parliamo parlare")).toBe(
      "grammar: primera persona del plural del presente de indicativo o de subjuntivo y primera persona del plural del imperativo de parlare",
    );
    expect(line("word-grammar parli parlare")).toBe(
      "grammar: segunda persona del singular del presente de indicativo y primera, segunda y tercera persona del singular del presente de subjuntivo de parlare",
    );
  });

  it("The gerund and a participle read as a word of its own", () => {
    expect(line("word-grammar parlando parlare")).toBe("grammar: gerundio de parlare");
    expect(pinnedBlock(ALL, "word-grammar parlante parlante", UI, ITALIAN)).toContain(
      "grammar: también puede ser el participio presente de parlare",
    );
  });

  it("No line shows a tag", () => {
    const grammarLines = pinned(ALL, UI, ITALIAN)
      .split("\n")
      .filter((l) => l.startsWith("grammar: "));
    expect(grammarLines.length).toBeGreaterThan(50);
    for (const l of grammarLines) {
      expect(l).not.toMatch(/[A-Za-z]+=|\b(Ind|Sub|Cnd|Imp|Pres|Past|Fut|Fin|Part|Ger|Inf|Sing|Plur|Masc|Fem)\b/);
    }
  });

  it("« d'accordo » answered by its expression, the whole selection covered", () => {
    expect(pinnedBlock(ALL, "phrase-gloss d'accordo", UI, ITALIAN)[1]).toBe("expression: d'accordo");
  });
});
