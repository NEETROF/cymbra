import type { review as fr } from "../fr/review.ts";

// The review's copy in Spanish — a draft after the French (src/i18n/README.md). Spanish has
// `many` for a round million; the forms are the plural's.

export const review: typeof fr = {
  start: "Repasar",
  nothingDue: "Nada que repasar por ahora.",
  remaining: {
    one: (n) => `${n} tarjeta por repasar`,
    many: (n) => `${n} tarjetas por repasar`,
    other: (n) => `${n} tarjetas por repasar`,
  },
  reveal: "Mostrar la respuesta",
  missed: "No la sabía",
  recalled: "La sabía",
  markKnown: "La conozco",
  hide: "No volver a mostrarla",
  noGloss: "No hay traducción para esta palabra.",
  noGlossExpression: "No hay traducción para esta expresión.",
  translationLabel: "En tu frase: traducción automática",
  translationAloneLabel: "Traducción automática",
  translating: "Traduciendo…",
  sessionDone: "Sesión terminada",
  reviewed: {
    one: (n) => `${n} palabra repasada`,
    many: (n) => `${n} palabras repasadas`,
    other: (n) => `${n} palabras repasadas`,
  },
  recovered: {
    one: (n) => `${n} recuperada por el camino`,
    many: (n) => `${n} recuperadas por el camino`,
    other: (n) => `${n} recuperadas por el camino`,
  },
  holding: {
    one: (n) => `${n} se mantiene ya más de un mes`,
    many: (n) => `${n} se mantienen ya más de un mes`,
    other: (n) => `${n} se mantienen ya más de un mes`,
  },
  known: {
    one: (n) => `${n} marcada como conocida`,
    many: (n) => `${n} marcadas como conocidas`,
    other: (n) => `${n} marcadas como conocidas`,
  },
  hidden: {
    one: (n) => `${n} oculta`,
    many: (n) => `${n} ocultas`,
    other: (n) => `${n} ocultas`,
  },
  more: (n) => `${n} más`,
  listenWord: "▶ Palabra",
  listenWordLabel: "Escuchar la palabra",
  listenExpression: "▶ Expresión",
  listenExpressionLabel: "Escuchar la expresión",
  listenSentence: "▶ Frase",
  listenSentenceLabel: "Escuchar la frase",
  listenForm: (form) => `▶ ${form}`,
  listenDictionaryFormLabel: (form) => `Escuchar la forma del diccionario «${form}»`,
  stop: "■ Detener",
  stopLabel: "Detener la lectura",
  language: "Idioma",
  deckCards: {
    one: (n) => `${n} tarjeta`,
    many: (n) => `${n} tarjetas`,
    other: (n) => `${n} tarjetas`,
  },
  dueCards: {
    one: (n) => `${n} por repasar`,
    many: (n) => `${n} por repasar`,
    other: (n) => `${n} por repasar`,
  },
  summarySeparator: " · ",
};
