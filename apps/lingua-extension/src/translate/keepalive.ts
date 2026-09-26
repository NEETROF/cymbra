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
//
// A frozen tab sends nothing. Firefox for Android freezes a tab when the reader switches to another
// app; the pings stop, the event page is torn down, and the engine with it — measured on the same
// tablet (2026-09-26): back after a minute, the next selection paid a 4.4 s cold start. So a page
// that was translating asks for the engine again when it becomes visible (add-lingua-translation-
// android D3), within the idle period only: it restores an engine that would still be loaded had
// the tab not been frozen, and nothing for a page that never translated or stopped long ago.

/** The ping's own message type — never the analyser's `lingua-rpc`, never the translate wire. */
export const KEEPALIVE_PING = "lingua-translate-keepalive";

/**
 * Well inside the shortest idle gap measured on the device that showed the problem (8 s), so a
 * teardown never falls between two pings.
 */
export const PING_MS = 5_000;

import { ENGINE_IDLE_MS, type TranslationRequest, type TranslationResult, type TranslatorPort } from "./port.ts";

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
 * Keep the engine's host busy for the life of this page — or until the returned stop is called.
 * `when` skips a tick without stopping: the setting pings only while it is on screen.
 *
 * A ping that fails means the extension context is gone — the page is orphaned, nothing else in
 * it works either, and there is nothing left to hold. Stop there rather than ping a dead
 * background forever.
 */
export function keepEngineWarm(deps: KeepaliveDeps = REAL, when: () => boolean = () => true): () => void {
  const stop = deps.every(PING_MS, () => {
    if (!when()) return;
    void (async () => {
      try {
        await deps.ping();
      } catch {
        stop();
      }
    })();
  });
  return stop;
}

/** What the return to a page needs: the time, and word that the page is visible again. */
export interface RestoreDeps {
  now(): number;
  /** Call `fn` each time the page becomes visible again. */
  onVisible(fn: () => void): void;
}

const REAL_RESTORE: RestoreDeps = {
  now: () => Date.now(),
  onVisible: (fn) =>
    document.addEventListener("visibilitychange", () => {
      if (document.visibilityState === "visible") fn();
    }),
};

/**
 * A translator that keeps the host busy from its first request onwards. Before that first
 * request the engine has never been loaded, so there is nothing to keep and nothing is sent.
 * From then on, a return to the page within the idle period of its last translation asks for the
 * engine again (D3).
 */
export function keepWarm(
  port: TranslatorPort,
  start: () => void = () => void keepEngineWarm(),
  restore: RestoreDeps = REAL_RESTORE,
): TranslatorPort {
  let last: number | null = null;
  return {
    translate(request: TranslationRequest): Promise<TranslationResult> {
      if (last === null) {
        start();
        restore.onVisible(() => {
          if (last !== null && restore.now() - last < ENGINE_IDLE_MS) port.warm?.();
        });
      }
      last = restore.now();
      return port.translate(request);
    },
    warm: () => port.warm?.(),
  };
}
