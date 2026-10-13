import { pluralForms } from "../index.ts";

// The review's copy (review/view.ts and review/review-page.ts), in French — the source module
// (add-lingua-interface-language). A count keeps its raw figure and « carte(s) » in every form
// (D2); the number is what the surface renders in bold. The card's two answers, its links, its
// end and its listen row are refine-lingua-review-session's (D4, D5, D8, D11).

export const review = {
  start: "Réviser",
  nothingDue: "Rien à réviser pour l'instant.",
  remaining: pluralForms({
    one: (n) => `${n} carte(s) à revoir`,
    other: (n) => `${n} carte(s) à revoir`,
  }),
  /** A card a miss brought back, on its second or third showing (add-lingua-review-repeat-hint). */
  repeat: (showing: number) => `À revoir · ${showing}ᵉ passage`,
  reveal: "Afficher la réponse",
  /** The two answers: a miss, graded `again`, on the left; a recall, graded `good`, on the right. */
  missed: "Pas su",
  recalled: "Su",
  markKnown: "Je connais",
  hide: "Ne plus me le montrer",
  noGloss: "Pas de traduction pour ce mot.",
  noGlossExpression: "Pas de traduction pour cette expression.",
  /** A card the pack cannot answer, translated on the device (add-lingua-review-translation D5). */
  translationLabel: "Dans votre phrase — traduction automatique",
  translationAloneLabel: "Traduction automatique",
  translating: "Traduction en cours…",
  /** The end of a session: its title, then one line per count, its figure in bold. */
  sessionDone: "Séance terminée",
  reviewed: pluralForms({
    one: (n) => `${n} mot revu`,
    other: (n) => `${n} mots revus`,
  }),
  recovered: pluralForms({
    one: (n) => `${n} rattrapé en route`,
    other: (n) => `${n} rattrapés en route`,
  }),
  holding: pluralForms({
    one: (n) => `${n} tient maintenant plus d'un mois`,
    other: (n) => `${n} tiennent maintenant plus d'un mois`,
  }),
  known: pluralForms({
    one: (n) => `${n} classé connu`,
    other: (n) => `${n} classés connus`,
  }),
  hidden: pluralForms({
    one: (n) => `${n} masqué`,
    other: (n) => `${n} masqués`,
  }),
  /** « Encore 10 »: another session of that many cards. */
  more: (n: string) => `Encore ${n}`,
  /** The listen row: the word or the expression, the sentence, and the dictionary form once revealed. */
  listenWord: "▶ Mot",
  listenWordLabel: "Écouter le mot",
  listenExpression: "▶ Expression",
  listenExpressionLabel: "Écouter l'expression",
  listenSentence: "▶ Phrase",
  listenSentenceLabel: "Écouter la phrase",
  listenForm: (form: string) => `▶ ${form}`,
  listenDictionaryFormLabel: (form: string) => `Écouter la forme du dictionnaire « ${form} »`,
  stop: "■ Arrêter",
  stopLabel: "Arrêter la lecture",
  language: "Langue",
  /** The summary line: « 12 carte(s) · 3 à revoir », its two counts in bold. */
  deckCards: pluralForms({
    one: (n) => `${n} carte(s)`,
    other: (n) => `${n} carte(s)`,
  }),
  dueCards: pluralForms({
    one: (n) => `${n} à revoir`,
    other: (n) => `${n} à revoir`,
  }),
  summarySeparator: " · ",
  /** Beside a book: the words looked up in it, still to sort (add-lingua-chapter-recap D6). */
  toSort: pluralForms({
    one: (n) => `${n} mot de ce livre à trier`,
    other: (n) => `${n} mots de ce livre à trier`,
  }),
};
