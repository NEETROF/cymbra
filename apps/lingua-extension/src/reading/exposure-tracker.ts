// Viewport-gated reading exposure (add-lingua-cefr-levels, slice 5c). A word counts
// as "read" only when its block scrolls into view AND stays visible for a short dwell
// — not merely because the page loaded (a word at the bottom of a long page you never
// reached must not count). It is a proxy, not eye-tracking: combined with the engine's
// distinct-day threshold and below-level-only promotion, it is a conservative signal.
// Each container fires at most once; the caller batches the lemmas and feeds the engine.

export class ExposureTracker {
  private io: IntersectionObserver | null = null;
  private readonly lemmas = new WeakMap<Element, string[]>();
  private readonly timers = new WeakMap<Element, ReturnType<typeof setTimeout>>();
  private readonly exposed = new WeakSet<Element>();
  /** Containers handed to `track()` before `start()`: observed as soon as it starts. */
  private readonly early = new Set<Element>();
  private started = false;

  constructor(
    private readonly onExposed: (lemmas: string[]) => void,
    private readonly dwellMs = 1500,
  ) {}

  start(): void {
    this.started = true;
    if (typeof IntersectionObserver !== "undefined") {
      this.io = new IntersectionObserver((entries) => this.onIntersections(entries), { threshold: 0.5 });
      for (const c of this.early) if (c.isConnected && !this.exposed.has(c)) this.io.observe(c);
    }
    this.early.clear();
  }

  stop(): void {
    this.io?.disconnect();
    this.io = null;
    this.started = false;
    this.early.clear();
  }

  /**
   * Observe each container for its lemmas; containers already exposed are skipped. Before
   * `start()` they are kept and observed once it runs: the page's first scan comes first.
   */
  track(byContainer: Map<Element, string[]>): void {
    for (const [container, lemmas] of byContainer) {
      if (this.exposed.has(container) || lemmas.length === 0) continue;
      this.lemmas.set(container, lemmas);
      if (this.io) this.io.observe(container);
      else if (!this.started) this.early.add(container);
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
