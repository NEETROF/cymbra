import { EPUB, type FoliateTocItem } from "foliate-js/epub.js";
import { type FoliateRelocate, View } from "foliate-js/view.js";
import type { ReaderFlow } from "../state/storage.ts";
import { openArchive } from "./archive.ts";
import type { BookLocation, BookRenderer, OpenedBook, SectionReady, TocEntry } from "./renderer.ts";
import { type LoadableSection, routeSections, workerServesSections } from "./section-server.ts";

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

  constructor() {
    // Listened to before the book opens: the first section loads during `init`.
    this.element.addEventListener("load", (e) => {
      const { doc, index } = (e as CustomEvent<SectionReady>).detail;
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
