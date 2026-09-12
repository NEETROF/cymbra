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

function renderAccount(state: AccountState | null): void {
  const signedIn = state?.signedIn ?? false;
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
  ($("calib") as HTMLInputElement).value = String(stats.calibration);
  $("calibv").textContent = String(stats.calibration);
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

  // Reset flow: open a menu, choose a scope (partial / complete), then confirm.
  // Two deliberate clicks minimum, and the destructive path is styled + spelled
  // out — a single stray click can never wipe the deck. `pendingScope` carries
  // the choice between the scope step and the confirm step.
  let pendingScope: "full" | "partial" | null = null;
  const closeResetMenu = (): void => {
    $("reset-menu").hidden = true;
    $("reset-confirm").hidden = true;
    pendingScope = null;
  };
  $("reset").addEventListener("click", () => {
    const menu = $("reset-menu");
    menu.hidden = !menu.hidden;
    $("reset-confirm").hidden = true;
    pendingScope = null;
  });
  $("reset-cancel").addEventListener("click", closeResetMenu);
  const askConfirm = (scope: "full" | "partial"): void => {
    pendingScope = scope;
    $("reset-warn").textContent =
      scope === "full"
        ? "⚠️ Effacer DÉFINITIVEMENT tes statuts, ton deck de révision et ta progression ? Action irréversible."
        : "Effacer tes statuts et ta calibration ? Ton deck de révision est conservé.";
    $("reset-confirm").hidden = false;
  };
  $("reset-partial").addEventListener("click", () => askConfirm("partial"));
  $("reset-full").addEventListener("click", () => askConfirm("full"));
  $("reset-no").addEventListener("click", () => {
    $("reset-confirm").hidden = true;
    pendingScope = null;
  });
  $("reset-yes").addEventListener("click", async () => {
    if (!pendingScope) return;
    const scope = pendingScope;
    closeResetMenu();
    await send({ type: "reset", scope });
    await refresh();
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
