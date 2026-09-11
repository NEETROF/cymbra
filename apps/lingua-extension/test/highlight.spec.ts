import { beforeEach, describe, expect, it } from "vitest";
import { injectPageStyles } from "@/reading/highlight.ts";

const STYLE_ID = "cymbra-lingua-style";
const CSS = ":root{--cymbra-lingua-unknown-fill:rgba(0,0,0,.1)}::highlight(cymbra-lingua-unknown){background:var(--cymbra-lingua-unknown-fill)}";

// jsdom exposes no `adoptedStyleSheets`, so these exercise the <style> fallback —
// the path that must self-heal when a morphing SPA (GitHub) strips the node.
beforeEach(() => {
  document.getElementById(STYLE_ID)?.remove();
});

describe("injectPageStyles", () => {
  it("injects the token sheet into the page", () => {
    injectPageStyles(CSS);
    const el = document.getElementById(STYLE_ID);
    expect(el).not.toBeNull();
    expect(el!.textContent).toContain("::highlight(cymbra-lingua-unknown)");
  });

  it("re-injects the sheet after a single-page-app navigation strips it", () => {
    injectPageStyles(CSS);
    // Simulate a morphing SPA (Turbo/idiomorph) removing our injected node.
    document.getElementById(STYLE_ID)!.remove();
    expect(document.getElementById(STYLE_ID)).toBeNull();
    // The next paint must restore the styling rather than assume it persisted.
    injectPageStyles(CSS);
    expect(document.getElementById(STYLE_ID)).not.toBeNull();
  });

  it("does not stack duplicate style nodes on repeated calls", () => {
    injectPageStyles(CSS);
    injectPageStyles(CSS);
    injectPageStyles(CSS);
    expect(document.querySelectorAll(`#${STYLE_ID}`).length).toBe(1);
  });
});
