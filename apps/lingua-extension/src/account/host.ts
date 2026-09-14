import { authErrorOf } from "../state/auth-errors.ts";
import type { Providers } from "../state/oidc.ts";
import type { Session } from "../state/session.ts";
import type { AccountMessage, AccountReply } from "./messages.ts";

// The background's account message handler (add-lingua-account-parity, design D2): the
// only place surfaces reach the session. Kept out of background.ts so the dispatch is
// unit-tested. Replies carry categories, never error strings.

export interface AccountHostDeps {
  session: Pick<
    Session,
    | "state"
    | "signInLocal"
    | "signInWithProvider"
    | "signOut"
    | "signUp"
    | "verifyEmail"
    | "resendVerification"
    | "requestPasswordReset"
    | "resetPassword"
  >;
  providers: () => Providers;
  /** Called after every successful sign-in (schedules a sync). */
  onSignedIn: () => void;
}

export async function handleAccountMessage(msg: AccountMessage, deps: AccountHostDeps): Promise<AccountReply> {
  const { session } = deps;
  const signedIn = (): AccountReply => {
    deps.onSignedIn();
    return { ok: true, state: session.state() };
  };
  try {
    switch (msg.type) {
      case "account:state":
        return { ok: true, state: session.state() };
      case "account:providers":
        return { ok: true, providers: deps.providers() };
      case "account:signInLocal":
        await session.signInLocal(msg.email, msg.password);
        return signedIn();
      case "account:signInGoogle":
      case "account:signInApple": {
        const provider = msg.type === "account:signInApple" ? "apple" : "google";
        const outcome = await session.signInWithProvider(provider);
        return outcome === "cancelled" ? { ok: false, cancelled: true, state: session.state() } : signedIn();
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
    }
  } catch (e) {
    return { ok: false, error: authErrorOf(e) };
  }
}
