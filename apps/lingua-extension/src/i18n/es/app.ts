import type { app as fr } from "../fr/app.ts";

// The Cymbra Lingua app's copy in Spanish — a draft after the French (src/i18n/README.md).

export const app: typeof fr = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Repaso",
  stats: "Estadísticas",
  savedAt: (when: string) => `Tarjetas de Safari, ${when}`,
  languageDue: (language: string, due: string) => `${language}: ${due}`,
  dueCards: {
    one: (n) => `${n} por repasar`,
    many: (n) => `${n} por repasar`,
    other: (n) => `${n} por repasar`,
  },
};
