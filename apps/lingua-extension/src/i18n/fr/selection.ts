// The selection card's copy (reading/selection-card.ts), in French — the source module
// (add-lingua-interface-language). The counts in the frequency bands are written by the surface
// (« 20 000 », as French groups them today).

export const selection = {
  expressionKind: "Expression — la carte gardera sa phrase d’origine.",
  selectionKind: "Sélection.",
  inDeck: "Dans ton deck — en cours d'apprentissage.",
  rare: (n: string) => `Rare — au-delà des ${n} mots les plus fréquents.`,
  uncommon: (n: string) => `Peu fréquent — au-delà des ${n} mots les plus fréquents.`,
  fairlyCommon: (n: string) => `Assez courant — parmi les ${n} mots les plus fréquents.`,
  common: (n: string) => `Courant — parmi les ${n} mots les plus fréquents.`,
  veryCommon: (n: string) => `Très courant — parmi les ${n} mots les plus fréquents.`,
  /** A sentence cut short, ending on a whole word. */
  truncated: (text: string) => `${text}…`,
};
