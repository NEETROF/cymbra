import type { app as fr } from "../fr/app.ts";

// The Cymbra Lingua app's copy in English — a draft after the French (src/i18n/README.md).

export const app: typeof fr = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Review",
  stats: "Statistics",
  savedAt: (when: string) => `Cards from Safari, ${when}`,
  languageDue: (language: string, due: string) => `${language}: ${due}`,
  dueCards: {
    one: (n) => `${n} due`,
    other: (n) => `${n} due`,
  },
};
