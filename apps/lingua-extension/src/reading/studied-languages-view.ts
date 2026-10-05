import { languageName } from "../analyzer/language-labels.ts";
import { SHIPPED_PAIRS } from "../analyzer/pairs.ts";
import type { LinguaPort } from "../analyzer/port.ts";
import type { StudiedLanguage } from "../analyzer/types.ts";

// « Langues étudiées » (add-lingua-language-choice D2): a box per language the package ships, ticked
// when the reader studies it. Ticking appends a language, unticking removes one, and the only ticked
// language cannot be removed. The choice is the reader's profile, in their backup: `persist` saves it,
// and every context restores it and follows. A profile language the package does not ship is not
// listed, and is kept. With one shipped language there is nothing to choose: the block hides.

export interface StudiedLanguagesView {
  /** Show the stored choice (it may have changed in another context). */
  refresh(): Promise<void>;
}

/** The studied side of each shipped pair, once each, in the pairs' order. */
export function shippedLanguages(pairs: readonly string[] = SHIPPED_PAIRS): StudiedLanguage[] {
  return [...new Set(pairs.map((pair) => pair.split("-")[0] as StudiedLanguage))];
}

/** Render the boxes into `block`, a settings block; `persist` saves the backup after a change. */
export function mountStudiedLanguages(
  block: HTMLElement,
  port: Pick<LinguaPort, "studiedLanguages" | "setStudiedLanguages">,
  persist: () => Promise<void>,
  pairs: readonly string[] = SHIPPED_PAIRS,
): StudiedLanguagesView {
  const doc = block.ownerDocument;
  const offered = shippedLanguages(pairs);
  block.hidden = offered.length < 2;
  const row = doc.createElement("div");
  row.className = "set-languages";
  const boxes = offered.map((language) => {
    const label = doc.createElement("label");
    label.className = "set-toggle";
    const box = doc.createElement("input");
    box.type = "checkbox";
    box.dataset.language = language;
    const name = doc.createElement("span");
    name.textContent = languageName(language);
    label.append(box, name);
    row.append(label);
    box.addEventListener("change", () => void choose(language, box.checked));
    return { language, box };
  });
  const note = doc.createElement("div");
  note.className = "set-note";
  note.textContent =
    "Chaque page est lue dans celle de tes langues qu'elle contient. La première cochée sert aux réglages et aux statistiques par défaut.";
  // The owner's word to readers (enable-lingua-spanish D6): « pour l'instant », never « bêta ».
  const offer = doc.createElement("div");
  offer.className = "set-note";
  offer.textContent = "Plusieurs langues à la fois : gratuit pour l'instant.";
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
    if (block.hidden) return;
    studied = await port.studiedLanguages();
    render();
  }

  return { refresh };
}
