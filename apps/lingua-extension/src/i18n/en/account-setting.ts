import type { accountSetting as fr } from "../fr/account-setting.ts";

// The Account block's copy in English — a draft after the French (src/i18n/README.md).

export const accountSetting: typeof fr = {
  invite: "Sign in to sync your deck and your words between devices.",
  google: "Continue with Google",
  apple: "Continue with Apple",
  local: "Email and password",
  signIn: "Sign in",
  forgot: "Forgot your password?",
  signUp: "Create an account",
  signedIn: "Signed in",
  handleMissing: "Choose a username",
  handleCta: "Choose your username to keep this account.",
  handleOpen: "Choose my username",
  connected: "Connected accounts",
  providerHint:
    "Account created with Google or Apple? Here, sign in by email once a password is set in “Connected accounts”, from a browser that offers Google or Apple, or in Cymbra Music. Creating an account with the same address would make a second one.",
  signOut: "Sign out",
  email: "Email",
  password: "Password",
  handle: (handle) => `@${handle}`,
};
