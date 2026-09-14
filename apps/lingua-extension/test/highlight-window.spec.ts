import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { TokenClass } from "@/analyzer/types.ts";
import { clear, HL_LEARNING, HL_UNKNOWN, render } from "@/reading/highlight.ts";
import type { ResolvedToken } from "@/reading/scan.ts";

// Viewport-windowed painting: only the blocks near the viewport get their ranges
// registered (WebKit re-evaluates every registered range per rendering update). jsdom has
// neither the Highlight API nor IntersectionObserver, so both are faked here, and animation
// frames are queued so the per-frame batching can be observed.

class FakeHighlight extends Set<Range> {
  constructor(...ranges: Range[]) {
    super(ranges);
  }
}

class FakeObserver {
  static last: FakeObserver | null = null;
  readonly observed: Element[] = [];
  disconnected = false;

  constructor(
    private readonly callback: IntersectionObserverCallback,
    readonly options?: IntersectionObserverInit,
  ) {
    FakeObserver.last = this;
  }

  observe(el: Element): void {
    this.observed.push(el);
  }

  disconnect(): void {
    this.disconnected = true;
  }

  /** Deliver intersection changes as the browser would. */
  fire(changes: [Element, boolean][]): void {
    const entries = changes.map(
      ([target, isIntersecting]) => ({ target, isIntersecting }) as IntersectionObserverEntry,
    );
    this.callback(entries, this as unknown as IntersectionObserver);
  }
}

let registry: Map<string, FakeHighlight>;
let frames: FrameRequestCallback[];

function flushFrames(): void {
  for (const cb of frames.splice(0)) cb(0);
}

function painted(name: string): Range[] {
  return [...(registry.get(name) ?? [])];
}

/** A paragraph container holding one painted token over its whole text. */
function token(text: string, cls: TokenClass = "Unknown"): ResolvedToken {
  const p = document.createElement("p");
  p.textContent = text;
  document.body.append(p);
  const range = document.createRange();
  range.setStart(p.firstChild!, 0);
  range.setEnd(p.firstChild!, text.length);
  return {
    token: { block: 0, start: 0, end: text.length, surface: text, lemma: text, class: cls, gloss: null },
    range,
    container: p,
  };
}

beforeEach(() => {
  registry = new Map();
  frames = [];
  FakeObserver.last = null;
  vi.stubGlobal("CSS", { highlights: registry });
  vi.stubGlobal("Highlight", FakeHighlight);
  vi.stubGlobal("IntersectionObserver", FakeObserver);
  vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => frames.push(cb));
  vi.stubGlobal("cancelAnimationFrame", () => {});
});

afterEach(() => {
  clear();
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
});

describe("viewport-windowed highlight painting", () => {
  it("observes each block container once, one viewport above and below", () => {
    const a = token("alpha");
    const b = token("beta");
    const a2: ResolvedToken = { ...token("gamma"), container: a.container };

    render([a, b, a2]);

    const observer = FakeObserver.last!;
    expect(observer.options?.rootMargin).toBe("100% 0px");
    expect(observer.observed).toEqual([a.container, b.container]);
  });

  it("paints only the tokens of blocks near the viewport", () => {
    const near = token("near");
    const far = token("far");
    render([near, far]);

    FakeObserver.last!.fire([
      [near.container!, true],
      [far.container!, false],
    ]);
    flushFrames();

    expect(painted(HL_UNKNOWN)).toEqual([near.range]);
  });

  it("follows the scroll as blocks enter and leave the window", () => {
    const first = token("first");
    const second = token("second");
    render([first, second]);
    const observer = FakeObserver.last!;

    observer.fire([[first.container!, true]]);
    flushFrames();
    observer.fire([
      [first.container!, false],
      [second.container!, true],
    ]);
    flushFrames();

    expect(painted(HL_UNKNOWN)).toEqual([second.range]);
  });

  it("batches intersection changes into one repaint per frame", () => {
    const a = token("a");
    const b = token("b");
    render([a, b]);
    const observer = FakeObserver.last!;

    observer.fire([[a.container!, true]]);
    observer.fire([[b.container!, true]]);

    expect(frames).toHaveLength(1);
    flushFrames();
    expect(painted(HL_UNKNOWN)).toHaveLength(2);
  });

  it("separates learning and unknown words into their registries", () => {
    const learning = token("learning", "Learning");
    const unknown = token("unknown");
    render([learning, unknown]);

    FakeObserver.last!.fire([
      [learning.container!, true],
      [unknown.container!, true],
    ]);
    flushFrames();

    expect(painted(HL_LEARNING)).toEqual([learning.range]);
    expect(painted(HL_UNKNOWN)).toEqual([unknown.range]);
  });

  it("restarts the window on every render, dropping the previous observer", () => {
    const a = token("a");
    render([a]);
    const previous = FakeObserver.last!;

    render([token("b")]);

    expect(previous.disconnected).toBe(true);
    expect(FakeObserver.last).not.toBe(previous);
  });

  it("paints everything at once where IntersectionObserver is unavailable", () => {
    vi.stubGlobal("IntersectionObserver", undefined);
    const a = token("a");
    const b = token("b");

    render([a, b]);

    expect(frames).toHaveLength(0);
    expect(painted(HL_UNKNOWN)).toEqual([a.range, b.range]);
  });

  it("clears the registries and stops observing", () => {
    const a = token("a");
    render([a]);
    const observer = FakeObserver.last!;
    observer.fire([[a.container!, true]]);
    flushFrames();

    clear();

    expect(observer.disconnected).toBe(true);
    expect(registry.has(HL_UNKNOWN)).toBe(false);
    expect(registry.has(HL_LEARNING)).toBe(false);
  });
});
