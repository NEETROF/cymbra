// The toolbar popup's copy (popup.html's static text and popup.ts), in French: the source the
// English and Spanish modules are typed after. Byte for byte what the surface shows today
// (add-lingua-interface-language, M23); no `as const`, so a translation may differ.

export const popup = {
  /** The page's title and heading. */
  title: "Cymbra Lingua",
  settings: "Réglages",
  back: "Retour",
  sessionLost: "Session expirée — reconnecte-toi pour synchroniser. Tes mots restent sur cet appareil.",
  highlightingOn: "Surlignage activé",
  highlightingOff: "Surlignage désactivé",
  highlightingOffNote: "Surlignage désactivé sur toutes les pages.",
  notAnalysed: "Cette page n'est pas encore analysée.",
  analyse: "Analyser cette page",
  alwaysHighlight: "Toujours surligner (toutes les pages)",
  /** The percentage before any figure. */
  noPercent: "—",
  knownOnPage: "de mots connus sur cette page",
  knownInChapter: "de mots connus dans ce chapitre",
  wordsAnalysed: "Mots analysés",
  wordsUnknown: "Mots inconnus",
  distinctUnknown: "dont mots différents",
  wordsTracked: "Mots suivis",
  chooseLevel: "Choisis ton niveau",
  /** « Niveau d'anglais : B1 » — the label from `languages`, the level as declared. */
  levelLine: (title: string, level: string) => `${title} : ${level}`,
  edit: "Modifier",
  deck: "Deck (en cours)",
  review: (due: string) => `Réviser (${due})`,
  signInToSync: "Se connecter pour synchroniser",
  signedIn: "Connecté",
  accountAndSync: "Compte et synchronisation",
  library: "Bibliothèque (livres EPUB)",
  statistics: "Statistiques d'apprentissage",
  openBookForFigures: "Ouvre un livre de ta bibliothèque pour voir ses chiffres.",
  handleToChoose: "Pseudo à choisir",
  syncOn: "Synchronisation activée",
  beginner: "Débutant",
  handle: (handle: string) => `@${handle}`,
};
