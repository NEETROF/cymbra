import type { Client } from "@connectrpc/connect";
import type { UserService } from "@/gen/user_pb";
import { AccountError, authErrorOf } from "../state/auth-errors.ts";

// The Cymbra ID account behind the session (add-lingua-account-parity, design D9): its
// handle — every account needs one, or the backend's orphan reaper deletes it — read and set
// over UserService. Failures are categorized like the Session's, so no caller ever sees a
// raw Connect error.

export interface AccountProfile {
  /** `null` until the reader picks one. */
  handle: string | null;
  version: number;
  displayName: string | null;
  preferences: string;
}

export interface AccountPort {
  profile(): Promise<AccountProfile>;
  checkHandle(handle: string): Promise<boolean>;
  /** Set the handle on `current`, sending its other fields back unchanged. */
  setHandle(handle: string, current: AccountProfile): Promise<AccountProfile>;
  deleteAccount(): Promise<void>;
}

interface ProtoAccount {
  handle?: string;
  displayName?: string;
  preferences: string;
  version: bigint;
}

function toProfile(a: ProtoAccount): AccountProfile {
  return {
    handle: a.handle ? a.handle : null,
    version: Number(a.version),
    displayName: a.displayName ?? null,
    preferences: a.preferences,
  };
}

async function categorized<T>(fn: () => Promise<T>): Promise<T> {
  try {
    return await fn();
  } catch (e) {
    throw new AccountError(authErrorOf(e));
  }
}

export function userServicePort(client: () => Client<typeof UserService>): AccountPort {
  return {
    profile: () => categorized(async () => toProfile(await client().getAccount({}))),
    checkHandle: (handle) => categorized(async () => (await client().checkHandleAvailability({ handle })).available),
    setHandle: (handle, current) =>
      categorized(async () =>
        toProfile(
          await client().updateAccount({
            handle,
            expectedVersion: BigInt(current.version),
            // UpdateAccount writes the whole record: send the untouched fields back as read.
            preferences: current.preferences,
            ...(current.displayName != null ? { displayName: current.displayName } : {}),
          }),
        ),
      ),
    deleteAccount: () =>
      categorized(async () => {
        await client().deleteAccount({});
      }),
  };
}
