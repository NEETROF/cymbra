import { resolveContentPort } from "./analyzer/create-port.ts";
import { interfaceLanguage } from "./i18n/index.ts";
import { readingCopy } from "./reading/reading-copy.ts";
import { followNativeLanguage } from "./reading/native-rebuild.ts";
import { pageHost, ReadingSession } from "./reading/session.ts";
import { SURFACE_CSS } from "./reading/surface-css.ts";

// Content-script entry: the reading module (reading/session.ts) mounted on the visited page —
// the page is both the document read and the one the popup, drawer and HUD mount in. The
// reader page mounts the same session on each section of a book instead (reader/).

// The reader can arrive two ways — a registered content script (after the <all_urls>
// grant) or an activeTab executeScript from the popup — so guard against running twice.
const GUARD = "__cymbraLinguaReading";

/**
 * Build the page's reading session and start it. The port is resolved before construction so a
 * CSP-blocked page can hand the session the messaging port instead of the in-content WASM engine.
 * The interface language is read beside it, with this script's first storage read, before the
 * session builds the surfaces it hands the copy to (localise-lingua-reading-surfaces D1); a read
 * that fails is French. A session that fails to start is taken down, leaving nothing on the page.
 */
async function startSession(): Promise<ReadingSession> {
  const [port, language] = await Promise.all([
    resolveContentPort(),
    interfaceLanguage({ get: (key) => chrome.storage.local.get(key) }),
  ]);
  const session = new ReadingSession(port, {
    css: SURFACE_CSS,
    surface: "page",
    language,
    copy: readingCopy(language),
  });
  try {
    await session.start(pageHost());
  } catch (e) {
    session.stop();
    throw e;
  }
  return session;
}

async function bootstrap(): Promise<void> {
  const w = window as unknown as Record<string, boolean>;
  if (w[GUARD]) return;
  w[GUARD] = true;
  try {
    // Built anew, reading the key first, when the reader chooses another native language
    // (add-lingua-native-language-choice D3).
    await followNativeLanguage(startSession);
  } catch (e) {
    // Surface a legible failure rather than dying as a silent unhandled rejection,
    // and allow a retry on the next injection.
    console.error("[Cymbra Lingua] reader failed to start:", e);
    w[GUARD] = false;
  }
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => void bootstrap());
} else {
  void bootstrap();
}
