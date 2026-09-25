import { beforeEach, describe, expect, it } from "vitest";
import { continueAtEdges, EDGE_IDLE_MS, EDGE_PUSH_PX, EDGE_SWIPE_PX } from "@/reader/scroll-edges.ts";

// The scrolled flow reads on into the next section (add-lingua-reader, "Scrolled flow chosen"):
// a push past the end of a chapter that starts there turns to the next one, and to the previous
// one at the top — never the inertia that carried the view to the edge.

let target: EventTarget;
let clock: number;
let at: "top" | "middle" | "bottom";
let scrolled: boolean;
let moves: string[];

beforeEach(() => {
  target = new EventTarget();
  clock = 10_000;
  at = "middle";
  scrolled = true;
  moves = [];
  continueAtEdges(
    target,
    {
      scrolled: () => scrolled,
      atTop: () => at === "top",
      atBottom: () => at === "bottom",
      next: () => void moves.push("next"),
      prev: () => void moves.push("prev"),
    },
    () => clock,
  );
});

/** A wheel event `ms` after the previous one, moving by `deltaY` in `deltaMode`'s unit. */
function wheel(deltaY: number, ms = 16, deltaMode = 0): void {
  clock += ms;
  const e = new Event("wheel");
  Object.assign(e, { deltaY, deltaMode });
  target.dispatchEvent(e);
}

function touch(type: "touchstart" | "touchend", y: number): void {
  const e = new Event(type);
  Object.defineProperty(e, "changedTouches", { value: [{ clientY: y }] });
  target.dispatchEvent(e);
}

describe("the wheel at an edge", () => {
  it("turns to the next section when a push starts at the end and goes far enough", () => {
    at = "bottom";
    wheel(EDGE_PUSH_PX / 2, EDGE_IDLE_MS);
    expect(moves).toEqual([]);
    wheel(EDGE_PUSH_PX / 2);
    expect(moves).toEqual(["next"]);
  });

  it("turns to the previous section from the top", () => {
    at = "top";
    wheel(-EDGE_PUSH_PX, EDGE_IDLE_MS);
    expect(moves).toEqual(["prev"]);
  });

  it("ignores the inertia that carried the view to the end, until the reader pushes again", () => {
    wheel(300, EDGE_IDLE_MS); // scrolling down through the chapter
    at = "bottom";
    for (let i = 0; i < 20; i++) wheel(60); // the flick keeps coming, now at the end
    expect(moves).toEqual([]);
    wheel(EDGE_PUSH_PX, EDGE_IDLE_MS); // a new push, after a pause
    expect(moves).toEqual(["next"]);
  });

  it("counts a line-based wheel (Firefox) in pixels", () => {
    at = "bottom";
    wheel(3, EDGE_IDLE_MS, 1);
    expect(moves).toEqual([]);
    wheel(3, 16, 1);
    wheel(3, 16, 1);
    expect(moves).toEqual(["next"]);
  });

  it("forgets a push that turned back or left the edge", () => {
    at = "bottom";
    wheel(EDGE_PUSH_PX - 10, EDGE_IDLE_MS);
    wheel(-20); // back up: no longer a push past the end
    at = "middle";
    wheel(20);
    at = "bottom";
    wheel(EDGE_PUSH_PX - 10);
    expect(moves).toEqual([]);
  });

  it("does nothing in the paginated flow: the page turns are the tap zones' and the arrows'", () => {
    scrolled = false;
    at = "bottom";
    wheel(EDGE_PUSH_PX * 3, EDGE_IDLE_MS);
    expect(moves).toEqual([]);
  });
});

describe("the finger at an edge", () => {
  it("turns to the next section on a swipe up that starts at the end", () => {
    at = "bottom";
    touch("touchstart", 500);
    touch("touchend", 500 - EDGE_SWIPE_PX);
    expect(moves).toEqual(["next"]);
  });

  it("turns to the previous section on a swipe down that starts at the top", () => {
    at = "top";
    touch("touchstart", 200);
    touch("touchend", 200 + EDGE_SWIPE_PX);
    expect(moves).toEqual(["prev"]);
  });

  it("leaves a swipe that reached the edge on its way, a short one, and the paginated flow", () => {
    touch("touchstart", 500);
    at = "bottom";
    touch("touchend", 100);
    touch("touchstart", 500);
    touch("touchend", 500 - EDGE_SWIPE_PX / 2);
    scrolled = false;
    touch("touchstart", 500);
    touch("touchend", 100);
    expect(moves).toEqual([]);
  });
});
