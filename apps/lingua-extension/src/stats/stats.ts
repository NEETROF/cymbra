import { createLinguaPort } from "../analyzer/create-port.ts";
import { messagedArea } from "../state/store.ts";
import { start } from "./page.ts";
import { followSurfaceLook } from "../reading/surface-look.ts";

// This page is a surface: it follows the reader's colours and text size (add-lingua-colour-settings D8, D9).
followSurfaceLook(document.documentElement);

// Standalone stats tab. The primary surface is now the side panel (same view via
// mountStats); this tab remains reachable and hydrates its own engine from the shared
// backup — the reader's data from the area the background owns, the interface language from
// chrome.storage.local (page.ts). Excluded from coverage (DOM wiring; page.ts is tested).

void start(document, {
  prefs: { get: (keys) => chrome.storage.local.get(keys) },
  store: messagedArea(),
  port: createLinguaPort(),
});
