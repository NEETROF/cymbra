import type { accountErrors as fr } from "../fr/account-errors.ts";

// The account's errors in plain words in English — a draft after the French (src/i18n/README.md).

export const accountErrors: typeof fr = {
  unavailable: "Cymbra can't be reached. Check your connection and try again.",
  rateLimited: "Too many attempts. Try again in a few minutes.",
  generic: "Something went wrong. Try again.",
  storageFull: "The extension's storage is full on this device. Reset your local data in Settings, then try again.",
  badCode: "Invalid or expired code. Request a new code.",
  sessionExpired: "Your session has expired. Sign in again.",
  linkAlreadyLinked: (provider) => `This ${provider} account is already linked to another Cymbra account.`,
  linkUnauthenticated: (provider) =>
    `Linking ${provider} didn't go through. Try again — if it keeps failing, sign in again.`,
  linkFailed: (provider) => `${provider} couldn't be linked. Try again.`,
  wrongCredentials: "Incorrect email or password.",
  emailNotVerified: "Your email address isn't verified yet.",
  checkCredentials: "Check the email and password you entered.",
  googleFailed: "Signing in with Google failed. Try again.",
  appleFailed: "Signing in with Apple failed. Try again.",
  emailTaken: "An account already uses this email.",
  weakPassword: "Invalid email or password too weak: choose a longer one.",
  badCodeOrWeakPassword: "Invalid or expired code, or password too weak.",
  handleJustTaken: "This username was just taken — choose another.",
  sessionExpiredErase: "Your session has expired. Sign in again to erase your data.",
  eraseFailed: "Erasing failed. Your data is intact: try again.",
  identitiesFailed: "Your sign-in methods couldn't be loaded. Try again.",
  onlyMethod: "You can't remove your only sign-in method.",
  unlinkFailed: "This method couldn't be removed. Try again.",
  addressTakenOrHasPassword:
    "This address is already used by a Cymbra account, or your account already has a password.",
  addressJustTaken: "This address was just taken by another account. Start over with another one.",
  providerGoogle: "Google",
  providerApple: "Apple",
  handleInvalid: (max) => `1 to ${max} letters or digits only (no spaces or symbols).`,
};
