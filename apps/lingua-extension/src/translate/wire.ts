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
    !!selection &&
    typeof selection === "object" &&
    Number.isInteger(selection.start) &&
    Number.isInteger(selection.end)
  );
}
