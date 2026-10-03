import type { AccountReply } from "../account/messages.ts";
import { createLinguaPort } from "../analyzer/create-port.ts";
import { levelQuestion } from "../analyzer/language-labels.ts";
import { acceptedLanguages } from "../analyzer/pairs.ts";
import type { LanguagePort } from "../analyzer/port.ts";
import { CEFR_LEVELS, type CefrLevel, type StudiedLanguage } from "../analyzer/types.ts";
import { mountStudiedLanguages } from "../reading/studied-languages-view.ts";
import { type AsyncStorageArea, hydrateEngine, saveBackup } from "../state/storage.ts";
import { messagedArea } from "../state/store.ts";
import { followSurfaceLook } from "../reading/surface-look.ts";

// This page is a surface: it follows the reader's colours and text size (add-lingua-colour-settings D8, D9).
followSurfaceLook(document.documentElement);

// First-run welcome tab, opened on install (Chromium/Firefox; a best-effort bonus —
// the popup's level call-to-action is the portable equivalent). It hydrates its own
// engine from the shared backup and lets the reader pick their languages, when the package
// ships several, and a CEFR level for each right away.
// If the pack carries no CEFR data the level step is hidden. Excluded from coverage
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
  void showAccountOffer();
  const port = createLinguaPort();
  await hydrateEngine(port, store);
  const persist = async (): Promise<void> => saveBackup(store, await port.backup());

  // The languages first, when the package ships several (add-lingua-language-choice D5); then a
  // level for each language the reader accepts.
  const studied = mountStudiedLanguages($("languages-section"), port, async () => {
    await persist();
    await renderLevels();
  });
  await studied.refresh();
  await renderLevels();

  async function renderLevels(): Promise<void> {
    const rows = $("level-rows");
    rows.replaceChildren();
    for (const language of await acceptedLanguages(port)) {
      const view = port.for(language);
      if (!(await view.hasLevels())) continue; // no CEFR data for this language
      rows.append(levelRow(language, view, await view.declaredLevel(), persist));
    }
    $("level-section").hidden = rows.childElementCount === 0;
  }
}

/** One language's level question and chips; a chip is saved at once. */
function levelRow(
  language: StudiedLanguage,
  view: LanguagePort,
  current: CefrLevel | null,
  persist: () => Promise<void>,
): HTMLElement {
  const row = document.createElement("div");
  const title = document.createElement("h2");
  title.textContent = levelQuestion(language);
  const chips = document.createElement("div");
  chips.className = "chips";
  const confirm = document.createElement("p");
  confirm.className = "confirm";
  confirm.hidden = true;
  const choices: [string, string][] = [
    ...CEFR_LEVELS.map((l): [string, string] => [l, l]),
    ["", "Débutant — je pars de zéro"],
  ];
  const buttons = choices.map(([value, label]) => {
    const b = document.createElement("button");
    b.className = value === "" ? "lvl beginner" : "lvl";
    b.dataset.lvl = value;
    b.textContent = label;
    chips.append(b);
    return b;
  });
  const mark = (level: CefrLevel | null): void => {
    for (const b of buttons) b.classList.toggle("active", (b.dataset.lvl ?? "") === (level ?? ""));
  };
  mark(current);
  for (const b of buttons) {
    b.addEventListener("click", async () => {
      const level = (b.dataset.lvl as CefrLevel) || null;
      await view.setDeclaredLevelAt(level, Date.now()); // stamp for cross-device LWW
      // With a declared level, presumption comes only from it (option B).
      await view.setCalibration(0);
      await persist();
      mark(level);
      confirm.hidden = false;
      confirm.textContent = level
        ? `Niveau enregistré : ${level}. Tu peux fermer cet onglet et commencer à lire.`
        : "C'est noté — on part de zéro. Tu peux fermer cet onglet et commencer à lire.";
    });
  }
  row.append(title, chips, confirm);
  return row;
}

void main();
