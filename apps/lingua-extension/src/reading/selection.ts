// Selection capture (the keyboard shortcut, task 1.6): turn the current selection —
// a word or a bounded phrase — into popup content with its source sentence extracted,
// so "+ Deck" can create a phrase card in context.

const SENTENCE_SPLIT = /(?<=[.!?])\s+/;
const BLOCK_SELECTOR = "p,li,td,th,blockquote,h1,h2,h3,h4,h5,h6,figcaption,article,section,div";

/** The maximum selection length treated as a capturable phrase. */
export const MAX_SELECTION_LENGTH = 120;

/** Extract the sentence around `needle` within the block that contains `node`. */
export function sentenceAround(node: Node | null, needle: string): string {
  if (!node) return "";
  const element = node.nodeType === Node.TEXT_NODE ? node.parentElement : (node as Element);
  const block = element?.closest?.(BLOCK_SELECTOR) ?? element ?? null;
  const full = (block?.textContent ?? node.textContent ?? "").replace(/\s+/g, " ").trim();
  if (!full) return "";
  const parts = full.split(SENTENCE_SPLIT);
  const hit = parts.find((p) => p.includes(needle));
  return (hit ?? full).trim();
}

/** A captured selection: the text, its source sentence, and an anchor rectangle. */
export interface Capture {
  text: string;
  sentence: string;
  rect: { left: number; bottom: number };
}

/**
 * Read the current selection into a Capture, or null when there is nothing usable
 * (empty, or longer than a phrase). Whitespace is collapsed.
 */
/** A "word" character for snapping: letters, digits, apostrophes, hyphens — so a
 *  selection that stops mid-word ("the parity pro|of") extends to the whole word. */
const WORD_CHAR = /[\p{L}\p{N}'’-]/u;

/** Extend a range outward so both ends land on whole-word boundaries. Each endpoint is
 *  snapped within its own text node — enough for the common in-paragraph selection. */
function snapRangeToWords(range: Range): void {
  const start = range.startContainer;
  if (start.nodeType === Node.TEXT_NODE) {
    const t = start.textContent ?? "";
    let i = range.startOffset;
    while (i > 0 && WORD_CHAR.test(t.charAt(i - 1))) i--;
    if (i !== range.startOffset) range.setStart(start, i);
  }
  const end = range.endContainer;
  if (end.nodeType === Node.TEXT_NODE) {
    const t = end.textContent ?? "";
    let j = range.endOffset;
    while (j < t.length && WORD_CHAR.test(t.charAt(j))) j++;
    if (j !== range.endOffset) range.setEnd(end, j);
  }
}

export function captureSelection(maxLength: number = MAX_SELECTION_LENGTH): Capture | null {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return null;
  const range = sel.getRangeAt(0).cloneRange();
  snapRangeToWords(range);
  const text = range.toString().trim().replace(/\s+/g, " ");
  if (!text || text.length > maxLength) return null;
  // Reflect the snap on the page so the highlight matches the captured card.
  try {
    sel.removeAllRanges();
    sel.addRange(range);
  } catch {
    // Some nodes reject re-selection; the capture itself is still correct.
  }
  const box = typeof range.getBoundingClientRect === "function" ? range.getBoundingClientRect() : null;
  const rect = { left: box?.left ?? 0, bottom: box?.bottom ?? 0 };
  const firstWord = text.split(" ")[0] ?? text;
  const sentence = sentenceAround(range.startContainer, firstWord);
  return { text, sentence, rect };
}
