import { IDBFactory } from "fake-indexeddb";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  BOOK_EXPOSURE_SOURCE,
  bookInAddress,
  bookSource,
  frameOffset,
  importLine,
  ReaderApp,
  type ReaderDeps,
  type ReaderSession,
} from "@/reader/app.ts";
import { COPY } from "@/reader/copy.ts";
import { Library } from "@/reader/library.ts";
import type { TocEntry } from "@/reader/renderer.ts";
import type { ReadingHost } from "@/reading/session.ts";
import type { ReaderFlow } from "@/state/storage.ts";
import { epub3Entries, pickedFile, useNodeBlob } from "./epub-fixtures.ts";
import { FakeRenderer } from "./fake-renderer.ts";

// The reader page (add-lingua-reader §4): the library, a book open with the reading module on
// each section, one paint per section, the position kept, the page turned by tap or key.

const TOC: TocEntry[] = [
  { label: "I. The Curse", href: "c1.xhtml", children: [{ label: "A note", href: "c1.xhtml#n", children: [] }] },
];

let root: HTMLElement;
let library: Library;
let renderers: FakeRenderer[];
let timers: { fn: () => void; ms: number; cleared: boolean }[];
let flowListeners: ((f: ReaderFlow) => void)[];

function fakeSession() {
  const hosts: ReadingHost[] = [];
  const session: ReaderSession & { hosts: ReadingHost[]; detached: number; dismissed: number } = {
    hosts,
    detached: 0,
    dismissed: 0,
    attach: vi.fn(async (host: ReadingHost) => void hosts.push(host)),
    detach: () => void session.detached++,
    dismiss: () => void session.dismissed++,
  };
  return session;
}

function app(over: Partial<ReaderDeps> = {}): ReaderApp {
  return new ReaderApp(root, {
    library,
    createRenderer: () => {
      const r = new FakeRenderer(["<p>It was a dark night.</p>", "<p>Next.</p>"], TOC);
      renderers.push(r);
      return r;
    },
    persistence: async () => "granted",
    loadFlow: async () => "paginated",
    watchFlow: (fn) => void flowListeners.push(fn),
    now: () => 1000,
    objectUrl: () => "blob:cover",
    revokeUrl: () => {},
    setTimer: (fn, ms) => {
      const t = { fn, ms, cleared: false };
      timers.push(t);
      return t;
    },
    clearTimer: (h) => void ((h as { cleared: boolean }).cleared = true),
    ...over,
  });
}

const $ = <T extends Element = HTMLElement>(sel: string): T => root.querySelector<T>(sel)!;
const $$ = (sel: string): HTMLElement[] => [...root.querySelectorAll<HTMLElement>(sel)];
const text = (sel: string): string => $(sel).textContent ?? "";
const settle = () => new Promise((r) => setTimeout(r, 0));

async function pick(...files: File[]): Promise<void> {
  const input = $<HTMLInputElement>("input[type=file]");
  Object.defineProperty(input, "files", { value: files, configurable: true });
  input.dispatchEvent(new Event("change"));
  await vi.waitFor(() => expect(text(".lib-status")).not.toBe(COPY.importing));
  await vi.waitFor(() => expect($$(".lib-book").length).toBeGreaterThan(0));
}

async function withBook(): Promise<{ a: ReaderApp; session: ReturnType<typeof fakeSession>; hash: string }> {
  const a = app();
  const session = fakeSession();
  await a.start(session);
  await pick(await pickedFile(epub3Entries(), "hound.epub"));
  const [book] = await library.list();
  return { a, session, hash: book.hash };
}

async function openFirst(): Promise<void> {
  $<HTMLButtonElement>(".lib-open").click();
  await vi.waitFor(() => expect(renderers.at(-1)?.openedWith).toBeTruthy());
  await settle();
}

beforeEach(async () => {
  useNodeBlob();
  root = document.createElement("main");
  document.body.append(root);
  library = await Library.open(new IDBFactory());
  renderers = [];
  timers = [];
  flowListeners = [];
  history.replaceState(null, "", "/reader.html");
});

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.innerHTML = "";
});

describe("the library view", () => {
  it("says the library is empty, and warns when the browser would not keep it", async () => {
    const a = app({ persistence: async () => "refused" });
    await a.start(fakeSession());
    expect($(".lib-empty").hidden).toBe(false);
    expect($(".lib-notice").hidden).toBe(false);
    expect(text(".lib-notice")).toBe(COPY.persistenceRefused);
    expect($(".reading-view").hidden).toBe(true);
  });

  it("imports picked files and shows each book with its title, authors and cover", async () => {
    const a = app();
    await a.start(fakeSession());
    expect($(".lib-notice").hidden).toBe(true);
    await pick(await pickedFile(epub3Entries(), "hound.epub"));
    expect(text(".lib-status")).toBe(COPY.imported("The Hound of the Baskervilles"));
    expect(text(".lib-title")).toBe("The Hound of the Baskervilles");
    expect(text(".lib-authors")).toBe("Arthur Conan Doyle, Sidney Paget");
    expect($<HTMLImageElement>(".lib-cover img").src).toBe("blob:cover");
    expect($(".lib-empty").hidden).toBe(true);
    expect($<HTMLInputElement>("input[type=file]").accept).toBe(".epub,application/epub+zip");
  });

  it("says, file by file, what became of each", async () => {
    const a = app();
    await a.start(fakeSession());
    const drm = epub3Entries({
      "META-INF/encryption.xml": `<encryption xmlns:e="http://www.w3.org/2001/04/xmlenc#"><e:EncryptedData>
        <e:EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes128-cbc"/></e:EncryptedData></encryption>`,
    });
    await pick(await pickedFile(epub3Entries(), "a.epub"), await pickedFile(drm, "drm.epub"));
    expect(text(".lib-status").split("\n")).toEqual([
      COPY.imported("The Hound of the Baskervilles"),
      COPY.importFailed.protected,
    ]);
  });

  it("deletes a book only once confirmed", async () => {
    await withBook();
    $<HTMLButtonElement>(".lib-remove").click();
    expect($(".lib-confirm").hidden).toBe(false);
    $<HTMLButtonElement>(".lib-remove-no").click();
    expect($(".lib-confirm").hidden).toBe(true);
    expect(await library.list()).toHaveLength(1);
    $<HTMLButtonElement>(".lib-remove").click();
    $<HTMLButtonElement>(".lib-remove-yes").click();
    await vi.waitFor(() => expect($$(".lib-book")).toHaveLength(0));
    expect(await library.list()).toEqual([]);
  });
});

describe("a book open", () => {
  it("opens where the reader stopped, names the book in the address, lists its contents", async () => {
    const { hash } = await withBook();
    await library.savePosition(hash, "epubcfi(/6/4)", 5);
    await openFirst();
    const r = renderers[0];
    expect(r.openedWith?.at).toBe("epubcfi(/6/4)");
    expect(r.flow).toBe("paginated");
    expect(location.hash).toBe(`#book=${hash}`);
    expect($(".library-view").hidden).toBe(true);
    expect($(".reading-view").hidden).toBe(false);
    expect(text(".reading-book-title")).toBe("The Hound of the Baskervilles");
    expect($$(".reading-toc-entry").map((b) => b.textContent)).toEqual(["I. The Curse", "A note"]);
    $<HTMLButtonElement>(".reading-toc-entry").click();
    expect(r.calls).toContain("goTo:c1.xhtml");
  });

  it("hides each section until it is painted, then shows it once", async () => {
    const { a, session } = await withBook();
    await openFirst();
    const book = $(".reading-book");
    expect(session.hosts).toHaveLength(1);
    expect(book.style.visibility).toBe("hidden");
    a.painted();
    expect(book.style.visibility).toBe("");
    expect(timers[0].cleared).toBe(true);
    // The next section: hidden again, and this time the cap shows it.
    renderers[0].showSection(1);
    expect(book.style.visibility).toBe("hidden");
    timers.at(-1)!.fn();
    expect(book.style.visibility).toBe("");
    // A later paint of the same section (a gesture) shows nothing twice.
    a.painted();
    expect(book.style.visibility).toBe("");
  });

  it("mounts the reading module on the section, whole, with the book as a card's source", async () => {
    const { session } = await withBook();
    await openFirst();
    const host = session.hosts[0];
    expect(host.paintWhole).toBe(true);
    expect(host.doc.body.textContent).toBe("It was a dark night.");
    expect(host.exposureSource()).toBe(BOOK_EXPOSURE_SOURCE);
    expect(host.source()).toBe("The Hound of the Baskervilles");
    renderers[0].relocate({ cfi: "epubcfi(/6/2!/4/2)", fraction: 0.25, section: "I. The Curse" });
    expect(host.source()).toBe("The Hound of the Baskervilles · I. The Curse");
  });

  it("keeps the position after every move, and shows where the reader is", async () => {
    const { session, hash } = await withBook();
    await openFirst();
    renderers[0].relocate({ cfi: "epubcfi(/6/8)", fraction: 0.426, section: "II. The Hound" });
    expect(text(".reading-progress")).toBe("43 %");
    expect(text(".reading-section-title")).toBe("II. The Hound");
    expect(session.dismissed).toBe(1);
    await vi.waitFor(async () => expect((await library.get(hash))?.location).toBe("epubcfi(/6/8)"));
  });

  it("turns the page by its outer thirds, by the arrows and by the keys, in the paginated flow", async () => {
    const { a } = await withBook();
    await openFirst();
    const r = renderers[0];
    Object.defineProperty(document.documentElement, "clientWidth", { value: 900, configurable: true });
    a.blankClick(new MouseEvent("click", { clientX: 50 }));
    a.blankClick(new MouseEvent("click", { clientX: 450 }));
    a.blankClick(new MouseEvent("click", { clientX: 850 }));
    $(".reading-book").dispatchEvent(new MouseEvent("click", { clientX: 880, bubbles: true }));
    const [prev, next] = $$(".reading-turn");
    prev.click();
    next.click();
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft" }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "PageDown" }));
    document.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", ctrlKey: true }));
    expect(r.calls).toEqual(["prev", "next", "next", "prev", "next", "prev", "next"]);
  });

  it("follows the flow chosen in the settings; a scrolled book does not turn on a tap", async () => {
    const { a } = await withBook();
    await openFirst();
    flowListeners.forEach((l) => l("scrolled"));
    expect(renderers[0].flow).toBe("scrolled");
    a.blankClick(new MouseEvent("click", { clientX: 5 }));
    expect(renderers[0].calls).toEqual([]);
  });

  it("shows the section's percentage through the session's indicator, and routes its actions", async () => {
    const a = app();
    const actions = { onReview: vi.fn(), onStats: vi.fn(), onSettings: vi.fn() };
    const indicator = a.indicator(actions);
    indicator.update({ analysable: true, percent: 87 });
    expect(text(".reading-pct")).toBe("87 %");
    indicator.update({ analysable: false, percent: null });
    expect(text(".reading-pct")).toBe("—");
    indicator.setHidden(true);
    expect($(".reading-pct").hidden).toBe(true);
    indicator.mount();
    const buttons = $$(".reading-action");
    buttons.find((b) => b.textContent === COPY.review)!.click();
    buttons.find((b) => b.textContent === COPY.stats)!.click();
    buttons.find((b) => b.textContent === COPY.settings)!.click();
    expect([actions.onReview, actions.onStats, actions.onSettings].map((f) => f.mock.calls.length)).toEqual([1, 1, 1]);
    const toc = buttons.find((b) => b.textContent === COPY.toc)!;
    toc.click();
    expect($(".reading-toc").hidden).toBe(false);
    toc.click();
    expect($(".reading-toc").hidden).toBe(true);
  });

  it("goes back to the library, closing the book and detaching the reading module", async () => {
    const { session } = await withBook();
    await openFirst();
    $<HTMLButtonElement>(".reading-back").click();
    await vi.waitFor(() => expect($(".library-view").hidden).toBe(false));
    expect(renderers[0].closed).toBe(true);
    expect(session.detached).toBeGreaterThan(0);
    expect(location.hash).toBe("");
  });

  it("reopens the book its address names", async () => {
    const { hash } = await withBook();
    history.replaceState(null, "", `/reader.html#book=${hash}`);
    await app().start(fakeSession());
    expect(renderers.at(-1)?.openedWith).toBeTruthy();
    expect($(".reading-view").hidden).toBe(false);
  });

  it("says so when the address names a book no longer here", async () => {
    history.replaceState(null, "", `/reader.html#book=${"f".repeat(64)}`);
    await app().start(fakeSession());
    expect($(".library-view").hidden).toBe(false);
    expect(text(".lib-status")).toBe(COPY.missing);
  });

  it("says so when the book cannot be opened", async () => {
    await withBook();
    const broken = new FakeRenderer();
    broken.open = async () => Promise.reject(new Error("bad OPF"));
    const a = app({ createRenderer: () => broken });
    await a.start(fakeSession());
    $<HTMLButtonElement>(".lib-open").click();
    await vi.waitFor(() => expect(text(".lib-status")).toBe(COPY.openFailed));
    expect($(".library-view").hidden).toBe(false);
  });
});

describe("the reader page's helpers", () => {
  it("names a card's source by the book and its chapter", () => {
    expect(bookSource("Emma", "Chapter 3")).toBe("Emma · Chapter 3");
    expect(bookSource("Emma", null)).toBe("Emma");
  });

  it("reads a book's hash out of the address, and nothing else", () => {
    expect(bookInAddress(`#book=${"a".repeat(64)}`)).toBe("a".repeat(64));
    expect(bookInAddress("#book=xyz")).toBeNull();
    expect(bookInAddress("")).toBeNull();
  });

  it("says in one line what became of an import", () => {
    const book = { title: "Emma" } as never;
    expect(importLine({ ok: true, book, existed: false })).toBe(COPY.imported("Emma"));
    expect(importLine({ ok: true, book, existed: true })).toBe(COPY.alreadyThere("Emma"));
    expect(importLine({ ok: false, reason: "storage" })).toBe(COPY.importFailed.storage);
  });

  it("moves a section's box by its frame's offset, and leaves a frameless one", () => {
    const frame = document.createElement("iframe");
    document.body.append(frame);
    frame.getBoundingClientRect = () => ({ left: 30, top: 70 }) as DOMRect;
    const box = { left: 1, top: 2, bottom: 3 };
    expect(frameOffset(frame.contentDocument!)(box)).toEqual({ left: 31, top: 72, bottom: 73 });
    expect(frameOffset(document.implementation.createHTMLDocument())(box)).toEqual(box);
  });
});
