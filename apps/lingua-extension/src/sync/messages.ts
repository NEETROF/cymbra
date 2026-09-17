import type { AccountReply } from "../account/messages.ts";
import type { AuthErrorKind } from "../state/auth-errors.ts";
import type { AsyncStorageArea } from "../state/storage.ts";

// The sync message protocol between the surfaces and the background, which owns the session
// and the scheduler (add-lingua-connected-clients §2.4). A plain request asks for a sync when
// a surface opens or a page loads; a forced one is « Synchroniser maintenant ». Replies carry
// a category, never an error string.

export type SyncMessage = { type: "sync:request"; force?: boolean };

export interface SyncReply {
  ok: boolean;
  /** Set on failure (`ok: false`). */
  error?: AuthErrorKind;
}

/** When this device last synced successfully (epoch millis), in chrome.storage.local. */
export const LAST_SYNC_KEY = "cymbra-lingua-last-sync";

export function isSyncMessage(m: unknown): m is SyncMessage {
  return (m as { type?: unknown } | null)?.type === "sync:request";
}

export type RuntimeSend = (message: unknown) => Promise<unknown>;

const runtimeSend: RuntimeSend = (message) => chrome.runtime.sendMessage(message);

/** A surface opened or a page loaded: ask for a sync. Never fails the caller. */
export async function requestSync(send: RuntimeSend = runtimeSend): Promise<void> {
  try {
    await send({ type: "sync:request" } satisfies SyncMessage);
  } catch {
    // The background is unreachable for now; the next trigger asks again.
  }
}

/** « Synchroniser maintenant »: run one exchange and report how it went. */
export async function syncNow(send: RuntimeSend = runtimeSend): Promise<SyncReply> {
  try {
    const reply = (await send({ type: "sync:request", force: true } satisfies SyncMessage)) as SyncReply | undefined;
    return reply ?? { ok: false, error: "unavailable" };
  } catch {
    return { ok: false, error: "unavailable" };
  }
}

/** Whether a Cymbra account is signed in on this device (the sync controls show only then). */
export async function syncAvailable(send: RuntimeSend = runtimeSend): Promise<boolean> {
  try {
    const reply = (await send({ type: "account:state" })) as AccountReply | undefined;
    return Boolean(reply?.state?.signedIn);
  } catch {
    return false;
  }
}

/** The last successful sync of this device, or null if it never synced. */
export async function loadLastSync(area: AsyncStorageArea): Promise<number | null> {
  const value = (await area.get(LAST_SYNC_KEY))[LAST_SYNC_KEY];
  return typeof value === "number" && value > 0 ? value : null;
}
