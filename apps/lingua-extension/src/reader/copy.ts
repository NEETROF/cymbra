import type { ImportFailure } from "./library.ts";

// Every sentence the reader page shows, in French like the rest of the extension. A failure
// is one plain sentence saying what happened — never a raw error.

export const COPY = {
  title: "Bibliothèque",
  importButton: "Importer un livre (EPUB)",
  empty: "Ta bibliothèque est vide. Importe un livre EPUB sans DRM : il reste sur cet appareil et s'ouvre hors ligne.",
  importing: "Import en cours…",
  imported: (title: string) => `« ${title} » est dans ta bibliothèque.`,
  alreadyThere: (title: string) => `« ${title} » était déjà dans ta bibliothèque.`,
  importFailed: {
    protected: "Ce livre est protégé par un DRM : Cymbra Lingua ne peut pas l'ouvrir.",
    notEpub: "Ce fichier n'est pas un livre EPUB.",
    unreadable: "Ce fichier ne peut pas être lu : il est peut-être incomplet ou abîmé.",
    storage: "Il n'y a plus assez de place sur cet appareil pour garder ce livre.",
  } satisfies Record<ImportFailure, string>,
  persistenceRefused:
    "Ton navigateur ne s'est pas engagé à garder tes livres : s'il manque de place, il pourrait les effacer. Tu pourras toujours les réimporter.",
  open: (title: string) => `Ouvrir « ${title} »`,
  remove: "Supprimer",
  removeConfirm: (title: string) => `Supprimer « ${title} » ? Les cartes que tu en as tirées restent dans ton deck.`,
  removeYes: "Oui, supprimer",
  cancel: "Annuler",
  missing: "Ce livre n'est plus dans ta bibliothèque.",
  openFailed: "Ce livre n'a pas pu être ouvert.",
  back: "Bibliothèque",
  toc: "Sommaire",
  display: "Aa",
  displayTitle: "Taille du texte et page",
  noToc: "Ce livre n'a pas de sommaire.",
  review: "Réviser",
  stats: "Stats",
  settings: "Réglages",
  prev: "Page précédente",
  next: "Page suivante",
  percentTitle: "Mots connus dans ce chapitre",
};
