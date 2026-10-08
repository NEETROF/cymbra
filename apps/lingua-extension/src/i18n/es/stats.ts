import type { stats as fr } from "../fr/stats.ts";

// The statistics' copy in Spanish — a draft after the French (src/i18n/README.md). Spanish
// guillemets take no inner space and no space before a colon; the numbers come formatted
// (`formatNumber`, the RAE's grouping).

export const stats: typeof fr = {
  pageTitle: "Estadísticas — Cymbra Lingua",
  heading: "Estadísticas de aprendizaje",
  wordsRead: "Palabras leídas",
  wordsLearned: "Palabras aprendidas",
  reviews: "Repasos",
  allDevices: "Todos tus dispositivos",
  thisDevice: "Este dispositivo",
  myDecisions: "Mis decisiones",
  confirmedByReading: "Confirmadas por la lectura",
  confirmedByReadingNote:
    "Por debajo de tu nivel y leídas en varios días distintos: pasadas a «conocidas» automáticamente.",
  validatedInReview: "Validadas en el repaso",
  validatedInReviewNote: "Marcadas «la conozco» durante un repaso.",
  seedTitle: "Reforzar un nivel",
  seedNote: "Añade palabras de un nivel a tu mazo de repaso, sin esperar a encontrarlas leyendo.",
  level: "Nivel",
  wordCount: "Número de palabras",
  order: "Orden",
  commonFirst: "corrientes primero",
  rareFirst: "raras primero",
  addToDeck: "Añadir al mazo",
  language: "Idioma",
  loading: "…",
  days: (n) => `${n} d`,
  agentNote: "Las sesiones de agente de IA (plugin de Claude Code) no se cuentan aquí.",
  noLevels: "Niveles MCER no disponibles para este idioma (el paquete no tiene datos MCER).",
  cardsAdded: (level) => ({
    one: (n) => `${n} tarjeta añadida al mazo (nivel ${level}).`,
    many: (n) => `${n} tarjetas añadidas al mazo (nivel ${level}).`,
    other: (n) => `${n} tarjetas añadidas al mazo (nivel ${level}).`,
  }),
  noCardsAdded: "No se ha añadido ninguna tarjeta: estas palabras ya están seguidas o en tu mazo.",
  known: "conocida",
  ignored: "ignorada",
  relearn: "Volver a aprender",
  markedTitle: "Palabras marcadas",
  noMarked: "Ninguna palabra marcada «conocida» o «ignorada» por ahora.",
  markedNote:
    "Marcadas «conocida» o «ignorada» (y por tanto ya no resaltadas). Vuelve a poner una «por aprender» para que se señale de nuevo. " +
    "Durante la lectura: Alt/Opción + clic (o pulsación larga en pantalla táctil) sobre una palabra para reabrirla.",

  vocabularyKnown: "Vocabulario conocido",
  vocabularyEstimated: "Vocabulario estimado",
  noEstimateFromLevel: "Tu nivel aún no presupone ninguna palabra: marca las que conoces para iniciar la estimación.",
  noEstimateYet: (action) => `Aún no hay estimación: ${action} en los ajustes, o marca palabras que conoces.`,
  declareYourLevel: "declara tu nivel",
  setCommonWords: "ajusta las palabras corrientes que conoces",
  ofDictionary: (n) => `de las ${n} palabras del diccionario`,
  words: {
    one: (n) => `${n} palabra`,
    many: (n) => `${n} palabras`,
    other: (n) => `${n} palabras`,
  },
  markedKnownWords: (dictionary) => `Las palabras que has marcado como conocidas, ${dictionary}.`,
  fromDeclaredLevel: "tu nivel declarado",
  fromCommonWordsSetting: "tu ajuste de las palabras más corrientes",
  confirmedCount: {
    one: (n) => ` (${n} de ellas confirmada)`,
    other: (n) => ` (${n} de ellas confirmadas)`,
  },
  approxWords: {
    one: (n) => `≈\u00A0${n} palabra`,
    other: (n) => `≈\u00A0${n} palabras`,
  },
  estimateNote: (source, dictionary, confirmed) =>
    `Según ${source} y tus palabras marcadas, extrapolado tramo de frecuencia por tramo, ${dictionary}${confirmed}.`,
  estimatedLevel: (level) => `nivel estimado ${level}`,
  thisLevel: "este nivel",
  common: "corrientes",
  taught: "enseñadas",
  typical: "estimadas",
  fraction: (known, total) => `${known} / ${total}`,
  approx: (n) => `≈\u00A0${n}`,
  noFigure: "–",
  legend: "Confirmadas (leídas / aprendidas), supuestas (por debajo de tu nivel), por aprender.",
  scopeCommon: "«corrientes»: las palabras más frecuentes hasta este nivel. ",
  scopeTaught: "«enseñadas»: las palabras básicas introducidas hasta este nivel por las listas de enseñanza. ",
  scopeTypical: (origin) => `«estimadas»: el vocabulario que suele tener un lector de este nivel, ${origin}`,
  extrapolated: "extrapolado de las palabras de los niveles inferiores a todo el diccionario.",
};
