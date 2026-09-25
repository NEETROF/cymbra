import { installGroupBy } from "./polyfill.ts";
import { createLinguaPort } from "../analyzer/create-port.ts";
import { ReadingSession } from "../reading/session.ts";
import { SURFACE_CSS } from "../reading/surface-css.ts";
import {
  type AsyncStorageArea,
  loadReaderFlow,
  READER_DISPLAY_KEY,
  READER_FLOW_KEY,
  readerDisplayOf,
  readerFlowOf,
} from "../state/storage.ts";
import { ReaderApp } from "./app.ts";
import { FoliateRenderer } from "./foliate.ts";
import { Library } from "./library.ts";
import { isReaderWhere, type ReaderWhereReply } from "./locate.ts";
import { requestPersistence } from "./persist.ts";
import { clearSections } from "./section-server.ts";

// The reader page's entry (add-lingua-reader): an extension page, so it loads from the
// installed bundle and never from the network; the engine is the one every extension page
// uses (createLinguaPort — in the page on Chromium, in the event page on Firefox and Safari).
// It hosts ONE reading session, attached to each section foliate-js renders; the popup, the
// drawer and the toolbar live in this page's own document. Excluded from coverage like every
// entry point: the page it wires (app.ts) is measured.

/** Preferences and marks: chrome.storage.local, as every surface reads them. */
const settings: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

/** Remembers that the library asked the browser to keep it (it asks once). */
const PERSIST_ASKED_KEY = "cymbra-lingua-library-persist-asked";

async function main(): Promise<void> {
  installGroupBy();
  const root = document.getElementById("reader-root");
  if (!root) return;
  if (__SECTIONS_FROM_WORKER__) await clearSections(caches);
  const library = await Library.open();
  const app = new ReaderApp(root, {
    library,
    createRenderer: () => new FoliateRenderer(),
    persistence: () =>
      requestPersistence({
        asked: async () => (await settings.get(PERSIST_ASKED_KEY))[PERSIST_ASKED_KEY] === true,
        markAsked: () => settings.set({ [PERSIST_ASKED_KEY]: true }),
      }),
    loadFlow: () => loadReaderFlow(settings),
    watchFlow: (onFlow) =>
      chrome.storage.onChanged.addListener((changes, area) => {
        if (area === "local" && changes[READER_FLOW_KEY]) onFlow(readerFlowOf(changes[READER_FLOW_KEY].newValue));
      }),
    displayArea: settings,
    watchDisplay: (onDisplay) =>
      chrome.storage.onChanged.addListener((changes, area) => {
        const changed = changes[READER_DISPLAY_KEY];
        if (area === "local" && changed) onDisplay(readerDisplayOf(changed.newValue));
      }),
  });

  // Answer the background looking for an open reader, so a second "Bibliothèque" focuses
  // this tab instead of opening another.
  const here = await chrome.tabs.getCurrent();
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!isReaderWhere(message) || here?.id == null) return false;
    sendResponse({ tabId: here.id, windowId: here.windowId } satisfies ReaderWhereReply);
    return false;
  });

  const session = new ReadingSession(createLinguaPort(), {
    css: SURFACE_CSS,
    surface: "book",
    indicator: (actions) => app.indicator(actions),
    onPainted: () => app.painted(),
    onBlankClick: (e) => app.blankClick(e),
  });
  await session.start(null);
  await app.start(session);
}

void main();
