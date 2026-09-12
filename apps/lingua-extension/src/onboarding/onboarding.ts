import { createLinguaPort } from "../analyzer/create-port.ts";
import type { CefrLevel } from "../analyzer/types.ts";
import { type AsyncStorageArea, hydrateEngine, saveBackup } from "../state/storage.ts";

// First-run welcome tab, opened on install (Chromium/Firefox; a best-effort bonus —
// the popup's level call-to-action is the portable equivalent). It hydrates its own
// engine from the shared backup and lets the reader pick their CEFR level right away.
// If the pack carries no CEFR data the level step is hidden. Excluded from coverage
// (DOM wiring; the engine/model are tested elsewhere).

const area: AsyncStorageArea = {
  get: (keys) => chrome.storage.local.get(keys),
  set: (items) => chrome.storage.local.set(items),
};

function $(id: string): HTMLElement {
  const el = document.getElementById(id);
  if (!el) throw new Error(`missing #${id}`);
  return el;
}

async function main(): Promise<void> {
  const port = createLinguaPort();
  await hydrateEngine(port, area);

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
      await port.setDeclaredLevel(level);
      // With a declared level, presumption comes only from it (option B).
      await port.setCalibration(0);
      await saveBackup(area, await port.backup());
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
