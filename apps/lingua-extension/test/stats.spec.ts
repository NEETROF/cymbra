import { describe, expect, it } from "vitest";
import type { CefrLevel, LevelRow } from "@/analyzer/types.ts";
import { barChartSvg } from "@/stats/chart.ts";
import { ladderHtml, vocabularyHtml } from "@/stats/ladder.ts";
import {
  buildSeries,
  consolidatedToMap,
  cumulativeTotals,
  dayWindow,
  estimatedPosition,
  groupMarkedWords,
  markedWords,
  roughCount,
} from "@/stats/model.ts";

const CEFR: readonly CefrLevel[] = ["A1", "A2", "B1", "B2", "C1", "C2"];

/** A ladder with the given band sizes, `known` of each band presumed known. */
function bands(totals: number[], known = 0): LevelRow[] {
  return totals.map((total, i) => ({ level: CEFR[i], confirmed: 0, presumed: known, toLearn: total - known, total }));
}

const fr = (n: number): string => n.toLocaleString("fr-FR");

describe("cumulativeTotals", () => {
  it("adds each level's words to those of every level below it", () => {
    expect(cumulativeTotals(bands([986, 1122, 1945, 2148, 726, 566]))).toEqual([986, 2108, 4053, 6201, 6927, 7493]);
  });

  it("is empty for an empty ladder", () => {
    expect(cumulativeTotals([])).toEqual([]);
  });
});

describe("ladderHtml", () => {
  it("shows each level's own words next to the running total up to that level", () => {
    const html = ladderHtml(bands([986, 1122, 1945, 2148, 726, 566], 10), "A2");
    expect(html).toContain(`${fr(10)} / ${fr(1122)}`); // A2's own band
    expect(html).toContain(`<span class="ladder-cum">${fr(2108)}</span>`); // A1 + A2
    expect(html).toContain(`<span class="ladder-cum">${fr(7493)}</span>`); // the whole list
    expect(html).toContain("cumulé");
    expect(html.match(/ladder-row--here/g)).toHaveLength(1);
  });

  it("explains that a level counts only its own base words", () => {
    expect(ladderHtml(bands([1, 1, 1, 1, 1, 1]), null)).toContain("Chaque niveau compte les mots qu'il introduit");
  });
});

describe("roughCount", () => {
  it("keeps two significant digits", () => {
    expect(roughCount(1285)).toBe(1300);
    expect(roughCount(15823)).toBe(16000);
    expect(roughCount(995)).toBe(1000);
  });

  it("leaves small counts as they are", () => {
    expect(roughCount(0)).toBe(0);
    expect(roughCount(99)).toBe(99);
  });
});

describe("vocabularyHtml", () => {
  it("shows the rounded estimate, the dictionary size and the confirmed words", () => {
    const html = vocabularyHtml({ estimated: 15823, confirmed: 120, universe: 25009 });
    expect(html).toContain(`≈&nbsp;${fr(16000)} mots`);
    expect(html).toContain(`${fr(25009)} mots du dictionnaire (dont ${fr(120)} confirmés)`);
  });

  it("asks for a level or marked words before there is anything to estimate", () => {
    expect(vocabularyHtml({ estimated: 0, confirmed: 0, universe: 25009 })).toContain("Pas encore d'estimation");
    expect(vocabularyHtml({ estimated: 0, confirmed: 0, universe: 0 })).toBe("");
  });

  it("omits the confirmed count when there is none", () => {
    expect(vocabularyHtml({ estimated: 3200, confirmed: 0, universe: 25009 })).not.toContain("confirmés");
  });
});

/** Build a full A1..C2 ladder; `known` gives the known fraction (0..1) per level. */
function ladder(known: Partial<Record<CefrLevel, number>>, total = 100): LevelRow[] {
  return CEFR.map((level) => {
    const confirmed = Math.round((known[level] ?? 0) * total);
    return { level, confirmed, presumed: 0, toLearn: total - confirmed, total };
  });
}

describe("estimatedPosition", () => {
  it("is the lowest level not yet cleared (>= 90% known)", () => {
    expect(estimatedPosition(ladder({ A1: 1, A2: 1, B1: 0.95, B2: 0.4, C1: 0 }))).toBe("B2");
  });

  it("counts confirmed + presumed toward mastery", () => {
    const rows = ladder({ A1: 1 });
    rows[1] = { level: "A2", confirmed: 50, presumed: 45, toLearn: 5, total: 100 }; // 95% → cleared
    expect(estimatedPosition(rows)).toBe("B1"); // first not-cleared after A1/A2
  });

  it("returns the top present level when everything is cleared", () => {
    expect(estimatedPosition(ladder({ A1: 1, A2: 1, B1: 1, B2: 1, C1: 1, C2: 1 }))).toBe("C2");
  });

  it("is null when the pack has no CEFR data (all bands empty)", () => {
    expect(estimatedPosition(ladder({}, 0))).toBeNull();
  });
});

describe("dayWindow", () => {
  it("is the inclusive window of `window` days ending at toDay", () => {
    expect(dayWindow(100, 7)).toEqual({ fromDay: 94, toDay: 100 });
    expect(dayWindow(100, 30)).toEqual({ fromDay: 71, toDay: 100 });
  });
});

describe("buildSeries", () => {
  it("zero-fills the window, aligns the metric arrays, and sums totals", () => {
    const byDay = {
      10: { exposures: 5, wordsLearned: 2, reviews: 1 },
      12: { exposures: 3, wordsLearned: 0, reviews: 4 },
    };
    const series = buildSeries(byDay, 10, 12);
    expect(series.days).toEqual([10, 11, 12]);
    expect(series.exposures).toEqual([5, 0, 3]);
    expect(series.wordsLearned).toEqual([2, 0, 0]);
    expect(series.reviews).toEqual([1, 0, 4]);
    expect(series.totals).toEqual({ exposures: 8, wordsLearned: 2, reviews: 5 });
  });

  it("produces all-zero series for an empty map", () => {
    const series = buildSeries({}, 1, 3);
    expect(series.exposures).toEqual([0, 0, 0]);
    expect(series.totals).toEqual({ exposures: 0, wordsLearned: 0, reviews: 0 });
  });
});

describe("consolidatedToMap", () => {
  it("sums consolidated rows per day", () => {
    const map = consolidatedToMap([
      { day: 5, exposures: 2, wordsLearned: 1, reviews: 0 },
      { day: 5, exposures: 3, wordsLearned: 0, reviews: 2 }, // e.g. a second language row
      { day: 6, exposures: 1, wordsLearned: 1, reviews: 1 },
    ]);
    expect(map[5]).toEqual({ exposures: 5, wordsLearned: 1, reviews: 2 });
    expect(map[6]).toEqual({ exposures: 1, wordsLearned: 1, reviews: 1 });
  });
});

describe("barChartSvg", () => {
  it("draws one rect per positive value, with a baseline and accessible label", () => {
    const svg = barChartSvg([2, 0, 5, 1], "var(--cymbra-lingua-teal)", "Mots rencontrés");
    expect(svg).toContain('role="img"');
    expect(svg).toContain('aria-label="Mots rencontrés"');
    expect((svg.match(/<rect /g) ?? []).length).toBe(3); // the zero draws no bar
    expect(svg).toContain("var(--cymbra-lingua-teal)");
  });

  it("handles an all-zero series without dividing by zero", () => {
    const svg = barChartSvg([0, 0, 0], "var(--cymbra-lingua-green)", "x");
    expect((svg.match(/<rect /g) ?? []).length).toBe(0);
    expect(svg).toContain("<line"); // baseline still drawn
  });
});

describe("markedWords", () => {
  const op = (lemma: string, status: string, updated_at: number) => ({ lemma, status, updated_at });

  it("keeps only known/ignored, newest decision first (ties alphabetical)", () => {
    const words = markedWords([
      op("run", "learning", 500), // dropped — learning lives in the deck
      op("seldom", "ignored", 300),
      op("city", "known", 300), // same ts as seldom → alphabetical: city before seldom
      op("holocene", "ignored", 900),
      op("abyss", "cleared", 950), // dropped — a withdrawn decision, already put back
    ]);
    expect(words.map((w) => `${w.lemma}:${w.status}`)).toEqual(["holocene:ignored", "city:known", "seldom:ignored"]);
  });

  it("returns an empty list when nothing is explicitly marked", () => {
    expect(markedWords([op("run", "learning", 1)])).toEqual([]);
    expect(markedWords([])).toEqual([]);
  });
});

describe("groupMarkedWords", () => {
  const op = (lemma: string, status: string, provenance: string, updated_at: number) => ({
    lemma,
    status,
    provenance,
    updated_at,
  });
  const lemmas = (words: { lemma: string }[]) => words.map((w) => w.lemma);

  it("splits the reader's decisions from reading and review confirmations, newest first", () => {
    const groups = groupMarkedWords([
      op("seldom", "ignored", "manual", 100),
      op("city", "known", "manual", 300),
      op("run", "known", "exposure", 200),
      op("cat", "known", "exposure", 400),
      op("nuance", "known", "srs", 500),
      op("quixotic", "learning", "manual", 900), // in the deck, not a marked word
      op("abyss", "cleared", "manual", 950), // already put back
    ]);
    expect(lemmas(groups.decision)).toEqual(["city", "seldom"]);
    expect(lemmas(groups.reading)).toEqual(["cat", "run"]);
    expect(lemmas(groups.review)).toEqual(["nuance"]);
    expect(groups.reading.every((w) => w.origin === "reading")).toBe(true);
  });

  it("keeps imported and unlabelled knowns, and every ignored word, with the decisions", () => {
    const groups = groupMarkedWords([
      op("holocene", "known", "import", 3),
      { lemma: "era", status: "known", updated_at: 2 },
      op("zyzzyva", "ignored", "exposure", 1),
    ]);
    expect(lemmas(groups.decision)).toEqual(["holocene", "era", "zyzzyva"]);
    expect(groups.reading).toEqual([]);
    expect(groups.review).toEqual([]);
  });
});
