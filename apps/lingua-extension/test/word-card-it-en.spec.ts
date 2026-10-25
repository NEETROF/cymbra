import { describe, expect, it } from "vitest";
import { card } from "@/i18n/en/card.ts";
import { grammar } from "@/i18n/en/grammar.ts";
import { selection } from "@/i18n/en/selection.ts";
import { golden, pinned, pinnedBlock, probes } from "./word-card-french.ts";

// add-lingua-pack-it-en D9, add-lingua-italian-word-card D8: the word card of an English-native reader
// of Italian, pinned from the it-en golden and never from the tables. Every `word-grammar` probe of
// crates/lingua-wasm/tests/baseline/it-en.golden — the Italian baseline's 32, the card's 24 and the 40
// lemmas — is rendered with the interface in English through `src/i18n/en/grammar.ts`, Italian
// studied, and every `phrase-gloss` probe as the whole-selection card shows it (`word-card-french.ts`);
// the lines are held in a snapshot. No reader sees them before change 25 lists it-en; the owner reads
// them first (task 6.1). Re-blessed with `yarn vitest run test/word-card-it-en.spec.ts -u` when the
// golden is (`lingua-pack-update` does, on its branch), and in the pull request that moves the
// wording, which says why (M9: the owner reviews every English line).

const UI = { grammar, card, selection };
const ITALIAN = "it";
const ALL = probes(golden("it-en"));

/** The grammar line of a probe, its first line after the heading. */
const line = (name: string) => pinnedBlock(ALL, name, UI, ITALIAN)[1];

describe("the word card of an English-native reader of Italian, over the it-en golden", () => {
  it("renders every grammar and phrase probe as the snapshot pins it", async () => {
    // The reference's 32 grammar probes, the card's 24 and the 40 lemmas; 48 phrases and the reader's.
    expect(ALL.filter((p) => p.kind === "word-grammar")).toHaveLength(96);
    expect(ALL.filter((p) => p.kind === "phrase-gloss")).toHaveLength(49);
    await expect(pinned(ALL, UI, ITALIAN)).toMatchFileSnapshot("./baseline/word-card-it-en.txt");
  });

  it("The passato remoto, the imperfect, the future and the conditional, in English", () => {
    expect(line("word-grammar parlò parlare")).toBe(
      "grammar: third-person singular past historic (passato remoto) of parlare",
    );
    expect(line("word-grammar parlavo parlare")).toBe("grammar: first-person singular imperfect indicative of parlare");
    expect(line("word-grammar parlerò parlare")).toBe("grammar: first-person singular future of parlare");
    expect(line("word-grammar parlerebbe parlare")).toBe("grammar: third-person singular conditional of parlare");
  });

  it("The indicative and the subjunctive said once (« parliamo »), and named apart where their persons differ (« parli »)", () => {
    expect(line("word-grammar parliamo parlare")).toBe(
      "grammar: first-person plural present indicative or subjunctive and first-person plural imperative of parlare",
    );
    expect(line("word-grammar parli parlare")).toBe(
      "grammar: second-person singular present indicative and first-, second- and third-person singular present subjunctive of parlare",
    );
  });

  it("The gerund and the participles", () => {
    expect(line("word-grammar parlando parlare")).toBe("grammar: gerund of parlare");
    expect(line("word-grammar parlanti parlare")).toBe("grammar: present participle of parlare");
    expect(line("word-grammar parlato parlare")).toBe("grammar: past participle of parlare");
    expect(line("word-grammar creduta credere")).toBe("grammar: feminine past participle of credere");
    expect(pinnedBlock(ALL, "word-grammar parlante parlante", UI, ITALIAN)).toContain(
      "grammar: may also be the present participle of parlare",
    );
  });

  it("A plural spelled like its dictionary form and a numeral are unnamed", () => {
    for (const name of ["word-grammar città città", "word-grammar mila mille"]) {
      expect(
        pinnedBlock(ALL, name, UI, ITALIAN).some((l) => l.startsWith("grammar: ")),
        name,
      ).toBe(false);
    }
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

  it("it-en's own readings of the section: « del » read from its contraction, « io » without the moon", () => {
    expect(pinnedBlock(ALL, "word-grammar del del", UI, ITALIAN)).toContain("row: some, of the, from the");
    expect(pinnedBlock(ALL, "word-grammar io io", UI, ITALIAN).join("\n")).not.toMatch(/\bIo\b/);
  });

  it("« d'accordo » answered by its expression, the whole selection covered", () => {
    expect(pinnedBlock(ALL, "phrase-gloss d'accordo", UI, ITALIAN)[1]).toBe("expression: d'accordo");
  });
});
