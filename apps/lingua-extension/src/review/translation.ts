import type { ReviewCard } from "../analyzer/port.ts";
import type { MarkedTranslation } from "../translate/markup.ts";
import type { OnlineTranslationRequest } from "../translate/online.ts";
import { TRANSLATION_WAIT_MS, type TranslationRequest, type TranslatorPort } from "../translate/port.ts";
import { cardHeadword, markedIn } from "./view.ts";

// The translation a review card is answered with when the pack has none (add-lingua-review-translation),
// or when the reader asks for it under a gloss (add-lingua-review-translation-fallback D1): the card's
// sentence, the word's place marked where the front marks it — the word card's own rule (D1, D3).
// Computed for display and dropped with the card unless the reader keeps the word's part of it (D3).

/**
 * Where the review's translation of the current card stands, as the answer shows it (D5). `online`:
 * the online service gave it (add-lingua-review-translation-fallback D2), not the engine.
 */
export type ReviewTranslation =
  | { state: "pending" }
  | { state: "translated"; translation: MarkedTranslation; alone: boolean; online?: boolean }
  | { state: "none" };

/** What to ask the engine for `card`, and whether it is the expression alone; null asks nothing. */
export interface ReviewTranslationRequest {
  request: TranslationRequest;
  alone: boolean;
}

/**
 * The request for `card`, gloss or not: in its sentence, the word marked where the front marks it,
 * in UTF-16 offsets; an expression its sentence does not hold alone, unmarked; a single word in that
 * case not at all — the guess the word card never shows (D3).
 */
export function cardTranslationRequest(card: ReviewCard): ReviewTranslationRequest | null {
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

/**
 * The request the engine is asked on its own: none for a card with a gloss (D1), which is translated
 * only when the reader asks (add-lingua-review-translation-fallback D1).
 */
export function reviewTranslationRequest(card: ReviewCard): ReviewTranslationRequest | null {
  return card.gloss ? null : cardTranslationRequest(card);
}

/**
 * What « Traduction en ligne » sends for a card (add-lingua-review-translation-fallback D2): the same
 * sentence and span the engine is asked — a single word in its sentence, marked by position, never
 * alone; an expression its sentence does not hold, alone, as the whole selection.
 */
export function onlineReviewRequest(asked: ReviewTranslationRequest): OnlineTranslationRequest {
  const { sentence, selection, language } = asked.request;
  return { sentence, selection: selection ?? { start: 0, end: sentence.length }, source: language };
}

/** The longest gloss the reader may keep, in UTF-16 code units — JavaScript's `length` (D3). */
export const KEEP_MAX_UNITS = 80;

/** Whether `text`, trimmed, may be kept as a gloss: not empty, at most KEEP_MAX_UNITS. */
export function keepable(text: string): boolean {
  const trimmed = text.trim();
  return trimmed.length > 0 && trimmed.length <= KEEP_MAX_UNITS;
}

/**
 * What keeping a translation would keep (add-lingua-review-translation-fallback D3): for an expression
 * translated alone, the whole translation; otherwise the marked text — the marks in range, in order,
 * an overlapping one skipped as the answer skips it, joined by « … ». `split` when it joins several
 * marks: the engine may have split the word's translation, or marked a word that is not part of it
 * (« <b>facilitamment appelé</b> une ruée <b>vers</b> le sucre », the iPhone spike), so such a
 * candidate is never kept in one tap — it prefills the field. Null without one.
 */
export interface KeepCandidate {
  text: string;
  split: boolean;
}

export function keepCandidate(translation: MarkedTranslation, alone: boolean): KeepCandidate | null {
  const { sentence, marks } = translation;
  if (alone) {
    const text = sentence.trim();
    return text ? { text, split: false } : null;
  }
  const parts: string[] = [];
  let at = 0;
  for (const { start, end } of [...marks].sort((a, b) => a.start - b.start)) {
    if (start < at || end > sentence.length) continue;
    const part = sentence.slice(start, end).trim();
    if (part) parts.push(part);
    at = end;
  }
  return parts.length > 0 ? { text: parts.join(" … "), split: parts.length > 1 } : null;
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
