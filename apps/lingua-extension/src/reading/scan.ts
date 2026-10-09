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
  /** The block container the token was read from (groups painting by block). */
  container?: Element;
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
/** What reading one block container amounts to, handed over once the reader has seen it. */
export interface BlockReading {
  /** Distinct lemmas, for the per-word exposure counters. */
  lemmas: string[];
  /** Occurrences that enter the percentage — the day's "words read". */
  read: number;
  /** Among them, the Unknown and Learning ones — the day's "new words seen". */
  unknown: number;
}

/**
 * Per block container, from the FULL analysis (every class except proper nouns — not just
 * painted tokens): its distinct lemmas and its read / new-word occurrence counts. Feeds
 * viewport-gated exposure so that below-level "presumed known" words, which are never
 * painted, still count as read when their container is on screen (add-lingua-cefr-levels,
 * slice 5c), and the daily reading stats count only what was seen
 * (refine-lingua-reading-stats). `read` follows the engine's `counted` rule, so a page
 * read to the end adds exactly its `counted`.
 */
export function readingByContainer(blocks: Block[], a: PageAnalysis): Map<Element, BlockReading> {
  const acc = new Map<Element, { lemmas: Set<string>; read: number; unknown: number }>();
  for (const token of a.tokens) {
    if (token.class === "ProperNounOutOfLexicon") continue;
    const block = blocks[token.block];
    if (!block) continue;
    let entry = acc.get(block.container);
    if (!entry) {
      entry = { lemmas: new Set(), read: 0, unknown: 0 };
      acc.set(block.container, entry);
    }
    entry.lemmas.add(token.lemma);
    entry.read++;
    if (token.class === "Unknown" || token.class === "Learning") entry.unknown++;
  }
  const out = new Map<Element, BlockReading>();
  for (const [container, e] of acc) out.set(container, { lemmas: [...e.lemmas], read: e.read, unknown: e.unknown });
  return out;
}

/** A block paired with its tokens — one entry per container, for the on-demand click path. */
export interface BlockTokens {
  block: Block;
  tokens: AnalyzedToken[];
}

/**
 * Analysed tokens grouped by their block container, each paired with the exact Block they
 * were analysed against (proper nouns dropped). Feeds the click path for words that aren't
 * painted (Known/Ignored): a click resolves only the clicked block's tokens on demand — so
 * a marked word can be reopened without holding a Range for every word on the page. Pairing
 * block+tokens from the SAME analysis is deliberate: if the DOM changed since, resolving the
 * stale block yields ranges over old nodes that no longer match the live caret (a clean
 * miss), never a wrong-word hit.
 */
export function clickableByContainer(blocks: Block[], a: PageAnalysis): Map<Element, BlockTokens> {
  const out = new Map<Element, BlockTokens>();
  for (const token of a.tokens) {
    if (token.class === "ProperNounOutOfLexicon") continue;
    const block = blocks[token.block];
    if (!block) continue;
    let entry = out.get(block.container);
    if (!entry) {
      entry = { block, tokens: [] };
      out.set(block.container, entry);
    }
    entry.tokens.push(token);
  }
  return out;
}

/**
 * Resolve one block's tokens to ranges and hit-test a caret against them — the on-demand
 * click path for a non-painted (Known/Ignored) word, so it can be reclassified. Bounded to
 * a single block's tokens.
 */
export function findTokenInBlock(
  block: Block,
  tokens: AnalyzedToken[],
  node: Node,
  offset: number,
): ResolvedToken | null {
  const resolved: ResolvedToken[] = [];
  for (const token of tokens) {
    const range = rangeForToken(block, token.start, token.end);
    if (range) resolved.push({ token, range, container: block.container });
  }
  return findTokenAt(resolved, node, offset);
}

export function resolveTokens(blocks: Block[], a: PageAnalysis): ResolvedToken[] {
  const resolved: ResolvedToken[] = [];
  for (const token of a.tokens) {
    if (!isPaintedClass(token.class)) continue;
    const block = blocks[token.block];
    if (!block) continue;
    const range = rangeForToken(block, token.start, token.end);
    if (range) resolved.push({ token, range, container: block.container });
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

/** Where a caret falls against a token's range: inside it (its start included), at its very
 *  end, or outside. */
function caretIn(range: Range, node: Node, offset: number): "inside" | "end" | "outside" {
  const { startContainer, endContainer, startOffset, endOffset } = range;
  if (startContainer === node && endContainer === node) {
    if (offset >= startOffset && offset < endOffset) return "inside";
    return offset === endOffset ? "end" : "outside";
  }
  try {
    if (range.comparePoint(node, offset) !== 0) return "outside";
  } catch {
    return "outside"; // node not comparable with this range (different tree)
  }
  return node === endContainer && offset === endOffset ? "end" : "inside";
}

/**
 * Hit-test a caret position (node + offset) against the resolved token ranges, half-open: a
 * range that starts at the caret or holds it wins over one that merely ends there, which is the
 * fallback. Two pieces of one written word may have spans of their own that meet at one offset
 * (`l’|homme`, add-lingua-french-tokenisation D7): a click on the left half of `homme`'s first
 * letter lands the caret there, and opens `homme`. A click at the very end of a word still opens
 * it, and pieces sharing one span (`don't`, `del`) open on the first, as before.
 */
export function findTokenAt(resolved: ResolvedToken[], node: Node, offset: number): ResolvedToken | null {
  let endingHere: ResolvedToken | null = null;
  for (const r of resolved) {
    const where = caretIn(r.range, node, offset);
    if (where === "inside") return r;
    if (where === "end") endingHere ??= r;
  }
  return endingHere;
}
