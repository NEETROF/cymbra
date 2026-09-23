import type { BookLocation, BookRenderer, OpenedBook, SectionReady, TocEntry } from "@/reader/renderer.ts";
import type { ReaderFlow } from "@/state/storage.ts";

/**
 * A renderer for tests: each "section" is a document of its own, built from the given HTML,
 * and a move is reported the way foliate-js reports it. `goTo`, `next` and `prev` record what
 * they were asked, so a test can assert the page drove the book.
 */
export class FakeRenderer implements BookRenderer {
  readonly element: HTMLElement = document.createElement("div");
  readonly calls: string[] = [];
  flow: ReaderFlow | null = null;
  openedWith: { file: Blob; at: string | null } | null = null;
  closed = false;
  private readyListeners: ((s: SectionReady) => void)[] = [];
  private relocateListeners: ((l: BookLocation) => void)[] = [];

  constructor(
    private readonly sections: string[] = ["<p>It was a dark and stormy night.</p>"],
    private readonly toc: TocEntry[] = [],
  ) {}

  async open(file: Blob, at: string | null, flow: ReaderFlow): Promise<OpenedBook> {
    this.openedWith = { file, at };
    this.flow = flow;
    this.showSection(0);
    return { toc: this.toc };
  }

  /** Load section `index` as foliate does: a fresh document, then the ready event. */
  showSection(index: number): Document {
    const doc = document.implementation.createHTMLDocument(`section ${index}`);
    doc.body.innerHTML = this.sections[index] ?? "";
    for (const l of this.readyListeners) l({ doc, index });
    return doc;
  }

  /** Report a move, as foliate's `relocate` event does. */
  relocate(location: BookLocation): void {
    for (const l of this.relocateListeners) l(location);
  }

  async goTo(target: string): Promise<void> {
    this.calls.push(`goTo:${target}`);
  }

  async next(): Promise<void> {
    this.calls.push("next");
  }

  async prev(): Promise<void> {
    this.calls.push("prev");
  }

  setFlow(flow: ReaderFlow): void {
    this.flow = flow;
  }

  onSectionReady(listener: (section: SectionReady) => void): void {
    this.readyListeners.push(listener);
  }

  onRelocate(listener: (location: BookLocation) => void): void {
    this.relocateListeners.push(listener);
  }

  close(): void {
    this.closed = true;
  }
}
