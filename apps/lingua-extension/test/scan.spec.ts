import { beforeEach, describe, expect, it } from "vitest";
import type { AnalyzerPort } from "@/analyzer/port.ts";
import type { AnalyzedToken, PageAnalysis } from "@/analyzer/types.ts";
import { collectBlocks } from "@/reading/blocks.ts";
import {
  clickableByContainer,
  findTokenAt,
  findTokenInBlock,
  readingByContainer,
  resolveTokens,
  scan,
  statsFromAnalysis,
} from "@/reading/scan.ts";

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
    phraseGloss: async () => ({ tokens: [] }),
    wordGrammar: async () => ({ gloss: null, senses: [], readings: [], others: [], pieces: [] }),
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

  // add-lingua-french-tokenisation D7: the pieces of an elision have spans of their own, which
  // meet at one offset. The tokens are synthetic: no French pack reaches the extension yet.
  it("opens the piece that starts at the caret when two pieces meet there", () => {
    document.body.innerHTML = `<p>Il voit l’homme.</p>`;
    const blocks = collectBlocks();
    // Byte spans: `l’` is 4 bytes (the typographic apostrophe is 3), `homme` follows.
    const a = analysis([
      tok({ start: 8, end: 12, surface: "le", lemma: "le", class: "Unknown" }),
      tok({ start: 12, end: 17, surface: "homme", lemma: "homme", class: "Unknown" }),
    ]);
    const resolved = resolveTokens(blocks, a);
    expect(resolved.map((r) => r.range.toString())).toEqual(["l’", "homme"]);
    const textNode = document.querySelector("p")!.firstChild!;
    // The left half of `homme`'s first letter: the caret lands at the boundary.
    expect(findTokenAt(resolved, textNode, 10)!.token.lemma).toBe("homme");
    // Inside `l’`, and at its start.
    expect(findTokenAt(resolved, textNode, 9)!.token.lemma).toBe("le");
    expect(findTokenAt(resolved, textNode, 8)!.token.lemma).toBe("le");
    // The end of `homme`, before the period, still opens it.
    expect(findTokenAt(resolved, textNode, 15)!.token.lemma).toBe("homme");
  });

  it("opens the first of two pieces sharing one span, as before", () => {
    document.body.innerHTML = `<p>They don't ship.</p>`;
    const blocks = collectBlocks();
    const a = analysis([
      tok({ start: 5, end: 10, surface: "do", lemma: "do", class: "Unknown" }),
      tok({ start: 5, end: 10, surface: "not", lemma: "not", class: "Unknown" }),
    ]);
    const resolved = resolveTokens(blocks, a);
    const textNode = document.querySelector("p")!.firstChild!;
    expect(findTokenAt(resolved, textNode, 5)!.token.lemma).toBe("do");
    expect(findTokenAt(resolved, textNode, 7)!.token.lemma).toBe("do");
    expect(findTokenAt(resolved, textNode, 10)!.token.lemma).toBe("do");
  });

  it("opens a word when the caret lands right after it, before a space", () => {
    document.body.innerHTML = `<p>The runner runs.</p>`;
    const blocks = collectBlocks();
    const a = analysis([
      tok({ start: 4, end: 10, surface: "runner", lemma: "runner", class: "Unknown" }),
      tok({ start: 11, end: 15, surface: "runs", lemma: "run", class: "Unknown" }),
    ]);
    const resolved = resolveTokens(blocks, a);
    const textNode = document.querySelector("p")!.firstChild!;
    expect(findTokenAt(resolved, textNode, 10)!.token.lemma).toBe("runner");
    expect(findTokenAt(resolved, textNode, 11)!.token.lemma).toBe("run");
  });

  it("resolves the boundary of two pieces split across inline markup", () => {
    // The ranges lie in different text nodes: the comparison falls back to the DOM's.
    document.body.innerHTML = `<p>Il voit <b>l’</b>homme.</p>`;
    const blocks = collectBlocks();
    const a = analysis([
      tok({ start: 8, end: 12, surface: "le", lemma: "le", class: "Unknown" }),
      tok({ start: 12, end: 17, surface: "homme", lemma: "homme", class: "Unknown" }),
    ]);
    const resolved = resolveTokens(blocks, a);
    const elided = document.querySelector("b")!.firstChild!;
    const rest = document.querySelector("b")!.nextSibling!;
    expect(findTokenAt(resolved, rest, 0)!.token.lemma).toBe("homme");
    expect(findTokenAt(resolved, elided, 1)!.token.lemma).toBe("le");
    // The end of `l’` is no start of `homme` in the DOM's terms: `le` is the fallback.
    expect(findTokenAt(resolved, elided, 2)!.token.lemma).toBe("le");
    expect(findTokenAt(resolved, document.querySelector("p")!.firstChild!, 2)).toBeNull();
    // A node of no tree the ranges are in answers nothing.
    expect(findTokenAt(resolved, document.createTextNode("homme"), 0)).toBeNull();
  });

  it("resolves a piece whose range spans inline markup, at its start and at its end", () => {
    document.body.innerHTML = `<p>Il voit l’<b>hom</b>me.</p>`;
    const blocks = collectBlocks();
    const a = analysis([
      tok({ start: 8, end: 12, surface: "le", lemma: "le", class: "Unknown" }),
      tok({ start: 12, end: 17, surface: "homme", lemma: "homme", class: "Unknown" }),
    ]);
    const resolved = resolveTokens(blocks, a);
    expect(resolved.map((r) => r.range.toString())).toEqual(["l’", "homme"]);
    const head = document.querySelector("p")!.firstChild!;
    const inside = document.querySelector("b")!.firstChild!;
    const tail = document.querySelector("b")!.nextSibling!;
    // `homme` starts in the bold text: a click on the left half of its `h` lands the caret there.
    expect(findTokenAt(resolved, inside, 0)!.token.lemma).toBe("homme");
    expect(findTokenAt(resolved, inside, 1)!.token.lemma).toBe("homme");
    // The end of the text node before is the end of `l’`, and no start of `homme` in the DOM's
    // terms: `le` is the fallback.
    expect(findTokenAt(resolved, head, 10)!.token.lemma).toBe("le");
    // The very end of `homme`, in another node than its start, still opens it.
    expect(findTokenAt(resolved, tail, 2)!.token.lemma).toBe("homme");
    expect(findTokenAt(resolved, tail, 3)).toBeNull();
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

describe("clickableByContainer", () => {
  it("pairs each container's block with its tokens and drops proper nouns", () => {
    document.body.innerHTML = `<p>alpha</p><p>beta</p>`;
    const blocks = collectBlocks(document.body);
    const a = analysis([
      tok({ block: 0, start: 0, end: 3, surface: "run", lemma: "run", class: "Known" }),
      tok({ block: 0, start: 0, end: 5, surface: "Paris", lemma: "paris", class: "ProperNounOutOfLexicon" }),
      tok({ block: 1, start: 0, end: 4, surface: "city", lemma: "city", class: "Ignored" }),
    ]);
    const map = clickableByContainer(blocks, a);
    const first = map.get(blocks[0].container)!;
    expect(first.block).toBe(blocks[0]); // block paired with its own tokens
    expect(first.tokens.map((t) => t.lemma)).toEqual(["run"]); // proper noun dropped
    expect(map.get(blocks[1].container)!.tokens.map((t) => t.lemma)).toEqual(["city"]);
  });
});

describe("findTokenInBlock", () => {
  it("hit-tests a non-painted (Known) word by resolving its block on demand", () => {
    document.body.innerHTML = `<p>The runner runs.</p>`;
    const blocks = collectBlocks(document.body);
    const a = analysis([tok({ block: 0, start: 4, end: 10, surface: "runner", lemma: "run", class: "Known" })]);
    const { block, tokens } = clickableByContainer(blocks, a).get(blocks[0].container)!;
    const textNode = document.querySelector("p")!.firstChild!;
    const hit = findTokenInBlock(block, tokens, textNode, 6);
    expect(hit?.token.lemma).toBe("run");
    expect(hit?.range.toString()).toBe("runner");
    expect(findTokenInBlock(block, tokens, textNode, 12)).toBeNull(); // past the word
  });
});

describe("readingByContainer", () => {
  it("groups distinct lemmas by block container, skipping proper nouns", () => {
    document.body.innerHTML = `<p>alpha</p><p>beta</p>`;
    const blocks = collectBlocks(document.body);
    const a = analysis([
      tok({ block: 0, start: 0, end: 3, surface: "run", lemma: "run", class: "Unknown" }),
      tok({ block: 0, start: 0, end: 4, surface: "runs", lemma: "run", class: "Learning" }), // dup lemma
      tok({ block: 0, start: 0, end: 5, surface: "Paris", lemma: "paris", class: "ProperNounOutOfLexicon" }),
      tok({ block: 1, start: 0, end: 4, surface: "city", lemma: "city", class: "Known" }), // below-level presumed
    ]);
    const map = readingByContainer(blocks, a);
    expect(map.get(blocks[0].container)?.lemmas).toEqual(["run"]); // deduped, proper noun dropped
    expect(map.get(blocks[1].container)?.lemmas).toEqual(["city"]); // Known (presumed) still counts as read
  });

  it("counts occurrences read and, among them, the unknown and learning ones", () => {
    document.body.innerHTML = `<p>alpha</p><p>beta</p>`;
    const blocks = collectBlocks(document.body);
    const a = analysis([
      tok({ block: 0, start: 0, end: 3, surface: "run", lemma: "run", class: "Unknown" }),
      tok({ block: 0, start: 4, end: 8, surface: "runs", lemma: "run", class: "Unknown" }), // every occurrence
      tok({ block: 0, start: 9, end: 13, surface: "seek", lemma: "seek", class: "Learning" }),
      tok({ block: 0, start: 14, end: 17, surface: "the", lemma: "the", class: "Known" }),
      tok({ block: 0, start: 18, end: 21, surface: "lol", lemma: "lol", class: "Ignored" }),
      tok({ block: 0, start: 22, end: 27, surface: "Paris", lemma: "paris", class: "ProperNounOutOfLexicon" }),
      tok({ block: 1, start: 0, end: 4, surface: "city", lemma: "city", class: "Known" }),
    ]);
    const map = readingByContainer(blocks, a);
    expect(map.get(blocks[0].container)).toMatchObject({ read: 5, unknown: 3 }); // proper noun not read
    expect(map.get(blocks[1].container)).toMatchObject({ read: 1, unknown: 0 });
  });

  it("adds up to the page's counted occurrences when every block is read", () => {
    document.body.innerHTML = `<p>alpha</p><p>beta</p>`;
    const blocks = collectBlocks(document.body);
    const tokens = [
      tok({ block: 0, start: 0, end: 3, surface: "run", lemma: "run", class: "Unknown" }),
      tok({ block: 0, start: 4, end: 7, surface: "the", lemma: "the", class: "Known" }),
      tok({ block: 1, start: 0, end: 5, surface: "Paris", lemma: "paris", class: "ProperNounOutOfLexicon" }),
      tok({ block: 1, start: 6, end: 10, surface: "city", lemma: "city", class: "Learning" }),
    ];
    const a = analysis(tokens, { counted: 3, known: 1 });
    const total = [...readingByContainer(blocks, a).values()].reduce((n, r) => n + r.read, 0);
    expect(total).toBe(a.counted);
  });

  it("ignores tokens whose block index is out of range", () => {
    document.body.innerHTML = `<p>alpha</p>`;
    const blocks = collectBlocks(document.body);
    const a = analysis([tok({ block: 9, start: 0, end: 3, surface: "x", lemma: "x", class: "Unknown" })]);
    expect(readingByContainer(blocks, a).size).toBe(0);
  });
});
