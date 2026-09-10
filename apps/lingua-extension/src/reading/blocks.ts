// DOM ↔ analysis bridge. The WASM engine analyses an array of text *blocks* and
// returns tokens as (block index, UTF-8 byte start, byte end). This module turns
// the live DOM into those blocks — one block per nearest block-level container, so
// the language detector and lemmatiser see whole sentences — while remembering, per
// block, which Text node each character came from, so a token's byte range maps back
// to a DOM Range for the CSS Custom Highlight API. No DOM is mutated here.

/** The host element id of the injected word popup (excluded from scanning). */
export const HOST_ID = "cymbra-lingua-host";

/** Ancestor selector: any text under these is never scanned. */
const EXCLUDED_SELECTOR = [
  "script",
  "style",
  "noscript",
  "textarea",
  "input",
  "select",
  "code",
  "pre",
  "[contenteditable]",
  "[data-cymbra-lingua-skip]",
  `#${HOST_ID}`,
].join(",");

/** Nearest "block" ancestor: text sharing one of these becomes one analysis block. */
const BLOCK_SELECTOR = [
  "p",
  "li",
  "td",
  "th",
  "dd",
  "dt",
  "blockquote",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "figcaption",
  "article",
  "section",
  "aside",
  "main",
  "header",
  "footer",
  "div",
].join(",");

/** A contiguous run of one Text node's characters within a block string. */
export interface Segment {
  node: Text;
  /** Character (UTF-16) index in the block string where this node's text starts. */
  blockStart: number;
  /** Length of this node's text in UTF-16 code units. */
  length: number;
}

/** One analysis block: the concatenated text of a container plus its segment map. */
export interface Block {
  text: string;
  segments: Segment[];
  container: Element;
}

const HAS_LETTER = /\p{L}/u;

/**
 * Collect analysis blocks under `root` (default document.body), preserving the
 * Text-node provenance of every character. Whitespace-only nodes are kept so
 * inter-word spacing survives concatenation; blocks with no letters are dropped.
 */
export function collectBlocks(root: ParentNode & Node = document.body): Block[] {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode(node: Node): number {
      const parent = (node as Text).parentElement;
      if (!parent) return NodeFilter.FILTER_REJECT;
      if (parent.closest(EXCLUDED_SELECTOR)) return NodeFilter.FILTER_REJECT;
      if (!node.nodeValue) return NodeFilter.FILTER_REJECT;
      return NodeFilter.FILTER_ACCEPT;
    },
  });

  const byContainer = new Map<Element, Block>();
  const rootEl = root instanceof Element ? root : document.body;

  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const text = n as Text;
    const value = text.nodeValue ?? "";
    const parent = text.parentElement;
    if (!parent) continue;
    const container = parent.closest(BLOCK_SELECTOR) ?? rootEl;

    let block = byContainer.get(container);
    if (!block) {
      block = { text: "", segments: [], container };
      byContainer.set(container, block);
    }
    block.segments.push({ node: text, blockStart: block.text.length, length: value.length });
    block.text += value;
  }

  return [...byContainer.values()].filter((b) => HAS_LETTER.test(b.text));
}

/** UTF-8 byte length of a single code point. */
function utf8Len(codePoint: number): number {
  if (codePoint < 0x80) return 1;
  if (codePoint < 0x800) return 2;
  if (codePoint < 0x10000) return 3;
  return 4;
}

/**
 * Convert a UTF-8 byte offset (as the engine reports) into a UTF-16 index into
 * `text` (as the DOM addresses characters). Byte offsets from the analyser always
 * land on a code-point boundary. ASCII is the fast path (identity).
 */
export function byteToCharOffset(text: string, byteOffset: number): number {
  if (byteOffset <= 0) return 0;
  let bytes = 0;
  let utf16 = 0;
  for (const ch of text) {
    if (bytes >= byteOffset) return utf16;
    bytes += utf8Len(ch.codePointAt(0)!);
    utf16 += ch.length;
  }
  return utf16;
}

/** Find the segment whose character span covers `charIndex` (end-exclusive when `end`). */
function segmentAt(block: Block, charIndex: number, end: boolean): Segment | null {
  for (const seg of block.segments) {
    const lo = seg.blockStart;
    const hi = seg.blockStart + seg.length;
    if (end ? charIndex > lo && charIndex <= hi : charIndex >= lo && charIndex < hi) return seg;
  }
  return null;
}

/**
 * Build a DOM Range for a token given its UTF-8 byte range within the block. Returns
 * null if the offsets no longer map (e.g. the DOM changed under a stale block).
 */
export function rangeForToken(block: Block, byteStart: number, byteEnd: number): Range | null {
  const charStart = byteToCharOffset(block.text, byteStart);
  const charEnd = byteToCharOffset(block.text, byteEnd);
  const startSeg = segmentAt(block, charStart, false);
  const endSeg = segmentAt(block, charEnd, true);
  if (!startSeg || !endSeg) return null;
  try {
    const range = document.createRange();
    range.setStart(startSeg.node, charStart - startSeg.blockStart);
    range.setEnd(endSeg.node, charEnd - endSeg.blockStart);
    return range;
  } catch {
    return null;
  }
}
