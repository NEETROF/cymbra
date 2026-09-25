import { BlobReader, BlobWriter, configure, TextWriter, ZipReader } from "@zip.js/zip.js";

// A book file seen as a zip archive — what the import reads its metadata and its encryption
// declaration from, and what foliate-js renders from. One reader for both, the one foliate-js
// is built against (@zip.js/zip.js, its `lib/zip-core.js` in the bundle: see build.mjs). It
// reads lazily out of the Blob, so a large illustrated book is never loaded whole.

// No worker (an extension page's CSP and Safari's extension process both prefer none) and
// no bundled codec: the browser's own DecompressionStream inflates.
configure({ useWebWorkers: false });

/** The shape foliate-js's EPUB class loads a book through (its `makeZipLoader`). */
export interface BookArchive {
  /** Every entry's path, as the archive names it. */
  readonly names: readonly string[];
  loadText(name: string): Promise<string | null>;
  loadBlob(name: string, type?: string): Promise<Blob | null>;
  getSize(name: string): number;
  /** Release the reader (the Blob itself stays wherever it lives). */
  close(): Promise<void>;
}

/** The four bytes every zip archive starts with — "PK\x03\x04". */
export async function isZip(file: Blob): Promise<boolean> {
  const head = new Uint8Array(await file.slice(0, 4).arrayBuffer());
  return head[0] === 0x50 && head[1] === 0x4b && head[2] === 0x03 && head[3] === 0x04;
}

/** Open `file` as a zip archive. Throws when it is not one, or is unreadable. */
export async function openArchive(file: Blob): Promise<BookArchive> {
  const reader = new ZipReader(new BlobReader(file));
  const entries = await reader.getEntries();
  const byName = new Map(entries.filter((e) => !e.directory).map((e) => [e.filename, e]));
  return {
    names: [...byName.keys()],
    async loadText(name) {
      const entry = byName.get(name);
      return entry && !entry.directory ? entry.getData(new TextWriter()) : null;
    },
    async loadBlob(name, type) {
      const entry = byName.get(name);
      return entry && !entry.directory ? entry.getData(new BlobWriter(type)) : null;
    },
    getSize: (name) => byName.get(name)?.uncompressedSize ?? 0,
    close: () => reader.close(),
  };
}
