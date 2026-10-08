import { pluralForms } from "../index.ts";

// The review's copy (review/view.ts and review/review-page.ts), in French — the source module
// (add-lingua-interface-language). A count keeps its raw figure and « carte(s) » in every form
// (D2); the number is what the surface renders in bold.

export const review = {
  again: "À revoir",
  hard: "Difficile",
  good: "Correct",
  easy: "Facile",
  start: "Réviser",
  nothingDue: "Rien à réviser pour l'instant.",
  remaining: pluralForms({
    one: (n) => `${n} carte(s) à revoir`,
    other: (n) => `${n} carte(s) à revoir`,
  }),
  reveal: "Afficher la réponse",
  sentence: (sentence: string) => `« ${sentence} »`,
  markKnown: "Je connais ✓",
  language: "Langue",
  backup: "Sauvegarder",
  restore: "Restaurer",
  sourcesAndPrivacy: "Sources & confidentialité",
  privacy: "Rien ne quitte votre appareil : l'analyse et les traductions sont locales.",
  /** The summary line: « 12 carte(s) · 3 à revoir », its two counts in bold. */
  deckCards: pluralForms({
    one: (n) => `${n} carte(s)`,
    other: (n) => `${n} carte(s)`,
  }),
  dueCards: pluralForms({
    one: (n) => `${n} à revoir`,
    other: (n) => `${n} à revoir`,
  }),
  summarySeparator: " · ",
  restored: "Sauvegarde restaurée.",
  notABackup: "Fichier de sauvegarde non reconnu.",
  sources: (names: string) => `Sources : ${names}`,
  sourceSeparator: " · ",
};
