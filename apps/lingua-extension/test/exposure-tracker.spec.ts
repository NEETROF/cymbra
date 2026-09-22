import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ExposureTracker } from "@/reading/exposure-tracker.ts";

// Viewport-gated reading exposure: a block's lemmas count as "read" only once the block has
// been on screen for the dwell, and only once ever. jsdom has no IntersectionObserver, so it
// is faked the way test/highlight-window.spec.ts fakes it — here the fake is also what lets a
// test scroll a block in and out on demand. The dwell runs on vitest's fake timers, so no
// test waits on a real clock.

class FakeObserver {
  static last: FakeObserver | null = null;
  readonly observed: Element[] = [];
  readonly unobserved: Element[] = [];
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

  unobserve(el: Element): void {
    this.unobserved.push(el);
  }

  disconnect(): void {
    this.disconnected = true;
  }

  /** Deliver visibility changes as the browser would when the page scrolls. */
  fire(changes: [Element, boolean][]): void {
    const entries = changes.map(
      ([target, isIntersecting]) => ({ target, isIntersecting }) as IntersectionObserverEntry,
    );
    this.callback(entries, this as unknown as IntersectionObserver);
  }
}

const DWELL = 1500;

let reported: string[][];
let tracker: ExposureTracker;

function makeTracker(dwellMs = DWELL): ExposureTracker {
  tracker = new ExposureTracker((lemmas) => void reported.push(lemmas), dwellMs);
  return tracker;
}

/** A paragraph on the page whose text is `lemmas`; tracking it is each test's own business. */
function block(id: string, lemmas: string[]): Element {
  const p = document.createElement("p");
  p.id = id;
  p.textContent = lemmas.join(" ");
  document.body.append(p);
  return p;
}

function io(): FakeObserver {
  return FakeObserver.last!;
}

beforeEach(() => {
  vi.useFakeTimers();
  reported = [];
  FakeObserver.last = null;
  document.body.innerHTML = "";
  vi.stubGlobal("IntersectionObserver", FakeObserver);
});

afterEach(() => {
  tracker.stop();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
});

describe("ExposureTracker", () => {
  it("watches blocks at half visibility, not at the first stray pixel", () => {
    makeTracker().start();

    // A block grazing the edge of the viewport was not read; half of it must be showing.
    expect(io().options?.threshold).toBe(0.5);
  });

  it("observes each tracked container", () => {
    const a = block("a", ["run"]);
    const b = block("b", ["walk"]);
    makeTracker().start();

    tracker.track(
      new Map([
        [a, ["run"]],
        [b, ["walk"]],
      ]),
    );

    expect(io().observed).toEqual([a, b]);
  });

  it("ignores a container with no lemmas to report", () => {
    const empty = block("empty", []);
    makeTracker().start();

    tracker.track(new Map([[empty, []]]));

    expect(io().observed).toEqual([]);
  });

  it("tracks nothing before it has been started", () => {
    const a = block("a", ["run"]);
    makeTracker();

    tracker.track(new Map([[a, ["run"]]]));

    expect(FakeObserver.last).toBeNull();
  });

  it("reports a container's lemmas once it has been visible for the dwell", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run", "walk"]]]));

    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL);

    expect(reported).toEqual([["run", "walk"]]);
  });

  it("reports nothing while the dwell is still running", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));

    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL - 1);

    expect(reported).toEqual([]);
  });

  it("never reports a container that scrolled away before the dwell elapsed", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));

    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL - 1);
    io().fire([[a, false]]); // scrolled past without reading it
    await vi.advanceTimersByTimeAsync(DWELL * 10);

    expect(reported).toEqual([]);
  });

  it("starts the dwell over when a container comes back into view", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));

    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL - 1);
    io().fire([[a, false]]);
    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL - 1);
    expect(reported).toEqual([]); // the earlier near-miss does not carry over

    await vi.advanceTimersByTimeAsync(1);
    expect(reported).toEqual([["run"]]);
  });

  it("shrugs off a container leaving the viewport with no dwell under way", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));

    io().fire([[a, false]]);
    await vi.advanceTimersByTimeAsync(DWELL * 10);

    expect(reported).toEqual([]);
  });

  it("keeps one dwell running when intersections repeat", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));

    // A resize or a threshold re-evaluation can re-deliver "visible" without a scroll-out;
    // re-arming the timer each time would push the dwell out forever.
    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL - 1);
    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(1);

    expect(reported).toEqual([["run"]]);
  });

  it("reports each container at most once, however often it is re-read", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));

    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL);
    io().fire([[a, false]]);
    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL * 10);

    expect(reported).toEqual([["run"]]);
    expect(io().unobserved).toEqual([a]); // and it stops costing the browser anything
  });

  it("skips a container it has already reported when the page is re-scanned", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));
    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL);

    // A re-scan hands the same container back with fresh lemmas; it must not be re-observed.
    tracker.track(new Map([[a, ["run", "walk"]]]));
    io().fire([[a, true]]);
    await vi.advanceTimersByTimeAsync(DWELL * 10);

    expect(io().observed).toEqual([a]);
    expect(reported).toEqual([["run"]]);
  });

  it("reports each container's own lemmas separately", async () => {
    const a = block("a", ["run"]);
    const b = block("b", ["walk"]);
    makeTracker().start();
    tracker.track(
      new Map([
        [a, ["run"]],
        [b, ["walk"]],
      ]),
    );

    io().fire([
      [a, true],
      [b, true],
    ]);
    await vi.advanceTimersByTimeAsync(DWELL);

    expect(reported).toEqual([["run"], ["walk"]]);
  });

  it("does nothing at all where IntersectionObserver is unavailable", async () => {
    vi.stubGlobal("IntersectionObserver", undefined);
    const a = block("a", ["run"]);
    makeTracker().start();

    tracker.track(new Map([[a, ["run"]]]));
    await vi.advanceTimersByTimeAsync(DWELL * 10);

    expect(reported).toEqual([]);
  });

  it("stops watching on teardown", () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));
    const observer = io();

    tracker.stop();
    tracker.track(new Map([[a, ["run"]]]));

    expect(observer.disconnected).toBe(true);
    expect(observer.observed).toEqual([a]); // the post-stop track() went nowhere
  });

  it("still reports a container whose dwell was already under way when it was stopped", async () => {
    const a = block("a", ["run"]);
    makeTracker().start();
    tracker.track(new Map([[a, ["run"]]]));
    io().fire([[a, true]]);

    tracker.stop();
    await vi.advanceTimersByTimeAsync(DWELL);

    // KNOWN GAP, pinned as it behaves rather than quietly changed here: stop() drops the
    // observer but clears no dwell timer, so a block that was mid-dwell still reports after
    // teardown — in content.ts that re-arms the exposure flush the caller just cancelled.
    expect(reported).toEqual([["run"]]);
  });
});
