// The Cymbra Lingua app's copy (app.html and app/view.ts), in French — the source module
// (add-lingua-apple-review). The app shows the extension's own Révision and Statistiques; this is
// only what is the app's: its tabs, its loading line and the day of the state it reviews.

export const app = {
  title: "Cymbra Lingua",
  heading: "Cymbra Lingua",
  review: "Révision",
  stats: "Statistiques",
  /** Shown while the engine loads the deck, before the review can show. */
  loading: "Chargement de tes cartes…",
  /** When the extension last saved the deck the app reviews: « Cartes de Safari, 12 oct. à 9:41 ». */
  savedAt: (when: string) => `Cartes de Safari, ${when}`,
};
