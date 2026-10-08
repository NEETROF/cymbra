import { SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { NativeLanguage } from "../analyzer/types.ts";
import { INTERFACE_LANGUAGE_KEY, type InterfaceLanguage, isInterfaceLanguage } from "../i18n/language.ts";
import { isStorageFull } from "./auth-errors.ts";
import { DAILY_KEY, DAY_ONLY_DAILY_KEY, PER_LANGUAGE_DAILY_KEY, RETIRED_DAILY_KEY } from "./dailystats.ts";
import { type AsyncStorageArea, loadStored, nativeLanguageOfStored, ROOT_KEY } from "./storage.ts";

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
  DAILY_KEY,
  // Read until the first labelled write carries its counts over, and kept for a downgraded build
  // (add-lingua-native-language-sync-client D3) — never retired, as the day-only counts are not.
  PER_LANGUAGE_DAILY_KEY,
  // Read until the first per-language write carries its counts over (add-lingua-language-stats-review D3).
  DAY_ONLY_DAILY_KEY,
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

/**
 * Why the owner wrote, when a surface must do more than read the keys again: the reader chose
 * another native language (add-lingua-native-language-choice D2, D3). Every engine is then built
 * again for it, an extension page reloads in its interface language and the content script builds
 * its reading session anew. A sync, a Réglages change and a restore from a file carry none.
 */
export interface NativeLanguageChange {
  type: "native-language";
  /** The native language chosen, which the interface language key already names. */
  native: NativeLanguage;
}

export type StoreChangeReason = NativeLanguageChange;

/**
 * How long after a change of native language the owner refuses a backup that names another one and
 * comes without that reason (add-lingua-native-language-choice D3). A context that started before
 * the change — a session taken down while its engine answered, a page about to reload, a port not
 * restored yet — may still save the backup its engine holds, and saving it would undo the reader's
 * choice. Long enough for every open surface to have heard the change; after it, a backup restored
 * from a file names whatever native language it names.
 */
export const NATIVE_CHANGE_GUARD_MS = 30_000;

/** A backup refused by the owner: it names the native language the reader just left. */
export class StaleNativeLanguageError extends Error {
  constructor(named: NativeLanguage, chosen: NativeLanguage) {
    super(`a backup naming "${named}" was refused: the reader just chose "${chosen}"`);
    this.name = "StaleNativeLanguageError";
  }
}

export interface StoreChange {
  /** Strictly increasing, so a surface can ignore an echo of its own write. */
  rev: number;
  keys: string[];
  /** Why, when it is more than a write (D2); absent otherwise, as every change before it. */
  reason?: StoreChangeReason;
}

/** Whether a stored change names a reason this build knows: an unknown one is no reason. */
function reasonOf(value: unknown): StoreChangeReason | undefined {
  const reason = value as Partial<NativeLanguageChange> | null | undefined;
  return reason?.type === "native-language" && isInterfaceLanguage(reason.native)
    ? { type: "native-language", native: reason.native }
    : undefined;
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

/**
 * See to it that the device holds the interface language before any surface asks for it
 * (add-lingua-interface-language D3). The key is read first and kept when it already names a
 * language: a background wakes often (Chromium suspends an idle service worker within a minute),
 * every surface's first `store:get` waits behind this, and the backup it would otherwise parse is
 * the reader's whole data. It is computed from the stored backup only when the key is absent or
 * unknown — the one case the migration needs, a device updated with a profile already stored. From
 * then on the owner's backup writes keep it in step (`ownerArea`).
 */
export async function rememberInterfaceLanguage(
  area: AsyncStorageArea,
  preferences: AsyncStorageArea,
  pairs: readonly string[] = SHIPPED_PAIRS,
): Promise<void> {
  const held = await preferences.get(INTERFACE_LANGUAGE_KEY);
  if (isInterfaceLanguage(held[INTERFACE_LANGUAGE_KEY])) return;
  const got = await area.get(ROOT_KEY);
  await preferences.set({ [INTERFACE_LANGUAGE_KEY]: nativeLanguageOfStored(got[ROOT_KEY], pairs) });
}

/** The owner's handle: a store whose backup writes also keep the interface language's key in step. */
export interface OwnerArea extends AsyncStorageArea {
  /**
   * Write `items`, then announce them. With a `reason`, the mirror of the interface language runs
   * before the announcement rather than after it: a page that reloads on a change of native
   * language reads the key the new profile names (add-lingua-native-language-choice D2). For
   * `NATIVE_CHANGE_GUARD_MS` after such a write, a backup written without a reason that names
   * another native language is refused — rejected with `StaleNativeLanguageError`, nothing written.
   */
  set(items: Record<string, unknown>, reason?: StoreChangeReason): Promise<void>;
  /**
   * Settles once every mirror the writes so far have scheduled has landed, or failed and been
   * said. The mirror is off the write path — a write resolves, and announces, before it runs — so
   * whatever must see the key current right after a write (a test; nothing in the extension, which
   * reads the key when a surface opens) awaits this.
   */
  mirrored(): Promise<void>;
}

/**
 * The owner's handle on its own store: every write says which keys changed. Both things
 * that follow a mutation hang off this — telling the surfaces, and scheduling the sync —
 * so neither can be forgotten when the store moves again.
 *
 * A write of the backup also mirrors its profile's native language into `preferences`
 * (chrome.storage.local) under the interface language's key: a profile change, a restore from
 * a file, a full reset and the first hydration all come through here, so the key cannot go
 * stale behind the profile. The mirror costs the write nothing: the backup is written again at
 * every status change — each word clicked — and parsing it (the reader's whole data) for a
 * profile that has not moved would delay every announcement, so the parse runs after the write
 * has resolved and announced, once for a burst of writes, and the key is written only when the
 * language differs from the one last mirrored. A mirror that cannot be written (the area full)
 * is said and costs the backup nothing: the key keeps its previous value, absent meaning French,
 * and the next write that finds it still different tries again.
 */
export function ownerArea(
  area: AsyncStorageArea,
  announce: (keys: string[], reason?: StoreChangeReason) => void,
  preferences?: AsyncStorageArea,
  pairs: readonly string[] = SHIPPED_PAIRS,
  now: () => number = () => Date.now(),
): OwnerArea {
  // The root value last written, whether a parse of it is already due, the language last
  // mirrored, and the chain the mirrors run on, one after the other.
  let latest: unknown;
  let due = false;
  let mirrored: InterfaceLanguage | undefined;
  let chain: Promise<void> = Promise.resolve();
  // The native language last chosen, while a backup naming another is refused.
  let guard: { native: NativeLanguage; until: number } | null = null;

  /** Refuse a stale backup: parsed only while a change of native language is that recent. */
  const refuseStale = (items: Record<string, unknown>): void => {
    if (!guard || !(ROOT_KEY in items)) return;
    if (now() >= guard.until) {
      guard = null;
      return;
    }
    const named = nativeLanguageOfStored(items[ROOT_KEY], pairs);
    if (named !== guard.native) throw new StaleNativeLanguageError(named, guard.native);
  };

  const mirror = async (): Promise<void> => {
    due = false;
    const language = nativeLanguageOfStored(latest, pairs);
    if (!preferences || language === mirrored) return;
    try {
      await preferences.set({ [INTERFACE_LANGUAGE_KEY]: language });
      mirrored = language;
    } catch (e) {
      console.warn("[Cymbra Lingua] could not mirror the interface language:", e);
    }
  };

  return {
    get: (keys) => area.get(keys),
    set: async (items, reason) => {
      if (!reason) refuseStale(items);
      await area.set(items);
      if (reason) guard = { native: reason.native, until: now() + NATIVE_CHANGE_GUARD_MS };
      if (preferences && ROOT_KEY in items) {
        latest = items[ROOT_KEY];
        if (reason) {
          // Before the announcement, after any mirror already due: each reads `latest`, this backup.
          chain = chain.then(mirror);
          await chain;
        } else if (!due) {
          // A macrotask later: writes that land in the same turn share one parse.
          due = true;
          chain = chain.then(() => new Promise<void>((resolve) => setTimeout(resolve, 0))).then(mirror);
        }
      }
      if (reason) announce(Object.keys(items), reason);
      else announce(Object.keys(items));
    },
    mirrored: () => chain,
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
 * Copy the reader's data out of chrome.storage.local, once, then release the copy: it sat in
 * the area whose fullness the move exists to escape, and it was only ever a rollback path
 * for the release that has now been verified on device (design D5).
 *
 * Two things this has to survive, both seen on a reader's phone:
 * - **the area being full.** Its own copies are what fill it, and the mark cannot be
 *   written into a full area — which would leave the migration "unfinished" for ever. A
 *   working extension beats a rollback path: the copies are dropped, then the mark lands.
 * - **a second run after a partial one.** Only keys the store does not already hold are
 *   copied, so a re-run can never put a stale copy over what the reader has done since.
 */
export async function migrateStore(from: AsyncStorageArea, to: AsyncStorageArea): Promise<string[]> {
  const marks = await from.get(MIGRATED_KEY);
  if (marks[MIGRATED_KEY] === true) {
    await dropPreviousCopies(from, to);
    return [];
  }
  const previous = await from.get([...STORE_KEYS]);
  const already = await to.get([...STORE_KEYS]);
  const items = Object.fromEntries(Object.entries(previous).filter(([key]) => !(key in already)));
  const moved = Object.keys(items);
  if (moved.length > 0) await to.set(items);
  try {
    await from.set({ [MIGRATED_KEY]: true });
  } catch (e) {
    if (!isStorageFull(e)) throw e;
    await from.set(Object.fromEntries([...STORE_KEYS].map((key) => [key, null])));
    await from.set({ [MIGRATED_KEY]: true });
    return moved;
  }
  await dropPreviousCopies(from, to);
  return moved;
}

/**
 * Let go of what the previous area still holds, key by key, and only where the store has it:
 * a copy is dropped because it is redundant, never because a mark says it should be.
 */
async function dropPreviousCopies(from: AsyncStorageArea, to: AsyncStorageArea): Promise<void> {
  const left = await from.get([...STORE_KEYS]);
  const names = Object.keys(left).filter((key) => left[key] != null);
  if (names.length === 0) return;
  const held = await to.get(names);
  const redundant = names.filter((key) => key in held);
  if (redundant.length > 0) await from.set(Object.fromEntries(redundant.map((key) => [key, null])));
}

/** Keys the reader's data no longer lives under: dropped wherever they are left. */
export const RETIRED_KEYS = [RETIRED_DAILY_KEY] as const;

/**
 * Forget the retired keys in every area that may still hold them — the whole-document daily
 * counts (refine-lingua-reading-stats), so no sync ever pushes them. Idempotent: nothing
 * writes a retired key, so after the first start there is nothing left to drop.
 */
export async function dropRetiredKeys(...areas: AsyncStorageArea[]): Promise<void> {
  for (const area of areas) {
    const left = await area.get([...RETIRED_KEYS]);
    const names = Object.keys(left);
    if (names.length > 0) await area.set(Object.fromEntries(names.map((key) => [key, null])));
  }
}

/**
 * Follow the owner's writes. Any context can: the marker lives in chrome.storage.local, so
 * a content script hears it like an extension page does.
 */
export function watchStore(onChanged: (keys: string[], reason?: StoreChangeReason) => void): () => void {
  const listener = (changes: Record<string, chrome.storage.StorageChange>, areaName: string): void => {
    if (areaName !== "local") return;
    const change = changes[STORE_CHANGED_KEY]?.newValue as StoreChange | undefined;
    if (!Array.isArray(change?.keys)) return;
    const reason = reasonOf(change.reason);
    if (reason) onChanged(change.keys, reason);
    else onChanged(change.keys);
  };
  chrome.storage.onChanged.addListener(listener);
  // What a context that outlives what it watched for calls: the content script's reading session,
  // built anew when the native language changes (add-lingua-native-language-choice D3).
  return () => chrome.storage.onChanged.removeListener(listener);
}

/** The saved engine backup, whenever the store says it changed; returns how to stop watching. */
export function watchBackup(area: AsyncStorageArea, onBackup: (backup: string) => void): () => void {
  return watchStore((keys) => {
    if (!keys.includes(ROOT_KEY)) return;
    void loadStored(area).then((stored) => {
      if (stored.kind === "v2") onBackup(stored.backup);
    });
  });
}
