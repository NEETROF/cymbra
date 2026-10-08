import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { sidepanel as enSidepanel } from "@/i18n/en/sidepanel.ts";
import { sidepanel as frSidepanel } from "@/i18n/fr/sidepanel.ts";
import { COPY_PENDING_ATTR, fillPage, setDocumentLanguage } from "@/i18n/index.ts";

// The side panel's page (localise-lingua-reading-surfaces D2, D5): its skeleton holds no text;
// filled from the French catalogue, every node holds byte for byte what the page held before this
// change, and the mark that hid the body is gone. sidepanel.ts is an entry script with no exported
// render: it holds no copy of its own (the views are the shared mounts).

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const HTML = readFileSync(join(root, "src/sidepanel/sidepanel.html"), "utf8");
const CSS = readFileSync(join(root, "src/sidepanel/sidepanel.css"), "utf8");

function page(): Document {
  return new DOMParser().parseFromString(HTML, "text/html");
}

/** Every `data-copy` node of the page, by selector, with the text the page held before. */
const TEXTS: Array<[string, string]> = [
  ["title", "Cymbra Lingua — révision"],
  ["h1", "Cymbra Lingua"],
  ["#views [data-view=review]", "Révision"],
  ["#views [data-view=stats]", "Statistiques"],
  ["#views [data-view=settings]", "Réglages"],
];

describe("the side panel's page, filled from the catalogue", () => {
  it("A page before its script: holds no text, and hides its body until filled", () => {
    const doc = page();
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    expect(doc.documentElement.hasAttribute("lang")).toBe(false);
    for (const [selector] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe("");
    expect(CSS).toMatch(/html\[data-copy-pending\] body \{\s*visibility: hidden;\s*\}/);
  });

  it("A surface without a spec today: the French catalogue gives every node the text the page held", () => {
    const doc = page();
    fillPage(doc, frSidepanel);
    setDocumentLanguage(doc, "fr");
    for (const [selector, text] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(text);
    expect(doc.querySelectorAll("[data-copy]").length).toBe(TEXTS.length);
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(doc.documentElement.lang).toBe("fr");
    // The skeleton is the one the script drives: the view buttons, the first active.
    expect(doc.querySelector("#views [data-view=review]")?.classList.contains("active")).toBe(true);
  });

  it("An English-native reader: the tabs are the English catalogue's, and the page says so", () => {
    const doc = page();
    fillPage(doc, enSidepanel);
    setDocumentLanguage(doc, "en");
    expect(doc.title).toBe("Cymbra Lingua — review");
    expect(doc.querySelector("#views [data-view=review]")?.textContent).toBe("Review");
    expect(doc.querySelector("#views [data-view=settings]")?.textContent).toBe("Settings");
    expect(doc.documentElement.lang).toBe("en");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
  });
});
