import type { StudiedLanguage } from "../analyzer/types.ts";
import { reader as frReader } from "../i18n/fr/reader.ts";
import { isZip, openArchive } from "./archive.ts";
import { type Protection, protectionOf } from "./drm.ts";
import { isEpubArchive, readEpubMeta } from "./epub-meta.ts";

// The reader's books, kept by the extension on the device (add-lingua-reader D5): a database
// of its own, apart from the store the background owns. That store holds the reader's STATE
// and has one owner so every surface agrees on it; a book is a blob tens of megabytes large,
// which must not cross a message, and the library has one writer anyway — the reader page.
// Its only concurrent writes are reading positions from two tabs, settled by their time.
//
// A book's record is written once, at import; its position lives in a store of its own. WebKit
// loses a Blob read back from IndexedDB when the record holding it is written again with it —
// the cover of the book just read came back unreadable (`NotFoundError`) on iOS and iPadOS — and
// a position is written at every page turn.
//
// A book is keyed by the SHA-256 of its file: importing the same file twice yields one book,
// and a later change can match a position against the same key on another device. The OPF's
// identifier is kept as metadata only — publishers reuse it across editions.

export const LIBRARY_DB = "cymbra-lingua-library";
const DB_VERSION = 3;
const BOOKS = "books";
const FILES = "files";
const POSITIONS = "positions";
/** The words looked up in a book (add-lingua-chapter-recap D2), keyed by `[hash, chapter, lemma]`. */
const LOOKUPS = "lookups";
/** How many lookups a book had when its recap was last put off (D4), keyed by `hash`. */
const RECAPS = "recaps";

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

/**
 * A word or an expression whose card the reader opened in a book (add-lingua-chapter-recap D1): once
 * per word and chapter, on this device only — never synchronised, never in the backup.
 */
export interface LookupRecord {
  hash: string;
  /** The chapter's label (the table-of-contents entry the page sat in), empty in a book without one. */
  chapter: string;
  /** The key its status and card are filed under: a word's dictionary form, an expression lowercased. */
  lemma: string;
  /** The form met. */
  surface: string;
  sentence: string;
  /** The studied language the card was read in. */
  language: StudiedLanguage;
  expression: boolean;
  /** The gloss the card showed, kept for an expression the pack cannot gloss again. */
  gloss: string | null;
  /** When it was first looked up (epoch millis). */
  at: number;
}

interface RecapRecord {
  hash: string;
  /** How many lookups the book had when its recap was put off. */
  dismissedAt: number;
}

/** The lookups store's index of a book's records. */
const BY_BOOK = "hash";

/** A book as its store keeps it: everything but where the reader is. */
type StoredBook = Omit<BookRecord, "location" | "locationAt">;

interface FileRecord {
  hash: string;
  file: Blob;
}

interface PositionRecord {
  hash: string;
  location: string;
  locationAt: number;
}

function stored(book: BookRecord): StoredBook {
  const { hash, title, authors, language, identifier, cover, size, addedAt } = book;
  return { hash, title, authors, language, identifier, cover, size, addedAt };
}

function withPosition(book: StoredBook, position: PositionRecord | undefined): BookRecord {
  return { ...book, location: position?.location ?? null, locationAt: position?.locationAt ?? 0 };
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

/**
 * Open the library database, creating its object stores on first use. Version 1 kept the
 * position inside the book's record: the upgrade copies it to its own store, reading the books
 * without writing one back. Version 3 adds the lookups and the recaps; nothing existing moves.
 */
export function openLibraryDb(factory: IDBFactory = indexedDB): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = factory.open(LIBRARY_DB, DB_VERSION);
    request.onupgradeneeded = (event) => {
      const db = request.result;
      if (!db.objectStoreNames.contains(BOOKS)) db.createObjectStore(BOOKS, { keyPath: "hash" });
      if (!db.objectStoreNames.contains(FILES)) db.createObjectStore(FILES, { keyPath: "hash" });
      if (!db.objectStoreNames.contains(LOOKUPS))
        db.createObjectStore(LOOKUPS, { keyPath: ["hash", "chapter", "lemma"] }).createIndex(BY_BOOK, "hash");
      if (!db.objectStoreNames.contains(RECAPS)) db.createObjectStore(RECAPS, { keyPath: "hash" });
      if (db.objectStoreNames.contains(POSITIONS)) return;
      const positions = db.createObjectStore(POSITIONS, { keyPath: "hash" });
      if (event.oldVersion !== 1) return;
      const cursor = request.transaction!.objectStore(BOOKS).openCursor();
      cursor.onsuccess = () => {
        const row = cursor.result;
        if (!row) return;
        const { hash, location, locationAt } = row.value as BookRecord;
        if (location) positions.put({ hash, location, locationAt } satisfies PositionRecord);
        row.continue();
      };
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

/**
 * A title out of a file name, for a book whose package names none — or `untitled`, the catalogue's
 * « Livre sans titre » in the interface language of the import (localise-lingua-reading-surfaces):
 * written once into the book's record, stored data rather than display-time copy.
 */
export function titleFromName(name: string | undefined, untitled: string = frReader.untitled): string {
  const base = (name ?? "").replace(/^.*[\\/]/, "").replace(/\.epub$/i, "");
  return base.replace(/[_]+/g, " ").trim() || untitled;
}

export class Library {
  constructor(
    private readonly db: IDBDatabase,
    /** What a book without a title is called at import; the French when not given. */
    private readonly untitled: string = frReader.untitled,
  ) {}

  static async open(factory?: IDBFactory, untitled?: string): Promise<Library> {
    return new Library(await openLibraryDb(factory), untitled);
  }

  /** Every book, the one read most recently first, then the newest import. */
  async list(): Promise<BookRecord[]> {
    const tx = this.db.transaction([BOOKS, POSITIONS]);
    const [books, positions] = await Promise.all([
      promised<StoredBook[]>(tx.objectStore(BOOKS).getAll()),
      promised<PositionRecord[]>(tx.objectStore(POSITIONS).getAll()),
    ]);
    const at = new Map(positions.map((p) => [p.hash, p]));
    return books
      .map((b) => withPosition(b, at.get(b.hash)))
      .sort((a, b) => b.locationAt - a.locationAt || b.addedAt - a.addedAt);
  }

  async get(hash: string): Promise<BookRecord | null> {
    const tx = this.db.transaction([BOOKS, POSITIONS]);
    const [book, position] = await Promise.all([
      promised<StoredBook | undefined>(tx.objectStore(BOOKS).get(hash)),
      promised<PositionRecord | undefined>(tx.objectStore(POSITIONS).get(hash)),
    ]);
    return book ? withPosition(book, position) : null;
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
    const tx = this.db.transaction([BOOKS, FILES, POSITIONS], "readwrite");
    const books = tx.objectStore(BOOKS);
    const [kept, position] = await Promise.all([
      promised<StoredBook | undefined>(books.get(book.hash)),
      promised<PositionRecord | undefined>(tx.objectStore(POSITIONS).get(book.hash)),
    ]);
    if (!kept) {
      books.put(stored(book));
      tx.objectStore(FILES).put({ hash: book.hash, file } satisfies FileRecord);
    }
    await committed(tx);
    return kept ? { book: withPosition(kept, position), existed: true } : { book, existed: false };
  }

  /**
   * Forget a book, its file and the words looked up in it. The cards captured from it keep their
   * own copy of its name.
   */
  async remove(hash: string): Promise<void> {
    const tx = this.db.transaction([BOOKS, FILES, POSITIONS, LOOKUPS, RECAPS], "readwrite");
    tx.objectStore(BOOKS).delete(hash);
    tx.objectStore(FILES).delete(hash);
    tx.objectStore(POSITIONS).delete(hash);
    tx.objectStore(RECAPS).delete(hash);
    const lookups = tx.objectStore(LOOKUPS);
    const keys = lookups.index(BY_BOOK).openKeyCursor(hash);
    keys.onsuccess = () => {
      const row = keys.result;
      if (!row) return;
      lookups.delete(row.primaryKey);
      row.continue();
    };
    await committed(tx);
  }

  /**
   * Remember a word looked up in a book (add-lingua-chapter-recap D1): the first time in its
   * chapter only — opening it again there keeps what was met first. Returns whether it was new.
   */
  async recordLookup(lookup: LookupRecord): Promise<boolean> {
    const tx = this.db.transaction([BOOKS, LOOKUPS], "readwrite");
    const lookups = tx.objectStore(LOOKUPS);
    const [known, kept] = await Promise.all([
      promised<number>(tx.objectStore(BOOKS).count(lookup.hash)),
      promised<number>(lookups.count([lookup.hash, lookup.chapter, lookup.lemma])),
    ]);
    const fresh = known > 0 && kept === 0;
    if (fresh) lookups.put(lookup);
    await committed(tx);
    return fresh;
  }

  /** The words looked up in a book, in the order they were met. */
  async lookupsOf(hash: string): Promise<LookupRecord[]> {
    const tx = this.db.transaction(LOOKUPS);
    const all = await promised<LookupRecord[]>(tx.objectStore(LOOKUPS).index(BY_BOOK).getAll(hash));
    return all.sort((a, b) => a.at - b.at);
  }

  /** How many lookups the book had when its recap was last put off; null when it never was (D4). */
  async recapDismissedAt(hash: string): Promise<number | null> {
    const got = await promised<RecapRecord | undefined>(this.db.transaction(RECAPS).objectStore(RECAPS).get(hash));
    return got?.dismissedAt ?? null;
  }

  /** The recap was put off while the book had `lookups` lookups. */
  async dismissRecap(hash: string, lookups: number): Promise<void> {
    const tx = this.db.transaction(RECAPS, "readwrite");
    tx.objectStore(RECAPS).put({ hash, dismissedAt: lookups } satisfies RecapRecord);
    await committed(tx);
  }

  /**
   * Remember where the reader is. Two tabs on the same book both write: the later move (by
   * `at`) is the one kept, whatever order the writes land in. Returns whether it was kept.
   */
  async savePosition(hash: string, location: string, at: number): Promise<boolean> {
    const tx = this.db.transaction([BOOKS, POSITIONS], "readwrite");
    const positions = tx.objectStore(POSITIONS);
    const [known, position] = await Promise.all([
      promised<number>(tx.objectStore(BOOKS).count(hash)),
      promised<PositionRecord | undefined>(positions.get(hash)),
    ]);
    const newer = known > 0 && at >= (position?.locationAt ?? 0);
    if (newer) positions.put({ hash, location, locationAt: at } satisfies PositionRecord);
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
      const meta = await readEpubMeta(archive, titleFromName(file.name, this.untitled));
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
