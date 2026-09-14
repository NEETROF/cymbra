import type { AuthErrorKind } from "../state/auth-errors.ts";
import type { Providers } from "../state/oidc.ts";

// The account message protocol between the surfaces (popup, account page, onboarding) and
// the background, which owns the single session (add-lingua-account-parity, design D2).
// Replies carry a category, never an error string (D7).

export type AccountMessage =
  | { type: "account:state" }
  | { type: "account:providers" }
  | { type: "account:signInLocal"; email: string; password: string }
  | { type: "account:signInGoogle" }
  | { type: "account:signInApple" }
  | { type: "account:signOut" }
  | { type: "account:signUp"; email: string; password: string; locale: string }
  | { type: "account:verifyEmail"; code: string }
  | { type: "account:resendVerification"; email: string; locale: string }
  | { type: "account:requestPasswordReset"; email: string; locale: string }
  | { type: "account:resetPassword"; code: string; newPassword: string };

export interface AccountState {
  signedIn: boolean;
}

export interface AccountReply {
  ok: boolean;
  /** Set on failure (`ok: false`). */
  error?: AuthErrorKind;
  /** A provider flow the reader closed: not a failure, nothing to show. */
  cancelled?: boolean;
  state?: AccountState;
  providers?: Providers;
}

/** The pending verification email (never the password), in chrome.storage.session. */
export const PENDING_EMAIL_KEY = "cymbra-lingua-pending-verify";

export function isAccountMessage(m: unknown): m is AccountMessage {
  const type = (m as { type?: unknown } | null)?.type;
  return typeof type === "string" && type.startsWith("account:");
}
