// foliate-js turns pages by the finger: it follows every `touchmove` in a section, pans the page
// with it and cancels the event. On Safari a selection's handle drag — and the drag that goes on
// from a press-and-hold — reaches the page as those same touch events, so the page slid under
// the finger and the selection never grew: no phrase could be selected in a book on iOS or
// iPadOS. A touch that works a selection is the platform's, and foliate does not see its moves.
//
// Only the moves are kept from it. foliate still sees the touch start and end, so a pan it did
// begin before the selection existed is settled back onto its page.

/** How far from the selected text a touch still grabs it: a handle's knob sits just outside. */
export const SELECTION_REACH_PX = 44;

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
  let single = false;
  let owned = false;
  const grabs = (): boolean => {
    const rects = start && single ? selectionRects(doc) : null;
    return !!rects && nearRects(rects, start!);
  };
  const opts = { capture: true, passive: true } as const;
  doc.addEventListener(
    "touchstart",
    (e) => {
      start = pointOf(e);
      single = (e as TouchEvent).touches?.length === 1;
      owned = grabs();
    },
    opts,
  );
  // The press-and-hold's selection appears under a finger already down: asked at each move.
  doc.addEventListener(
    "touchmove",
    (e) => {
      if (!owned) owned = grabs();
      if (owned) e.stopPropagation();
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
