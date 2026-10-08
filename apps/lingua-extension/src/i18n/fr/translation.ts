// The translation setting's copy (reading/translation-setting.ts), in French — the source module
// (add-lingua-interface-language). A size is written by the surface (« 25,8 », `formatNumber` with
// one decimal) and takes its unit here, with the plain space the surface writes today (D4).

export const translation = {
  toggle: "Traduction étendue",
  attribution: "Modèle de traduction : Firefox Translations (Mozilla), licence MPL 2.0.",
  ready: "Prête : tes sélections sont traduites sur cet appareil.",
  interrupted: "Téléchargement interrompu.",
  removed: "Le navigateur a supprimé le modèle de cet appareil. Il faut le télécharger à nouveau.",
  missing: "Il manque un modèle pour une de tes langues.",
  cancel: "Annuler",
  download: "Télécharger",
  retry: "Réessayer",
  resume: "Reprendre",
  again: "Télécharger à nouveau",
  /** « 25,8 Mo ». */
  megabytes: (amount: string) => `${amount} Mo`,
  /** What the cost sentence names when the catalogue gives no size. */
  theModel: "le modèle",
  memoryPivot: "environ 340 Mo",
  memorySingle: "environ 200 Mo",
  /** `download` is a size or `theModel`; `memory` is `memoryPivot` or `memorySingle`. */
  cost: (download: string, memory: string) =>
    `Traduit tes phrases sur cet appareil, sans rien envoyer. Télécharge ${download} une fois, puis utilise ` +
    `${memory} de mémoire pendant la traduction. Réglage propre à cet appareil.`,
  failedNetwork: "Le téléchargement a échoué : pas de connexion. Réessaie une fois en ligne.",
  failedUnavailable: "Le téléchargement a échoué : le serveur ne répond pas. Réessaie plus tard.",
  failedNotTheModel: "Le téléchargement a échoué : le fichier reçu n'est pas le bon modèle. Réessaie plus tard.",
  failedStorageSized: (size: string) => `Pas assez de place sur cet appareil pour le modèle (${size}).`,
  failedStorage: "Pas assez de place sur cet appareil pour le modèle.",
  failedUnknown: "Le téléchargement a échoué. Réessaie plus tard.",
  /** « 12,3 Mo sur 25,8 Mo », added after a state's sentence. */
  progress: (received: string, total: string) => ` ${received} sur ${total}`,
  /** `progress` is `progress`, or nothing while the total is unknown. */
  downloading: (progress: string) => `Téléchargement du modèle…${progress}`,
  missingSized: (size: string) => `Il manque un modèle pour une de tes langues. ${size} à télécharger.`,
};
