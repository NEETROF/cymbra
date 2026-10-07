// What a pair's judged marks decide — decision D2 of release-lingua-spanish-translation, as
// README.md states it. Here, beside the judgments, so that test/translate-marks.spec.ts holds
// MARKED_PAIRS (src/translate/markup.ts) to the rule the README states, and not to a rule of its own.

/**
 * The engine's verdicts of a `judged-<pair>.tsv` text, counted as the README reads D2: correct and
 * withheld, the marks shown (every selection not withheld), and all selections.
 */
export function judgedCounts(tsv) {
  const [header, ...rows] = tsv.trim().split("\n");
  const engine = header.split("\t").indexOf("engine");
  const verdicts = rows.map((row) => row.split("\t")[engine]);
  const withheld = verdicts.filter((v) => v === "withheld").length;
  return {
    correct: verdicts.filter((v) => v === "correct").length,
    shown: verdicts.length - withheld,
    withheld,
    total: verdicts.length,
  };
}

/** D2's first tier: at least 90 % of the shown marks correct, and at most 25 % of all selections withheld. */
export function firstTier({ correct, shown, withheld, total }) {
  return correct / shown >= 0.9 && withheld / total <= 0.25;
}
