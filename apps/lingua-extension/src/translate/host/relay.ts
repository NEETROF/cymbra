// The background's side of a translation: mark the selection in its sentence, hand the markup
// to wherever the engine lives, and read the answer back. It awaits and does no work of its
// own, so the background keeps answering everything else while the engine translates.

import { markSelection, readMarked } from "../markup.ts";
import { type TranslationRequest, type TranslationResult, UNAVAILABLE } from "../port.ts";
import type { EngineAccess } from "./engine.ts";

export type RelayLog = (message: string, detail?: unknown) => void;

const LOG: RelayLog = (message, detail) => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

export async function relayTranslation(
  engine: EngineAccess,
  request: TranslationRequest,
  log: RelayLog = LOG,
): Promise<TranslationResult> {
  if (!request.sentence.trim()) return UNAVAILABLE;
  try {
    const reply = await engine.translate(markSelection(request.sentence, request.selection));
    if (!reply.ok) {
      log("no translation:", reply.reason);
      return UNAVAILABLE;
    }
    return { kind: "translated", translation: readMarked(reply.html) };
  } catch (e: unknown) {
    log("translation failed:", e);
    return UNAVAILABLE;
  }
}
