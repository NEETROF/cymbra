import type { AnalyzerPort } from "../analyzer/port.ts";
import { type AnalyzedToken, isPaintedClass, type PageAnalysis } from "../analyzer/types.ts";
import { type Block, collectBlocks, rangeForToken } from "./blocks.ts";

// Orchestrates one analysis pass: DOM → blocks → AnalyzerPort → resolved token
// ranges + page stats. This layer is pure w.r.t. the highlight API (it only builds
// Ranges, which jsdom supports), so it is unit-tested with a fake AnalyzerPort;
// the CSS.highlights painting lives in highlight.ts.

/** A token resolved to a live DOM Range (only Learning/Unknown are resolved). */
export interface ResolvedToken {
  token: AnalyzedToken;
  range: Range;
}

/** Page-level counts the badge and popup display. */
export interface ScanStats {
  analysable: boolean;
  /** Known-token percentage 0–100, or null when nothing was counted. */
  percent: number | null;
  /** Occurrences that entered the percentage. */
  counted: number;
  /** Occurrences counting as known. */
  known: number;
  unknownOccurrences: number;
  learningOccurrences: number;
  /** Distinct dictionary forms (never the word "lemma" in any UI that shows these). */
  distinctUnknown: number;
  distinctLearning: number;
}

export interface ScanResult {
  blocks: Block[];
  resolved: ResolvedToken[];
  stats: ScanStats;
}

/** Stats for a page that could not be analysed (too little studied-language text). */
export function notAnalysableStats(): ScanStats {
  return {
    analysable: false,
    percent: null,
    counted: 0,
    known: 0,
    unknownOccurrences: 0,
    learningOccurrences: 0,
    distinctUnknown: 0,
    distinctLearning: 0,
  };
}

/** Derive the display stats from a PageAnalysis (pure). */
export function statsFromAnalysis(a: PageAnalysis): ScanStats {
  if (!a.analysable) return notAnalysableStats();
  const distinctUnknown = new Set<string>();
  const distinctLearning = new Set<string>();
  let unknownOccurrences = 0;
  let learningOccurrences = 0;
  for (const t of a.tokens) {
    if (t.class === "Unknown") {
      unknownOccurrences++;
      distinctUnknown.add(t.lemma);
    } else if (t.class === "Learning") {
      learningOccurrences++;
      distinctLearning.add(t.lemma);
    }
  }
  return {
    analysable: true,
    percent: a.percent,
    counted: a.counted,
    known: a.known,
    unknownOccurrences,
    learningOccurrences,
    distinctUnknown: distinctUnknown.size,
    distinctLearning: distinctLearning.size,
  };
}

/** Resolve the paintable tokens (Learning/Unknown) to DOM Ranges (pure). */
/**
 * Distinct lemmas per block container, from the FULL analysis (every class except
 * proper nouns — not just painted tokens). Feeds viewport-gated exposure so that
 * below-level "presumed known" words, which are never painted, still count as read
 * when their container is on screen (add-lingua-cefr-levels, slice 5c).
 */
export function lemmasByContainer(blocks: Block[], a: PageAnalysis): Map<Element, string[]> {
  const sets = new Map<Element, Set<string>>();
  for (const token of a.tokens) {
    if (token.class === "ProperNounOutOfLexicon") continue;
    const block = blocks[token.block];
    if (!block) continue;
    let set = sets.get(block.container);
    if (!set) {
      set = new Set();
      sets.set(block.container, set);
    }
    set.add(token.lemma);
  }
  const out = new Map<Element, string[]>();
  for (const [container, set] of sets) out.set(container, [...set]);
  return out;
}

export function resolveTokens(blocks: Block[], a: PageAnalysis): ResolvedToken[] {
  const resolved: ResolvedToken[] = [];
  for (const token of a.tokens) {
    if (!isPaintedClass(token.class)) continue;
    const block = blocks[token.block];
    if (!block) continue;
    const range = rangeForToken(block, token.start, token.end);
    if (range) resolved.push({ token, range });
  }
  return resolved;
}

/** Run one full analysis pass over `root`. */
export async function scan(port: AnalyzerPort, root: ParentNode & Node = document.body): Promise<ScanResult> {
  const blocks = collectBlocks(root);
  if (blocks.length === 0) {
    return { blocks, resolved: [], stats: notAnalysableStats() };
  }
  const analysis = await port.analyse(blocks.map((b) => b.text));
  const resolved = resolveTokens(blocks, analysis);
  return { blocks, resolved, stats: statsFromAnalysis(analysis) };
}

/** Hit-test a caret position (node + offset) against the resolved token ranges. */
export function findTokenAt(resolved: ResolvedToken[], node: Node, offset: number): ResolvedToken | null {
  for (const r of resolved) {
    const { startContainer, endContainer, startOffset, endOffset } = r.range;
    if (startContainer === node && endContainer === node) {
      if (offset >= startOffset && offset <= endOffset) return r;
      continue;
    }
    try {
      if (r.range.comparePoint(node, offset) === 0) return r;
    } catch {
      // node not comparable with this range (different tree) — skip.
    }
  }
  return null;
}
