// foliate-js turns pages by the finger: it follows every `touchmove` in a section, pans the page
// with it and cancels the event. On Safari a selection's handle drag — and the drag that goes on
// from a press-and-hold — reaches the page as those same touch events, so the page slid under
// the finger and the selection never grew: no phrase could be selected in a book on iOS or
// iPadOS. Firefox for Android does the same with the drag from a press-and-hold (measured on a
// Boox Go 10.3, Firefox 157, 2026-10-04: the page followed the finger, whatever the page-turn
// mode). A touch that works a selection is the platform's, and foliate does not see its moves.
//
// Firefox for Android also needs the press-and-hold itself: the selection did not show under the
// finger in time, and foliate cancelled the still finger's tiny moves, which can stop the platform's
// long press. So a finger held still for HOLD_MS is the platform's whatever the selection says, and
// the moves of a finger not yet held nor swiping go to no one.
//
// Only the moves are kept from it. foliate still sees the touch start and end, so a pan it did
// begin before the selection existed is settled back onto its page.

/** How far from the selected text a touch still grabs it: a handle's knob sits just outside. */
export const SELECTION_REACH_PX = 44;

/** How long a still finger takes to make a press-and-hold: Android starts its long press at 400 ms. */
export const HOLD_MS = 400;

/** How far a still finger may drift. Past it before HOLD_MS, the touch is a swipe: foliate's. */
export const STILL_PX = 10;

export interface TouchPoint {
  x: number;
  y: number;
}

/** Whether `p` is on one of `rects`, or within `reach` of it. */
export function nearRects(rects: Iterable<DOMRectReadOnly>, p: TouchPoint, reach = SELECTION_REACH_PX): boolean {
  for (const r of rects) {
    if (p.x >= r.left - reach && p.x <= r.right + reach && p.y >= r.top - reach && p.y <= r.bottom + reach) {
      return true;
    }
  }
  return false;
}

/** The selected text's boxes, or null when nothing is selected. */
function selectionRects(doc: Document): DOMRectReadOnly[] | null {
  const sel = doc.getSelection();
  if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return null;
  return [...sel.getRangeAt(0).getClientRects()];
}

function pointOf(e: Event): TouchPoint | null {
  const t = (e as TouchEvent).touches?.[0] ?? (e as TouchEvent).changedTouches?.[0];
  return t ? { x: t.clientX, y: t.clientY } : null;
}

/**
 * Keep foliate-js's page pan off the touches that work `doc`'s selection. Listens in the capture
 * phase, ahead of foliate's own listeners on the document, and stops only the moves.
 */
export function guardSelectionTouches(doc: Document): void {
  let start: TouchPoint | null = null;
  let startedAt = 0;
  let single = false;
  let owned = false;
  let swiping = false;
  const grabs = (): boolean => {
    const rects = start && single ? selectionRects(doc) : null;
    return !!rects && nearRects(rects, start!);
  };
  /** Whether a move of a touch neither held nor swiping yet goes to no one: a still finger's drift. */
  const drifts = (e: Event): boolean => {
    if (owned || swiping || !single || !start) return false;
    if (e.timeStamp - startedAt >= HOLD_MS) {
      owned = true; // held still, then moved: the platform's press-and-hold
      return false;
    }
    const p = pointOf(e);
    if (p && Math.hypot(p.x - start.x, p.y - start.y) > STILL_PX) {
      swiping = true;
      return false;
    }
    return true;
  };
  const opts = { capture: true, passive: true } as const;
  doc.addEventListener(
    "touchstart",
    (e) => {
      start = pointOf(e);
      startedAt = e.timeStamp;
      single = (e as TouchEvent).touches?.length === 1;
      owned = grabs();
      swiping = false;
    },
    opts,
  );
  // The press-and-hold's selection appears under a finger already down: asked at each move.
  doc.addEventListener(
    "touchmove",
    (e) => {
      const drift = drifts(e);
      if (!owned) owned = grabs();
      if (owned || drift) e.stopPropagation();
    },
    opts,
  );
  const end = (): void => {
    start = null;
    owned = false;
  };
  doc.addEventListener("touchend", end, opts);
  doc.addEventListener("touchcancel", end, opts);
}
