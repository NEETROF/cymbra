import { describe, expect, it } from "vitest";
import { grammar } from "@/i18n/it/grammar.ts";
import type { StudiedLanguageCode } from "@/i18n/index.ts";
import { golden, type Probe, probes, renderGrammar } from "./word-card-french.ts";

// add-lingua-italian-card-wording D7: the Italian renderer's grammar lines, pinned on the real packs'
// readings before any Italian reader sees them. No pack is glossed in Italian yet, but a form's
// readings are the same whatever the native (`cross_native.rs`, the baselines' `studied_side`
// tests), so every `word-grammar` probe of the en-es, es-en, fr-en and it-fr goldens — English,
// Spanish, French and Italian studied — is rendered with the interface in Italian through
// `src/i18n/it/grammar.ts`, its grammar lines only (`renderGrammar`): no gloss, no sense heading,
// which are the native side's and which no Italian reader sees. The full cards, glosses included,
// are the en-it, fr-it and es-it packs' (the Italian programme's changes 36, 47 and 51). Re-blessed
// with `yarn vitest run test/word-card-grammar-it.spec.ts -u` when one of those goldens is
// (`lingua-pack-update` does, on its branch), and in the pull request that moves the wording, which
// says why.

const GOLDENS: [pair: string, studied: StudiedLanguageCode, probes: number][] = [
  ["en-es", "en", 72],
  ["es-en", "es", 68],
  ["fr-en", "fr", 97],
  ["it-fr", "it", 96],
];
const ALL = new Map<string, Probe[]>(GOLDENS.map(([pair]) => [pair, probes(golden(pair))]));

/** The text the snapshot pins: each golden's probes, in its order, under the golden's name. */
function pinned(): string {
  return `${GOLDENS.flatMap(([pair, studied]) => [
    `## ${pair}, ${studied} studied`,
    ...ALL.get(pair)!.flatMap((probe) => renderGrammar(probe, grammar, studied)),
  ]).join("\n")}\n`;
}

/** The grammar lines of one probe of a golden. */
function lines(pair: string, name: string): string[] {
  const studied = GOLDENS.find(([p]) => p === pair)![1];
  const probe = ALL.get(pair)!.find((p) => p.name === `word-grammar ${name}`);
  expect(probe, `${pair} ${name}`).toBeDefined();
  const block = renderGrammar(probe!, grammar, studied);
  // The block is in the pinned text, whole and once.
  expect(pinned().split(/\n(?=#{2,3} )/).filter((b) => b === block.join("\n"))).toHaveLength(1);
  return block.slice(1).map((line) => line.replace(/^grammar: /, ""));
}

describe("the Italian renderer's grammar lines, over the en-es, es-en, fr-en and it-fr goldens", () => {
  it("renders every grammar probe as the snapshot pins it", async () => {
    for (const [pair, , count] of GOLDENS) {
      expect(ALL.get(pair)!.filter((p) => p.kind === "word-grammar"), pair).toHaveLength(count);
    }
    await expect(pinned()).toMatchFileSnapshot("./baseline/word-card-grammar-it.txt");
  });

  it("holds no gloss, no sense heading and no tag", () => {
    const text = pinned();
    expect(text).not.toMatch(/^(page|row|gloss|expression|\s+\[)/m);
    const grammarLines = text.split("\n").filter((l) => l.startsWith("grammar: "));
    expect(grammarLines.length).toBeGreaterThan(120);
    for (const l of grammarLines) {
      expect(l).not.toMatch(/[A-Za-z]+=|\b(Ind|Sub|Cnd|Imp|Pres|Past|Fut|Fin|Part|Ger|Inf|Sing|Plur|Masc|Fem)\b/);
    }
  });

  it("English forms, as the Italian Wiktionary names them", () => {
    expect(lines("en-es", "went go")).toEqual(["passato semplice di go"]);
    expect(lines("en-es", "running run")).toEqual(["forma in -ing di run"]);
    expect(lines("en-es", "gone go")).toEqual(["participio passato di go"]);
    expect(lines("en-es", "has have")).toEqual(["terza persona singolare del presente semplice di have"]);
    expect(lines("en-es", "read read")).toEqual(["può anche essere il passato semplice e il participio passato di read"]);
  });

  it("Spanish forms, by the Italian name of the same tense", () => {
    expect(lines("es-en", "hablaba hablar")).toEqual([
      "prima e terza persona singolare dell'indicativo imperfetto di hablar",
    ]);
    expect(lines("es-en", "rápidas rápido")).toEqual(["femminile plurale di rápido"]);
    expect(lines("es-en", "cantaríamos cantar")).toEqual(["prima persona plurale del condizionale presente di cantar"]);
  });

  it("French forms, by the Italian name of the same tense", () => {
    expect(lines("fr-en", "fut être")).toEqual(["terza persona singolare del passato remoto di être"]);
    expect(lines("fr-en", "parlant parler")).toEqual(["participio presente di parler"]);
  });

  it("Italian forms, by Treccani's names, the mood before the tense", () => {
    expect(lines("it-fr", "parlò parlare")).toEqual(["terza persona singolare del passato remoto di parlare"]);
    expect(lines("it-fr", "parlavo parlare")).toEqual([
      "prima persona singolare dell'indicativo imperfetto di parlare",
    ]);
    expect(lines("it-fr", "parlando parlare")).toEqual(["gerundio di parlare"]);
    expect(lines("it-fr", "creduta credere")).toEqual(["participio passato femminile di credere"]);
  });
});
