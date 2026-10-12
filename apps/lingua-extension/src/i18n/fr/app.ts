import { pluralForms } from "../index.ts";

// The Cymbra Lingua app's copy (app.html and app/view.ts), in French — the source module
// (add-lingua-apple-review). The app shows the extension's own Révision and Statistiques; this is
// only what is the app's: its tabs, the day of the state it reviews, and each language's due count.

export const app = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Révision",
  stats: "Statistiques",
  /** When the extension last saved the deck the app reviews: « Cartes de Safari, 12 oct. à 9:41 ». */
  savedAt: (when: string) => `Cartes de Safari, ${when}`,
  /** One studied language's due count on the home: « Anglais : 3 à revoir ». */
  languageDue: (language: string, due: string) => `${language} : ${due}`,
  dueCards: pluralForms({
    one: (n) => `${n} à revoir`,
    other: (n) => `${n} à revoir`,
  }),
};
