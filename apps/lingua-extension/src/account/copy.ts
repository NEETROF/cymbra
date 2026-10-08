import { account as enAccount } from "../i18n/en/account.ts";
import { account as esAccount } from "../i18n/es/account.ts";
import { account as frAccount } from "../i18n/fr/account.ts";
import { DEFAULT_INTERFACE_LANGUAGE, formatCount, type InterfaceLanguage } from "../i18n/index.ts";
import type { AuthErrorKind } from "../state/auth-errors.ts";
import { HANDLE_MAX_LENGTH } from "./handle.ts";

// Reader-facing copy for account failures, chosen by flow context × category
// (add-lingua-account-parity, design D7), in the interface language: the catalogue's `account`
// module (localise-lingua-account-onboarding D3). A provider failure is never worded as a password
// error.

/** The account page's copy: the catalogue's `account` module, in the interface language. */
export type AccountCopy = typeof frAccount;

const ACCOUNT_COPY: Record<InterfaceLanguage, AccountCopy> = { fr: frAccount, en: enAccount, es: esAccount };

/** The account page's module for the interface language (French when none is given). */
export function accountCopy(language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE): AccountCopy {
  return ACCOUNT_COPY[language];
}

export type FlowContext =
  | "signInEmail"
  | "signInGoogle"
  | "signInApple"
  | "signUp"
  | "verify"
  | "resend"
  | "forgot"
  | "reset"
  | "handle"
  | "eraseData"
  // Comptes connectés (add-lingua-connected-accounts D6)
  | "connected"
  | "linkGoogle"
  | "linkApple"
  | "unlink"
  | "setPassword"
  | "verifyPassword";

/** Linking Google or Apple: the provider's own words, never a password error. */
function linkCopy(
  provider: string,
  kind: AuthErrorKind,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
): string {
  const c = accountCopy(language);
  if (kind === "alreadyExists") return c.linkAlreadyLinked(provider);
  // A rejected bearer or a rejected id_token: both say so the same way, and retrying settles it.
  if (kind === "unauthenticated") return c.linkUnauthenticated(provider);
  return c.linkFailed(provider);
}

export function errorCopy(
  context: FlowContext,
  kind: AuthErrorKind,
  language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE,
): string {
  const c = accountCopy(language);
  if (kind === "storageFull") return c.storageFull;
  if (kind === "unavailable") return c.unavailable;
  if (kind === "rateLimited") return c.rateLimited;
  switch (context) {
    case "signInEmail":
      if (kind === "unauthenticated") return c.wrongCredentials;
      if (kind === "failedPrecondition") return c.emailNotVerified;
      if (kind === "invalidArgument") return c.checkCredentials;
      return c.generic;
    case "signInGoogle":
      return c.googleFailed;
    case "signInApple":
      return c.appleFailed;
    case "signUp":
      if (kind === "alreadyExists") return c.emailTaken;
      if (kind === "invalidArgument") return c.weakPassword;
      return c.generic;
    case "verify":
      if (kind === "invalidArgument" || kind === "notFound" || kind === "unauthenticated") return c.badCode;
      return c.generic;
    case "reset":
      if (kind === "invalidArgument" || kind === "notFound" || kind === "unauthenticated")
        return c.badCodeOrWeakPassword;
      return c.generic;
    case "handle":
      if (kind === "alreadyExists" || kind === "conflict") return c.handleJustTaken;
      if (kind === "invalidArgument") return c.handleInvalid(formatCount(language, HANDLE_MAX_LENGTH));
      if (kind === "unauthenticated") return c.sessionExpired;
      return c.generic;
    case "eraseData":
      if (kind === "unauthenticated") return c.sessionExpiredErase;
      return c.eraseFailed;
    case "connected":
      if (kind === "unauthenticated") return c.sessionExpired;
      return c.identitiesFailed;
    case "linkGoogle":
      return linkCopy(c.providerGoogle, kind, language);
    case "linkApple":
      return linkCopy(c.providerApple, kind, language);
    case "unlink":
      if (kind === "failedPrecondition") return c.onlyMethod;
      if (kind === "unauthenticated") return c.sessionExpired;
      return c.unlinkFailed;
    case "setPassword":
      if (kind === "invalidArgument") return c.weakPassword;
      if (kind === "alreadyExists") return c.addressTakenOrHasPassword;
      if (kind === "unauthenticated") return c.sessionExpired;
      return c.generic;
    case "verifyPassword":
      if (kind === "alreadyExists") return c.addressJustTaken;
      if (kind === "invalidArgument" || kind === "notFound" || kind === "unauthenticated") return c.badCode;
      return c.generic;
    case "resend":
    case "forgot":
      return c.generic;
  }
}
