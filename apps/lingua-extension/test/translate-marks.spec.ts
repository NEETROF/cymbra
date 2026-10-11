import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { nativeOf, studiedOf } from "@/analyzer/pairs.ts";
import { MARKED_PAIRS } from "@/translate/markup.ts";
import type { ResultLine } from "../tool/marks/measure.mjs";
import { firstTier, judgedCounts } from "../tool/marks/tier.mjs";

// The marks measurement as committed (release-lingua-spanish-translation D2,
// measure-lingua-translation-matrix-marks, tool/marks/README.md): a pair is listed in MARKED_PAIRS
// only once its judged marks reach the programme's first tier — at least 90 % of the shown marks
// correct, at most 25 % of the selections withheld. Measured and filed per pair since
// generalise-lingua-translation-routes-by-pair D6, judged in the pair's native language; the corpus
// stays per studied language. This holds the list to the files, so a pair is never marked on
// another's measurement; the tier is tool/marks/tier.mjs's, the rule the README states, not one of
// this test's own. The checks named "structural" read the harness and the files' names, not a behaviour.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const marks = join(root, "tool", "marks");

/** The engine's marks of `judged-<pair>.tsv`, counted. */
const judged = (pair: string) => judgedCounts(readFileSync(join(marks, `judged-${pair}.tsv`), "utf8"));
/** The lines of `results-<pair>.jsonl`. */
const results = (pair: string): ResultLine[] =>
  readFileSync(join(marks, `results-${pair}.jsonl`), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line) as ResultLine);
/** Every pair with a judged file. */
const judgedPairs = () =>
  readdirSync(marks)
    .map((name) => /^judged-(.+)\.tsv$/.exec(name)?.[1])
    .filter((pair): pair is string => pair !== undefined);

describe("A pair's marks are measured before they are shown", () => {
  it("structural: Measuring again — the harness measures a pair, on the corpus of its studied language, and files results and judgments by pair", () => {
    const harness = readFileSync(join(root, "tool", "measure_marks.mjs"), "utf8");
    expect(harness).toContain("--pair");
    expect(harness).toContain("catalogue.routes[pair]");
    expect(harness).toContain("results-${pair}.jsonl");
    expect(harness).toContain("./marks/measure.mjs"); // the loop, answering a trap as the extension does (D2)
    expect(harness).toContain("stopWords(native)"); // the native language's stop words (D1)
    // The experiment only when the pair's table exists (D3) and holds a gloss (add-lingua-french-translation D6).
    expect(harness).toContain("readGlossTables(TABLES, pair, studied)");
    expect(readFileSync(join(marks, "measure.mjs"), "utf8")).toMatch(/existsSync\(glossTable\)/);
    const corpus = JSON.parse(readFileSync(join(marks, "corpus.json"), "utf8")) as {
      items: { lang: string; id: string }[];
    };
    expect(new Set(corpus.items.map((item) => item.lang))).toEqual(new Set(["en", "es", "fr"])); // per studied language
    for (const pair of MARKED_PAIRS) {
      const lines = results(pair);
      const items = corpus.items.filter((item) => item.lang === studiedOf(pair));
      expect(lines).toHaveLength(items.length);
      expect(lines.map((line) => line.id)).toEqual(items.map((item) => item.id)); // the studied language's selections, in order
      expect(judged(pair).total).toBe(lines.length);
    }
    // Nothing is filed by studied language any more.
    for (const language of ["en", "es", "fr"]) {
      expect(existsSync(join(marks, `results-${language}.jsonl`))).toBe(false);
      expect(existsSync(join(marks, `judged-${language}.tsv`))).toBe(false);
    }
  });

  it("A studied language added to the corpus: English, Spanish and French, 100 selections each, every step's three taken from one sentence (add-lingua-french-translation D4)", () => {
    const corpus = JSON.parse(readFileSync(join(marks, "corpus.json"), "utf8")) as {
      source: Record<string, { repo: string; commit: string; file: string; sha256: string }>;
      rule: string;
      items: { k: number; id: string; lang: string; word: string; start: number; end: number }[];
    };
    expect(Object.keys(corpus.source)).toEqual(["en", "es", "fr"]);
    // French PUD at the commit and sha256 add-lingua-french-forms-tables pins (its D9).
    expect(corpus.source.fr).toEqual({
      repo: "UD_French-PUD",
      commit: "db260db10fe728853c549760801229ef4e7b16e1",
      file: "fr_pud-ud-test.conllu",
      sha256: "4dfed37b83d76e77fd2e9963d0be00d723e9a010e7e2a746f8b7640c48063c10",
    });
    expect(corpus.rule).toMatch(/the next sentence \(every language\)$/);
    for (const language of ["en", "es", "fr"]) {
      expect(
        corpus.items.filter((item) => item.lang === language),
        language,
      ).toHaveLength(100);
    }
    const steps = new Map<number, typeof corpus.items>();
    for (const item of corpus.items) steps.set(item.k, [...(steps.get(item.k) ?? []), item]);
    expect([...steps.keys()]).toEqual([...Array(100).keys()]);
    for (const [k, items] of steps) {
      expect(
        items.map((item) => item.lang),
        `k ${k}`,
      ).toEqual(["en", "es", "fr"]);
      expect(new Set(items.map((item) => item.id)).size, `k ${k}`).toBe(1); // one sentence, read in three languages
    }
    // A selection is a word, letters only, never an elided piece (« l' ») or a contraction (« du »).
    for (const item of corpus.items.filter((i) => i.lang === "fr")) {
      expect(item.word).toMatch(/^\p{L}{3,}$/u);
      expect(item.end - item.start).toBe(item.word.length);
    }
    // Added, French moved no other language's selection: each step's English and Spanish items are the
    // ones every committed result of en-fr, es-fr, es-en and en-es was measured on, in order.
    for (const pair of ["en-fr", "es-fr", "es-en", "en-es"]) {
      const items = corpus.items.filter((item) => item.lang === studiedOf(pair));
      expect(
        results(pair).map((line) => [line.k, line.id, line.word]),
        pair,
      ).toEqual(items.map((item) => [item.k, item.id, item.word]));
    }
  });

  it("every result line's key is `translation`, its value the bracketed sentence without its brackets, its marks the bracketed slices (D1)", () => {
    for (const pair of MARKED_PAIRS) {
      for (const line of results(pair)) {
        expect(line).not.toHaveProperty("french");
        expect(line).toHaveProperty("translation");
        if (line.trapped) {
          expect(line.translation).toBeNull();
          expect(line.marks).toEqual([]);
          continue;
        }
        expect(line.shown?.replace(/[⟦⟧]/gu, "")).toBe(line.translation);
        expect([...(line.shown ?? "").matchAll(/⟦([^⟧]*)⟧/gu)].map((m) => m[1])).toEqual(line.marks);
      }
    }
  });

  it("The shipped pairs today: en-fr and es-fr reached the first tier on the committed judgments, and are marked", () => {
    expect(MARKED_PAIRS).toContain("en-fr");
    expect(MARKED_PAIRS).toContain("es-fr");
    for (const pair of MARKED_PAIRS) expect(firstTier(judged(pair)), pair).toBe(true);
    expect(judged("en-fr")).toEqual({ correct: 96, shown: 97, withheld: 3, total: 100 });
  });

  it("Spanish on the first tier: es-fr's judged marks are 89 of 90 correct, 10 % withheld — marked, as en-fr's are", () => {
    expect(judged("es-fr")).toEqual({ correct: 89, shown: 90, withheld: 10, total: 100 });
    expect(MARKED_PAIRS).toContain("es-fr");
  });

  it("Spanish below the first tier: short of 90 % correct or past 25 % withheld, a pair is not on the tier the list requires", () => {
    // The rule D2 fixed before the run (tool/marks/tier.mjs), on measurements that would have failed it.
    expect(firstTier({ correct: 80, shown: 90, withheld: 10, total: 100 })).toBe(false);
    expect(firstTier({ correct: 74, shown: 74, withheld: 26, total: 100 })).toBe(false);
    expect(firstTier({ correct: 81, shown: 90, withheld: 10, total: 100 })).toBe(true);
    // Counted as the README reads D2: withheld over all selections, correct over the marks shown.
    expect(judgedCounts("k\tengine\n1\tcorrect\n2\twithheld\n3\twrong\n4\tcorrect\n")).toEqual({
      correct: 2,
      shown: 3,
      withheld: 1,
      total: 4,
    });
  });

  it("A pair measured in another native language: es-en, measured on the Spanish selections and judged in English, is listed on its own figures — 94 of 96, 4 % withheld", () => {
    expect(nativeOf("es-en")).toBe("en");
    expect(judged("es-en")).toEqual({ correct: 94, shown: 96, withheld: 4, total: 100 });
    expect(firstTier(judged("es-en"))).toBe(true);
    expect(MARKED_PAIRS).toContain("es-en");
    // Its own measurement, not es-fr's: the same selections, another route, another native language.
    expect(results("es-en").map((line) => line.id)).toEqual(results("es-fr").map((line) => line.id));
    expect(judged("es-en")).not.toEqual(judged("es-fr"));
  });

  it("An English-native reader of Spanish: es-en is on the first tier, so the Spanish selection is marked in the English sentence once change 34 ships it (relay.ts follows the list)", () => {
    expect(MARKED_PAIRS).toContain("es-en");
    const english = results("es-en").find((line) => line.id === "n01016014");
    expect(english?.shown).toContain("⟦South Korean⟧");
  });

  it("A selection the engine traps on: a sentence trapped twice is recorded so and counted as withheld — none was, en-es measured 96 of 96, 4 % withheld", () => {
    // The rule: a trapped selection's row reads withheld / trapped twice, which judgedCounts counts as withheld, unchanged.
    expect(judgedCounts("k\tengine\tengine_reason\n1\tcorrect\t\n2\twithheld\ttrapped twice\n")).toEqual({
      correct: 1,
      shown: 1,
      withheld: 1,
      total: 2,
    });
    expect(judged("en-es")).toEqual({ correct: 96, shown: 96, withheld: 4, total: 100 });
    expect(firstTier(judged("en-es"))).toBe(true);
    expect(MARKED_PAIRS).toContain("en-es");
    for (const pair of ["es-en", "en-es"]) {
      const lines = results(pair);
      expect(lines.every((line) => line.trapped === false)).toBe(true);
      expect(lines.every((line) => line.alone !== null)).toBe(true); // no fragment trapped twice: every mark reconciled
    }
    // The experiment is filled only where the pair's gloss table is committed (D3): es-en's is, en-es's is not yet.
    expect(results("es-en").every((line) => line.gloss !== null)).toBe(true);
    expect(results("en-es").every((line) => line.gloss === null)).toBe(true);
  });

  it("A French selection, judged in English: fr-en, measured on the French selections through the fr-en model, is listed on its own figures — 95 of 96, 4 % withheld (add-lingua-french-translation D6)", () => {
    expect(nativeOf("fr-en")).toBe("en");
    expect(judged("fr-en")).toEqual({ correct: 95, shown: 96, withheld: 4, total: 100 });
    expect(firstTier(judged("fr-en"))).toBe(true);
    expect(MARKED_PAIRS).toContain("fr-en");
    // The French selection marked in the English sentence: « A travers » → « Throughout », an expression's rendering.
    const english = results("fr-en").find((line) => line.k === 6);
    expect(english?.word).toBe("travers");
    expect(english?.shown).toMatch(/^⟦Throughout⟧ history/u);
  });

  it("French through English, judged in Spanish: fr-es, the same French selections through fr-en then en-es, is listed on its own figures — 90 of 91, 9 % withheld, whatever fr-en's are", () => {
    expect(nativeOf("fr-es")).toBe("es");
    expect(judged("fr-es")).toEqual({ correct: 90, shown: 91, withheld: 9, total: 100 });
    expect(firstTier(judged("fr-es"))).toBe(true);
    expect(MARKED_PAIRS).toContain("fr-es");
    // Its own measurement: the same selections as fr-en, another route, another native language.
    expect(results("fr-es").map((line) => line.id)).toEqual(results("fr-en").map((line) => line.id));
    expect(judged("fr-es")).not.toEqual(judged("fr-en"));
    // No request trapped, every fragment reconciled, one judge — the doubtful lines named, and the tier
    // held with them counted wrong (fr-en k 39 and k 92: 93 / 96; fr-es k 25: 89 / 91).
    for (const pair of ["fr-en", "fr-es"]) {
      const lines = results(pair);
      expect(lines.every((line) => line.trapped === false && line.alone !== null && line.alone !== "")).toBe(true);
      // The experiment left empty: no French gloss table is committed yet (D6).
      expect(lines.every((line) => line.gloss === null)).toBe(true);
      const rows = readFileSync(join(marks, `judged-${pair}.tsv`), "utf8")
        .trim()
        .split("\n")
        .slice(1);
      const doubtful = rows
        .filter((row) => row.split("\t")[6]!.startsWith("doubtful"))
        .map((row) => row.split("\t")[0]);
      expect(doubtful, pair).toEqual(pair === "fr-en" ? ["39", "92"] : ["25"]);
      const strict = { ...judged(pair), correct: judged(pair).correct - doubtful.length };
      expect(firstTier(strict), pair).toBe(true);
    }
  });

  it("structural: the list is exactly the judged pairs on the first tier — a pair with no judged file, or short of the tier, is not listed", () => {
    expect(MARKED_PAIRS).toEqual(["en-fr", "es-fr", "es-en", "en-es", "fr-en", "fr-es"]);
    expect(new Set(judgedPairs())).toEqual(new Set(MARKED_PAIRS));
    for (const pair of judgedPairs()) expect(MARKED_PAIRS.includes(pair), pair).toBe(firstTier(judged(pair)));
    expect(existsSync(join(marks, "judged-de-fr.tsv"))).toBe(false);
    expect(MARKED_PAIRS).not.toContain("de-fr");
  });
});
