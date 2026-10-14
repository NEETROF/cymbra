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
  /**
   * « Traduction étendue » in the app (add-lingua-review-translation-fallback D7): the offer to
   * download its own copy of the models, made once, and where the download stands.
   */
  modelOffer: (size: string) => `Traduire aussi dans l'app ? Téléchargement de ${size}`,
  modelDownload: "Télécharger",
  modelLater: "Plus tard",
  modelDownloading: (percent: string) => `Téléchargement des modèles… ${percent} %`,
  modelFailed: "Le téléchargement n'a pas abouti.",
  modelRetry: "Réessayer",
  /** « 25,8 Mo ». */
  megabytes: (amount: string) => `${amount} Mo`,
  /** The app's own session, for « Traduction en ligne » only (D8). */
  signedIn: "Connecté à Cymbra pour la traduction en ligne",
  signOut: "Se déconnecter",
  signIn: "Se connecter pour la traduction en ligne",
  signInNote: "Dans l'app, la traduction en ligne demande de te connecter avec Apple ou Google.",
};
