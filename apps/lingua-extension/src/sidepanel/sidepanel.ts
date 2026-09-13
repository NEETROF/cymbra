import { createLinguaPort } from "../analyzer/create-port.ts";
import { mountSettings, type SettingsView } from "../reading/settings-view.ts";
import { mountReview, type ReviewPage } from "../review/review-page.ts";
import { type AsyncStorageArea, hydrateEngine, saveBackup } from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";

/** Transient key the popup / HUD set to open the panel straight on a view. */
const PANEL_VIEW_KEY = "cymbra-lingua-panel-view";

// Side-panel controller (a surface the extension owns). The page is pushed by the browser
// and survives navigation. It hydrates its own engine from the shared backup and renders
// the three views (Révision / Statistiques / Réglages) through the SAME modules the in-page
// drawer uses — mountReview / mountStats / mountSettings (one impl, two hosts). Excluded
// from coverage (DOM wiring; the logic is tested in the shared modules' specs).

const area: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

const now = (): number => Math.floor(Date.now() / 1000);
const port = createLinguaPort();

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

async function persist(): Promise<void> {
  await saveBackup(area, await port.backup());
}

type PanelView = "review" | "stats" | "settings";

let review: ReviewPage | null = null;
let settings: SettingsView | null = null;

/** Switch views; each is (re)mounted/refreshed on show so it reflects the current state. */
async function showView(view: PanelView): Promise<void> {
  $("view-review").hidden = view !== "review";
  $("view-stats").hidden = view !== "stats";
  $("view-settings").hidden = view !== "settings";
  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.classList.toggle("active", b.dataset.view === view);
  }
  if (view === "review") {
    review ??= mountReview($("view-review"), port, area, { now });
    await review.refresh();
  } else if (view === "stats") {
    await mountStats($("view-stats"), port, area);
  } else {
    settings ??= mountSettings($("view-settings"), port, area, { persist });
    await settings.refresh();
  }
}

async function main(): Promise<void> {
  await hydrateEngine(port, area);
  await showView("review");

  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.addEventListener("click", () => void showView((b.dataset.view as PanelView) ?? "review"));
  }

  // The popup / in-page HUD can request opening straight on a view.
  try {
    const sess = await chrome.storage.session.get(PANEL_VIEW_KEY);
    const requested = sess[PANEL_VIEW_KEY];
    if (requested === "review" || requested === "stats" || requested === "settings") {
      await chrome.storage.session.remove(PANEL_VIEW_KEY);
      await showView(requested);
    }
  } catch {
    /* storage.session may be unavailable; default to the review view */
  }
}

void main();
