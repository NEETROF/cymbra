// Selection capture: turn the current selection — a word or a bounded phrase — into popup
// content with its source sentence extracted, so "+ Deck" can create a phrase card in
// context. The selection itself is the trigger, on every pointer: SelectionWatcher debounces
// `selectionchange` (the one event a mouse drag, a keyboard shortcut and a touch handle drag
// all emit) and classifies what settled. The reader never clears, re-applies or suppresses
// the page selection — on iOS the platform's own press-and-hold IS the selection gesture, and
// fighting it is what made phrase capture unreachable on a phone.

const SENTENCE_SPLIT = /(?<=[.!?])\s+/;
const BLOCK_SELECTOR = "p,li,td,th,blockquote,h1,h2,h3,h4,h5,h6,figcaption,article,section,div";

/** The maximum selection length treated as a capturable phrase. */
export const MAX_SELECTION_LENGTH = 120;

/** Time (ms) a selection must hold still before it counts as settled. */
export const SELECTION_SETTLE_MS = 350;

/** The same, while a pointer is still down. A slow drag pauses for longer than 350 ms all
 *  the time, and popping the card up mid-gesture would be worse than waiting; the lift
 *  captures immediately anyway. It stays a delay rather than a block because a native
 *  selection-handle drag may never send the lift at all. */
export const SELECTION_HELD_MS = 1200;

/** Collapse runs of whitespace WITHOUT trimming: every offset below counts on this text. */
const collapse = (text: string): string => text.replace(/\s+/g, " ");

/** The block a node belongs to — the unit a sentence is looked for in. */
function blockOf(node: Node | null): Element | null {
  if (!node) return null;
  const element = node.nodeType === Node.TEXT_NODE ? node.parentElement : (node as Element);
  return element?.closest?.(BLOCK_SELECTOR) ?? element ?? null;
}

/**
 * The sentence a range sits in, found by its POSITION in the block rather than by looking
 * the words up in the text.
 *
 * Searching for the words is what it used to do, and a word that occurs inside an earlier
 * one wins that search: in "Check the input first. Then put it away.", selecting `put`
 * matched `input` and the card kept the wrong sentence — which is then synchronised as the
 * card's own content. Offsets cannot be fooled that way. A range that leaves its block is
 * clamped to the block it starts in, which is the sentence the reader pointed at.
 */
export function sentenceForRange(range: Range): string {
  return sentenceAndSelection(range).sentence;
}

/** A sentence, and where in it the selection sits — as [start, end) offsets into `sentence`. */
export interface SentenceSelection {
  sentence: string;
  /**
   * The selection's span in `sentence`, tightened to its text (no surrounding blanks), or null
   * when its position could not be established — a range with no block, or from another tree.
   */
  selection: { start: number; end: number } | null;
}

/**
 * `sentenceForRange`, keeping the offsets it finds on the way. The translator marks the
 * selection in its sentence by POSITION, for the same reason the sentence itself is found by
 * position: a word occurring earlier in the sentence (`put` inside `input`) would otherwise
 * take the mark.
 */
export function sentenceAndSelection(range: Range): SentenceSelection {
  const block = blockOf(range.startContainer);
  const full = collapse(block?.textContent ?? range.startContainer.textContent ?? "");
  if (!full.trim()) return { sentence: "", selection: null };
  if (!block) return { sentence: full.trim(), selection: null };

  // Where the range falls in that collapsed text: the text before it, collapsed the same way.
  const before = range.cloneRange();
  try {
    before.setStart(block, 0);
    before.setEnd(range.startContainer, range.startOffset);
  } catch {
    return { sentence: full.trim(), selection: null }; // a range from another tree: no position to trust
  }
  const start = Math.min(collapse(before.toString()).length, full.length);
  const end = Math.min(start + collapse(range.toString()).length, full.length);

  // Walk the sentences, keeping each one's span, and take those the range touches. They are
  // contiguous and the text is collapsed, so the join is exactly the slice between the first
  // and the last — which is what lets the offsets carry over.
  let at = 0;
  let first = -1;
  let last = -1;
  for (const part of full.split(SENTENCE_SPLIT)) {
    const from = full.indexOf(part, at);
    const to = from + part.length;
    at = to;
    if (to > start && from < Math.max(end, start + 1)) {
      if (first < 0) first = from;
      last = to;
    }
  }
  const [from, to] = first < 0 ? [0, full.length] : [first, last];
  const raw = full.slice(from, to);
  const sentence = raw.trim();
  const shift = from + (raw.length - raw.trimStart().length);

  let s = Math.max(0, Math.min(start - shift, sentence.length));
  let e = Math.max(s, Math.min(end - shift, sentence.length));
  while (s < e && /\s/.test(sentence.charAt(s))) s++;
  while (e > s && /\s/.test(sentence.charAt(e - 1))) e--;
  return { sentence, selection: e > s ? { start: s, end: e } : null };
}

/** A captured selection: the text, its source sentence, an anchor rectangle, and the
 *  word-snapped range it came from (so a single word can be hit-tested for its status). */
export interface Capture {
  text: string;
  sentence: string;
  /** Where the selection sits in `sentence`, found by position (null when unknown). */
  selection: SentenceSelection["selection"];
  rect: { left: number; top: number; bottom: number };
  range: Range;
}

/** What a settled selection is: one word — a hyphenated compound included — or a bounded
 *  multi-word expression. */
export type CaptureKind = "word" | "phrase";

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

/**
 * Read a selection into a Capture, or null when there is nothing usable (empty, or longer
 * than a phrase). Whitespace is collapsed and both ends are snapped to whole words. The
 * snap stays internal: the page's own selection is left exactly as the reader made it.
 */
export function captureFrom(sel: Selection | null, maxLength: number = MAX_SELECTION_LENGTH): Capture | null {
  if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return null;
  const range = sel.getRangeAt(0).cloneRange();
  snapRangeToWords(range);
  const text = range.toString().trim().replace(/\s+/g, " ");
  if (!text || text.length > maxLength) return null;
  const box = typeof range.getBoundingClientRect === "function" ? range.getBoundingClientRect() : null;
  const rect = { left: box?.left ?? 0, top: box?.top ?? 0, bottom: box?.bottom ?? 0 };
  const { sentence, selection } = sentenceAndSelection(range);
  return { text, sentence, selection, rect, range };
}

/** The selection of a window — the page's by default, or a book section's — captured. */
export function captureSelection(
  maxLength: number = MAX_SELECTION_LENGTH,
  win: Pick<Window, "getSelection"> = window,
): Capture | null {
  return captureFrom(win.getSelection(), maxLength);
}

/** One word, or an expression: whitespace makes a phrase, which is also what the core means
 *  by an expression. A hyphen does not — "repo-wide" is one token to the analyser, so it
 *  opens the word card, from its page token when there is one. */
export function classifySelection(text: string): CaptureKind {
  return /\s/.test(text) ? "phrase" : "word";
}

/** Whether a node sits inside one of the reader's own injected surfaces (popup, drawer,
 *  HUD), which all carry the marker `blocks.ts` and `observer.ts` already honour. */
function insideReaderUi(node: Node | null): boolean {
  // By node type, not `instanceof Element`: a node of a book section's iframe belongs to
  // another realm, and would never be an instance of this window's Element.
  const el = node?.nodeType === Node.ELEMENT_NODE ? (node as Element) : (node?.parentElement ?? null);
  return !!el?.closest?.("[data-cymbra-lingua-skip]");
}

export interface SelectionWatcherOptions {
  /** Called once a usable selection has settled. */
  onCapture: (kind: CaptureKind, capture: Capture) => void;
  /** Read the current selection (injected in tests). */
  read?: () => Selection | null;
  /** The window whose selection is read when `read` is not given: the page's by default,
   *  a book section's in the reader. */
  win?: Pick<Window, "getSelection">;
  settleMs?: number;
  heldMs?: number;
  maxLength?: number;
  setTimer?: (fn: () => void, ms: number) => unknown;
  clearTimer?: (handle: unknown) => void;
}

/**
 * Debounces `selectionchange` into a single capture: `notify()` on every selection event,
 * `hold()`/`release()` around a pointer gesture. A lift is a reliable "the selection is
 * finished now" signal where it exists, so the mouse stays instant; while the pointer is
 * down the settle time merely stretches, so the card never pops up mid-drag AND a native
 * selection-handle drag — which dispatches no pointer event at all — is still captured.
 */
export class SelectionWatcher {
  private timer: unknown = null;
  /** Whether a pointer is currently down (a drag in progress). */
  private held = false;
  /** The last selection emitted, so a repeated event does not re-render the popup. */
  private lastKey: string | null = null;

  constructor(private readonly opts: SelectionWatcherOptions) {}

  /** The selection moved: restart the settle timer, or stand down if it is unusable. */
  notify(): void {
    this.cancel();
    if (!this.usable()) {
      this.lastKey = null; // a collapsed selection re-arms the same text next time
      return;
    }
    this.arm();
  }

  /** A pointer went down: a drag is starting, so give the selection room to grow. */
  hold(): void {
    this.held = true;
    if (this.timer !== null) {
      this.cancel();
      this.arm();
    }
  }

  /** The pointer lifted (or its gesture was cancelled): the selection is final. */
  release(): void {
    this.held = false;
    this.cancel();
    if (this.usable()) this.emit();
    else this.lastKey = null;
  }

  private arm(): void {
    const ms = this.held ? (this.opts.heldMs ?? SELECTION_HELD_MS) : (this.opts.settleMs ?? SELECTION_SETTLE_MS);
    this.timer = this.setTimer(() => {
      this.timer = null;
      this.emit();
    }, ms);
  }

  /** Drop any pending capture (the reader left, or the popup took over). */
  cancel(): void {
    if (this.timer === null) return;
    if (this.opts.clearTimer) this.opts.clearTimer(this.timer);
    else clearTimeout(this.timer as ReturnType<typeof setTimeout>);
    this.timer = null;
  }

  private setTimer(fn: () => void, ms: number): unknown {
    return this.opts.setTimer ? this.opts.setTimer(fn, ms) : setTimeout(fn, ms);
  }

  private selection(): Selection | null {
    return this.opts.read ? this.opts.read() : (this.opts.win ?? window).getSelection();
  }

  /** A selection worth waiting on: non-empty, and not inside the reader's own UI. */
  private usable(): boolean {
    const sel = this.selection();
    if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return false;
    return !insideReaderUi(sel.anchorNode);
  }

  private emit(): void {
    if (!this.usable()) return;
    const cap = captureFrom(this.selection(), this.opts.maxLength ?? MAX_SELECTION_LENGTH);
    if (!cap) return;
    const key = `${cap.text}@${Math.round(cap.rect.left)},${Math.round(cap.rect.top)}`;
    if (key === this.lastKey) return;
    this.lastKey = key;
    this.opts.onCapture(classifySelection(cap.text), cap);
  }
}
