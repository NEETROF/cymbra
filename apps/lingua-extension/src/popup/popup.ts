import { errorCopy } from "../account/copy.ts";
import { type AccountReply, type AccountState, PENDING_EMAIL_KEY } from "../account/messages.ts";
import type { Provider } from "../state/oidc.ts";
import { hasShortcutEditor } from "../state/platform.ts";
import { isPersistedSignInError, SIGNIN_ERROR_KEY } from "../state/session.ts";
import { loadEnabled, loadHudHidden, ROOT_KEY, saveEnabled, saveHudHidden } from "../state/storage.ts";
import { requestSync } from "../sync/messages.ts";
import type { CefrLevel } from "../analyzer/types.ts";

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
  /** No level decision yet (« Débutant » is a decision): show the call to action. */
  needsLevel: boolean;
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

/** Whether a Cymbra ID session is active — decides the reset warning's wording. */
let accountSignedIn = false;

function renderAccount(state: AccountState | null): void {
  const signedIn = state?.signedIn ?? false;
  accountSignedIn = signedIn;
  $("acct-in").hidden = !signedIn;
  $("acct-out").hidden = signedIn;
  $("acct-error").hidden = true;
  if (signedIn) void renderHandle();
  else $("acct-handle-cta").hidden = true;
}

/**
 * Show the account's handle, or ask for one: a Cymbra account without a handle is deleted by
 * the backend's orphan reaper, so the popup keeps offering the account page's handle step.
 */
async function renderHandle(): Promise<void> {
  const res = (await sendRuntime({ type: "account:profile" })) as AccountReply | null;
  const needsHandle = res?.ok === true && res.handle == null;
  $("acct-handle").textContent =
    res?.ok && res.handle ? `@${res.handle}` : needsHandle ? "Pseudo à choisir" : "Synchronisation activée";
  $("acct-handle-cta").hidden = !needsHandle;
}

function showAccountError(message: string): void {
  const el = $("acct-error");
  el.textContent = message;
  el.hidden = false;
}

/** Drop the persisted sign-in error (storage.session); tolerates it being unavailable. */
async function clearSignInError(): Promise<void> {
  try {
    await chrome.storage.session.set({ [SIGNIN_ERROR_KEY]: null });
  } catch {
    // storage.session may be unavailable; nothing to clear.
  }
}

/**
 * Surface (once) a sign-in failure the background persisted while this popup was torn
 * down by Google's auth window. Read from storage.session directly — the worker may have
 * napped since — then clear it so it never shows stale. No-op when already signed in.
 */
async function surfaceSignInError(): Promise<void> {
  try {
    const got = await chrome.storage.session.get(SIGNIN_ERROR_KEY);
    const err = got[SIGNIN_ERROR_KEY];
    if (isPersistedSignInError(err)) {
      if (!accountSignedIn) showAccountError(errorCopy(providerContext(err.provider), err.kind));
      await clearSignInError();
    }
  } catch {
    // storage.session may be unavailable in some contexts; nothing to surface.
  }
}

function providerContext(provider: Provider): "signInGoogle" | "signInApple" {
  return provider === "apple" ? "signInApple" : "signInGoogle";
}

/** Open the account page — a tab, which survives the reader leaving for their mailbox. */
async function openAccountPage(view: "signup" | "forgot" | "verify" | "handle" | "data"): Promise<void> {
  try {
    await chrome.tabs.create({ url: chrome.runtime.getURL(`account.html#${view}`) });
  } catch {
    // Tab creation refused; nothing actionable in the popup.
  }
  window.close();
}

/** Show only the providers this build and browser support (add-lingua-account-parity D5). */
async function renderProviders(): Promise<void> {
  const res = (await sendRuntime({ type: "account:providers" })) as AccountReply | null;
  const google = Boolean(res?.providers?.google);
  const apple = Boolean(res?.providers?.apple);
  $("signin-google").hidden = !google;
  $("signin-apple").hidden = !apple;
  // No provider (Safari, Firefox for Android, or none configured): email is the only way
  // in, so show its form already unfolded.
  if (!google && !apple) document.querySelector<HTMLDetailsElement>(".acct-local")?.setAttribute("open", "");
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
    // The picker lives in Réglages; the main panel gets a compact reminder, or a
    // call-to-action until a level has been chosen (asked at first use).
    $("level-block").hidden = false;
    $("calib-block").hidden = true;
    // « Débutant » is a decision (no level, but chosen): only a missing decision asks again.
    const current = stats.needsLevel ? null : (stats.declaredLevel ?? "");
    for (const b of document.querySelectorAll<HTMLButtonElement>("#level-chips .lvl")) {
      b.classList.toggle("active", (b.dataset.lvl ?? "") === current);
    }
    $("level-hint").textContent = stats.declaredLevel
      ? `Les mots sous ${stats.declaredLevel} ne sont plus surlignés.`
      : stats.needsLevel
        ? "Choisis ton niveau — rien n'est présumé connu pour l'instant."
        : "Débutant — rien n'est présumé connu.";
    $("level-cta").hidden = !stats.needsLevel;
    $("level-indicator").hidden = stats.needsLevel;
    $("level-current").textContent = stats.declaredLevel ?? "Débutant";
  } else {
    $("level-block").hidden = true;
    $("calib-block").hidden = false;
    ($("calib") as HTMLInputElement).value = String(stats.calibration);
    $("calibv").textContent = String(stats.calibration);
    $("level-cta").hidden = true;
    $("level-indicator").hidden = true;
  }
}

/**
 * Open a panel view, staying in the page as much as possible and landing in the SAME place
 * as the in-page HUD:
 *  - Chromium: the native Side Panel (docks beside the page).
 *  - Firefox and Safari: the in-page drawer — a page element cannot open Firefox's sidebar and
 *    Safari has no panel API, so both the popup and the HUD use the drawer, which never leaves
 *    the page (and works on mobile too). The reader is always injected there, so the message
 *    reaches the active tab.
 */
async function openReviewSurface(view: "review" | "stats"): Promise<void> {
  if (__REVIEW_IN_PAGE__) {
    await send({ type: "openDrawer", view });
    window.close();
    return;
  }
  // Set the exact view: the panel reads it on load AND reacts to it changing while already
  // open, so "Réviser" switches an open panel off Statistiques (an open panel ignores a
  // second sidePanel.open()).
  try {
    await chrome.storage.session.set({ "cymbra-lingua-panel-view": view });
  } catch {
    // storage.session may be unavailable; the panel just opens on the review view.
  }
  const tabId = await activeTabId();
  if (tabId != null) await chrome.sidePanel.open({ tabId });
  window.close();
}

/** Open the browser's per-extension keyboard-shortcut editor (Chrome) / add-ons page (Firefox). */
async function openShortcutsConfig(): Promise<void> {
  // Extensions may open these internal pages via tabs.create; Firefox has no direct
  // shortcuts URL, so about:addons (its editor lives under the gear menu) is the target.
  const url = __TARGET__ === "firefox" ? "about:addons" : "chrome://extensions/shortcuts";
  try {
    await chrome.tabs.create({ url });
  } catch {
    // Some builds block internal URLs; nothing actionable to show in the popup.
  }
  window.close();
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
  calib.addEventListener("change", async () => {
    await send({ type: "setCalibration", value: Number(calib.value) });
    await refresh(); // the calibration moves the known-word percentage; reflect it now
  });

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

  // Settings view (gear icon): the level picker + the destructive reset live
  // here, off the main page. Opening or leaving it returns the reset flow to
  // rest. The main-panel level CTA / "Modifier" also route here.
  const openSettings = (): void => {
    showRest();
    $("main-view").hidden = true;
    $("settings-view").hidden = false;
  };
  $("settings-open").addEventListener("click", openSettings);
  $("level-cta").addEventListener("click", openSettings);
  $("level-edit").addEventListener("click", openSettings);
  $("settings-back").addEventListener("click", () => {
    showRest();
    $("settings-view").hidden = true;
    $("main-view").hidden = false;
  });

  $("review").addEventListener("click", () => void openReviewSurface("review"));

  $("analyse").addEventListener("click", () => void analyseCurrentPage());
  $("always").addEventListener("click", async () => {
    if (await chrome.permissions.request({ origins: ["<all_urls>"] })) window.close();
  });

  $("enabled").addEventListener("change", async () => {
    const enabled = ($("enabled") as HTMLInputElement).checked;
    await saveEnabled(storageArea, enabled);
    await applyEnabled(enabled);
  });

  // Opening a provider's auth window steals focus and tears this popup down, so the awaited
  // result usually never arrives here (res === null). That is fine: the background finishes
  // the sign-in, and on reopen the popup shows the signed-in state — or the persisted
  // failure via surfaceSignInError(). A null result is NOT a failure to report here; only
  // act when the popup actually survived. A closed provider window is a cancel: no message.
  const providerSignIn = async (provider: Provider): Promise<void> => {
    const type = provider === "apple" ? "account:signInApple" : "account:signInGoogle";
    const res = (await sendRuntime({ type })) as AccountReply | null;
    if (res?.ok) renderAccount(res.state ?? { signedIn: true });
    // Safari: the host app now shows the provider's sheet; the next popup open collects the token.
    else if (res?.handedOff) window.close();
    else if (res && !res.cancelled) {
      // Shown live — drop the background's persisted copy so it does not re-show next open.
      showAccountError(errorCopy(providerContext(provider), res.error ?? "unknown"));
      await clearSignInError();
    }
  };
  $("signin-google").addEventListener("click", () => void providerSignIn("google"));
  $("signin-apple").addEventListener("click", () => void providerSignIn("apple"));

  $("signin-local").addEventListener("click", async () => {
    const email = ($("acct-email") as HTMLInputElement).value.trim();
    const password = ($("acct-password") as HTMLInputElement).value;
    if (!email || !password) return;
    // Local sign-in never opens an auth window, so this popup stays alive: a null result
    // is a real transport/worker hiccup (not a torn-down popup) — reported as unreachable.
    const res = (await sendRuntime({ type: "account:signInLocal", email, password })) as AccountReply | null;
    if (res?.ok) renderAccount(res.state ?? { signedIn: true });
    else if (res?.error === "failedPrecondition") {
      // Unverified email: continue on the account page's code step. Only the email goes
      // across (storage.session); the password never leaves this popup (design D6).
      try {
        await chrome.storage.session.set({ [PENDING_EMAIL_KEY]: email });
      } catch {
        // storage.session unavailable: the page opens on sign-in instead.
      }
      await openAccountPage("verify");
    } else showAccountError(errorCopy("signInEmail", res?.error ?? "unavailable"));
  });
  $("acct-signup").addEventListener("click", () => void openAccountPage("signup"));
  $("acct-forgot").addEventListener("click", () => void openAccountPage("forgot"));
  $("acct-handle-open").addEventListener("click", () => void openAccountPage("handle"));
  $("acct-data").addEventListener("click", () => void openAccountPage("data"));

  $("signout").addEventListener("click", async () => {
    const res = (await sendRuntime({ type: "account:signOut" })) as AccountReply | null;
    renderAccount(res?.state ?? { signedIn: false });
  });

  $("open-stats").addEventListener("click", () => void openReviewSurface("stats"));
  const shortcutsConfig = $("shortcuts-config");
  shortcutsConfig.hidden = !hasShortcutEditor();
  shortcutsConfig.addEventListener("click", () => void openShortcutsConfig());

  // In-page HUD visibility: checked = shown. The content script reacts via storage.onChanged.
  const hudToggle = $("hud-toggle") as HTMLInputElement;
  hudToggle.checked = !(await loadHudHidden(storageArea));
  hudToggle.addEventListener("change", async () => {
    await saveHudHidden(storageArea, !hudToggle.checked);
  });

  await applyEnabled(await loadEnabled(storageArea));
  // Safari: exchange an id_token the host app handed back before reading the account state;
  // a failure is persisted by the background and shown by surfaceSignInError() below.
  if (__NATIVE_PROVIDERS__) await sendRuntime({ type: "account:collectHandedToken" });
  renderAccount(((await sendRuntime({ type: "account:state" })) as AccountReply | null)?.state ?? null);
  await renderProviders();
  await surfaceSignInError(); // show a sign-in failure that happened after the popup closed

  // Opening the popup asks for a sync. What it pulls changes the backup, which the page
  // restores: re-read the counts once the page has caught up.
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area === "local" && changes[ROOT_KEY]) {
      setTimeout(() => void applyEnabled(($("enabled") as HTMLInputElement).checked), 300);
    }
  });
  void requestSync();
}

void main();
