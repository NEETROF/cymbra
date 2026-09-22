import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { HOST_ID } from "@/reading/blocks.ts";
import { ReadingObservers } from "@/reading/observer.ts";

// Dynamic-content watching: a debounced MutationObserver turns DOM changes into a set of
// dirty *block* containers (never a whole-page re-walk), and an IntersectionObserver
// decides which of them are worth re-scanning now. jsdom ships MutationObserver — the real
// one is used here, so the observe options (subtree, characterData) are genuinely
// exercised; it delivers its records on a microtask, hence the `await` in `settle()`. jsdom
// has no IntersectionObserver, so that one is faked the way test/highlight-window.spec.ts
// fakes it, which also lets a test say exactly what the reader can see.

class FakeObserver {
  static last: FakeObserver | null = null;
  readonly observed: Element[] = [];
  disconnected = false;

  constructor(private readonly callback: IntersectionObserverCallback) {
    FakeObserver.last = this;
  }

  observe(el: Element): void {
    this.observed.push(el);
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

const DEBOUNCE = 50;

let seen: Element[][];
let obs: ReadingObservers;

/** A ReadingObservers whose rescans land in `seen`, debounced fast enough to keep tests terse. */
function makeObservers(debounceMs = DEBOUNCE): ReadingObservers {
  obs = new ReadingObservers({ onRescan: (containers) => void seen.push(containers), debounceMs });
  return obs;
}

/** The same, with no debounce configured, so the module's own default applies. */
function makeUntunedObservers(): ReadingObservers {
  obs = new ReadingObservers({ onRescan: (containers) => void seen.push(containers) });
  return obs;
}

/** Install the fake IntersectionObserver. Must run before `start()`, which captures it. */
function useIntersectionObserver(): void {
  vi.stubGlobal("IntersectionObserver", FakeObserver);
}

/** Let jsdom deliver its mutation records (a microtask), then run the clock forward. */
async function settle(ms: number): Promise<void> {
  await Promise.resolve();
  await vi.advanceTimersByTimeAsync(ms);
}

function el(id: string): Element {
  return document.getElementById(id)!;
}

/** Rewrite a block's text, the way a live page edits a paragraph in place. */
function retext(id: string, text: string): void {
  (el(id).firstChild as Text).data = text;
}

beforeEach(() => {
  vi.useFakeTimers();
  seen = [];
  FakeObserver.last = null;
  document.body.innerHTML = "";
});

afterEach(() => {
  obs.stop();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
});

describe("ReadingObservers — dirty block containers", () => {
  it("reports the nearest block container of a change, not the whole page", async () => {
    document.body.innerHTML = `<article><p id="para">the <span id="word">runner</span> runs</p></article>`;
    makeObservers().start();

    retext("word", "walker");
    await settle(DEBOUNCE + 1);

    // The <span> is inline, so the dirty unit is its <p> — not <article>, and not <body>.
    expect(seen).toEqual([[el("para")]]);
  });

  it("coalesces a burst of changes into one rescan, each container named once", async () => {
    document.body.innerHTML = `<p id="a">alpha</p><p id="b">beta</p>`;
    makeObservers().start();

    retext("a", "alpha one");
    retext("a", "alpha two"); // a second hit inside an already-dirty container
    retext("b", "beta one");
    await settle(DEBOUNCE + 1);

    expect(seen).toHaveLength(1);
    expect(seen[0]).toEqual([el("a"), el("b")]);
  });

  it("restarts the debounce while changes keep arriving", async () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers(100).start();

    retext("a", "one");
    await settle(60);
    retext("a", "two"); // resets the window: the first change must not flush at 100ms
    await settle(60);
    expect(seen).toHaveLength(0);

    await settle(50);
    expect(seen).toHaveLength(1);
  });

  it("waits 250 ms before rescanning when no debounce is configured", async () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeUntunedObservers().start();

    retext("a", "changed");
    await settle(249);
    expect(seen).toHaveLength(0);

    await settle(1);
    expect(seen).toEqual([[el("a")]]);
  });

  it("reports the block of a newly inserted node", async () => {
    document.body.innerHTML = `<div id="feed"></div>`;
    makeObservers().start();

    const post = document.createElement("p");
    post.textContent = "a freshly loaded post";
    el("feed").append(post);
    await settle(DEBOUNCE + 1);

    // Both the insertion point and the inserted block are dirty: the inserted <p> is its
    // own block, and <div id="feed"> is the block the mutation targeted.
    expect(seen).toEqual([[el("feed"), post]]);
  });

  it("ignores a node that was added and removed again before the batch was delivered", async () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers().start();

    const ghost = document.createTextNode(" flicker");
    el("a").append(ghost);
    ghost.remove(); // detached by the time the record is read: it has no block of its own
    await settle(DEBOUNCE + 1);

    expect(seen).toEqual([[el("a")]]);
  });

  it("falls back to the body for a change with no block-level ancestor", async () => {
    makeObservers().start();

    document.body.append(document.createTextNode("loose prose"));
    await settle(DEBOUNCE + 1);

    expect(seen).toEqual([[document.body]]);
  });

  it("never reports the reader's own injected UI", async () => {
    document.body.innerHTML =
      `<div id="${HOST_ID}"><p id="hud">12%</p></div>` +
      `<aside data-cymbra-lingua-skip><p id="drawer">review</p></aside>` +
      `<p id="article">the runner runs</p>`;
    makeObservers().start();

    retext("hud", "34%"); // the HUD repaints constantly; it must never trigger a re-analysis
    retext("drawer", "next card");
    retext("article", "the walker walks");
    await settle(DEBOUNCE + 1);

    expect(seen).toEqual([[el("article")]]);
  });

  it("schedules nothing at all when only the reader's own UI changed", async () => {
    document.body.innerHTML = `<div id="${HOST_ID}"><p id="hud">12%</p></div>`;
    makeObservers().start();

    retext("hud", "34%");
    await settle(DEBOUNCE * 10);

    expect(seen).toEqual([]);
  });

  it("skips a container that left the DOM before its rescan", async () => {
    document.body.innerHTML = `<div id="wrap"><p id="doomed">alpha</p></div>`;
    makeObservers().start();

    retext("doomed", "changed");
    const doomed = el("doomed");
    doomed.remove();
    await settle(DEBOUNCE + 1);

    // The removal itself dirties the wrapper, which is legitimate; the detached <p> is not
    // reported — re-scanning it would walk nodes that are no longer on the page.
    expect(seen).toEqual([[el("wrap")]]);
    expect(seen.flat()).not.toContain(doomed);
  });

  it("watches only the root it was given", async () => {
    document.body.innerHTML = `<div id="scope"><p id="inside">alpha</p></div><p id="outside">beta</p>`;
    makeObservers().start(el("scope"));

    retext("outside", "changed");
    await settle(DEBOUNCE + 1);
    expect(seen).toEqual([]);

    retext("inside", "changed");
    await settle(DEBOUNCE + 1);
    expect(seen).toEqual([[el("inside")]]);
  });
});

describe("ReadingObservers — visibility priority", () => {
  beforeEach(() => useIntersectionObserver());

  it("tracks every container it is handed", () => {
    document.body.innerHTML = `<p id="a">alpha</p><p id="b">beta</p>`;
    makeObservers().start();

    obs.track([el("a"), el("b")]);

    expect(FakeObserver.last!.observed).toEqual([el("a"), el("b")]);
  });

  it("tracks nothing before it has been started", () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers();

    obs.track([el("a")]);

    expect(FakeObserver.last).toBeNull();
  });

  it("rescans an on-screen container straight away", async () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers().start();
    obs.track([el("a")]);
    FakeObserver.last!.fire([[el("a"), true]]);

    retext("a", "changed");
    await settle(DEBOUNCE + 1);

    expect(seen).toEqual([[el("a")]]);
  });

  it("queues an off-screen container until it scrolls into view", async () => {
    document.body.innerHTML = `<p id="below">alpha</p>`;
    makeObservers().start();
    obs.track([el("below")]); // observed but never reported visible: it is below the fold

    retext("below", "changed");
    await settle(DEBOUNCE + 1);
    expect(seen).toEqual([]);

    FakeObserver.last!.fire([[el("below"), true]]);
    expect(seen).toEqual([[el("below")]]);
  });

  it("drops a queued container that left the DOM before it scrolled in", async () => {
    document.body.innerHTML = `<div id="wrap"><p id="below">alpha</p></div>`;
    makeObservers().start();
    obs.track([el("below")]);
    const below = el("below");

    retext("below", "changed");
    await settle(DEBOUNCE + 1);
    below.remove();
    FakeObserver.last!.fire([[below, true]]);

    expect(seen).toEqual([]);
  });

  it("queues a container again once it has scrolled back out", async () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers().start();
    obs.track([el("a")]);
    const io = FakeObserver.last!;
    io.fire([[el("a"), true]]);

    io.fire([[el("a"), false]]);
    retext("a", "changed");
    await settle(DEBOUNCE + 1);
    expect(seen).toEqual([]);

    io.fire([[el("a"), true]]);
    expect(seen).toEqual([[el("a")]]);
  });

  it("does not rescan when an intersection drains nothing", () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers().start();
    obs.track([el("a")]);

    FakeObserver.last!.fire([[el("a"), true]]);

    expect(seen).toEqual([]);
  });

  it("treats every container as on-screen where IntersectionObserver is unavailable", async () => {
    vi.stubGlobal("IntersectionObserver", undefined);
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers().start();
    obs.track([el("a")]); // a no-op, yet the container must still be rescanned

    retext("a", "changed");
    await settle(DEBOUNCE + 1);

    expect(seen).toEqual([[el("a")]]);
  });
});

describe("ReadingObservers — teardown", () => {
  it("cancels a rescan that was already pending", async () => {
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers().start();
    retext("a", "changed");
    await Promise.resolve();

    obs.stop();
    await settle(DEBOUNCE * 10);

    expect(seen).toEqual([]);
  });

  it("stops watching mutations and intersections", async () => {
    useIntersectionObserver();
    document.body.innerHTML = `<p id="a">alpha</p>`;
    makeObservers().start();
    obs.track([el("a")]);

    obs.stop();
    retext("a", "changed");
    await settle(DEBOUNCE * 10);

    expect(FakeObserver.last!.disconnected).toBe(true);
    expect(seen).toEqual([]);
  });
});
