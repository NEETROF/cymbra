// The scrolled flow reads on from one section to the next (add-lingua-reader, "Scrolled flow
// chosen": the book scrolls continuously). foliate-js lays out one section at a time: the wheel
// or the finger stops dead at the end of a chapter, and only the arrows or PageDown reach the
// next one — a short chapter looked like a scrolled flow that did not work. Its `next()` and
// `prev()` already cross into the adjacent section once the view is at an edge; this module
// sends them when the reader keeps pushing past that edge.
//
// Only a push that starts at the edge counts. A wheel or a flick that carries the view to the
// end of a chapter keeps sending events for a while (inertia): were those to count, a chapter
// would be skipped the moment it was reached. So the reader reaches the end, stops, and pushes
// once more.

/** What the book view tells and does, as far as its edges go. */
export interface ScrollEdges {
  /** Whether the book is in the scrolled flow — in pages, the edges are the page turns' job. */
  scrolled(): boolean;
  atTop(): boolean;
  atBottom(): boolean;
  next(): void;
  prev(): void;
}

/** How far past the edge a push must go, in pixels: a notch or two of a mouse wheel. */
export const EDGE_PUSH_PX = 120;
/** The pause that ends one wheel gesture: a push starts after it. */
export const EDGE_IDLE_MS = 400;
/** How far a finger must travel past the edge. */
export const EDGE_SWIPE_PX = 80;

/** The pixels a wheel event moves by, whatever unit it counts in (Firefox counts lines). */
function wheelPixels(e: WheelEvent): number {
  if (e.deltaMode === 1) return e.deltaY * 16;
  if (e.deltaMode === 2) return e.deltaY * 800;
  return e.deltaY;
}

function touchY(e: Event): number | null {
  const t = (e as TouchEvent).changedTouches?.[0];
  return t ? t.clientY : null;
}

/** Continue into the adjacent section when a push past an edge of `target`'s view starts there. */
export function continueAtEdges(
  target: EventTarget,
  edges: ScrollEdges,
  now: () => number = () => performance.now(),
): void {
  let last = Number.NEGATIVE_INFINITY;
  /** The edge a push started at, once a new gesture began there; null until then. */
  let armed: "top" | "bottom" | null = null;
  let push = 0;
  const opts = { passive: true } as const;

  target.addEventListener(
    "wheel",
    (e) => {
      const t = now();
      const fresh = t - last >= EDGE_IDLE_MS;
      last = t;
      if (!edges.scrolled()) return;
      const dy = wheelPixels(e as WheelEvent);
      const edge = dy > 0 && edges.atBottom() ? "bottom" : dy < 0 && edges.atTop() ? "top" : null;
      if (!edge) {
        armed = null;
        push = 0;
        return;
      }
      if (fresh || armed !== edge) {
        // A gesture that did not start at this edge (inertia arriving) arms it, and counts nothing.
        armed = fresh ? edge : null;
        push = 0;
        if (!fresh) return;
      }
      push += Math.abs(dy);
      if (push < EDGE_PUSH_PX) return;
      armed = null;
      push = 0;
      if (edge === "bottom") edges.next();
      else edges.prev();
    },
    opts,
  );

  let startY: number | null = null;
  let startEdge: "top" | "bottom" | null = null;
  target.addEventListener(
    "touchstart",
    (e) => {
      startY = touchY(e);
      startEdge = !edges.scrolled() ? null : edges.atBottom() ? "bottom" : edges.atTop() ? "top" : null;
    },
    opts,
  );
  target.addEventListener(
    "touchend",
    (e) => {
      const endY = touchY(e);
      const from = startEdge;
      startEdge = null;
      if (!from || startY === null || endY === null) return;
      const travelled = startY - endY; // positive: the finger moved up, towards what follows
      if (from === "bottom" && travelled >= EDGE_SWIPE_PX && edges.atBottom()) edges.next();
      else if (from === "top" && -travelled >= EDGE_SWIPE_PX && edges.atTop()) edges.prev();
    },
    opts,
  );
}
