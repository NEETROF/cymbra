import { createLinguaPort } from "../analyzer/create-port.ts";
import { mountSettings, type SettingsView } from "../reading/settings-view.ts";
import { ReviewController } from "../review/session.ts";
import { renderReview } from "../review/view.ts";
import { dailyRecorder } from "../state/dailystats.ts";
import { type AsyncStorageArea, hydrateEngine, ROOT_KEY, saveBackup } from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";

/** Transient key the popup sets to open the panel straight on the stats view. */
const PANEL_VIEW_KEY = "cymbra-lingua-panel-view";

// Side-panel controller (a surface the extension owns). The page is pushed by the
// browser and survives navigation. It holds its own engine hydrated from the shared
// backup, runs the FSRS review through the shared ReviewController + renderReview,
// offers lossless backup/restore, and shows the pack's attributions + a privacy note.
// Excluded from coverage (DOM wiring; the review logic is tested in session/view specs).

const area: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

const now = (): number => Math.floor(Date.now() / 1000);
const port = createLinguaPort();
let controller = new ReviewController(port, now, dailyRecorder(area));

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

async function persist(): Promise<void> {
  await saveBackup(area, await port.backup());
}

async function refreshSummary(): Promise<void> {
  const [deck, due] = [await port.deckCount(), await port.dueCount(now())];
  $("summary").innerHTML = `<b>${deck}</b> carte(s) · <b>${due}</b> à revoir`;
}

const actions = {
  start: () => void run(() => controller.start(), false),
  reveal: () => void run(() => controller.reveal(), false),
  grade: (rating: Parameters<typeof controller.grade>[0]) => void run(() => controller.grade(rating), true),
  markKnown: () => void run(() => controller.markKnown(), true),
};

async function run(produce: () => Promise<ReturnType<ReviewController["view"]>>, changed: boolean): Promise<void> {
  const view = await produce();
  if (changed) {
    await persist();
    await refreshSummary();
  }
  renderReview($("review"), view, actions);
}

function download(name: string, text: string): void {
  const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

async function doRestore(file: File): Promise<void> {
  try {
    await port.restore(await file.text());
    await persist();
    controller = new ReviewController(port, now, dailyRecorder(area));
    await refreshSummary();
    renderReview($("review"), controller.view(), actions);
    $("msg").textContent = "Sauvegarde restaurée.";
  } catch {
    $("msg").textContent = "Fichier de sauvegarde non reconnu.";
  }
}

async function loadAttributions(): Promise<void> {
  const licences = await port.licences();
  $("licences").textContent = licences.length ? `Sources : ${licences.join(" · ")}` : "";
  $("notice").textContent = await port.notice();
}

type PanelView = "review" | "stats" | "settings";

let settings: SettingsView | null = null;

/** Switch between the review, stats and settings views; stats and settings are refreshed
 * each time they are shown, so they always reflect the current statuses + level. */
async function showView(view: PanelView): Promise<void> {
  $("view-review").hidden = view !== "review";
  $("view-stats").hidden = view !== "stats";
  $("view-settings").hidden = view !== "settings";
  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.classList.toggle("active", b.dataset.view === view);
  }
  if (view === "stats") await mountStats($("view-stats"), port, area);
  if (view === "settings") {
    // One settings impl, shared with the in-page drawer (mountSettings). A reset re-creates
    // the controller so the review view reflects the wiped deck.
    settings ??= mountSettings($("view-settings"), port, area, {
      persist,
      onReset: async () => {
        controller = new ReviewController(port, now, dailyRecorder(area));
        await refreshSummary();
        renderReview($("review"), controller.view(), actions);
      },
    });
    await settings.refresh();
  }
}

async function main(): Promise<void> {
  await hydrateEngine(port, area);
  await refreshSummary();
  renderReview($("review"), controller.view(), actions);
  await loadAttributions();

  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.addEventListener("click", () => void showView((b.dataset.view as PanelView) ?? "review"));
  }

  // The popup / in-page HUD can request opening straight on the stats or settings view.
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

  $("backup").addEventListener("click", async () => download("cymbra-lingua-backup.json", await port.backup()));
  const fileInput = $("restore-file") as HTMLInputElement;
  $("restore").addEventListener("click", () => fileInput.click());
  fileInput.addEventListener("change", () => {
    const file = fileInput.files?.[0];
    if (file) void doRestore(file);
  });

  // Keep in sync with reading gestures made elsewhere, unless mid-review.
  chrome.storage.onChanged.addListener((changes, areaName) => {
    const root = changes[ROOT_KEY];
    const backup = (root?.newValue as { backup?: string } | undefined)?.backup;
    if (areaName !== "local" || typeof backup !== "string") return;
    if (controller.view().phase === "reviewing") return;
    void port.restore(backup).then(refreshSummary);
  });
}

void main();
