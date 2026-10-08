import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";
import { sidepanel as enSidepanel } from "@/i18n/en/sidepanel.ts";
import { sidepanel as esSidepanel } from "@/i18n/es/sidepanel.ts";
import { sidepanel as frSidepanel } from "@/i18n/fr/sidepanel.ts";
import { COPY_PENDING_ATTR, fillPageInLanguage, INTERFACE_LANGUAGE_KEY, type InterfaceLanguage } from "@/i18n/index.ts";
import { PENDING_RULE, pageArea, refusingArea, REVEAL_KEYFRAMES } from "./helpers.ts";

// The side panel's page (localise-lingua-reading-surfaces D2, D5): its skeleton holds no text;
// opened as sidepanel.ts opens it — `fillPageInLanguage` over the preferences area, with the page's
// modules — every node holds byte for byte what the page held before this change in French, the page
// says its language, and the mark that hid the body is gone, a storage that cannot be read included.
// sidepanel.ts is an entry script with no exported render: it holds no copy of its own (the page's
// modules and the views are panel.ts's, whose start review-stats-copy.spec.ts drives).

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const HTML = readFileSync(join(root, "src/sidepanel/sidepanel.html"), "utf8");
const CSS = readFileSync(join(root, "src/sidepanel/sidepanel.css"), "utf8");

/** The page's modules by interface language, as sidepanel.ts picks them. */
const SIDEPANEL: Record<InterfaceLanguage, typeof frSidepanel> = { fr: frSidepanel, en: enSidepanel, es: esSidepanel };

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

afterEach(() => {
  vi.restoreAllMocks();
});

describe("the side panel's page, filled from the catalogue", () => {
  it("A page before its script: holds no text, and hides its body until filled", () => {
    const doc = page();
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    expect(doc.documentElement.hasAttribute("lang")).toBe(false);
    for (const [selector] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe("");
    // Hidden while pending — and shown after a moment even if the script never fills it.
    expect(CSS).toMatch(PENDING_RULE);
    expect(CSS).toMatch(REVEAL_KEYFRAMES);
  });

  it("A surface without a spec today: opened as sidepanel.ts opens it, every node holds the text the page held", async () => {
    const doc = page();
    await fillPageInLanguage(doc, pageArea(), (l) => SIDEPANEL[l]);
    for (const [selector, text] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(text);
    expect(doc.querySelectorAll("[data-copy]").length).toBe(TEXTS.length);
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(doc.documentElement.lang).toBe("fr");
    // The skeleton is the one the script drives: the view buttons, the first active.
    expect(doc.querySelector("#views [data-view=review]")?.classList.contains("active")).toBe(true);
  });

  it("A storage that cannot be read: the page still shows, in French, and says so", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const doc = page();
    await fillPageInLanguage(doc, refusingArea(), (l) => SIDEPANEL[l]);
    for (const [selector, text] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(text);
    expect(doc.documentElement.lang).toBe("fr");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
  });

  it("An English-native reader: the tabs are the English catalogue's, and the page says so", async () => {
    const doc = page();
    await fillPageInLanguage(doc, pageArea({ [INTERFACE_LANGUAGE_KEY]: "en" }), (l) => SIDEPANEL[l]);
    expect(doc.title).toBe("Cymbra Lingua — review");
    expect(doc.querySelector("#views [data-view=review]")?.textContent).toBe("Review");
    expect(doc.querySelector("#views [data-view=settings]")?.textContent).toBe("Settings");
    expect(doc.documentElement.lang).toBe("en");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
  });
});
