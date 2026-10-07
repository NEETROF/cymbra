// The engine host's contracts. Everything under translate/host/ runs off every thread that
// paints — the background, the offscreen document, the engine's own worker — and no surface
// may reach it (test/lint-translator-placement.spec.ts).

/** A reply from the engine: its markup answer, or why there is none (for the log, not the reader). */
export type EngineReply = { ok: true; html: string } | { ok: false; reason: string };

/**
 * Where the engine is, as the background sees it: something that turns markup in a pair's studied
 * language into markup in its native language, through that pair's route
 * (generalise-lingua-translation-model-state D5, generalise-lingua-translation-routes-by-pair D2).
 */
export interface EngineAccess {
  translate(markup: string, pair: string): Promise<EngineReply>;
  /**
   * Load the engine and `pair`'s route without translating, because a translation is coming — a
   * selection has begun, a page that was translating is back (add-lingua-translation-android D2, D3).
   * It arms the idle release as a translation does. True once the route is loaded.
   */
  warm(pair: string): Promise<boolean>;
}

/** The worker's answer to `load` when the model is not on the device: off, or never finished. */
export const NO_MODEL = "the model is not on this device";

/**
 * The worker's answer to `load` for a route of more than two models: the engine pivots through one
 * language at most (add-lingua-spanish-translation-pivot D2), and no catalogue route chains more.
 */
export const LONG_ROUTE = "a route of more than two models is not supported";

/** The engine worker's protocol. Requests carry an id so replies can arrive in any order, and the pair whose route they go through. */
export type WorkerRequest =
  { id: number; op: "load"; pair: string } | { id: number; op: "translate"; markup: string; pair: string };
/**
 * A refusal flagged `trap` is not one the engine gave: the engine trapped under the request
 * (harden-lingua-translation-engine D1), the worker has closed itself, and its owner starts a fresh
 * one. A worker built before the flag existed never sends it, and reads as a refusal.
 */
export type WorkerResponse =
  { id: number; ok: true; html?: string } | { id: number; ok: false; error: string; trap?: true };

/**
 * Whether `error` is a WebAssembly trap rather than a refusal the engine made on purpose (D1): the
 * runtime's own error — "memory access out of bounds" — or the glue's `abort()`, which throws one
 * whose message starts with `Aborted(`. A trapped instance's memory is not to be trusted again.
 * `NO_MODEL`, `LONG_ROUTE`, "the engine is not loaded", a missing file: none of these is one.
 */
export function isTrap(error: unknown): boolean {
  if (typeof WebAssembly !== "undefined" && error instanceof WebAssembly.RuntimeError) return true;
  return error instanceof Error && error.message.startsWith("Aborted(");
}

export function isEngineReply(value: unknown): value is EngineReply {
  const v = value as Partial<{ ok: boolean; html: unknown; reason: unknown }> | null;
  if (!v || typeof v.ok !== "boolean") return false;
  return v.ok ? typeof v.html === "string" : typeof v.reason === "string";
}
