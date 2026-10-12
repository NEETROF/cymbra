import type { LinguaPort } from "../analyzer/port.ts";
import {
  applyAppDaily,
  applyAppOps,
  encodeSharedState,
  parseAppOps,
  RELAYED_PREFS,
  RELAYED_STORE_KEYS,
} from "./app-relay.ts";
import type { NativeSend } from "./native-signin.ts";
import { type AsyncStorageArea, loadStored, saveBackup } from "./storage.ts";

// The extension's side of the relay with the Cymbra Lingua app (add-lingua-apple-review D3),
// Safari only: the background hands its state to its native handler, which keeps it in the App
// Group for the app's review, and takes back what a review in the app changed. Pure: the
// background supplies the native send, its store, its preferences and the sync's engine.

/** How long a burst of saves waits before the state is handed over once. */
export const SHARE_DEBOUNCE_MS = 3000;

export interface ReviewRelayDeps {
  send: NativeSend;
  /** The reader's data, as its owner writes it (writes announce themselves). */
  store: AsyncStorageArea;
  /** chrome.storage.local: the relayed preferences. */
  prefs: AsyncStorageArea;
  /** An engine the relay may restore and apply on — the sync's, under `exclusive`. */
  port: LinguaPort;
  /** Run `task` with no sync running on the same engine. */
  exclusive: <T>(task: () => Promise<T>) => Promise<T>;
  /** Epoch millis. */
  now: () => number;
  setTimer?: (run: () => void, ms: number) => unknown;
  clearTimer?: (timer: unknown) => void;
}

export interface ReviewRelay {
  /** Hand the saved state over now. Never fails: an app without the handler keeps what it had. */
  share(): Promise<void>;
  /** The state was saved: hand it over once the burst is over. */
  stateChanged(): void;
  /** Apply what a review in the app changed, if anything waits; resolves how many changes landed. */
  collect(): Promise<number>;
}

export function reviewRelay(deps: ReviewRelayDeps): ReviewRelay {
  const setTimer = deps.setTimer ?? ((run, ms) => setTimeout(run, ms));
  const clearTimer = deps.clearTimer ?? ((timer) => clearTimeout(timer as ReturnType<typeof setTimeout>));
  let pending: unknown = null;
  let collecting: Promise<number> | null = null;

  async function share(): Promise<void> {
    try {
      const stored = await loadStored(deps.store);
      if (stored.kind !== "v2") return;
      const prefs = await deps.prefs.get([...RELAYED_PREFS]);
      const storeItems = await deps.store.get([...RELAYED_STORE_KEYS]);
      await deps.send({
        type: "review.saveState",
        state: encodeSharedState({ backup: stored.backup, savedAt: deps.now(), prefs, storeItems }),
      });
    } catch (e) {
      console.warn("[Cymbra Lingua] could not hand the state to the app:", e);
    }
  }

  async function take(): Promise<number> {
    let reply: { ops?: unknown; writtenAt?: unknown } | null;
    try {
      reply = (await deps.send({ type: "review.readOps" })) as typeof reply;
    } catch {
      return 0; // no handler: nothing waits
    }
    const ops = parseAppOps(reply?.ops);
    const writtenAt = reply?.writtenAt;
    if (typeof writtenAt !== "number") return 0;
    let changed = 0;
    if (ops) {
      changed = await deps.exclusive(async () => {
        // The latest backup first: the ops land on what the reader has now, never on a copy.
        const stored = await loadStored(deps.store);
        if (stored.kind !== "v2") return 0;
        await deps.port.restore(stored.backup);
        const n = await applyAppOps(deps.port, ops);
        if (n > 0) await saveBackup(deps.store, await deps.port.backup());
        return n;
      });
      // The day's reviews and words learned in the app, counted once whatever the payload repeats.
      if (ops.daily) await applyAppDaily(deps.store, ops.daily, deps.now());
    }
    // Dropped only once saved (or unreadable): a failure above leaves them for the next collection.
    await deps.send({ type: "review.clearOps", writtenAt }).catch(() => undefined);
    return changed;
  }

  return {
    share,
    stateChanged() {
      if (pending !== null) clearTimer(pending);
      pending = setTimer(() => {
        pending = null;
        void share();
      }, SHARE_DEBOUNCE_MS);
    },
    collect() {
      collecting ??= take()
        .catch((e: unknown) => {
          console.warn("[Cymbra Lingua] could not apply the app's review:", e);
          return 0;
        })
        .finally(() => {
          collecting = null;
        });
      return collecting;
    },
  };
}
