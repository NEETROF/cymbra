import type { CefrLevel } from "../analyzer/types.ts";
import { loadEnabled, saveEnabled } from "../state/storage.ts";

// Icon-popup controller (a surface the extension owns). It holds no engine and no
// storage of its own: it asks the active tab's content script for stats and drives
// calibration / reset / review through messages, so the content script (which owns the
// engine and persists) stays the single writer. The one thing it writes directly is the
// global enabled flag (a plain setting, not engine state); content scripts react to it
// via storage.onChanged. Excluded from coverage (DOM wiring).

const storageArea = {
  get: (keys: string | string[] | null) => chrome.storage.local.get(keys),
  set: (items: Record<string, unknown>) => chrome.storage.local.set(items),
};

interface PageStats {
  analysable: boolean;
  percent: number | null;
  counted: number;
  unknownOccurrences: number;
  distinctUnknown: number;
  calibration: number;
  declaredLevel: CefrLevel | null;
  hasLevels: boolean;
  trackedCount: number;
  deckCount: number;
  dueCount: number;
}

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

async function activeTabId(): Promise<number | undefined> {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return tab?.id;
}

async function send(message: unknown): Promise<unknown> {
  const tabId = await activeTabId();
  if (tabId == null) return null;
  try {
    return await chrome.tabs.sendMessage(tabId, message);
  } catch {
    return null; // no content script on this tab
  }
}

/** Message the background (account/session lives there), tolerating an asleep worker. */
async function sendRuntime(message: unknown): Promise<unknown> {
  try {
    return await chrome.runtime.sendMessage(message);
  } catch {
    return null;
  }
}

interface AccountState {
  signedIn: boolean;
}
interface AccountResult {
  ok: boolean;
  error?: string;
  state?: AccountState;
}

/** Whether a Cymbra ID session is active — decides the reset warning's wording. */
let accountSignedIn = false;

function renderAccount(state: AccountState | null): void {
  const signedIn = state?.signedIn ?? false;
  accountSignedIn = signedIn;
  $("acct-in").hidden = !signedIn;
  $("acct-out").hidden = signedIn;
  $("acct-error").hidden = true;
}

function showAccountError(message: string): void {
  const el = $("acct-error");
  el.textContent = message;
  el.hidden = false;
}

function render(stats: PageStats | null): void {
  const present = stats !== null;
  $("setup").hidden = present;
  $("controls").hidden = !present;
  if (!stats) return;

  $("analysed").hidden = !stats.analysable;
  $("note").hidden = stats.analysable;
  if (stats.analysable) {
    const pct = stats.percent ?? 0;
    $("pct").textContent = stats.percent == null ? "—" : `${pct}%`;
    ($("bar") as HTMLElement).style.width = `${pct}%`;
    $("counted").textContent = String(stats.counted);
    $("unknown").textContent = String(stats.unknownOccurrences);
    $("distinct").textContent = String(stats.distinctUnknown);
  }

  $("tracked").textContent = String(stats.trackedCount);
  $("deck").textContent = String(stats.deckCount);
  $("due").textContent = String(stats.dueCount);

  // With CEFR data, the reader declares a level (the frequency slider is the
  // fallback for language packs without CEFR levels).
  if (stats.hasLevels) {
    $("level-block").hidden = false;
    $("calib-block").hidden = true;
    const current = stats.declaredLevel ?? "";
    for (const b of document.querySelectorAll<HTMLButtonElement>("#level-chips .lvl")) {
      b.classList.toggle("active", (b.dataset.lvl ?? "") === current);
    }
    $("level-hint").textContent = stats.declaredLevel
      ? `Les mots sous ${stats.declaredLevel} ne sont plus surlignés.`
      : "Choisis ton niveau — rien n'est présumé connu pour l'instant.";
  } else {
    $("level-block").hidden = true;
    $("calib-block").hidden = false;
    ($("calib") as HTMLInputElement).value = String(stats.calibration);
    $("calibv").textContent = String(stats.calibration);
  }
}

async function analyseCurrentPage(): Promise<void> {
  const tabId = await activeTabId();
  if (tabId == null) return;
  await chrome.scripting.executeScript({ target: { tabId }, files: ["content.js"] });
  setTimeout(() => void refresh(), 500);
}

async function refresh(): Promise<void> {
  render((await send({ type: "getStats" })) as PageStats | null);
}

/** Reflect the global enabled flag: off hides the reader panels; on shows them. */
async function applyEnabled(enabled: boolean): Promise<void> {
  ($("enabled") as HTMLInputElement).checked = enabled;
  $("enabled-label").textContent = enabled ? "Surlignage activé" : "Surlignage désactivé";
  $("disabled-note").hidden = enabled;
  if (!enabled) {
    $("setup").hidden = true;
    $("controls").hidden = true;
    return;
  }
  await refresh();
}

async function main(): Promise<void> {
  const calib = $("calib") as HTMLInputElement;
  calib.addEventListener("input", () => {
    $("calibv").textContent = calib.value;
  });
  calib.addEventListener("change", () => void send({ type: "setCalibration", value: Number(calib.value) }));

  // CEFR level picker: a chip declares the level; "Débutant" (empty value) clears
  // it — nothing presumed known. The content script sets it on the engine.
  for (const chip of document.querySelectorAll<HTMLButtonElement>("#level-chips .lvl")) {
    chip.addEventListener("click", async () => {
      await send({ type: "setLevel", value: chip.dataset.lvl ?? "" });
      await refresh();
    });
  }

  // Reset flow — a small wizard whose steps REPLACE one another, so the popup
  // shows exactly one thing at a time:
  //   rest    → only the "Réinitialiser…" button
  //   scope   → the button is hidden; the two scope choices + Annuler
  //   confirm → the warning + Oui, confirmer + Annuler (scope choices hidden)
  // Any "Annuler" (and confirming) returns to rest — just the button. A stray
  // click can never wipe the deck (choose scope, then confirm). `pendingScope`
  // carries the choice from the scope step to the confirm step.
  let pendingScope: "full" | "partial" | null = null;
  const showRest = (): void => {
    $("reset-menu").hidden = true;
    $("reset").hidden = false;
    $("reset-scope").hidden = false; // ready for the next open
    $("reset-confirm").hidden = true;
    pendingScope = null;
  };
  $("reset").addEventListener("click", () => {
    $("reset").hidden = true;
    $("reset-menu").hidden = false;
    $("reset-scope").hidden = false;
    $("reset-confirm").hidden = true;
    pendingScope = null;
  });
  $("reset-cancel").addEventListener("click", showRest);
  const askConfirm = (scope: "full" | "partial"): void => {
    pendingScope = scope;
    let warn: string;
    if (scope === "partial") {
      warn = "Effacer tes statuts et ta calibration ? Ton deck de révision est conservé.";
    } else if (accountSignedIn) {
      warn =
        "Effacer les données de cet appareil (statuts, deck, progression) ? " +
        "Comme tu es connecté, elles seront re-téléchargées depuis le serveur à la prochaine synchronisation.";
    } else {
      warn =
        "⚠️ Effacer DÉFINITIVEMENT tes statuts, ton deck de révision et ta progression ? " +
        "Tu n'es pas connecté : cette action est irréversible.";
    }
    $("reset-warn").textContent = warn;
    // Replace the scope step with the confirmation.
    $("reset-scope").hidden = true;
    $("reset-confirm").hidden = false;
  };
  $("reset-partial").addEventListener("click", () => askConfirm("partial"));
  $("reset-full").addEventListener("click", () => askConfirm("full"));
  $("reset-no").addEventListener("click", showRest); // Annuler = exit the whole flow
  $("reset-yes").addEventListener("click", async () => {
    if (!pendingScope) return;
    const scope = pendingScope;
    showRest();
    await send({ type: "reset", scope });
    await refresh();
  });

  // Settings view (gear icon): the rarely-used, destructive reset lives here,
  // off the main page. Opening or leaving it returns the reset flow to rest.
  $("settings-open").addEventListener("click", () => {
    showRest();
    $("main-view").hidden = true;
    $("settings-view").hidden = false;
  });
  $("settings-back").addEventListener("click", () => {
    showRest();
    $("settings-view").hidden = true;
    $("main-view").hidden = false;
  });

  $("review").addEventListener("click", async () => {
    const tabId = await activeTabId();
    if (tabId != null) await chrome.sidePanel.open({ tabId });
    window.close();
  });

  $("analyse").addEventListener("click", () => void analyseCurrentPage());
  $("always").addEventListener("click", async () => {
    if (await chrome.permissions.request({ origins: ["<all_urls>"] })) window.close();
  });

  $("enabled").addEventListener("change", async () => {
    const enabled = ($("enabled") as HTMLInputElement).checked;
    await saveEnabled(storageArea, enabled);
    await applyEnabled(enabled);
  });

  $("signin-google").addEventListener("click", async () => {
    const res = (await sendRuntime({ type: "account:signInGoogle" })) as AccountResult | null;
    if (res?.ok) renderAccount(res.state ?? { signedIn: true });
    else showAccountError(res?.error ?? "Connexion impossible.");
  });

  $("signin-local").addEventListener("click", async () => {
    const email = ($("acct-email") as HTMLInputElement).value.trim();
    const password = ($("acct-password") as HTMLInputElement).value;
    if (!email || !password) return;
    const res = (await sendRuntime({ type: "account:signInLocal", email, password })) as AccountResult | null;
    if (res?.ok) renderAccount(res.state ?? { signedIn: true });
    else showAccountError(res?.error ?? "Email ou mot de passe incorrect.");
  });

  $("signout").addEventListener("click", async () => {
    const res = (await sendRuntime({ type: "account:signOut" })) as AccountResult | null;
    renderAccount(res?.state ?? { signedIn: false });
  });

  $("open-stats").addEventListener("click", () => {
    void chrome.tabs.create({ url: chrome.runtime.getURL("stats.html") });
    window.close();
  });

  await applyEnabled(await loadEnabled(storageArea));
  renderAccount((await sendRuntime({ type: "account:state" })) as AccountState | null);
}

void main();
