// Where the downloaded model lives (add-lingua-translation-delivery D5): an IndexedDB database of
// its own, `lingua-model`, apart from the reader's store — and never chrome.storage.local, whose
// quota is what filled up in the exposure-counter incident. Each file's DECOMPRESSED bytes are
// kept under their sha256, so a cold start reads them (~24 ms, measured) instead of decompressing
// 37 MB again; a `complete` record says which model version has every file.
//
// Only verified bytes are ever written, so a file that is present IS verified: resuming a download
// fetches exactly the files that are missing. Every operation opens and closes its own connection —
// an open one would hold off the deletion that turning the setting off performs.

import { MODEL_ROLES, type ModelManifest } from "./model-manifest.ts";

export const MODEL_DB = "lingua-model";
const VERSION = 1;
const FILES = "files";
const META = "meta";
const COMPLETE = "complete";

interface CompleteRecord {
  version: string;
  files: string[];
}

export interface ModelDb {
  /** Whether a verified file is stored under this sha256. */
  has(sha256: string): Promise<boolean>;
  get(sha256: string): Promise<Uint8Array | null>;
  /** Store a file's bytes, already verified against `sha256`. */
  put(sha256: string, bytes: Uint8Array): Promise<void>;
  /** Record that every file of `manifest` is stored. */
  markComplete(manifest: ModelManifest): Promise<void>;
  /** Whether `manifest`'s model is on the device, every file of it. */
  complete(manifest: ModelManifest): Promise<boolean>;
  /** Delete the whole database. */
  erase(): Promise<void>;
}

function request<T>(req: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error ?? new Error("IndexedDB request failed"));
  });
}

function done(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onabort = tx.onerror = () => reject(tx.error ?? new Error("IndexedDB transaction failed"));
  });
}

export function modelDb(factory: IDBFactory = indexedDB): ModelDb {
  const open = (): Promise<IDBDatabase> =>
    new Promise((resolve, reject) => {
      const req = factory.open(MODEL_DB, VERSION);
      req.onupgradeneeded = () => {
        const db = req.result;
        if (!db.objectStoreNames.contains(FILES)) db.createObjectStore(FILES);
        if (!db.objectStoreNames.contains(META)) db.createObjectStore(META);
      };
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error ?? new Error(`${MODEL_DB} could not be opened`));
    });

  /** Run `body` in one transaction and close the connection whatever happens. */
  const within = async <T>(
    stores: string[],
    mode: IDBTransactionMode,
    body: (tx: IDBTransaction) => Promise<T>,
  ): Promise<T> => {
    const db = await open();
    try {
      const tx = db.transaction(stores, mode);
      const finished = done(tx);
      const result = await body(tx);
      await finished;
      return result;
    } finally {
      db.close();
    }
  };

  const count = (tx: IDBTransaction, sha256: string): Promise<number> => request(tx.objectStore(FILES).count(sha256));

  return {
    has: (sha256) => within([FILES], "readonly", async (tx) => (await count(tx, sha256)) > 0),

    get: (sha256) =>
      within([FILES], "readonly", async (tx) => {
        const value = (await request(tx.objectStore(FILES).get(sha256))) as ArrayBuffer | Uint8Array | undefined;
        if (value == null) return null;
        return value instanceof Uint8Array ? value : new Uint8Array(value);
      }),

    put: (sha256, bytes) =>
      within([FILES], "readwrite", async (tx) => {
        await request(tx.objectStore(FILES).put(bytes, sha256));
      }),

    markComplete: (manifest) =>
      within([META], "readwrite", async (tx) => {
        const record: CompleteRecord = {
          version: manifest.version,
          files: MODEL_ROLES.map((role) => manifest.files[role].sha256),
        };
        await request(tx.objectStore(META).put(record, COMPLETE));
      }),

    complete: (manifest) =>
      within([FILES, META], "readonly", async (tx) => {
        const record = (await request(tx.objectStore(META).get(COMPLETE))) as CompleteRecord | undefined;
        if (record?.version !== manifest.version) return false;
        for (const role of MODEL_ROLES) {
          if ((await count(tx, manifest.files[role].sha256)) === 0) return false;
        }
        return true;
      }),

    erase: () =>
      new Promise<void>((resolve, reject) => {
        const req = factory.deleteDatabase(MODEL_DB);
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error ?? new Error(`${MODEL_DB} could not be deleted`));
      }),
  };
}
