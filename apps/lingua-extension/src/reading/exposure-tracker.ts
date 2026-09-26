// Viewport-gated reading exposure (add-lingua-cefr-levels, slice 5c). A word counts
// as "read" only when its block scrolls into view AND stays visible for a short dwell
// — not merely because the page loaded (a word at the bottom of a long page you never
// reached must not count). It is a proxy, not eye-tracking: combined with the engine's
// distinct-day threshold and below-level-only promotion, it is a conservative signal.
// Each container fires at most once; the caller batches the lemmas and feeds the engine.
//
// Containers tracked before `start()` — the first paint tracks them before the observer exists —
// are remembered and observed on start (fix-lingua-dynamic-rescan D1): reading counts from the
// first paint, whatever order the caller works in.

export class ExposureTracker {
  private io: IntersectionObserver | null = null;
  private readonly lemmas = new WeakMap<Element, string[]>();
  private readonly timers = new WeakMap<Element, ReturnType<typeof setTimeout>>();
  private readonly exposed = new WeakSet<Element>();
  /** Tracked before `start()`: observed once there is an observer. */
  private readonly early = new Set<Element>();

  constructor(
    private readonly onExposed: (lemmas: string[]) => void,
    private readonly dwellMs = 1500,
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

  /** Observe each container for its lemmas; containers already exposed are skipped. Before
   *  `start()`, they are remembered and observed as soon as it runs. */
  track(byContainer: Map<Element, string[]>): void {
    for (const [container, lemmas] of byContainer) {
      if (this.exposed.has(container) || lemmas.length === 0) continue;
      this.lemmas.set(container, lemmas);
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

  /** The container stayed visible for the dwell: record its lemmas once and stop watching it. */
  private confirm(el: Element): void {
    this.timers.delete(el);
    if (this.exposed.has(el)) return;
    this.exposed.add(el);
    this.io?.unobserve(el);
    const lemmas = this.lemmas.get(el);
    if (lemmas && lemmas.length > 0) this.onExposed(lemmas);
  }
}
