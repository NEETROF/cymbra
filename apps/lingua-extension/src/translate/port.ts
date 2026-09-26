// The seam the reading code translates through (add-lingua-translation-engine). It has the
// shape of LinguaPort for the same reason: the caller never learns where the engine runs.
//
// It differs in one respect, and on purpose. LinguaPort has an in-context implementation —
// on Chromium the analyser runs in the content script itself. This seam has NONE. The engine
// blocks for as long as a translation takes, so on any thread that paints it would be a
// freeze; the only implementation a caller can obtain sends the request away. That is
// enforced by test/lint-translator-placement.spec.ts, not left to discipline.

import type { MarkedTranslation, Span } from "./markup.ts";

/** What the reader selected, and where: the translator marks it in its sentence by position. */
export interface TranslationRequest {
  /** The sentence the selection sits in, as the page shows it. */
  sentence: string;
  /** The selection's [start, end) in `sentence`, or null to translate the sentence unmarked. */
  selection: Span | null;
}

/**
 * The engine's answer. `unavailable` covers every reason there is none — no model supplied,
 * an engine that did not start within its bound, a request that timed out — because the
 * caller does the same thing in each case: answer as it would with no engine at all.
 */
export type TranslationResult = { kind: "translated"; translation: MarkedTranslation } | { kind: "unavailable" };

export interface TranslatorPort {
  translate(request: TranslationRequest): Promise<TranslationResult>;
  /**
   * A translation is coming — a selection has begun (add-lingua-translation-android D2): have the
   * engine load now, answer nothing. A hint, never a condition: a port without it translates the
   * same, only colder.
   */
  warm?(): void;
}

export const UNAVAILABLE: TranslationResult = { kind: "unavailable" };

/**
 * How long the engine stays loaded after the last translation asked. A starting value: long
 * enough to cover a pause in reading, short enough not to hold ~200 MiB for an afternoon.
 *
 * Here, not beside the engine's worker, because a page reads it too: a page that translated
 * within this period asks for the engine again when it comes back (add-lingua-translation-android
 * D3) — it restores only an engine that would still be loaded had the tab not been frozen.
 */
export const ENGINE_IDLE_MS = 10 * 60_000;
