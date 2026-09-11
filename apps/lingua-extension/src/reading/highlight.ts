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

/**
 * Inject the token sheet (which defines the palette custom properties and the two
 * ::highlight() rules) into the page once. The caller passes the sheet text so this
 * module imports no asset and stays testable.
 */
export function injectPageStyles(cssText: string, doc: Document = document): void {
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
