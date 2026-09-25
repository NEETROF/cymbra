import { errorCopy } from "../account/copy.ts";
import { type AccountReply, type AccountState, PENDING_EMAIL_KEY } from "../account/messages.ts";
import { createLinguaPort } from "../analyzer/create-port.ts";
import { type CefrLevel, STUDIED_LANGUAGE } from "../analyzer/types.ts";
import { mountSettings, type SettingsView } from "../reading/settings-view.ts";
import { browserSpeechEngine, createSpeaker } from "../reading/speech.ts";
import type { Provider } from "../state/oidc.ts";
import { isPersistedSignInError, SIGNIN_ERROR_KEY } from "../state/session.ts";
import {
  type AsyncStorageArea,
  hydrateEngine,
  loadEnabled,
  ROOT_KEY,
  saveBackup,
  saveEnabled,
  SESSION_LOST_KEY,
  storedVoicePreference,
} from "../state/storage.ts";
import { messagedArea, watchStore } from "../state/store.ts";
import { requestSync } from "../sync/messages.ts";
import { isReaderUrl, READER_PAGE } from "../reader/locate.ts";
import type { OpenPageMessage } from "../state/open-page.ts";

// Icon-popup controller (a surface the extension owns). It asks the active tab's content
// script for the page's stats, and writes the global enabled flag directly (a plain setting;
// content scripts react via storage.onChanged). Its Réglages are NOT its own: they are
// `mountSettings`, the same builder the side panel and the in-page drawer render, driven —
// like the side panel — by an engine port of this page, created the first time Réglages
// open. A hand copy lived here once, and each block added to the shared view since was
// missing from it (the read-aloud voice, found in dogfooding); `test/lint-settings-hosts.spec.ts`
// now refuses one.
// Excluded from coverage (DOM wiring).

const storageArea: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

/** The reader's data, owned by the background (Réglages persist the engine's backup there). */
const store: AsyncStorageArea = messagedArea();

interface PageStats {
  /** "book" when the tab is the extension's reader, showing a section of a book. */
  surface?: "page" | "book";
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

/** Whether the active tab is the extension's book reader — an extension page, which the
 *  popup cannot inject into and must not offer to analyse (add-lingua-reader D8). */
async function activeTabIsReader(): Promise<boolean> {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  return isReaderUrl(tab?.url, chrome.runtime.getURL(READER_PAGE));
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

/** Whether a Cymbra ID session is active — a lost-session mark or a sign-in error then no longer applies. */
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

/** Say, above everything else, that a session this device held was refused by the server. */
async function refreshSessionLost(): Promise<void> {
  const got = await storageArea.get(SESSION_LOST_KEY);
  // Signed in again (here or in another surface): whatever the mark still says, the
  // session is not lost — say nothing rather than something stale.
  $("session-lost").hidden = accountSignedIn || got[SESSION_LOST_KEY] !== true;
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

function render(stats: PageStats | null, onReader: boolean): void {
  const present = stats !== null;
  // A reader tab is never analysed from here: the reader page reads its books itself.
  $("setup").hidden = present || onReader;
  $("controls").hidden = !present;
  if (!stats) return;

  const book = stats.surface === "book" || onReader;
  $("pct-label").textContent = book ? "de mots connus dans ce chapitre" : "de mots connus sur cette page";
  $("analysed").hidden = !stats.analysable;
  $("note").hidden = stats.analysable;
  $("note").textContent = book
    ? "Ouvre un livre de ta bibliothèque pour voir ses chiffres."
    : "Pas de texte anglais détecté sur cette page.";
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

  // With CEFR data, the reader declares a level in Réglages; the main panel gets a compact
  // reminder, or a call-to-action until a level has been chosen (asked at first use).
  // « Débutant » is a decision (no level, but chosen): only a missing decision asks again.
  $("level-cta").hidden = !stats.hasLevels || !stats.needsLevel;
  $("level-indicator").hidden = !stats.hasLevels || stats.needsLevel;
  $("level-current").textContent = stats.declaredLevel ?? "Débutant";
}

let settings: SettingsView | null = null;

/**
 * Réglages: the shared view, mounted the first time they open. Its engine port and speaker
 * are this page's own, as the side panel's are: a level, a calibration or a reset chosen here
 * is persisted to the store, which every page restores — the popup never reaches into a tab.
 */
async function showSettings(): Promise<void> {
  $("main-view").hidden = true;
  $("settings-view").hidden = false;
  if (!settings) {
    const port = createLinguaPort();
    await hydrateEngine(port, store);
    settings = mountSettings($("settings-body"), port, storageArea, {
      persist: async () => saveBackup(store, await port.backup()),
      store,
      speaker: createSpeaker(browserSpeechEngine(), STUDIED_LANGUAGE, storedVoicePreference(storageArea)),
    });
  }
  await settings.refresh();
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

async function analyseCurrentPage(): Promise<void> {
  const tabId = await activeTabId();
  if (tabId == null) return;
  await chrome.scripting.executeScript({ target: { tabId }, files: ["content.js"] });
  setTimeout(() => void refresh(), 500);
}

async function refresh(): Promise<void> {
  render((await send({ type: "getStats" })) as PageStats | null, await activeTabIsReader());
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
  // Settings view (gear icon), also reached from the main panel's level call-to-action and
  // « Modifier ». Leaving it re-reads the page's stats: a level or a calibration chosen there
  // moves the percentage.
  const openSettings = (): void => void showSettings();
  $("settings-open").addEventListener("click", openSettings);
  $("level-cta").addEventListener("click", openSettings);
  $("level-edit").addEventListener("click", openSettings);
  $("settings-back").addEventListener("click", () => {
    $("settings-view").hidden = true;
    $("main-view").hidden = false;
    void refresh();
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
  // The same destination as the settings' row: the background opens the reader, or brings
  // forward the one already open.
  $("open-library").addEventListener("click", async () => {
    await sendRuntime({ type: "openPage", url: READER_PAGE } satisfies OpenPageMessage);
    window.close();
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
  await refreshSessionLost();
  // What a sync pulled changes the store, which the page restores: re-read the counts
  // once it has caught up.
  watchStore((keys) => {
    if (keys.includes(ROOT_KEY)) {
      setTimeout(() => void applyEnabled(($("enabled") as HTMLInputElement).checked), 300);
    }
  });
  chrome.storage.onChanged.addListener((changes, area) => {
    if (area !== "local") return;
    if (changes[SESSION_LOST_KEY]) void refreshSessionLost();
  });
  void requestSync("surface");
}

void main();
