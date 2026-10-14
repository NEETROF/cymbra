import type { ReviewCard } from "../analyzer/port.ts";
import type { MarkedTranslation } from "../translate/markup.ts";
import { TRANSLATION_WAIT_MS, type TranslationRequest, type TranslatorPort } from "../translate/port.ts";
import { cardHeadword, markedIn } from "./view.ts";

// The translation a review card is answered with when the pack has none (add-lingua-review-translation):
// the card's sentence, the word's place marked where the front marks it — the word card's own rule
// (D1, D3). Computed for display and dropped with the card, never stored (D6).

/** Where the review's translation of the current card stands, as the answer shows it (D5). */
export type ReviewTranslation =
  { state: "pending" } | { state: "translated"; translation: MarkedTranslation; alone: boolean } | { state: "none" };

/** What to ask the engine for `card`, and whether it is the expression alone; null asks nothing. */
export interface ReviewTranslationRequest {
  request: TranslationRequest;
  alone: boolean;
}

/**
 * The request for a card the pack cannot answer: none for a card with a gloss (D1). In its
 * sentence, the word marked where the front marks it, in UTF-16 offsets; an expression its sentence
 * does not hold alone, unmarked; a single word in that case not at all — the guess the word card
 * never shows (D3).
 */
export function reviewTranslationRequest(card: ReviewCard): ReviewTranslationRequest | null {
  if (card.gloss) return null;
  const language = card.language ?? "en";
  // The word without its edge punctuation, marked where the front marks it
  // (fix-lingua-selection-card-punctuation D4).
  const headword = cardHeadword(card);
  const sentence = card.sentence.trim();
  const marked = markedIn(card);
  if (marked) {
    const start = marked.before.length;
    return { request: { sentence, selection: { start, end: start + marked.text.length }, language }, alone: false };
  }
  if (!/\s/u.test(headword)) return null;
  return { request: { sentence: headword, selection: null, language }, alone: true };
}

/** The engine's answer, or null — never later than TRANSLATION_WAIT_MS, as the word card's. */
export function translateBounded(
  translator: TranslatorPort,
  request: TranslationRequest,
): Promise<MarkedTranslation | null> {
  return new Promise((resolve) => {
    const timer = setTimeout(() => resolve(null), TRANSLATION_WAIT_MS);
    const done = (translation: MarkedTranslation | null): void => {
      clearTimeout(timer);
      resolve(translation);
    };
    translator.translate(request).then(
      (result) => done(result.kind === "translated" ? result.translation : null),
      () => done(null),
    );
  });
}
