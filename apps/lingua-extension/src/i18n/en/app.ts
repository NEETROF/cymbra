import type { app as fr } from "../fr/app.ts";

// The Cymbra Lingua app's copy in English — a draft after the French (src/i18n/README.md).

export const app: typeof fr = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Review",
  stats: "Statistics",
  loading: "Loading your cards…",
  savedAt: (when: string) => `Cards from Safari, ${when}`,
  modelOffer: (size: string) => `Translate in the app too? A ${size} download`,
  modelDownload: "Download",
  modelLater: "Not now",
  modelDownloading: (percent: string) => `Downloading the models… ${percent}%`,
  modelFailed: "The download did not go through.",
  modelRetry: "Try again",
  megabytes: (amount: string) => `${amount} MB`,
  signedIn: "Signed in to Cymbra for online translation",
  signedInAs: (account: string) => `Signed in as ${account} for online translation`,
  signOut: "Sign out",
  signIn: "Sign in for online translation",
  emailCancel: "Cancel",
  syncOffer: "Sign in to sync your reviews and use online translation.",
  syncSignIn: "Sign in",
  syncLater: "Not now",
  syncOn: "Signed in to Cymbra: your reviews sync",
  syncOnAs: (account: string) => `Signed in as ${account}: your reviews sync`,
  syncOtherAccount: "Safari is signed in to another account: your reviews will go through Safari.",
  syncSafariSignedOut: "Sign Safari in to this account too to sync from the app.",
  syncOlderExtension: "Update the Safari extension to sync from the app.",
  syncFailed: "Sync isn't possible right now: your reviews will go later.",
};
