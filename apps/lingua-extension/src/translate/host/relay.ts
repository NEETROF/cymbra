// The background's side of a translation: mark the selection in its sentence, hand the markup
// to wherever the engine lives, and read the answer back. It awaits and does no work of its
// own, so the background keeps answering everything else while the engine translates.
//
// A selection costs two requests: the sentence with the selection tagged, which is what the
// reader sees, and the selection on its own, which only checks where the tag landed
// (reconcile.ts). The second is a check, never a dependency: if it fails, the tag's own mark
// stands, exactly as it would without it.

import { escapeText, markSelection, readMarked, selectedText } from "../markup.ts";
import { type TranslationRequest, type TranslationResult, UNAVAILABLE } from "../port.ts";
import { reconcileMarks } from "../reconcile.ts";
import type { EngineAccess, EngineReply } from "./engine.ts";

export type RelayLog = (message: string, detail?: unknown) => void;

const LOG: RelayLog = (message, detail) => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

export async function relayTranslation(
  engine: EngineAccess,
  request: TranslationRequest,
  log: RelayLog = LOG,
): Promise<TranslationResult> {
  if (!request.sentence.trim()) return UNAVAILABLE;
  try {
    const fragment = selectedText(request.sentence, request.selection);
    const [reply, alone] = await Promise.all([
      engine.translate(markSelection(request.sentence, request.selection)),
      fragment ? engine.translate(escapeText(fragment)).catch((): EngineReply | null => null) : null,
    ]);
    if (!reply.ok) {
      log("no translation:", reply.reason);
      return UNAVAILABLE;
    }
    const marked = readMarked(reply.html);
    if (!alone?.ok) return { kind: "translated", translation: marked };
    return { kind: "translated", translation: reconcileMarks(marked, readMarked(alone.html).sentence) };
  } catch (e: unknown) {
    log("translation failed:", e);
    return UNAVAILABLE;
  }
}
