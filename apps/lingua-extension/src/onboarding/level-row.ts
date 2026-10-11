import { estimatedLevelsNote, levelName, levelQuestion } from "../analyzer/language-labels.ts";
import type { LanguagePort } from "../analyzer/port.ts";
import { CEFR_LEVELS, type CefrLevel, type StudiedLanguage } from "../analyzer/types.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
import { onboardingCopy } from "./copy.ts";

// Onboarding's level question for one language, apart from the page's wiring so a test mounts it
// alone (onboarding.ts starts the page when imported). Its question, a level's « (estimé) » and the
// estimated levels' note are the labels module's (add-lingua-native-language-labels D2), its chips
// and confirmations the catalogue's (localise-lingua-account-onboarding D1), all in the interface
// language it is handed — with no default: the page hands it the one it reads.

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
  interfaceLanguage: InterfaceLanguage,
): HTMLElement {
  const copy = onboardingCopy(interfaceLanguage);
  const row = document.createElement("div");
  const title = document.createElement("h2");
  title.textContent = levelQuestion(interfaceLanguage, language);
  const chips = document.createElement("div");
  chips.className = "chips";
  const confirm = document.createElement("p");
  confirm.className = "confirm";
  confirm.hidden = true;
  const choices: [string, string][] = [...CEFR_LEVELS.map((l): [string, string] => [l, l]), ["", copy.beginnerChip]];
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
        ? copy.levelSaved(levelName(interfaceLanguage, level, estimated))
        : copy.beginnerSaved;
    });
  }
  row.append(title, chips);
  if (estimated) {
    const why = document.createElement("p");
    why.className = "note";
    why.textContent = estimatedLevelsNote(interfaceLanguage, language);
    row.append(why);
  }
  row.append(confirm);
  return row;
}
