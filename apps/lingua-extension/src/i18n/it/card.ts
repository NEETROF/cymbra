import type { card as fr } from "../fr/card.ts";

// The word card's copy in Italian — a draft after the French (src/i18n/README.md;
// add-lingua-italian-interface D1). The hour the online limit resets follows « alle ore », whose
// article agrees with « ore » and never elides (D4).

export const card: typeof fr = {
  rowsLabel: "Parola per parola: non è una traduzione dell'espressione.",
  translationLabel: "Nella tua frase: traduzione automatica",
  onlineTranslate: "Traduzione online",
  onlineTranslating: "Traduzione online…",
  onlineTranslationLabel: "Nella tua frase: traduzione automatica online",
  onlineUnavailable: "Traduzione online non disponibile al momento.",
  onlineLimit: (time: string) => `Limite giornaliero raggiunto: di nuovo disponibile alle ore ${time}.`,
  waiting: "Ricerca nel pacchetto…",
  translating: "Traduzione in corso…",
  noGloss: "Nessuna traduzione nel pacchetto.",
  noGlossExpression: "Nessuna traduzione nel pacchetto per questa espressione.",
  previousPageIcon: "‹",
  previousPage: "Significati precedenti",
  nextPageIcon: "›",
  nextPage: "Significati successivi",
  stop: "■ Ferma",
  stopLabel: "Ferma la lettura",
  listenSelection: "▶ Selezione",
  listenSelectionLabel: "Ascolta la selezione",
  listenForm: (form) => `▶ ${form}`,
  listenSeenFormLabel: (form) => `Ascolta la forma vista «${form}»`,
  listenDictionaryFormLabel: (form) => `Ascolta la forma del dizionario «${form}»`,
  listenWord: "▶ Parola",
  listenWordLabel: "Ascolta la parola",
  listenSentence: "▶ Frase",
  listenSentenceLabel: "Ascolta la frase",
  senseSeparator: "; ",
  row: (form, gloss) => `${form} → ${gloss}`,
  seenForm: (form) => `forma vista: «${form}»`,
  known: "La conosco",
  addToDeck: "+ Mazzo",
  ignore: "Ignora",
  relearn: "Rimetti da imparare",
  closeIcon: "✕",
  close: "Chiudi",
};
