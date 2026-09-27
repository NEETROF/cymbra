// Viewport-gated reading exposure (add-lingua-cefr-levels, slice 5c). A word counts
// as "read" only when its block scrolls into view AND stays visible for a short dwell
// — not merely because the page loaded (a word at the bottom of a long page you never
// reached must not count). It is a proxy, not eye-tracking: combined with the engine's
// distinct-day threshold and below-level-only promotion, it is a conservative signal.
// Each container fires at most once; the caller batches the lemmas and feeds the engine.
// The payload is the caller's: lemmas alone, or a `BlockReading` that also carries the
// block's occurrence counts for the daily reading stats (refine-lingua-reading-stats).
//
// Containers tracked before `start()` — the first paint tracks them before the observer exists —
// are remembered and observed on start (fix-lingua-dynamic-rescan D1): reading counts from the
// first paint, whatever order the caller works in.

/** A payload with nothing to report is never observed: an empty array by default. */
function emptyArray(payload: unknown): boolean {
  return Array.isArray(payload) && payload.length === 0;
}

export class ExposureTracker<T = string[]> {
  private io: IntersectionObserver | null = null;
  private readonly payloads = new WeakMap<Element, T>();
  private readonly timers = new WeakMap<Element, ReturnType<typeof setTimeout>>();
  private readonly exposed = new WeakSet<Element>();
  /** Tracked before `start()`: observed once there is an observer. */
  private readonly early = new Set<Element>();

  constructor(
    private readonly onExposed: (payload: T) => void,
    private readonly dwellMs = 1500,
    private readonly isEmpty: (payload: T) => boolean = emptyArray,
  ) {}

  /** Begin watching, with the observer of the window the blocks live in (a book section's
   *  iframe has its own); the page's by default. */
  start(win: Window & typeof globalThis = window): void {
    const early = [...this.early];
    this.early.clear();
    if (typeof win.IntersectionObserver === "undefined") return;
    this.io = new win.IntersectionObserver((entries) => this.onIntersections(entries), { threshold: 0.5 });
    for (const c of early) if (c.isConnected && !this.exposed.has(c)) this.io.observe(c);
  }

  stop(): void {
    this.io?.disconnect();
    this.io = null;
    this.early.clear();
  }

  /** Observe each container for its payload; containers already exposed are skipped, and a
   *  re-tracked one keeps the latest payload. Before `start()`, they are remembered and
   *  observed as soon as it runs. */
  track(byContainer: Map<Element, T>): void {
    for (const [container, payload] of byContainer) {
      if (this.exposed.has(container) || this.isEmpty(payload)) continue;
      this.payloads.set(container, payload);
      if (this.io) this.io.observe(container);
      else this.early.add(container);
    }
  }

  private onIntersections(entries: IntersectionObserverEntry[]): void {
    for (const e of entries) {
      const el = e.target as Element;
      if (e.isIntersecting) {
        if (this.exposed.has(el) || this.timers.has(el)) continue;
        this.timers.set(
          el,
          setTimeout(() => this.confirm(el), this.dwellMs),
        );
      } else {
        const t = this.timers.get(el);
        if (t !== undefined) {
          clearTimeout(t);
          this.timers.delete(el);
        }
      }
    }
  }

  /** The container stayed visible for the dwell: report its payload once and stop watching it. */
  private confirm(el: Element): void {
    this.timers.delete(el);
    if (this.exposed.has(el)) return;
    this.exposed.add(el);
    this.io?.unobserve(el);
    const payload = this.payloads.get(el);
    if (payload !== undefined && !this.isEmpty(payload)) this.onExposed(payload);
  }
}
