import type { selection as fr } from "../fr/selection.ts";

// The selection card's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). A count follows the feminine plural article « le », which never
// elides (« le 20.000 parole », D4).

export const selection: typeof fr = {
  expressionKind: "Espressione: la carta conserverà la sua frase d'origine.",
  selectionKind: "Selezione.",
  inDeck: "Nel tuo mazzo: in apprendimento.",
  rare: (n) => `Rara: oltre le ${n} parole più frequenti.`,
  uncommon: (n) => `Poco frequente: oltre le ${n} parole più frequenti.`,
  fairlyCommon: (n) => `Abbastanza comune: tra le ${n} parole più frequenti.`,
  common: (n) => `Comune: tra le ${n} parole più frequenti.`,
  veryCommon: (n) => `Molto comune: tra le ${n} parole più frequenti.`,
  truncated: (text) => `${text}…`,
  readAs: (language) => `Parola ${language}`,
};
