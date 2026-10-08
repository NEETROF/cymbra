import type { AccountReply, AccountState } from "../account/messages.ts";
import { createLinguaPort } from "../analyzer/create-port.ts";
import { chooseLevelPrompt, levelTitle, noTextDetected } from "../analyzer/language-labels.ts";
import { DEFAULT_LANGUAGE, readingLanguage } from "../analyzer/pairs.ts";
import type { CefrLevel, StudiedLanguage } from "../analyzer/types.ts";
import { popup as enPopup } from "../i18n/en/popup.ts";
import { popup as esPopup } from "../i18n/es/popup.ts";
import { popup as frPopup } from "../i18n/fr/popup.ts";
import {
  DEFAULT_INTERFACE_LANGUAGE,
  fillPageInLanguage,
  formatCount,
  formatPercent,
  type InterfaceLanguage,
  NODE_SLOT,
  renderAround,
} from "../i18n/index.ts";
import { mountSettings, type SettingsTab, type SettingsView } from "../reading/settings-view.ts";
import { browserSpeechEngine, createSpeaker } from "../reading/speech.ts";
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
import { followSurfaceLook } from "../reading/surface-look.ts";

// This page is a surface: it follows the reader's colours and text size (add-lingua-colour-settings D8, D9).
followSurfaceLook(document.documentElement);
// Safari shows this page in a popover sized from its content (popup.css caps it on an iPad).
if (__TARGET__ === "safari") document.documentElement.dataset.popover = "";

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

/** The popup's copy by interface language (localise-lingua-reading-surfaces D1). */
const POPUP_COPY: Record<InterfaceLanguage, typeof frPopup> = { fr: frPopup, en: enPopup, es: esPopup };

/** The interface language and the copy, read with the first storage read, before anything renders. */
let language: InterfaceLanguage = DEFAULT_INTERFACE_LANGUAGE;
let copy: typeof frPopup = frPopup;

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
  /** Levels estimated from word frequency (add-lingua-spanish-levels); absent from an older build. */
  levelsEstimated?: boolean;
  /** No level decision yet (« Débutant » is a decision): show the call to action. */
  needsLevel: boolean;
  trackedCount: number;
  deckCount: number;
  dueCount: number;
  /** The page's language and the reader's accepted ones; absent from a content script of an older build. */
  language?: StudiedLanguage;
  languages?: StudiedLanguage[];
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

async function readAccount(): Promise<AccountState | null> {
  return ((await sendRuntime({ type: "account:state" })) as AccountReply | null)?.state ?? null;
}

/** Whether a Cymbra ID session is active — a lost-session mark then no longer applies. */
let accountSignedIn = false;

/**
 * The account, in one line: signing in, and all it leads to, lives in Réglages › Données
 * (`mountSettings`), the same in every surface. Signed in, the handle — or that one is still to
 * choose: an account without one is deleted by the backend's orphan reaper.
 */
async function renderAccount(state: AccountState | null): Promise<void> {
  const signedIn = state?.signedIn ?? false;
  accountSignedIn = signedIn;
  $("account-signin").hidden = signedIn;
  $("account-in").hidden = !signedIn;
  if (!signedIn) return;
  const res = (await sendRuntime({ type: "account:profile" })) as AccountReply | null;
  $("account-who").textContent =
    res?.ok && res.handle ? copy.handle(res.handle) : res?.ok ? copy.handleToChoose : copy.syncOn;
}

/** Say, above everything else, that a session this device held was refused by the server. */
async function refreshSessionLost(): Promise<void> {
  const got = await storageArea.get(SESSION_LOST_KEY);
  // Signed in again (here or in another surface): whatever the mark still says, the
  // session is not lost — say nothing rather than something stale.
  $("session-lost").hidden = accountSignedIn || got[SESSION_LOST_KEY] !== true;
}

/**
 * A provider sign-in that failed after Google's window tore this popup down, persisted by the
 * background: Réglages › Données says it (and clears it), so open them there.
 */
async function hasSignInError(): Promise<boolean> {
  try {
    return isPersistedSignInError((await chrome.storage.session.get(SIGNIN_ERROR_KEY))[SIGNIN_ERROR_KEY]);
  } catch {
    return false; // storage.session may be unavailable; nothing to surface.
  }
}

function render(stats: PageStats | null, onReader: boolean): void {
  const present = stats !== null;
  // A reader tab is never analysed from here: the reader page reads its books itself.
  $("setup").hidden = present || onReader;
  $("controls").hidden = !present;
  if (!stats) return;

  const book = stats.surface === "book" || onReader;
  $("pct-label").textContent = book ? copy.knownInChapter : copy.knownOnPage;
  $("analysed").hidden = !stats.analysable;
  $("note").hidden = stats.analysable;
  // Named from the language the page is read in (add-lingua-language-choice D1).
  const studied = stats.language ?? DEFAULT_LANGUAGE;
  $("note").textContent = book ? copy.openBookForFigures : noTextDetected(language, stats.languages ?? [studied]);
  if (stats.analysable) {
    const pct = stats.percent ?? 0;
    $("pct").textContent = stats.percent == null ? copy.noPercent : formatPercent(language, pct, "tight");
    ($("bar") as HTMLElement).style.width = `${pct}%`;
    $("counted").textContent = formatCount(language, stats.counted);
    $("unknown").textContent = formatCount(language, stats.unknownOccurrences);
    $("distinct").textContent = formatCount(language, stats.distinctUnknown);
  }

  $("tracked").textContent = formatCount(language, stats.trackedCount);
  $("deck").textContent = formatCount(language, stats.deckCount);
  $("review").textContent = copy.review(formatCount(language, stats.dueCount));

  // With CEFR data, the reader declares a level in Réglages; the main panel gets a compact
  // reminder, or a call-to-action until a level has been chosen (asked at first use).
  // « Débutant » is a decision (no level, but chosen): only a missing decision asks again.
  $("level-cta").hidden = !stats.hasLevels || !stats.needsLevel;
  $("level-indicator").hidden = !stats.hasLevels || stats.needsLevel;
  $("level-cta").textContent = chooseLevelPrompt(language, studied);
  // « Niveau de … : B1 »: the line's message rendered around the bold level (D1).
  const level = $("level-current");
  level.textContent = stats.declaredLevel ?? copy.beginner;
  renderAround(
    $("level-line"),
    copy.levelLine(levelTitle(language, studied, stats.levelsEstimated ?? false), NODE_SLOT),
    level,
  );
}

let settings: SettingsView | null = null;

/**
 * Réglages: the shared view, mounted the first time they open. Its engine port and speaker
 * are this page's own, as the side panel's are: a level, a calibration or a reset chosen here
 * is persisted to the store, which every page restores — the popup never reaches into a tab.
 */
async function showSettings(tab?: SettingsTab): Promise<void> {
  $("main-view").hidden = true;
  $("settings-view").hidden = false;
  if (!settings) {
    const port = createLinguaPort();
    await hydrateEngine(port, store);
    settings = mountSettings($("settings-body"), port, storageArea, {
      persist: async () => saveBackup(store, await port.backup()),
      store,
      speaker: createSpeaker(browserSpeechEngine(), await readingLanguage(port), storedVoicePreference(storageArea)),
      // Safari: the host app now shows the provider's sheet; the next open collects the token.
      onHandedOff: () => window.close(),
      // The language read with this page's first storage read (localise-lingua-settings D1).
      interfaceLanguage: language,
    });
  }
  if (tab) settings.show(tab);
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
  $("enabled-label").textContent = enabled ? copy.highlightingOn : copy.highlightingOff;
  $("disabled-note").hidden = enabled;
  if (!enabled) {
    $("setup").hidden = true;
    $("controls").hidden = true;
    return;
  }
  await refresh();
}

async function main(): Promise<void> {
  // The interface language first, with this page's first storage read: the page's static copy is
  // filled from the catalogue before anything shows (the body is hidden until then — D2), and
  // every text rendered below is that language's. A read that fails is French: the page shows.
  ({ language, copy } = await fillPageInLanguage(document, storageArea, (l) => POPUP_COPY[l]));

  // Settings view (gear icon), also reached from the main panel's level call-to-action and
  // « Modifier ». Leaving it re-reads the page's stats: a level or a calibration chosen there
  // moves the percentage.
  $("settings-open").addEventListener("click", () => void showSettings());
  // The level is what these two lead to: its tab, whichever one the reader left open.
  const openLevel = (): void => void showSettings("language");
  $("level-cta").addEventListener("click", openLevel);
  $("level-edit").addEventListener("click", openLevel);
  $("settings-back").addEventListener("click", () => {
    $("settings-view").hidden = true;
    $("main-view").hidden = false;
    void refresh();
    // Signed in or out in Réglages › Données: the account line follows.
    void readAccount().then(renderAccount).then(refreshSessionLost);
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

  const openAccount = (): void => void showSettings("data");
  $("account-signin").addEventListener("click", openAccount);
  $("account-manage").addEventListener("click", openAccount);

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
  const state = await readAccount();
  await renderAccount(state);
  // A sign-in failure that happened after the popup closed: Réglages › Données say it.
  if (!state?.signedIn && (await hasSignInError())) await showSettings("data");

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
