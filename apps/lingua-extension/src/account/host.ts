import { authErrorOf } from "../state/auth-errors.ts";
import type { HandedIdToken } from "../state/native-signin.ts";
import type { Provider, Providers } from "../state/oidc.ts";
import type { Session } from "../state/session.ts";
import type { AccountMessage, AccountReply } from "./messages.ts";
import type { AccountPort, AccountProfile } from "./profile.ts";

// The background's account message handler (add-lingua-account-parity, design D2): the
// only place surfaces reach the session and the Cymbra ID account. Kept out of
// background.ts so the dispatch is unit-tested. Replies carry categories, never error
// strings.

export interface AccountHostDeps {
  session: Pick<
    Session,
    | "state"
    | "signInLocal"
    | "signInWithProvider"
    | "signInWithIdToken"
    | "signOut"
    | "signUp"
    | "verifyEmail"
    | "resendVerification"
    | "requestPasswordReset"
    | "resetPassword"
  >;
  /** The signed-in account's profile (handle) over UserService. */
  account: AccountPort;
  /** Synchronous from the browser's APIs; asynchronous on Safari, where the host app answers. */
  providers: () => Providers | Promise<Providers>;
  /**
   * Safari only (add-lingua-connected-clients D6): Apple and Google run in the host app,
   * which hands the id_token back through the native handler. Absent elsewhere.
   */
  handOff?: {
    /** Open the host app on the provider's sheet. */
    open: (provider: Provider) => Promise<void>;
    /** The pending id_token, returned once; null when none is waiting. */
    take: () => Promise<HandedIdToken | null>;
  };
  /**
   * Erase the reader's Lingua data on the server and on this device
   * (add-lingua-privacy-controls), serialized with the sync loop. Throws on failure,
   * before anything local is touched.
   */
  eraseLinguaData: () => Promise<void>;
  /** Called after every successful sign-in (schedules a sync). */
  onSignedIn: () => void;
}

export async function handleAccountMessage(msg: AccountMessage, deps: AccountHostDeps): Promise<AccountReply> {
  const { session, account } = deps;
  const signedIn = (): AccountReply => {
    deps.onSignedIn();
    return { ok: true, state: session.state() };
  };
  try {
    switch (msg.type) {
      case "account:state":
        return { ok: true, state: session.state() };
      case "account:providers":
        return { ok: true, providers: await deps.providers() };
      case "account:signInLocal":
        await session.signInLocal(msg.email, msg.password);
        return signedIn();
      case "account:signInGoogle":
      case "account:signInApple": {
        const provider = msg.type === "account:signInApple" ? "apple" : "google";
        if (deps.handOff) {
          await deps.handOff.open(provider);
          return { ok: false, handedOff: true, state: session.state() };
        }
        const outcome = await session.signInWithProvider(provider);
        return outcome === "cancelled" ? { ok: false, cancelled: true, state: session.state() } : signedIn();
      }
      case "account:collectHandedToken": {
        const handed = deps.handOff ? await deps.handOff.take() : null;
        if (!handed) return { ok: true, state: session.state() };
        try {
          await session.signInWithIdToken(handed.provider, handed.idToken);
        } catch (e) {
          return { ok: false, error: authErrorOf(e), provider: handed.provider };
        }
        return { ...signedIn(), provider: handed.provider };
      }
      case "account:signOut":
        await session.signOut();
        return { ok: true, state: session.state() };
      case "account:signUp":
        await session.signUp(msg.email, msg.password, msg.locale);
        return { ok: true };
      case "account:verifyEmail":
        await session.verifyEmail(msg.code);
        return { ok: true };
      case "account:resendVerification":
        await session.resendVerification(msg.email, msg.locale);
        return { ok: true };
      case "account:requestPasswordReset":
        await session.requestPasswordReset(msg.email, msg.locale);
        return { ok: true };
      case "account:resetPassword":
        await session.resetPassword(msg.code, msg.newPassword);
        return { ok: true };
      case "account:profile": {
        if (!session.state().signedIn) return { ok: false, error: "unauthenticated" };
        const profile = await account.profile();
        return { ok: true, state: session.state(), handle: profile.handle };
      }
      case "account:checkHandle":
        return { ok: true, available: await account.checkHandle(msg.handle) };
      case "account:setHandle": {
        // Read the account fresh so the write carries the current version and fields.
        const updated = await account.setHandle(msg.handle, await account.profile());
        return { ok: true, state: session.state(), handle: updated.handle };
      }
      case "account:eraseLinguaData": {
        if (!session.state().signedIn) return { ok: false, error: "unauthenticated" };
        await deps.eraseLinguaData();
        return { ok: true, state: session.state() };
      }
      case "account:abandon": {
        // Music's rule (handle-onboarding): leaving the handle step deletes a handle-less
        // account — the reaper would delete it anyway — and only signs out one that has a
        // handle. A profile that cannot be read counts as "has one": never delete on a guess.
        let current: AccountProfile | null = null;
        try {
          current = await account.profile();
        } catch {
          current = null;
        }
        if (current && current.handle == null) {
          try {
            await account.deleteAccount();
          } catch {
            // Best effort: the backend's orphan reaper purges it later.
          }
        }
        await session.signOut();
        return { ok: true, state: session.state() };
      }
    }
  } catch (e) {
    return { ok: false, error: authErrorOf(e) };
  }
}
