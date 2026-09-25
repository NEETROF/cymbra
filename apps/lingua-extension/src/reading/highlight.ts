import type { ResolvedToken } from "./scan.ts";

// CSS Custom Highlight API rendering: two registries, zero DOM mutation. The tints
// and the ::highlight() rules come from the injected token sheet (tokens.css) — no
// color literal lives here (the "no hex outside the token sheet" rule). Everything
// here is feature-detected: on a browser without CSS.highlights it silently no-ops.

/** Highlight registry names; must match the ::highlight() rules in tokens.css. */
export const HL_UNKNOWN = "cymbra-lingua-unknown";
export const HL_LEARNING = "cymbra-lingua-learning";

const STYLE_ID = "cymbra-lingua-style";

/** A window with its own highlight registry: the page's, or a book section's iframe. */
type HighlightWindow = Window & typeof globalThis;

/** Where highlights are painted. `CSS.highlights` is per window: a book section rendered in an
 *  iframe paints through the iframe's registry, not the page's. */
export interface PaintTarget {
  /** The document whose ranges are painted. */
  doc: Document;
  /**
   * Paint only the blocks within a viewport of the visible area — a long web page, which
   * stalls Safari otherwise. Off in the reader: a book section is painted whole, so turning
   * a page inside it paints nothing new (add-lingua-reader D4).
   */
  window: boolean;
}

/** The page the content script reads, painted by viewport window. */
function pageTarget(): PaintTarget {
  return { doc: document, window: true };
}

function windowOf(doc: Document): HighlightWindow | null {
  return (doc.defaultView as HighlightWindow | null) ?? null;
}

/** Whether the CSS Custom Highlight API is available in this browser. */
export function highlightsSupported(win: HighlightWindow | null = window): boolean {
  return !!win && typeof win.CSS !== "undefined" && !!win.CSS.highlights;
}

/** The constructable sheet of each document, built once from the token text, re-adopted cheaply.
 *  One per document: a sheet can only be adopted by the document whose window constructed it. */
const adoptedSheets = new WeakMap<Document, CSSStyleSheet>();

function canAdopt(doc: Document): boolean {
  return "adoptedStyleSheets" in doc && typeof windowOf(doc)?.CSSStyleSheet === "function";
}

/**
 * Ensure the token sheet (the `--cymbra-lingua-*` palette **and** the two
 * `::highlight()` rules) is applied to the page. Idempotent and **self-healing**:
 * call it again to restore styling that a single-page-app navigation stripped.
 *
 * A morphing SPA (GitHub's Turbo/idiomorph) reconciles the DOM to the incoming
 * markup, silently removing an injected `<style>` node it does not know about.
 * When that happens the `::highlight()` rules and the `:root` palette they resolve
 * against both vanish, so highlights stop painting — while clicks still resolve
 * against in-memory ranges, which is exactly the "highlights gone, popup fine"
 * symptom. So prefer a **constructable stylesheet adopted by the document**:
 * `adoptedStyleSheets` is a document property, not a node, and morphing never
 * touches it. Fall back to a `<style>` node (re-added when missing) where
 * constructable sheets are unavailable.
 */
export function injectPageStyles(cssText: string, doc: Document = document): void {
  if (canAdopt(doc)) {
    try {
      let sheet = adoptedSheets.get(doc);
      if (!sheet) {
        sheet = new (windowOf(doc)!.CSSStyleSheet)();
        sheet.replaceSync(cssText);
        adoptedSheets.set(doc, sheet);
      }
      if (!doc.adoptedStyleSheets.includes(sheet)) {
        doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet];
      }
      return;
    } catch {
      // Constructable sheets misbehaved — fall through to the <style> node.
    }
  }
  if (doc.getElementById(STYLE_ID)) return;
  const style = doc.createElement("style");
  style.id = STYLE_ID;
  style.textContent = cssText;
  doc.documentElement.appendChild(style);
}

/** How far past the viewport a block still gets painted: one viewport above and below. */
const WINDOW_MARGIN = "100% 0px";

/** One document's painting: its tokens by block container, and the containers near the viewport. */
class Painter {
  private byContainer = new Map<Element, ResolvedToken[]>();
  private readonly nearViewport = new Set<Element>();
  private windowObserver: IntersectionObserver | null = null;
  private paintFrame = 0;

  constructor(private readonly win: HighlightWindow) {}

  private paintRanges(tokens: Iterable<ResolvedToken>): void {
    const learning: Range[] = [];
    const unknown: Range[] = [];
    for (const r of tokens) {
      if (!r.range) continue;
      (r.token.class === "Learning" ? learning : unknown).push(r.range);
    }
    // The registry and the Highlight constructor of the document's own window.
    this.win.CSS.highlights.set(HL_LEARNING, new this.win.Highlight(...learning));
    this.win.CSS.highlights.set(HL_UNKNOWN, new this.win.Highlight(...unknown));
  }

  private *nearTokens(): Generator<ResolvedToken> {
    for (const c of this.nearViewport) yield* this.byContainer.get(c) ?? [];
  }

  private schedulePaint(): void {
    if (this.paintFrame) return;
    this.paintFrame = this.win.requestAnimationFrame(() => {
      this.paintFrame = 0;
      this.paintRanges(this.nearTokens());
    });
  }

  render(resolved: ResolvedToken[], windowed: boolean): void {
    this.stopWindow();
    if (!windowed || typeof this.win.IntersectionObserver === "undefined") {
      this.paintRanges(resolved);
      return;
    }
    for (const r of resolved) {
      const container = r.container ?? r.range.startContainer.parentElement;
      if (!container) continue;
      const list = this.byContainer.get(container);
      if (list) list.push(r);
      else this.byContainer.set(container, [r]);
    }
    const observer = new this.win.IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) this.nearViewport.add(e.target);
          else this.nearViewport.delete(e.target);
        }
        this.schedulePaint();
      },
      { rootMargin: WINDOW_MARGIN },
    );
    this.windowObserver = observer;
    // The observer reports every target once on observe, which paints the first window.
    for (const c of this.byContainer.keys()) observer.observe(c);
    if (this.byContainer.size === 0) this.paintRanges([]);
  }

  clear(): void {
    this.stopWindow();
    this.win.CSS.highlights.delete(HL_LEARNING);
    this.win.CSS.highlights.delete(HL_UNKNOWN);
  }

  private stopWindow(): void {
    this.windowObserver?.disconnect();
    this.windowObserver = null;
    this.nearViewport.clear();
    this.byContainer = new Map();
    if (this.paintFrame) this.win.cancelAnimationFrame(this.paintFrame);
    this.paintFrame = 0;
  }
}

const painters = new WeakMap<Document, Painter>();

function painterFor(doc: Document): Painter | null {
  const win = windowOf(doc);
  if (!highlightsSupported(win)) return null;
  let painter = painters.get(doc);
  if (!painter) {
    painter = new Painter(win!);
    painters.set(doc, painter);
  }
  return painter;
}

/**
 * Paint the resolved tokens into the two highlight registries of the target's window, by
 * class. No-op when the API is missing. On a web page only the blocks within a viewport of
 * the visible area are painted: WebKit re-evaluates every registered range on each rendering
 * update, so a long page (15 000 ranges) stalls Safari for seconds. An IntersectionObserver
 * keeps the painted window following the scroll; without one — or with `window: false`, as
 * the reader asks — everything is painted at once.
 */
export function render(resolved: ResolvedToken[], target: PaintTarget = pageTarget()): void {
  painterFor(target.doc)?.render(resolved, target.window);
}

/** Remove both highlight registries of the target's window. */
export function clear(target: Pick<PaintTarget, "doc"> = pageTarget()): void {
  painterFor(target.doc)?.clear();
}
