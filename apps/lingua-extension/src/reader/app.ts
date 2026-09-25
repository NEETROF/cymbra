import { type BookDisplayView, mountBookDisplay } from "../reading/book-display-view.ts";
import type { HudActions, HudState } from "../reading/hud.ts";
import type { Box, ReadingHost, ReadingIndicator } from "../reading/session.ts";
import {
  type AsyncStorageArea,
  DEFAULT_READER_DISPLAY,
  loadReaderDisplay,
  type ReaderDisplay,
  type ReaderFlow,
} from "../state/storage.ts";
import { COPY } from "./copy.ts";
import type { BookRecord, ImportResult, Library } from "./library.ts";
import type { Persistence } from "./persist.ts";
import type { BookLocation, BookRenderer, SectionReady, TocEntry } from "./renderer.ts";

// The reader page (add-lingua-reader): a library of the reader's own EPUB files, and a book
// open in it with the reading module mounted on each section. Built as plain DOM into a given
// root and driven through seams — the library, the renderer, the reading session — so it is
// tested without foliate-js or an engine; reader.ts wires the real ones.
//
// One paint per page turn (design D4): a section is hidden from the moment its document is
// loaded until the reading module has painted it, or until a cap if the engine is slow — then
// revealed once. The session paints a section whole, so turning a page inside it paints
// nothing new.

/** What the page needs of the reading session. */
export interface ReaderSession {
  attach(host: ReadingHost): Promise<void>;
  detach(): void;
  dismiss(): void;
}

export interface ReaderDeps {
  library: Library;
  /** A fresh renderer for each book opened. */
  createRenderer: () => BookRenderer;
  /** Ask the browser to keep the library (once), and say what it answered. */
  persistence: () => Promise<Persistence>;
  loadFlow: () => Promise<ReaderFlow>;
  /** Call back when the reader changes the flow in the settings, wherever they are rendered. */
  watchFlow: (onFlow: (flow: ReaderFlow) => void) => void;
  /** Where the text size and the page are kept: the "Aa" panel writes there, the book follows. */
  displayArea: AsyncStorageArea;
  /** Call back when the text size or the page changes, from this page or from Réglages. */
  watchDisplay: (onDisplay: (display: ReaderDisplay) => void) => void;
  now?: () => number;
  /** The longest a section stays hidden waiting for its first paint. */
  revealCapMs?: number;
  objectUrl?: (blob: Blob) => string;
  revokeUrl?: (url: string) => void;
  setTimer?: (fn: () => void, ms: number) => unknown;
  clearTimer?: (handle: unknown) => void;
}

/**
 * How long a section waits for its highlights before it is shown anyway, so a slow or absent
 * engine never blanks the book. Provisional until the spike's device numbers (task 1.3) pick
 * it: the laptop paints a chapter in well under this, the e-ink tablet is the one to measure.
 */
export const REVEAL_CAP_MS = 1500;

/** The tag reading exposures in a book are recorded under — no title: it names no book. */
export const BOOK_EXPOSURE_SOURCE = "reading:book";

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

function button(className: string, text: string, onClick: () => void, label?: string): HTMLButtonElement {
  const b = el("button", className, text);
  b.type = "button";
  if (label) {
    b.title = label;
    b.setAttribute("aria-label", label);
  }
  // Keep a selection in the book alive: the capture reads it after the click.
  b.addEventListener("mousedown", (e) => e.preventDefault());
  b.addEventListener("click", onClick);
  return b;
}

/** A card's source for a word met in a book: the book, and the chapter when there is one. */
export function bookSource(title: string, section: string | null): string {
  return section ? `${title} · ${section}` : title;
}

/** A box of a section's viewport, moved into the reader page's: the section's frame is where it sits. */
export function frameOffset(doc: Document): (box: Box) => Box {
  return (box) => {
    const frame = doc.defaultView?.frameElement?.getBoundingClientRect();
    if (!frame) return box;
    return { left: box.left + frame.left, top: box.top + frame.top, bottom: box.bottom + frame.top };
  };
}

export class ReaderApp {
  private readonly libraryView = el("section", "library-view");
  private readonly notice = el("p", "lib-notice");
  private readonly status = el("div", "lib-status");
  private readonly empty = el("p", "lib-empty", COPY.empty);
  private readonly shelf = el("ul", "lib-books");
  private readonly fileInput = el("input");

  private readonly readingView = el("section", "reading-view");
  private readonly bookTitle = el("span", "reading-book-title");
  private readonly sectionTitle = el("span", "reading-section-title");
  private readonly percent = el("span", "reading-pct", "—");
  private readonly bookHost = el("div", "reading-book");
  private readonly progress = el("span", "reading-progress");
  private readonly tocPanel = el("nav", "reading-toc");
  private readonly displayPanel = el("div", "reading-display");
  private displayView: BookDisplayView | null = null;
  private display: ReaderDisplay = DEFAULT_READER_DISPLAY;

  private covers: string[] = [];
  private session: ReaderSession | null = null;
  private actions: HudActions | null = null;
  private renderer: BookRenderer | null = null;
  private book: BookRecord | null = null;
  private section: string | null = null;
  /** Where the page last was (a CFI), to tell a real move from foliate settling in place. */
  private at: string | null = null;
  private flow: ReaderFlow = "paginated";
  /** Whether the section on screen was revealed yet (design D4: once per section). */
  private revealed = true;
  private revealTimer: unknown = null;

  constructor(
    private readonly root: HTMLElement,
    private readonly deps: ReaderDeps,
  ) {
    this.buildLibrary();
    this.buildReading();
    root.replaceChildren(this.libraryView, this.readingView);
    this.readingView.hidden = true;
  }

  /** Start with the reading session the page hosts: the library first, or the book the address names. */
  async start(session: ReaderSession): Promise<void> {
    this.session = session;
    this.flow = await this.deps.loadFlow();
    this.deps.watchFlow((flow) => {
      this.flow = flow;
      this.renderer?.setFlow(flow);
    });
    this.applyDisplay(await loadReaderDisplay(this.deps.displayArea));
    this.deps.watchDisplay((display) => this.applyDisplay(display));
    const persistence = await this.deps.persistence();
    this.notice.hidden = persistence !== "refused";
    this.notice.textContent = persistence === "refused" ? COPY.persistenceRefused : "";
    const hash = bookInAddress(location.hash);
    if (hash) await this.openBook(hash);
    else await this.showLibrary();
  }

  /** What the session shows the reading state through: the toolbar's percentage and buttons. */
  indicator(actions: HudActions): ReadingIndicator {
    this.actions = actions;
    return {
      mount: () => {},
      update: (state: HudState) => {
        this.percent.textContent = state.analysable && state.percent != null ? `${state.percent} %` : "—";
      },
      setHidden: (hidden: boolean) => {
        this.percent.hidden = hidden;
      },
    };
  }

  /** The session painted the section on screen: reveal it, the first time. */
  painted(): void {
    this.reveal();
  }

  /** A tap on nothing in a section's text: its outer thirds turn the page (see `turnByZone`). */
  blankClick(e: MouseEvent): void {
    const doc = (e.target as Node | null)?.ownerDocument ?? null;
    this.turnByZone(doc ? frameOffset(doc)({ left: e.clientX, top: 0, bottom: 0 }).left : e.clientX);
  }

  /**
   * A tap at `x` (in this page's viewport) on the book: the left third turns back, the right
   * third forward, the middle does nothing — in the paginated flow only, where there is no
   * scrolling to do it instead.
   */
  private turnByZone(x: number): void {
    if (this.flow !== "paginated" || !this.renderer) return;
    const width = this.root.ownerDocument.documentElement.clientWidth || window.innerWidth;
    if (x < width / 3) void this.renderer.prev();
    else if (x > (width * 2) / 3) void this.renderer.next();
  }

  // — The library —

  private buildLibrary(): void {
    const head = el("header", "lib-head");
    head.append(el("h1", undefined, COPY.title));
    const pick = el("label", "lib-import");
    this.fileInput.type = "file";
    this.fileInput.accept = ".epub,application/epub+zip";
    this.fileInput.multiple = true;
    this.fileInput.hidden = true;
    this.fileInput.addEventListener("change", () => void this.importPicked());
    pick.append(COPY.importButton, this.fileInput);
    head.append(pick);
    this.notice.hidden = true;
    this.status.setAttribute("role", "status");
    this.libraryView.append(head, this.notice, this.status, this.empty, this.shelf);
  }

  private async importPicked(): Promise<void> {
    const files = [...(this.fileInput.files ?? [])];
    this.fileInput.value = "";
    if (files.length === 0) return;
    this.status.textContent = COPY.importing;
    const lines: string[] = [];
    for (const file of files) lines.push(importLine(await this.deps.library.importFile(file, this.now())));
    this.status.textContent = lines.join("\n");
    await this.renderShelf();
  }

  private async showLibrary(): Promise<void> {
    this.closeBook();
    this.readingView.hidden = true;
    this.libraryView.hidden = false;
    this.root.ownerDocument.title = `${COPY.title} — Cymbra Lingua`;
    if (location.hash) history.replaceState(null, "", location.pathname + location.search);
    await this.renderShelf();
  }

  private async renderShelf(): Promise<void> {
    for (const url of this.covers) this.revoke(url);
    this.covers = [];
    const books = await this.deps.library.list();
    this.empty.hidden = books.length > 0;
    this.shelf.replaceChildren(...books.map((b) => this.card(b)));
  }

  private card(book: BookRecord): HTMLLIElement {
    const item = el("li", "lib-book");
    const open = el("button", "lib-open");
    open.type = "button";
    open.setAttribute("aria-label", COPY.open(book.title));
    const cover = el("div", "lib-cover");
    if (book.cover) {
      const img = el("img");
      img.alt = "";
      const url = this.deps.objectUrl ? this.deps.objectUrl(book.cover) : URL.createObjectURL(book.cover);
      this.covers.push(url);
      img.src = url;
      cover.append(img);
    } else {
      cover.textContent = book.title.charAt(0).toUpperCase();
    }
    open.append(cover, el("span", "lib-title", book.title));
    if (book.authors.length > 0) open.append(el("span", "lib-authors", book.authors.join(", ")));
    open.addEventListener("click", () => void this.openBook(book.hash));

    const confirm = el("div", "lib-confirm");
    confirm.hidden = true;
    const remove = button("lib-remove", COPY.remove, () => {
      remove.hidden = true;
      confirm.hidden = false;
    });
    confirm.append(
      el("p", undefined, COPY.removeConfirm(book.title)),
      button("lib-remove-yes", COPY.removeYes, () => void this.remove(book.hash)),
      button("lib-remove-no", COPY.cancel, () => {
        confirm.hidden = true;
        remove.hidden = false;
      }),
    );
    item.append(open, remove, confirm);
    return item;
  }

  private async remove(hash: string): Promise<void> {
    await this.deps.library.remove(hash);
    this.status.textContent = "";
    await this.renderShelf();
  }

  // — A book —

  private buildReading(): void {
    const bar = el("header", "reading-bar");
    const titles = el("div", "reading-titles");
    titles.append(this.bookTitle, this.sectionTitle);
    this.percent.title = COPY.percentTitle;
    const act = (text: string, pick: (a: HudActions) => () => void): HTMLButtonElement =>
      button("reading-action", text, () => {
        if (this.actions) pick(this.actions)();
      });
    bar.append(
      button("reading-back", `‹ ${COPY.back}`, () => void this.showLibrary()),
      titles,
      this.percent,
      button("reading-action", COPY.toc, () => this.toggleToc()),
      button("reading-action reading-display-toggle", COPY.display, () => this.toggleDisplay(), COPY.displayTitle),
      act(COPY.review, (a) => a.onReview),
      act(COPY.stats, (a) => a.onStats),
      act(COPY.settings, (a) => a.onSettings),
    );
    const foot = el("footer", "reading-foot");
    foot.append(
      button("reading-turn", "‹", () => void this.renderer?.prev(), COPY.prev),
      this.progress,
      button("reading-turn", "›", () => void this.renderer?.next(), COPY.next),
    );
    this.tocPanel.hidden = true;
    this.displayPanel.hidden = true;
    this.displayView = mountBookDisplay(this.displayPanel, this.deps.displayArea);
    // A tap on the page's margins lands in this document, not in the section's: same zones.
    this.bookHost.addEventListener("click", (e) => this.turnByZone(e.clientX));
    // The book, and the panels laid over it: they start under the bar, however many rows it takes.
    const stage = el("div", "reading-stage");
    stage.append(this.bookHost, this.tocPanel, this.displayPanel);
    this.readingView.append(bar, stage, foot);
    this.root.ownerDocument.addEventListener("keydown", (e) => this.onKey(e));
  }

  private async openBook(hash: string): Promise<void> {
    const book = await this.deps.library.get(hash);
    const file = book ? await this.deps.library.file(hash) : null;
    if (!book || !file) {
      await this.showLibrary();
      this.status.textContent = COPY.missing;
      return;
    }
    this.closeBook();
    this.book = book;
    this.section = null;
    this.at = null;
    this.bookTitle.textContent = book.title;
    this.sectionTitle.textContent = "";
    this.progress.textContent = "";
    this.percent.textContent = "—";
    this.root.ownerDocument.title = `${book.title} — Cymbra Lingua`;
    history.replaceState(null, "", `${location.pathname}${location.search}#book=${hash}`);
    this.libraryView.hidden = true;
    this.readingView.hidden = false;

    const renderer = this.deps.createRenderer();
    this.renderer = renderer;
    renderer.setDisplay(this.display);
    this.bookHost.replaceChildren(renderer.element);
    renderer.onSectionReady((s) => void this.onSection(s));
    renderer.onRelocate((l) => this.onRelocate(l));
    try {
      const { toc } = await renderer.open(file, book.location, this.flow);
      this.renderToc(toc);
    } catch {
      await this.showLibrary();
      this.status.textContent = COPY.openFailed;
    }
  }

  private closeBook(): void {
    this.cancelReveal();
    this.revealed = true;
    this.session?.detach();
    this.renderer?.close();
    this.renderer = null;
    this.book = null;
    this.tocPanel.hidden = true;
    this.displayPanel.hidden = true;
    this.bookHost.replaceChildren();
  }

  /** A section's document is loaded: hide the book until it is painted, and read it. */
  private async onSection({ doc }: SectionReady): Promise<void> {
    const book = this.book;
    if (!book || !this.session) return;
    this.hideUntilPainted();
    doc.addEventListener("keydown", (e) => this.onKey(e));
    await this.session.attach({
      doc,
      win: (doc.defaultView ?? window) as Window & typeof globalThis,
      paintWhole: true,
      toSurface: frameOffset(doc),
      source: () => bookSource(book.title, this.section),
      exposureSource: () => BOOK_EXPOSURE_SOURCE,
    });
  }

  /**
   * What is hidden while a section paints: the book's content, not its page. Hiding the whole
   * area showed the dark page behind it — a black flash at every chapter; the paper stays.
   */
  private content(): HTMLElement | null {
    return this.renderer?.element ?? null;
  }

  private hideUntilPainted(): void {
    this.cancelReveal();
    this.revealed = false;
    const content = this.content();
    if (content) content.style.visibility = "hidden";
    const cap = this.deps.revealCapMs ?? REVEAL_CAP_MS;
    const reveal = (): void => this.reveal();
    this.revealTimer = this.deps.setTimer ? this.deps.setTimer(reveal, cap) : setTimeout(reveal, cap);
  }

  private reveal(): void {
    if (this.revealed) return;
    this.revealed = true;
    this.cancelReveal();
    const content = this.content();
    if (content) content.style.visibility = "";
  }

  private cancelReveal(): void {
    if (this.revealTimer === null) return;
    if (this.deps.clearTimer) this.deps.clearTimer(this.revealTimer);
    else clearTimeout(this.revealTimer as ReturnType<typeof setTimeout>);
    this.revealTimer = null;
  }

  private onRelocate(location: BookLocation): void {
    const book = this.book;
    if (!book) return;
    this.section = location.section;
    this.sectionTitle.textContent = location.section ?? "";
    this.progress.textContent = `${Math.round(location.fraction * 100)} %`;
    // foliate-js settles the page after every lift of a finger — a tap, a press-and-hold — and
    // reports it even when nothing moved; closing the card then would close the one that lift
    // just opened. Only a real move dismisses it and is worth writing down.
    if (location.cfi === this.at) return;
    this.at = location.cfi;
    // The word popup is anchored to a word that just moved.
    this.session?.dismiss();
    void this.deps.library.savePosition(book.hash, location.cfi, this.now());
  }

  private renderToc(toc: TocEntry[]): void {
    const list = (entries: TocEntry[]): HTMLUListElement => {
      const ul = el("ul");
      for (const entry of entries) {
        const li = el("li");
        li.append(
          button("reading-toc-entry", entry.label || entry.href, () => {
            this.tocPanel.hidden = true;
            void this.renderer?.goTo(entry.href);
          }),
        );
        if (entry.children.length > 0) li.append(list(entry.children));
        ul.append(li);
      }
      return ul;
    };
    this.tocPanel.replaceChildren(toc.length > 0 ? list(toc) : el("p", "reading-toc-empty", COPY.noToc));
  }

  private toggleToc(): void {
    this.tocPanel.hidden = !this.tocPanel.hidden;
    if (!this.tocPanel.hidden) this.displayPanel.hidden = true;
  }

  /** The "Aa" panel: the text size and the page, shown as they are stored now. */
  private toggleDisplay(): void {
    this.displayPanel.hidden = !this.displayPanel.hidden;
    if (this.displayPanel.hidden) return;
    this.tocPanel.hidden = true;
    void this.displayView?.refresh();
  }

  /** Show the book at this size, on this page: the paper or night behind it, and its text. */
  private applyDisplay(display: ReaderDisplay): void {
    this.display = display;
    this.bookHost.dataset.theme = display.theme;
    this.renderer?.setDisplay(display);
  }

  private onKey(e: KeyboardEvent): void {
    if (!this.renderer || this.readingView.hidden || e.altKey || e.ctrlKey || e.metaKey) return;
    if (e.key === "ArrowLeft" || e.key === "PageUp") void this.renderer.prev();
    else if (e.key === "ArrowRight" || e.key === "PageDown") void this.renderer.next();
    else if (e.key === "Escape" && !(this.tocPanel.hidden && this.displayPanel.hidden)) {
      this.tocPanel.hidden = true;
      this.displayPanel.hidden = true;
    } else return;
    e.preventDefault();
  }

  private now(): number {
    return (this.deps.now ?? Date.now)();
  }

  private revoke(url: string): void {
    if (this.deps.revokeUrl) this.deps.revokeUrl(url);
    else URL.revokeObjectURL(url);
  }
}

/** The book an address names (`reader.html#book=<hash>`), so a reload reopens it. */
export function bookInAddress(hash: string): string | null {
  const m = /^#book=([0-9a-f]{64})$/.exec(hash);
  return m ? m[1] : null;
}

/** One line saying what became of an imported file. */
export function importLine(result: ImportResult): string {
  if (!result.ok) return COPY.importFailed[result.reason];
  return result.existed ? COPY.alreadyThere(result.book.title) : COPY.imported(result.book.title);
}
