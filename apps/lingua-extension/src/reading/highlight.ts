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

/**
 * Paint the resolved tokens into the two highlight registries by class. No-op when
 * the API is missing. Learning and Unknown each get one Highlight of all their ranges.
 */
export function render(resolved: ResolvedToken[]): void {
  if (!highlightsSupported()) return;
  const learning: Range[] = [];
  const unknown: Range[] = [];
  for (const r of resolved) {
    if (!r.range) continue;
    (r.token.class === "Learning" ? learning : unknown).push(r.range);
  }
  CSS.highlights.set(HL_LEARNING, new Highlight(...learning));
  CSS.highlights.set(HL_UNKNOWN, new Highlight(...unknown));
}

/** Remove both highlight registries. */
export function clear(): void {
  if (!highlightsSupported()) return;
  CSS.highlights.delete(HL_LEARNING);
  CSS.highlights.delete(HL_UNKNOWN);
}
