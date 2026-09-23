import type { ReaderFlow } from "../state/storage.ts";

// The seam between the reader page and the engine that draws a book (add-lingua-reader D2).
// foliate-js sits behind it (foliate.ts, a thin adapter); the page only ever sees this
// interface, so its tests run on a fake (test/fake-renderer.ts) — jsdom cannot host foliate's
// iframes — and a later change of engine stays behind this file.

/** One entry of the book's table of contents. */
export interface TocEntry {
  label: string;
  /** Where it leads, as the renderer's `goTo` takes it. */
  href: string;
  children: TocEntry[];
}

/** Where the reader is in the book. */
export interface BookLocation {
  /** The position to reopen at: an EPUB CFI. */
  cfi: string;
  /** How far through the book, from 0 to 1. */
  fraction: number;
  /** The label of the table-of-contents entry the page sits in, when the book has one. */
  section: string | null;
}

/** A section of the book, loaded in its own document and about to be laid out. */
export interface SectionReady {
  doc: Document;
  /** Its place in the book's reading order. */
  index: number;
}

export interface OpenedBook {
  toc: TocEntry[];
}

export interface BookRenderer {
  /** The element the book is drawn in, for the page to place and to hide while it paints. */
  readonly element: HTMLElement;
  /** Open `file` at `at` (a CFI from `BookLocation`), or at the start of its text. */
  open(file: Blob, at: string | null, flow: ReaderFlow): Promise<OpenedBook>;
  /** Go to a table-of-contents target or a CFI. */
  goTo(target: string): Promise<void>;
  next(): Promise<void>;
  prev(): Promise<void>;
  setFlow(flow: ReaderFlow): void;
  /** Called with each section's document as soon as it is loaded — before it is laid out. */
  onSectionReady(listener: (section: SectionReady) => void): void;
  /** Called after every move: a page turned, a link followed, a section opened. */
  onRelocate(listener: (location: BookLocation) => void): void;
  close(): void;
}
