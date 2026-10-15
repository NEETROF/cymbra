import type { badge as fr } from "../fr/badge.ts";

// The toolbar icon's title in Spanish — a draft after the French (src/i18n/README.md). Spanish has
// `many` for a round million; the forms are the plural's.

export const badge: typeof fr = {
  name: "Cymbra Lingua",
  cards: {
    one: (n) => `Cymbra Lingua — ${n} tarjeta por repasar`,
    many: (n) => `Cymbra Lingua — ${n} tarjetas por repasar`,
    other: (n) => `Cymbra Lingua — ${n} tarjetas por repasar`,
  },
  perLanguage: (list: string) => `Cymbra Lingua — por repasar: ${list}`,
  languageCount: (language: string, n: string) => `${language} ${n}`,
  listSeparator: ", ",
};
