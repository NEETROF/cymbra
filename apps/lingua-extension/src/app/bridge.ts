// The Cymbra Lingua app's bridge between its page and the App Group (add-lingua-apple-review D3):
// the page posts a message to the web view's `lingua` handler (ViewController.swift, a
// WKScriptMessageHandlerWithReply) and awaits its reply. The handler only stores and returns the
// relay's strings; it never reads them.

/** What the page asks the app. */
export type AppMessage =
  /** The extension's state, as its handler left it: `{state}`, or `{}` before the extension has run. */
  | { type: "review.state" }
  /** The changes this app wrote and the extension has not taken yet: `{ops, writtenAt}`, or `{}`. */
  | { type: "review.pendingOps" }
  /** Replace the changes waiting for the extension with `ops`. */
  | { type: "review.writeOps"; ops: string }
  /** The page cannot show the state it was given: the app shows how to turn the extension on. */
  | { type: "review.unavailable" }
  /**
   * A review session ended saying it was complete (« Séance terminée »): the app counts it, and may
   * ask StoreKit for a review — never a prompt of its own (add-lingua-comeback-hints D11).
   */
  | { type: "review.sessionFinished" }
  /**
   * The app's own session (add-lingua-review-translation-fallback D8): open the app's Apple / Google
   * sheet and answer `{provider, idToken}`, or `{}` when the reader closed it or it failed.
   */
  | { type: "auth.signIn" }
  /** The refresh token the app's Keychain holds: `{refresh}`, or `{}`. */
  | { type: "auth.readRefresh" }
  /** Keep `refresh` in the app's Keychain, replacing the one kept. */
  | { type: "auth.writeRefresh"; refresh: string }
  /** Forget the refresh token: signed out, or the session refused. */
  | { type: "auth.clear" };

export type AppSend = (message: AppMessage) => Promise<unknown>;

interface ReplyingHandler {
  postMessage(message: unknown): Promise<unknown>;
}

/** The web view's handler, or null outside the app (a page opened elsewhere has nothing to ask). */
export function webkitSend(): AppSend | null {
  const handler = (globalThis as { webkit?: { messageHandlers?: { lingua?: ReplyingHandler } } }).webkit
    ?.messageHandlers?.lingua;
  return handler ? (message) => handler.postMessage(message) : null;
}
