import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { reader as enReader } from "@/i18n/en/reader.ts";
import { reader as frReader } from "@/i18n/fr/reader.ts";
import { COPY_PENDING_ATTR, fillPage, setDocumentLanguage } from "@/i18n/index.ts";
import { ReaderApp, type ReaderDeps } from "@/reader/app.ts";
import { COPY, readerCopy, readerModule } from "@/reader/copy.ts";
import { type Library, titleFromName } from "@/reader/library.ts";
import type { AsyncStorageArea } from "@/state/storage.ts";

// The reader's page and copy (localise-lingua-reading-surfaces D1, D2, D4): reader.html's title
// filled from the catalogue, `COPY` the French in the shape the page reads (reader-app.spec asserts
// it unchanged), and the page built in English — its figures written as English writes them.

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const HTML = readFileSync(join(root, "src/reader/reader.html"), "utf8");
const CSS = readFileSync(join(root, "src/reader/reader.css"), "utf8");

describe("the reader's page, filled from the catalogue", () => {
  it("A page before its script: holds no text, and hides its body until filled", () => {
    const doc = new DOMParser().parseFromString(HTML, "text/html");
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(true);
    expect(doc.documentElement.hasAttribute("lang")).toBe(false);
    expect(doc.title).toBe("");
    expect(CSS).toMatch(/html\[data-copy-pending\] body \{\s*visibility: hidden;\s*\}/);
  });

  it("Every reader today: the French catalogue gives the page the title it held", () => {
    const doc = new DOMParser().parseFromString(HTML, "text/html");
    fillPage(doc, readerModule("fr"));
    setDocumentLanguage(doc, "fr");
    expect(doc.title).toBe("Bibliothèque — Cymbra Lingua");
    expect(doc.querySelectorAll("[data-copy]").length).toBe(1);
    expect(doc.documentElement.hasAttribute(COPY_PENDING_ATTR)).toBe(false);
    expect(doc.documentElement.lang).toBe("fr");
  });

  it("An English-native reader: the title is the English catalogue's", () => {
    const doc = new DOMParser().parseFromString(HTML, "text/html");
    fillPage(doc, readerModule("en"));
    setDocumentLanguage(doc, "en");
    expect(doc.title).toBe("Library — Cymbra Lingua");
    expect(doc.documentElement.lang).toBe("en");
  });
});

describe("the reader's copy", () => {
  it("Every reader today: COPY is the French module in the shape the page reads, failures by reason", () => {
    expect(COPY.title).toBe("Bibliothèque");
    expect(COPY.importFailed).toEqual({
      protected: frReader.importProtected,
      notEpub: frReader.importNotEpub,
      unreadable: frReader.importUnreadable,
      storage: frReader.importStorage,
    });
    expect(COPY.importFailed.notEpub).toBe("Ce fichier n'est pas un livre EPUB.");
    expect(COPY.removeConfirm("Dune")).toBe(
      "Supprimer « Dune » ? Les cartes que tu en as tirées restent dans ton deck.",
    );
    expect(readerCopy()).toEqual(COPY);
    expect("importProtected" in COPY).toBe(false);
  });

  it("builds the same shape for English", () => {
    const en = readerCopy("en");
    expect(en.importFailed.protected).toBe(enReader.importProtected);
    expect(en.open("Dune")).toBe("Open “Dune”");
    expect(Object.keys(en).sort()).toEqual(Object.keys(COPY).sort());
  });

  it("names a book without a title in the interface language of the import, French without one", () => {
    expect(titleFromName("")).toBe("Livre sans titre");
    expect(titleFromName(undefined, enReader.untitled)).toBe("Untitled book");
    expect(titleFromName("my_book.epub", enReader.untitled)).toBe("my book");
  });
});

describe("the reader page in English", () => {
  let root: HTMLElement;

  function memoryArea(): AsyncStorageArea {
    const store: Record<string, unknown> = {};
    return {
      async get(keys) {
        const list = keys == null ? Object.keys(store) : Array.isArray(keys) ? keys : [keys];
        return Object.fromEntries(list.filter((k) => k in store).map((k) => [k, store[k]]));
      },
      async set(items) {
        Object.assign(store, items);
      },
    };
  }

  function app(over: Partial<ReaderDeps> = {}): ReaderApp {
    return new ReaderApp(root, {
      library: {} as Library,
      createRenderer: () => {
        throw new Error("no book is opened here");
      },
      persistence: async () => "granted",
      loadFlow: async () => "paginated",
      watchFlow: () => {},
      displayArea: memoryArea(),
      watchDisplay: () => {},
      ...over,
    });
  }

  beforeEach(() => {
    root = document.createElement("main");
    document.body.append(root);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("An English-native reader: the library, the toolbar and the figure are the English catalogue's", () => {
    const a = app({ language: "en", copy: readerCopy("en") });
    expect(root.querySelector(".lib-head h1")?.textContent).toBe("Library");
    expect(root.querySelector(".lib-empty")?.textContent).toBe(enReader.empty);
    expect(root.querySelector(".lib-import")?.textContent).toBe("Import a book (EPUB)");
    expect(root.querySelector(".reading-back")?.textContent).toBe("‹ Library");
    expect(root.querySelector(".reading-pct")?.getAttribute("title")).toBe("Known words in this chapter");
    const indicator = a.indicator({ onReview() {}, onStats() {}, onSettings() {} });
    indicator.update({ analysable: true, percent: 45 });
    expect(root.querySelector(".reading-pct")?.textContent).toBe("45%");
  });

  it("writes the chapter's figure as each language does: French spaced, Spanish with its narrow space", () => {
    const fr = app().indicator({ onReview() {}, onStats() {}, onSettings() {} });
    fr.update({ analysable: true, percent: 45 });
    expect(root.querySelector(".reading-pct")?.textContent).toBe("45 %");
    root.replaceChildren();
    const es = app({ language: "es", copy: readerCopy("es") }).indicator({
      onReview() {},
      onStats() {},
      onSettings() {},
    });
    es.update({ analysable: true, percent: 45 });
    expect(root.querySelector(".reading-pct")?.textContent).toBe("45 %");
  });
});
