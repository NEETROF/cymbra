import { type StoreChangeReason, watchStore } from "../state/store.ts";

// The content script's reading session follows the reader's native language
// (add-lingua-native-language-choice D3). A page being read is never reloaded for it — the reader
// would lose their place, and the page what it was doing — so the session is taken down and built
// anew instead: a new engine port, which resolves the new native language from the store, and the
// interface language read again first (localise-lingua-reading-surfaces D1), so the HUD, the drawer,
// the word card and the selection card come back in the new language with their hosts' `lang`. A
// synced backup is still the session's own to restore (`onExternalChange`); a change of native
// language, announced with its reason, is the one case that rebuilds.

/** What the content script builds, and takes down: its reading session. */
export interface PageSession {
  stop(): void;
}

/**
 * Start a session with `build`, then build it anew on every announced change of native language,
 * one after the other: the previous session is stopped before the next is built, and a build that
 * failed is followed by a fresh one on the next change. Resolves once the first session started;
 * rejects when it could not — the content script says so, and the next injection retries.
 */
export async function followNativeLanguage(
  build: () => Promise<PageSession>,
  watch: (onChanged: (keys: string[], reason?: StoreChangeReason) => void) => unknown = watchStore,
): Promise<void> {
  let session: Promise<PageSession> = build();
  await session;
  watch((_keys, reason) => {
    if (reason?.type !== "native-language") return;
    session = session.then(
      (previous) => {
        previous.stop();
        return build();
      },
      () => build(),
    );
    session.catch((e: unknown) => {
      console.error("[Cymbra Lingua] reader failed to start again in the new language:", e);
    });
  });
}
