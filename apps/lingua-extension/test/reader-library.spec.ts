import { IDBFactory, IDBObjectStore } from "fake-indexeddb";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { type BookRecord, Library, LIBRARY_DB, sha256Hex, titleFromName } from "@/reader/library.ts";
import { epub3Entries, pickedFile, useNodeBlob, zip } from "./epub-fixtures.ts";

// The reader's books (add-lingua-reader 3.1): one per distinct file, keyed by its hash, kept
// with its file; a protected or foreign file stores nothing; the reading position is the
// latest move, whatever tab wrote it.

beforeEach(useNodeBlob);
afterEach(() => vi.unstubAllGlobals());

/** A fresh database per test: fake-indexeddb keeps one per factory. */
async function library(): Promise<Library> {
  return Library.open(new IDBFactory());
}

function record(hash: string, over: Partial<BookRecord> = {}): BookRecord {
  return {
    hash,
    title: hash,
    authors: [],
    language: null,
    identifier: null,
    cover: null,
    size: 1,
    addedAt: 0,
    location: null,
    locationAt: 0,
    ...over,
  };
}

describe("the library database", () => {
  it("is its own database, apart from the reader's store", async () => {
    const factory = new IDBFactory();
    await Library.open(factory);
    expect((await factory.databases()).map((d) => d.name)).toEqual([LIBRARY_DB]);
  });

  it("keeps a book with its file, and forgets both", async () => {
    const lib = await library();
    const file = new Blob(["book bytes"]);
    await lib.add(record("h1", { title: "One" }), file);
    expect((await lib.get("h1"))?.title).toBe("One");
    expect(await (await lib.file("h1"))?.text()).toBe("book bytes");
    await lib.savePosition("h1", "epubcfi(/6/2)", 5);
    await lib.remove("h1");
    expect(await lib.get("h1")).toBeNull();
    expect(await lib.file("h1")).toBeNull();
    expect(await lib.list()).toEqual([]);
    await lib.add(record("h1"), file);
    expect(await lib.get("h1")).toMatchObject({ location: null, locationAt: 0 });
  });

  it("lists the book read last first, then the newest import", async () => {
    const lib = await library();
    await lib.add(record("old", { addedAt: 1 }), new Blob(["a"]));
    await lib.add(record("new", { addedAt: 2 }), new Blob(["b"]));
    await lib.add(record("read", { addedAt: 0 }), new Blob(["c"]));
    expect((await lib.list()).map((b) => b.hash)).toEqual(["new", "old", "read"]);
    await lib.savePosition("read", "epubcfi(/6/2)", 10);
    expect((await lib.list()).map((b) => b.hash)).toEqual(["read", "new", "old"]);
  });

  it("keeps the latest move when two tabs write the position out of order", async () => {
    const lib = await library();
    await lib.add(record("h"), new Blob(["x"]));
    expect(await lib.savePosition("h", "later", 200)).toBe(true);
    expect(await lib.savePosition("h", "earlier", 100)).toBe(false);
    expect(await lib.get("h")).toMatchObject({ location: "later", locationAt: 200 });
  });

  it("never writes a book back when the reader moves: WebKit would lose its cover", async () => {
    const lib = await library();
    const cover = new Blob(["png bytes"], { type: "image/png" });
    await lib.add(record("h", { cover }), new Blob(["x"]));
    const put = vi.spyOn(IDBObjectStore.prototype, "put");
    await lib.savePosition("h", "epubcfi(/6/2)", 1);
    await lib.savePosition("h", "epubcfi(/6/4)", 2);
    expect(put.mock.contexts.map((store) => (store as IDBObjectStore).name)).toEqual(["positions", "positions"]);
    expect(await (await lib.get("h"))?.cover?.text()).toBe("png bytes");
  });

  it("keeps the positions of a library from version 1, which held them in the book", async () => {
    const factory = new IDBFactory();
    const v1 = await new Promise<IDBDatabase>((resolve) => {
      const open = factory.open(LIBRARY_DB, 1);
      open.onupgradeneeded = () => {
        open.result.createObjectStore("books", { keyPath: "hash" });
        open.result.createObjectStore("files", { keyPath: "hash" });
      };
      open.onsuccess = () => resolve(open.result);
    });
    const tx = v1.transaction("books", "readwrite");
    tx.objectStore("books").put(record("read", { location: "epubcfi(/6/8)", locationAt: 7 }));
    tx.objectStore("books").put(record("unread"));
    await new Promise((resolve) => (tx.oncomplete = resolve));
    v1.close();

    const lib = await Library.open(factory);
    expect(await lib.get("read")).toMatchObject({ location: "epubcfi(/6/8)", locationAt: 7 });
    expect(await lib.get("unread")).toMatchObject({ location: null, locationAt: 0 });
    expect((await lib.list()).map((b) => b.hash)).toEqual(["read", "unread"]);
  });

  it("does not invent a book for a position", async () => {
    const lib = await library();
    expect(await lib.savePosition("ghost", "x", 1)).toBe(false);
    expect(await lib.get("ghost")).toBeNull();
  });
});

describe("importing a file", () => {
  it("imports an EPUB with its title, authors and cover, keyed by its hash", async () => {
    const lib = await library();
    const file = await pickedFile(epub3Entries(), "hound.epub", "application/epub+zip");
    const result = await lib.importFile(file, 1234);
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.existed).toBe(false);
    expect(result.book).toMatchObject({
      hash: await sha256Hex(file),
      title: "The Hound of the Baskervilles",
      authors: ["Arthur Conan Doyle", "Sidney Paget"],
      language: "en-GB",
      identifier: "urn:uuid:1234-abcd",
      size: file.size,
      addedAt: 1234,
      location: null,
    });
    expect(result.book.cover?.type).toBe("image/png");
    expect((await lib.file(result.book.hash))?.size).toBe(file.size);
  });

  it("imports a file named without its extension and of a generic type", async () => {
    const lib = await library();
    const result = await lib.importFile(await pickedFile(epub3Entries(), "download", "application/octet-stream"));
    expect(result).toMatchObject({ ok: true, book: { title: "The Hound of the Baskervilles" } });
  });

  it("holds one book for the same file imported twice, and keeps its position", async () => {
    const lib = await library();
    const bytes = await zip(epub3Entries());
    const first = await lib.importFile(new File([bytes], "a.epub"), 1);
    if (!first.ok) throw new Error("first import failed");
    await lib.savePosition(first.book.hash, "epubcfi(/6/4)", 50);
    const second = await lib.importFile(new File([bytes], "renamed.epub"), 2);
    expect(second).toMatchObject({ ok: true, existed: true, book: { location: "epubcfi(/6/4)", addedAt: 1 } });
    expect(await lib.list()).toHaveLength(1);
  });

  it("refuses a protected book and stores nothing of it", async () => {
    const lib = await library();
    const protectedBook = epub3Entries({
      "META-INF/encryption.xml": `<encryption xmlns:enc="http://www.w3.org/2001/04/xmlenc#"><enc:EncryptedData>
        <enc:EncryptionMethod Algorithm="http://www.w3.org/2001/04/xmlenc#aes128-cbc"/></enc:EncryptedData></encryption>`,
      "META-INF/rights.xml": "<rights/>",
    });
    const result = await lib.importFile(await pickedFile(protectedBook, "drm.epub"));
    expect(result).toEqual({ ok: false, reason: "protected", protection: "adept" });
    expect(await lib.list()).toEqual([]);
  });

  it("imports a book whose only encryption is an obfuscated font", async () => {
    const lib = await library();
    const fonts = epub3Entries({
      "META-INF/encryption.xml": `<encryption xmlns:enc="http://www.w3.org/2001/04/xmlenc#"><enc:EncryptedData>
        <enc:EncryptionMethod Algorithm="http://www.idpf.org/2008/embedding"/></enc:EncryptedData></encryption>`,
    });
    expect((await lib.importFile(await pickedFile(fonts, "se.epub"))).ok).toBe(true);
  });

  it("refuses a file that is not an EPUB", async () => {
    const lib = await library();
    expect(await lib.importFile(new File(["%PDF-1.7 …"], "paper.epub"))).toEqual({ ok: false, reason: "notEpub" });
    expect(await lib.importFile(await pickedFile({ "a.txt": "x" }, "notes.zip"))).toEqual({
      ok: false,
      reason: "notEpub",
    });
    expect(await lib.list()).toEqual([]);
  });

  it("says a truncated archive cannot be read", async () => {
    const lib = await library();
    const whole = await zip(epub3Entries());
    const truncated = new File([whole.slice(0, 40)], "cut.epub");
    expect(await lib.importFile(truncated)).toEqual({ ok: false, reason: "unreadable" });
  });

  it("says so when the device refuses to store the book", async () => {
    const lib = await library();
    vi.spyOn(lib, "add").mockRejectedValue(new DOMException("full", "QuotaExceededError"));
    expect(await lib.importFile(await pickedFile(epub3Entries(), "big.epub"))).toEqual({
      ok: false,
      reason: "storage",
    });
  });
});

describe("titleFromName", () => {
  it("makes a title out of a file name", () => {
    expect(titleFromName("Moby_Dick.epub")).toBe("Moby Dick");
    expect(titleFromName("C:\\books\\War and Peace.EPUB")).toBe("War and Peace");
    expect(titleFromName("")).toBe("Livre sans titre");
    expect(titleFromName(undefined)).toBe("Livre sans titre");
  });
});
