import type { LinguaPort } from "../analyzer/port.ts";
import { dailyRecorder } from "../state/dailystats.ts";
import { type AsyncStorageArea, ROOT_KEY, saveBackup } from "../state/storage.ts";
import { ReviewController } from "./session.ts";
import { type ReviewActions, renderReview } from "./view.ts";

// The full Révision page — summary + the FSRS review widget + lossless backup/restore +
// the pack's Sources & confidentialité — built as plain DOM into a container so ONE
// implementation serves both hosts: the native side panel and the in-page drawer (same
// pattern as mountSettings / mountStats). It owns a ReviewController on the host's port and
// persists via saveBackup, so every other surface reacts through storage.onChanged.

export interface ReviewPageOptions {
  /** Epoch-seconds clock (Date.now()/1000 in production). */
  now: () => number;
}

export interface ReviewPage {
  /** Re-sync the summary + review view from the engine (call each time it is shown). */
  refresh: () => Promise<void>;
  /** Whether a review session is under way (a restore would end it). */
  reviewing: () => boolean;
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, className?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  return node;
}

/** Mount the Révision page into `container`. */
export function mountReview(
  container: HTMLElement,
  port: LinguaPort,
  area: AsyncStorageArea,
  opts: ReviewPageOptions,
): ReviewPage {
  container.replaceChildren();
  let controller = new ReviewController(port, opts.now, dailyRecorder(area));
  let lastBackup: string | null = null;

  const summary = el("div", "summary");
  const review = el("div", "review");

  const tools = el("div", "tools");
  const backupBtn = el("button");
  backupBtn.type = "button";
  backupBtn.textContent = "Sauvegarder";
  const restoreBtn = el("button");
  restoreBtn.type = "button";
  restoreBtn.textContent = "Restaurer";
  const fileInput = el("input");
  fileInput.type = "file";
  fileInput.accept = "application/json";
  fileInput.hidden = true;
  tools.append(backupBtn, restoreBtn, fileInput);
  const msg = el("div", "msg");

  const details = el("details");
  const detailsSummary = el("summary");
  detailsSummary.textContent = "Sources & confidentialité";
  const privacy = el("div", "privacy");
  privacy.textContent = "Rien ne quitte votre appareil : l'analyse et les traductions sont locales.";
  const licences = el("div");
  const notice = el("pre", "notice");
  details.append(detailsSummary, privacy, licences, notice);

  container.append(summary, review, tools, msg, details);

  async function persist(): Promise<void> {
    const backup = await port.backup();
    lastBackup = backup; // ignore our own storage.onChanged echo
    await saveBackup(area, backup);
  }

  async function refreshSummary(): Promise<void> {
    const [deck, due] = [await port.deckCount(), await port.dueCount(opts.now())];
    summary.replaceChildren(
      bold(String(deck)),
      document.createTextNode(" carte(s) · "),
      bold(String(due)),
      document.createTextNode(" à revoir"),
    );
  }

  async function run(produce: () => Promise<ReturnType<ReviewController["view"]>>, changed: boolean): Promise<void> {
    const view = await produce();
    if (changed) {
      await persist();
      await refreshSummary();
    }
    renderReview(review, view, actions);
  }

  const actions: ReviewActions = {
    start: () => void run(() => controller.start(), false),
    reveal: () => void run(() => controller.reveal(), false),
    grade: (rating) => void run(() => controller.grade(rating), true),
    markKnown: () => void run(() => controller.markKnown(), true),
  };

  function download(name: string, text: string): void {
    const url = URL.createObjectURL(new Blob([text], { type: "application/json" }));
    const a = el("a");
    a.href = url;
    a.download = name;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  async function doRestore(file: File): Promise<void> {
    try {
      await port.restore(await file.text());
      await persist();
      controller = new ReviewController(port, opts.now, dailyRecorder(area));
      await refreshSummary();
      renderReview(review, controller.view(), actions);
      msg.textContent = "Sauvegarde restaurée.";
    } catch {
      msg.textContent = "Fichier de sauvegarde non reconnu.";
    }
  }

  backupBtn.addEventListener("click", async () => download("cymbra-lingua-backup.json", await port.backup()));
  restoreBtn.addEventListener("click", () => fileInput.click());
  fileInput.addEventListener("change", () => {
    const file = fileInput.files?.[0];
    if (file) void doRestore(file);
  });

  void loadAttributions();
  async function loadAttributions(): Promise<void> {
    const names = await port.licences();
    licences.textContent = names.length ? `Sources : ${names.join(" · ")}` : "";
    notice.textContent = await port.notice();
  }

  // Keep in sync with changes made elsewhere (a reading gesture, a reset in Réglages, or
  // another surface), unless mid-review or it is our own echo. The controller caches its
  // state, so it is rebuilt to reflect the restored engine.
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName !== "local") return;
    const backup = (changes[ROOT_KEY]?.newValue as { backup?: string } | undefined)?.backup;
    if (typeof backup !== "string" || backup === lastBackup) return;
    if (controller.view().phase === "reviewing") return;
    void port.restore(backup).then(() => {
      controller = new ReviewController(port, opts.now, dailyRecorder(area));
      void refreshSummary();
      renderReview(review, controller.view(), actions);
    });
  });

  return {
    refresh: async () => {
      await refreshSummary();
      renderReview(review, controller.view(), actions);
    },
    reviewing: () => controller.view().phase === "reviewing",
  };
}

function bold(text: string): HTMLElement {
  const b = document.createElement("b");
  b.textContent = text;
  return b;
}
