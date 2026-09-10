import { beforeEach, describe, expect, it } from "vitest";
import type { AnalyzerPort } from "@/analyzer/port.ts";
import type { AnalyzedToken, PageAnalysis } from "@/analyzer/types.ts";
import { collectBlocks } from "@/reading/blocks.ts";
import { findTokenAt, resolveTokens, scan, statsFromAnalysis } from "@/reading/scan.ts";

beforeEach(() => {
  document.body.innerHTML = "";
});

function tok(
  t: Partial<AnalyzedToken> & Pick<AnalyzedToken, "start" | "end" | "surface" | "lemma" | "class">,
): AnalyzedToken {
  return { block: 0, gloss: null, ...t };
}

/** A fake port whose analyse() returns a canned analysis (the WASM is not exercised here). */
function fakePort(analysis: PageAnalysis): AnalyzerPort {
  return {
    analyse: async () => analysis,
    setCalibration: async () => {},
    setStatus: async () => {},
    gloss: async () => undefined,
  };
}

const analysis = (tokens: AnalyzedToken[], over: Partial<PageAnalysis> = {}): PageAnalysis => ({
  analyzer_version: "1.0.0",
  analysable: true,
  tokens,
  counted: 0,
  known: 0,
  percent: null,
  ...over,
});

describe("statsFromAnalysis", () => {
  it("counts occurrences and distinct forms per class", () => {
    const a = analysis(
      [
        tok({ start: 0, end: 3, surface: "run", lemma: "run", class: "Known" }),
        tok({ start: 4, end: 10, surface: "seldom", lemma: "seldom", class: "Unknown" }),
        tok({ start: 11, end: 17, surface: "seldom", lemma: "seldom", class: "Unknown" }),
        tok({ start: 18, end: 22, surface: "ship", lemma: "ship", class: "Learning" }),
      ],
      { counted: 4, known: 1, percent: 25 },
    );
    const s = statsFromAnalysis(a);
    expect(s).toMatchObject({
      analysable: true,
      percent: 25,
      counted: 4,
      known: 1,
      unknownOccurrences: 2,
      distinctUnknown: 1,
      learningOccurrences: 1,
      distinctLearning: 1,
    });
  });

  it("reports a not-analysable page", () => {
    expect(statsFromAnalysis(analysis([], { analysable: false })).analysable).toBe(false);
  });
});

describe("resolveTokens", () => {
  it("resolves only Learning/Unknown tokens to ranges", () => {
    document.body.innerHTML = `<p>The runner runs.</p>`; // "The runner runs."
    const blocks = collectBlocks();
    const a = analysis([
      tok({ start: 0, end: 3, surface: "The", lemma: "the", class: "Known" }),
      tok({ start: 4, end: 10, surface: "runner", lemma: "run", class: "Unknown" }),
    ]);
    const resolved = resolveTokens(blocks, a);
    expect(resolved).toHaveLength(1);
    expect(resolved[0]!.range.toString()).toBe("runner");
  });
});

describe("findTokenAt", () => {
  it("hit-tests a caret position inside a token range", () => {
    document.body.innerHTML = `<p>The runner runs.</p>`;
    const blocks = collectBlocks();
    const a = analysis([tok({ start: 4, end: 10, surface: "runner", lemma: "run", class: "Unknown" })]);
    const resolved = resolveTokens(blocks, a);
    const textNode = document.querySelector("p")!.firstChild!;
    expect(findTokenAt(resolved, textNode, 6)!.token.lemma).toBe("run");
    expect(findTokenAt(resolved, textNode, 12)).toBeNull(); // past the word
  });
});

describe("scan", () => {
  it("analyses the DOM through the port and resolves ranges", async () => {
    document.body.innerHTML = `<p>The runner runs.</p>`;
    const a = analysis([tok({ start: 4, end: 10, surface: "runner", lemma: "run", class: "Unknown" })], {
      counted: 3,
      known: 2,
      percent: 67,
    });
    const result = await scan(fakePort(a));
    expect(result.stats.percent).toBe(67);
    expect(result.resolved).toHaveLength(1);
    expect(result.resolved[0]!.range.toString()).toBe("runner");
  });

  it("returns a not-analysable result for an empty page without calling the port", async () => {
    let called = false;
    const port: AnalyzerPort = { ...fakePort(analysis([])), analyse: async () => ((called = true), analysis([])) };
    const result = await scan(port);
    expect(result.stats.analysable).toBe(false);
    expect(called).toBe(false);
  });
});
