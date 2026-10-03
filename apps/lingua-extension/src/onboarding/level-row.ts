import { estimatedLevelsNote, levelName, levelQuestion } from "../analyzer/language-labels.ts";
import type { LanguagePort } from "../analyzer/port.ts";
import { CEFR_LEVELS, type CefrLevel, type StudiedLanguage } from "../analyzer/types.ts";

// Onboarding's level question for one language, apart from the page's wiring so a test mounts it
// alone (onboarding.ts starts the page when imported).

/**
 * One language's level question and chips; a chip is saved at once. Levels estimated from word
 * frequency say so, under the chips and in the confirmation (add-lingua-spanish-levels).
 */
export function levelRow(
  language: StudiedLanguage,
  view: Pick<LanguagePort, "setDeclaredLevelAt" | "setCalibration">,
  current: CefrLevel | null,
  estimated: boolean,
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
        ? `Niveau enregistré : ${levelName(level, estimated)}. Tu peux fermer cet onglet et commencer à lire.`
        : "C'est noté — on part de zéro. Tu peux fermer cet onglet et commencer à lire.";
    });
  }
  row.append(title, chips);
  if (estimated) {
    const why = document.createElement("p");
    why.className = "note";
    why.textContent = estimatedLevelsNote(language);
    row.append(why);
  }
  row.append(confirm);
  return row;
}
