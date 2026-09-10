import { HOST_ID } from "./blocks.ts";

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
  if (el.closest(`#${HOST_ID},[data-cymbra-lingua-skip]`)) return null; // ignore our own UI
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

  constructor(private readonly opts: ReadingObserverOptions) {}

  /** Begin observing mutations under `root` for dynamic re-analysis. */
  start(root: Node = document.body): void {
    this.mo = new MutationObserver((records) => this.onMutations(records));
    this.mo.observe(root, { childList: true, subtree: true, characterData: true });
    if (typeof IntersectionObserver !== "undefined") {
      this.io = new IntersectionObserver((entries) => this.onIntersections(entries));
    }
  }

  stop(): void {
    this.mo?.disconnect();
    this.io?.disconnect();
    if (this.timer !== null) clearTimeout(this.timer);
    this.mo = this.io = null;
  }

  /** Track a set of block containers for visibility (call after each scan). */
  track(containers: Iterable<Element>): void {
    if (!this.io) return;
    for (const c of containers) this.io.observe(c);
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
      // With no IntersectionObserver (or not yet observed) treat as visible.
      if (!this.io || this.visible.has(c)) nowVisible.push(c);
      else this.pending.add(c);
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
