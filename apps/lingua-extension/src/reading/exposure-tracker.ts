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

  constructor(
    private readonly onExposed: (lemmas: string[]) => void,
    private readonly dwellMs = 1500,
  ) {}

  start(): void {
    if (typeof IntersectionObserver === "undefined") return;
    this.io = new IntersectionObserver((entries) => this.onIntersections(entries), { threshold: 0.5 });
  }

  stop(): void {
    this.io?.disconnect();
    this.io = null;
  }

  /** Observe each container for its lemmas; containers already exposed are skipped. */
  track(byContainer: Map<Element, string[]>): void {
    if (!this.io) return;
    for (const [container, lemmas] of byContainer) {
      if (this.exposed.has(container) || lemmas.length === 0) continue;
      this.lemmas.set(container, lemmas);
      this.io.observe(container);
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
