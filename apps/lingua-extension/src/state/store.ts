import { type AsyncStorageArea, loadStored, ROOT_KEY } from "./storage.ts";

// Where the reader's own data lives (change: move-lingua-store-to-indexeddb). The engine
// backup, the daily statistics and the sync cursors grow with the reader, and
// chrome.storage.local caps an extension in the single-digit megabytes — enforced on
// WRITE, so a full area fails every mutation at once (dogfooding, TestFlight 70/71).
// IndexedDB's quota is measured against the disk instead.
//
// A content script cannot open the extension's IndexedDB — it runs in the visited page's
// origin — so the background owns the database (design D1) and every other surface reaches
// it through this module's messaged area, which speaks the same `AsyncStorageArea` the
// reading, review, stats and sync code already consume (D2). Preferences, tokens and the
// transient marks stay in chrome.storage.local (D4).

const DB_NAME = "cymbra-lingua";
const DB_VERSION = 1;
const OBJECT_STORE = "state";

/** The reader's data, moved out of chrome.storage.local. */
export const STORE_KEYS = [
  ROOT_KEY,
  "cymbra-lingua-daily",
  "cymbra-lingua-device",
  "cymbra-lingua-status-cursor",
  "cymbra-lingua-card-cursor",
  "cymbra-lingua-erased-at",
] as const;

/**
 * The marker the owner bumps after every write, in chrome.storage.local, naming the keys
 * that changed. Surfaces follow the store through its `storage.onChanged` — the one channel
 * that reaches every context (content scripts included), needs no permission, and survives
 * a background page the browser suspends, which a long-lived port does not.
 */
export const STORE_CHANGED_KEY = "cymbra-lingua-store-changed";

/** Set once the reader's data has been copied out of chrome.storage.local (design D5). */
export const MIGRATED_KEY = "cymbra-lingua-store-migrated";

export interface StoreChange {
  /** Strictly increasing, so a surface can ignore an echo of its own write. */
  rev: number;
  keys: string[];
}

export type StoreMessage =
  { type: "store:get"; keys: string | string[] | null } | { type: "store:set"; items: Record<string, unknown> };

export interface StoreReply {
  ok: boolean;
  items?: Record<string, unknown>;
}

export function isStoreMessage(m: unknown): m is StoreMessage {
  const type = (m as { type?: unknown } | null)?.type;
  return type === "store:get" || type === "store:set";
}

/** Open the database, creating its single object store on first use. */
export function openStore(factory: IDBFactory = indexedDB): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = factory.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      if (!request.result.objectStoreNames.contains(OBJECT_STORE)) {
        request.result.createObjectStore(OBJECT_STORE);
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("cannot open the Lingua store"));
    request.onblocked = () => reject(new Error("the Lingua store is blocked by another version"));
  });
}

function promised<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("Lingua store request failed"));
  });
}

/** The owner's area: the database itself. Only the background builds one. */
export function idbArea(db: IDBDatabase): AsyncStorageArea {
  return {
    async get(keys) {
      const tx = db.transaction(OBJECT_STORE, "readonly");
      const store = tx.objectStore(OBJECT_STORE);
      const wanted = keys == null ? await promised(store.getAllKeys()) : Array.isArray(keys) ? keys : [keys];
      const out: Record<string, unknown> = {};
      for (const key of wanted) {
        const value = await promised(store.get(key as IDBValidKey));
        if (value !== undefined) out[String(key)] = value;
      }
      return out;
    },
    async set(items) {
      const tx = db.transaction(OBJECT_STORE, "readwrite");
      const store = tx.objectStore(OBJECT_STORE);
      for (const [key, value] of Object.entries(items)) {
        // A null means "forget this" (the callers' convention), like a missing key.
        if (value === null) await promised<undefined>(store.delete(key));
        else await promised<IDBValidKey>(store.put(value, key));
      }
      await new Promise<void>((resolve, reject) => {
        tx.oncomplete = () => resolve();
        tx.onerror = () => reject(tx.error ?? new Error("Lingua store write failed"));
        tx.onabort = () => reject(tx.error ?? new Error("Lingua store write aborted"));
      });
    },
  };
}

export type RuntimeSend = (message: unknown) => Promise<unknown>;

const runtimeSend: RuntimeSend = (message) => chrome.runtime.sendMessage(message);

/**
 * Every surface's area: the owner answers. Awaiting the reply is also what keeps a
 * suspended background alive until the write has landed.
 */
export function messagedArea(send: RuntimeSend = runtimeSend): AsyncStorageArea {
  return {
    async get(keys) {
      const reply = (await send({ type: "store:get", keys } satisfies StoreMessage)) as StoreReply | undefined;
      return reply?.items ?? {};
    },
    async set(items) {
      await send({ type: "store:set", items } satisfies StoreMessage);
    },
  };
}

/**
 * Copy the reader's data out of chrome.storage.local, once. The copies are LEFT in place
 * for one release, so a rollback still finds a deck (design D5); the mark is written last,
 * so a run cut short is simply redone.
 */
export async function migrateStore(from: AsyncStorageArea, to: AsyncStorageArea): Promise<string[]> {
  const marks = await from.get(MIGRATED_KEY);
  if (marks[MIGRATED_KEY] === true) return [];
  const previous = await from.get([...STORE_KEYS]);
  const moved = Object.keys(previous);
  if (moved.length > 0) await to.set(previous);
  await from.set({ [MIGRATED_KEY]: true });
  return moved;
}

/**
 * Follow the owner's writes. Any context can: the marker lives in chrome.storage.local, so
 * a content script hears it like an extension page does.
 */
export function watchStore(onChanged: (keys: string[]) => void): void {
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName !== "local") return;
    const change = changes[STORE_CHANGED_KEY]?.newValue as StoreChange | undefined;
    if (Array.isArray(change?.keys)) onChanged(change.keys);
  });
}

/** The saved engine backup, whenever the store says it changed. */
export function watchBackup(area: AsyncStorageArea, onBackup: (backup: string) => void): void {
  watchStore((keys) => {
    if (!keys.includes(ROOT_KEY)) return;
    void loadStored(area).then((stored) => {
      if (stored.kind === "v2") onBackup(stored.backup);
    });
  });
}
