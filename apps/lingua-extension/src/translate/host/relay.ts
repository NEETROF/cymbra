// The background's side of a translation: mark the selection in its sentence, hand the markup
// to wherever the engine lives, and read the answer back. It awaits and does no work of its
// own, so the background keeps answering everything else while the engine translates.
//
// A selection costs two requests: the sentence with the selection tagged, which is what the
// reader sees, and the selection on its own, which only checks where the tag landed
// (reconcile.ts). The second is a check, never a dependency: if it fails, the tag's own mark
// stands, exactly as it would without it. A language whose marks are not measured costs one: the
// sentence untagged, answered without a mark (add-lingua-spanish-translation-pivot D3).

import { escapeText, MARKED_LANGUAGES, markSelection, readMarked, selectedText } from "../markup.ts";
import { type TranslationRequest, type TranslationResult, UNAVAILABLE } from "../port.ts";
import { reconcileMarks } from "../reconcile.ts";
import type { EngineAccess, EngineReply } from "./engine.ts";

export type RelayLog = (message: string, detail?: unknown) => void;

const LOG: RelayLog = (message, detail) => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

export async function relayTranslation(
  engine: Pick<EngineAccess, "translate">,
  request: TranslationRequest,
  log: RelayLog = LOG,
): Promise<TranslationResult> {
  if (!request.sentence.trim()) return UNAVAILABLE;
  if (!MARKED_LANGUAGES.includes(request.language)) return relayUnmarked(engine, request, log);
  try {
    const fragment = selectedText(request.sentence, request.selection);
    const [reply, alone] = await Promise.all([
      engine.translate(markSelection(request.sentence, request.selection), request.language),
      fragment ? engine.translate(escapeText(fragment), request.language).catch((): EngineReply | null => null) : null,
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

/** The sentence alone, untagged, and its translation without a mark. */
async function relayUnmarked(
  engine: Pick<EngineAccess, "translate">,
  request: TranslationRequest,
  log: RelayLog,
): Promise<TranslationResult> {
  try {
    const reply = await engine.translate(escapeText(request.sentence), request.language);
    if (!reply.ok) {
      log("no translation:", reply.reason);
      return UNAVAILABLE;
    }
    return { kind: "translated", translation: { sentence: readMarked(reply.html).sentence, marks: [] } };
  } catch (e: unknown) {
    log("translation failed:", e);
    return UNAVAILABLE;
  }
}

/**
 * A warm (add-lingua-translation-android D2, D3) for `language`, answered only where a translation
 * could be: with that language's models not ready nothing is loaded, not even to find a model
 * missing. A route said to be ready that the engine cannot load calls `onFailed`, so the background
 * can see whether it is still there — as it does after a translation that got no answer.
 */
export async function relayWarm(
  ready: (language: string) => Promise<boolean>,
  engine: Pick<EngineAccess, "warm">,
  language: string,
  onFailed: () => void = () => {},
  log: RelayLog = LOG,
): Promise<boolean> {
  try {
    if (!(await ready(language))) return false;
  } catch {
    return false;
  }
  try {
    if (await engine.warm(language)) return true;
  } catch (e: unknown) {
    log("the engine could not be warmed:", e);
  }
  onFailed();
  return false;
}
