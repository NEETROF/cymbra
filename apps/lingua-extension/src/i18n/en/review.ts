import type { review as fr } from "../fr/review.ts";

// The review's copy in English — a draft after the French (src/i18n/README.md).

export const review: typeof fr = {
  start: "Review",
  nothingDue: "Nothing to review for now.",
  remaining: {
    one: (n) => `${n} card to review`,
    other: (n) => `${n} cards to review`,
  },
  repeat: (showing: number) => `Again · ${showing === 2 ? "2nd" : showing === 3 ? "3rd" : `${showing}th`} time`,
  practice: "Practice anyway",
  practiceRemaining: {
    one: (n) => `Practice · ${n} card`,
    other: (n) => `Practice · ${n} cards`,
  },
  reveal: "Show the answer",
  missed: "Didn't know",
  recalled: "Knew it",
  markKnown: "I know it",
  hide: "Don't show it again",
  noGloss: "No translation for this word.",
  noGlossExpression: "No translation for this expression.",
  translationLabel: "In your sentence — machine translation",
  translationAloneLabel: "Machine translation",
  translating: "Translating…",
  sessionDone: "Session complete",
  reviewed: {
    one: (n) => `${n} word reviewed`,
    other: (n) => `${n} words reviewed`,
  },
  recovered: {
    one: (n) => `${n} caught up along the way`,
    other: (n) => `${n} caught up along the way`,
  },
  holding: {
    one: (n) => `${n} now holds for over a month`,
    other: (n) => `${n} now hold for over a month`,
  },
  known: {
    one: (n) => `${n} marked as known`,
    other: (n) => `${n} marked as known`,
  },
  hidden: {
    one: (n) => `${n} hidden`,
    other: (n) => `${n} hidden`,
  },
  more: (n) => `${n} more`,
  listenWord: "▶ Word",
  listenWordLabel: "Listen to the word",
  listenExpression: "▶ Phrase",
  listenExpressionLabel: "Listen to the phrase",
  listenSentence: "▶ Sentence",
  listenSentenceLabel: "Listen to the sentence",
  listenForm: (form) => `▶ ${form}`,
  listenDictionaryFormLabel: (form) => `Listen to the dictionary form “${form}”`,
  stop: "■ Stop",
  stopLabel: "Stop reading",
  language: "Language",
  deckCards: {
    one: (n) => `${n} card`,
    other: (n) => `${n} cards`,
  },
  dueCards: {
    one: (n) => `${n} to review`,
    other: (n) => `${n} to review`,
  },
  summarySeparator: " · ",
  toSort: {
    one: (n) => `${n} word from this book to sort`,
    other: (n) => `${n} words from this book to sort`,
  },
};
