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
  const { sentence, selection } = m.request as Partial<TranslationRequest>;
  if (typeof sentence !== "string") return false;
  if (selection === null) return true;
  return (
    !!selection && typeof selection === "object" && Number.isInteger(selection.start) && Number.isInteger(selection.end)
  );
}

/**
 * Load the engine now, translate nothing (add-lingua-translation-android D2, D3): a selection has
 * begun, or a page that was translating is back. Its own type, like the keep-warm ping's — the
 * background answers it only when a model is ready, and loading is all it does.
 */
export const WARM_TYPE = "lingua-translate-warm";

export interface WarmMessage {
  type: typeof WARM_TYPE;
}

export function isWarmMessage(message: unknown): message is WarmMessage {
  return (message as Partial<WarmMessage> | null)?.type === WARM_TYPE;
}
