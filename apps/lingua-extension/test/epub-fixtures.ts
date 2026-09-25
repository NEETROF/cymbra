import { Blob as NodeBlob, File as NodeFile } from "node:buffer";
import { vi } from "vitest";
import { BlobWriter, TextReader, Uint8ArrayReader, ZipWriter } from "@zip.js/zip.js";

// EPUB files built in memory for the reader's tests. jsdom's Blob has no `arrayBuffer()` and
// does not survive fake-indexeddb's structured clone, so the tests that read files swap in
// Node's own Blob and File (the ones a browser's match: sliceable, readable, cloneable).

/** Use Node's Blob and File for the current test file. Call from `beforeEach`. */
export function useNodeBlob(): void {
  vi.stubGlobal("Blob", NodeBlob);
  vi.stubGlobal("File", NodeFile);
}

export const CONTAINER = `<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>`;

/** An EPUB 3 package with a title, two authors, a language, an identifier and a cover. */
export const OPF3 = `<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="isbn">urn:isbn:0000</dc:identifier>
    <dc:identifier id="uid">urn:uuid:1234-abcd</dc:identifier>
    <dc:title>  The   Hound of the Baskervilles </dc:title>
    <dc:creator>Arthur Conan Doyle</dc:creator>
    <dc:creator>Sidney Paget</dc:creator>
    <dc:language>en-GB</dc:language>
  </metadata>
  <manifest>
    <item id="c1" href="text/chapter%201.xhtml" media-type="application/xhtml+xml"/>
    <item id="cov" href="images/cover.png" media-type="image/png" properties="cover-image"/>
  </manifest>
  <spine><itemref idref="c1"/></spine>
</package>`;

/** An EPUB 2 package: the cover named by a meta, the author by its role. */
export const OPF2 = `<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" xmlns:opf="http://www.idpf.org/2007/opf" version="2.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Pride and Prejudice</dc:title>
    <dc:creator opf:role="aut">Jane Austen</dc:creator>
    <dc:creator opf:role="ill">C. E. Brock</dc:creator>
    <dc:identifier>pp-2</dc:identifier>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="cover-img" href="../cover.jpg" media-type="image/jpeg"/>
  </manifest>
  <spine/>
</package>`;

export const CHAPTER = `<?xml version="1.0"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><p>It was a dark and stormy night.</p></body></html>`;

/** 1×1 PNG, the cover. */
export const PNG = new Uint8Array([
  0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0x0d, 0x49, 0x48, 0x44, 0x52, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
  0, 0, 0, 0x1f, 0x15, 0xc4, 0x89,
]);

export type Entries = Record<string, string | Uint8Array | null>;

/** The entries of a well-formed EPUB 3, to extend or override. */
export function epub3Entries(extra: Entries = {}): Entries {
  return {
    mimetype: "application/epub+zip",
    "META-INF/container.xml": CONTAINER,
    "OEBPS/content.opf": OPF3,
    "OEBPS/text/chapter 1.xhtml": CHAPTER,
    "OEBPS/images/cover.png": PNG,
    ...extra,
  };
}

/** Zip `entries` into a Blob; `null` values are left out, `mimetype` goes first, stored. */
export async function zip(entries: Record<string, string | Uint8Array | null>): Promise<Blob> {
  const writer = new ZipWriter(new BlobWriter("application/epub+zip"));
  const names = Object.keys(entries).sort((a, b) => (a === "mimetype" ? -1 : b === "mimetype" ? 1 : 0));
  for (const name of names) {
    const value = entries[name];
    if (value == null) continue;
    const reader = typeof value === "string" ? new TextReader(value) : new Uint8ArrayReader(value);
    await writer.add(name, reader, { level: name === "mimetype" ? 0 : 5 });
  }
  return writer.close();
}

/** A picked file: `name` and `type` as a browser's file picker reports them. */
export async function pickedFile(
  entries: Record<string, string | Uint8Array | null>,
  name: string,
  type = "",
): Promise<File> {
  return new File([await zip(entries)], name, { type });
}
