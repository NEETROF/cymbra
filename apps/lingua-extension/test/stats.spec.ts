import { describe, expect, it } from "vitest";
import type { CefrLevel, LevelRow } from "@/analyzer/types.ts";
import { barChartSvg } from "@/stats/chart.ts";
import { buildSeries, consolidatedToMap, dayWindow, estimatedPosition } from "@/stats/model.ts";

const CEFR: readonly CefrLevel[] = ["A1", "A2", "B1", "B2", "C1", "C2"];

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
