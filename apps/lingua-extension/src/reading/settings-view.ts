import type { LinguaPort } from "../analyzer/port.ts";
import { CEFR_LEVELS, type CefrLevel } from "../analyzer/types.ts";
import { needsLevelChoice } from "../state/level-choice.ts";
import { hasShortcutEditor } from "../state/platform.ts";
import { type AsyncStorageArea, loadHudHidden, saveHudHidden } from "../state/storage.ts";
import {
  LAST_SYNC_KEY,
  loadLastSync,
  syncAvailable,
  syncNow as requestSyncNow,
  type SyncReply,
} from "../sync/messages.ts";
import { lastSyncLabel, syncErrorCopy } from "../sync/status.ts";
import { clearSyncCursors } from "../sync/sync.ts";

// The Réglages view, built as plain DOM into a given container so ONE implementation
// serves two hosts: the native side panel and the in-page drawer (same pattern as review's
// renderReview). It drives the host's own port and persists via `persist`, so every other
// surface reacts through storage.onChanged; `onReset` lets the host refresh its review
// after a wipe. Level + calibration mirror the content script's onSetLevel / onReset.

export interface SettingsOptions {
  /** Persist engine state after a change (backup → storage). */
  persist: () => Promise<void>;
  /** Refresh the host's review/summary after a reset (the deck may have changed). */
  onReset?: () => Promise<void> | void;
  /** The reader's data, for the reset that clears the sync cursors. */
  store: AsyncStorageArea;
  /** The Synchronisation controls' seam; the background messages by default. */
  sync?: SyncControls;
}

/** What the Synchronisation block needs from the background and the store. */
export interface SyncControls {
  /** Whether a Cymbra account is signed in (the block shows only then). */
  available: () => Promise<boolean>;
  /** Run one exchange now. */
  syncNow: () => Promise<SyncReply>;
  /** This device's last successful sync (epoch millis), or null. */
  lastSync: () => Promise<number | null>;
  now: () => number;
  /** Call `onChange` whenever a sync completes elsewhere (the background). */
  watch: (onChange: () => void) => void;
}

function runtimeSyncControls(area: AsyncStorageArea): SyncControls {
  return {
    available: () => syncAvailable(),
    syncNow: () => requestSyncNow(),
    lastSync: () => loadLastSync(area),
    now: () => Date.now(),
    watch: (onChange) =>
      chrome.storage.onChanged.addListener((changes, areaName) => {
        if (areaName === "local" && changes[LAST_SYNC_KEY]) onChange();
      }),
  };
}

export interface SettingsView {
  /** Re-sync the controls with the engine (call each time the view is shown). */
  refresh: () => Promise<void>;
}

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text != null) node.textContent = text;
  return node;
}

function settingBlock(label: string): HTMLDivElement {
  const block = el("div", "set-block");
  block.append(el("div", "set-label", label));
  return block;
}

function shortcut(...parts: (string | HTMLElement)[]): HTMLLIElement {
  const li = el("li");
  li.append(...parts);
  return li;
}

function kbd(key: string): HTMLElement {
  return el("kbd", undefined, key);
}

/** Mount the Réglages controls into `container`. Returns a `refresh()` to re-sync state. */
export function mountSettings(
  container: HTMLElement,
  port: LinguaPort,
  area: AsyncStorageArea,
  opts: SettingsOptions,
): SettingsView {
  container.replaceChildren();

  // — Niveau d'anglais —
  const levelBlock = settingBlock("Niveau d'anglais");
  const chips = el("div", "level-chips");
  const chipButtons = new Map<string, HTMLButtonElement>();
  const addChip = (value: string, label: string): void => {
    const b = el("button", value === "" ? "lvl lvl-beginner" : "lvl", label);
    b.type = "button";
    b.dataset.lvl = value;
    b.addEventListener("click", () => void setLevel((value as CefrLevel) || null));
    chips.append(b);
    chipButtons.set(value, b);
  };
  for (const lvl of CEFR_LEVELS) addChip(lvl, lvl);
  addChip("", "Débutant");
  const hint = el("div", "set-note");
  const calibBlock = el("div", "calib");
  calibBlock.hidden = true;
  const calibValue = el("b", undefined, "3000");
  const calibLabel = el("label");
  calibLabel.append("Je connais les ", calibValue, " mots les plus courants");
  const calib = el("input");
  calib.type = "range";
  calib.min = "500";
  calib.max = "10000";
  calib.step = "100";
  calib.value = "3000";
  calibBlock.append(calibLabel, calib);
  levelBlock.append(chips, hint, calibBlock);

  // — Barre sur la page —
  const barBlock = settingBlock("Barre sur la page");
  const toggleRow = el("label", "set-toggle");
  const toggle = el("input");
  toggle.type = "checkbox";
  toggleRow.append(toggle, el("span", undefined, "Afficher la pastille de pourcentage"));
  barBlock.append(
    toggleRow,
    el("div", "set-note", "Pastille discrète en bas de la page : pourcentage + accès au deck et aux réglages."),
  );

  // — Raccourcis & gestes —
  const scBlock = settingBlock("Raccourcis & gestes");
  const scList = el("ul", "set-shortcuts");
  scList.append(
    shortcut(kbd("Alt"), "+", kbd("Maj"), "+", kbd("S"), " — panneau latéral"),
    shortcut(kbd("Alt"), "+", kbd("Maj"), "+", kbd("D"), " — panneau de révision sur la page"),
    shortcut(kbd("Alt"), "+", kbd("L"), " — capturer la sélection"),
    shortcut(kbd("Alt"), "/", kbd("Option"), "-clic (ou appui long) sur un mot — le reclasser"),
  );
  const scConfig = el("button", "linklike", "Configurer les raccourcis du navigateur");
  scConfig.type = "button";
  scConfig.addEventListener("click", () => {
    const url = __TARGET__ === "firefox" ? "about:addons" : "chrome://extensions/shortcuts";
    void chrome.tabs.create({ url });
  });
  scBlock.append(scList);
  if (hasShortcutEditor()) scBlock.append(scConfig);

  // — Synchronisation (signed in only): when this device last synced, and a manual run —
  const sync = opts.sync ?? runtimeSyncControls(area);
  const syncBlock = settingBlock("Synchronisation");
  syncBlock.hidden = true;
  const syncStatus = el("div", "set-note");
  const syncBtn = el("button", "set-reset", "Synchroniser maintenant");
  syncBtn.type = "button";
  const syncMsg = el("div", "set-note");
  syncBlock.append(syncStatus, syncBtn, syncMsg);

  // — Réinitialisation (scope choice; a full wipe needs an extra confirm) —
  const resetBlock = settingBlock("Réinitialisation");
  const localNote = el("div", "set-note", "Efface tes données locales. À n'utiliser qu'exceptionnellement.");
  resetBlock.append(localNote);
  const resetBtn = el("button", "set-reset", "Réinitialiser…");
  resetBtn.type = "button";
  const menu = el("div");
  menu.hidden = true;
  const resetPartial = el("button", "set-reset", "Partielle — statuts + calibration (garde le deck)");
  resetPartial.type = "button";
  const resetFull = el("button", "set-danger", "Complète — tout effacer");
  resetFull.type = "button";
  const resetCancel = el("button", "set-reset", "Annuler");
  resetCancel.type = "button";
  menu.append(resetPartial, resetFull, resetCancel);
  const confirm = el("div");
  confirm.hidden = true;
  const warn = el("div", "set-warn");
  const confirmYes = el("button", "set-danger", "Oui, confirmer");
  confirmYes.type = "button";
  const confirmNo = el("button", "set-reset", "Annuler");
  confirmNo.type = "button";
  confirm.append(warn, confirmYes, confirmNo);
  const resetMsg = el("div", "set-note");

  // Signed in, erasing this device erases nothing: the next exchange pulls it all back —
  // it even clears the cursors, which guarantees the return. So the same action is offered
  // for what it actually is, and a real erasure is pointed at where it lives.
  const localOnly = el("div");
  localOnly.append(resetBtn, menu, confirm);
  const synced = el("div");
  synced.hidden = true;
  const restartNote = el(
    "div",
    "set-note",
    "Tes données sont sur ton compte. Vider cet appareil n'efface rien : la synchronisation les ramène. À utiliser si l'état local semble faux.",
  );
  const restartBtn = el("button", "set-reset", "Repartir du serveur");
  restartBtn.type = "button";
  const eraseNote = el(
    "div",
    "set-note",
    "Pour effacer partout et définitivement, utilise « Effacer mes données Lingua » dans ton compte.",
  );
  const eraseLink = el("button", "linklike", "Gérer mes données");
  eraseLink.type = "button";
  eraseLink.addEventListener("click", () => {
    void chrome.tabs.create({ url: chrome.runtime.getURL("account.html#data") });
  });
  synced.append(restartNote, restartBtn, eraseNote, eraseLink);
  resetBlock.append(localOnly, synced, resetMsg);

  const showReset = (showMenu: boolean, showConfirm: boolean): void => {
    resetBtn.hidden = showMenu || showConfirm;
    menu.hidden = !showMenu;
    confirm.hidden = !showConfirm;
  };
  resetBtn.addEventListener("click", () => showReset(true, false));
  resetCancel.addEventListener("click", () => showReset(false, false));
  confirmNo.addEventListener("click", () => showReset(false, false));
  resetPartial.addEventListener("click", () => {
    showReset(false, false);
    void doReset("partial");
  });
  resetFull.addEventListener("click", () => {
    warn.textContent = "Effacer statuts, deck de révision et progression ? Action définitive hors sync.";
    showReset(false, true);
  });
  confirmYes.addEventListener("click", () => {
    showReset(false, false);
    void doReset("full");
  });

  container.append(levelBlock, barBlock, scBlock, syncBlock, resetBlock);

  // — Live wiring —
  calib.addEventListener("input", () => {
    calibValue.textContent = calib.value;
  });
  calib.addEventListener("change", async () => {
    await port.setCalibration(Number(calib.value));
    await opts.persist();
  });
  toggle.addEventListener("change", async () => {
    await saveHudHidden(area, !toggle.checked);
  });
  syncBtn.addEventListener("click", () => void runSync());
  restartBtn.addEventListener("click", () => void restartFromServer());
  sync.watch(() => void refreshSync());

  async function runSync(): Promise<void> {
    syncBtn.disabled = true;
    syncMsg.textContent = "Synchronisation…";
    const reply = await sync.syncNow();
    syncMsg.textContent = reply.ok ? "" : syncErrorCopy(reply.error);
    syncBtn.disabled = false;
    await refreshSync();
  }

  async function refreshSync(): Promise<void> {
    const signedIn = await sync.available();
    syncBlock.hidden = !signedIn;
    localNote.hidden = signedIn;
    localOnly.hidden = signedIn;
    synced.hidden = !signedIn;
    syncStatus.textContent = lastSyncLabel(await sync.lastSync(), sync.now());
  }

  /** Empty this device and pull the account's state back. Nothing is lost by design. */
  async function restartFromServer(): Promise<void> {
    restartBtn.disabled = true;
    resetMsg.textContent = "Reprise depuis le serveur…";
    await doReset("full");
    const reply = await sync.syncNow();
    resetMsg.textContent = reply.ok ? "Repris depuis le serveur." : syncErrorCopy(reply.error);
    restartBtn.disabled = false;
    await refreshSync();
  }

  async function setLevel(level: CefrLevel | null): Promise<void> {
    await port.setDeclaredLevelAt(level, Date.now());
    await port.setCalibration(0);
    await opts.persist();
    await refresh();
  }

  async function doReset(scope: "full" | "partial"): Promise<void> {
    if (scope === "partial") {
      await port.resetStatuses();
    } else {
      await port.reset();
      await clearSyncCursors(opts.store);
    }
    await port.setCalibration((await port.hasLevels()) ? 0 : 3000);
    await opts.persist();
    await opts.onReset?.();
    await refresh();
    if (!synced.hidden) return; // the signed-in path writes its own message
    resetMsg.textContent = scope === "partial" ? "Statuts et calibration réinitialisés." : "Données effacées.";
  }

  async function refresh(): Promise<void> {
    const [hasLevels, declared, needsChoice] = [
      await port.hasLevels(),
      await port.declaredLevel(),
      await needsLevelChoice(port),
    ];
    // Nothing is highlighted as chosen until a decision exists (« Débutant » is one).
    const current = needsChoice ? null : (declared ?? "");
    for (const [value, b] of chipButtons) b.classList.toggle("active", value === current);
    hint.textContent = declared
      ? `Les mots sous ${declared} ne sont plus surlignés.`
      : needsChoice
        ? "Choisis ton niveau — rien n'est présumé connu pour l'instant."
        : "Débutant — rien n'est présumé connu.";
    calibBlock.hidden = hasLevels;
    if (!hasLevels) {
      const cal = await port.calibration();
      calib.value = String(cal);
      calibValue.textContent = String(cal);
    }
    toggle.checked = !(await loadHudHidden(area));
    await refreshSync();
  }

  void refresh();
  return { refresh };
}
