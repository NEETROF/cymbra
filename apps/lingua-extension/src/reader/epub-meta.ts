import type { BookArchive } from "./archive.ts";

// What the library shows of a book, read at import from the archive itself: the container
// names the package document (the OPF), and the OPF carries the Dublin Core metadata and the
// manifest the cover is found in. Read here rather than through the renderer so an import
// needs no rendering engine, and is tested without one.

export const EPUB_MIMETYPE = "application/epub+zip";

const NS = {
  CONTAINER: "urn:oasis:names:tc:opendocument:xmlns:container",
  DC: "http://purl.org/dc/elements/1.1/",
};

export interface EpubMeta {
  title: string;
  authors: string[];
  /** The declared language (BCP 47), or null. */
  language: string | null;
  /** The publication's `dc:identifier` — kept as metadata only: publishers reuse it across
   *  editions, so the library keys a book by its file's hash instead (design D5). */
  identifier: string | null;
  cover: Blob | null;
}

/**
 * Whether an archive is an EPUB, by its content rather than its name or type: an Android file
 * picker may hand over `book` as `application/octet-stream`. The `mimetype` entry says so; an
 * archive that lacks one but carries the container document is read as one too.
 */
export async function isEpubArchive(archive: BookArchive): Promise<boolean> {
  const mimetype = await archive.loadText("mimetype");
  if (mimetype != null) return mimetype.trim() === EPUB_MIMETYPE;
  return archive.names.includes("META-INF/container.xml");
}

function parseXml(text: string | null): Document | null {
  if (!text) return null;
  const doc = new DOMParser().parseFromString(text, "application/xml");
  return doc.getElementsByTagName("parsererror").length > 0 ? null : doc;
}

/** The text of an element, whitespace collapsed; empty when absent. */
function textOf(el: Element | null | undefined): string {
  return (el?.textContent ?? "").replace(/\s+/g, " ").trim();
}

/** The archive path of `href`, relative to the package document at `base`. */
export function resolvePath(base: string, href: string): string {
  const url = new URL(href, `https://book.invalid/${base}`);
  return decodeURIComponent(url.pathname.slice(1));
}

/** The package document's path, as the container names it. */
async function opfPath(archive: BookArchive): Promise<string | null> {
  const container = parseXml(await archive.loadText("META-INF/container.xml"));
  const rootfile =
    container?.getElementsByTagNameNS(NS.CONTAINER, "rootfile")[0] ?? container?.getElementsByTagName("rootfile")[0];
  return rootfile?.getAttribute("full-path") ?? null;
}

function dcAll(opf: Document, name: string): Element[] {
  return [...opf.getElementsByTagNameNS(NS.DC, name)];
}

/** Authors: the creators marked as such (EPUB 2's `opf:role="aut"`), or every creator. */
function authorsOf(opf: Document): string[] {
  const creators = dcAll(opf, "creator");
  const role = (el: Element): string | null =>
    el.getAttribute("opf:role") ?? el.getAttributeNS("http://www.idpf.org/2007/opf", "role");
  const marked = creators.filter((el) => role(el) === "aut");
  return (marked.length > 0 ? marked : creators).map(textOf).filter((a) => a.length > 0);
}

/** The package's unique identifier, or its first `dc:identifier`. */
function identifierOf(opf: Document): string | null {
  const id = opf.documentElement.getAttribute("unique-identifier");
  const unique = id ? [...opf.getElementsByTagName("*")].find((el) => el.getAttribute("id") === id) : undefined;
  const text = textOf(unique ?? dcAll(opf, "identifier")[0]);
  return text || null;
}

/** The manifest item holding the cover image: EPUB 3's `cover-image`, then EPUB 2's meta. */
function coverItem(opf: Document): Element | null {
  const items = [...opf.getElementsByTagName("item")];
  const byProperty = items.find((it) => (it.getAttribute("properties") ?? "").split(/\s+/).includes("cover-image"));
  if (byProperty) return byProperty;
  const meta = [...opf.getElementsByTagName("meta")].find((m) => m.getAttribute("name") === "cover");
  const id = meta?.getAttribute("content");
  const byMeta = id ? items.find((it) => it.getAttribute("id") === id) : undefined;
  if (byMeta) return byMeta;
  return (
    items.find(
      (it) => /cover/i.test(it.getAttribute("id") ?? "") && (it.getAttribute("media-type") ?? "").startsWith("image/"),
    ) ?? null
  );
}

/**
 * Read a book's title, authors, language, identifier and cover. Whatever the archive does not
 * carry comes back empty; the title falls back on `fallbackTitle` (the file's name), so a
 * book never shows up nameless.
 */
export async function readEpubMeta(archive: BookArchive, fallbackTitle: string): Promise<EpubMeta> {
  const path = await opfPath(archive);
  const opf = path ? parseXml(await archive.loadText(path)) : null;
  if (!path || !opf) return { title: fallbackTitle, authors: [], language: null, identifier: null, cover: null };
  const item = coverItem(opf);
  const href = item?.getAttribute("href");
  const cover = href ? await archive.loadBlob(resolvePath(path, href), item?.getAttribute("media-type") ?? "") : null;
  return {
    title: textOf(dcAll(opf, "title")[0]) || fallbackTitle,
    authors: authorsOf(opf),
    language: textOf(dcAll(opf, "language")[0]) || null,
    identifier: identifierOf(opf),
    cover,
  };
}
