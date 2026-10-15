import type { badge as fr } from "../fr/badge.ts";

// The toolbar icon's title in English — a draft after the French (src/i18n/README.md).

export const badge: typeof fr = {
  name: "Cymbra Lingua",
  cards: {
    one: (n) => `Cymbra Lingua — ${n} card to review`,
    other: (n) => `Cymbra Lingua — ${n} cards to review`,
  },
  perLanguage: (list: string) => `Cymbra Lingua — to review: ${list}`,
  languageCount: (language: string, n: string) => `${language} ${n}`,
  listSeparator: ", ",
};
