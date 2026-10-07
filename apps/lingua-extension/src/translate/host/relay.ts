// The background's side of a translation: mark the selection in its sentence, hand the markup
// to wherever the engine lives, and read the answer back. It awaits and does no work of its
// own, so the background keeps answering everything else while the engine translates.
//
// A translation goes through a pair: the document's language the page asked in, and the reader's
// native language, which the background reads from their stored profile and `answerTranslation`
// joins (generalise-lingua-translation-routes-by-pair D2). A page never names a pair. The background
// itself is glue, outside the coverage gate: what it does with a request is here, and tested.
//
// A selection costs two requests: the sentence with the selection tagged, which is what the
// reader sees, and the selection on its own, which only checks where the tag landed
// (reconcile.ts). The second is a check, never a dependency: if it fails, the tag's own mark
// stands, exactly as it would without it. A pair whose marks are not measured costs one: the
// sentence untagged, answered without a mark (add-lingua-spanish-translation-pivot D3, routes-by-pair D4).

import { pairOf } from "../../analyzer/pairs.ts";
import { escapeText, MARKED_PAIRS, markSelection, readMarked, selectedText, withoutFootnotes } from "../markup.ts";
import { type TranslationRequest, type TranslationResult, UNAVAILABLE } from "../port.ts";
import { reconcileMarks } from "../reconcile.ts";
import type { EngineAccess, EngineReply } from "./engine.ts";

export type RelayLog = (message: string, detail?: unknown) => void;

const LOG: RelayLog = (message, detail) => console.warn(`[Cymbra Lingua] ${message}`, detail ?? "");

/** What answering a page takes of the background: the engine, the reader's profile, and what the device recorded. */
export interface AnswerDeps {
  engine: Pick<EngineAccess, "translate" | "warm">;
  /** The reader's native language, from their stored profile: the half of the pair a page does not send. */
  native: () => Promise<string>;
  /** Whether every model of `pair`'s route is on the device, as recorded (model-controller.ts `ready`). */
  ready: (pair: string) => Promise<boolean>;
  /**
   * A pair asked for that is not recorded ready, or a route said to be ready that gave no answer:
   * what is recorded may no longer hold, so the background reconciles it (`model.status()`), and the
   * page's gate follows through storage.
   */
  onNotReady: () => void;
  log?: RelayLog;
}

/**
 * A page's request, answered: the pair is formed from the document's language the page asked in
 * and the reader's native language (routes-by-pair D2); unless that pair is recorded ready the
 * answer is unavailable and the engine is never started for it — not even to find a model missing;
 * otherwise the request is relayed through the pair's route. Never throws: a setting that cannot
 * be read answers as without a model.
 */
export async function answerTranslation(deps: AnswerDeps, request: TranslationRequest): Promise<TranslationResult> {
  let pair: string;
  try {
    pair = pairOf(request.language, await deps.native());
    if (!(await deps.ready(pair))) {
      deps.onNotReady();
      return UNAVAILABLE;
    }
  } catch (e: unknown) {
    (deps.log ?? LOG)("the translation setting could not be read:", e);
    return UNAVAILABLE;
  }
  const result = await relayTranslation(deps.engine, request, pair, deps.log);
  if (result.kind === "unavailable") deps.onNotReady();
  return result;
}

/** A page's warm for the document's `language`: its pair formed as for a translation, then `relayWarm`. */
export async function answerWarm(deps: AnswerDeps, language: string): Promise<boolean> {
  let pair: string;
  try {
    pair = pairOf(language, await deps.native());
  } catch {
    return false;
  }
  return relayWarm(deps.ready, deps.engine, pair, deps.onNotReady, deps.log);
}

/** Relay `asked`, a request in its document's language, through `pair`'s route. */
export async function relayTranslation(
  engine: Pick<EngineAccess, "translate">,
  asked: TranslationRequest,
  pair: string,
  log: RelayLog = LOG,
): Promise<TranslationResult> {
  // The footnote calls go before anything reaches the engine; the selection moves with its text.
  const request = { ...asked, ...withoutFootnotes(asked.sentence, asked.selection) };
  if (!request.sentence.trim()) return UNAVAILABLE;
  if (!MARKED_PAIRS.includes(pair)) return relayUnmarked(engine, request, pair, log);
  try {
    const fragment = selectedText(request.sentence, request.selection);
    const [reply, alone] = await Promise.all([
      engine.translate(markSelection(request.sentence, request.selection), pair),
      fragment ? engine.translate(escapeText(fragment), pair).catch((): EngineReply | null => null) : null,
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
  pair: string,
  log: RelayLog,
): Promise<TranslationResult> {
  try {
    const reply = await engine.translate(escapeText(request.sentence), pair);
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
 * A warm (add-lingua-translation-android D2, D3) for `pair`, answered only where a translation
 * could be: with that pair's models not ready nothing is loaded, not even to find a model
 * missing. A route said to be ready that the engine cannot load calls `onFailed`, so the background
 * can see whether it is still there — as it does after a translation that got no answer.
 */
export async function relayWarm(
  ready: (pair: string) => Promise<boolean>,
  engine: Pick<EngineAccess, "warm">,
  pair: string,
  onFailed: () => void = () => {},
  log: RelayLog = LOG,
): Promise<boolean> {
  try {
    if (!(await ready(pair))) return false;
  } catch {
    return false;
  }
  try {
    if (await engine.warm(pair)) return true;
  } catch (e: unknown) {
    log("the engine could not be warmed:", e);
  }
  onFailed();
  return false;
}
