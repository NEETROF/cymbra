import { docOf, EXCLUDED_SELECTOR } from "./blocks.ts";

// Dynamic-content handling (task 1.4). A debounced MutationObserver turns DOM changes
// into a set of dirty block-level containers — never a whole-page re-walk. An
// IntersectionObserver prioritises what the reader can see: dirty containers that are
// on-screen are re-scanned immediately; off-screen ones are queued and re-scanned when
// they scroll into view, so a pathological SPA degrades gracefully instead of janking.
//
// Neither depends on the order its caller works in (fix-lingua-dynamic-rescan D1, D2). The first
// scan tracks the painted containers BEFORE `start()` creates the IntersectionObserver; they are
// remembered and observed on start. And a changed container that was never tracked — a section
// loaded into a container that held no text at first paint — is observed the moment it is queued,
// so it is rescanned once it is on screen instead of waiting forever.

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

/** Nearest block-level container of a node, or its document's body. */
function blockOf(node: Node): Element | null {
  const el = node.nodeType === Node.TEXT_NODE ? node.parentElement : (node as Element);
  if (!el) return null;
  // What the reader never reads — our own UI, a video player's caption line, code, form fields —
  // never schedules a rescan either.
  if (el.closest(EXCLUDED_SELECTOR)) return null;
  return el.closest(BLOCK_SELECTOR) ?? docOf(el).body;
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
  /** Tracked before `start()`: observed once there is an observer. */
  private readonly early = new Set<Element>();
  /** What the IntersectionObserver watches, so nothing is observed twice. */
  private watched = new WeakSet<Element>();

  constructor(private readonly opts: ReadingObserverOptions) {}

  /** Begin observing mutations under `root` for dynamic re-analysis. The observers are
   *  the root's own window's, so a book section's iframe is watched by its own. */
  start(root: Node = document.body): void {
    const win = (docOf(root).defaultView ?? window) as Window & typeof globalThis;
    this.mo = new win.MutationObserver((records) => this.onMutations(records));
    this.mo.observe(root, { childList: true, subtree: true, characterData: true });
    if (typeof win.IntersectionObserver !== "undefined") {
      this.io = new win.IntersectionObserver((entries) => this.onIntersections(entries));
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
    this.watched = new WeakSet();
  }

  /** Track a set of block containers for visibility (call after each scan). Before `start()`,
   *  they are remembered and observed as soon as it runs. */
  track(containers: Iterable<Element>): void {
    for (const c of containers) {
      if (this.io) this.watch(c);
      else this.early.add(c);
    }
  }

  private watch(container: Element): void {
    if (!this.io || this.watched.has(container)) return;
    this.watched.add(container);
    this.io.observe(container);
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
      // With no IntersectionObserver at all, everything counts as visible.
      if (!this.io || this.visible.has(c)) {
        nowVisible.push(c);
        continue;
      }
      // Off screen, or not known to be on screen yet: queued until it intersects. A container
      // that was never tracked is observed now — its first intersection drains it if it is on
      // screen (D2). Treating it as visible instead would rescan an infinite feed filling far
      // below the fold at once, undoing the visible-first priority.
      this.pending.add(c);
      this.watch(c);
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
