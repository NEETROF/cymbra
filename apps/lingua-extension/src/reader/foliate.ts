import { EPUB, type FoliateTocItem } from "foliate-js/epub.js";
import { type FoliateRelocate, View } from "foliate-js/view.js";
import { DEFAULT_READER_DISPLAY, type ReaderDisplay, type ReaderFlow } from "../state/storage.ts";
import { openArchive } from "./archive.ts";
import { bookStyles, nightColoursOf, scaleFontSizes } from "./book-style.ts";
import type { BookLocation, BookRenderer, OpenedBook, SectionReady, TocEntry } from "./renderer.ts";
import { continueAtEdges, type ScrollEdges } from "./scroll-edges.ts";
import { type LoadableSection, routeSections, workerServesSections } from "./section-server.ts";
import { guardSelectionTouches } from "./touch-guard.ts";

// The thin adapter over foliate-js (add-lingua-reader D2): the only file that knows its
// element, its events and its options. Excluded from coverage with the vendored tree — it
// needs a real browser to lay a book out (vitest.config.ts); the page drives it through the
// BookRenderer seam, which its tests fake.

function tocOf(items: FoliateTocItem[] | null | undefined): TocEntry[] {
  return (items ?? [])
    .filter((it) => it.href)
    .map((it) => ({ label: (it.label ?? "").trim(), href: it.href!, children: tocOf(it.subitems) }));
}

export class FoliateRenderer implements BookRenderer {
  // `view.js` registers <foliate-view> on import; the element is the View class.
  readonly element = new View();
  private readonly ready: ((s: SectionReady) => void)[] = [];
  private readonly moved: ((l: BookLocation) => void)[] = [];
  private last: BookLocation | null = null;
  private display: ReaderDisplay = DEFAULT_READER_DISPLAY;

  /** The view's edges in the scrolled flow, to read on into the next section (scroll-edges.ts). */
  private readonly edges: ScrollEdges = {
    scrolled: () => !!this.element.renderer?.scrolled,
    atTop: () => (this.element.renderer?.start ?? 0) <= 1,
    atBottom: () => {
      const r = this.element.renderer;
      return !!r && r.viewSize - r.end <= 2;
    },
    next: () => void this.next(),
    prev: () => void this.prev(),
  };

  constructor() {
    // A push over the book's margins lands on the element, over its text in the section's document.
    continueAtEdges(this.element, this.edges);
    // Listened to before the book opens: the first section loads during `init`.
    this.element.addEventListener("load", (e) => {
      const { doc, index } = (e as CustomEvent<SectionReady>).detail;
      // Only Safari hands a selection's handle drag to the page as touch events (touch-guard.ts).
      if (__TARGET__ === "safari") guardSelectionTouches(doc);
      continueAtEdges(doc, this.edges);
      for (const l of this.ready) l({ doc, index });
    });
    this.element.addEventListener("relocate", (e) => {
      const { cfi, fraction, tocItem } = (e as CustomEvent<FoliateRelocate>).detail;
      this.last = { cfi, fraction: fraction ?? 0, section: tocItem?.label?.trim() || null };
      for (const l of this.moved) l(this.last);
    });
  }

  async open(file: Blob, at: string | null, flow: ReaderFlow): Promise<OpenedBook> {
    const book = await new EPUB(await openArchive(file)).init();
    // Each style sheet of the book, as it loads: its absolute font sizes follow the text size.
    book.transformTarget?.addEventListener("data", (e) => {
      const detail = (e as CustomEvent<{ type: string; data: unknown }>).detail;
      if (detail.type !== "text/css") return;
      detail.data = Promise.resolve(detail.data).then((css) => (typeof css === "string" ? scaleFontSizes(css) : css));
    });
    if (__SECTIONS_FROM_WORKER__) {
      // Chromium: each section from the service worker, never a blob: document (section-server.ts).
      const pageUrl = (path: string): string => chrome.runtime.getURL(path);
      let served: Promise<boolean> | null = null;
      routeSections(book.sections as LoadableSection[], {
        caches,
        fetch: (url) => fetch(url),
        pageUrl,
        served: () => (served ??= workerServesSections((url) => fetch(url), pageUrl)),
        token: crypto.randomUUID(),
      });
    }
    await this.element.open(book);
    this.applyDisplay();
    this.setFlow(flow);
    try {
      await this.element.init({ lastLocation: at, showTextStart: true });
    } catch {
      // A saved position this edition no longer resolves: start at the text instead.
      await this.element.init({ lastLocation: null, showTextStart: true });
    }
    return { toc: tocOf(book.toc) };
  }

  async goTo(target: string): Promise<void> {
    await this.element.goTo(target);
  }

  next(): Promise<void> {
    return this.element.next();
  }

  prev(): Promise<void> {
    return this.element.prev();
  }

  setDisplay(display: ReaderDisplay): void {
    this.display = display;
    this.applyDisplay();
  }

  private applyDisplay(): void {
    this.element.renderer?.setStyles(bookStyles(this.display, nightColoursOf(this.element.ownerDocument)));
  }

  setFlow(flow: ReaderFlow): void {
    // No `animated` attribute: a page turn is one jump, which an e-ink screen shows once.
    this.element.renderer?.setAttribute("flow", flow);
  }

  onSectionReady(listener: (section: SectionReady) => void): void {
    this.ready.push(listener);
  }

  onRelocate(listener: (location: BookLocation) => void): void {
    this.moved.push(listener);
  }

  close(): void {
    this.element.close();
    this.element.remove();
  }
}
