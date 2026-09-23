import { resolveContentPort } from "./analyzer/create-port.ts";
import { pageHost, ReadingSession } from "./reading/session.ts";
import { SURFACE_CSS } from "./reading/surface-css.ts";

// Content-script entry: the reading module (reading/session.ts) mounted on the visited page —
// the page is both the document read and the one the popup, drawer and HUD mount in. The
// reader page mounts the same session on each section of a book instead (reader/).

// The reader can arrive two ways — a registered content script (after the <all_urls>
// grant) or an activeTab executeScript from the popup — so guard against running twice.
const GUARD = "__cymbraLinguaReading";

async function bootstrap(): Promise<void> {
  const w = window as unknown as Record<string, boolean>;
  if (w[GUARD]) return;
  w[GUARD] = true;
  try {
    // The port is resolved before construction so a CSP-blocked page can hand the session
    // the messaging port instead of the in-content WASM engine.
    const port = await resolveContentPort();
    await new ReadingSession(port, { css: SURFACE_CSS, surface: "page" }).start(pageHost());
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
