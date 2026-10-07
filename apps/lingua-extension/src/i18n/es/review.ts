import type { review as fr } from "../fr/review.ts";

// The review's copy in Spanish — a draft after the French (src/i18n/README.md). Spanish has
// `many` for a round million; the forms are the plural's.

export const review: typeof fr = {
  again: "Otra vez",
  hard: "Difícil",
  good: "Bien",
  easy: "Fácil",
  start: "Repasar",
  nothingDue: "Nada que repasar por ahora.",
  remaining: {
    one: (n) => `${n} tarjeta por repasar`,
    many: (n) => `${n} tarjetas por repasar`,
    other: (n) => `${n} tarjetas por repasar`,
  },
  reveal: "Mostrar la respuesta",
  sentence: (sentence) => `«${sentence}»`,
  markKnown: "La conozco ✓",
  language: "Idioma",
  backup: "Guardar copia",
  restore: "Restaurar",
  sourcesAndPrivacy: "Fuentes y privacidad",
  privacy: "Nada sale de tu dispositivo: el análisis y las traducciones son locales.",
  deckCards: {
    one: (n) => `${n} tarjeta`,
    many: (n) => `${n} tarjetas`,
    other: (n) => `${n} tarjetas`,
  },
  dueCards: {
    one: (n) => `${n} por repasar`,
    many: (n) => `${n} por repasar`,
    other: (n) => `${n} por repasar`,
  },
  summarySeparator: " · ",
  restored: "Copia restaurada.",
  notABackup: "Archivo de copia no reconocido.",
  sources: (names) => `Fuentes: ${names}`,
  sourceSeparator: " · ",
};
