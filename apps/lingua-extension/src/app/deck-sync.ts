import type { CardOp, LinguaPort, StatusOp } from "../analyzer/port.ts";
import { accountDigest } from "../state/app-relay.ts";
import type { AsyncStorageArea } from "../state/storage.ts";
import { getOrCreateDeviceId, SYNC_BOOKKEEPING_KEYS, SyncEngine, type SyncClients } from "../sync/sync.ts";

// The Cymbra Lingua app synchronises the reviews made in it (add-lingua-app-deck-sync), with the
// extension's own SyncEngine — one implementation, the app's options (D1): it pushes only what its
// openings changed, the very ops it hands to the extension through the App Group (D3); never the day's
// statistics, which the extension counts and sends once (D9); never a declared level. It is a device of
// its own: its device id and pull cursors live in the page's localStorage, keyed by the account's
// digest (D10). It synchronises only when the extension of the same device is signed in to the same
// account (D7), and respects an erasure made elsewhere since Safari's snapshot (D8).
//
// When (D5): a full exchange at opening (waited at most APP_PULL_WAIT_MS before the review mounts,
// D2) and on return to the foreground — throttled to one per APP_SYNC_THROTTLE_MS —, a push debounced
// APP_PUSH_DEBOUNCE_MS after a save, a push then a full exchange at a session's end, and a push at once
// when the app goes to the background. One exchange at a time; the port is the review's own engine, so
// a full exchange restores and applies on it only while no session is under way.

/** How long the opening waits for the server before the review opens on Safari's snapshot (D2). */
export const APP_PULL_WAIT_MS = 3000;
/** A burst of answers is one push (D5). */
export const APP_PUSH_DEBOUNCE_MS = 5000;
/** At most one full exchange this often, counted from the last successful one (D2). */
export const APP_SYNC_THROTTLE_MS = 10_000;
/** How often a pull waiting for a session's end looks again. */
const IDLE_POLL_MS = 1000;

/** Where the app keeps its synchronisation's bookkeeping, for one account: the page's localStorage (D10). */
export const APP_SYNC_KEY = "cymbra-lingua-app-sync";

/** What the account line says of the synchronisation (D6, D7, D11). */
export type AppSyncLine =
  /** Not signed in inside the app. */
  | "off"
  /** Signed in to the extension's account: the reviews synchronise. */
  | "synced"
  /** Safari's extension is signed in to another account. */
  | "otherAccount"
  /** Safari's extension is signed out. */
  | "safariSignedOut"
  /** Safari's extension predates the account digest. */
  | "olderExtension"
  /** The last exchange failed (offline, server unreachable): the reviews leave later. */
  | "failed"
  /** Signed in, the account not read yet. */
  | "checking";

type LocalStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

export interface AppDeckSyncDeps {
  /** The review's engine. */
  port: LinguaPort;
  /** The opening's store (the backup, the day's counts): its saves write the pending changes. */
  store: AsyncStorageArea;
  /** What the app hands over to the extension now: what it pushes (D3). */
  pendingOps: () => Promise<{ cards: CardOp[]; statuses: StatusOp[] }>;
  /** The extension's account digest as its state names it: null signed out, undefined from an older one. */
  sharedAccount: string | null | undefined;
  /** The erasure mark Safari's snapshot acted on. */
  sharedErasedAt: number;
  /** Drop the pending changes made before an erasure (the opening's `discardPending`, D8). */
  discardPending: (erasedAt: number) => Promise<void>;
  /** Whether the app holds a session of its own. */
  signedIn: () => boolean;
  /** The app's account id, from `UserService.GetAccount` over its session. */
  userId: () => Promise<string>;
  /** The sync clients, over the app's session. */
  clients: () => SyncClients;
  /** The page's localStorage, or null where it cannot be used (the exchange then pulls from the start). */
  local: LocalStorage | null;
  /** A review session is under way: the engine is not restored under it. */
  reviewing: () => boolean;
  /** A pull changed the engine (or an erasure reset it): the review reads it again. */
  onApplied?: () => Promise<void> | void;
  /** The line changed. */
  onChange?: () => void;
  /** Epoch millis. */
  now: () => number;
  setTimer?: (run: () => void, ms: number) => unknown;
  clearTimer?: (timer: unknown) => void;
}

export interface AppDeckSync {
  /** The opening's exchange, waited at most APP_PULL_WAIT_MS; never fails. */
  open(): Promise<void>;
  /** The opening's store saved the backup: push, debounced — at once with a full exchange after a session. */
  saved(): void;
  /** The app goes to the background: push what waits, at once. */
  background(): Promise<void>;
  /** The app's session changed (signed in, out): read the account again, and exchange if it now may. */
  accountChanged(): Promise<void>;
  /** What the account line says. */
  line(): AppSyncLine;
  /** Whether the app synchronises now (signed in to the extension's account). */
  active(): boolean;
}

interface Bookkeeping {
  account: string;
  items: Record<string, unknown>;
  /** When the last full exchange succeeded (epoch millis). */
  syncedAt?: number;
}

function readBook(local: LocalStorage | null, account: string): Bookkeeping {
  try {
    const parsed = JSON.parse(local?.getItem(APP_SYNC_KEY) ?? "null") as Partial<Bookkeeping> | null;
    if (parsed?.account === account && parsed.items !== null && typeof parsed.items === "object") {
      return {
        account,
        items: parsed.items,
        ...(typeof parsed.syncedAt === "number" ? { syncedAt: parsed.syncedAt } : {}),
      };
    }
  } catch {
    /* unreadable: start again */
  }
  // Another account's, or none: dropped (D10).
  return { account, items: {} };
}

function writeBook(local: LocalStorage | null, book: Bookkeeping | null): void {
  try {
    if (book) local?.setItem(APP_SYNC_KEY, JSON.stringify(book));
    else local?.removeItem(APP_SYNC_KEY);
  } catch {
    /* kept for this launch only: losing it costs a full pull */
  }
}

/**
 * The engine's storage in the app: the reader's data in the opening's store, the synchronisation's
 * bookkeeping (device id, cursors, erasure mark, languages, labels mark) in `book`.
 */
export function layeredArea(store: AsyncStorageArea, book: Bookkeeping, save: () => void): AsyncStorageArea {
  const kept = (key: string): boolean => SYNC_BOOKKEEPING_KEYS.includes(key);
  return {
    async get(keys) {
      const list = keys == null ? null : Array.isArray(keys) ? keys : [keys];
      const fromStore = await store.get(list === null ? null : list.filter((k) => !kept(k)));
      const out: Record<string, unknown> = {};
      for (const [key, value] of Object.entries(fromStore)) if (!kept(key)) out[key] = value;
      for (const key of list ?? Object.keys(book.items)) if (kept(key) && key in book.items) out[key] = book.items[key];
      return out;
    },
    async set(items) {
      const toStore: Record<string, unknown> = {};
      let bookChanged = false;
      for (const [key, value] of Object.entries(items)) {
        if (kept(key)) {
          book.items[key] = value;
          bookChanged = true;
        } else toStore[key] = value;
      }
      if (bookChanged) save();
      if (Object.keys(toStore).length > 0) await store.set(toStore);
    },
  };
}

export function appDeckSync(deps: AppDeckSyncDeps): AppDeckSync {
  const setTimer = deps.setTimer ?? ((run, ms) => setTimeout(run, ms));
  const clearTimer = deps.clearTimer ?? ((timer) => clearTimeout(timer as ReturnType<typeof setTimeout>));
  /** The app's account digest, once read; null signed out or before it is known. */
  let digest: string | null = null;
  /** The account could not be read (offline): said as a failure. */
  let unreadable = false;
  let failed = false;
  let book: Bookkeeping | null = null;
  let engine: SyncEngine | null = null;
  /** The erasure mark this opening already acted on: its state postdates it (D8). */
  let actedOn = 0;
  /** A write of the engine's own (a pull applied) is not an answer to push. */
  let ownWrite = false;
  let chain: Promise<unknown> = Promise.resolve();
  let pushTimer: unknown = null;
  /** A session's end is already waiting for its exchange: an answer saves twice (its backup, its count). */
  let ending = false;

  const active = (): boolean => deps.signedIn() && digest !== null && deps.sharedAccount === digest;

  function line(): AppSyncLine {
    if (!deps.signedIn()) return "off";
    if (deps.sharedAccount === undefined) return "olderExtension";
    if (deps.sharedAccount === null) return "safariSignedOut";
    if (unreadable) return "failed";
    if (digest === null) return "checking";
    if (digest !== deps.sharedAccount) return "otherAccount";
    return failed ? "failed" : "synced";
  }

  /** One exchange at a time: a push asked during a pull runs after it. */
  function single<T>(task: () => Promise<T>): Promise<T> {
    const next = chain.then(task, task);
    chain = next.catch(() => undefined);
    return next;
  }

  /** Resolves once no review session is under way. */
  function idle(): Promise<void> {
    if (!deps.reviewing()) return Promise.resolve();
    return new Promise((resolve) => {
      const look = (): void => {
        if (deps.reviewing()) setTimer(look, IDLE_POLL_MS);
        else resolve();
      };
      setTimer(look, IDLE_POLL_MS);
    });
  }

  async function engineFor(account: string): Promise<SyncEngine> {
    if (engine && book?.account === account) return engine;
    const current = readBook(deps.local, account);
    book = current;
    writeBook(deps.local, current);
    const save = (): void => writeBook(deps.local, current);
    const plain = layeredArea(deps.store, current, save);
    // The pulled state the engine writes back is not an answer of the reader's.
    const storage: AsyncStorageArea = {
      get: (keys) => plain.get(keys),
      async set(items) {
        ownWrite = true;
        try {
          await plain.set(items);
        } finally {
          ownWrite = false;
        }
      },
    };
    engine = new SyncEngine({
      port: deps.port,
      // The review's engine is restored only between sessions, from the backup as it is then (D2).
      idle,
      storage,
      clients: deps.clients,
      deviceId: await getOrCreateDeviceId(storage),
      ops: deps.pendingOps,
      pushStats: false,
      pushLevels: false,
      hostErasedAt: () => Math.max(deps.sharedErasedAt, actedOn),
      onErased: async (mark) => {
        actedOn = mark;
        ownWrite = true;
        try {
          await deps.discardPending(mark);
        } finally {
          ownWrite = false;
        }
      },
    });
    return engine;
  }

  /** Read the app's account digest, once per session. */
  async function readAccount(): Promise<void> {
    if (!deps.signedIn()) {
      if (digest !== null || book) writeBook(deps.local, null);
      digest = null;
      book = null;
      engine = null;
      unreadable = false;
      return;
    }
    if (digest !== null) return;
    try {
      digest = await accountDigest(await deps.userId());
      unreadable = false;
    } catch {
      unreadable = true;
    }
  }

  function report(ok: boolean): void {
    if (failed === ok) {
      failed = !ok;
      deps.onChange?.();
    } else failed = !ok;
  }

  async function exchange(full: boolean): Promise<void> {
    if (!active() || digest === null) return;
    try {
      const sync = await engineFor(digest);
      if (!full) {
        await sync.push();
        report(true);
        return;
      }
      const erasedBefore = actedOn;
      const result = await sync.sync();
      if (book) {
        book.syncedAt = deps.now();
        writeBook(deps.local, book);
      }
      report(true);
      if ((result?.pulled ?? 0) > 0 || actedOn !== erasedBefore) await deps.onApplied?.();
    } catch (e: unknown) {
      // Never a raw message (D6): the line says it by category; the pending changes stay in the App
      // Group, and leave at a later opening.
      console.warn(
        "[Cymbra Lingua] the app's synchronisation failed:",
        (e as { code?: unknown } | null)?.code ?? "error",
      );
      report(false);
    }
  }

  function throttled(): boolean {
    if (digest === null) return false;
    const syncedAt = (book?.account === digest ? book : readBook(deps.local, digest)).syncedAt ?? 0;
    return deps.now() - syncedAt < APP_SYNC_THROTTLE_MS;
  }

  async function open(): Promise<void> {
    if (!deps.signedIn()) {
      await readAccount();
      return;
    }
    // The account is read within the wait too: offline, nothing holds the review back.
    const run = single(async () => {
      await readAccount();
      deps.onChange?.();
      if (active() && !throttled()) await exchange(true);
    });
    let timer: unknown = null;
    const waited = new Promise<void>((resolve) => {
      timer = setTimer(resolve, APP_PULL_WAIT_MS);
    });
    // Past the wait the review opens on Safari's snapshot; the exchange applies its answer once no
    // session is under way (the engine's restores wait for it), then the review reads it again.
    await Promise.race([run, waited]);
    clearTimer(timer);
  }

  function flushPush(): Promise<void> {
    if (pushTimer !== null) {
      clearTimer(pushTimer);
      pushTimer = null;
    }
    return single(() => exchange(false));
  }

  return {
    open,
    saved() {
      if (ownWrite || !active()) return;
      if (!deps.reviewing()) {
        // A session ended (its last answer saved with no card left): push, then a full exchange.
        if (ending) return;
        ending = true;
        void flushPush().then(() =>
          single(() => {
            ending = false;
            return exchange(true);
          }),
        );
        return;
      }
      if (pushTimer !== null) clearTimer(pushTimer);
      pushTimer = setTimer(() => {
        pushTimer = null;
        void single(() => exchange(false));
      }, APP_PUSH_DEBOUNCE_MS);
    },
    background: () => (active() ? flushPush() : Promise.resolve()),
    async accountChanged() {
      digest = null;
      await readAccount();
      deps.onChange?.();
      if (active()) await single(() => exchange(true));
    },
    line,
    active,
  };
}
