import { languageName } from "../analyzer/language-labels.ts";
import { DEFAULT_NATIVE, pairsOf, SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import type { StudiedLanguage } from "../analyzer/types.ts";
import { studiedLanguages as frStudiedLanguages } from "../i18n/fr/studied-languages.ts";
import type { InterfaceLanguage } from "../i18n/index.ts";
import type { StudiedLanguagesCopy } from "./settings-copy.ts";

// « Langues étudiées » (add-lingua-language-choice D2): a box per language the package ships, ticked
// when the reader studies it. Ticking appends a language, unticking removes one, and the only ticked
// language cannot be removed. The choice is the reader's profile, in their backup: `persist` saves it,
// and every context restores it and follows. A profile language the package does not ship is not
// listed, and is kept. With one shipped language there is nothing to choose: the block hides.
// The languages offered are those of the pairs glossed in the reader's native language
// (generalise-lingua-native-language D7): French until the port says otherwise, so a reader of
// French sees the boxes from the start, as before. Its two notes are the catalogue's
// `studied-languages` module, handed by the settings view in the interface language
// (localise-lingua-settings), which also names the languages (add-lingua-native-language-labels D2).
// The language has no default: a host that forgot it would name the languages in French to every
// reader, and no French assertion would notice — the onboarding page hands it French explicitly,
// until localise-lingua-account-onboarding hands it the one it reads.

export interface StudiedLanguagesView {
  /** Show the stored choice (it may have changed in another context). */
  refresh(): Promise<void>;
}

/** The studied side of each shipped pair, once each, in the pairs' order. */
export function shippedLanguages(pairs: readonly string[] = SHIPPED_PAIRS): StudiedLanguage[] {
  return [...new Set(pairs.map((pair) => pair.split("-")[0] as StudiedLanguage))];
}

/**
 * Render the boxes into `block`, a settings block; `persist` saves the backup after a change; `copy`
 * is the block's module in the interface language and `language` that language, which names the
 * boxes.
 */
export function mountStudiedLanguages(
  block: HTMLElement,
  port: Pick<LinguaPort, "studiedLanguages" | "setStudiedLanguages" | "nativeLanguage">,
  persist: () => Promise<void>,
  pairs: readonly string[] = SHIPPED_PAIRS,
  copy: StudiedLanguagesCopy = frStudiedLanguages,
  language: InterfaceLanguage,
): StudiedLanguagesView {
  const doc = block.ownerDocument;
  const row = doc.createElement("div");
  row.className = "set-languages";
  let shownFor: string | null = null;
  let offered: StudiedLanguage[] = [];
  let boxes: { language: StudiedLanguage; box: HTMLInputElement }[] = [];

  /** A box per studied language of `native`'s pairs; the block hides when there is one. */
  function offerFor(native: string): void {
    if (native === shownFor) return;
    shownFor = native;
    offered = shippedLanguages(pairsOf(native, pairs));
    block.hidden = offered.length < 2;
    const labels = offered.map((studied) => {
      const label = doc.createElement("label");
      label.className = "set-toggle";
      const box = doc.createElement("input");
      box.type = "checkbox";
      box.dataset.language = studied;
      const name = doc.createElement("span");
      name.textContent = languageName(language, studied);
      label.append(box, name);
      box.addEventListener("change", () => void choose(studied, box.checked));
      return { language: studied, box, label };
    });
    boxes = labels.map(({ language, box }) => ({ language, box }));
    row.replaceChildren(...labels.map(({ label }) => label));
  }
  offerFor(DEFAULT_NATIVE);
  const note = doc.createElement("div");
  note.className = "set-note";
  note.textContent = copy.studiedNote;
  // The owner's word to readers (enable-lingua-spanish D6): « pour l'instant », never « bêta ».
  const offer = doc.createElement("div");
  offer.className = "set-note";
  offer.textContent = copy.severalLanguagesOffer;
  block.append(row, note, offer);

  let studied: StudiedLanguage[] = [];

  function render(): void {
    const ticked = offered.filter((language) => studied.includes(language));
    for (const { language, box } of boxes) {
      box.checked = studied.includes(language);
      // The last language the reader studies here stays: a reader always studies one.
      box.disabled = box.checked && ticked.length === 1;
    }
  }

  async function choose(language: StudiedLanguage, ticked: boolean): Promise<void> {
    const next = ticked ? [...studied.filter((l) => l !== language), language] : studied.filter((l) => l !== language);
    if (next.length === 0 || next.every((l) => !offered.includes(l))) {
      render(); // the last shipped language: refused, the box ticks again
      return;
    }
    studied = next;
    render();
    await port.setStudiedLanguages(next);
    await persist();
  }

  async function refresh(): Promise<void> {
    offerFor(await port.nativeLanguage());
    if (block.hidden) return;
    studied = await port.studiedLanguages();
    render();
  }

  return { refresh };
}
