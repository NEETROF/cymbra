// The capture's books (add-site-lingua-showcase D3): one public-domain chapter per studied language,
// committed as plain text under fixtures/books/ (a few `Name: value` lines, a blank line, then the
// paragraphs, one blank line apart), assembled into an EPUB 3 here with @zip.js/zip.js, as
// test/epub-fixtures.ts builds the reader's test books.
import { TextReader, Uint8ArrayWriter, ZipWriter } from "@zip.js/zip.js";

/**
 * A fixture book's text: its header (`Title`, `Author`, `Language`, `Chapter`) and its paragraphs.
 * @param {string} text
 * @returns {{ title: string, author: string, language: string, chapter: string, paragraphs: string[] }}
 */
export function parseBook(text) {
  const normalised = text.replace(/\r\n/g, "\n");
  const split = normalised.indexOf("\n\n");
  if (split < 0) throw new Error("a fixture book needs a header, a blank line, then its text");
  const header = {};
  for (const line of normalised.slice(0, split).split("\n")) {
    const m = /^(\w+):\s*(.+)$/.exec(line);
    if (!m) throw new Error(`not a header line: ${JSON.stringify(line)}`);
    header[m[1].toLowerCase()] = m[2].trim();
  }
  for (const key of ["title", "author", "language", "chapter"]) {
    if (!header[key]) throw new Error(`the fixture book's header has no ${key}`);
  }
  const paragraphs = normalised
    .slice(split + 2)
    .split(/\n\s*\n/)
    .map((p) => p.replace(/\s+/g, " ").trim())
    .filter(Boolean);
  if (paragraphs.length === 0) throw new Error(`${header.title} has no text`);
  return { title: header.title, author: header.author, language: header.language, chapter: header.chapter, paragraphs };
}

/** `text` safe in XML text and attribute values. */
export function xmlEscape(text) {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

/**
 * The EPUB 3 package of a book: path → content, `mimetype` first.
 * @param {ReturnType<typeof parseBook>} book
 * @param {string} id a stable identifier (the same book, the same bytes)
 */
export function epubEntries(book, id) {
  const lang = xmlEscape(book.language);
  const title = xmlEscape(book.title);
  const chapter = `<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="${lang}" lang="${lang}">
<head><meta charset="utf-8"/><title>${xmlEscape(book.chapter)}</title></head>
<body>
<h2>${xmlEscape(book.chapter)}</h2>
${book.paragraphs.map((p) => `<p>${xmlEscape(p)}</p>`).join("\n")}
</body>
</html>
`;
  const nav = `<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="${lang}" lang="${lang}">
<head><meta charset="utf-8"/><title>${title}</title></head>
<body><nav epub:type="toc"><ol><li><a href="chapter-1.xhtml">${xmlEscape(book.chapter)}</a></li></ol></nav></body>
</html>
`;
  const opf = `<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid" xml:lang="${lang}">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">${xmlEscape(id)}</dc:identifier>
    <dc:title>${title}</dc:title>
    <dc:creator>${xmlEscape(book.author)}</dc:creator>
    <dc:language>${lang}</dc:language>
    <meta property="dcterms:modified">2026-10-14T00:00:00Z</meta>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="c1" href="chapter-1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine><itemref idref="c1"/></spine>
</package>
`;
  return {
    mimetype: "application/epub+zip",
    "META-INF/container.xml": `<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>
`,
    "OEBPS/content.opf": opf,
    "OEBPS/nav.xhtml": nav,
    "OEBPS/chapter-1.xhtml": chapter,
  };
}

/** The book zipped: `mimetype` first and stored, the rest deflated, every date fixed. */
export async function assembleEpub(book, id) {
  const writer = new ZipWriter(new Uint8ArrayWriter());
  const lastModDate = new Date(Date.UTC(2026, 9, 14));
  for (const [name, content] of Object.entries(epubEntries(book, id))) {
    await writer.add(name, new TextReader(content), {
      level: name === "mimetype" ? 0 : 6,
      lastModDate,
      extendedTimestamp: false,
    });
  }
  return writer.close();
}
