// Réglages' copy (reading/settings-view.ts and reading/studied-languages-view.ts), in French — the
// source module (add-lingua-interface-language). The level, colour, display, translation and
// account blocks have modules of their own.

export const settings = {
  /** `windowsLanguage` and `language` come from `languages` (« Anglais (États-Unis) », « Anglais »). */
  installVoiceHelp: (windowsLanguage: string, language: string) =>
    `Pour une voix sur l'appareil, sans changer la langue du système ni du navigateur : sous Windows, Paramètres › Heure et langue › Langue et région › Ajouter une langue › ${windowsLanguage}, sans la définir comme langue d'affichage, avec la synthèse vocale ; sous macOS, Réglages Système › Accessibilité › Contenu énoncé › Voix du système › Gérer les voix › ${language}. Relance ensuite le navigateur.`,
  installVoiceFallback: " Si l'installation échoue, active les voix en ligne ci-dessous.",
  studiedLanguages: "Langues étudiées",
  beginner: "Débutant",
  /** The calibration slider's label, its count in bold. */
  knowCommonest: (n: string) => `Je connais les ${n} mots les plus courants`,
  wordsBelowLevel: (level: string) => `Les mots sous ${level} ne sont plus surlignés.`,
  chooseLevelHint: "Choisis ton niveau — rien n'est présumé connu pour l'instant.",
  beginnerHint: "Débutant — rien n'est présumé connu.",
  barOnPage: "Barre sur la page",
  showPercentPill: "Afficher la pastille de pourcentage",
  pillNote: "Pastille discrète en bas de la page : pourcentage + accès au deck et aux réglages.",
  readAloud: "Lecture à voix haute",
  readingVoice: "Voix de lecture",
  listen: "▶ Écouter",
  stop: "■ Arrêter",
  onDeviceNote: "Voix installées sur cet appareil : le texte lu ne quitte pas l'appareil.",
  useAndroidVoice: "Utiliser la voix d'Android",
  androidNote:
    "Firefox ne peut pas garantir que la voix d'Android reste sur l'appareil : selon le moteur choisi dans les réglages d'Android, le texte lu peut passer par le réseau.",
  infoIcon: "ⓘ",
  useRemoteVoices: "Utiliser les voix en ligne du navigateur",
  remoteNote:
    "En secours : le texte lu est envoyé aux serveurs du fournisseur de la voix (Google, pour Chrome) et quitte l'appareil. Une voix installée le remplace dès qu'elle est là.",
  books: "Livres",
  openLibrary: "Ouvrir la bibliothèque",
  continuousFlow: "Défilement continu (au lieu de pages)",
  booksNote: "Tes livres EPUB sans DRM, lus hors ligne avec le surlignage. Ils restent sur cet appareil.",
  display: "Affichage",
  displayNote:
    "Le texte des livres et toute l'interface : cartes, tiroir, menus. Le thème vaut aussi pour l'interface ; un préréglage e-ink la garde en noir sur blanc.",
  colours: "Couleurs",
  translation: "Traduction",
  shortcuts: "Raccourcis & gestes",
  /** The keys, as the interface names them. */
  keyAlt: "Alt",
  keyShift: "Maj",
  keyOption: "Option",
  keyS: "S",
  keyD: "D",
  keyL: "L",
  keyPlus: "+",
  keyOr: "/",
  /** `keys` is the rendered key combination. */
  shortcutSidePanel: (keys: string) => `${keys} — panneau latéral`,
  shortcutDrawer: (keys: string) => `${keys} — panneau de révision sur la page`,
  shortcutCapture: (keys: string) => `${keys} — capturer la sélection`,
  shortcutReclassify: (keys: string) => `${keys}-clic (ou appui long) sur un mot — le reclasser`,
  configureShortcuts: "Configurer les raccourcis du navigateur",
  account: "Compte",
  sync: "Synchronisation",
  syncNow: "Synchroniser maintenant",
  reset: "Réinitialisation",
  resetNote: "Efface tes données locales. À n'utiliser qu'exceptionnellement.",
  resetButton: "Réinitialiser…",
  resetPartial: "Partielle — statuts + calibration (garde le deck)",
  resetFull: "Complète — tout effacer",
  cancel: "Annuler",
  confirm: "Oui, confirmer",
  restartNote:
    "Tes données sont sur ton compte. Vider cet appareil n'efface rien : la synchronisation les ramène. À utiliser si l'état local semble faux.",
  restartFromServer: "Repartir du serveur",
  eraseNote: "Pour effacer partout et définitivement, utilise « Effacer mes données Lingua » dans ton compte.",
  manageData: "Gérer mes données",
  fullResetWarning: "Effacer statuts, deck de révision et progression ? Action définitive hors sync.",
  tabLanguage: "Langue",
  tabLook: "Apparence",
  tabPages: "Pages & livres",
  tabData: "Données",
  automaticVoice: (name: string) => `Automatique (${name})`,
  automatic: "Automatique",
  otherVoices: "Autres voix",
  syncing: "Synchronisation…",
  restarting: "Reprise depuis le serveur…",
  restarted: "Repris depuis le serveur.",
  partialResetDone: "Statuts et calibration réinitialisés.",
  dataErased: "Données effacées.",
  /** The tab row's accessible name. */
  tabs: "Réglages",

  // — Langues étudiées (reading/studied-languages-view.ts) —
  studiedNote:
    "Chaque page est lue dans celle de tes langues qu'elle contient. La première cochée sert aux réglages et aux statistiques par défaut.",
  severalLanguagesOffer: "Plusieurs langues à la fois : gratuit pour l'instant.",
};
