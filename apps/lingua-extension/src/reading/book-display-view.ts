import {
  type AsyncStorageArea,
  loadReaderDisplay,
  type ReaderDisplay,
  readerDisplayOf,
  type ReaderTheme,
  type ReaderTurn,
  saveReaderDisplay,
  TEXT_SCALE_MAX,
  TEXT_SCALE_MIN,
  TEXT_SCALE_STEP,
} from "../state/storage.ts";

// How the reader shows text — its size and its theme (add-lingua-reader D10) — and how a book's
// page turns (add-lingua-page-slide). The size scales the book's text and every surface of the
// extension; the theme is the book's page and, with the Cymbra or custom colours, the surfaces'
// (add-lingua-colour-settings D8, D9); the turn is the reader page's alone. One builder,
// rendered in the two places the reader looks for it: the reader's own "Aa" panel, and Réglages,
// which shows the size and theme under Apparence › Affichage and the turn under Pages & livres ›
// Livres, beside the continuous flow (`turnContainer`). Buttons, not a slider: a tap is one step, which an e-ink screen
// redraws once. The choice is saved; every surface follows the stored value.

export interface BookDisplayOptions {
  /** Where the page-turn choice goes, when not with the size and theme (Réglages › Livres). */
  turnContainer?: HTMLElement;
}

export interface BookDisplayView {
  /** Show the stored choice (it may have changed in the other place). */
  refresh(): Promise<void>;
}

function el<K extends keyof HTMLElementTagNameMap>(
  doc: Document,
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = doc.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function button(doc: Document, className: string, text: string, label: string): HTMLButtonElement {
  const b = el(doc, "button", className, text);
  b.type = "button";
  b.setAttribute("aria-label", label);
  b.title = label;
  return b;
}

const THEMES: { theme: ReaderTheme; text: string }[] = [
  { theme: "paper", text: "Papier" },
  { theme: "dark", text: "Sombre" },
];

const TURNS: { turn: ReaderTurn; text: string }[] = [
  { turn: "instant", text: "Directe" },
  { turn: "slide", text: "Glissée" },
];

/** A row of mutually exclusive buttons, each saving its own choice. */
function segmented<T>(
  doc: Document,
  label: string,
  options: { value: T; text: string }[],
  onPick: (value: T) => void,
): { row: HTMLDivElement; buttons: { value: T; b: HTMLButtonElement }[] } {
  const group = el(doc, "div", "set-segmented");
  group.setAttribute("role", "group");
  group.setAttribute("aria-label", label);
  const buttons = options.map(({ value, text }) => {
    const b = el(doc, "button", "set-segment", text);
    b.type = "button";
    b.addEventListener("click", () => onPick(value));
    return { value, b };
  });
  group.append(...buttons.map(({ b }) => b));
  const row = el(doc, "div", "set-display-row");
  row.append(el(doc, "span", undefined, label), group);
  return { row, buttons };
}

/** Render the text size, page and page-turn choice into `container`. */
export function mountBookDisplay(
  container: HTMLElement,
  area: AsyncStorageArea,
  opts: BookDisplayOptions = {},
): BookDisplayView {
  let display: ReaderDisplay | null = null;
  // The document the view is mounted in: the reader page's, a panel's or a drawer's shadow.
  const doc = container.ownerDocument;

  const smaller = button(doc, "set-step", "A−", "Réduire le texte");
  const larger = button(doc, "set-step", "A+", "Agrandir le texte");
  const size = el(doc, "output", "set-step-value");
  const sizeRow = el(doc, "div", "set-display-row");
  const stepper = el(doc, "div", "set-stepper");
  stepper.append(smaller, size, larger);
  sizeRow.append(el(doc, "span", undefined, "Taille du texte"), stepper);

  const pages = segmented(
    doc,
    "Thème",
    THEMES.map(({ theme, text }) => ({ value: theme, text })),
    (theme) => void choose({ theme }),
  );
  // Directe by default: a slide is several frames, each of which an e-ink screen redraws.
  const turns = segmented(
    doc,
    "Tourne des pages",
    TURNS.map(({ turn, text }) => ({ value: turn, text })),
    (turn) => void choose({ turn }),
  );

  const box = el(doc, "div", "set-display");
  box.append(sizeRow, pages.row);
  if (opts.turnContainer) {
    const turnBox = el(doc, "div", "set-display");
    turnBox.append(turns.row);
    opts.turnContainer.append(turnBox);
  } else {
    box.append(turns.row);
  }
  container.append(box);

  function render(): void {
    if (!display) return;
    size.textContent = `${display.textScale} %`;
    smaller.disabled = display.textScale <= TEXT_SCALE_MIN;
    larger.disabled = display.textScale >= TEXT_SCALE_MAX;
    for (const { value, b } of pages.buttons) b.setAttribute("aria-pressed", String(display.theme === value));
    for (const { value, b } of turns.buttons) b.setAttribute("aria-pressed", String(display.turn === value));
  }

  async function choose(change: Partial<ReaderDisplay>): Promise<void> {
    display = readerDisplayOf({ ...(display ?? (await loadReaderDisplay(area))), ...change });
    render();
    await saveReaderDisplay(area, display);
  }

  smaller.addEventListener("click", () => void choose({ textScale: (display?.textScale ?? 100) - TEXT_SCALE_STEP }));
  larger.addEventListener("click", () => void choose({ textScale: (display?.textScale ?? 100) + TEXT_SCALE_STEP }));

  async function refresh(): Promise<void> {
    display = await loadReaderDisplay(area);
    render();
  }

  void refresh();
  return { refresh };
}
