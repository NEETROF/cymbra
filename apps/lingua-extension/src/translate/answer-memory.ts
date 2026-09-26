// The translations a page has already received (add-lingua-translation-android D4).
//
// Measured on a Galaxy Tab S6 Lite (2026-09-26): adjusting the selection handles over one sentence
// asked the engine for it five times in six seconds, 0.4 s of CPU each, and the first selection of
// a visit sent the same request twice while the engine was still loading. The engine marks the
// selection inside its answer, so an answer belongs to a sentence AND a span: the same pair is
// answered from here, a different span of the same sentence is asked for.
//
// Only translations are kept — an `unavailable` answer is asked again next time, since the model
// may be ready by then. The memory is the page's: it lives in this object, is never stored, never
// crosses tabs, and goes with the page.

import type { TranslationRequest, TranslationResult, TranslatorPort } from "./port.ts";

/** How many answers a page keeps: the most recent, a reading session's worth of re-selections. */
export const ANSWER_MEMORY_SIZE = 32;

function keyOf(request: TranslationRequest): string {
  return JSON.stringify([request.sentence, request.selection?.start ?? null, request.selection?.end ?? null]);
}

/** `port`, answering a request it has already answered — or is answering — without asking again. */
export function rememberAnswers(port: TranslatorPort, size: number = ANSWER_MEMORY_SIZE): TranslatorPort {
  const kept = new Map<string, TranslationResult>();
  const asking = new Map<string, Promise<TranslationResult>>();
  return {
    translate(request: TranslationRequest): Promise<TranslationResult> {
      const key = keyOf(request);
      const answer = kept.get(key);
      if (answer) {
        kept.delete(key); // most recent last: the Map's order is the eviction order
        kept.set(key, answer);
        return Promise.resolve(answer);
      }
      const pending = asking.get(key);
      if (pending) return pending;
      const asked = port.translate(request).then(
        (result) => {
          asking.delete(key);
          if (result.kind === "translated") {
            kept.set(key, result);
            if (kept.size > size) kept.delete(kept.keys().next().value!);
          }
          return result;
        },
        (e: unknown) => {
          asking.delete(key); // a failure is not an answer: the next request asks again
          throw e;
        },
      );
      asking.set(key, asked);
      return asked;
    },
    warm: () => port.warm?.(),
  };
}
