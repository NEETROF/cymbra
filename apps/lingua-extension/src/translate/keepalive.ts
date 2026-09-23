// Keeping the engine loaded between two selections.
//
// Measured on a Galaxy Tab S6 Lite (Firefox): the event page is torn down when idle, and with it
// go the engine's Worker and the 36.7 MB of model it had read. A selection made a minute after
// the last one therefore pays a full cold start, which on that hardware costs more than the card
// is willing to wait — so it answers with the pack's word-by-word instead. A minute between two
// selections is ordinary reading, so nearly every selection paid it.
//
// An open port does NOT hold that page: measured on the same device, the background restarted
// every 8-24 s with two reader ports connected throughout. What is left is to keep the page
// *busy* — a message it answers is activity, where a connection sitting open is not. So the
// reader pings while it is on the page, often enough to land inside the shortest gap observed.
//
// Nothing is warmed eagerly, and nothing is held before there is something to hold: the pings
// start with the page's FIRST translation (`keepWarm`), so a reader who selects nothing never
// wakes the background at all. What this buys is that the model is not thrown away before the
// second selection.

/** The ping's own message type — never the analyser's `lingua-rpc`, never the translate wire. */
export const KEEPALIVE_PING = "lingua-translate-keepalive";

/**
 * Well inside the shortest idle gap measured on the device that showed the problem (8 s), so a
 * teardown never falls between two pings.
 */
export const PING_MS = 5_000;

import type { TranslationRequest, TranslationResult, TranslatorPort } from "./port.ts";

export interface KeepaliveDeps {
  /** Sends one ping. Rejects, or throws, once the extension context is gone. */
  ping(): Promise<unknown>;
  /** Starts the repeating timer; returns how to stop it. */
  every(ms: number, tick: () => void): () => void;
}

const REAL: KeepaliveDeps = {
  ping: () => chrome.runtime.sendMessage({ type: KEEPALIVE_PING }),
  every: (ms, tick) => {
    const handle = setInterval(tick, ms);
    return () => clearInterval(handle);
  },
};

/**
 * Keep the engine's host busy for the life of this page.
 *
 * A ping that fails means the extension context is gone — the page is orphaned, nothing else in
 * it works either, and there is nothing left to hold. Stop there rather than ping a dead
 * background forever.
 */
export function keepEngineWarm(deps: KeepaliveDeps = REAL): void {
  const stop = deps.every(PING_MS, () => {
    void (async () => {
      try {
        await deps.ping();
      } catch {
        stop();
      }
    })();
  });
}

/**
 * A translator that keeps the host busy from its first request onwards. Before that first
 * request the engine has never been loaded, so there is nothing to keep and nothing is sent.
 */
export function keepWarm(port: TranslatorPort, start: () => void = keepEngineWarm): TranslatorPort {
  let started = false;
  return {
    translate(request: TranslationRequest): Promise<TranslationResult> {
      if (!started) {
        started = true;
        start();
      }
      return port.translate(request);
    },
  };
}
