import type { AccountReply } from "../account/messages.ts";
import { createLinguaPort } from "../analyzer/create-port.ts";
import { acceptedLanguages } from "../analyzer/pairs.ts";
import { fillPageInLanguage } from "../i18n/index.ts";
import { mountStudiedLanguages } from "../reading/studied-languages-view.ts";
import { onboardingCopy, studiedLanguagesCopy } from "./copy.ts";
import { levelRow } from "./level-row.ts";
import { type AsyncStorageArea, hydrateEngine, saveBackup } from "../state/storage.ts";
import { messagedArea } from "../state/store.ts";
import { followSurfaceLook } from "../reading/surface-look.ts";
import { mountNativeStep, presetThenStart } from "./native-step.ts";

// This page is a surface: it follows the reader's colours and text size (add-lingua-colour-settings D8, D9).
followSurfaceLook(document.documentElement);

// First-run welcome tab, opened on install (Chromium/Firefox; a best-effort bonus —
// the popup's level call-to-action is the portable equivalent). It hydrates its own
// engine from the shared backup and lets the reader pick their languages, when the package
// ships several, and a CEFR level for each right away.
// If the pack carries no CEFR data the level step is hidden. It speaks the interface language,
// read before its engine (localise-lingua-account-onboarding D1). Excluded from coverage
// (DOM wiring; the engine/model are tested elsewhere).

/**
 * The reader's data, owned by the background — never `chrome.storage.local`, which the store
 * left (change: move-lingua-store-to-indexeddb). Reading the backup from the old address gave
 * this tab an empty engine, and writing the chosen level back put it where nothing reads: the
 * level the reader picked here was lost.
 */
const store: AsyncStorageArea = messagedArea();

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

/**
 * Optional, skippable account step (add-lingua-account-parity, design D8). Hidden when
 * already signed in; "Plus tard" just hides it — nothing else depends on it.
 */
async function showAccountOffer(): Promise<void> {
  try {
    const reply = (await chrome.runtime.sendMessage({ type: "account:state" })) as AccountReply | undefined;
    if (reply?.state?.signedIn) return;
  } catch {
    // Worker unreachable: offer anyway, the account page handles it.
  }
  const section = $("account-section");
  section.hidden = false;
  $("account-create").addEventListener(
    "click",
    () => void chrome.tabs.create({ url: chrome.runtime.getURL("account.html#signup") }),
  );
  $("account-later").addEventListener("click", () => {
    section.hidden = true;
  });
}

async function main(): Promise<void> {
  // The interface language first, before the engine — a read: the store's owner alone writes the
  // key. The page's static copy is filled from the catalogue and its lang said before anything
  // shows (the body is hidden until then); a read that fails is French.
  const { language: interfaceLanguage } = await fillPageInLanguage(
    document,
    { get: (key) => chrome.storage.local.get(key) },
    onboardingCopy,
  );
  void showAccountOffer();
  const port = createLinguaPort();
  await hydrateEngine(port, store);
  const persist = async (): Promise<void> => saveBackup(store, await port.backup());

  // The languages first, when the package ships several (add-lingua-language-choice D5); then a
  // level for each language the reader accepts — both in the interface language read above, which
  // the views that name the languages take with no default (add-lingua-native-language-labels).
  const studied = mountStudiedLanguages(
    $("languages-section"),
    port,
    async () => {
      await persist();
      await renderLevels();
    },
    undefined,
    studiedLanguagesCopy(interfaceLanguage),
    interfaceLanguage,
  );
  await studied.refresh();
  await renderLevels();
  // The native language, the first question when two ship; the page reloads when it changes
  // (add-lingua-native-language-choice D3, D4). Built only then (`__NATIVE_CHOICE__`).
  if (__NATIVE_CHOICE__) mountNativeStep($("languages-section"), interfaceLanguage, store);

  async function renderLevels(): Promise<void> {
    const rows = $("level-rows");
    rows.replaceChildren();
    for (const language of await acceptedLanguages(port)) {
      const view = port.for(language);
      if (!(await view.hasLevels())) continue; // no CEFR data for this language
      rows.append(
        levelRow(language, view, await view.declaredLevel(), await view.levelsEstimated(), persist, interfaceLanguage),
      );
    }
    $("level-section").hidden = rows.childElementCount === 0;
  }
}

// A new install's native language is preset before the page paints (add-lingua-native-language-choice
// D4) — while two native languages ship; until then the page starts as it did.
void (__NATIVE_CHOICE__ ? presetThenStart(main) : main());
