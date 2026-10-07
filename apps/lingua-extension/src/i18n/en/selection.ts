import type { selection as fr } from "../fr/selection.ts";

// The selection card's copy in English — a draft after the French (src/i18n/README.md).

export const selection: typeof fr = {
  expressionKind: "Expression — the card will keep its original sentence.",
  selectionKind: "Selection.",
  inDeck: "In your deck — being learned.",
  rare: (n) => `Rare — beyond the ${n} most frequent words.`,
  uncommon: (n) => `Uncommon — beyond the ${n} most frequent words.`,
  fairlyCommon: (n) => `Fairly common — among the ${n} most frequent words.`,
  common: (n) => `Common — among the ${n} most frequent words.`,
  veryCommon: (n) => `Very common — among the ${n} most frequent words.`,
  truncated: (text) => `${text}…`,
};
