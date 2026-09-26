// The engine host's contracts. Everything under translate/host/ runs off every thread that
// paints — the background, the offscreen document, the engine's own worker — and no surface
// may reach it (test/lint-translator-placement.spec.ts).

/** A reply from the engine: its markup answer, or why there is none (for the log, not the reader). */
export type EngineReply = { ok: true; html: string } | { ok: false; reason: string };

/** Where the engine is, as the background sees it: something that turns markup into markup. */
export interface EngineAccess {
  translate(markup: string): Promise<EngineReply>;
  /**
   * Load the engine without translating, because a translation is coming — a selection has begun,
   * a page that was translating is back (add-lingua-translation-android D2, D3). It arms the idle
   * release as a translation does. True once the engine is loaded.
   */
  warm(): Promise<boolean>;
}

/** The worker's answer to `load` when the model is not on the device: off, or never finished. */
export const NO_MODEL = "the model is not on this device";

/** The engine worker's protocol. Requests carry an id so replies can arrive in any order. */
export type WorkerRequest = { id: number; op: "load" } | { id: number; op: "translate"; markup: string };
export type WorkerResponse = { id: number; ok: true; html?: string } | { id: number; ok: false; error: string };

export function isEngineReply(value: unknown): value is EngineReply {
  const v = value as Partial<{ ok: boolean; html: unknown; reason: unknown }> | null;
  if (!v || typeof v.ok !== "boolean") return false;
  return v.ok ? typeof v.html === "string" : typeof v.reason === "string";
}
