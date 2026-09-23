// Keeping the engine loaded between two selections.
//
// Measured on a Galaxy Tab S6 Lite (Firefox): the event page is torn down after ~30 s of
// inactivity, and with it go the engine's Worker and the 36.7 MB of model it had read. A
// selection made a minute after the last one therefore pays a full cold start, which on that
// hardware costs more than the card is willing to wait — so it answers with the pack's
// word-by-word instead. A minute between two selections is ordinary reading, so nearly every
// selection paid it.
//
// An open port is what keeps an event page loaded, so the reader holds one for as long as it
// is on the page. It carries no messages: its existence IS the signal, and it closes with the
// page. Nothing is warmed eagerly — the engine still loads on the first translation, and what
// this buys is that it is not thrown away before the second.

/** Named so the background can tell it from the analyser's traffic. */
export const KEEPALIVE_PORT = "lingua-translate-keepalive";

/** After the background goes away for its own reasons (an update, a crash), before retrying. */
export const RECONNECT_MS = 1_000;

/** The slice of a runtime port this needs — a test hands in a fake. */
export interface KeepalivePort {
  onDisconnect: { addListener(listener: () => void): void };
}

export interface KeepaliveDeps {
  /** Opens the port. Throws once the extension context is gone. */
  connect(): KeepalivePort;
  schedule(retry: () => void, ms: number): void;
}

const REAL: KeepaliveDeps = {
  connect: () => chrome.runtime.connect({ name: KEEPALIVE_PORT }),
  schedule: (retry, ms) => {
    setTimeout(retry, ms);
  },
};

/**
 * Hold a port open for the life of this page, reopening it if the background goes away anyway.
 *
 * A `connect` that throws means the extension context itself is gone — the page is orphaned,
 * nothing else in it works either, and there is nothing left to hold on to. Stop there rather
 * than retry forever in a page that can no longer be served.
 */
export function keepEngineWarm(deps: KeepaliveDeps = REAL): void {
  const open = (): void => {
    let port: KeepalivePort;
    try {
      port = deps.connect();
    } catch {
      return;
    }
    port.onDisconnect.addListener(() => deps.schedule(open, RECONNECT_MS));
  };
  open();
}
