import { describe, expect, it } from "vitest";
import { barChartSvg } from "@/stats/chart.ts";
import { buildSeries, consolidatedToMap, dayWindow } from "@/stats/model.ts";

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
