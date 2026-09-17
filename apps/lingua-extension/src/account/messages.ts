import type { AuthErrorKind } from "../state/auth-errors.ts";
import type { Provider, Providers } from "../state/oidc.ts";

// The account message protocol between the surfaces (popup, account page, onboarding) and
// the background, which owns the single session (add-lingua-account-parity, design D2).
// Replies carry a category, never an error string (D7).

export type AccountMessage =
  | { type: "account:state" }
  | { type: "account:providers" }
  | { type: "account:signInLocal"; email: string; password: string }
  | { type: "account:signInGoogle" }
  | { type: "account:signInApple" }
  /** Safari: exchange an id_token the host app handed back, if one is pending. */
  | { type: "account:collectHandedToken" }
  | { type: "account:signOut" }
  | { type: "account:signUp"; email: string; password: string; locale: string }
  | { type: "account:verifyEmail"; code: string }
  | { type: "account:resendVerification"; email: string; locale: string }
  | { type: "account:requestPasswordReset"; email: string; locale: string }
  | { type: "account:resetPassword"; code: string; newPassword: string }
  | { type: "account:profile" }
  | { type: "account:checkHandle"; handle: string }
  | { type: "account:setHandle"; handle: string }
  | { type: "account:abandon" }
  /** « Effacer mes données Lingua »: the server and this device; the account stays. */
  | { type: "account:eraseLinguaData" };

export interface AccountState {
  signedIn: boolean;
}

export interface AccountReply {
  ok: boolean;
  /** Set on failure (`ok: false`). */
  error?: AuthErrorKind;
  /** A provider flow the reader closed: not a failure, nothing to show. */
  cancelled?: boolean;
  /** Safari: the sign-in went on in the host app; the token is collected on return. */
  handedOff?: boolean;
  /** On account:collectHandedToken, the provider of the token that was collected. */
  provider?: Provider;
  state?: AccountState;
  providers?: Providers;
  /** The account's handle (`null` = none yet), on account:profile and account:setHandle. */
  handle?: string | null;
  /** On account:checkHandle. */
  available?: boolean;
}

/** The pending verification email (never the password), in chrome.storage.session. */
export const PENDING_EMAIL_KEY = "cymbra-lingua-pending-verify";

export function isAccountMessage(m: unknown): m is AccountMessage {
  const type = (m as { type?: unknown } | null)?.type;
  return typeof type === "string" && type.startsWith("account:");
}
