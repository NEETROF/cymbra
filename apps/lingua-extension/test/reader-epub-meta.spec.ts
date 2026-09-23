import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { isZip, openArchive } from "@/reader/archive.ts";
import { isEpubArchive, readEpubMeta, resolvePath } from "@/reader/epub-meta.ts";
import { CHAPTER, CONTAINER, epub3Entries, OPF2, useNodeBlob, zip } from "./epub-fixtures.ts";

// What the library shows of a book is read from the archive at import: title, authors,
// language, identifier and cover (add-lingua-reader 3.2), and the archive is recognised as an
// EPUB by its content, not its name.

beforeEach(useNodeBlob);
afterEach(() => vi.unstubAllGlobals());

async function archiveOf(entries: Record<string, string | Uint8Array | null>) {
  return openArchive(await zip(entries));
}

describe("recognising an EPUB", () => {
  it("knows a zip by its first bytes", async () => {
    expect(await isZip(await zip(epub3Entries()))).toBe(true);
    expect(await isZip(new Blob(["%PDF-1.7"]))).toBe(false);
    expect(await isZip(new Blob([]))).toBe(false);
  });

  it("reads the mimetype entry", async () => {
    expect(await isEpubArchive(await archiveOf(epub3Entries()))).toBe(true);
    expect(await isEpubArchive(await archiveOf(epub3Entries({ mimetype: "application/zip" })))).toBe(false);
  });

  it("accepts an archive with a container and no mimetype entry", async () => {
    expect(await isEpubArchive(await archiveOf(epub3Entries({ mimetype: null })))).toBe(true);
  });

  it("refuses a zip that is neither", async () => {
    expect(await isEpubArchive(await archiveOf({ "photo.jpg": "x" }))).toBe(false);
  });
});

describe("readEpubMeta", () => {
  it("reads an EPUB 3 package: title, authors, language, unique identifier, cover", async () => {
    const meta = await readEpubMeta(await archiveOf(epub3Entries()), "fallback");
    expect(meta).toMatchObject({
      title: "The Hound of the Baskervilles",
      authors: ["Arthur Conan Doyle", "Sidney Paget"],
      language: "en-GB",
      identifier: "urn:uuid:1234-abcd",
    });
    expect(meta.cover?.type).toBe("image/png");
    expect(meta.cover?.size).toBeGreaterThan(0);
  });

  it("reads an EPUB 2 package: the cover by its meta, the author by its role", async () => {
    const entries = {
      mimetype: "application/epub+zip",
      "META-INF/container.xml": CONTAINER,
      "OEBPS/content.opf": OPF2,
      "cover.jpg": new Uint8Array([0xff, 0xd8, 0xff]),
    };
    const meta = await readEpubMeta(await archiveOf(entries), "fallback");
    expect(meta).toMatchObject({ title: "Pride and Prejudice", authors: ["Jane Austen"], identifier: "pp-2" });
    expect(meta.language).toBeNull();
    expect(meta.cover?.type).toBe("image/jpeg");
  });

  it("falls back on the file's name when the package cannot be read", async () => {
    const noOpf = epub3Entries({ "OEBPS/content.opf": null });
    expect(await readEpubMeta(await archiveOf(noOpf), "my-book")).toEqual({
      title: "my-book",
      authors: [],
      language: null,
      identifier: null,
      cover: null,
    });
    const broken = epub3Entries({ "OEBPS/content.opf": "<package><metadata>" });
    expect((await readEpubMeta(await archiveOf(broken), "my-book")).title).toBe("my-book");
    const noContainer = epub3Entries({ "META-INF/container.xml": null });
    expect((await readEpubMeta(await archiveOf(noContainer), "my-book")).title).toBe("my-book");
  });

  it("keeps a book whose package names no title nor cover", async () => {
    const bare = `<package xmlns="http://www.idpf.org/2007/opf"><metadata/><manifest/></package>`;
    const meta = await readEpubMeta(await archiveOf(epub3Entries({ "OEBPS/content.opf": bare })), "bare");
    expect(meta).toMatchObject({ title: "bare", authors: [], cover: null, identifier: null });
  });

  it("finds a cover by its id when nothing else names it", async () => {
    const opf = `<package xmlns="http://www.idpf.org/2007/opf"><metadata/><manifest>
      <item id="my-cover" href="c.gif" media-type="image/gif"/></manifest></package>`;
    const entries = { ...epub3Entries({ "OEBPS/content.opf": opf }), "OEBPS/c.gif": "GIF89a" };
    expect((await readEpubMeta(await archiveOf(entries), "x")).cover?.type).toBe("image/gif");
  });

  it("reads the text of a chapter it was asked for, and nothing it was not", async () => {
    const archive = await archiveOf(epub3Entries());
    expect(await archive.loadText("OEBPS/text/chapter 1.xhtml")).toBe(CHAPTER);
    expect(await archive.loadText("nope")).toBeNull();
    expect(await archive.loadBlob("nope")).toBeNull();
    expect(archive.getSize("OEBPS/text/chapter 1.xhtml")).toBe(CHAPTER.length);
    expect(archive.getSize("nope")).toBe(0);
    await archive.close();
  });
});

describe("resolvePath", () => {
  it("resolves a manifest href against the package document, decoding it", () => {
    expect(resolvePath("OEBPS/content.opf", "text/chapter%201.xhtml")).toBe("OEBPS/text/chapter 1.xhtml");
    expect(resolvePath("OEBPS/content.opf", "../cover.jpg")).toBe("cover.jpg");
    expect(resolvePath("content.opf", "img/a.png")).toBe("img/a.png");
  });
});
