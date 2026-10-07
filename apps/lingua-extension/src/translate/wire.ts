// The translation protocol between the surfaces that ask and the background that answers.
// Its own message type, never the analyser's `lingua-rpc`: a translation must not be able to
// queue behind an analysis, or an analysis behind a translation.

import type { TranslationRequest, TranslationResult } from "./port.ts";

export const TRANSLATE_TYPE = "lingua-translate";

export interface TranslateMessage {
  type: typeof TRANSLATE_TYPE;
  request: TranslationRequest;
}

export type TranslateReply = TranslationResult;

export function isTranslateMessage(message: unknown): message is TranslateMessage {
  const m = message as Partial<TranslateMessage> | null;
  if (m?.type !== TRANSLATE_TYPE || !m.request || typeof m.request !== "object") return false;
  const { sentence, selection, language } = m.request as Partial<TranslationRequest>;
  if (typeof sentence !== "string" || typeof language !== "string" || !language) return false;
  if (selection === null) return true;
  return (
    !!selection && typeof selection === "object" && Number.isInteger(selection.start) && Number.isInteger(selection.end)
  );
}

/**
 * Load the engine now, translate nothing (add-lingua-translation-android D2, D3): a selection has
 * begun, or a page that was translating is back. Its own type, like the keep-warm ping's — the
 * background answers it only when the language's models are ready, and loading is all it does.
 */
export const WARM_TYPE = "lingua-translate-warm";

export interface WarmMessage {
  type: typeof WARM_TYPE;
  /**
   * The document's language; the background forms the pair whose route to load with the reader's
   * native language (generalise-lingua-translation-model-state D5, routes-by-pair D2).
   */
  language: string;
}

export function isWarmMessage(message: unknown): message is WarmMessage {
  const m = message as Partial<WarmMessage> | null;
  return m?.type === WARM_TYPE && typeof m.language === "string" && m.language.length > 0;
}
