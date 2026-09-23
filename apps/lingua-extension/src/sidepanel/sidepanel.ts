import { createLinguaPort } from "../analyzer/create-port.ts";
import { STUDIED_LANGUAGE } from "../analyzer/types.ts";
import { mountSettings, type SettingsView } from "../reading/settings-view.ts";
import { browserSpeechEngine, createSpeaker } from "../reading/speech.ts";
import { mountReview, type ReviewPage } from "../review/review-page.ts";
import { type AsyncStorageArea, hydrateEngine, saveBackup, storedVoicePreference } from "../state/storage.ts";
import { messagedArea, watchBackup } from "../state/store.ts";
import { mountStats } from "../stats/view.ts";
import { requestSync } from "../sync/messages.ts";

/** Transient key the popup / HUD set to open the panel straight on a view. */
const PANEL_VIEW_KEY = "cymbra-lingua-panel-view";

// Side-panel controller (a surface the extension owns). The page is pushed by the browser
// and survives navigation. It hydrates its own engine from the shared backup and renders
// the three views (Révision / Statistiques / Réglages) through the SAME modules the in-page
// drawer uses — mountReview / mountStats / mountSettings (one impl, two hosts). Excluded
// from coverage (DOM wiring; the logic is tested in the shared modules' specs).

/** Preferences and marks (the HUD toggle, the last-sync time). */
const area: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

/** The reader's data, owned by the background (change: move-lingua-store-to-indexeddb). */
const store: AsyncStorageArea = messagedArea();

const now = (): number => Math.floor(Date.now() / 1000);
const port = createLinguaPort();
/** This page's own synthesiser, for the Réglages voice preview. */
const speaker = createSpeaker(browserSpeechEngine(), STUDIED_LANGUAGE, storedVoicePreference(area));

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

/** The last backup this panel wrote, so its own storage.onChanged echo is ignored. */
let lastBackup: string | null = null;

async function persist(): Promise<void> {
  lastBackup = await port.backup();
  await saveBackup(store, lastBackup);
}

type PanelView = "review" | "stats" | "settings";

let review: ReviewPage | null = null;
let settings: SettingsView | null = null;
let current: PanelView = "review";

/** Switch views; each is (re)mounted/refreshed on show so it reflects the current state. */
async function showView(view: PanelView): Promise<void> {
  current = view;
  $("view-review").hidden = view !== "review";
  $("view-stats").hidden = view !== "stats";
  $("view-settings").hidden = view !== "settings";
  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.classList.toggle("active", b.dataset.view === view);
  }
  if (view === "review") {
    review ??= mountReview($("view-review"), port, store, { now });
    await review.refresh();
  } else if (view === "stats") {
    await mountStats($("view-stats"), port, store);
  } else {
    settings ??= mountSettings($("view-settings"), port, area, { persist, store, speaker });
    await settings.refresh();
  }
}

async function main(): Promise<void> {
  await hydrateEngine(port, store);
  await showView("review");

  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.addEventListener("click", () => void showView((b.dataset.view as PanelView) ?? "review"));
  }

  // The popup / in-page HUD request a view through this flag. Read it on load AND react to
  // it changing while the panel is already open — chrome.sidePanel.open() on an open panel
  // is a no-op, so without this listener clicking "Réviser" while on Statistiques would not
  // switch the view.
  const applyRequestedView = (v: unknown): void => {
    if (v === "review" || v === "stats" || v === "settings") {
      void chrome.storage.session.remove(PANEL_VIEW_KEY);
      void showView(v);
    }
  };
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName === "session" && changes[PANEL_VIEW_KEY]) applyRequestedView(changes[PANEL_VIEW_KEY].newValue);
  });
  // A change made elsewhere (a reading gesture, a sync pull): Révision follows it on its
  // own; Statistiques and Réglages are redrawn from the restored engine, unless a review
  // is under way in the hidden Révision view.
  watchBackup(store, (backup) => {
    if (backup === lastBackup) return;
    if (current === "review" || review?.reviewing()) return;
    void port.restore(backup).then(() => showView(current));
  });
  void requestSync("surface");
  try {
    const sess = await chrome.storage.session.get(PANEL_VIEW_KEY);
    applyRequestedView(sess[PANEL_VIEW_KEY]);
  } catch {
    /* storage.session may be unavailable; default to the review view */
  }
}

void main();
