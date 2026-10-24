import { describe, expect, it } from "vitest";
import { card } from "@/i18n/fr/card.ts";
import { grammar } from "@/i18n/fr/grammar.ts";
import { selection } from "@/i18n/fr/selection.ts";
import { golden, pinned, pinnedBlock, probes } from "./word-card-french.ts";

// add-lingua-italian-word-card D6: the word card of a French-native reader of Italian, pinned from
// the it-fr golden and never from the tables. Every `word-grammar` probe of
// crates/lingua-wasm/tests/baseline/it-fr.golden — the Italian baseline's 32, the card's 24 and the
// 40 lemmas — is rendered with the interface in French through `src/i18n/fr/grammar.ts`, Italian
// studied, in French school terms for Italian grammar, and every `phrase-gloss` probe as the
// whole-selection card shows it (`word-card-french.ts`); the lines are held in a snapshot. No reader
// sees them before change 20 (enable-lingua-italian) lists it-fr; the owner reads them first
// (task 4.1: the French terms, the merges, the order). Re-blessed with
// `yarn vitest run test/word-card-it-fr.spec.ts -u` when the golden is (`lingua-pack-update` does,
// on its branch), and in the pull request that moves the wording, which says why.

const UI = { grammar, card, selection };
const ITALIAN = "it";
const ALL = probes(golden("it-fr"));

/** The grammar line of a probe, its first line after the heading. */
const line = (name: string) => pinnedBlock(ALL, name, UI, ITALIAN)[1];

describe("the word card of a French-native reader of Italian, over the it-fr golden", () => {
  it("renders every grammar and phrase probe as the snapshot pins it", async () => {
    // The reference's 32 grammar probes, the card's 24 and the 40 lemmas; 48 phrases and the reader's.
    expect(ALL.filter((p) => p.kind === "word-grammar")).toHaveLength(96);
    expect(ALL.filter((p) => p.kind === "phrase-gloss")).toHaveLength(49);
    await expect(pinned(ALL, UI, ITALIAN)).toMatchFileSnapshot("./baseline/word-card-it-fr.txt");
  });

  it("The passato remoto, the imperfetto, the future and the conditional, in French school terms", () => {
    expect(line("word-grammar parlò parlare")).toBe("grammar: 3e personne du singulier du passé simple de parlare");
    expect(line("word-grammar parlavo parlare")).toBe(
      "grammar: 1re personne du singulier de l’imparfait de l’indicatif de parlare",
    );
    expect(line("word-grammar parlerò parlare")).toBe("grammar: 1re personne du singulier du futur de parlare");
    expect(line("word-grammar parlerebbe parlare")).toBe(
      "grammar: 3e personne du singulier du conditionnel de parlare",
    );
  });

  it("The indicative and the subjunctive said once (« parliamo »), and named apart where their persons differ (« parli »)", () => {
    expect(line("word-grammar parliamo parlare")).toBe(
      "grammar: 1re personne du pluriel du présent de l’indicatif ou du subjonctif et 1re personne du pluriel de l’impératif de parlare",
    );
    expect(line("word-grammar parli parlare")).toBe(
      "grammar: 2e personne du singulier du présent de l’indicatif et 1re, 2e et 3e personnes du singulier du présent du subjonctif de parlare",
    );
    expect(line("word-grammar mangiate mangiare")).toMatch(
      /2e personne du pluriel du présent de l’indicatif ou du subjonctif et 2e personne du pluriel de l’impératif de mangiare$/,
    );
  });

  it("The gerund and the participles", () => {
    expect(line("word-grammar parlando parlare")).toBe("grammar: gérondif de parlare");
    expect(line("word-grammar parlanti parlare")).toBe("grammar: participe présent de parlare");
    expect(line("word-grammar parlato parlare")).toBe("grammar: participe passé de parlare");
    expect(line("word-grammar creduta credere")).toBe("grammar: participe passé féminin de credere");
    // `parlante` is a word of its own: the present participle is a « peut aussi être » line naming
    // the verb (D6).
    expect(pinnedBlock(ALL, "word-grammar parlante parlante", UI, ITALIAN)).toContain(
      "grammar: peut aussi être le participe présent de parlare",
    );
  });

  it("A plural spelled like its dictionary form, a numeral and a determiner's plural without a gender are unnamed", () => {
    for (const name of ["word-grammar città città", "word-grammar mila mille"]) {
      expect(
        pinnedBlock(ALL, name, UI, ITALIAN).some((l) => l.startsWith("grammar: ")),
        name,
      ).toBe(false);
    }
    // `tali`'s determiner reading is unnamed; its line is the other dictionary form's, *talo*.
    expect(pinnedBlock(ALL, "word-grammar tali tale", UI, ITALIAN).filter((l) => l.startsWith("grammar: "))).toEqual([
      "grammar: peut aussi être le masculin pluriel de talo",
    ]);
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
