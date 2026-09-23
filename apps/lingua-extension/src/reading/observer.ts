import { READER_IGNORED_SELECTOR } from "./blocks.ts";

// Dynamic-content handling (task 1.4). A debounced MutationObserver turns DOM changes
// into a set of dirty block-level containers — never a whole-page re-walk. An
// IntersectionObserver prioritises what the reader can see: dirty containers that are
// on-screen are re-scanned immediately; off-screen ones are queued and re-scanned when
// they scroll into view, so a pathological SPA degrades gracefully instead of janking.

const BLOCK_SELECTOR = [
  "p",
  "li",
  "td",
  "th",
  "dd",
  "dt",
  "blockquote",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "figcaption",
  "article",
  "section",
  "aside",
  "main",
  "header",
  "footer",
  "div",
].join(",");

/** Nearest block-level container of a node, or the body. */
function blockOf(node: Node): Element | null {
  const el = node.nodeType === Node.TEXT_NODE ? node.parentElement : (node as Element);
  if (!el) return null;
  if (el.closest(READER_IGNORED_SELECTOR)) return null; // our own UI, opted-out subtrees, player captions
  return el.closest(BLOCK_SELECTOR) ?? document.body;
}

export interface ReadingObserverOptions {
  /** Re-scan these containers (already filtered to visible-first ordering). */
  onRescan: (containers: Element[]) => void;
  /** Debounce window for coalescing mutation bursts. */
  debounceMs?: number;
}

export class ReadingObservers {
  private mo: MutationObserver | null = null;
  private io: IntersectionObserver | null = null;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private readonly dirty = new Set<Element>();
  private readonly visible = new WeakSet<Element>();
  private readonly pending = new Set<Element>();
  /** Containers the IntersectionObserver watches — a queued one must be among them to drain. */
  private watched = new WeakSet<Element>();
  /** Containers handed to `track()` before `start()`: observed as soon as it starts. */
  private readonly early = new Set<Element>();

  constructor(private readonly opts: ReadingObserverOptions) {}

  /** Begin observing mutations under `root` for dynamic re-analysis. */
  start(root: Node = document.body): void {
    this.mo = new MutationObserver((records) => this.onMutations(records));
    this.mo.observe(root, { childList: true, subtree: true, characterData: true });
    if (typeof IntersectionObserver !== "undefined") {
      this.io = new IntersectionObserver((entries) => this.onIntersections(entries));
      for (const c of this.early) if (c.isConnected) this.watch(c);
    }
    this.early.clear();
  }

  stop(): void {
    this.mo?.disconnect();
    this.io?.disconnect();
    if (this.timer !== null) clearTimeout(this.timer);
    this.mo = this.io = null;
    this.early.clear();
    this.watched = new WeakSet(); // a later start() creates a new observer that watches nothing yet
  }

  /**
   * Track a set of block containers for visibility (call after each scan). Before `start()`
   * they are kept and observed once it runs: the first scan happens before the observers exist.
   */
  track(containers: Iterable<Element>): void {
    for (const c of containers) {
      if (this.io) this.watch(c);
      else if (!this.mo) this.early.add(c);
    }
  }

  private watch(c: Element): void {
    if (!this.io || this.watched.has(c)) return;
    this.watched.add(c);
    this.io.observe(c);
  }

  private onMutations(records: MutationRecord[]): void {
    for (const rec of records) {
      const container = blockOf(rec.target);
      if (container) this.dirty.add(container);
      for (const node of rec.addedNodes) {
        const c = blockOf(node);
        if (c) this.dirty.add(c);
      }
    }
    if (this.dirty.size === 0) return;
    if (this.timer !== null) clearTimeout(this.timer);
    this.timer = setTimeout(() => this.flush(), this.opts.debounceMs ?? 250);
  }

  private flush(): void {
    this.timer = null;
    const nowVisible: Element[] = [];
    for (const c of this.dirty) {
      if (!c.isConnected) continue;
      // With no IntersectionObserver, everything counts as visible. Otherwise a container not
      // known to be on screen waits — watched, so its first intersection drains it if it is.
      if (!this.io || this.visible.has(c)) nowVisible.push(c);
      else {
        this.pending.add(c);
        this.watch(c);
      }
    }
    this.dirty.clear();
    if (nowVisible.length > 0) this.opts.onRescan(nowVisible);
  }

  private onIntersections(entries: IntersectionObserverEntry[]): void {
    const drained: Element[] = [];
    for (const e of entries) {
      const el = e.target as Element;
      if (e.isIntersecting) {
        this.visible.add(el);
        if (this.pending.delete(el) && el.isConnected) drained.push(el);
      } else {
        this.visible.delete(el);
      }
    }
    if (drained.length > 0) this.opts.onRescan(drained);
  }
}
