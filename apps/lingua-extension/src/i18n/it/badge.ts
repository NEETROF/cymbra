import type { badge as fr } from "../fr/badge.ts";

// The toolbar icon's title in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). Italian has `many` for a round million, which takes « di »
// (« 1.000.000 di carte »).

export const badge: typeof fr = {
  name: "Cymbra Lingua",
  cards: {
    one: (n) => `Cymbra Lingua — ${n} carta da ripassare`,
    many: (n) => `Cymbra Lingua — ${n} di carte da ripassare`,
    other: (n) => `Cymbra Lingua — ${n} carte da ripassare`,
  },
  perLanguage: (list: string) => `Cymbra Lingua — da ripassare: ${list}`,
  languageCount: (language: string, n: string) => `${language} ${n}`,
  listSeparator: ", ",
};
