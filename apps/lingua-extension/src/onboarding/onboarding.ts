import type { AccountReply } from "../account/messages.ts";
import { createLinguaPort } from "../analyzer/create-port.ts";
import type { CefrLevel } from "../analyzer/types.ts";
import { type AsyncStorageArea, hydrateEngine, saveBackup } from "../state/storage.ts";
import { messagedArea } from "../state/store.ts";

// First-run welcome tab, opened on install (Chromium/Firefox; a best-effort bonus —
// the popup's level call-to-action is the portable equivalent). It hydrates its own
// engine from the shared backup and lets the reader pick their CEFR level right away.
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

  if (!(await port.hasLevels())) return; // no CEFR data → welcome text only

  $("level-section").hidden = false;
  const current = await port.declaredLevel();
  const chips = Array.from(document.querySelectorAll<HTMLButtonElement>("#level-chips .lvl"));
  const mark = (level: CefrLevel | null): void => {
    for (const c of chips) c.classList.toggle("active", (c.dataset.lvl ?? "") === (level ?? ""));
  };
  mark(current);

  for (const chip of chips) {
    chip.addEventListener("click", async () => {
      const level = (chip.dataset.lvl as CefrLevel) || null;
      await port.setDeclaredLevelAt(level, Date.now()); // stamp for cross-device LWW
      // With a declared level, presumption comes only from it (option B).
      await port.setCalibration(0);
      await saveBackup(store, await port.backup());
      mark(level);
      const confirm = $("level-confirm");
      confirm.hidden = false;
      confirm.textContent = level
        ? `Niveau enregistré : ${level}. Tu peux fermer cet onglet et commencer à lire.`
        : "C'est noté — on part de zéro. Tu peux fermer cet onglet et commencer à lire.";
    });
  }
}

void main();
