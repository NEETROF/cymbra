import { DEFAULT_NATIVE, pairsOf, SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { NativeLanguage } from "../analyzer/types.ts";
import { INTERFACE_LANGUAGE_KEY, type InterfaceLanguage, isInterfaceLanguage } from "../i18n/language.ts";
import { isStorageFull } from "./auth-errors.ts";
import { DAILY_KEY, DAY_ONLY_DAILY_KEY, PER_LANGUAGE_DAILY_KEY, RETIRED_DAILY_KEY } from "./dailystats.ts";
import { type AsyncStorageArea, loadStored, nativeLanguageOfStored, ROOT_KEY, STORAGE_VERSION } from "./storage.ts";

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
 * its reading session anew. A restore from a file that names another native language is announced
 * as this change too (`RestoreFromFile`); a sync and a Réglages change carry none.
 */
export interface NativeLanguageChange {
  type: "native-language";
  /** The native language chosen, which the interface language key already names. */
  native: NativeLanguage;
}

export type StoreChangeReason = NativeLanguageChange;

/**
 * A backup restored from a file (add-lingua-native-language-choice, task 4.5): the reader's own
 * gesture, so the native language it names is theirs from then on. The owner records it as their
 * last choice (`LAST_NATIVE_KEY`) and, when it is another than theirs, announces a change of native
 * language, so that every surface follows it as it follows a choice. The one reason a surface's
 * write may carry: a change of native language is asked of the background (`lingua-native-language`).
 */
export interface RestoreFromFile {
  type: "restore";
}

export const RESTORE_FROM_FILE: RestoreFromFile = { type: "restore" };

/** Why the owner is asked to write, when it is more than a write: a change of native language, a restore from a file. */
export type StoreWriteReason = StoreChangeReason | RestoreFromFile;

/**
 * The native language the reader last chose, recorded by the owner beside the backup, in the store it
 * owns — so it outlives the background, which Chromium stops when idle and Safari suspends
 * (add-lingua-native-language-choice D3, task 4.5). A context that started before a change — a session
 * taken down while its engine answered, a page about to reload, a port not restored yet, a tab the
 * browser restores from its back/forward cache, which heard nothing while it was there — may still
 * save the backup its engine holds, at any time; saved, it would undo the reader's choice. So the
 * owner refuses a backup written without a reason that names another native language than this one.
 *
 * Written in the same write as the backup, by a change of native language and by a restore from a
 * file — never by a caller (`ownerArea`) — so the two never disagree in the area that holds them; it
 * is not one of `STORE_KEYS`, never having lived in chrome.storage.local. Absent on a device where
 * neither ever happened — every extension installed before it, every reader who never chose: no
 * context there can hold another native language's engine, so nothing is refused, and the backup is
 * not read for it.
 */
export const LAST_NATIVE_KEY = "cymbra-lingua-last-native";

/** A backup refused by the owner: it names another native language than the reader's last choice. */
export class StaleNativeLanguageError extends Error {
  constructor(named: NativeLanguage, chosen: NativeLanguage) {
    super(`a backup naming "${named}" was refused: the reader chose "${chosen}"`);
    this.name = "StaleNativeLanguageError";
  }
}

/**
 * The record read back: the native language it names, French when no shipped pair is glossed in it —
 * as a profile naming one is read (`nativeLanguageOf`, M22) — or null when nothing is recorded.
 */
function recordedNative(value: unknown, pairs: readonly string[]): NativeLanguage | null {
  if (!isInterfaceLanguage(value)) return null;
  return pairsOf(value, pairs).length > 0 ? value : DEFAULT_NATIVE;
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
  | { type: "store:get"; keys: string | string[] | null }
  | { type: "store:set"; items: Record<string, unknown>; reason?: RestoreFromFile };

export interface StoreReply {
  ok: boolean;
  items?: Record<string, unknown>;
}

export function isStoreMessage(m: unknown): m is StoreMessage {
  const type = (m as { type?: unknown } | null)?.type;
  return type === "store:get" || type === "store:set";
}

/** The reason a surface's write carries, as the owner hears it: a restore from a file, or none. */
export function surfaceWriteReason(message: StoreMessage): RestoreFromFile | undefined {
  if (message.type !== "store:set") return undefined;
  return (message.reason as { type?: unknown } | undefined)?.type === "restore" ? RESTORE_FROM_FILE : undefined;
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
   * Write `items`, then announce them. On a change of native language, the mirror of the interface
   * language runs before the announcement rather than after it: a page that reloads on the change
   * reads the key the new profile names (add-lingua-native-language-choice D2). A restore from a file
   * that names another native language than the reader's is announced as that change.
   *
   * The change, and a restore from a file, record the native language as the reader's last choice
   * (`LAST_NATIVE_KEY`), in the same write as the backup; from then on, a backup written without a
   * reason that names another is refused — rejected with `StaleNativeLanguageError`, nothing written,
   * nothing announced — whenever it comes, after a restart of the background as before it. A write
   * naming `LAST_NATIVE_KEY` itself is refused: the record is the owner's.
   */
  set(items: Record<string, unknown>, reason?: StoreWriteReason): Promise<void>;
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
 *
 * It also keeps the reader's choice of native language (`LAST_NATIVE_KEY`, `OwnerArea.set`). A
 * device where none was ever recorded reads one key once per background, and parses nothing more;
 * once one is, every write of the backup is parsed before it is written, and the mirror takes the
 * native language from that parse rather than parsing the backup again.
 */
export function ownerArea(
  area: AsyncStorageArea,
  announce: (keys: string[], reason?: StoreChangeReason) => void,
  preferences?: AsyncStorageArea,
  pairs: readonly string[] = SHIPPED_PAIRS,
): OwnerArea {
  // The root value last written, the native language it names when the guard already read it,
  // whether a parse of it is already due, the language last mirrored, and the chain the mirrors run
  // on, one after the other.
  let latest: unknown;
  let latestNative: NativeLanguage | undefined;
  let due = false;
  let mirrored: InterfaceLanguage | undefined;
  let chain: Promise<void> = Promise.resolve();
  // The reader's last choice as the store records it (`LAST_NATIVE_KEY`), null when none is: read at
  // the first write of the backup in this background's life, then kept in step here, its one writer.
  let chosen: { native: NativeLanguage | null } | undefined;
  let reading: Promise<void> | null = null;

  /** Read the record once; a read that fails is tried again by the next write. */
  const readChosen = (): Promise<void> =>
    (reading ??= area.get(LAST_NATIVE_KEY).then(
      (got) => {
        // A choice recorded while the read was under way is newer than what it found.
        chosen ??= { native: recordedNative(got[LAST_NATIVE_KEY], pairs) };
      },
      (e: unknown) => {
        reading = null;
        throw e;
      },
    ));

  /**
   * Write `items` — the backup — with `native` recorded as the reader's last choice, in one write. The
   * record is kept in step before the write is asked, so a backup written after it is checked against
   * it; a write that fails leaves the store as it was, which is then read again.
   */
  const record = async (items: Record<string, unknown>, native: NativeLanguage): Promise<void> => {
    chosen = { native };
    try {
      await area.set({ ...items, [LAST_NATIVE_KEY]: native });
    } catch (e) {
      chosen = undefined;
      reading = null;
      throw e;
    }
  };

  /**
   * Refuse a backup that names another native language than the reader's last choice: the native
   * language it names, when a choice is recorded and it was read for it. Synchronous, and nothing
   * awaits between it and the write it guards: a choice recorded before is checked against, one
   * recorded after is written after.
   */
  const refuseStale = (value: unknown, native: NativeLanguage | null): NativeLanguage | undefined => {
    if (native === null) return undefined;
    const named = nativeLanguageOfStored(value, pairs);
    if (named !== native) throw new StaleNativeLanguageError(named, native);
    return named;
  };

  /**
   * A restore from a file: the native language it names is recorded as the reader's last choice when
   * it is another than theirs, or when a choice was recorded already; with none, theirs is the one
   * their stored backup names. Says whether it moved.
   */
  const restore = async (items: Record<string, unknown>): Promise<{ named: NativeLanguage; moved: boolean }> => {
    const named = nativeLanguageOfStored(items[ROOT_KEY], pairs);
    while (!chosen) await readChosen();
    const stored = chosen.native ?? nativeLanguageOfStored((await area.get(ROOT_KEY))[ROOT_KEY], pairs);
    while (!chosen) await readChosen();
    const moved = named !== (chosen.native ?? stored);
    if (moved || chosen.native !== null) await record(items, named);
    else await area.set(items);
    return { named, moved };
  };

  /** Write `language` as the interface language's key, unless it is the one last written. */
  const write = async (language: InterfaceLanguage): Promise<void> => {
    if (!preferences || language === mirrored) return;
    try {
      await preferences.set({ [INTERFACE_LANGUAGE_KEY]: language });
      mirrored = language;
    } catch (e) {
      console.warn("[Cymbra Lingua] could not mirror the interface language:", e);
    }
  };
  const mirror = async (): Promise<void> => {
    due = false;
    await write(latestNative ?? nativeLanguageOfStored(latest, pairs));
  };

  return {
    get: (keys) => area.get(keys),
    set: async (items, reason) => {
      if (LAST_NATIVE_KEY in items) throw new Error(`"${LAST_NATIVE_KEY}" is written by the store's owner alone`);
      const root = ROOT_KEY in items;
      // The change of native language to announce, and the native language the backup names when it
      // is known without parsing it again.
      let change: StoreChangeReason | undefined;
      let named: NativeLanguage | undefined;
      if (reason?.type === "native-language" && root) {
        change = reason;
        named = reason.native;
        await record(items, reason.native);
      } else if (reason?.type === "restore" && root) {
        const restored = await restore(items);
        named = restored.named;
        if (restored.moved) change = { type: "native-language", native: restored.named };
      } else {
        if (root) {
          while (!chosen) await readChosen();
          named = refuseStale(items[ROOT_KEY], chosen.native);
        }
        await area.set(items);
      }
      if (preferences && root) {
        latest = items[ROOT_KEY];
        latestNative = named;
        if (change) {
          // Before the announcement, after any mirror already due: the language the change names,
          // which is the one this backup's profile names — nothing to parse.
          const language = change.native;
          chain = chain.then(() => write(language));
          await chain;
        } else if (!due) {
          // A macrotask later: writes that land in the same turn share one parse.
          due = true;
          chain = chain.then(() => new Promise<void>((resolve) => setTimeout(resolve, 0))).then(mirror);
        }
      }
      if (change) announce(Object.keys(items), change);
      else announce(Object.keys(items));
    },
    mirrored: () => chain,
  };
}

export type RuntimeSend = (message: unknown) => Promise<unknown>;

const runtimeSend: RuntimeSend = (message) => chrome.runtime.sendMessage(message);

/** The store as a surface writes it: a write may say it restores a backup from a file. */
export interface ReasonedArea extends AsyncStorageArea {
  set(items: Record<string, unknown>, reason?: RestoreFromFile): Promise<void>;
}

/**
 * Every surface's area: the owner answers. Awaiting the reply is also what keeps a
 * suspended background alive until the write has landed.
 */
export function messagedArea(send: RuntimeSend = runtimeSend): ReasonedArea {
  return {
    async get(keys) {
      const reply = (await send({ type: "store:get", keys } satisfies StoreMessage)) as StoreReply | undefined;
      return reply?.items ?? {};
    },
    async set(items, reason) {
      const message: StoreMessage = reason ? { type: "store:set", items, reason } : { type: "store:set", items };
      await send(message);
    },
  };
}

/**
 * Save a backup the reader restored from a file, saying so (add-lingua-native-language-choice, task
 * 4.5): the owner takes the native language it names as the reader's, where a backup naming another
 * would otherwise be refused. `area` is the store: a surface's messaged area, or the owner's own.
 */
export async function saveRestoredBackup(area: ReasonedArea, backup: string): Promise<void> {
  await area.set({ [ROOT_KEY]: { v: STORAGE_VERSION, backup } }, RESTORE_FROM_FILE);
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
