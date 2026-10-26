import type { stats as fr } from "../fr/stats.ts";

// The statistics' copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). Italian guillemets take no inner space and no space before a
// colon; the numbers come formatted (`formatNumber`, « 20.000 »); a count follows the feminine plural
// article « le », which never elides (« le 20.000 parole », D4); a round million takes « di ». The
// estimate's source carries its own contracted « da » (« dal tuo livello », « dalla tua impostazione »).

export const stats: typeof fr = {
  pageTitle: "Statistiche — Cymbra Lingua",
  heading: "Statistiche di apprendimento",
  wordsRead: "Parole lette",
  wordsLearned: "Parole imparate",
  reviews: "Ripassi",
  allDevices: "Tutti i tuoi dispositivi",
  thisDevice: "Questo dispositivo",
  myDecisions: "Le mie decisioni",
  confirmedByReading: "Confermate dalla lettura",
  confirmedByReadingNote:
    "Al di sotto del tuo livello e lette in più giorni diversi: passate a «conosciuta» automaticamente.",
  validatedInReview: "Confermate nel ripasso",
  validatedInReviewNote: "Segnate «la conosco» durante un ripasso.",
  seedTitle: "Rafforza un livello",
  seedNote: "Aggiungi parole di un livello al tuo mazzo di ripasso, senza aspettare di incontrarle leggendo.",
  level: "Livello",
  wordCount: "Numero di parole",
  order: "Ordine",
  commonFirst: "prima le comuni",
  rareFirst: "prima le rare",
  addToDeck: "Aggiungi al mazzo",
  language: "Lingua",
  loading: "…",
  days: (n) => `${n} g`,
  agentNote: "Le sessioni dell'agente IA (plugin di Claude Code) non sono conteggiate qui.",
  noLevels: (scale) => `Livelli ${scale} non disponibili per questa lingua (pacchetto senza dati ${scale}).`,
  cardsAdded: (level) => ({
    one: (n) => `${n} carta aggiunta al mazzo (livello ${level}).`,
    many: (n) => `${n} di carte aggiunte al mazzo (livello ${level}).`,
    other: (n) => `${n} carte aggiunte al mazzo (livello ${level}).`,
  }),
  noCardsAdded: "Nessuna carta aggiunta: queste parole sono già seguite, nel tuo mazzo o senza traduzione.",
  known: "conosciuta",
  ignored: "ignorata",
  relearn: "Da imparare di nuovo",
  markedTitle: "Parole segnate",
  noMarked: "Nessuna parola segnata «conosciuta» o «ignorata» per ora.",
  markedNote:
    "Segnate «conosciuta» o «ignorata» (e quindi non più evidenziate). Rimettine una «da imparare» perché venga segnalata di nuovo. " +
    "Durante la lettura: Alt/Opzione + clic (o pressione prolungata sullo schermo touch) su una parola per riaprirla.",

  vocabularyKnown: "Vocabolario conosciuto",
  vocabularyEstimated: "Vocabolario stimato",
  noEstimateFromLevel:
    "Il tuo livello non presuppone ancora nessuna parola: segna quelle che conosci per avviare la stima.",
  noEstimateYet: (action) => `Ancora nessuna stima: ${action} nelle impostazioni, oppure segna le parole che conosci.`,
  declareYourLevel: "indica il tuo livello",
  setCommonWords: "regola le parole comuni che conosci",
  ofDictionary: (n) => `sulle ${n} parole del dizionario`,
  words: {
    one: (n) => `${n} parola`,
    many: (n) => `${n} di parole`,
    other: (n) => `${n} parole`,
  },
  markedKnownWords: (dictionary) => `Le parole che hai segnato come conosciute, ${dictionary}.`,
  fromDeclaredLevel: "dal tuo livello indicato",
  fromCommonWordsSetting: "dalla tua impostazione delle parole più comuni",
  confirmedCount: {
    one: (n) => ` (di cui ${n} confermata)`,
    other: (n) => ` (di cui ${n} confermate)`,
  },
  approxWords: {
    one: (n) => `≈\u00A0${n} parola`,
    many: (n) => `≈\u00A0${n} di parole`,
    other: (n) => `≈\u00A0${n} parole`,
  },
  estimateNote: (source, dictionary, confirmed) =>
    `Estrapolato ${source} e dalle tue parole segnate, fascia di frequenza per fascia, ${dictionary}${confirmed}.`,
  estimatedLevel: (level) => `livello stimato ${level}`,
  thisLevel: "questo livello",
  common: "comuni",
  taught: "insegnate",
  typical: "stimate",
  fraction: (known, total) => `${known} / ${total}`,
  approx: (n) => `≈\u00A0${n}`,
  noFigure: "–",
  legend: "Confermate (lette / imparate), presunte (al di sotto del tuo livello), da imparare.",
  scopeCommon: "«comuni»: le parole più frequenti fino a questo livello. ",
  scopeTaught: "«insegnate»: le parole di base introdotte fino a questo livello dalle liste didattiche. ",
  scopeTypical: (origin) => `«stimate»: il vocabolario che ha in genere chi legge a questo livello, ${origin}`,
  extrapolated: "estrapolato dalle parole dei livelli inferiori su tutto il dizionario.",
};
