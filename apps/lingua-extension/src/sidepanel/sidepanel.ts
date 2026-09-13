import { createLinguaPort } from "../analyzer/create-port.ts";
import type { CefrLevel } from "../analyzer/types.ts";
import { ReviewController } from "../review/session.ts";
import { renderReview } from "../review/view.ts";
import { dailyRecorder } from "../state/dailystats.ts";
import {
  type AsyncStorageArea,
  hydrateEngine,
  loadHudHidden,
  ROOT_KEY,
  saveBackup,
  saveHudHidden,
} from "../state/storage.ts";
import { mountStats } from "../stats/view.ts";
import { clearSyncCursors } from "../sync/sync.ts";

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
  if (view === "settings") await renderSettings();
}

/** Reflect current settings into the Réglages view (level, calibration, HUD toggle). */
async function renderSettings(): Promise<void> {
  const [hasLevels, declared] = [await port.hasLevels(), await port.declaredLevel()];
  const current = declared ?? "";
  for (const b of document.querySelectorAll<HTMLButtonElement>("#s-level-chips .lvl")) {
    b.classList.toggle("active", (b.dataset.lvl ?? "") === current);
  }
  $("s-level-hint").textContent = declared
    ? `Les mots sous ${declared} ne sont plus surlignés.`
    : "Choisis ton niveau — rien n'est présumé connu pour l'instant.";
  $("s-calib-block").hidden = hasLevels;
  if (!hasLevels) {
    const cal = await port.calibration();
    ($("s-calib") as HTMLInputElement).value = String(cal);
    $("s-calibv").textContent = String(cal);
  }
  ($("s-hud-toggle") as HTMLInputElement).checked = !(await loadHudHidden(area));
}

/** Declare the CEFR level (null = Débutant); mirrors the content script's onSetLevel. */
async function setLevel(level: CefrLevel | null): Promise<void> {
  await port.setDeclaredLevelAt(level, Date.now());
  await port.setCalibration(0);
  await persist();
  await renderSettings();
}

/** Reset local data; mirrors the content script's onReset, driven by the panel's port. */
async function doReset(scope: "full" | "partial"): Promise<void> {
  if (scope === "partial") {
    await port.resetStatuses();
  } else {
    await port.reset();
    await clearSyncCursors(area);
  }
  await port.setCalibration((await port.hasLevels()) ? 0 : 3000);
  await persist();
  controller = new ReviewController(port, now, dailyRecorder(area));
  await refreshSummary();
  renderReview($("review"), controller.view(), actions);
  await renderSettings();
  $("s-reset-msg").textContent = scope === "partial" ? "Statuts et calibration réinitialisés." : "Données effacées.";
}

/** Wire the Réglages controls once (state is refreshed by renderSettings on each show). */
function wireSettings(): void {
  for (const b of document.querySelectorAll<HTMLButtonElement>("#s-level-chips .lvl")) {
    b.addEventListener("click", () => void setLevel((b.dataset.lvl as CefrLevel) || null));
  }
  const calib = $("s-calib") as HTMLInputElement;
  calib.addEventListener("input", () => {
    $("s-calibv").textContent = calib.value;
  });
  calib.addEventListener("change", async () => {
    await port.setCalibration(Number(calib.value));
    await persist();
  });
  ($("s-hud-toggle") as HTMLInputElement).addEventListener("change", async (e) => {
    await saveHudHidden(area, !(e.target as HTMLInputElement).checked);
  });
  $("s-shortcuts-config").addEventListener("click", () => {
    const url = __TARGET__ === "firefox" ? "about:addons" : "chrome://extensions/shortcuts";
    void chrome.tabs.create({ url });
  });

  // Reset wizard: Réinitialiser… → scope choice; a full wipe needs an extra confirm.
  const showReset = (menu: boolean, confirm: boolean): void => {
    $("s-reset").hidden = menu || confirm;
    $("s-reset-menu").hidden = !menu;
    $("s-reset-confirm").hidden = !confirm;
  };
  $("s-reset").addEventListener("click", () => showReset(true, false));
  $("s-reset-cancel").addEventListener("click", () => showReset(false, false));
  $("s-reset-partial").addEventListener("click", () => {
    showReset(false, false);
    void doReset("partial");
  });
  $("s-reset-full").addEventListener("click", () => {
    $("s-reset-warn").textContent = "Effacer statuts, deck de révision et progression ? Action définitive hors sync.";
    showReset(false, true);
  });
  $("s-reset-no").addEventListener("click", () => showReset(false, false));
  $("s-reset-yes").addEventListener("click", () => {
    showReset(false, false);
    void doReset("full");
  });
}

async function main(): Promise<void> {
  await hydrateEngine(port, area);
  await refreshSummary();
  renderReview($("review"), controller.view(), actions);
  await loadAttributions();

  for (const b of document.querySelectorAll<HTMLButtonElement>("#views button")) {
    b.addEventListener("click", () => void showView((b.dataset.view as PanelView) ?? "review"));
  }
  wireSettings();

  // The popup / in-page HUD can request opening straight on the stats or settings view.
  try {
    const sess = await chrome.storage.session.get(PANEL_VIEW_KEY);
    const requested = sess[PANEL_VIEW_KEY];
    if (requested === "stats" || requested === "settings") {
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
