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

/** A captured selection: the text, its source sentence, an anchor rectangle, and the
 *  word-snapped range it came from (so a single word can be hit-tested for its status). */
export interface Capture {
  text: string;
  sentence: string;
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
  const firstWord = text.split(" ")[0] ?? text;
  const sentence = sentenceAround(range.startContainer, firstWord);
  return { text, sentence, rect, range };
}

/** The page selection, captured. */
export function captureSelection(maxLength: number = MAX_SELECTION_LENGTH): Capture | null {
  return captureFrom(window.getSelection(), maxLength);
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
  const el = node instanceof Element ? node : (node?.parentElement ?? null);
  return !!el?.closest?.("[data-cymbra-lingua-skip]");
}

export interface SelectionWatcherOptions {
  /** Called once a usable selection has settled. */
  onCapture: (kind: CaptureKind, capture: Capture) => void;
  /** Read the current selection (injected in tests). */
  read?: () => Selection | null;
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
    return this.opts.read ? this.opts.read() : window.getSelection();
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
