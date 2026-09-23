import { isZip, openArchive } from "./archive.ts";
import { type Protection, protectionOf } from "./drm.ts";
import { isEpubArchive, readEpubMeta } from "./epub-meta.ts";

// The reader's books, kept by the extension on the device (add-lingua-reader D5): a database
// of its own, apart from the store the background owns. That store holds the reader's STATE
// and has one owner so every surface agrees on it; a book is a blob tens of megabytes large,
// which must not cross a message, and the library has one writer anyway — the reader page.
// Its only concurrent writes are reading positions from two tabs, settled by their time.
//
// A book is keyed by the SHA-256 of its file: importing the same file twice yields one book,
// and a later change can match a position against the same key on another device. The OPF's
// identifier is kept as metadata only — publishers reuse it across editions.

export const LIBRARY_DB = "cymbra-lingua-library";
const DB_VERSION = 1;
const BOOKS = "books";
const FILES = "files";

export interface BookRecord {
  /** SHA-256 of the file, hex: the book's key. */
  hash: string;
  title: string;
  authors: string[];
  language: string | null;
  identifier: string | null;
  cover: Blob | null;
  /** The file's size in bytes. */
  size: number;
  /** When it was imported (epoch millis). */
  addedAt: number;
  /** Where the reader stopped (an EPUB CFI), or null before the first page. */
  location: string | null;
  /** When `location` was written (epoch millis): the latest write wins between two tabs. */
  locationAt: number;
}

interface FileRecord {
  hash: string;
  file: Blob;
}

/** Why a file was not added: each maps to one plain sentence (copy.ts). */
export type ImportFailure = "notEpub" | "unreadable" | "protected" | "storage";

export type ImportResult =
  { ok: true; book: BookRecord; existed: boolean } | { ok: false; reason: ImportFailure; protection?: Protection };

function promised<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("Lingua library request failed"));
  });
}

function committed(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error ?? new Error("Lingua library write failed"));
    tx.onabort = () => reject(tx.error ?? new Error("Lingua library write aborted"));
  });
}

/** Open the library database, creating its two object stores on first use. */
export function openLibraryDb(factory: IDBFactory = indexedDB): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = factory.open(LIBRARY_DB, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(BOOKS)) db.createObjectStore(BOOKS, { keyPath: "hash" });
      if (!db.objectStoreNames.contains(FILES)) db.createObjectStore(FILES, { keyPath: "hash" });
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("cannot open the Lingua library"));
    request.onblocked = () => reject(new Error("the Lingua library is blocked by another version"));
  });
}

/** The SHA-256 of a file, as lowercase hex. */
export async function sha256Hex(file: Blob): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", await file.arrayBuffer());
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/** A title out of a file name, for a book whose package names none. */
export function titleFromName(name: string | undefined): string {
  const base = (name ?? "").replace(/^.*[\\/]/, "").replace(/\.epub$/i, "");
  return base.replace(/[_]+/g, " ").trim() || "Livre sans titre";
}

export class Library {
  constructor(private readonly db: IDBDatabase) {}

  static async open(factory?: IDBFactory): Promise<Library> {
    return new Library(await openLibraryDb(factory));
  }

  /** Every book, the one read most recently first, then the newest import. */
  async list(): Promise<BookRecord[]> {
    const books = await promised<BookRecord[]>(this.db.transaction(BOOKS).objectStore(BOOKS).getAll());
    return books.sort((a, b) => b.locationAt - a.locationAt || b.addedAt - a.addedAt);
  }

  async get(hash: string): Promise<BookRecord | null> {
    return (await promised<BookRecord | undefined>(this.db.transaction(BOOKS).objectStore(BOOKS).get(hash))) ?? null;
  }

  /** The book's file, as it was imported. */
  async file(hash: string): Promise<Blob | null> {
    const got = await promised<FileRecord | undefined>(this.db.transaction(FILES).objectStore(FILES).get(hash));
    return got?.file ?? null;
  }

  /**
   * Keep a book and its file, unless a book with the same hash is already here — then the one
   * already kept stays as it is, reading position included, and is returned.
   */
  async add(book: BookRecord, file: Blob): Promise<{ book: BookRecord; existed: boolean }> {
    const tx = this.db.transaction([BOOKS, FILES], "readwrite");
    const books = tx.objectStore(BOOKS);
    const kept = await promised<BookRecord | undefined>(books.get(book.hash));
    if (!kept) {
      books.put(book);
      tx.objectStore(FILES).put({ hash: book.hash, file } satisfies FileRecord);
    }
    await committed(tx);
    return kept ? { book: kept, existed: true } : { book, existed: false };
  }

  /** Forget a book and its file. The cards captured from it keep their own copy of its name. */
  async remove(hash: string): Promise<void> {
    const tx = this.db.transaction([BOOKS, FILES], "readwrite");
    tx.objectStore(BOOKS).delete(hash);
    tx.objectStore(FILES).delete(hash);
    await committed(tx);
  }

  /**
   * Remember where the reader is. Two tabs on the same book both write: the later move (by
   * `at`) is the one kept, whatever order the writes land in. Returns whether it was kept.
   */
  async savePosition(hash: string, location: string, at: number): Promise<boolean> {
    const tx = this.db.transaction(BOOKS, "readwrite");
    const books = tx.objectStore(BOOKS);
    const book = await promised<BookRecord | undefined>(books.get(hash));
    const newer = !!book && at >= book.locationAt;
    if (book && newer) books.put({ ...book, location, locationAt: at });
    await committed(tx);
    return newer;
  }

  /**
   * Import a file picked by the reader: an EPUB by its content, whatever its name or type;
   * refused when protected; stored once per distinct file. Nothing is stored on a refusal.
   */
  async importFile(file: Blob & { name?: string }, now: number = Date.now()): Promise<ImportResult> {
    if (!(await isZip(file))) return { ok: false, reason: "notEpub" };
    let archive;
    try {
      archive = await openArchive(file);
    } catch {
      return { ok: false, reason: "unreadable" };
    }
    try {
      if (!(await isEpubArchive(archive))) return { ok: false, reason: "notEpub" };
      const protection = protectionOf(await archive.loadText("META-INF/encryption.xml"), archive.names);
      if (protection) return { ok: false, reason: "protected", protection };
      const meta = await readEpubMeta(archive, titleFromName(file.name));
      const book: BookRecord = {
        hash: await sha256Hex(file),
        ...meta,
        size: file.size,
        addedAt: now,
        location: null,
        locationAt: 0,
      };
      try {
        return { ok: true, ...(await this.add(book, file)) };
      } catch {
        return { ok: false, reason: "storage" };
      }
    } catch {
      return { ok: false, reason: "unreadable" };
    } finally {
      await archive.close().catch(() => {});
    }
  }
}
