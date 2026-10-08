import { createLinguaPort } from "../analyzer/create-port.ts";
import { fillPageInLanguage, type InterfaceLanguageArea } from "../i18n/index.ts";
import { type AsyncStorageArea, hydrateEngine } from "../state/storage.ts";
import { messagedArea } from "../state/store.ts";
import { mountStats, statsCopy } from "./view.ts";
import { followSurfaceLook } from "../reading/surface-look.ts";

// This page is a surface: it follows the reader's colours and text size (add-lingua-colour-settings D8, D9).
followSurfaceLook(document.documentElement);

// Standalone stats tab. The primary surface is now the side panel (same view via
// mountStats); this tab remains reachable and hydrates its own engine from the shared
// backup. Excluded from coverage (DOM wiring; the model + chart are unit-tested).

/** The reader's data, owned by the background — the same area every other surface asks. */
const store: AsyncStorageArea = messagedArea();

/** Preferences: chrome.storage.local, where the interface language is kept, as every surface reads it. */
const prefs: InterfaceLanguageArea = { get: (keys) => chrome.storage.local.get(keys) };

async function main(): Promise<void> {
  // The interface language first, with this page's first storage read: the page's static copy is
  // filled from the catalogue before anything shows (the body is hidden until then), and the view
  // below speaks the same language. A read that fails is French: the page shows.
  const { language } = await fillPageInLanguage(document, prefs, statsCopy);
  const root = document.getElementById("stats-root");
  if (!root) return;
  const port = createLinguaPort();
  await hydrateEngine(port, store);
  await mountStats(root, port, store, undefined, language);
}

void main();
