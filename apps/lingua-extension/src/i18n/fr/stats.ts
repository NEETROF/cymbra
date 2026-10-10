import { pluralForms } from "../index.ts";

// The statistics' copy (stats/view.ts, stats/ladder.ts and stats.html), in French — the source
// module (add-lingua-interface-language). The ladder's no-break spaces are written as the surface
// writes them, as escapes: U+00A0 before a colon, U+202F inside the guillemets (M23).

export const stats = {
  pageTitle: "Statistiques — Cymbra Lingua",
  heading: "Statistiques d'apprentissage",
  wordsRead: "Mots lus",
  wordsLearned: "Mots appris",
  reviews: "Révisions",
  allDevices: "Tous tes appareils",
  thisDevice: "Cet appareil",
  myDecisions: "Mes décisions",
  confirmedByReading: "Confirmés par la lecture",
  confirmedByReadingNote: "Sous ton niveau et lus plusieurs jours différents : passés « connu » automatiquement.",
  validatedInReview: "Validés en révision",
  validatedInReviewNote: "Marqués « je connais » pendant une révision.",
  seedTitle: "Renforcer un niveau",
  seedNote: "Ajoute des mots d'un niveau à ton deck de révision, sans attendre de les croiser en lisant.",
  level: "Niveau",
  wordCount: "Nombre de mots",
  order: "Ordre",
  commonFirst: "courants d'abord",
  rareFirst: "rares d'abord",
  addToDeck: "Ajouter au deck",
  language: "Langue",
  /** The scope before it is known. */
  loading: "…",
  /** A range button: « 30 j ». */
  days: (n: string) => `${n} j`,
  agentNote: "Les sessions d'agent IA (plugin Claude Code) ne sont pas comptées ici.",
  /** `scale` is `languages.levelScale`, « CEFR » (add-lingua-native-language-labels D3). */
  noLevels: (scale: string) => `Niveaux ${scale} indisponibles pour cette langue (pack sans données ${scale}).`,
  /** « 3 cartes ajoutées au deck (niveau B1). » — the forms for one level. */
  cardsAdded: (level: string) =>
    pluralForms({
      one: (n) => `${n} carte ajoutée au deck (niveau ${level}).`,
      other: (n) => `${n} cartes ajoutées au deck (niveau ${level}).`,
    }),
  noCardsAdded: "Aucune carte ajoutée — ces mots sont déjà suivis, dans ton deck ou sans traduction.",
  known: "connu",
  ignored: "ignoré",
  relearn: "Remettre à apprendre",
  markedTitle: "Mots marqués",
  noMarked: "Aucun mot marqué « connu » ou « ignoré » pour l'instant.",
  markedNote:
    "Marqués « connu » ou « ignoré » (donc plus surlignés). Remets-en un « à apprendre » pour qu'il soit de nouveau signalé. " +
    "En lecture : Alt/Option-clic (ou appui long sur tactile) sur un mot pour le rouvrir.",

  // — The vocabulary estimate and the ladder (stats/ladder.ts) —
  vocabularyKnown: "Vocabulaire connu",
  vocabularyEstimated: "Vocabulaire estimé",
  noEstimateFromLevel:
    "Ton niveau ne présume encore aucun mot\u00A0: marque ceux que tu connais pour lancer l'estimation.",
  /** `action` is `declareYourLevel` or `setCommonWords`. */
  noEstimateYet: (action: string) =>
    `Pas encore d'estimation\u00A0: ${action} dans les réglages, ou marque des mots que tu connais.`,
  declareYourLevel: "déclare ton niveau",
  setCommonWords: "règle les mots courants que tu connais",
  ofDictionary: (n: string) => `sur les ${n} mots du dictionnaire`,
  /** « 20 000 mots » — the number as the surface groups it. */
  words: pluralForms({
    one: (n) => `${n} mots`,
    other: (n) => `${n} mots`,
  }),
  markedKnownWords: (dictionary: string) => `Les mots que tu as marqués connus, ${dictionary}.`,
  fromDeclaredLevel: "ton niveau déclaré",
  fromCommonWordsSetting: "ton réglage des mots les plus courants",
  /** « (dont 12 345 confirmés) » — the French plural for every count, as the surface always wrote it. */
  confirmedCount: pluralForms({
    one: (n) => ` (dont ${n} confirmés)`,
    other: (n) => ` (dont ${n} confirmés)`,
  }),
  /** « ≈ 16 000 mots » — the French plural for every count, as the surface always wrote it. */
  approxWords: pluralForms({
    one: (n) => `≈\u00A0${n} mots`,
    other: (n) => `≈\u00A0${n} mots`,
  }),
  /** `source` is `fromDeclaredLevel` or `fromCommonWordsSetting`; `confirmed` is `confirmedCount` or nothing. */
  estimateNote: (source: string, dictionary: string, confirmed: string) =>
    `D'après ${source} et tes mots marqués, extrapolé tranche de fréquence par tranche, ${dictionary}${confirmed}.`,
  estimatedLevel: (level: string) => `niveau estimé ${level}`,
  thisLevel: "ce niveau",
  common: "courants",
  taught: "enseignés",
  typical: "estimés",
  fraction: (known: string, total: string) => `${known} / ${total}`,
  approx: (n: string) => `≈\u00A0${n}`,
  noFigure: "–",
  legend: "Confirmés (lus / appris), présumés (sous ton niveau), à apprendre.",
  scopeCommon: "«\u202Fcourants\u202F»\u00A0: les mots les plus fréquents jusqu'à ce niveau. ",
  scopeTaught:
    "«\u202Fenseignés\u202F»\u00A0: les mots de base introduits jusqu'à ce niveau par les listes d'enseignement. ",
  /** `origin` is `extrapolated`, or `languages.borrowedTypicalNote`. */
  scopeTypical: (origin: string) =>
    `«\u202Festimés\u202F»\u00A0: le vocabulaire qu'a en général un lecteur de ce niveau, ${origin}`,
  extrapolated: "extrapolé des mots des niveaux inférieurs sur tout le dictionnaire.",
};
