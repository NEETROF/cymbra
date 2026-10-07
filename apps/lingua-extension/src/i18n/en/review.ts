import type { review as fr } from "../fr/review.ts";

// The review's copy in English — a draft after the French (src/i18n/README.md).

export const review: typeof fr = {
  again: "Again",
  hard: "Hard",
  good: "Good",
  easy: "Easy",
  start: "Review",
  nothingDue: "Nothing to review for now.",
  remaining: {
    one: (n) => `${n} card to review`,
    other: (n) => `${n} cards to review`,
  },
  reveal: "Show the answer",
  sentence: (sentence) => `“${sentence}”`,
  markKnown: "I know it ✓",
  language: "Language",
  backup: "Back up",
  restore: "Restore",
  sourcesAndPrivacy: "Sources & privacy",
  privacy: "Nothing leaves your device: analysis and translations are local.",
  deckCards: {
    one: (n) => `${n} card`,
    other: (n) => `${n} cards`,
  },
  dueCards: {
    one: (n) => `${n} to review`,
    other: (n) => `${n} to review`,
  },
  summarySeparator: " · ",
  restored: "Backup restored.",
  notABackup: "Backup file not recognized.",
  sources: (names) => `Sources: ${names}`,
  sourceSeparator: " · ",
};
