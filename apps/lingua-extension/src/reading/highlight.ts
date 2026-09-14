import type { ResolvedToken } from "./scan.ts";

// CSS Custom Highlight API rendering: two registries, zero DOM mutation. The tints
// and the ::highlight() rules come from the injected token sheet (tokens.css) — no
// color literal lives here (the "no hex outside the token sheet" rule). Everything
// here is feature-detected: on a browser without CSS.highlights it silently no-ops.

/** Highlight registry names; must match the ::highlight() rules in tokens.css. */
export const HL_UNKNOWN = "cymbra-lingua-unknown";
export const HL_LEARNING = "cymbra-lingua-learning";

const STYLE_ID = "cymbra-lingua-style";

/** Whether the CSS Custom Highlight API is available in this browser. */
export function highlightsSupported(): boolean {
  return typeof CSS !== "undefined" && !!CSS.highlights;
}

/** The constructable sheet, built once from the token text, re-adopted cheaply. */
let adoptedSheet: CSSStyleSheet | null = null;

function canAdopt(doc: Document): boolean {
  return "adoptedStyleSheets" in doc && typeof CSSStyleSheet === "function";
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
      if (!adoptedSheet) {
        adoptedSheet = new CSSStyleSheet();
        adoptedSheet.replaceSync(cssText);
      }
      if (!doc.adoptedStyleSheets.includes(adoptedSheet)) {
        doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, adoptedSheet];
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

/** The painted tokens grouped by their block container, and the containers near the viewport. */
let byContainer = new Map<Element, ResolvedToken[]>();
const nearViewport = new Set<Element>();
let windowObserver: IntersectionObserver | null = null;
let paintFrame = 0;

function paintRanges(tokens: Iterable<ResolvedToken>): void {
  const learning: Range[] = [];
  const unknown: Range[] = [];
  for (const r of tokens) {
    if (!r.range) continue;
    (r.token.class === "Learning" ? learning : unknown).push(r.range);
  }
  CSS.highlights.set(HL_LEARNING, new Highlight(...learning));
  CSS.highlights.set(HL_UNKNOWN, new Highlight(...unknown));
}

function* nearTokens(): Generator<ResolvedToken> {
  for (const c of nearViewport) yield* byContainer.get(c) ?? [];
}

function schedulePaint(): void {
  if (paintFrame) return;
  paintFrame = requestAnimationFrame(() => {
    paintFrame = 0;
    paintRanges(nearTokens());
  });
}

/**
 * Paint the resolved tokens into the two highlight registries by class. No-op when
 * the API is missing. Only the blocks within a viewport of the visible area are painted:
 * WebKit re-evaluates every registered range on each rendering update, so a long page
 * (15 000 ranges) stalls Safari for seconds. An IntersectionObserver keeps the painted
 * window following the scroll; without one, everything is painted at once.
 */
export function render(resolved: ResolvedToken[]): void {
  if (!highlightsSupported()) return;
  windowObserver?.disconnect();
  nearViewport.clear();
  byContainer = new Map();
  if (typeof IntersectionObserver === "undefined") {
    paintRanges(resolved);
    return;
  }
  for (const r of resolved) {
    const container = r.container ?? r.range.startContainer.parentElement;
    if (!container) continue;
    const list = byContainer.get(container);
    if (list) list.push(r);
    else byContainer.set(container, [r]);
  }
  windowObserver = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (e.isIntersecting) nearViewport.add(e.target);
        else nearViewport.delete(e.target);
      }
      schedulePaint();
    },
    { rootMargin: WINDOW_MARGIN },
  );
  // The observer reports every target once on observe, which paints the first window.
  for (const c of byContainer.keys()) windowObserver.observe(c);
  if (byContainer.size === 0) paintRanges([]);
}

/** Remove both highlight registries. */
export function clear(): void {
  if (!highlightsSupported()) return;
  windowObserver?.disconnect();
  windowObserver = null;
  nearViewport.clear();
  byContainer = new Map();
  if (paintFrame) cancelAnimationFrame(paintFrame);
  paintFrame = 0;
  CSS.highlights.delete(HL_LEARNING);
  CSS.highlights.delete(HL_UNKNOWN);
}
