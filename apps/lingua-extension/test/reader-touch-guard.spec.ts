import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { guardSelectionTouches, nearRects, SELECTION_REACH_PX } from "@/reader/touch-guard.ts";

// On Safari a selection's handle drag reaches the page as touch events, and foliate-js pans
// the page on every move it sees: the guard keeps the moves of a touch working the selection
// away from it (add-lingua-reader, the Safari selection).

const box = (left: number, top: number, width: number, height: number): DOMRectReadOnly =>
  ({ left, top, right: left + width, bottom: top + height, width, height, x: left, y: top }) as DOMRectReadOnly;

/** A section of its own per test, as foliate-js gives each one: a frame's document. */
let frame: HTMLIFrameElement;
let doc: Document;
/** The section's own window: its events and ranges are of its realm. */
let win: Window & typeof globalThis;
/** What foliate-js's own listener on the document receives, after the guard's. */
let seen: string[];

/** A touch event as the section sees it, with `fingers` down at (x, y). */
function touch(type: string, x: number, y: number, fingers = 1): Event {
  const e = new win.Event(type, { bubbles: true, cancelable: true });
  const point = { clientX: x, clientY: y };
  Object.defineProperty(e, "touches", { value: type === "touchend" ? [] : Array(fingers).fill(point) });
  Object.defineProperty(e, "changedTouches", { value: [point] });
  return e;
}

const p = (): Element => doc.querySelector("p")!;

/** The platform selects "seldom", whose box is (100, 200, 60 × 20). */
function selectWord(): void {
  const node = p().firstChild!;
  const range = doc.createRange();
  range.setStart(node, 5);
  range.setEnd(node, 11);
  doc.getSelection()!.removeAllRanges();
  doc.getSelection()!.addRange(range);
}

beforeEach(() => {
  frame = document.createElement("iframe");
  document.body.append(frame);
  doc = frame.contentDocument!;
  win = frame.contentWindow as Window & typeof globalThis;
  doc.body.innerHTML = `<p>They seldom ship on Friday.</p>`;
  // jsdom lays nothing out: the selected word's box, as a browser would give it.
  Object.defineProperty(win.Range.prototype, "getClientRects", {
    value: () => [box(100, 200, 60, 20)],
    configurable: true,
  });
  seen = [];
  guardSelectionTouches(doc);
  for (const type of ["touchstart", "touchmove", "touchend"]) doc.addEventListener(type, () => seen.push(type));
});

afterEach(() => frame.remove());

describe("nearRects", () => {
  it("counts a point on the text, and one within reach of it — where a handle's knob sits", () => {
    const rects = [box(100, 200, 60, 20)];
    expect(nearRects(rects, { x: 130, y: 210 })).toBe(true);
    expect(nearRects(rects, { x: 160 + SELECTION_REACH_PX, y: 220 + SELECTION_REACH_PX })).toBe(true);
    expect(nearRects(rects, { x: 161 + SELECTION_REACH_PX, y: 210 })).toBe(false);
  });
});

describe("guardSelectionTouches", () => {
  it("keeps a handle drag's moves from the page's pan, and lets its start and end through", () => {
    selectWord();
    p().dispatchEvent(touch("touchstart", 165, 230));
    p().dispatchEvent(touch("touchmove", 200, 230));
    p().dispatchEvent(touch("touchmove", 260, 230));
    p().dispatchEvent(touch("touchend", 260, 230));
    expect(seen).toEqual(["touchstart", "touchend"]);
  });

  it("takes over a press-and-hold that becomes a selection under the finger", () => {
    p().dispatchEvent(touch("touchstart", 130, 210));
    selectWord(); // the platform selected the word the finger is holding
    p().dispatchEvent(touch("touchmove", 180, 210));
    p().dispatchEvent(touch("touchend", 180, 210));
    expect(seen).toEqual(["touchstart", "touchend"]);
  });

  it("leaves a swipe to the page's pan: no selection, or one out of reach", () => {
    p().dispatchEvent(touch("touchstart", 400, 500));
    p().dispatchEvent(touch("touchmove", 300, 500));
    p().dispatchEvent(touch("touchend", 300, 500));
    selectWord();
    p().dispatchEvent(touch("touchstart", 400, 500));
    p().dispatchEvent(touch("touchmove", 300, 500));
    expect(seen).toEqual(["touchstart", "touchmove", "touchend", "touchstart", "touchmove"]);
  });

  it("leaves a two-finger gesture alone, and forgets a touch once it is cancelled", () => {
    selectWord();
    p().dispatchEvent(touch("touchstart", 130, 210, 2));
    p().dispatchEvent(touch("touchmove", 150, 210, 2));
    p().dispatchEvent(touch("touchcancel", 150, 210));
    p().dispatchEvent(touch("touchmove", 150, 210));
    expect(seen).toEqual(["touchstart", "touchmove", "touchmove"]);
  });
});
