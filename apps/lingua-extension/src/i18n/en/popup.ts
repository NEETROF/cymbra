import type { popup as fr } from "../fr/popup.ts";

// The toolbar popup's copy in English — a draft after the French (src/i18n/README.md).

export const popup: typeof fr = {
  title: "Cymbra Lingua",
  settings: "Settings",
  back: "Back",
  sessionLost: "Session expired — sign in again to sync. Your words stay on this device.",
  highlightingOn: "Highlighting on",
  highlightingOff: "Highlighting off",
  highlightingOffNote: "Highlighting is off on every page.",
  notAnalysed: "This page hasn't been analyzed yet.",
  analyse: "Analyze this page",
  alwaysHighlight: "Always highlight (every page)",
  noPercent: "—",
  knownOnPage: "of words known on this page",
  knownInChapter: "of words known in this chapter",
  wordsAnalysed: "Words analyzed",
  wordsUnknown: "Unknown words",
  distinctUnknown: "of which distinct words",
  wordsTracked: "Words tracked",
  chooseLevel: "Choose your level",
  levelLine: (title, level) => `${title}: ${level}`,
  edit: "Change",
  deck: "Deck (learning)",
  review: (due) => `Review (${due})`,
  signInToSync: "Sign in to sync",
  signedIn: "Signed in",
  accountAndSync: "Account and sync",
  library: "Library (EPUB books)",
  statistics: "Learning statistics",
  openBookForFigures: "Open a book from your library to see its figures.",
  handleToChoose: "Choose a username",
  syncOn: "Sync on",
  beginner: "Beginner",
  handle: (handle) => `@${handle}`,
  // — The anonymous counts' first-run sentence (add-lingua-usage-counts D7) —
  usageInform:
    "Lingua counts a few steps anonymously — install, first steps, returns — with no identifier of any kind; nothing you read leaves your device. You can turn these counts off in Settings → Data.",
  usageAsk:
    "May Lingua count a few steps anonymously — install, first steps, returns — with no identifier of any kind? Nothing you read leaves your device, and you can change your mind in Settings → Data.",
  usageYes: "Yes, count",
  usageNo: "No, thanks",
  usageYesDone: "Thank you! You can turn these counts off at any time in Settings → Data.",
  usageNoDone: "Noted: nothing will be counted.",
  usagePrivacy: "Privacy policy",
};
