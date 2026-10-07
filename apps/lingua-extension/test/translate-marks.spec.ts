import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { studiedOf } from "@/analyzer/pairs.ts";
import { MARKED_PAIRS } from "@/translate/markup.ts";
import { firstTier, judgedCounts } from "../tool/marks/tier.mjs";

// The marks measurement as committed (release-lingua-spanish-translation D2, tool/marks/README.md):
// a pair is listed in MARKED_PAIRS only once its judged marks reach the programme's first tier —
// at least 90 % of the shown marks correct, at most 25 % of the selections withheld. Measured and
// filed per pair since generalise-lingua-translation-routes-by-pair D6; the corpus stays per
// studied language. This holds the list to the files, so a pair is never marked on another's
// measurement; the tier is tool/marks/tier.mjs's, the rule the README states, not one of this
// test's own. The two checks named "structural" read the harness and the files' names, not a
// behaviour.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const marks = join(root, "tool", "marks");

/** The engine's marks of `judged-<pair>.tsv`, counted. */
const judged = (pair: string) => judgedCounts(readFileSync(join(marks, `judged-${pair}.tsv`), "utf8"));

describe("A pair's marks are measured before they are shown", () => {
  it("structural: Measuring again — the harness measures a pair, on the corpus of its studied language, and files results and judgments by pair", () => {
    const harness = readFileSync(join(root, "tool", "measure_marks.mjs"), "utf8");
    expect(harness).toContain("--pair");
    expect(harness).toContain("catalogue.routes[pair]");
    expect(harness).toContain("results-${pair}.jsonl");
    const corpus = JSON.parse(readFileSync(join(marks, "corpus.json"), "utf8")) as { items: { lang: string }[] };
    expect(new Set(corpus.items.map((item) => item.lang))).toEqual(new Set(["en", "es"])); // per studied language
    for (const pair of MARKED_PAIRS) {
      const results = readFileSync(join(marks, `results-${pair}.jsonl`), "utf8")
        .trim()
        .split("\n");
      expect(results).toHaveLength(corpus.items.filter((item) => item.lang === studiedOf(pair)).length);
      expect(judged(pair).total).toBe(results.length);
    }
    // Nothing is filed by studied language any more.
    for (const language of ["en", "es"]) {
      expect(existsSync(join(marks, `results-${language}.jsonl`))).toBe(false);
      expect(existsSync(join(marks, `judged-${language}.tsv`))).toBe(false);
    }
  });

  it("The shipped pairs today: en-fr and es-fr reached the first tier on the committed judgments, and are the pairs marked", () => {
    expect(MARKED_PAIRS).toEqual(["en-fr", "es-fr"]);
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

  it("structural: a pair with no judged file is not listed — its selection is translated without a mark (relay.ts)", () => {
    for (const pair of ["es-en", "en-es"]) {
      expect(existsSync(join(marks, `judged-${pair}.tsv`))).toBe(false);
      expect(MARKED_PAIRS).not.toContain(pair);
    }
  });
});
