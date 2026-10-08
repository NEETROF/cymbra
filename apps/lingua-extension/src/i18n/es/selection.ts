import type { selection as fr } from "../fr/selection.ts";

// The selection card's copy in Spanish — a draft after the French (src/i18n/README.md).

export const selection: typeof fr = {
  expressionKind: "Expresión: la tarjeta conservará su frase original.",
  selectionKind: "Selección.",
  inDeck: "En tu mazo: en aprendizaje.",
  rare: (n) => `Rara: más allá de las ${n} palabras más frecuentes.`,
  uncommon: (n) => `Poco frecuente: más allá de las ${n} palabras más frecuentes.`,
  fairlyCommon: (n) => `Bastante corriente: entre las ${n} palabras más frecuentes.`,
  common: (n) => `Corriente: entre las ${n} palabras más frecuentes.`,
  veryCommon: (n) => `Muy corriente: entre las ${n} palabras más frecuentes.`,
  truncated: (text) => `${text}…`,
};
