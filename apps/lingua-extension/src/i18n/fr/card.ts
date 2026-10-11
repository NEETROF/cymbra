// The word card's copy (reading/wordpopup.ts), in French — the source module (add-lingua-interface-language).

export const card = {
  rowsLabel: "Mot à mot — ce n'est pas une traduction de l'expression.",
  translationLabel: "Dans votre phrase — traduction automatique",
  /** « Traduction en ligne » (add-lingua-online-translation): the button, never naming the service. */
  onlineTranslate: "Traduction en ligne",
  onlineTranslating: "Traduction en ligne…",
  /** The label of a translation the online service gave. */
  onlineTranslationLabel: "Dans votre phrase — traduction automatique en ligne",
  onlineUnavailable: "Traduction en ligne indisponible pour le moment.",
  /** `time` is the hour the limit resets, as the interface language writes it (« 14:05 »). */
  onlineLimit: (time: string) => `Limite quotidienne atteinte — de nouveau disponible à ${time}.`,
  waiting: "Recherche dans le pack…",
  translating: "Traduction en cours…",
  noGloss: "Pas de traduction dans le pack.",
  noGlossExpression: "Pas de traduction dans le pack pour cette expression.",
  previousPageIcon: "‹",
  previousPage: "Sens précédents",
  nextPageIcon: "›",
  nextPage: "Sens suivants",
  stop: "■ Arrêter",
  stopLabel: "Arrêter la lecture",
  listenSelection: "▶ Sélection",
  listenSelectionLabel: "Écouter la sélection",
  /** The button that reads one form, named by it. */
  listenForm: (form: string) => `▶ ${form}`,
  listenSeenFormLabel: (form: string) => `Écouter la forme vue « ${form} »`,
  listenDictionaryFormLabel: (form: string) => `Écouter la forme du dictionnaire « ${form} »`,
  listenWord: "▶ Mot",
  listenWordLabel: "Écouter le mot",
  listenSentence: "▶ Phrase",
  listenSentenceLabel: "Écouter la phrase",
  /** Between the senses of one group. */
  senseSeparator: "; ",
  /** One word-for-word row: the form, then its gloss. */
  row: (form: string, gloss: string) => `${form} → ${gloss}`,
  seenForm: (form: string) => `forme vue : « ${form} »`,
  known: "Je connais",
  addToDeck: "+ Deck",
  ignore: "Ignorer",
  relearn: "Remettre à apprendre",
  closeIcon: "✕",
  close: "Fermer",
};
