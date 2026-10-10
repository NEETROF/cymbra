import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { JSDOM } from "jsdom";
import { describe, expect, it } from "vitest";
import { languageName } from "@/analyzer/language-labels.ts";
import { pairsOf, studiedOf } from "@/analyzer/pairs.ts";
import type { NativeLanguage } from "@/analyzer/types.ts";
import { drawer as enDrawer } from "@/i18n/en/drawer.ts";
import { settings as enSettings } from "@/i18n/en/settings.ts";
import { drawer as esDrawer } from "@/i18n/es/drawer.ts";
import { settings as esSettings } from "@/i18n/es/settings.ts";
import { drawer as frDrawer } from "@/i18n/fr/drawer.ts";
import { settings as frSettings } from "@/i18n/fr/settings.ts";

// The Safari host app's activation page (apps/lingua-apple, localise-lingua-apple-host D2): Main.html
// keeps its French text in place, each text under a data-copy key; copy.js holds the page's copy per
// language — the French byte for byte Main.html's (M23), the English and Spanish drafts after it —
// and fills the page on DOMContentLoaded in the language ViewController.swift injects before load,
// saying it in `lang`. Script.js takes the macOS-before-13 variants from the same table. The page is
// loaded here as the app loads it: window.linguaLanguage set first, copy.js then Script.js, deferred.

const RESOURCES = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "lingua-apple",
  "Shared (App)",
  "Resources",
);
const html = readFileSync(join(RESOURCES, "Base.lproj", "Main.html"), "utf8");
const copyJs = readFileSync(join(RESOURCES, "copy.js"), "utf8");
const scriptJs = readFileSync(join(RESOURCES, "Script.js"), "utf8");

type Table = Record<string, string>;

interface LinguaCopy {
  COPY: Record<string, Table>;
  FRAGMENTS: string[];
  language(): string;
  text(key: string): string;
  fill(doc: Document, lang: string): void;
}

interface Page {
  document: Document;
  copy: LinguaCopy;
  show(platform: string, enabled?: boolean, useSettingsInsteadOfPreferences?: boolean): void;
}

/** The page as the host app shows it, `language` being what Swift injected (nothing: opened by hand). */
function load(language?: string): Page {
  const dom = new JSDOM(html, { runScripts: "outside-only" });
  const window = dom.window as typeof dom.window & {
    linguaLanguage?: string;
    linguaCopy: LinguaCopy;
    show: Page["show"];
  };
  if (language !== undefined) window.linguaLanguage = language;
  window.eval(copyJs);
  window.eval(scriptJs);
  window.document.dispatchEvent(new dom.window.Event("DOMContentLoaded"));
  return { document: window.document, copy: window.linguaCopy, show: window.show };
}

/** The page's texts by key, as copy.js writes them: markup for the fragments, text otherwise. */
function pageTexts(document: Document, fragments: string[]): Table {
  const texts: Table = {};
  for (const node of document.querySelectorAll<HTMLElement>("[data-copy]")) {
    const key = node.dataset.copy!;
    expect(texts, `data-copy="${key}" twice`).not.toHaveProperty(key);
    texts[key] = fragments.includes(key) ? node.innerHTML : node.textContent!;
  }
  for (const node of document.querySelectorAll<HTMLElement>("[data-copy-alt]")) {
    texts[node.dataset.copyAlt!] = node.getAttribute("alt")!;
  }
  return texts;
}

// Before macOS 13, Safari called its settings "Préférences": what Script.js showed then, pinned.
const BEFORE_13 = {
  stateUnknownPreferences: "Active Cymbra Lingua dans les préférences de Safari, section Extensions.",
  stateOnPreferences: "Extension active. Tu peux la désactiver dans les préférences de Safari, section Extensions.",
  stateOffPreferences: "Extension désactivée. Active-la dans les préférences de Safari, section Extensions.",
};

const LANGUAGES = ["fr", "en", "es"];

/** The first language a native language's pairs study, as its page writes it inside a sentence. */
const named1 = (native: string): string =>
  languageName(native as NativeLanguage, studiedOf(pairsOf(native)[0]!) as NativeLanguage).toLowerCase();

describe("the activation page's copy table", () => {
  const { copy, document } = load();

  it("holds the three languages, every key in each, nothing empty", () => {
    expect(Object.keys(copy.COPY).sort()).toEqual([...LANGUAGES].sort());
    const keys = Object.keys(copy.COPY.fr).sort();
    for (const language of LANGUAGES) {
      expect(Object.keys(copy.COPY[language]).sort(), language).toEqual(keys);
      for (const [key, text] of Object.entries(copy.COPY[language])) {
        expect(typeof text, `${language}.${key}`).toBe("string");
        expect(text.trim(), `${language}.${key}`).not.toBe("");
      }
    }
  });

  it("is the page's French, byte for byte, one entry per data-copy node and the icon's alt", () => {
    const onPage = pageTexts(document, copy.FRAGMENTS);
    const { stateUnknownPreferences, stateOnPreferences, stateOffPreferences, ...shown } = copy.COPY.fr;
    expect(onPage).toEqual(shown);
    expect({ stateUnknownPreferences, stateOnPreferences, stateOffPreferences }).toEqual(BEFORE_13);
  });

  it("marks exactly the entries that hold markup as fragments", () => {
    for (const language of LANGUAGES) {
      for (const [key, text] of Object.entries(copy.COPY[language])) {
        expect(text.includes("<"), `${language}.${key}`).toBe(copy.FRAGMENTS.includes(key));
      }
    }
    for (const key of copy.FRAGMENTS) expect(copy.COPY.fr[key]).toContain("<strong>");
  });

  it("names every language a native language's pairs study, in the pairs' order, and never its own (enable-lingua-french D7)", () => {
    // Each page speaks to the natives of its language: French natives study English and Spanish
    // (en-fr, es-fr), English natives Spanish and French (es-en, fr-en), Spanish natives English and
    // French (en-es, fr-es) — read from the bundle's list, named as the extension names them.
    const named = (native: NativeLanguage) =>
      pairsOf(native).map((pair) => languageName(native, studiedOf(pair) as NativeLanguage).toLowerCase());
    expect(named("fr")).toEqual(["anglais", "espagnol"]);
    expect(named("en")).toEqual(["spanish", "french"]);
    expect(named("es")).toEqual(["inglés", "francés"]);
    for (const native of ["fr", "en", "es"] as const) {
      const own = languageName(native, native).toLowerCase();
      for (const key of ["lede", "step3"]) {
        const text = copy.COPY[native][key].toLowerCase();
        const at = named(native).map((name) => text.indexOf(name));
        expect(
          at.every((i) => i >= 0),
          `${native}.${key} names ${named(native).join(" and ")}`,
        ).toBe(true);
        expect(
          [...at].sort((a, b) => a - b),
          `${native}.${key}: in the pairs' order`,
        ).toEqual(at);
        expect(text, `${native}.${key} names its own language`).not.toContain(own);
      }
    }
  });

  it("says, in its last step, where a second language is ticked: the extension's settings, its language tab", () => {
    // Réglages' first tab holds the studied languages' boxes and a level block per ticked language
    // (settings-view.ts): the page names both as the extension does, in its language.
    const places = { fr: [frDrawer, frSettings], en: [enDrawer, enSettings], es: [esDrawer, esSettings] } as const;
    for (const [language, [drawer, settings]] of Object.entries(places)) {
      const step3 = copy.COPY[language].step3;
      expect(step3, language).toContain(`<strong>${drawer.settings}</strong>`);
      expect(step3, language).toContain(`<strong>${settings.tabLanguage}</strong>`);
      // The first language's level is chosen before the second is ticked.
      const first = step3.toLowerCase().indexOf(named1(language));
      expect(first, language).toBeGreaterThanOrEqual(0);
      expect(first, language).toBeLessThan(step3.toLowerCase().indexOf(drawer.settings.toLowerCase()));
    }
  });

  it("is in English and Spanish no copy of the French", () => {
    for (const language of ["en", "es"]) {
      for (const [key, text] of Object.entries(copy.COPY[language])) {
        if (key !== "title") expect(text, `${language}.${key} left in French`).not.toBe(copy.COPY.fr[key]);
      }
      expect(copy.COPY[language].title).toBe("Cymbra Lingua");
    }
  });

  it("speaks Spanish to the reader, not to readers (tú, no vosotros)", () => {
    const spanish = Object.values(copy.COPY.es).join(" ");
    expect(spanish).not.toMatch(/\b(vosotros|vuestr[oa]s?|abrid|activad|recargad|elegid|pod[eé]is)\b/i);
    expect(spanish).toMatch(/\b(abre|activa|elige|recarga|puedes)\b/);
  });
});

describe("filling the page", () => {
  it("leaves a French page untouched, asked for French, nothing, or a language it has not", () => {
    const plain = load().document.documentElement.outerHTML;
    for (const language of ["fr", undefined, "de", "", "FR"]) {
      const page = load(language);
      expect(page.document.documentElement.outerHTML, String(language)).toBe(plain);
      expect(page.document.documentElement.lang).toBe("fr");
      expect(page.copy.language()).toBe("fr");
    }
  });

  for (const language of ["en", "es"]) {
    it(`fills the page in ${language} and says it in lang`, () => {
      const page = load(language);
      expect(page.document.documentElement.lang).toBe(language);
      expect(page.copy.language()).toBe(language);
      const { stateUnknownPreferences, stateOnPreferences, stateOffPreferences, ...shown } = page.copy.COPY[language];
      expect(pageTexts(page.document, page.copy.FRAGMENTS)).toEqual(shown);
      // The fragments' markup is the table's, as markup.
      expect(page.document.querySelectorAll("[data-copy='step1'] strong")).toHaveLength(1);
      expect(page.document.querySelector("[data-copy='step1']")!.textContent).not.toContain("<");
      // The macOS-before-13 variants follow the same language.
      page.show("mac", true, false);
      expect(page.document.querySelector("p.platform-mac.state-on")!.textContent).toBe(stateOnPreferences);
      expect(page.document.querySelector("p.platform-mac.state-off")!.textContent).toBe(stateOffPreferences);
      expect(page.document.querySelector("p.platform-mac.state-unknown")!.textContent).toBe(stateUnknownPreferences);
    });
  }

  it("shows the French variants before macOS 13, as before", () => {
    const page = load("fr");
    page.show("mac", false, false);
    expect(page.document.querySelector("p.platform-mac.state-on")!.textContent).toBe(BEFORE_13.stateOnPreferences);
    expect(page.document.querySelector("p.platform-mac.state-off")!.textContent).toBe(BEFORE_13.stateOffPreferences);
    expect(page.document.querySelector("p.platform-mac.state-unknown")!.textContent).toBe(
      BEFORE_13.stateUnknownPreferences,
    );
    expect(page.document.body.classList.contains("state-off")).toBe(true);
  });

  it("keeps show()'s platform and state classes", () => {
    const page = load("es");
    page.show("ios");
    expect(page.document.body.className).toBe("platform-ios");
    page.show("mac", true, true);
    expect(page.document.body.classList.contains("state-on")).toBe(true);
    // Settings, not preferences, from macOS 13 on: the state texts are the table's plain ones.
    expect(page.document.querySelector("p.platform-mac.state-on")!.textContent).toBe(page.copy.COPY.es.stateOn);
  });
});
