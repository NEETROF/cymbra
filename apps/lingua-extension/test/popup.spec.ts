import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it, vi } from "vitest";
import { popup as enPopup } from "@/i18n/en/popup.ts";
import { popup as esPopup } from "@/i18n/es/popup.ts";
import { popup as frPopup } from "@/i18n/fr/popup.ts";
import {
  COPY_PENDING_ATTR,
  fillPageInLanguage,
  INTERFACE_LANGUAGE_KEY,
  type InterfaceLanguage,
  NODE_SLOT,
  renderAround,
} from "@/i18n/index.ts";
import { PENDING_RULE, pageArea, refusingArea, REVEAL_KEYFRAMES } from "./helpers.ts";

// The popup's page (localise-lingua-reading-surfaces D2, D5): its skeleton holds no text; opened as
// popup.ts opens it — `fillPageInLanguage` over the preferences area, with the popup's modules —
// every node and attribute holds byte for byte what the page held before this change in French
// (the inventory below is that page's text), the page says its language, and the mark that hid the
// body is gone, a storage that cannot be read included. popup.ts itself is an entry script with no
// exported render: its own literals are guarded by its exit from the lint's baseline.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const HTML = readFileSync(join(root, "src/popup/popup.html"), "utf8");
const CSS = readFileSync(join(root, "src/popup/popup.css"), "utf8");

/** The popup's modules by interface language, as popup.ts picks them. */
const POPUP: Record<InterfaceLanguage, typeof frPopup> = { fr: frPopup, en: enPopup, es: esPopup };

function page(): Document {
  return new DOMParser().parseFromString(HTML, "text/html");
}

/** Every `data-copy` text node of the page, by selector, with the text the page held before. */
const TEXTS: Array<[string, string]> = [
  ["title", "Cymbra Lingua"],
  ["h1", "Cymbra Lingua"],
  ["#session-lost", "Session expirée — reconnecte-toi pour synchroniser. Tes mots restent sur cet appareil."],
  ["#enabled-label", "Surlignage activé"],
  ["#disabled-note", "Surlignage désactivé sur toutes les pages."],
  ["#setup .note", "Cette page n'est pas encore analysée."],
  ["#analyse", "Analyser cette page"],
  ["#always", "Toujours surligner (toutes les pages)"],
  ["#pct", "—"],
  ["#pct-label", "de mots connus sur cette page"],
  ["#analysed > div:nth-of-type(4) span", "Mots analysés"],
  ["#analysed > div:nth-of-type(5) span", "Mots inconnus"],
  ["#analysed > div:nth-of-type(6) span", "dont mots différents"],
  ["#controls > .row span", "Mots suivis"],
  ["#level-cta", "Choisis ton niveau"],
  ["#level-edit", "Modifier"],
  [".deck span", "Deck (en cours)"],
  ["#account-signin", "Se connecter pour synchroniser"],
  ["#account-in .row span", "Connecté"],
  ["#account-manage", "Compte et synchronisation"],
  ["#open-library", "Bibliothèque (livres EPUB)"],
  ["#open-stats", "Statistiques d'apprentissage"],
  [".settings-title", "Réglages"],
];

/** Every `data-copy-<attribute>` of the page, with the value the page held before. */
const ATTRIBUTES: Array<[string, string, string]> = [
  ["#settings-open", "aria-label", "Réglages"],
  ["#settings-open", "title", "Réglages"],
  ["#settings-back", "aria-label", "Retour"],
  ["#settings-back", "title", "Retour"],
];

function expectFrench(doc: Document): void {
  for (const [selector, text] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe(text);
  for (const [selector, attribute, value] of ATTRIBUTES) {
    expect(doc.querySelector(selector)?.getAttribute(attribute), `${selector} ${attribute}`).toBe(value);
  }
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("the popup's page, filled from the catalogue", () => {
  it("A page before its script: holds no text, and hides its body until filled", () => {
    const doc = page();
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    expect(doc.documentElement.hasAttribute("lang")).toBe(false);
    for (const [selector] of TEXTS) expect(doc.querySelector(selector)?.textContent, selector).toBe("");
    for (const [selector, attribute] of ATTRIBUTES) {
      expect(doc.querySelector(selector)?.hasAttribute(attribute), `${selector} ${attribute}`).toBe(false);
    }
    // Hidden while pending — and shown after a moment even if the script never fills it.
    expect(CSS).toMatch(PENDING_RULE);
    expect(CSS).toMatch(REVEAL_KEYFRAMES);
  });

  it("A surface without a spec today: opened as popup.ts opens it, every node holds the text the page held", async () => {
    const doc = page();
    const { language, copy } = await fillPageInLanguage(doc, pageArea(), (l) => POPUP[l]);
    expect(language).toBe("fr");
    expect(copy).toBe(frPopup);
    expectFrench(doc);
    // Every keyed node and attribute of the page is in the inventory: none goes unasserted.
    expect(doc.querySelectorAll("[data-copy]").length).toBe(TEXTS.length);
    const attributes = [...doc.querySelectorAll("*")].flatMap((el) =>
      [...el.attributes].filter((a) => a.name.startsWith("data-copy-") && a.name !== COPY_PENDING_ATTR),
    );
    expect(attributes.length).toBe(ATTRIBUTES.length);
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(doc.documentElement.lang).toBe("fr");
  });

  it("A storage that cannot be read: the page still shows, in French, and says so", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const doc = page();
    const { language } = await fillPageInLanguage(doc, refusingArea(), (l) => POPUP[l]);
    expect(language).toBe("fr");
    expectFrench(doc);
    expect(doc.documentElement.lang).toBe("fr");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(warn).toHaveBeenCalledWith(expect.stringContaining("[Cymbra Lingua]"), expect.any(Error));
  });

  it("Every reader today: the texts the script assembles are the ones the page assembled", () => {
    // « Réviser (<span id="due">0</span>) » and « <span>Niveau</span> : <b>—</b> » were the page's.
    expect(frPopup.review("0")).toBe("Réviser (0)");
    expect(frPopup.levelLine("Niveau d'anglais", "B1")).toBe("Niveau d'anglais : B1");
    expect(frPopup.beginner).toBe("Débutant");
    // The level line as the script renders it into the page: the label, then the bold level.
    const doc = page();
    const line = doc.getElementById("level-line")!;
    const level = doc.getElementById("level-current")!;
    level.textContent = "B1";
    renderAround(line, frPopup.levelLine("Niveau d'anglais", NODE_SLOT), level);
    expect(line.textContent).toBe("Niveau d'anglais : B1");
    expect(line.querySelector("b")).toBe(level);
  });

  it("An English-native reader: the page is the English catalogue's, and says so", async () => {
    const doc = page();
    const { language } = await fillPageInLanguage(doc, pageArea({ [INTERFACE_LANGUAGE_KEY]: "en" }), (l) => POPUP[l]);
    expect(language).toBe("en");
    expect(doc.title).toBe("Cymbra Lingua");
    expect(doc.querySelector("#enabled-label")?.textContent).toBe("Highlighting on");
    expect(doc.querySelector("#open-library")?.textContent).toBe("Library (EPUB books)");
    expect(doc.querySelector("#settings-open")?.getAttribute("aria-label")).toBe("Settings");
    expect(enPopup.review("3")).toBe("Review (3)");
    expect(doc.documentElement.lang).toBe("en");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
  });
});
